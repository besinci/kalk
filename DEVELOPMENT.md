# Developing Kalk

Requirements: [Rust](https://rustup.rs) (stable), Node.js 20+, and the
[Tauri prerequisites](https://tauri.app/start/prerequisites/) for your OS.

```sh
npm install
npm run tauri dev               # run with hot reload; click the tray icon
KALK_SHOW=1 npm run tauri dev   # open the popup immediately
npm run tauri build             # release build + installer
npm run check                   # type-check the UI
npm run screenshots             # regenerate docs/ and Store screenshots (needs Edge or Chrome)
cd src-tauri && cargo test      # timer, log and stats tests
```

**Debug builds run fast:** every minute in the settings counts as a second (a
reminder after 45 s, snooze 5 s), the idle reset is at least 30 s, active
hours are ignored and launch at login is left alone. Release builds use real
minutes.

## Layout

```
src/
  routes/+page.svelte   the popup: info, reminder, settings and stats views
  lib/Squatter.svelte   the animated character
  lib/Settings.svelte   settings view
  lib/Stats.svelte      stats view
  lib/i18n.svelte.ts    popup strings (en, tr)
src-tauri/src/
  lib.rs                app setup, plugins, window/run events
  timer.rs              the break timer: pure logic, fake-clock tests
  breaks.rs             runs the timer; commands for the popup; tray tooltip
  idle.rs               system idle time (Windows / macOS)
  settings.rs           settings.json
  db.rs                 SQLite break log
  stats.rs              streak and 7-day stats
  popup.rs              popup show/hide, reminder mode, positioning
  store.rs              Microsoft Store / Mac App Store differences
  tray.rs               tray icon + right-click menu
  i18n.rs, sound.rs     tray strings, reminder chime
assets/                 icon sources
```

## Releasing

```sh
node scripts/bump-version.mjs 0.2.0     # package.json, Cargo.toml, tauri.conf.json
(cd src-tauri && cargo check)           # updates Cargo.lock
git commit -am "Release 0.2.0" && git tag v0.2.0 && git push origin main v0.2.0
```

The tag runs the Release workflow, which builds both platforms into a
**draft** release with the updater files (`latest.json` and signatures).
Review it and publish; installed copies offer the update within a few hours.

Secrets the workflow uses:

| Secret | What |
|---|---|
| `TAURI_SIGNING_PRIVATE_KEY` | updater signing key (`~/.tauri/kalk-updater.key`; keep a backup — losing it means existing installs can't verify new updates) |
| `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` | optional; the key has no password |
| `APPLE_CERTIFICATE`, `APPLE_CERTIFICATE_PASSWORD`, `APPLE_SIGNING_IDENTITY` | Developer ID Application certificate (`.p12`, base64) for macOS signing |
| `APPLE_API_ISSUER`, `APPLE_API_KEY`, `APPLE_API_KEY_P8` | App Store Connect API key for notarization |

Without the Apple secrets the macOS build is unsigned.

**Microsoft Store:** the Release workflow also uploads a `kalk-msix` artifact
(built by `scripts/msix.mjs`). Submit it as a new package in Partner Center,
with the images from `npm run screenshots` (`store/screenshots/`) and the texts
in `store/listing.md`.

**Mac App Store:** run the **App Store** workflow by hand (Actions tab) after
a release. It builds with the `app-store` feature and
`src-tauri/tauri.appstore.conf.json` (sandboxed, macOS 13+, no self-updater,
launch at login through SMAppService and off by default), signs a `.pkg` and
uploads it to App Store Connect, where it appears under TestFlight. Listing
texts are in `store/appstore.md`, screenshots in `store/appstore/`. Extra
secrets it uses:

| Secret | What |
|---|---|
| `APPSTORE_CERTIFICATE` | Apple Distribution certificate (`.p12`, base64) |
| `APPSTORE_INSTALLER_CERTIFICATE` | Mac Installer Distribution certificate (`.p12`, base64) |
| `APPSTORE_CERTIFICATE_PASSWORD` | password of both `.p12` files |
| `APPSTORE_PROVISIONING_PROFILE` | Mac App Store provisioning profile for `io.github.besinci.kalk` (base64) |

It uploads with the same `APPLE_API_*` key as notarization.

Icons: `python3 assets/make_icons.py assets && npx tauri icon assets/app-icon.png`
then copy `assets/tray-*.png` into `src-tauri/icons/`.
