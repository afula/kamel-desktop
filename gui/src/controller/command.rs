use druid::Selector;

use signal::{ChannelId, Platform};

pub const SET_CURRENT_CHANNEL: Selector<ChannelId> = Selector::new("app.set-current-channel");

pub const SET_CURRENT_PLATFORM: Selector<Platform> = Selector::new("app.set-current-channel");
