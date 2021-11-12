use crate::states::SignalState;
use druid::keyboard_types::Key;
use druid::widget::Controller;
use druid::{Env, Event, EventCtx, Widget};

pub struct MessageInputController;

impl<W> Controller<SignalState, W> for MessageInputController
    where
        W: Widget<SignalState>,
{
    fn event(
        &mut self,
        child: &mut W,
        ctx: &mut EventCtx,
        event: &Event,
        data: &mut SignalState,
        env: &Env,
    ) {
        match event {
            Event::KeyDown(key) if key.key == Key::Enter && !key.mods.shift() => {
                if !data.data.input.is_empty() {
                    /*                    // TODO: do this based on current cursor position
                    let count = data.editing_message.match_indices("```").count();
                    if count % 2 == 0 {
                        let formatted = markdown::parse_markdown(&*data.editing_message);
                        let formatted = markdown::markdown_to_html(formatted);
                        match data.txs.action_tx.try_send(UserAction::SendMessage(
                            data.current_channel.clone(),
                            data.editing_message.clone(),
                            Arc::new(formatted),
                        )) {
                            Ok(_) => (),
                            Err(TrySendError::Full(_)) => panic!("idk what to do here :("),
                            Err(TrySendError::Closed(_)) => panic!("oh no"),
                        }
                        data.editing_message = Arc::new(String::new());
                        ctx.set_handled();
                    }*/
                } else {
                    ctx.set_handled();
                }
            }

            _ => (),
        }
        child.event(ctx, event, data, env);
    }
}
