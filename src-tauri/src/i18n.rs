//! Strings shown by the Rust side (tray menu and tooltip). The popup has its
//! own copy in src/lib/i18n.ts.

#[derive(Clone, Copy)]
pub enum Key {
    Open,
    PauseHour,
    PauseToday,
    Resume,
    Quit,
}

pub fn t(lang: &str, key: Key) -> &'static str {
    let tr = lang == "tr";
    match key {
        Key::Open if tr => "Kalk'ı aç",
        Key::Open => "Open Kalk",
        Key::PauseHour if tr => "1 saat duraklat",
        Key::PauseHour => "Pause for 1 hour",
        Key::PauseToday if tr => "Bugünlük duraklat",
        Key::PauseToday => "Pause for today",
        Key::Resume if tr => "Devam et",
        Key::Resume => "Resume",
        Key::Quit if tr => "Kalk'tan çık",
        Key::Quit => "Quit Kalk",
    }
}

pub fn next_break_in(lang: &str, left: &str) -> String {
    match lang {
        "tr" => format!("Kalk — sonraki mola: {left}"),
        _ => format!("Kalk — next break in {left}"),
    }
}

pub fn paused_until(lang: &str, time: &str) -> String {
    match lang {
        "tr" => format!("Kalk — {time} saatine kadar duraklatıldı"),
        _ => format!("Kalk — paused until {time}"),
    }
}

pub fn off_hours(lang: &str) -> String {
    match lang {
        "tr" => "Kalk — aktif saatlerin dışında".into(),
        _ => "Kalk — outside active hours".into(),
    }
}

pub fn time_for_squats(lang: &str, reps: u32) -> String {
    match lang {
        "tr" => format!("Kalk — {reps} squat zamanı!"),
        _ => format!("Kalk — time for {reps} squats!"),
    }
}

/// "23 min" / "23 dk", or "0:42" under a minute.
pub fn duration(lang: &str, secs: u64) -> String {
    if secs >= 60 {
        let unit = if lang == "tr" { "dk" } else { "min" };
        format!("{} {unit}", secs.div_ceil(60))
    } else {
        format!("0:{secs:02}")
    }
}
