use crate::controller::command;
use crate::states::{Channel, SignalState};
use druid::{commands, im::Vector, AppDelegate, Command, DelegateCtx, Env, Handled, Target};

#[derive(Default)]
pub struct ApplicationDelegate;

impl AppDelegate<SignalState> for ApplicationDelegate {
    fn command(
        &mut self,
        _ctx: &mut DelegateCtx,
        _target: Target,
        cmd: &Command,
        state: &mut SignalState,
        _env: &Env,
    ) -> Handled {
        if let Some(file_info) = cmd.get(commands::OPEN_FILE) {
            return Handled::Yes;
        }
        if let Some(channel_id) = cmd.get(command::SET_CURRENT_CHANNEL) {
            state.data.current_channel.replace(channel_id.to_owned());
            return Handled::Yes;
        }
        if let Some(incoming_msg) = cmd.get(command::SET_INCOMING_MSG) {
            let channel_id = &incoming_msg.id;
            let channel = state.data.channels.get_mut(channel_id);
            match channel {
                Some(channel) => {
                    channel.messages.push_back(incoming_msg.message.to_owned());
                }
                None => {
                    state.data.channels.insert(
                        channel_id.to_owned(),
                        Channel {
                            id: channel_id.to_owned(),
                            name: incoming_msg.name.to_owned(),
                            group_data: None,
                            messages: Vector::from(vec![incoming_msg.message.to_owned()]),
                            unread_messages: 0,
                        },
                    );
                }
            }

            return Handled::Yes;
        }
        Handled::No
    }
}
