//! Store builds.
//!
//! Microsoft Store: the same kalk.exe wrapped in an MSIX package (see
//! scripts/msix.mjs). Inside the package the Store handles updates, and
//! launch at login goes through the manifest's StartupTask: a Run-key entry
//! written from inside a package is virtualized and never runs.
//!
//! Mac App Store: built with the `app-store` feature. The App Store handles
//! updates, and the sandbox rules out the LaunchAgent that
//! tauri-plugin-autostart writes, so launch at login goes through SMAppService.

/// Matches `TaskId` in the MSIX manifest.
#[cfg(windows)]
const STARTUP_TASK: &str = "KalkStartup";

/// Whether this copy came from a store, which then updates it.
pub fn packaged() -> bool {
    #[cfg(windows)]
    {
        use std::sync::OnceLock;
        static PACKAGED: OnceLock<bool> = OnceLock::new();
        *PACKAGED.get_or_init(|| windows::ApplicationModel::Package::Current().is_ok())
    }
    #[cfg(not(windows))]
    cfg!(feature = "app-store")
}

/// Turns the package's startup task on or off. Only call when `packaged()`.
#[cfg(windows)]
pub fn set_launch_at_login(enable: bool) -> windows::core::Result<()> {
    use windows::ApplicationModel::{StartupTask, StartupTaskState};
    use windows::core::HSTRING;

    let task = StartupTask::GetAsync(&HSTRING::from(STARTUP_TASK))?.join()?;
    if !enable {
        return task.Disable();
    }
    // Turned off by the user in Task Manager or Settings: only they can turn
    // it back on, so this stays off without an error.
    let state = task.RequestEnableAsync()?.join()?;
    if state == StartupTaskState::DisabledByUser {
        eprintln!("kalk: launch at login is turned off in Windows settings");
    }
    Ok(())
}

/// Adds or removes Kalk from the user's login items.
#[cfg(all(target_os = "macos", feature = "app-store"))]
pub fn set_launch_at_login(enable: bool) -> Result<(), String> {
    use objc2_service_management::{SMAppService, SMAppServiceStatus};
    // SAFETY: plain Objective-C calls on the app's own service object.
    unsafe {
        let service = SMAppService::mainAppService();
        if enable == (service.status() == SMAppServiceStatus::Enabled) {
            return Ok(());
        }
        let result = if enable {
            service.registerAndReturnError()
        } else {
            service.unregisterAndReturnError()
        };
        result.map_err(|e| e.localizedDescription().to_string())
    }
}
