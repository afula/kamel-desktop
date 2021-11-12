use crate::states::{ChannelId, IncomingMsg, Platform};
use druid::Selector;

pub const SET_CURRENT_CHANNEL: Selector<ChannelId> = Selector::new("app.set-current-channel");

pub const SET_CURRENT_PLATFORM: Selector<Platform> = Selector::new("app.set-current-channel");

pub const SET_INCOMING_MSG: Selector<IncomingMsg> = Selector::new("app.set-incoming-msg");
