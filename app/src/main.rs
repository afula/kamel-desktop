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
use gui::states::{SignalState, OWNER};
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
    init_file_logger(1)?;
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
    use std::sync::Arc;
    let mut processor = SignalProcessor { data: loaded_data };

    let signal_state = SignalState::default();
    use tokio::runtime::Runtime;

    let rt = Runtime::new().unwrap();

    std::thread::spawn(move || {
        tokio::task::LocalSet::new().block_on(&rt, async {
            processor.process(signal_manager, event_sink).await;
        });
    });
    /*    tokio::task::spawn(async move {
        processor.process(signal_manager, event_sink).await;
    });*/
    // task.await.unwrap();
    let state_for_print = signal_state.clone();
    std::thread::spawn(move || loop {
        println!("druid state: {:?}", &state_for_print.data.channels);
        std::thread::sleep(std::time::Duration::from_secs(1));
    });

    launcher
        .log_to_console()
        .launch(signal_state)
        .expect("launch failed");
    Ok(())
}
