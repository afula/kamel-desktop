use druid::Selector;
use crate::states::{ChannelId, Platform};


pub const SET_CURRENT_CHANNEL: Selector<ChannelId> = Selector::new("app.set-current-channel");

pub const SET_CURRENT_PLATFORM: Selector<Platform> = Selector::new("app.set-current-channel");
