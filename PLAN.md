# Kalk — Plan v1

> **Kalk** (Turkish for "get up!") — a tiny tray app that reminds you to do
> 10 bodyweight squats every 45 minutes of active computer use.

## Why

A 2024 study in the *Scandinavian Journal of Medicine & Science in Sports*
compared four ways of spending 8.5 hours of sitting: uninterrupted, one
30-minute walk, a 3-minute walk every 45 minutes, and 10 squats every
45 minutes. Frequent short breaks (squats **or** short walks — they performed
about the same) lowered blood-sugar response more than the single 30-minute
walk. Participants were 18 men with overweight or obesity, so the popup text
states the finding honestly and links the paper rather than overclaiming.

Kalk turns "squats every 45 minutes" into a habit with almost no friction.

## Identity

| Item | Value |
|---|---|
| Display name | Kalk |
| Repo | `github.com/besinci/kalk` |
| Bundle ID | `io.github.besinci.kalk` |
| License | MIT |
| Tagline | Kalk — 10 squats every 45 minutes |

Note: "Kalk" is also the name of several calculators (KDE Kalk, xoofx/kalk,
the Kalker Rust crate). Different category, no real conflict; the tagline and
repo topics carry discoverability.

## Stack

- **Tauri 2** (Rust) + **Svelte 5 + TypeScript**, built with Vite.
- Tauri features/plugins: `tray-icon`, `positioner` (anchor popup to tray),
  `autostart`, `store` (settings), `os` (system locale), `sql` (SQLite),
  `updater`.
- Idle detection: `user-idle` crate (native idle APIs on Windows/macOS).
- All data is local. No accounts, no telemetry.

## Timer logic (Rust)

- Background loop ticks every ~10 s, reads system idle time.
- Active time accumulates only while idle < threshold (default 10 min).
- Idle ≥ threshold → timer resets (user already got up).
- Sleep/lock detected via wall-clock jumps between ticks → reset.
- Outside active hours (default 09:00–18:00, Mon–Fri) the timer doesn't run.
- Active time ≥ interval (default 45 min) → reminder fires.

## Popup — one window, two modes

**Info mode** (user clicks the tray icon)
- Animated squatting character
- Short study summary + "Read the study" link
- "Next break in 23 min"
- Today's progress ("4 / 8 breaks")
- Stats + settings buttons
- Hides when it loses focus

**Reminder mode** (opens automatically)
- "Time for 10 squats!" + animation with rep counter
- Buttons: **Done**, **Snooze 5 min**, **Skip**
- Always on top, does **not** steal keyboard focus
- Stays open until answered
- If left unanswered and the user then goes idle past the threshold, the
  reminder is marked *pending*; when they return, the popup asks
  **"Did you do your squats?"** (Yes → done, No → missed)

Window is frameless, anchored to the tray icon. macOS: menu bar only, no Dock
icon. Tray tooltip shows the countdown. Right-click tray menu: Open, Pause 1 h,
Pause for today, Resume, Quit.

## Tracking

- Every reminder logged with timestamp and outcome:
  `done | skipped | snoozed | missed`.
- Stats view: today's count, current streak (days with ≥ N breaks done),
  7-day bar chart, total squats.

## Settings

- Interval (45 min), reps (10), idle threshold (10 min)
- Active hours and days
- Launch at login (on by default), sound on/off
- Language: System / English / Türkçe
- Pause 1 h / rest of day
- Reset stats

## i18n

- `svelte-i18n` with `en.json` and `tr.json`.
- Default = system locale: `tr-*` → Turkish, everything else → English.
- Tray menu labels (Rust side) use the same locale.

## Animation

- One original cartoon character, SVG + CSS keyframes.
- Sharp at any DPI, tiny, can sync to a "1… 2… 3…" rep counter.
- Respects `prefers-reduced-motion`.

## Distribution & CI

- Public GitHub repo, releases via GitHub Actions + `tauri-action` on tags.
- **macOS**: universal `.dmg` (Intel + Apple Silicon), signed with Developer
  ID and notarized. Secrets: `APPLE_CERTIFICATE`, `APPLE_CERTIFICATE_PASSWORD`,
  `APPLE_SIGNING_IDENTITY`, `APPLE_API_ISSUER`, `APPLE_API_KEY`,
  `APPLE_API_KEY_PATH` (or Apple ID + app-specific password + team ID).
- **Windows**: NSIS `.exe`. Unsigned in v1 (SmartScreen warning). Later:
  SignPath (free for OSS) or Azure Trusted Signing.
- Auto-update: `tauri-plugin-updater` reading `latest.json` from GitHub
  Releases, signed with a Tauri update key (`TAURI_SIGNING_PRIVATE_KEY`).

## Milestones

1. **Scaffold** — tray icon, popup anchored to tray, hide on blur, no Dock
   icon on macOS, tray right-click menu.
2. **Timer engine** — idle detection, reset, sleep handling, active hours;
   unit tests with a fake clock / fake idle source.
3. **Reminder mode** — auto-open without focus, Done/Snooze/Skip, "did you
   do them?" flow, SQLite logging.
4. **Info mode** — character animation, study text, countdown.
5. **Settings + i18n** (EN/TR, system default).
6. **Stats view.**
7. **Release** — CI, macOS signing + notarization, updater, README with
   screenshots.
