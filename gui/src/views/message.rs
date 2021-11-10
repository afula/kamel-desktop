use crate::views::theme;
use druid::widget::{Either, Label, LineBreaking, List, Scroll};
use druid::{lens, Insets, LensExt, Widget, WidgetExt};
use signal::{AppData, Message};
use uuid::Uuid;
use crate::states::OWNER;
pub fn make_message_list() -> impl Widget<AppData> {
    // let mut owner = Uuid::default();
    let playlist = Scroll::new(
        List::new(|| unsafe {
            Either::new(
                |data: &Message, _| OWNER == data.from_id,
                Label::raw()
                    .with_line_break_mode(LineBreaking::WordWrap)
                    .with_text_size(theme::TEXT_SIZE_SMALL)
                    .lens(lens::Identity.map(
                        // Expose shared data with children data
                        |data: &Message| data.message.as_ref().unwrap().to_owned(),
                        |_, _| {},
                    ))
                    .fix_width(60.)
                    .align_right()
                    // .with_text_alignment(TextAlignment::Start);
                    .padding(Insets::uniform_xy(theme::grid(2.0), theme::grid(0.6)))
                    // .link()
                    .on_click(|ctx, queue, _| {
                        // ctx.submit_command();
                    }),
                Label::raw()
                    .with_line_break_mode(LineBreaking::WordWrap)
                    .with_text_size(theme::TEXT_SIZE_SMALL)
                    .lens(lens::Identity.map(
                        // Expose shared data with children data
                        |data: &Message| data.message.as_ref().unwrap().to_owned(),
                        |_, _| {},
                    ))
                    .fix_width(60.)
                    .align_left()
                    .padding(Insets::uniform_xy(theme::grid(2.0), theme::grid(0.6)))
                    // .link()
                    .on_click(|ctx, queue, _| {
                        // ctx.submit_command();
                    }),
            )
        })
        .with_spacing(10.),
    )
    .vertical()
    // .fix_height(300.0)
    // .expand_height()
    .lens(lens::Identity.map(
        // Expose shared data with children data
        |data: &AppData| {
            // owner = data.user_id;
            let channel_id = data.current_channel.as_ref().unwrap();
            let messages = data.channels.get(&channel_id).unwrap().messages.clone();
            messages
        },
        |_, _| {},
    ));
    playlist
}
