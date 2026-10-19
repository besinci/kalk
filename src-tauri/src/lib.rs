mod breaks;
mod db;
mod i18n;
mod idle;
mod popup;
mod settings;
mod sound;
mod stats;
mod store;
mod timer;
mod tray;
mod update;

use tauri::{Manager, RunEvent, WindowEvent};

#[tauri::command]
fn quit_app(app: tauri::AppHandle) {
    app.exit(0);
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        // A second launch just opens the existing popup instead of adding a
        // second tray icon.
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            popup::show(app);
        }))
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_positioner::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .plugin(tauri_plugin_updater::Builder::new().build())
        .manage(popup::PopupState::default())
        .setup(|app| {
            // Menu-bar only: no Dock icon, no app menu.
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            tray::build(app.handle())?;
            breaks::start(app.handle());
            update::start(app.handle());

            // Dev convenience: `KALK_SHOW=1 npm run tauri dev` opens the popup
            // immediately instead of waiting for a tray click.
            #[cfg(debug_assertions)]
            if std::env::var_os("KALK_SHOW").is_some() {
                popup::show(app.handle());
            }
            Ok(())
        })
        .on_window_event(|window, event| {
            if window.label() != popup::LABEL {
                return;
            }
            match event {
                WindowEvent::Focused(false) => popup::on_blur(window.app_handle()),
                WindowEvent::Moved(pos) => popup::on_moved(window.app_handle(), *pos),
                // Alt+F4 / Cmd+W hide the popup; only "Quit" exits.
                WindowEvent::CloseRequested { api, .. } => {
                    api.prevent_close();
                    popup::hide(window.app_handle());
                }
                _ => {}
            }
        })
        .invoke_handler(tauri::generate_handler![
            popup::hide_popup,
            popup::set_popup_pinned,
            popup::reset_popup_position,
            quit_app,
            breaks::get_status,
            breaks::answer,
            breaks::resume_breaks,
            breaks::get_settings,
            breaks::set_settings,
            breaks::get_stats,
            breaks::reset_stats,
            update::get_update,
            update::install_update,
        ])
        .build(tauri::generate_context!())
        .expect("error while building Kalk")
        .run(|_app, event| {
            // Keep running in the tray when no window is visible; an explicit
            // app.exit(code) carries a code and is allowed through.
            if let RunEvent::ExitRequested {
                api, code: None, ..
            } = event
            {
                api.prevent_exit();
            }
        });
}
