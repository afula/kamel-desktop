use anyhow::anyhow;
use anyhow::Context;
use druid::im::Vector;
use druid::{Data, ExtEventSink, Lens, Target};
use gui::controller::command;
use gui::states::IncomingMsg;
use gui::views::style::Theme;
use log::{error, info};
use presage::prelude::{
    content::{ContentBody, DataMessage, Metadata, SyncMessage},
    proto::{
        data_message::{Quote, Reaction},
        sync_message::Sent,
        GroupContextV2,
    },
    AttachmentSpec, Content, GroupMasterKey, GroupSecretParams, ServiceAddress,
};
use signal::{signal::Manager, AppData, ChannelId, Event, Message};
use std::str::FromStr;
use tokio::time::sleep;
use tokio_stream::StreamExt;
use uuid::Uuid;

async fn is_online() -> bool {
    tokio::net::TcpStream::connect("detectportal.firefox.com:80")
        .await
        .is_ok()
}
// #[derive(Data,Lens)]
pub struct SignalProcessor {
    pub data: AppData,
    pub event_sink: ExtEventSink,
}
impl SignalProcessor {
    pub async fn process(&mut self, signal_manager: Manager) -> anyhow::Result<()> {
        let (tx, mut rx) = tokio::sync::mpsc::channel::<Event>(1024);

        /*    tokio::spawn({
            let tx = tx.clone();
            async move {
                let mut reader = EventStream::new().fuse();
                while let Some(event) = reader.next().await {
                    match event {
                        Ok(CEvent::Key(key)) => tx.send(Event::Input(key)).await.unwrap(),
                        Ok(CEvent::Resize(cols, rows)) => {
                            tx.send(Event::Resize { cols, rows }).await.unwrap()
                        }
                        Ok(CEvent::Mouse(button)) => tx.send(Event::Click(button)).await.unwrap(),
                        _ => (),
                    }
                }
            }
        });*/

        let inner_tx = tx.clone();

        tokio::task::spawn_local(async move {
            loop {
                let messages = if !is_online().await {
                    tokio::time::sleep(std::time::Duration::from_secs(10)).await;
                    continue;
                } else {
                    match signal_manager.receive_messages().await {
                        Ok(messages) => {
                            info!("connected and listening for incoming messages");
                            messages
                        }
                        Err(e) => {
                            let e = anyhow::Error::from(e).context(
                                "failed to initialize the stream of Signal messages.\n\
                            Maybe the device was unlinked? Please try to restart with '--relink` flag.",
                            );
                            inner_tx
                                .send(Event::Quit(Some(e)))
                                .await
                                .expect("logic error: events channel closed");
                            return;
                        }
                    }
                };

                tokio::pin!(messages);
                while let Some(message) = messages.next().await {
                    inner_tx
                        .send(Event::Message(message))
                        .await
                        .expect("logic error: events channel closed")
                }
                info!("messages channel disconnected. trying to reconnect.")
            }
        });

        let mut res = Ok(()); // result on quit

        loop {
            sleep(std::time::Duration::from_secs(1)).await;
            println!("1 s have elapsed");
            match rx.recv().await {
                Some(Event::Message(content)) => {
                    if let Err(e) = self.on_message(content).await {
                        error!("failed on incoming message: {}", e);
                    }
                    println!("processor state: {:?}", &self.data.channels);
                }
                Some(Event::Quit(e)) => {
                    if let Some(e) = e {
                        res = Err(e);
                    };
                    break;
                }
                None => {
                    break;
                }
                _ => {}
            }
            if self.data.should_quit {
                break;
            }
        }
        res
    }
    pub async fn on_message(&mut self, content: Content) -> anyhow::Result<()> {
        log::info!("incoming: {:?}", content);

        let user_id = self.data.user_id;

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
                let channel_idx = self.data.ensure_own_channel_exists();
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
                let mut from = String::new();
                let channel_id = if let Some(GroupContextV2 {
                    master_key: Some(master_key),
                    revision: Some(revision),
                    ..
                }) = group_v2
                {
                    // message to a group
                    let master_key = master_key
                        .try_into()
                        .map_err(|_| anyhow!("invalid master key"))?;
                    let (id, name) = self
                        .data
                        .ensure_group_channel_exists(master_key, revision)
                        .await
                        .context("failed to create group channel")?;
                    from = name;
                    id
                } else if let (Some(destination_uuid), Some(destination_e164)) = (
                    destination_uuid.and_then(|s| s.parse().ok()),
                    destination_e164,
                ) {
                    // message to a contact
                    from = destination_e164.to_owned();
                    self.data
                        .ensure_contact_channel_exists(destination_uuid, &destination_e164)
                        .await
                } else {
                    return Ok(());
                };

                let quote = quote.and_then(Message::from_quote).map(Box::new);
                let message = Message {
                    quote,
                    ..Message::new(user_id, text, timestamp)
                };
                self.send_gui_incoming_msg(&from,&channel_id,&message)?;
                (channel_id, message)
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
                let (channel_id, from) = if let Some(GroupContextV2 {
                    master_key: Some(master_key),
                    revision: Some(revision),
                    ..
                }) = group_v2
                {
                    // incoming group message
                    let master_key = master_key
                        .try_into()
                        .map_err(|_| anyhow!("invalid group master key"))?;
                    let (channel_id,_name) = self
                        .data
                        .ensure_group_channel_exists(master_key, revision)
                        .await
                        .context("failed to create group channel")?;
                    let from = self
                        .data
                        .ensure_user_is_known(uuid, profile_key, phone_number)
                        .await
                        .to_string();

                    (channel_id, from)
                } else {
                    // incoming direct message
                    let name = self
                        .data
                        .ensure_user_is_known(uuid, profile_key, phone_number)
                        .await
                        .to_string();
                    let channel_id = self.data.ensure_contact_channel_exists(uuid, &name).await;
                    let from = self.data.channels.get(&channel_id).unwrap().name.clone();

                    (channel_id, from)
                };

                self.data.notify(&from, &text);

                let quote = quote.and_then(Message::from_quote).map(Box::new);
                let message = Message {
                    quote,
                    ..Message::new(uuid, text, timestamp)
                };
                self.send_gui_incoming_msg(from.as_str(),&channel_id,&message)?;
                (channel_id, message)
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

                self.data.handle_reaction(
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
                } else if sender_uuid == self.data.user_id {
                    // reaction from us => target author is the user channel
                    ChannelId::User(target_author_uuid.parse()?)
                } else {
                    // reaction is from somebody else => they are the user channel
                    ChannelId::User(sender_uuid)
                };

                self.data.handle_reaction(
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

        self.data.add_message_to_channel(channel_idx, message);

        Ok(())
    }
    fn send_gui_incoming_msg(&self, name: &str,id: &signal::ChannelId, message: &signal::Message) -> anyhow::Result<()>{
            use gui::states::Attachment;
            use gui::states::ChannelId;
            use gui::states::GroupIdentifierBytes;
            use gui::states::Message;
            let channel_id = match &id {
                signal::ChannelId::User(user) => ChannelId::User(user.to_string()),
                signal::ChannelId::Group(group) => {
                    ChannelId::Group(group.to_owned().into())
                }
            };
            let quote_msg = match &message.quote {
                None => None,
                Some(q) => {
                    let mut attachments = Vector::new();
                    for attachment in &q.attachments {
                        attachments.push_back(Attachment {
                            id: attachment.id.to_owned(),
                            content_type: attachment.content_type.to_owned(),
                            filename: attachment.filename.to_string_lossy().to_string(),
                            size: 0,
                        })
                    }
                    Some(Box::new(Message {
                        from_id: q.from_id.to_string(),
                        message: q.message.to_owned(),
                        quote: None,
                        attachments,
                        reactions: Default::default(),
                    }))
                }
            };
            let mut attachments = Vector::new();
            for attachment in &message.attachments {
                attachments.push_back(Attachment {
                    id: attachment.id.to_owned(),
                    content_type: attachment.content_type.to_owned(),
                    filename: attachment.filename.to_string_lossy().to_string(),
                    size: 0,
                })
            }
            let message = Message {
                from_id: message.from_id.to_string(),
                message: message.message.to_owned(),
                quote: quote_msg,
                attachments,
                reactions: Default::default(),
            };
            let incoming_msg = IncomingMsg {
                id: channel_id,
                name: name.to_string(),
                message,
            };
            self.event_sink
                .submit_command(command::SET_INCOMING_MSG, incoming_msg, Target::Auto)
                .context("send Direct/group message by us from a different device")?;
        Ok(())
    }
}


