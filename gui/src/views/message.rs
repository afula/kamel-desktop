use crate::states::Message;
use crate::states::SignalData;
use crate::states::OWNER;
use crate::views::style;
use crate::widgets::dynamic_sized_box::DynamicSizedBox;
use druid::im::Vector;
use druid::widget::{Either, Label, LineBreaking, List, Scroll};
use druid::{lens, Color, Insets, LensExt, TextAlignment, UnitPoint, Widget, WidgetExt};
pub fn make_message_list() -> impl Widget<SignalData> {
    // let mut owner = Uuid::default();
    let message_list = Scroll::new(
        List::new(|| unsafe {
            Either::new(
                |data: &Message, _| OWNER == data.from_id,
                {
                    let dynamic_label = Label::raw()
                        .with_line_break_mode(LineBreaking::WordWrap)
                        .with_text_size(style::TEXT_SIZE_MEDIUM)
                        .with_text_alignment(TextAlignment::Start)
                        .padding(Insets::new(5.0 * 2.0, 5.0 * 2.0, 5.0 * 2.0, 5.0 * 2.0))
                        .lens(lens::Identity.map(
                            // Expose shared data with children data
                            |data: &Message| data.message.as_ref().unwrap().to_owned(),
                            |_, _| {},
                        ))
                        .align_vertical(UnitPoint::LEFT)
                        .background(Color::rgb8(66, 245, 188))
                        .rounded(10.0)
                        // .fix_width(120.)
                        // .align_right()
                        // .expand()
                        // .padding(Insets::uniform_xy(style::grid(2.0), style::grid(0.6)))
                        // .background(Color::rgb(0.5, 0.5, 0.5))
                        // .rounded(15.0)
                        .on_click(|ctx, queue, _| {
                            // ctx.submit_command();
                        });
                    let dynamic_box = DynamicSizedBox::new(dynamic_label)
                        // widget will be half the height of the box constraints
                        // the parent gives
                        // .with_height(0.5)
                        // widget will be 1/3 the width of its parent given box constraints
                        .with_width(0.33)
                        .align_right();
                    // .rounded(15.0);
                    // .background(Color::WHITE);
                    dynamic_box
                },
                {
                    let dynamic_label = Label::raw()
                        .with_line_break_mode(LineBreaking::WordWrap)
                        // .with_text_alignment(TextAlignment::ce)
                        .with_text_size(style::TEXT_SIZE_SMALL)
                        .lens(lens::Identity.map(
                            // Expose shared data with children data
                            |data: &Message| data.message.as_ref().unwrap().to_owned(),
                            |_, _| {},
                        ))
                        .align_vertical(UnitPoint::LEFT)
                        .background(Color::rgb8(128, 194, 164))
                        .rounded(10.0)
                        // .fix_width(120.)
                        // .expand()
                        // .align_left()
                        // .padding(Insets::uniform_xy(style::grid(2.0), style::grid(0.6)))
                        // .background(Color::rgb(0.5, 0.5, 0.5))
                        // .rounded(15.0)
                        .on_click(|ctx, queue, _| {
                            // ctx.submit_command();
                        });
                    let dynamic_box = DynamicSizedBox::new(dynamic_label)
                        // widget will be half the height of the box constraints
                        // the parent gives
                        // .with_height(0.5)
                        // widget will be 1/3 the width of its parent given box constraints
                        .with_width(0.33)
                        // .rounded(15.0)
                        .align_left();
                    // .background(Color::WHITE);
                    dynamic_box
                },
            )
        })
        .with_spacing(10.),
    )
    .vertical()
    .padding(Insets::new(0.0, 0.0, 10.0, 5.0))
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
