//! Tray / menu-bar icon. Left click toggles the popup, right click opens the menu.

use tauri::image::Image;
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Runtime};

use crate::popup;

pub const ID: &str = "kalk";

fn icon() -> tauri::Result<Image<'static>> {
    // macOS: monochrome template image, recoloured by the menu bar.
    // Windows: full-colour icon, readable on light and dark taskbars.
    #[cfg(target_os = "macos")]
    let bytes = include_bytes!("../icons/tray-template.png");
    #[cfg(not(target_os = "macos"))]
    let bytes = include_bytes!("../icons/tray-color.png");
    Image::from_bytes(bytes)
}

pub fn build<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    // TODO(milestone 5): localise labels; (milestone 2): pause / resume items.
    let open = MenuItem::with_id(app, "open", "Open Kalk", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit Kalk", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&open, &PredefinedMenuItem::separator(app)?, &quit])?;

    TrayIconBuilder::with_id(ID)
        .icon(icon()?)
        .icon_as_template(cfg!(target_os = "macos"))
        .tooltip("Kalk")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "open" => popup::show(app),
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            let app = tray.app_handle();
            // Lets the positioner learn where the icon is on screen.
            tauri_plugin_positioner::on_tray_event(app, &event);
            popup::mark_tray_located(app);

            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                popup::toggle(app);
            }
        })
        .build(app)?;
    Ok(())
}
