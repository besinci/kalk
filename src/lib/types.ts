// Mirrors the structs the Rust side sends (src-tauri/src/breaks.rs).

import type { Lang } from "./i18n.svelte";

export type Phase = "counting" | "paused" | "off_hours" | "reminding" | "asking";
export type Outcome = "done" | "skipped" | "snoozed" | "missed";

export interface Status {
  phase: Phase;
  secs_left: number;
  /** Local time, ISO 8601. */
  paused_until: string | null;
  done_today: number;
  daily_goal: number;
  reps: number;
  snooze_secs: number;
}

export interface Settings {
  interval_mins: number;
  reps: number;
  daily_goal: number;
  idle_reset: boolean;
  idle_mins: number;
  snooze_mins: number;
  active_hours: boolean;
  /** "HH:MM" */
  start: string;
  end: string;
  /** 0 = Monday … 6 = Sunday. */
  days: number[];
  launch_at_login: boolean;
  sound: boolean;
  language: "system" | Lang;
  theme: "system" | "light" | "dark";
}

export interface Stats {
  today: number;
  goal: number;
  streak: number;
  /** Oldest first, ending today. `date` is "YYYY-MM-DD". */
  week: { date: string; done: number }[];
  total_squats: number;
}

export interface SettingsView {
  settings: Settings;
  lang: Lang;
  dev: boolean;
}
