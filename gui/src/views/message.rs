use crate::states::Message;
use crate::states::SignalData;
use crate::states::OWNER;
use crate::views::style;
use druid::im::Vector;
use druid::widget::{Either, Label, LineBreaking, List, Scroll};
use druid::{lens, Insets, LensExt, Widget, WidgetExt};
pub fn make_message_list() -> impl Widget<SignalData> {
    // let mut owner = Uuid::default();
    let message_list = Scroll::new(
        List::new(|| unsafe {
            Either::new(
                |data: &Message, _| OWNER == data.from_id,
                Label::raw()
                    .with_line_break_mode(LineBreaking::WordWrap)
                    .with_text_size(style::TEXT_SIZE_SMALL)
                    .lens(lens::Identity.map(
                        // Expose shared data with children data
                        |data: &Message| data.message.as_ref().unwrap().to_owned(),
                        |_, _| {},
                    ))
                    .fix_width(60.)
                    .align_right()
                    // .with_text_alignment(TextAlignment::Start);
                    .padding(Insets::uniform_xy(style::grid(2.0), style::grid(0.6)))
                    // .link()
                    .on_click(|ctx, queue, _| {
                        // ctx.submit_command();
                    }),
                Label::raw()
                    .with_line_break_mode(LineBreaking::WordWrap)
                    .with_text_size(style::TEXT_SIZE_SMALL)
                    .lens(lens::Identity.map(
                        // Expose shared data with children data
                        |data: &Message| data.message.as_ref().unwrap().to_owned(),
                        |_, _| {},
                    ))
                    .fix_width(60.)
                    .align_left()
                    .padding(Insets::uniform_xy(style::grid(2.0), style::grid(0.6)))
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
        |data: &SignalData| {
            // owner = data.user_id;
            match data.current_channel.as_ref() {
                Some(channel_id) => {
                    if let Some(channel) = data.channels.get(&channel_id) {
                        channel.messages.clone()
                    } else {
                        Vector::new()
                    }
                }
                None => Vector::new(),
            }
        },
        |_, _| {},
    ));
    message_list
}
