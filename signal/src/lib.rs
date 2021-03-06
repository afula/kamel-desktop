pub mod config;
pub mod signal;
pub mod storage;
pub mod util;

use crate::config::Config;
use crate::signal::{
    GroupIdentifierBytes, GroupMasterKeyBytes, PresageManager, ResolvedGroup, SignalManager,
};
use crate::storage::{JsonStorage, Storage};
use crate::util::{LazyRegex, ATTACHMENT_REGEX, URL_REGEX};

use anyhow::{anyhow, Context as _};

use log::error;
use notify_rust::Notification;
use phonenumber::{Mode, PhoneNumber};
use presage::prelude::{
    content::{ContentBody, DataMessage, Metadata, SyncMessage},
    proto::{
        data_message::{Quote, Reaction},
        sync_message::Sent,
        GroupContextV2,
    },
    AttachmentSpec, Content, GroupMasterKey, GroupSecretParams, ServiceAddress,
};
use regex_automata::Regex;
use serde::{Deserialize, Serialize};
use unicode_width::UnicodeWidthStr;
use uuid::Uuid;

use std::borrow::Cow;
use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};
use std::path::Path;

pub struct App {
    pub config: Config,
    signal_manager: Box<dyn SignalManager>,
    storage: Box<dyn Storage>,
    pub user_id: Uuid,
    pub data: AppData,
    pub should_quit: bool,
    url_regex: LazyRegex,
    attachment_regex: LazyRegex,
    display_help: bool,
}

#[derive(Default, Serialize, Deserialize)]
pub struct AppData {
    pub channels: HashMap<ChannelId, Channel>,
    pub names: HashMap<Uuid, String>,
    pub input: String,
    /// Input position in bytes (not number of chars)
    #[serde(skip)]
    pub input_cursor: usize,
    /// Input position in chars
    #[serde(skip)]
    pub input_cursor_chars: usize,

    #[serde(skip)]
    pub config: Config,
    #[serde(skip)]
    signal_manager: Option<PresageManager>,
    #[serde(skip)]
    storage: Option<JsonStorage>,
    #[serde(skip)]
    pub user_id: Uuid,
    pub should_quit: bool,
    #[serde(skip)]
    url_regex: Option<LazyRegex>,
    #[serde(skip)]
    attachment_regex: Option<LazyRegex>,
    #[serde(skip)]
    display_help: bool,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "JsonChannel")]
pub struct Channel {
    pub id: ChannelId,
    pub name: String,
    pub group_data: Option<GroupData>,
    #[serde(serialize_with = "Channel::serialize_msgs")]
    pub messages: Vec<Message>,
    pub unread_messages: usize,
    // pub selected: Option<Message>
}

/// Proxy type which allows us to apply post-deserialization conversion.
///
/// Used to migrate the schema. Change this type only in backwards-compatible way.
#[derive(Deserialize)]
pub struct JsonChannel {
    pub id: ChannelId,
    pub name: String,
    #[serde(default)]
    pub group_data: Option<GroupData>,
    #[serde(deserialize_with = "Channel::deserialize_msgs")]
    pub messages: Vec<Message>,
    #[serde(default)]
    pub unread_messages: usize,
}

impl TryFrom<JsonChannel> for Channel {
    type Error = anyhow::Error;
    fn try_from(channel: JsonChannel) -> anyhow::Result<Self> {
        let mut channel = Channel {
            id: channel.id,
            name: channel.name,
            group_data: channel.group_data,
            messages: channel.messages,
            unread_messages: channel.unread_messages,
        };

        // 1. The master key in ChannelId::Group was replaced by group identifier,
        // the former was stored in group_data.
        match (channel.id, channel.group_data.as_mut()) {
            (ChannelId::Group(id), Some(group_data)) if group_data.master_key_bytes == [0; 32] => {
                group_data.master_key_bytes = id;
                channel.id = ChannelId::from_master_key_bytes(id)?;
            }
            _ => (),
        }
        Ok(channel)
    }
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GroupData {
    #[serde(default)]
    pub master_key_bytes: GroupMasterKeyBytes,
    pub members: Vec<Uuid>,
    pub revision: u32,
}

impl Channel {
    fn user_id(&self) -> Option<Uuid> {
        match self.id {
            ChannelId::User(id) => Some(id),
            ChannelId::Group(_) => None,
        }
    }

    fn selected_message(&self) -> Option<&Message> {
        // Messages are shown in reversed order => selected is reversed
        None
    }

    fn serialize_msgs<S>(messages: &Vec<Message>, ser: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        // the messages StatefulList becomes the vec that was messages.items
        messages.serialize(ser)
    }

    fn deserialize_msgs<'de, D>(deserializer: D) -> Result<Vec<Message>, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        let tmp: Vec<Message> = serde::de::Deserialize::deserialize(deserializer)?;
        Ok(tmp)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ChannelId {
    User(Uuid),
    Group(GroupIdentifierBytes),
}

impl From<Uuid> for ChannelId {
    fn from(id: Uuid) -> Self {
        ChannelId::User(id)
    }
}

impl ChannelId {
    pub fn from_master_key_bytes(bytes: impl AsRef<[u8]>) -> anyhow::Result<Self> {
        let master_key_ar = bytes
            .as_ref()
            .try_into()
            .map_err(|_| anyhow!("invalid group master key"))?;
        let master_key = GroupMasterKey::new(master_key_ar);
        let secret_params = GroupSecretParams::derive_from_master_key(master_key);
        let group_id = secret_params.get_group_identifier();
        Ok(Self::Group(group_id))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Message {
    pub from_id: Uuid,
    pub message: Option<String>,
    pub arrived_at: u64,
    #[serde(default)]
    pub quote: Option<Box<Message>>,
    #[serde(default)]
    pub attachments: Vec<signal::Attachment>,
    #[serde(default)]
    pub reactions: Vec<(Uuid, String)>,
}

impl Message {
    pub fn new(from_id: Uuid, message: String, arrived_at: u64) -> Self {
        Self {
            from_id,
            message: Some(message),
            arrived_at,
            quote: None,
            attachments: Default::default(),
            reactions: Default::default(),
        }
    }

    pub fn from_quote(quote: Quote) -> Option<Message> {
        Some(Message {
            from_id: quote.author_uuid?.parse().ok()?,
            message: quote.text,
            arrived_at: quote.id?,
            quote: None,
            attachments: Default::default(),
            reactions: Default::default(),
        })
    }
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug)]
pub enum Event {
    Message(Content),
    Quit(Option<anyhow::Error>),
}

impl AppData {
    pub fn try_new(
        config: Config,
        signal_manager: PresageManager,
        storage: JsonStorage,
    ) -> anyhow::Result<Self> {
        let user_id = signal_manager.user_id();
        let data = storage.load_app_data(user_id, config.user.name.clone())?;
        Ok(Self {
            channels: data.channels,
            names: data.names,
            input: data.input,
            input_cursor: 0,
            input_cursor_chars: 0,
            config,
            signal_manager: Some(signal_manager),
            storage: Some(storage),
            user_id,
            should_quit: false,
            url_regex: Some(LazyRegex::new(URL_REGEX)),
            attachment_regex: Some(LazyRegex::new(ATTACHMENT_REGEX)),
            display_help: false,
        })
    }

    pub fn save(&self) -> anyhow::Result<()> {
        self.storage.as_ref().unwrap().save_app_data(&self)
    }

    pub fn name_by_id(&self, id: Uuid) -> &str {
        name_by_id(&self.names, id)
    }

    pub fn put_char(&mut self, c: char) {
        let idx = self.input_cursor;
        self.input.insert(idx, c);
        self.input_cursor += c.len_utf8();
        self.input_cursor_chars += 1;
    }

    /*    /// Tries to open the first url in the selected message.
        ///
        /// Does nothing if no message is selected and no url is contained in the message.
        fn try_open_url(&mut self) -> Option<()> {
            let channel_idx = self.channels.state.selected()?;
            let channel = &self.channels.items[channel_idx];
            let message = channel.selected_message()?;
            let re = self.url_regex.compiled();
            open_url(message, re)?;
            self.reset_message_selection();
            Some(())
        }
    */
    /// Returns Some(_) reaction if input is a reaction.
    ///
    /// Inner is None, if the reaction should be removed.
    fn take_reaction(&mut self) -> Option<Option<String>> {
        if self.input.is_empty() {
            Some(None)
        } else {
            let emoji = to_emoji(&self.input)?.to_string();
            // self.take_input();//TODO
            Some(Some(emoji))
        }
    }

    pub fn add_reaction(&mut self, channel_id: ChannelId) -> Option<()> {
        let reaction = self.take_reaction()?;
        let channel = &self.channels.get(&channel_id).unwrap();
        let message = channel.selected_message()?;
        let remove = reaction.is_none();
        let emoji = reaction.or_else(|| {
            // find emoji which should be removed
            // if no emoji found => there is no reaction from us => nothing to remove
            message.reactions.iter().find_map(|(id, emoji)| {
                if id == &self.signal_manager.as_ref().unwrap().user_id() {
                    Some(emoji.clone())
                } else {
                    None
                }
            })
        })?;

        self.signal_manager.as_ref().unwrap().send_reaction(
            channel,
            message,
            emoji.clone(),
            remove,
        );

        let channel_id = channel.id;
        let arrived_at = message.arrived_at;
        self.handle_reaction(
            channel_id,
            arrived_at,
            self.signal_manager.as_ref().unwrap().user_id(),
            emoji,
            remove,
            false,
        );

        // self.reset_unread_messages();
        // self.bubble_up_channel(channel_idx);
        // self.reset_message_selection();

        self.save().unwrap();
        Some(())
    }

    /*    fn reset_message_selection(&mut self) {
        if let Some(idx) = self.channels.state.selected() {
            let channel = &mut self.channels.items[idx];
            channel.messages.state.select(None);
            channel.messages.rendered = Default::default();
        }
    }*/

    /*    pub async fn on_message(&mut self, content: Content) -> anyhow::Result<()> {
        log::info!("incoming: {:?}", content);

        let user_id = self.user_id;

        let (channel_idx, message) = match (content.metadata, content.body) {
            // Private note message
            (
                _,
                ContentBody::SynchronizeMessage(SyncMessage {
                                                    sent:
                                                    Some(Sent {
                                                             destination_uuid: Some(destination_uuid),
                                                             timestamp: Some(timestamp),
                                                             message:
                                                             Some(DataMessage {
                                                                      body: Some(text), ..
                                                                  }),
                                                             ..
                                                         }),
                                                    ..
                                                }),
            ) if destination_uuid.parse() == Ok(user_id) => {
                let channel_idx = self.ensure_own_channel_exists();
                let message = Message::new(user_id, text, timestamp);
                (channel_idx, message)
            }
            // Direct/group message by us from a different device
            (
                Metadata {
                    sender:
                    ServiceAddress {
                        uuid: Some(sender_uuid),
                        ..
                    },
                    ..
                },
                ContentBody::SynchronizeMessage(SyncMessage {
                                                    sent:
                                                    Some(Sent {
                                                             destination_e164,
                                                             destination_uuid,
                                                             timestamp: Some(timestamp),
                                                             message:
                                                             Some(DataMessage {
                                                                      body: Some(text),
                                                                      group_v2,
                                                                      quote,
                                                                      ..
                                                                  }),
                                                             ..
                                                         }),
                                                    ..
                                                }),
            ) if sender_uuid == user_id => {
                let channel_idx = if let Some(GroupContextV2 {
                                                  master_key: Some(master_key),
                                                  revision: Some(revision),
                                                  ..
                                              }) = group_v2
                {
                    // message to a group
                    let master_key = master_key
                        .try_into()
                        .map_err(|_| anyhow!("invalid master key"))?;
                    self.ensure_group_channel_exists(master_key, revision)
                        .await
                        .context("failed to create group channel")?
                } else if let (Some(destination_uuid), Some(destination_e164)) = (
                    destination_uuid.and_then(|s| s.parse().ok()),
                    destination_e164,
                ) {
                    // message to a contact
                    self.ensure_contact_channel_exists(destination_uuid, &destination_e164)
                        .await
                } else {
                    return Ok(());
                };

                let quote = quote.and_then(Message::from_quote).map(Box::new);
                let message = Message {
                    quote,
                    ..Message::new(user_id, text, timestamp)
                };
                (channel_idx, message)
            }
            // Incoming direct/group message
            (
                Metadata {
                    sender:
                    ServiceAddress {
                        uuid: Some(uuid),
                        phonenumber: Some(phone_number),
                        ..
                    },
                    ..
                },
                ContentBody::DataMessage(DataMessage {
                                             body: Some(text),
                                             group_v2,
                                             timestamp: Some(timestamp),
                                             profile_key: Some(profile_key),
                                             quote,
                                             ..
                                         }),
            ) => {
                let (channel_idx, from) = if let Some(GroupContextV2 {
                                                          master_key: Some(master_key),
                                                          revision: Some(revision),
                                                          ..
                                                      }) = group_v2
                {
                    // incoming group message
                    let master_key = master_key
                        .try_into()
                        .map_err(|_| anyhow!("invalid group master key"))?;
                    let channel_idx = self
                        .ensure_group_channel_exists(master_key, revision)
                        .await
                        .context("failed to create group channel")?;
                    let from = self
                        .ensure_user_is_known(uuid, profile_key, phone_number)
                        .await
                        .to_string();

                    (channel_idx, from)
                } else {
                    // incoming direct message
                    let name = self
                        .ensure_user_is_known(uuid, profile_key, phone_number)
                        .await
                        .to_string();
                    let channel_idx = self.ensure_contact_channel_exists(uuid, &name).await;
                    let from = self.channels.items[channel_idx].name.clone();

                    (channel_idx, from)
                };

                self.notify(&from, &text);

                let quote = quote.and_then(Message::from_quote).map(Box::new);
                let message = Message {
                    quote,
                    ..Message::new(uuid, text, timestamp)
                };
                (channel_idx, message)
            }
            // reactions
            (
                Metadata {
                    sender:
                    ServiceAddress {
                        uuid: Some(sender_uuid),
                        ..
                    },
                    ..
                },
                ContentBody::SynchronizeMessage(SyncMessage {
                                                    sent:
                                                    Some(Sent {
                                                             destination_uuid,
                                                             message:
                                                             Some(DataMessage {
                                                                      body: None,
                                                                      group_v2,
                                                                      reaction:
                                                                      Some(Reaction {
                                                                               emoji: Some(emoji),
                                                                               remove,
                                                                               target_author_uuid: Some(target_author_uuid),
                                                                               target_sent_timestamp: Some(target_sent_timestamp),
                                                                               ..
                                                                           }),
                                                                      ..
                                                                  }),
                                                             ..
                                                         }),
                                                    ..
                                                }),
            ) => {
                let channel_id = if let Some(GroupContextV2 {
                                                 master_key: Some(master_key),
                                                 ..
                                             }) = group_v2
                {
                    ChannelId::from_master_key_bytes(master_key)?
                } else if let Some(uuid) = destination_uuid {
                    ChannelId::User(uuid.parse()?)
                } else {
                    ChannelId::User(target_author_uuid.parse()?)
                };

                self.handle_reaction(
                    channel_id,
                    target_sent_timestamp,
                    sender_uuid,
                    emoji,
                    remove.unwrap_or(false),
                    true,
                );
                return Ok(());
            }
            (
                Metadata {
                    sender:
                    ServiceAddress {
                        uuid: Some(sender_uuid),
                        ..
                    },
                    ..
                },
                ContentBody::DataMessage(DataMessage {
                                             body: None,
                                             group_v2,
                                             reaction:
                                             Some(Reaction {
                                                      emoji: Some(emoji),
                                                      remove,
                                                      target_sent_timestamp: Some(target_sent_timestamp),
                                                      target_author_uuid: Some(target_author_uuid),
                                                      ..
                                                  }),
                                             ..
                                         }),
            ) => {
                let channel_id = if let Some(GroupContextV2 {
                                                 master_key: Some(master_key),
                                                 ..
                                             }) = group_v2
                {
                    ChannelId::from_master_key_bytes(master_key)?
                } else if sender_uuid == self.user_id {
                    // reaction from us => target author is the user channel
                    ChannelId::User(target_author_uuid.parse()?)
                } else {
                    // reaction is from somebody else => they are the user channel
                    ChannelId::User(sender_uuid)
                };

                self.handle_reaction(
                    channel_id,
                    target_sent_timestamp,
                    sender_uuid,
                    emoji,
                    remove.unwrap_or(false),
                    true,
                );
                return Ok(());
            }
            _ => return Ok(()),
        };

        self.add_message_to_channel(channel_idx, message);

        Ok(())
    }*/

    pub fn handle_reaction(
        &mut self,
        channel_id: ChannelId,
        target_sent_timestamp: u64,
        sender_uuid: Uuid,
        emoji: String,
        remove: bool,
        notify: bool,
    ) -> Option<()> {
        let mut channel = self.channels.get_mut(&channel_id).unwrap();
        let message = channel
            .messages
            .iter_mut()
            .find(|m| m.arrived_at == target_sent_timestamp)?;
        let reaction_idx = message
            .reactions
            .iter()
            .position(|(from_id, _)| from_id == &sender_uuid);
        let is_added = if let Some(idx) = reaction_idx {
            if remove {
                message.reactions.swap_remove(idx);
                false
            } else {
                message.reactions[idx].1 = emoji.clone();
                true
            }
        } else {
            message.reactions.push((sender_uuid, emoji.clone()));
            true
        };

        if is_added && channel_id != ChannelId::User(self.user_id) {
            // Notification
            let sender_name = name_by_id(&self.names, sender_uuid);
            let summary = if let ChannelId::Group(_) = channel.id {
                Cow::from(format!("{} in {}", sender_name, channel.name))
            } else {
                Cow::from(sender_name)
            };
            let mut notification = format!("{} reacted {}", summary, emoji);
            if let Some(text) = message.message.as_ref() {
                notification.push_str(" to: ");
                notification.push_str(text);
            }
            if notify {
                self.notify(&summary, &notification);
            }

            // self.touch_channel(channel_idx);
        } else {
            self.save().unwrap();
        }

        Some(())
    }

    pub async fn ensure_group_channel_exists(
        &mut self,
        master_key: GroupMasterKeyBytes,
        revision: u32,
    ) -> anyhow::Result<ChannelId> {
        let id = ChannelId::from_master_key_bytes(master_key)?;

        if let Some(channel) = self.channels.get_mut(&id) {
            let is_stale = match channel.group_data.as_ref() {
                Some(group_data) => group_data.revision != revision,
                None => true,
            };
            if is_stale {
                let ResolvedGroup {
                    name,
                    group_data,
                    profile_keys,
                } = self
                    .signal_manager
                    .clone()
                    .unwrap()
                    .resolve_group(master_key)
                    .await?;

                self.try_ensure_users_are_known(
                    group_data
                        .members
                        .iter()
                        .copied()
                        .zip(profile_keys.into_iter()),
                )
                .await;

                let channel = self.channels.get_mut(&id).unwrap();
                channel.name = name;
                channel.group_data = Some(group_data);
            }
        } else {
            let ResolvedGroup {
                name,
                group_data,
                profile_keys,
            } = self
                .signal_manager
                .clone()
                .unwrap()
                .resolve_group(master_key)
                .await?;

            self.try_ensure_users_are_known(
                group_data
                    .members
                    .iter()
                    .copied()
                    .zip(profile_keys.into_iter()),
            )
            .await;

            self.channels.insert(
                id,
                Channel {
                    id,
                    name,
                    group_data: Some(group_data),
                    messages: Vec::new(),
                    unread_messages: 0,
                },
            );
        }
        Ok(id)
    }

    pub async fn ensure_user_is_known(
        &mut self,
        uuid: Uuid,
        profile_key: Vec<u8>,
        phone_number: PhoneNumber,
    ) -> &str {
        if self
            .try_ensure_user_is_known(uuid, profile_key)
            .await
            .is_none()
        {
            let phone_number_name = phone_number.format().mode(Mode::E164).to_string();
            self.names.insert(uuid, phone_number_name);
        }
        self.names.get(&uuid).unwrap()
    }

    async fn try_ensure_user_is_known(&mut self, uuid: Uuid, profile_key: Vec<u8>) -> Option<&str> {
        let is_phone_number_or_unknown = self
            .names
            .get(&uuid)
            .map(util::is_phone_number)
            .unwrap_or(true);
        if is_phone_number_or_unknown {
            let name = match profile_key.try_into() {
                Ok(key) => {
                    self.signal_manager
                        .as_ref()
                        .unwrap()
                        .contact_name(uuid, key)
                        .await
                }
                Err(_) => None,
            };
            self.names.insert(uuid, name?);
        }
        self.names.get(&uuid).map(|s| s.as_str())
    }

    async fn try_ensure_users_are_known(
        &mut self,
        users_with_keys: impl Iterator<Item = (Uuid, Vec<u8>)>,
    ) {
        // TODO: Run in parallel
        for (uuid, profile_key) in users_with_keys {
            self.try_ensure_user_is_known(uuid, profile_key).await;
        }
    }

    pub fn ensure_own_channel_exists(&mut self) -> ChannelId {
        let user_id = self.user_id;
        if let Some((channel_id, _)) = self
            .channels
            .iter_mut()
            .find(|channel| channel.1.user_id() == Some(user_id))
        {
            channel_id.to_owned()
        } else {
            self.channels.insert(
                user_id.into(),
                Channel {
                    id: user_id.into(),
                    name: self.config.user.name.clone(),
                    group_data: None,
                    messages: Vec::new(),
                    unread_messages: 0,
                },
            );
            user_id.into()
        }
    }

    pub async fn ensure_contact_channel_exists(&mut self, uuid: Uuid, name: &str) -> ChannelId {
        let channel_id = uuid.into();
        if let Some(channel) = self.channels.get_mut(&channel_id) {
            if let Some(name) = self.names.get(&uuid) {
                if &channel.name != name {
                    channel.name = name.clone();
                }
            }
            channel_id
        } else {
            self.channels.insert(
                channel_id,
                Channel {
                    id: uuid.into(),
                    name: name.to_string(),
                    group_data: None,
                    messages: Vec::new(),
                    unread_messages: 0,
                },
            );
            channel_id
        }
    }

    pub fn add_message_to_channel(&mut self, channel_id: ChannelId, message: Message) {
        let channel = &mut self.channels.get_mut(&channel_id).unwrap();

        channel.messages.push(message);
        /*        if let Some(idx) = channel.messages.state.selected() {
            // keep selection on the old message
            channel.messages.state.select(Some(idx + 1));
        }*/

        // self.touch_channel(channel_idx);
    }

    /*    fn touch_channel(&mut self, channel_idx: usize) {
        if self.channels.state.selected() != Some(channel_idx) {
            self.channels.items[channel_idx].unread_messages += 1;
        } else {
            self.reset_unread_messages();
        }

        self.bubble_up_channel(channel_idx);
        self.save().unwrap();
    }*/

    /*    fn bubble_up_channel(&mut self, channel_idx: usize) {
        // bubble up channel to the beginning of the list
        let channels = &mut self.channels;
        for (prev, next) in (0..channel_idx).zip(1..channel_idx + 1).rev() {
            channels.items.swap(prev, next);
        }
        match channels.state.selected() {
            Some(selected_idx) if selected_idx == channel_idx => channels.state.select(Some(0)),
            Some(selected_idx) if selected_idx < channel_idx => {
                channels.state.select(Some(selected_idx + 1));
            }
            _ => {}
        };
    }*/

    pub fn notify(&self, summary: &str, text: &str) {
        if let Err(e) = Notification::new().summary(summary).body(text).show() {
            error!("failed to send notification: {}", e);
        }
    }

    fn extract_attachments(&mut self, input: &str) -> (String, Vec<(AttachmentSpec, Vec<u8>)>) {
        let mut offset = 0;
        let mut clean_input = String::new();

        let re = self.attachment_regex.as_mut().unwrap().compiled();
        let attachments = re.find_iter(input.as_bytes()).filter_map(|(start, end)| {
            let path_str = &input[start..end].strip_prefix("file://")?;

            let path = Path::new(path_str);
            let contents = std::fs::read(path).ok()?;

            clean_input.push_str(input[offset..start].trim_end_matches(""));
            offset = end;

            let content_type = mime_guess::from_path(path)
                .first()
                .map(|mime| mime.essence_str().to_string())
                .unwrap_or_default();
            let spec = AttachmentSpec {
                content_type,
                length: contents.len(),
                file_name: Path::new(path)
                    .file_name()
                    .map(|f| f.to_string_lossy().into()),
                preview: None,
                voice_note: None,
                borderless: None,
                width: None,
                height: None,
                caption: None,
                blur_hash: None,
            };
            Some((spec, contents))
        });

        let attachments = attachments.collect();
        clean_input.push_str(&input[offset..]);
        let clean_input = clean_input.trim().to_string();

        (clean_input, attachments)
    }
}

pub fn name_by_id(names: &HashMap<Uuid, String>, id: Uuid) -> &str {
    names.get(&id).map(|s| s.as_ref()).unwrap_or("Unknown Name")
}

/// Returns an emoji string if `s` is an emoji or if `s` is a GitHub emoji shortcode.
fn to_emoji(s: &str) -> Option<&str> {
    let s = s.trim();
    if emoji::lookup_by_glyph::lookup(s).is_some() {
        Some(s)
    } else {
        let s = s.strip_prefix(':')?.strip_suffix(':')?;
        let emoji = gh_emoji::get(s)?;
        Some(emoji)
    }
}

fn open_url(message: &Message, url_regex: &Regex) -> Option<()> {
    let text = message.message.as_ref()?;
    let (start, end) = url_regex.find(text.as_bytes())?;
    let url = &text[start..end];
    if let Err(e) = opener::open(url) {
        error!("failed to open {}: {}", url, e);
    }
    Some(())
}

/*#[cfg(test)]
mod tests {
    use super::*;

    use crate::config::User;
    use crate::signal::test::SignalManagerMock;
    use crate::storage::test::InMemoryStorage;

    use std::cell::RefCell;
    use std::rc::Rc;

    fn test_app() -> (App, Rc<RefCell<Vec<Message>>>) {
        let signal_manager = SignalManagerMock::new();
        let sent_messages = signal_manager.sent_messages.clone();

        let mut app = App::try_new(
            Config::with_user(User {
                name: "Tyler Durden".to_string(),
                phone_number: "+0000000000".to_string(),
            }),
            Box::new(signal_manager),
            Box::new(InMemoryStorage::new()),
        )
            .unwrap();

        appself.channels.items.push(Channel {
            id: ChannelId::User(Uuid::new_v4()),
            name: "test".to_string(),
            group_data: Some(GroupData {
                master_key_bytes: GroupMasterKeyBytes::default(),
                members: vec![app.user_id],
                revision: 1,
            }),
            messages: StatefulList::with_items(vec![Message {
                from_id: app.user_id,
                message: Some("First message".to_string()),
                arrived_at: 0,
                quote: Default::default(),
                attachments: Default::default(),
                reactions: Default::default(),
            }]),
            unread_messages: 1,
        });
        appself.channels.state.select(Some(0));

        (app, sent_messages)
    }

    #[test]
    fn test_send_input() {
        let (mut app, sent_messages) = test_app();
        let input = "Hello, World!";
        for c in input.chars() {
            app.put_char(c);
        }
        app.send_input(0).unwrap();

        let sent = sent_messages.borrow();
        assert_eq!(sent.len(), 1);
        assert_eq!(sent[0].message.as_ref().unwrap(), input);

        assert_eq!(appself.channels.items[0].unread_messages, 0);

        assert_eq!(appself.input, "");
        assert_eq!(appself.input_cursor, 0);
        assert_eq!(appself.input_cursor_chars, 0);
    }

    #[test]
    fn test_send_input_with_emoji() {
        let (mut app, sent_messages) = test_app();
        let input = "????";
        for c in input.chars() {
            app.put_char(c);
        }
        assert_eq!(appself.input_cursor, 4);
        assert_eq!(appself.input_cursor_chars, 1);

        app.send_input(0).unwrap();

        let sent = sent_messages.borrow();
        assert_eq!(sent.len(), 1);
        assert_eq!(sent[0].message.as_ref().unwrap(), input);

        assert_eq!(appself.input, "");
        assert_eq!(appself.input_cursor, 0);
        assert_eq!(appself.input_cursor_chars, 0);
    }

    #[test]
    fn test_send_input_with_emoji_codepoint() {
        let (mut app, sent_messages) = test_app();
        let input = ":thumbsup:";
        for c in input.chars() {
            app.put_char(c);
        }

        app.send_input(0).unwrap();

        let sent = sent_messages.borrow();
        assert_eq!(sent.len(), 1);
        assert_eq!(sent[0].message.as_ref().unwrap(), "????");
    }

    #[test]
    fn test_add_reaction_with_emoji() {
        let (mut app, _sent_messages) = test_app();

        appself.channels.items[0].messages.state.select(Some(0));

        app.put_char('????');
        app.add_reaction(0);

        let reactions = &appself.channels.items[0].messages.items[0].reactions;
        assert_eq!(reactions.len(), 1);
        assert_eq!(reactions[0], (app.user_id, "????".to_string()));
    }

    #[test]
    fn test_add_reaction_with_emoji_codepoint() {
        let (mut app, _sent_messages) = test_app();

        appself.channels.items[0].messages.state.select(Some(0));

        for c in ":thumbsup:".chars() {
            app.put_char(c);
        }
        app.add_reaction(0);

        let reactions = &appself.channels.items[0].messages.items[0].reactions;
        assert_eq!(reactions.len(), 1);
        assert_eq!(reactions[0], (app.user_id, "????".to_string()));
    }

    #[test]
    fn test_remove_reaction() {
        let (mut app, _sent_messages) = test_app();

        appself.channels.items[0].messages.state.select(Some(0));
        let reactions = &mut appself.channels.items[0].messages.items[0].reactions;
        reactions.push((app.user_id, "????".to_string()));

        app.add_reaction(0);

        let reactions = &appself.channels.items[0].messages.items[0].reactions;
        assert!(reactions.is_empty());
    }

    #[test]
    fn test_add_invalid_reaction() {
        let (mut app, _sent_messages) = test_app();

        appself.channels.items[0].messages.state.select(Some(0));

        for c in ":thumbsup".chars() {
            app.put_char(c);
        }
        app.add_reaction(0);

        assert_eq!(appself.input, ":thumbsup");
        let reactions = &appself.channels.items[0].messages.items[0].reactions;
        assert!(reactions.is_empty());
    }
}*/
