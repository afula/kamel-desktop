#![windows_subsystem = "windows"]

use async_std::net::TcpStream;
use async_std::task::block_on;
use druid::piet::{PietTextLayoutBuilder, TextStorage as PietTextStorage};
use druid::text::{Attribute, RichText, TextStorage};
use druid::widget::prelude::*;
use druid::widget::{Button, Controller, Flex, Label, LineBreaking, RadioGroup, RawLabel, Scroll};
use druid::{
    AppLauncher, Color, Data, FontFamily, FontStyle, FontWeight, Lens, LocalizedString,
    TextAlignment, Widget, WidgetExt, WindowDesc,
};
use futures::{AsyncReadExt, AsyncWriteExt};
use gui::delegate::application::ApplicationDelegate;
use gui::states::{OutgoingMsg, SignalState, OWNER};
use gui::views::make_ui;
use kamel::signal::processor::SignalProcessor;
use log::{error, info};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio_stream::StreamExt;

use signal::storage::Storage;
use signal::util::{LazyRegex, ATTACHMENT_REGEX, URL_REGEX};
use signal::{signal::PresageManager, storage::JsonStorage, AppData};
fn init_file_logger(verbosity: u8) -> anyhow::Result<()> {
    use log::LevelFilter;
    use log4rs::append::file::FileAppender;
    use log4rs::config::{Appender, Config, Root};
    use log4rs::encode::pattern::PatternEncoder;

    let logfile = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new(
            "[{d} {l} {M}] {m} in {f}:{L}\n",
        )))
        .build("kamel.log")?;

    let config = Config::builder()
        .appender(Appender::builder().build("logfile", Box::new(logfile)))
        .build(Root::builder().appender("logfile").build(match verbosity {
            1 => LevelFilter::Info,
            2 => LevelFilter::Debug,
            _ => LevelFilter::Trace,
        }))?;

    log4rs::init_config(config)?;
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_file_logger(2)?;
    log_panics::init();
    let main_window = WindowDesc::new(make_ui())
        .title(LocalizedString::new("kamel-window-title").with_placeholder("Kamel"));
    let launcher = AppLauncher::with_window(main_window);
    let event_sink = launcher.get_external_handle();

    let (signal_manager, config) = signal::signal::ensure_linked_device(false).await?;
    let storage = JsonStorage::new(
        config.data_path.clone(),
        signal::config::fallback_data_path(),
    );
    let loaded_data =
        AppData::try_new(config, PresageManager::new(signal_manager.clone()), storage)?;
    let user_id = loaded_data.user_id.to_owned().to_string();
    unsafe {
        OWNER = user_id.to_owned();
    }

    let mut processor = SignalProcessor {
        data: loaded_data,
        event_sink,
    };

    let mut signal_state = SignalState::default();
    use tokio::runtime::Runtime;

    let rt = Runtime::new().unwrap();

    let (outgoing_msg_sender, outgoing_msg_receiver) =
        tokio::sync::mpsc::channel::<OutgoingMsg>(1024);

    signal_state.data.outgoing_msg_sender = Some(outgoing_msg_sender);
    signal_state.data.user_id = user_id;

    std::thread::spawn(move || {
        tokio::task::LocalSet::new().block_on(&rt, async {
            processor
                .process(signal_manager, outgoing_msg_receiver)
                .await;
        });
    });

    launcher
        .log_to_console()
        .delegate(ApplicationDelegate)
        .launch(signal_state)
        .expect("launch failed");
    Ok(())
}
