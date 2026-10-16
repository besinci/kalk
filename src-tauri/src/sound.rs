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

#[cfg(target_os = "macos")]
pub fn chime() {
    // Waited on in a thread so the finished process doesn't linger.
    std::thread::spawn(|| {
        let _ = std::process::Command::new("afplay")
            .arg("/System/Library/Sounds/Glass.aiff")
            .status();
    });
}

#[cfg(not(any(windows, target_os = "macos")))]
pub fn chime() {}
