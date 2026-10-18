//! The Microsoft Store build is the same kalk.exe wrapped in an MSIX package
//! (see scripts/msix.mjs). Inside the package the Store handles updates, and
//! launch at login goes through the manifest's StartupTask: a Run-key entry
//! written from inside a package is virtualized and never runs.

/// Matches `TaskId` in the MSIX manifest.
#[cfg(windows)]
const STARTUP_TASK: &str = "KalkStartup";

/// Whether this copy runs from an MSIX package, i.e. came from the Store.
pub fn packaged() -> bool {
    #[cfg(windows)]
    {
        use std::sync::OnceLock;
        static PACKAGED: OnceLock<bool> = OnceLock::new();
        *PACKAGED.get_or_init(|| windows::ApplicationModel::Package::Current().is_ok())
    }
    #[cfg(not(windows))]
    false
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
