use druid::kurbo::Line;
use druid::lens::LensExt;
use druid::text::format::ParseFormatter;
use druid::widget::{prelude::*, Flex, Label, Painter, TextBox, WidgetExt};
use druid::{
    AppDelegate, Command, DelegateCtx, Handled, Selector, Target, Widget, WindowDesc, WindowId,
};
use std::sync::Arc;

use signal::AppData;

#[derive(Debug, Default)]
pub struct MainMenuDelegate;

impl AppDelegate<AppData> for MainMenuDelegate {
    fn command(
        &mut self,
        ctx: &mut DelegateCtx,
        _target: Target,
        cmd: &Command,
        data: &mut AppData,
        _env: &Env,
    ) -> Handled {
        if let Some(info) = cmd.get(druid::commands::OPEN_FILE) {
            /*            match Ufo::load(info.path()) {
                Ok(ufo) => data.workspace.set_file(ufo, info.path().to_owned()),
                Err(e) => log::error!("failed to open file {:?}: '{:?}'", info.path(), e),
            };*/
            Handled::Yes
        } else if cmd.is(druid::commands::SAVE_FILE) {
            /*            if let Err(e) = data.workspace.save() {
                log::error!("saving failed: '{}'", e);
            }*/
            Handled::Yes
        } else if let Some(info) = cmd.get(druid::commands::SAVE_FILE_AS) {
            /*            Arc::make_mut(&mut data.workspace.font).path = Some(info.path().into());
            if let Err(e) = data.workspace.save() {
                log::error!("saving failed: '{}'", e);
            }*/
            Handled::Yes
        }
        /*        else if let Some(payload) = cmd.get(EDIT_GLYPH) {
            match data.workspace.open_glyphs.get(payload).to_owned() {
                Some(id) => {
                    ctx.submit_command(druid::commands::SHOW_WINDOW.to(*id));
                }
                None => {
                    let session = data.workspace.get_or_create_session(payload);
                    let session_id = session.id;
                    let new_win = WindowDesc::new(make_editor(&session))
                        .title(move |d: &AppState, _: &_| {
                            d.workspace
                                .sessions
                                .get(&session_id)
                                .map(|s| s.name.to_string())
                                .unwrap_or_else(|| "Unknown".to_string())
                        })
                        .window_size(Size::new(900.0, 800.0))
                        .menu(crate::menus::make_menu);

                    let id = new_win.id;
                    ctx.new_window(new_win);

                    Arc::make_mut(&mut data.workspace.open_glyphs).insert(payload.clone(), id);
                }
            }
            Handled::Yes
        }*/
        else {
            Handled::No
        }
    }
}
