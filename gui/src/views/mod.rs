// On Windows platform, don't show a console when opening the app.
#![windows_subsystem = "windows"]

pub mod menu;
pub mod message;
pub mod style;
pub mod theme;

use druid::{
    commands, lens,
    tests::helpers::widget_ids,
    widget::{
        Button, Controller, CrossAxisAlignment, Flex, Label, LineBreaking, List, MainAxisAlignment,
        Painter, Scroll, Slider, Split,
    },
    AppDelegate, AppLauncher, Color, Command, Data, DelegateCtx, Env, Event, EventCtx,
    ExtEventSink, FileDialogOptions, FileSpec, Handled, ImageBuf, Insets, Lens, LensExt,
    RenderContext, Selector, Target, UnitPoint, Widget, WidgetExt, WindowDesc,
};

use crate::controller::command;
use crate::controller::{
    channel::ChannelController, input::MessageInputController, platform::PlatformController,
};
use crate::delegate::main_menu::MainMenuDelegate;
use crate::states::{AppState, Channel, SignalData, SignalState};
use crate::views::{menu::make_menu, theme::ThemeScope};
use druid::im::Vector;
use druid::widget::{Either, FlexParams, Image, Spinner, TextBox};
use message::make_message_list;

use signal::ChannelId;
use std::{
    sync::{mpsc, mpsc::Sender, Arc},
    thread,
    time::Duration,
};
pub fn make_ui() -> impl Widget<SignalState> {
    let channels = Scroll::new(
        List::new(|| {
            Label::raw()
                .with_line_break_mode(LineBreaking::Clip)
                .with_text_size(style::TEXT_SIZE_SMALL)
                .lens(lens::Identity.map(
                    // Expose shared data with children data
                    |data: &Channel| data.name.to_owned(),
                    |_, _| {},
                ))
                .expand_width()
                .center()
                .padding(Insets::uniform_xy(style::grid(2.0), style::grid(0.6)))
                // .link()
                .on_click(|ctx, channel, _| {
                    let channel_id = channel.id.to_owned();
                    ctx.submit_command(command::SET_CURRENT_CHANNEL.with(channel_id));
                })
            // .controller(ChannelController)
        })
        .with_spacing(10.),
    )
    .vertical()
    // .fix_height(300.0)
    // .expand_height()
    .lens(lens::Identity.map(
        // Expose shared data with children data
        |data: &SignalState| {
            data.data
                .channels
                .iter()
                .map(|(_, channel)| channel.to_owned())
                .collect::<Vector<Channel>>()
        },
        |_, _| {},
    ));

    /*    let signal = Label::new(format!("#{}", "Signal"))
        // .align_vertical(UnitPoint::LEFT)
        .padding(10.0)
        .expand()
        .center()
        // .height(50.0)
        .background(Color::rgb(0.5, 0.5, 0.5))
        .on_click(|ctx, _, _| {
            ctx.submit_command(command::SET_CURRENT_PLATFORM.with(Platform::Signal));
        });
    // .controller(PlatformController);

    let matrix = Label::new(format!("#{}", "Matrix"))
        // .align_vertical(UnitPoint::LEFT)
        .padding(10.0)
        .expand()
        .center()
        .height(50.0)
        .background(Color::rgb(0.5, 0.5, 0.5))
        .on_click(|ctx, _, _| {
            ctx.submit_command(command::SET_CURRENT_PLATFORM.with(Platform::Matrix));
        });
    // .controller(PlatformController);

    let platform = Flex::row()
        .with_flex_child(signal, 1.0)
        .with_default_spacer()
        .with_flex_child(matrix, 1.0);*/
    let sidebar = Flex::column()
        .cross_axis_alignment(CrossAxisAlignment::Start)
        .must_fill_main_axis(true)
        // .with_child(platform)
        // .with_default_spacer()
        .with_flex_child(channels, 1.0)
        // .with_child(playlist)
        // .with_default_spacer()
        // .with_child(volume_slider())
        .padding(if cfg!(target_os = "macos") {
            // Accommodate the window controls on Mac.
            Insets::new(0.0, 24.0, 0.0, 0.0)
        } else {
            Insets::ZERO
        })
        .background(style::BACKGROUND_LIGHT);

    let message_list = Either::new(
        |data: &SignalData, _| match data.current_channel.as_ref() {
            Some(channel_id) => {
                if let Some(channel) = data.channels.get(channel_id) {
                    channel.messages.is_empty()
                } else {
                    true
                }
            }
            None => true,
        },
        Spinner::new(),
        make_message_list(),
    )
    .lens(SignalState::data);

    let textinput = TextBox::multiline()
        .with_placeholder("Send a message!")
        .lens(SignalState::data.then(SignalData::input))
        .expand_width()
        .controller(MessageInputController)
        .border(Color::grey(0.6), 1.5)
        .rounded(10.0)
        .padding(Insets::new(7.0, 0.0, 7.0, 9.0))
        .scroll()
        .vertical();

    let user_name = Label::new(format!("Messaging with #{}", "User"))
        .with_line_break_mode(LineBreaking::Clip)
        .with_text_size(style::TEXT_SIZE_SMALL)
        .align_vertical(UnitPoint::LEFT)
        .padding(10.0)
        .expand()
        .height(35.0)
        .background(Color::rgb(0.7, 0.7, 0.7));

    let user = Flex::row()
        // .main_axis_alignment(MainAxisAlignment::Start)
        .with_flex_child(user_name, 1.0)
        // .expand_height()
        .background(style::BACKGROUND_LIGHT);

    let main_list = Flex::column()
        // .main_axis_alignment(MainAxisAlignment::SpaceAround)
        .with_child(user)
        .with_flex_child(message_list.expand(), 1.0)
        .with_child(textinput)
        .expand_height()
        .background(style::BACKGROUND_LIGHT);

    let main = Flex::column()
        .with_flex_child(
            Either::new(
                |data: &SignalState, _| data.data.current_channel.is_some(),
                main_list,
                Image::new(ImageBuf::empty()),
            ),
            1.0,
        )
        .background(style::BACKGROUND_LIGHT);

    let split = Split::columns(sidebar, main)
        .split_point(0.2)
        .bar_size(1.0)
        .min_size(150.0, 300.0)
        .min_bar_area(1.0)
        .solid_bar(true);
    // split
    ThemeScope::new(split)
}
