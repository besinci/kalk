//! Tray / menu-bar icon. Left click toggles the popup, right click opens the menu.

use tauri::image::Image;
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager, Runtime};

use crate::breaks::{self, Pause};
use crate::i18n::{self, Key};
use crate::popup;

pub const ID: &str = "kalk";

/// Menu items that get relabelled or enabled after the menu is built.
struct Items<R: Runtime> {
    open: MenuItem<R>,
    hour: MenuItem<R>,
    today: MenuItem<R>,
    resume: MenuItem<R>,
    quit: MenuItem<R>,
}

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
    // Labels are set by `set_language` once settings are loaded.
    let open = MenuItem::with_id(app, "open", "Open Kalk", true, None::<&str>)?;
    let hour = MenuItem::with_id(app, "pause_hour", "Pause for 1 hour", true, None::<&str>)?;
    let today = MenuItem::with_id(app, "pause_today", "Pause for today", true, None::<&str>)?;
    let resume = MenuItem::with_id(app, "resume", "Resume", false, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit Kalk", true, None::<&str>)?;
    let menu = Menu::with_items(
        app,
        &[
            &open,
            &PredefinedMenuItem::separator(app)?,
            &hour,
            &today,
            &resume,
            &PredefinedMenuItem::separator(app)?,
            &quit,
        ],
    )?;
    app.manage(Items {
        open,
        hour,
        today,
        resume,
        quit,
    });

    TrayIconBuilder::with_id(ID)
        .icon(icon()?)
        .icon_as_template(cfg!(target_os = "macos"))
        .tooltip("Kalk")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "open" => popup::show(app),
            "pause_hour" => breaks::pause(app, Pause::Hour),
            "pause_today" => breaks::pause(app, Pause::Today),
            "resume" => breaks::resume(app),
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

pub fn sync_menu<R: Runtime>(app: &AppHandle<R>, paused: bool) {
    let Some(items) = app.try_state::<Items<R>>() else {
        return;
    };
    let _ = items.hour.set_enabled(!paused);
    let _ = items.today.set_enabled(!paused);
    let _ = items.resume.set_enabled(paused);
}

pub fn set_tooltip<R: Runtime>(app: &AppHandle<R>, text: &str) {
    if let Some(tray) = app.tray_by_id(ID) {
        let _ = tray.set_tooltip(Some(text));
    }
}

pub fn set_language<R: Runtime>(app: &AppHandle<R>, lang: &str) {
    let Some(items) = app.try_state::<Items<R>>() else {
        return;
    };
    for (item, key) in [
        (&items.open, Key::Open),
        (&items.hour, Key::PauseHour),
        (&items.today, Key::PauseToday),
        (&items.resume, Key::Resume),
        (&items.quit, Key::Quit),
    ] {
        let _ = item.set_text(i18n::t(lang, key));
    }
}
