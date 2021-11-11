// pub mod config;
// pub mod signal_ll;
// pub mod storage_ll;
// pub mod util;
//
// use crate::config::Config;
// use crate::signal_ll::{
//     GroupIdentifierBytes, GroupMasterKeyBytes, PresageManager, ResolvedGroup, SignalManager,
// };
// use crate::storage_ll::{JsonStorage, Storage};
// use crate::util::{LazyRegex, ATTACHMENT_REGEX, URL_REGEX};
// use anyhow::{anyhow, Context as _};
// use crossterm::event::{KeyCode, KeyEvent, MouseEvent};
// use druid::{
//     im::{HashMap, Vector},
//     Data, Lens,
// };
// use druid_enums::Matcher;
// use log::error;
// use notify_rust::Notification;
// use phonenumber::{Mode, PhoneNumber};
// use presage::prelude::{
//     content::{ContentBody, DataMessage, Metadata, SyncMessage},
//     proto::{
//         data_message::{Quote, Reaction},
//         sync_message::Sent,
//         GroupContextV2,
//     },
//     AttachmentSpec, Content, GroupMasterKey, GroupSecretParams, ServiceAddress,
// };
// use regex_automata::Regex;
// use serde::{Deserialize, Serialize};
// use std::borrow::Cow;
// use std::convert::{TryFrom, TryInto};
// use std::path::Path;
// use std::str::FromStr;
// use unicode_width::UnicodeWidthStr;
// use uuid::Uuid;
//
// #[derive(Debug, Data, Clone, PartialEq, Eq)]
// pub enum Platform {
//     Signal,
//     Matrix,
// }
//
// impl Default for Platform {
//     fn default() -> Self {
//         Self::Signal
//     }
// }
// /*#[derive(Serialize, Default, Deserialize, Data, Lens, Clone)]
// pub struct AppData111 {
//     #[serde(serialize_with = "AppData::serialize_channels")]
//     #[serde(deserialize_with = "AppData::deserialize_channels")]
//     pub channels: HashMap<ChannelId, Channel>,
//     // #[data(ignore)]
//     #[serde(serialize_with = "AppData::serialize_names")]
//     #[serde(deserialize_with = "AppData::deserialize_names")]
//     pub names: HashMap<Uuid, String>,
//     pub input: String,
//
//     #[serde(skip)]
//     #[data(ignore)]
//     pub input_cursor: usize,
//     /// Input position in chars
//     #[data(ignore)]
//     #[serde(skip)]
//     pub input_cursor_chars: usize,
//     #[serde(skip)]
//     pub current_channel: Option<ChannelId>,
//     #[serde(skip)]
//     pub current_platform: Option<Platform>,
//     #[serde(skip)]
//     pub user_id: String,
// }*/
//
// #[derive(Serialize, Default, Deserialize,Clone)]
// pub struct AppData {
//     // pub data: AppData111,
//
//     #[serde(serialize_with = "AppData::serialize_channels")]
//     #[serde(deserialize_with = "AppData::deserialize_channels")]
//     pub channels: HashMap<ChannelId, Channel>,
//     // #[data(ignore)]
//     #[serde(serialize_with = "AppData::serialize_names")]
//     #[serde(deserialize_with = "AppData::deserialize_names")]
//     pub names: HashMap<Uuid, String>,
//     pub input: String,
//     // *****************************************************//
//     #[data(ignore)]
//     pub config: Option<Config>,
//     #[data(ignore)]
//     #[serde(skip)]
//     signal_manager: Option<PresageManager>,
//     #[data(ignore)]
//     #[serde(skip)]
//     storage: Option<JsonStorage>,
//     #[data(ignore)]
//     #[serde(skip)]
//     pub should_quit: bool,
//     #[data(ignore)]
//     #[serde(skip)]
//     pub url_regex: Option<LazyRegex>,
//     #[data(ignore)]
//     #[serde(skip)]
//     pub attachment_regex: Option<LazyRegex>,
// }
//
// impl AppData {
//     fn serialize_channels<S>(
//         channels: &HashMap<ChannelId, Channel>,
//         ser: S,
//     ) -> Result<S::Ok, S::Error>
//     where
//         S: serde::ser::Serializer,
//     {
//         // the messages StatefulList becomes the vec that was messages.items
//         let mut channels_clone = std::collections::HashMap::with_capacity(channels.len());
//         for (k, v) in channels.iter() {
//             channels_clone.insert(k, v);
//         }
//         channels_clone.serialize(ser)
//     }
//
//     fn deserialize_channels<'de, D>(
//         deserializer: D,
//     ) -> Result<HashMap<ChannelId, Channel>, D::Error>
//     where
//         D: serde::de::Deserializer<'de>,
//     {
//         let tmp: std::collections::HashMap<ChannelId, Channel> =
//             serde::de::Deserialize::deserialize(deserializer)?;
//         Ok(HashMap::from(tmp))
//     }
//     fn serialize_names<S>(names: &HashMap<Uuid, String>, ser: S) -> Result<S::Ok, S::Error>
//     where
//         S: serde::ser::Serializer,
//     {
//         // the messages StatefulList becomes the vec that was messages.items
//         let mut names_clone = std::collections::HashMap::with_capacity(names.len());
//         for (k, v) in names.iter() {
//             names_clone.insert(k, v);
//         }
//         names_clone.serialize(ser)
//     }
//
//     fn deserialize_names<'de, D>(deserializer: D) -> Result<HashMap<Uuid, String>, D::Error>
//     where
//         D: serde::de::Deserializer<'de>,
//     {
//         let tmp: std::collections::HashMap<Uuid, String> =
//             serde::de::Deserialize::deserialize(deserializer)?;
//         Ok(HashMap::from(tmp))
//     }
// }
// #[derive(Debug, Serialize, Deserialize, Data, Lens, Clone)]
// #[serde(try_from = "JsonChannel")]
// pub struct Channel {
//     pub id: ChannelId,
//     pub name: String,
//     pub group_data: Option<GroupData>,
//     #[serde(serialize_with = "Channel::serialize_msgs")]
//     pub messages: Vector<Message>,
//     pub unread_messages: usize,
//     #[serde(skip)]
//     pub current_message: Option<Message>,
// }
//
// /// Proxy type which allows us to apply post-deserialization conversion.
// ///
// /// Used to migrate the schema. Change this type only in backwards-compatible way.
// #[derive(Deserialize)]
// pub struct JsonChannel {
//     pub id: ChannelId,
//     pub name: String,
//     #[serde(default)]
//     pub group_data: Option<GroupData>,
//     #[serde(deserialize_with = "Channel::deserialize_msgs")]
//     pub messages: Vector<Message>,
//     #[serde(default)]
//     pub unread_messages: usize,
// }
//
// impl TryFrom<JsonChannel> for Channel {
//     type Error = anyhow::Error;
//     fn try_from(channel: JsonChannel) -> anyhow::Result<Self> {
//         let mut channel = Channel {
//             id: channel.id,
//             name: channel.name,
//             group_data: channel.group_data,
//             messages: Vector::from(channel.messages),
//             unread_messages: channel.unread_messages,
//             current_message: None,
//         };
//
//         // 1. The master key in ChannelId::Group was replaced by group identifier,
//         // the former was stored in group_data.
//         match (channel.id.clone(), channel.group_data.as_mut()) {
//             (ChannelId::Group(id), Some(group_data)) if group_data.master_key_bytes == [0; 32] => {
//                 group_data.master_key_bytes = id;
//                 channel.id = ChannelId::from_master_key_bytes(id)?;
//             }
//             _ => (),
//         }
//         Ok(channel)
//     }
// }
//
// #[derive(Debug, Serialize, Deserialize, Data, Lens, Clone)]
// pub struct GroupData {
//     #[serde(default)]
//     pub master_key_bytes: GroupMasterKeyBytes,
//     #[data(ignore)]
//     pub members: Vec<Uuid>,
//     pub revision: u32,
// }
//
// impl Channel {
//     fn user_id(&self) -> Option<String> {
//         match self.id.to_owned() {
//             ChannelId::User(id) => Some(id),
//             ChannelId::Group(_) => None,
//         }
//     }
//
//     fn selected_message(&self) -> Option<&Message> {
//         // Messages are shown in reversed order => selected is reversed
//         self.current_message.as_ref()
//         // .and_then(|idx| self.messages.items.len().checked_sub(idx + 1))
//         // .and_then(|idx| self.messages.get(idx))
//     }
//
//     fn serialize_msgs<S>(messages: &Vector<Message>, ser: S) -> Result<S::Ok, S::Error>
//     where
//         S: serde::ser::Serializer,
//     {
//         // the messages StatefulList becomes the vec that was messages.items
//         let mut message_clone = Vec::with_capacity(messages.len());
//         for m in messages.iter() {
//             message_clone.push(m);
//         }
//         message_clone.serialize(ser)
//     }
//
//     fn deserialize_msgs<'de, D>(deserializer: D) -> Result<Vector<Message>, D::Error>
//     where
//         D: serde::de::Deserializer<'de>,
//     {
//         let tmp: Vec<Message> = serde::de::Deserialize::deserialize(deserializer)?;
//         Ok(Vector::from(tmp))
//     }
// }
//
// #[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Data)]
// pub enum ChannelId {
//     User(String),
//     Group(GroupIdentifierBytes),
// }
//
// impl From<Uuid> for ChannelId {
//     fn from(id: Uuid) -> Self {
//         ChannelId::User(id.to_string())
//     }
// }
//
// impl ChannelId {
//     pub fn from_master_key_bytes(bytes: impl AsRef<[u8]>) -> anyhow::Result<Self> {
//         let master_key_ar = bytes
//             .as_ref()
//             .try_into()
//             .map_err(|_| anyhow!("invalid group master key"))?;
//         let master_key = GroupMasterKey::new(master_key_ar);
//         let secret_params = GroupSecretParams::derive_from_master_key(master_key);
//         let group_id = secret_params.get_group_identifier();
//         Ok(Self::Group(group_id))
//     }
// }
//
// #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Data, Lens)]
// pub struct Message {
//     // #[data(ignore)]
//     // #[serde(skip)]
//     // pub from_id: Uuid,
//     pub from_id: String,
//     pub message: Option<String>,
//     pub arrived_at: u64,
//     #[serde(default)]
//     #[data(ignore)]
//     pub quote: Option<Box<Message>>,
//     // #[serde(default)]
//     #[serde(serialize_with = "Message::serialize_attach")]
//     #[serde(deserialize_with = "Message::deserialize_attach")]
//     pub attachments: Vector<signal_ll::Attachment>,
//     #[serde(default)]
//     #[data(ignore)]
//     pub reactions: Vec<(Uuid, String)>,
// }
//
// impl Message {
//     pub fn new(from_id: Uuid, message: String, arrived_at: u64) -> Self {
//         Self {
//             from_id: from_id.to_string(),
//             message: Some(message),
//             arrived_at,
//             quote: None,
//             attachments: Default::default(),
//             reactions: Default::default(),
//         }
//     }
//
//     pub fn from_quote(quote: Quote) -> Option<Message> {
//         Some(Message {
//             from_id: quote.author_uuid?.parse().ok()?,
//             message: quote.text,
//             arrived_at: quote.id?,
//             quote: None,
//             attachments: Default::default(),
//             reactions: Default::default(),
//         })
//     }
//
//     fn serialize_attach<S>(messages: &Vector<signal_ll::Attachment>, ser: S) -> Result<S::Ok, S::Error>
//     where
//         S: serde::ser::Serializer,
//     {
//         // the messages StatefulList becomes the vec that was messages.items
//         let mut message_clone = Vec::with_capacity(messages.len());
//         for m in messages.iter() {
//             message_clone.push(m);
//         }
//         message_clone.serialize(ser)
//     }
//
//     fn deserialize_attach<'de, D>(deserializer: D) -> Result<Vector<signal_ll::Attachment>, D::Error>
//     where
//         D: serde::de::Deserializer<'de>,
//     {
//         let tmp: Vec<signal_ll::Attachment> = serde::de::Deserialize::deserialize(deserializer)?;
//         Ok(Vector::from(tmp))
//     }
// }
//
// #[allow(clippy::large_enum_variant)]
// #[derive(Debug)]
// pub enum Event {
//     Input(KeyEvent),
//     Message(Content),
//     Quit(Option<anyhow::Error>),
// }
//
// impl AppData {
//     pub fn try_new(
//         config: Config,
//         signal_manager: PresageManager,
//         storage: JsonStorage,
//     ) -> anyhow::Result<Self> {
//         let user_id = signal_manager.user_id();
//         let data = storage.load_app_data(user_id, config.user.name.clone())?;
//
//         Ok(Self {
//             channels: data.channels,
//             names: data.names,
//             input: data.input,
//             config: Some(config),
//             signal_manager: Some(signal_manager),
//             storage: Some(storage),
//             should_quit: false,
//             url_regex: Some(LazyRegex::new(URL_REGEX)),
//             attachment_regex: Some(LazyRegex::new(ATTACHMENT_REGEX)),
//         })
//     }
//
//     pub fn save(&self) -> anyhow::Result<()> {
//         let storage = self.storage.as_ref().unwrap().clone();
//         storage.save_app_data(&self)
//     }
//
//     pub fn name_by_id(&self, id: Uuid) -> &str {
//         name_by_id(&self.names, id)
//     }
//
//     pub fn put_char(&mut self, c: char) {
//         let idx = self.input_cursor;
//         self.input.insert(idx, c);
//         self.input_cursor += c.len_utf8();
//         self.input_cursor_chars += 1;
//     }
//
//     /// Tries to open the first url in the selected message.
//     ///
//     /// Does nothing if no message is selected and no url is contained in the message.
//     fn try_open_url(&mut self) -> Option<()> {
//         let channel_idx = self.current_channel.as_ref().unwrap();
//         let channel = &self.channels.get(&channel_idx).unwrap();
//         let message = channel.selected_message()?;
//         let mut url_regex = self.url_regex.as_ref().unwrap().clone();
//         let re = url_regex.compiled();
//         open_url(message, re)?;
//         // self.reset_message_selection();
//         Some(())
//     }
//
//     /// Returns Some(_) reaction if input is a reaction.
//     ///
//     /// Inner is None, if the reaction should be removed.
//     fn take_reaction(&mut self) -> Option<Option<String>> {
//         if self.input.is_empty() {
//             Some(None)
//         } else {
//             let emoji = to_emoji(&self.input)?.to_string();
//             self.take_input();
//             Some(Some(emoji))
//         }
//     }
//
//     pub fn add_reaction(&mut self, channel_idx: ChannelId) -> Option<()> {
//         let reaction = self.take_reaction()?;
//         let channel = &self.channels.get(&channel_idx).unwrap();
//         let message = channel.selected_message()?;
//         let remove = reaction.is_none();
//         let emoji = reaction.or_else(|| {
//             // find emoji which should be removed
//             // if no emoji found => there is no reaction from us => nothing to remove
//             message.reactions.iter().find_map(|(id, emoji)| {
//                 if id == &self.signal_manager.as_ref().unwrap().user_id() {
//                     Some(emoji.clone())
//                 } else {
//                     None
//                 }
//             })
//         })?;
//
//         self.signal_manager.as_ref().unwrap().send_reaction(
//             channel,
//             message,
//             emoji.clone(),
//             remove,
//         );
//
//         let channel_id = channel.id.clone();
//         let arrived_at = message.arrived_at;
//         self.handle_reaction(
//             channel_id,
//             arrived_at,
//             self.signal_manager.as_ref().unwrap().user_id(),
//             emoji,
//             remove,
//             false,
//         );
//
//         /*        self.reset_unread_messages();
//         self.bubble_up_channel(channel_idx);*/
//         // self.reset_message_selection();
//
//         self.save().unwrap();
//         Some(())
//     }
//
//     /*    fn reset_message_selection(&mut self) {
//         if let Some(idx) = self.channels.state.selected() {
//             let channel = &mut self.channels.items[idx];
//             channel.messages.state.select(None);
//             channel.messages.rendered = Default::default();
//         }
//     }*/
//
//     fn take_input(&mut self) -> String {
//         self.input_cursor = 0;
//         self.input_cursor_chars = 0;
//         std::mem::take(&mut self.input)
//     }
//
//     fn send_input(&mut self) -> anyhow::Result<()> {
//         let input = self.take_input();
//         let (input, attachments) = self.extract_attachments(&input);
//         let channel_id = self.current_channel.as_ref().unwrap();
//         let channel = self.channels.get_mut(&channel_id).unwrap();
//         let quote = channel.selected_message();
//         let sent_message =
//             self.signal_manager
//                 .as_ref()
//                 .unwrap()
//                 .send_text(channel, input, quote, attachments);
//
//         let sent_with_quote = sent_message.quote.is_some();
//         channel.messages.push_back(sent_message);
//
//         // self.reset_unread_messages();
//         /*        if sent_with_quote {
//             self.reset_message_selection();
//         }*/
//         // self.bubble_up_channel(channel_idx);
//         self.save()
//     }
//
//     /*    pub fn select_previous_channel(&mut self) {
//         if self.reset_unread_messages() {
//             self.save().unwrap();
//         }
//         self.channels.previous();
//     }*/
//
//     /*    pub fn select_next_channel(&mut self) {
//         if self.reset_unread_messages() {
//             self.save().unwrap();
//         }
//         self.channels.next();
//     }*/
//
//     /*    pub fn on_pgup(&mut self) {
//             let select = self.channels.state.selected().unwrap_or_default();
//             self.channels.items[select].messages.next();
//         }
//
//         pub fn on_pgdn(&mut self) {
//             let select = self.channels.state.selected().unwrap_or_default();
//             self.channels.items[select].messages.previous();
//         }
//     */
//     /*    pub fn reset_unread_messages(&mut self) -> bool {
//         if let Some(selected_idx) = self.channels.state.selected() {
//             if self.channels.items[selected_idx].unread_messages > 0 {
//                 self.channels.items[selected_idx].unread_messages = 0;
//                 return true;
//             }
//         }
//         false
//     }*/
//
//     /*    pub async fn on_message(&mut self, content: Content) -> anyhow::Result<()> {
//         log::info!("incoming: {:?}", content);
//
//         let user_id = self.user_id;
//
//         let (channel_idx, message) = match (content.metadata, content.body) {
//             // Private note message
//             (
//                 _,
//                 ContentBody::SynchronizeMessage(SyncMessage {
//                     sent:
//                         Some(Sent {
//                             destination_uuid: Some(destination_uuid),
//                             timestamp: Some(timestamp),
//                             message:
//                                 Some(DataMessage {
//                                     body: Some(text), ..
//                                 }),
//                             ..
//                         }),
//                     ..
//                 }),
//             ) if destination_uuid.parse() == Ok(user_id) => {
//                 let channel_idx = self.ensure_own_channel_exists();
//                 let message = Message::new(user_id, text, timestamp);
//                 (channel_idx, message)
//             }
//             // Direct/group message by us from a different device
//             (
//                 Metadata {
//                     sender:
//                         ServiceAddress {
//                             uuid: Some(sender_uuid),
//                             ..
//                         },
//                     ..
//                 },
//                 ContentBody::SynchronizeMessage(SyncMessage {
//                     sent:
//                         Some(Sent {
//                             destination_e164,
//                             destination_uuid,
//                             timestamp: Some(timestamp),
//                             message:
//                                 Some(DataMessage {
//                                     body: Some(text),
//                                     group_v2,
//                                     quote,
//                                     ..
//                                 }),
//                             ..
//                         }),
//                     ..
//                 }),
//             ) if sender_uuid == user_id => {
//                 let channel_idx = if let Some(GroupContextV2 {
//                     master_key: Some(master_key),
//                     revision: Some(revision),
//                     ..
//                 }) = group_v2
//                 {
//                     // message to a group
//                     let master_key = master_key
//                         .try_into()
//                         .map_err(|_| anyhow!("invalid master key"))?;
//                     self.ensure_group_channel_exists(master_key, revision)
//                         .await
//                         .context("failed to create group channel")?
//                 } else if let (Some(destination_uuid), Some(destination_e164)) = (
//                     destination_uuid.and_then(|s| s.parse().ok()),
//                     destination_e164,
//                 ) {
//                     // message to a contact
//                     self.ensure_contact_channel_exists(destination_uuid, &destination_e164)
//                         .await
//                 } else {
//                     return Ok(());
//                 };
//
//                 let quote = quote.and_then(Message::from_quote).map(Box::new);
//                 let message = Message {
//                     quote,
//                     ..Message::new(user_id, text, timestamp)
//                 };
//                 (channel_idx, message)
//             }
//             // Incoming direct/group message
//             (
//                 Metadata {
//                     sender:
//                         ServiceAddress {
//                             uuid: Some(uuid),
//                             phonenumber: Some(phone_number),
//                             ..
//                         },
//                     ..
//                 },
//                 ContentBody::DataMessage(DataMessage {
//                     body: Some(text),
//                     group_v2,
//                     timestamp: Some(timestamp),
//                     profile_key: Some(profile_key),
//                     quote,
//                     ..
//                 }),
//             ) => {
//                 let (channel_idx, from) = if let Some(GroupContextV2 {
//                     master_key: Some(master_key),
//                     revision: Some(revision),
//                     ..
//                 }) = group_v2
//                 {
//                     // incoming group message
//                     let master_key = master_key
//                         .try_into()
//                         .map_err(|_| anyhow!("invalid group master key"))?;
//                     let channel_idx = self
//                         .ensure_group_channel_exists(master_key, revision)
//                         .await
//                         .context("failed to create group channel")?;
//                     let from = self
//                         .ensure_user_is_known(uuid, profile_key, phone_number)
//                         .await
//                         .to_string();
//
//                     (channel_idx, from)
//                 } else {
//                     // incoming direct message
//                     let name = self
//                         .ensure_user_is_known(uuid, profile_key, phone_number)
//                         .await
//                         .to_string();
//                     let channel_idx = self.ensure_contact_channel_exists(uuid, &name).await;
//                     let from = self.channels.get(&channel_idx).unwrap().name.clone();
//
//                     (channel_idx, from)
//                 };
//
//                 self.notify(&from, &text);
//
//                 let quote = quote.and_then(Message::from_quote).map(Box::new);
//                 let message = Message {
//                     quote,
//                     ..Message::new(uuid, text, timestamp)
//                 };
//                 (channel_idx, message)
//             }
//             // reactions
//             (
//                 Metadata {
//                     sender:
//                         ServiceAddress {
//                             uuid: Some(sender_uuid),
//                             ..
//                         },
//                     ..
//                 },
//                 ContentBody::SynchronizeMessage(SyncMessage {
//                     sent:
//                         Some(Sent {
//                             destination_uuid,
//                             message:
//                                 Some(DataMessage {
//                                     body: None,
//                                     group_v2,
//                                     reaction:
//                                         Some(Reaction {
//                                             emoji: Some(emoji),
//                                             remove,
//                                             target_author_uuid: Some(target_author_uuid),
//                                             target_sent_timestamp: Some(target_sent_timestamp),
//                                             ..
//                                         }),
//                                     ..
//                                 }),
//                             ..
//                         }),
//                     ..
//                 }),
//             ) => {
//                 let channel_id = if let Some(GroupContextV2 {
//                     master_key: Some(master_key),
//                     ..
//                 }) = group_v2
//                 {
//                     ChannelId::from_master_key_bytes(master_key)?
//                 } else if let Some(uuid) = destination_uuid {
//                     ChannelId::User(uuid.parse()?)
//                 } else {
//                     ChannelId::User(target_author_uuid.parse()?)
//                 };
//
//                 self.handle_reaction(
//                     channel_id,
//                     target_sent_timestamp,
//                     sender_uuid,
//                     emoji,
//                     remove.unwrap_or(false),
//                     true,
//                 );
//                 return Ok(());
//             }
//             (
//                 Metadata {
//                     sender:
//                         ServiceAddress {
//                             uuid: Some(sender_uuid),
//                             ..
//                         },
//                     ..
//                 },
//                 ContentBody::DataMessage(DataMessage {
//                     body: None,
//                     group_v2,
//                     reaction:
//                         Some(Reaction {
//                             emoji: Some(emoji),
//                             remove,
//                             target_sent_timestamp: Some(target_sent_timestamp),
//                             target_author_uuid: Some(target_author_uuid),
//                             ..
//                         }),
//                     ..
//                 }),
//             ) => {
//                 let channel_id = if let Some(GroupContextV2 {
//                     master_key: Some(master_key),
//                     ..
//                 }) = group_v2
//                 {
//                     ChannelId::from_master_key_bytes(master_key)?
//                 } else if sender_uuid == self.user_id {
//                     // reaction from us => target author is the user channel
//                     ChannelId::User(target_author_uuid.parse()?)
//                 } else {
//                     // reaction is from somebody else => they are the user channel
//                     ChannelId::User(sender_uuid.to_string())
//                 };
//
//                 self.handle_reaction(
//                     channel_id,
//                     target_sent_timestamp,
//                     sender_uuid,
//                     emoji,
//                     remove.unwrap_or(false),
//                     true,
//                 );
//                 return Ok(());
//             }
//             _ => return Ok(()),
//         };
//
//         self.add_message_to_channel(channel_idx, message);
//
//         Ok(())
//     }*/
//
//     pub fn handle_reaction(
//         &mut self,
//         channel_id: ChannelId,
//         target_sent_timestamp: u64,
//         sender_uuid: Uuid,
//         emoji: String,
//         remove: bool,
//         notify: bool,
//     ) -> Option<()> {
//         /*        let channel_idx = self
//
//         .channels
//         .items
//         .iter()
//         .position(|channel| channel.id == channel_id)?;*/
//         // let channel = &mut self.channels.items[channel_idx];
//         let channel = self.channels.get_mut(&channel_id).unwrap();
//         let message = channel
//             .messages
//             // .items
//             .iter_mut()
//             .find(|m| m.arrived_at == target_sent_timestamp)?;
//         let reaction_idx = message
//             .reactions
//             .iter()
//             .position(|(from_id, _)| from_id == &sender_uuid);
//         let is_added = if let Some(idx) = reaction_idx {
//             if remove {
//                 message.reactions.swap_remove(idx);
//                 false
//             } else {
//                 message.reactions[idx].1 = emoji.clone();
//                 true
//             }
//         } else {
//             message.reactions.push((sender_uuid, emoji.clone()));
//             true
//         };
//
//         if is_added && channel_id != ChannelId::User(self.user_id.to_string()) {
//             // Notification
//             let sender_name = name_by_id(&self.names, sender_uuid);
//             let summary = if let ChannelId::Group(_) = channel.id {
//                 Cow::from(format!("{} in {}", sender_name, channel.name))
//             } else {
//                 Cow::from(sender_name)
//             };
//             let mut notification = format!("{} reacted {}", summary, emoji);
//             if let Some(text) = message.message.as_ref() {
//                 notification.push_str(" to: ");
//                 notification.push_str(text);
//             }
//             if notify {
//                 self.notify(&summary, &notification);
//             }
//
//             // self.touch_channel(channel_idx);
//         } else {
//             self.save().unwrap();
//         }
//
//         Some(())
//     }
//
//     pub async fn ensure_group_channel_exists(
//         &mut self,
//         master_key: GroupMasterKeyBytes,
//         revision: u32,
//     ) -> anyhow::Result<ChannelId> {
//         let id = ChannelId::from_master_key_bytes(master_key)?;
//         let channel = self.channels.get(&id);
//         if channel.is_some() {
//             let is_stale = match channel.as_ref().unwrap().group_data.as_ref() {
//                 Some(group_data) => group_data.revision != revision,
//                 None => true,
//             };
//             if is_stale {
//                 let ResolvedGroup {
//                     name,
//                     group_data,
//                     profile_keys,
//                 } = self
//                     .signal_manager
//                     .clone()
//                     .unwrap()
//                     .resolve_group(master_key)
//                     .await?;
//
//                 self.try_ensure_users_are_known(
//                     group_data
//                         .members
//                         .iter()
//                         .copied()
//                         .zip(profile_keys.into_iter()),
//                 )
//                 .await;
//                 // channel.and_then(|c|{
//                 //     c.name = name;
//                 //     c.group_data.replace(group_data);
//                 //     Some(c)
//                 // });
//                 let c = Channel {
//                     id: id.to_owned(),
//                     name,
//                     group_data: None,
//                     messages: Vector::new(),
//                     unread_messages: 0,
//                     current_message: None,
//                 };
//                 self.channels.insert(id.to_owned(), c);
//                 // let channel =  channel.unwrap();
//                 // channel.name = name;
//                 // channel.group_data.replace(group_data);
//             }
//         } else {
//             let ResolvedGroup {
//                 name,
//                 group_data,
//                 profile_keys,
//             } = self
//                 .signal_manager
//                 .clone()
//                 .unwrap()
//                 .resolve_group(master_key)
//                 .await?;
//
//             self.try_ensure_users_are_known(
//                 group_data
//                     .members
//                     .iter()
//                     .copied()
//                     .zip(profile_keys.into_iter()),
//             )
//             .await;
//
//             self.channels.insert(
//                 id.to_owned(),
//                 Channel {
//                     id: id.to_owned(),
//                     name,
//                     group_data: Some(group_data),
//                     messages: Vector::new(),
//                     unread_messages: 0,
//                     current_message: None,
//                 },
//             );
//         }
//         Ok(id.to_owned())
//     }
//
//     pub async fn ensure_user_is_known(
//         &mut self,
//         uuid: Uuid,
//         profile_key: Vec<u8>,
//         phone_number: PhoneNumber,
//     ) -> &str {
//         if self
//             .try_ensure_user_is_known(uuid, profile_key)
//             .await
//             .is_none()
//         {
//             let phone_number_name = phone_number.format().mode(Mode::E164).to_string();
//             self.names.insert(uuid, phone_number_name);
//         }
//         self.names.get(&uuid).unwrap()
//     }
//
//     async fn try_ensure_user_is_known(&mut self, uuid: Uuid, profile_key: Vec<u8>) -> Option<&str> {
//         let is_phone_number_or_unknown = self
//             .names
//             .get(&uuid)
//             .map(util::is_phone_number)
//             .unwrap_or(true);
//         if is_phone_number_or_unknown {
//             let name = match profile_key.try_into() {
//                 Ok(key) => {
//                     self.signal_manager
//                         .as_ref()
//                         .unwrap()
//                         .contact_name(uuid, key)
//                         .await
//                 }
//                 Err(_) => None,
//             };
//             self.names.insert(uuid, name?);
//         }
//         self.names.get(&uuid).map(|s| s.as_str())
//     }
//
//     async fn try_ensure_users_are_known(
//         &mut self,
//         users_with_keys: impl Iterator<Item = (Uuid, Vec<u8>)>,
//     ) {
//         // TODO: Run in parallel
//         for (uuid, profile_key) in users_with_keys {
//             self.try_ensure_user_is_known(uuid, profile_key).await;
//         }
//     }
//
//     pub fn ensure_own_channel_exists(&mut self) -> ChannelId {
//         let user_id = self.user_id.to_owned();
//         let channel_id = ChannelId::from(Uuid::from_str(user_id.as_str()).unwrap());
//         if let Some((channel_id, _)) = self
//             .channels
//             .iter_mut()
//             .find(|channel| channel.1.user_id() == Some(user_id.to_owned()))
//         {
//             channel_id.to_owned()
//         } else {
//             self.channels.insert(
//                 channel_id.to_owned(),
//                 Channel {
//                     id: channel_id.to_owned(),
//                     name: self.config.as_ref().unwrap().user.name.clone(),
//                     group_data: None,
//                     messages: Vector::new(),
//                     unread_messages: 0,
//                     current_message: None,
//                 },
//             );
//             channel_id
//         }
//     }
//
//     pub async fn ensure_contact_channel_exists(&mut self, uuid: Uuid, name: &str) -> ChannelId {
//         if let Some(channel_id) = self
//             .channels
//             .iter()
//             .find(|channel| channel.1.user_id() == Some(uuid.to_string()))
//             .map(|(id, _)| id.to_owned())
//         {
//             let id = channel_id.clone();
//             if let Some(name) = self.names.get(&uuid) {
//                 let channel = self.channels.get_mut(&channel_id).unwrap();
//                 if &channel.name != name {
//                     channel.name = name.clone();
//                 }
//             }
//             id
//         } else {
//             self.channels.insert(
//                 uuid.into(),
//                 Channel {
//                     id: uuid.into(),
//                     name: name.to_string(),
//                     group_data: None,
//                     messages: Vector::new(),
//                     unread_messages: 0,
//                     current_message: None,
//                 },
//             );
//             uuid.into()
//         }
//     }
//
//     pub fn add_message_to_channel(&mut self, channel_id: ChannelId, message: Message) {
//         let channel = self.channels.get_mut(&channel_id).unwrap();
//
//         channel.messages.push_back(message.to_owned());
//         // if let Some(idx) = channel.messages.state.selected() {
//         //     // keep selection on the old message
//         //     channel.messages.state.select(Some(idx + 1));
//         // }
//         channel.current_message.replace(message);
//         log::info!("add message to channel: {:?}=={:?}",&channel_id,&channel)
//         // self.touch_channel(channel_idx);
//     }
//
//     /*    fn touch_channel(&mut self, channel_idx: usize) {
//         if self.channels.state.selected() != Some(channel_idx) {
//             self.channels.items[channel_idx].unread_messages += 1;
//         } else {
//             self.reset_unread_messages();
//         }
//
//         self.bubble_up_channel(channel_idx);
//         self.save().unwrap();
//     }*/
//
//     /*    fn bubble_up_channel(&mut self, channel_idx: usize) {
//         // bubble up channel to the beginning of the list
//         let channels = &mut self.channels;
//         for (prev, next) in (0..channel_idx).zip(1..channel_idx + 1).rev() {
//             channels.items.swap(prev, next);
//         }
//         match channels.state.selected() {
//             Some(selected_idx) if selected_idx == channel_idx => channels.state.select(Some(0)),
//             Some(selected_idx) if selected_idx < channel_idx => {
//                 channels.state.select(Some(selected_idx + 1));
//             }
//             _ => {}
//         };
//     }*/
//
//     pub fn notify(&self, summary: &str, text: &str) {
//         if let Err(e) = Notification::new().summary(summary).body(text).show() {
//             error!("failed to send notification: {}", e);
//         }
//     }
//
//     fn extract_attachments(&mut self, input: &str) -> (String, Vec<(AttachmentSpec, Vec<u8>)>) {
//         let mut offset = 0;
//         let mut clean_input = String::new();
//         let mut attachment_regex = self.attachment_regex.clone().unwrap();
//         let re = attachment_regex.compiled();
//         let attachments = re.find_iter(input.as_bytes()).filter_map(|(start, end)| {
//             let path_str = &input[start..end].strip_prefix("file://")?;
//
//             let path = Path::new(path_str);
//             let contents = std::fs::read(path).ok()?;
//
//             clean_input.push_str(input[offset..start].trim_end_matches(""));
//             offset = end;
//
//             let content_type = mime_guess::from_path(path)
//                 .first()
//                 .map(|mime| mime.essence_str().to_string())
//                 .unwrap_or_default();
//             let spec = AttachmentSpec {
//                 content_type,
//                 length: contents.len(),
//                 file_name: Path::new(path)
//                     .file_name()
//                     .map(|f| f.to_string_lossy().into()),
//                 preview: None,
//                 voice_note: None,
//                 borderless: None,
//                 width: None,
//                 height: None,
//                 caption: None,
//                 blur_hash: None,
//             };
//             Some((spec, contents))
//         });
//
//         let attachments = attachments.collect();
//         clean_input.push_str(&input[offset..]);
//         let clean_input = clean_input.trim().to_string();
//
//         (clean_input, attachments)
//     }
// }
//
// pub fn name_by_id(names: &HashMap<Uuid, String>, id: Uuid) -> &str {
//     names.get(&id).map(|s| s.as_ref()).unwrap_or("Unknown Name")
// }
//
// /// Returns an emoji string if `s` is an emoji or if `s` is a GitHub emoji shortcode.
// fn to_emoji(s: &str) -> Option<&str> {
//     let s = s.trim();
//     if emoji::lookup_by_glyph::lookup(s).is_some() {
//         Some(s)
//     } else {
//         let s = s.strip_prefix(':')?.strip_suffix(':')?;
//         let emoji = gh_emoji::get(s)?;
//         Some(emoji)
//     }
// }
//
// fn open_url(message: &Message, url_regex: &Regex) -> Option<()> {
//     let text = message.message.as_ref()?;
//     let (start, end) = url_regex.find(text.as_bytes())?;
//     let url = &text[start..end];
//     if let Err(e) = opener::open(url) {
//         error!("failed to open {}: {}", url, e);
//     }
//     Some(())
// }
//
// /*#[cfg(test)]
// mod tests {
//     use super::*;
//
//     use crate::config::User;
//     use crate::signal::test::SignalManagerMock;
//     use crate::storage::test::InMemoryStorage;
//
//     use std::cell::RefCell;
//     use std::rc::Rc;
//
//     fn test_app() -> (App, Rc<RefCell<Vec<Message>>>) {
//         let signal_manager.as_ref().unwrap() = SignalManagerMock::new();
//         let sent_messages = signal_manager.as_ref().unwrap().sent_messages.clone();
//
//         let mut app = App::try_new(
//             Config::with_user(User {
//                 name: "Tyler Durden".to_string(),
//                 phone_number: "+0000000000".to_string(),
//             }),
//             Box::new(signal_manager.as_ref().unwrap()),
//             Box::new(InMemoryStorage::new()),
//         )
//         .unwrap();
//
//         app.channels.items.push(Channel {
//             id: ChannelId::User(Uuid::new_v4()),
//             name: "test".to_string(),
//             group_data: Some(GroupData {
//                 master_key_bytes: GroupMasterKeyBytes::default(),
//                 members: vec![app.user_id],
//                 revision: 1,
//             }),
//             messages: StatefulList::with_items(vec![Message {
//                 from_id: app.user_id,
//                 message: Some("First message".to_string()),
//                 arrived_at: 0,
//                 quote: Default::default(),
//                 attachments: Default::default(),
//                 reactions: Default::default(),
//             }]),
//             unread_messages: 1,
//         });
//         app.channels.state.select(Some(0));
//
//         (app, sent_messages)
//     }
//
//     #[test]
//     fn test_send_input() {
//         let (mut app, sent_messages) = test_app();
//         let input = "Hello, World!";
//         for c in input.chars() {
//             app.put_char(c);
//         }
//         app.send_input(0).unwrap();
//
//         let sent = sent_messages.borrow();
//         assert_eq!(sent.len(), 1);
//         assert_eq!(sent[0].message.as_ref().unwrap(), input);
//
//         assert_eq!(app.channels.items[0].unread_messages, 0);
//
//         assert_eq!(app.input, "");
//         assert_eq!(app.input_cursor, 0);
//         assert_eq!(app.input_cursor_chars, 0);
//     }
//
//     #[test]
//     fn test_send_input_with_emoji() {
//         let (mut app, sent_messages) = test_app();
//         let input = "👻";
//         for c in input.chars() {
//             app.put_char(c);
//         }
//         assert_eq!(app.input_cursor, 4);
//         assert_eq!(app.input_cursor_chars, 1);
//
//         app.send_input(0).unwrap();
//
//         let sent = sent_messages.borrow();
//         assert_eq!(sent.len(), 1);
//         assert_eq!(sent[0].message.as_ref().unwrap(), input);
//
//         assert_eq!(app.input, "");
//         assert_eq!(app.input_cursor, 0);
//         assert_eq!(app.input_cursor_chars, 0);
//     }
//
//     #[test]
//     fn test_send_input_with_emoji_codepoint() {
//         let (mut app, sent_messages) = test_app();
//         let input = ":thumbsup:";
//         for c in input.chars() {
//             app.put_char(c);
//         }
//
//         app.send_input(0).unwrap();
//
//         let sent = sent_messages.borrow();
//         assert_eq!(sent.len(), 1);
//         assert_eq!(sent[0].message.as_ref().unwrap(), "👍");
//     }
//
//     #[test]
//     fn test_add_reaction_with_emoji() {
//         let (mut app, _sent_messages) = test_app();
//
//         app.channels.items[0].messages.state.select(Some(0));
//
//         app.put_char('👍');
//         app.add_reaction(0);
//
//         let reactions = &app.channels.items[0].messages.items[0].reactions;
//         assert_eq!(reactions.len(), 1);
//         assert_eq!(reactions[0], (app.user_id, "👍".to_string()));
//     }
//
//     #[test]
//     fn test_add_reaction_with_emoji_codepoint() {
//         let (mut app, _sent_messages) = test_app();
//
//         app.channels.items[0].messages.state.select(Some(0));
//
//         for c in ":thumbsup:".chars() {
//             app.put_char(c);
//         }
//         app.add_reaction(0);
//
//         let reactions = &app.channels.items[0].messages.items[0].reactions;
//         assert_eq!(reactions.len(), 1);
//         assert_eq!(reactions[0], (app.user_id, "👍".to_string()));
//     }
//
//     #[test]
//     fn test_remove_reaction() {
//         let (mut app, _sent_messages) = test_app();
//
//         app.channels.items[0].messages.state.select(Some(0));
//         let reactions = &mut app.channels.items[0].messages.items[0].reactions;
//         reactions.push((app.user_id, "👍".to_string()));
//
//         app.add_reaction(0);
//
//         let reactions = &app.channels.items[0].messages.items[0].reactions;
//         assert!(reactions.is_empty());
//     }
//
//     #[test]
//     fn test_add_invalid_reaction() {
//         let (mut app, _sent_messages) = test_app();
//
//         app.channels.items[0].messages.state.select(Some(0));
//
//         for c in ":thumbsup".chars() {
//             app.put_char(c);
//         }
//         app.add_reaction(0);
//
//         assert_eq!(app.input, ":thumbsup");
//         let reactions = &app.channels.items[0].messages.items[0].reactions;
//         assert!(reactions.is_empty());
//     }
// }*/
