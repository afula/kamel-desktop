use crate::states::{Channel, Message, OutgoingMsg, SignalState, Attachment,OWNER};
use druid::keyboard_types::Key;
use druid::widget::Controller;
use druid::{Env, Event, EventCtx, Widget};
use druid::im::Vector;

pub struct MessageInputController;

impl<W> Controller<SignalState, W> for MessageInputController
where
    W: Widget<SignalState>,
{
    fn event(
        &mut self,
        child: &mut W,
        ctx: &mut EventCtx,
        event: &Event,
        state: &mut SignalState,
        env: &Env,
    ) {
        match event {
            Event::KeyDown(key) if key.key == Key::Enter && !key.mods.shift() => {
                if !state.data.input.is_empty() {
                    println!("current input: {:?}", &state.data.input);
                    let channel_id = state.data.current_channel.as_ref().unwrap();
                    let mut attachments = Vector::new();
                    let attachment = Attachment{
                        id: "".to_string(),
                        content_type: "".to_string(),
                        filename: "/home/damo/Pictures/Screenshot_2021-10-05_07-01-21.png".to_string(),
                        size: 0
                    };
                    attachments.push_back(attachment);
                    let message = Message {
                        from_id: state.data.user_id.to_owned(),
                        message: Some(state.data.input.to_owned()),
                        arrived_at: 0,
                        quote: None,
                        attachments,
                        reactions: Default::default(),
                    };
                    let outgoing_msg = OutgoingMsg {
                        channel_id: channel_id.to_owned(),
                        message: message.to_owned(),
                    };
                    //send to message processor
                    state
                        .data
                        .outgoing_msg_sender
                        .as_ref()
                        .unwrap()
                        .try_send(outgoing_msg)
                        .map_err(|e| e); //TODO
                                         //put back in the gui relative channel message container
                    let channel = state.data.channels.get_mut(channel_id);
                    match channel {
                        Some(channel) => {
                            channel.messages.push_back(message);
                            unsafe {
                                println!("gui state: {:?} --- {:?}", &channel.messages, OWNER);
                            }
                        }
                        None => {
                            log::error!("try to send message to a unknown channel");
                            // state.data.channels.insert(
                            //     channel_id.to_owned(),
                            //     Channel {
                            //         id: channel_id.to_owned(),
                            //         name: incoming_msg.name.to_owned(),
                            //         group_data: None,
                            //         messages: Vector::from(vec![incoming_msg.message.to_owned()]),
                            //         unread_messages: 0,
                            //     },
                            // );
                        }
                    }
                    state.data.input.clear();
                    ctx.set_handled();
                } else {
                    ctx.set_handled();
                }
            }
            _ => (),
        }
        child.event(ctx, event, state, env);
    }
}
