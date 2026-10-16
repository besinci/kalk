# Kalk

**Kalk** (Turkish for *"get up!"*) sits in your tray / menu bar and reminds you
to do **10 bodyweight squats after every 45 minutes** of active computer use.

> A 2024 study ([Scand J Med Sci Sports, doi:10.1111/sms.14628](https://doi.org/10.1111/sms.14628))
> found that breaking up 8.5 hours of sitting with 10 squats — or a 3‑minute
> walk — every 45 minutes lowered blood‑sugar response more than a single
> 30‑minute walk.

Windows and macOS. Everything stays on your machine — no accounts, no telemetry.

**Status:** early development — see [PLAN.md](PLAN.md) for the roadmap.

## Development

Requirements: [Rust](https://rustup.rs) (stable), Node.js 20+, and the
[Tauri prerequisites](https://tauri.app/start/prerequisites/) for your OS.

```sh
npm install
npm run tauri dev          # run with hot reload; click the tray icon
KALK_SHOW=1 npm run tauri dev   # open the popup immediately (debug builds)
npm run tauri build        # release build + installer
```

The tray popup hides when it loses focus or on <kbd>Esc</kbd>. Quit from the
popup or the tray icon's right‑click menu.

### Layout

```
src/                 Svelte popup UI (SvelteKit SPA, adapter-static)
src-tauri/src/
  lib.rs             app setup, plugins, window/run events
  tray.rs            tray icon + right-click menu
  popup.rs           popup show/hide/toggle and positioning
assets/              icon sources (make_icons.py regenerates them)
```

Icons: `python3 assets/make_icons.py assets && npx tauri icon assets/app-icon.png`
then copy `assets/tray-*.png` into `src-tauri/icons/`.

## License

MIT
