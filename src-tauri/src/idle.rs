//! Time since the last keyboard / mouse input, from the OS.

use std::time::Duration;

#[cfg(windows)]
pub fn idle_time() -> Duration {
    use windows::Win32::System::SystemInformation::GetTickCount;
    use windows::Win32::UI::Input::KeyboardAndMouse::{GetLastInputInfo, LASTINPUTINFO};

    let mut info = LASTINPUTINFO {
        cbSize: size_of::<LASTINPUTINFO>() as u32,
        dwTime: 0,
    };
    // SAFETY: `info` is a valid LASTINPUTINFO with cbSize set.
    if !unsafe { GetLastInputInfo(&mut info) }.as_bool() {
        return Duration::ZERO;
    }
    // Both are 32-bit millisecond tick counts that wrap after ~49.7 days.
    let now = unsafe { GetTickCount() };
    Duration::from_millis(now.wrapping_sub(info.dwTime).into())
}

#[cfg(target_os = "macos")]
pub fn idle_time() -> Duration {
    #[link(name = "CoreGraphics", kind = "framework")]
    unsafe extern "C" {
        fn CGEventSourceSecondsSinceLastEventType(state: i32, event_type: u32) -> f64;
    }
    const COMBINED_SESSION_STATE: i32 = 0;
    const ANY_INPUT_EVENT: u32 = u32::MAX;
    // SAFETY: plain C function taking and returning scalars.
    let secs =
        unsafe { CGEventSourceSecondsSinceLastEventType(COMBINED_SESSION_STATE, ANY_INPUT_EVENT) };
    Duration::try_from_secs_f64(secs).unwrap_or(Duration::ZERO)
}

/// No idle source on other platforms: the user always counts as active.
#[cfg(not(any(windows, target_os = "macos")))]
pub fn idle_time() -> Duration {
    Duration::ZERO
}
