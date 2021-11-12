use crate::states::SignalState;
use druid::commands;
use druid::platform_menus;
use druid::{
    Data, Env, FileDialogOptions, FileSpec, KbKey, LocalizedString, Menu, MenuItem, Point, SysMods,
    WindowId,
};

pub fn make_menu(_window: Option<WindowId>, data: &SignalState, _: &Env) -> Menu<SignalState> {
    let menu = if cfg!(target_os = "macos") {
        Menu::empty().entry(platform_menus::mac::application::default())
    } else {
        Menu::empty()
    };

    menu
        // .entry(file_menu(data))
        .entry(edit_menu())
        .entry(view_menu())
}

fn edit_menu<T: Data>() -> Menu<T> {
    Menu::new(LocalizedString::new("common-menu-edit-menu"))
        .entry(platform_menus::common::undo())
        .entry(platform_menus::common::redo())
        .separator()
        .entry(platform_menus::common::cut().enabled(false))
        .entry(platform_menus::common::copy())
        .entry(platform_menus::common::paste())
        .entry(
            MenuItem::new(LocalizedString::new("menu-item-delete").with_placeholder("Delete")), // .on_activate(|ctx, _, _| ctx.submit_command(command::DELETE)),
        )
        .separator()
        .entry(
            MenuItem::new(
                LocalizedString::new("menu-item-select-all").with_placeholder("Select All"),
            )
            // .on_activate(|ctx, _, _| ctx.submit_command(command::SELECT_ALL))
            .hotkey(SysMods::Cmd, "a"),
        )
        .entry(
            MenuItem::new(
                LocalizedString::new("menu-item-deselect-all").with_placeholder("Deselect All"),
            )
            // .on_activate(|ctx, _, _| ctx.submit_command(command::DESELECT_ALL))
            .hotkey(SysMods::AltCmd, "A"),
        )
}

fn view_menu<T: Data>() -> Menu<T> {
    Menu::new(LocalizedString::new("menu-view-menu").with_placeholder("View"))
        .entry(
            MenuItem::new(
                LocalizedString::new("menu-item-increase-zoom").with_placeholder("Zoom In"),
            )
            // .on_activate(|ctx, _, _| ctx.submit_command(command::ZOOM_IN))
            .hotkey(SysMods::Cmd, "+"),
        )
        .entry(
            MenuItem::new(
                LocalizedString::new("menu-item-decrease-zoom").with_placeholder("Zoom Out"),
            )
            // .on_activate(|ctx, _, _| ctx.submit_command(command::ZOOM_OUT))
            .hotkey(SysMods::Cmd, "-"),
        )
        .entry(
            MenuItem::new(
                LocalizedString::new("menu-item-reset-zoom").with_placeholder("Reset Zoom"),
            )
            // .on_activate(|ctx, _, _| ctx.submit_command(command::ZOOM_DEFAULT))
            .hotkey(SysMods::Cmd, "0"),
        )
}
/*fn file_menu(data: &SignalState) -> Menu<SignalState> {
    let has_path = data.workspace.font.path.is_some();
    let mut menu = Menu::new(LocalizedString::new("common-menu-file-menu"))
        .entry(platform_menus::mac::file::new_file().enabled(false))
        .entry(
            MenuItem::new(LocalizedString::new("common-menu-file-open"))
                .on_activate(|ctx, _, _| {
                    ctx.submit_command(
                        commands::SHOW_OPEN_PANEL
                            .with(FileDialogOptions::new().allowed_types(vec![UFO_FILE_TYPE])),
                    )
                })
                .hotkey(SysMods::Cmd, "o"),
        )
        .separator()
        .entry(platform_menus::mac::file::close());
    if has_path {
        menu = menu.entry(platform_menus::mac::file::save()).entry(
            MenuItem::new(LocalizedString::new("common-menu-file-save-as"))
                .on_activate(|ctx, _, _| {
                    ctx.submit_command(
                        commands::SHOW_SAVE_PANEL
                            .with(FileDialogOptions::new().allowed_types(vec![UFO_FILE_TYPE])),
                    )
                })
                .hotkey(SysMods::CmdShift, "S"),
        );
    } else {
        menu = menu.entry(
            MenuItem::new(LocalizedString::new("common-menu-file-save-as"))
                .on_activate(|ctx, _, _| {
                    ctx.submit_command(
                        commands::SHOW_SAVE_PANEL
                            .with(FileDialogOptions::new().allowed_types(vec![UFO_FILE_TYPE])),
                    )
                })
                .hotkey(SysMods::Cmd, "s"),
        );
    }
    menu.separator()
        .entry(platform_menus::mac::file::page_setup().enabled(false))
        .entry(platform_menus::mac::file::print().enabled(false))
}*/
