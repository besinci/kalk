// Popup strings. The tray menu and tooltip have their own copy in
// src-tauri/src/i18n.rs.

const en = {
  settings: "Settings",
  close: "Close (Esc)",
  pin: "Keep open",
  unpin: "Close when clicking outside",
  back: "Back",
  quit: "Quit Kalk",
  updateAvailable: "Kalk {v} is available",
  updateNow: "Update",
  updating: "Updating…",

  nextBreakIn: "Next squat break in",
  pausedUntil: "Paused until",
  resumeNow: "Resume now",
  offHours: "Outside active hours",
  off: "Off",
  doneToday: "Today: {n} / {goal} breaks",
  whyTitle: "10 squats every 45 minutes",
  whyBody: "Better for blood sugar than one 30‑minute walk.",
  readStudy: "Read the study →",

  remindTitle: "Time for {reps} squats!",
  remindBody: "Stand up, squat {reps} times, sit back down.",
  done: "Done",
  snooze: "Snooze {d}",
  skip: "Skip",
  askTitle: "Welcome back!",
  askBody: "Did you do your {reps} squats?",
  yes: "Yes",
  no: "No",

  interval: "Break every",
  reps: "Squats per break",
  dailyGoal: "Daily goal",
  breaks: "breaks",

  stats: "Stats",
  today: "Today",
  streak: "Streak",
  streakDays: "{n} days",
  totalSquats: "Total squats",
  last7: "Last 7 days",
  goal: "Goal {n}",
  dayBreaks: "{day}: {n} breaks",
  idleReset: "Restart the countdown when I'm away",
  idleAfter: "Away after",
  snoozeFor: "Snooze for",
  activeHours: "Only during active hours",
  from: "From",
  to: "to",
  launchAtLogin: "Launch at login",
  sound: "Sound",
  language: "Language",
  system: "System",
  lightTheme: "Light theme",
  darkTheme: "Dark theme",
  breaksSection: "Breaks",
  activeHoursSection: "Schedule",
  general: "General",
  autoSaved: "Changes are saved automatically",
  saved: "Saved",
  resetStats: "Reset stats",
  resetConfirm: "Delete all logged breaks? Click again to confirm.",
  devNote: "Dev build: minutes run as seconds and active hours are ignored.",
  min: "min",
  sec: "s",
};

type Strings = typeof en;

const tr: Strings = {
  settings: "Ayarlar",
  close: "Kapat (Esc)",
  pin: "Açık tut",
  unpin: "Dışarı tıklayınca kapat",
  back: "Geri",
  quit: "Kalk'tan çık",
  updateAvailable: "Kalk {v} hazır",
  updateNow: "Güncelle",
  updating: "Güncelleniyor…",

  nextBreakIn: "Sonraki squat molasına",
  pausedUntil: "Duraklatma bitişi",
  resumeNow: "Şimdi devam et",
  offHours: "Aktif saatlerin dışında",
  off: "Kapalı",
  doneToday: "Bugün: {n} / {goal} mola",
  whyTitle: "Her 45 dakikada 10 squat",
  whyBody: "Kan şekeri için tek bir 30 dakikalık yürüyüşten daha iyi.",
  readStudy: "Çalışmayı oku →",

  remindTitle: "{reps} squat zamanı!",
  remindBody: "Kalk, {reps} squat yap, otur.",
  done: "Yaptım",
  snooze: "{d} ertele",
  skip: "Atla",
  askTitle: "Tekrar hoş geldin!",
  askBody: "{reps} squat'ı yaptın mı?",
  yes: "Evet",
  no: "Hayır",

  interval: "Mola aralığı",
  reps: "Mola başına squat",
  dailyGoal: "Günlük hedef",
  breaks: "mola",

  stats: "İstatistikler",
  today: "Bugün",
  streak: "Seri",
  streakDays: "{n} gün",
  totalSquats: "Toplam squat",
  last7: "Son 7 gün",
  goal: "Hedef {n}",
  dayBreaks: "{day}: {n} mola",
  idleReset: "Uzaklaşınca sayacı baştan başlat",
  idleAfter: "Uzaklaşma süresi",
  snoozeFor: "Erteleme süresi",
  activeHours: "Yalnızca aktif saatlerde",
  from: "Başlangıç",
  to: "bitiş",
  launchAtLogin: "Oturum açılınca başlat",
  sound: "Ses",
  language: "Dil",
  system: "Sistem",
  lightTheme: "Açık tema",
  darkTheme: "Koyu tema",
  breaksSection: "Molalar",
  activeHoursSection: "Zamanlama",
  general: "Genel",
  autoSaved: "Değişiklikler otomatik kaydedilir",
  saved: "Kaydedildi",
  resetStats: "İstatistikleri sıfırla",
  resetConfirm: "Tüm kayıtlı molalar silinsin mi? Onaylamak için tekrar tıkla.",
  devNote: "Geliştirme sürümü: dakikalar saniye olarak çalışır, aktif saatler yok sayılır.",
  min: "dk",
  sec: "sn",
};

const strings = { en, tr };
export type Lang = keyof typeof strings;

let current = $state<Lang>("en");

export function setLang(lang: Lang) {
  current = lang;
  document.documentElement.lang = lang;
}

export function lang(): Lang {
  return current;
}

/** Looks up a string and fills in `{name}` placeholders. */
export function t(key: keyof Strings, params: Record<string, string | number> = {}) {
  return strings[current][key].replace(/\{(\w+)\}/g, (_, name) => String(params[name] ?? ""));
}

/** "23 min" / "23 dk", or "0:42" under a minute. */
export function fmtLeft(secs: number) {
  return secs >= 60 ? `${Math.ceil(secs / 60)} ${t("min")}` : `0:${String(secs).padStart(2, "0")}`;
}

/** "5 min" / "15 s". */
export function fmtSpan(secs: number) {
  return secs >= 60 ? `${Math.round(secs / 60)} ${t("min")}` : `${secs} ${t("sec")}`;
}

export function fmtClock(iso: string) {
  return new Date(iso).toLocaleTimeString(current, { hour: "2-digit", minute: "2-digit" });
}

/** "Fri 16 Oct" from "2026-10-16". */
export function fmtDay(date: string) {
  const [y, m, d] = date.split("-").map(Number);
  return new Date(y, m - 1, d).toLocaleDateString(current, {
    weekday: "short",
    day: "numeric",
    month: "short",
  });
}

/** Short weekday name from "2026-10-16". */
export function weekdayOf(date: string) {
  const [y, m, d] = date.split("-").map(Number);
  return new Date(y, m - 1, d).toLocaleDateString(current, { weekday: "short" });
}

/** Short weekday name; 0 = Monday. */
export function weekday(day: number) {
  // 2026-10-12 was a Monday.
  return new Date(2026, 9, 12 + day).toLocaleDateString(current, { weekday: "short" });
}
