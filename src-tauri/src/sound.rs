//! The reminder chime: the system's own notification sound.

#[cfg(windows)]
pub fn chime() {
    use windows::Win32::Media::Audio::{PlaySoundW, SND_ALIAS, SND_ASYNC, SND_NODEFAULT};
    use windows::core::w;
    // SAFETY: a static, NUL-terminated alias name; SND_ASYNC returns at once.
    let _ = unsafe {
        PlaySoundW(
            w!("Notification.Default"),
            None,
            SND_ALIAS | SND_ASYNC | SND_NODEFAULT,
        )
    };
}

/// NSSound rather than running `afplay`, which the App Store sandbox may not
/// allow. Sounds looked up by name stay cached, so it plays to the end.
#[cfg(target_os = "macos")]
pub fn chime() {
    use objc2_app_kit::NSSound;
    use objc2_foundation::NSString;
    if let Some(sound) = NSSound::soundNamed(&NSString::from_str("Glass")) {
        sound.play();
    }
}

#[cfg(not(any(windows, target_os = "macos")))]
pub fn chime() {}
