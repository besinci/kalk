// Renders the popup with mocked Tauri IPC and saves the screenshots:
//   docs/screenshots/        popup views for the README and site (340×480 @2x)
//   store/screenshots/<lang>/ 1920×1080 Microsoft Store images
// Usage: node scripts/screenshots.mjs
// Uses Edge or Chrome; set KALK_BROWSER to the executable to pick another.
import { existsSync, mkdirSync, writeFileSync } from "node:fs";
import path from "node:path";
import puppeteer from "puppeteer-core";
import { createServer } from "vite";

const BROWSERS = [
  process.env.KALK_BROWSER,
  "C:/Program Files (x86)/Microsoft/Edge/Application/msedge.exe",
  "C:/Program Files/Google/Chrome/Application/chrome.exe",
  "/Applications/Microsoft Edge.app/Contents/MacOS/Microsoft Edge",
  "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
].filter(Boolean);
const browserPath = BROWSERS.find((p) => existsSync(p));
if (!browserPath) {
  console.error("No Edge or Chrome found; set KALK_BROWSER to a Chromium executable.");
  process.exit(1);
}

// Sample data: a Friday afternoon a few days into a streak.
const settings = {
  interval_mins: 45, reps: 10, daily_goal: 8, idle_mins: 10, snooze_mins: 5,
  active_hours: true, start: "09:00", end: "18:00", days: [0, 1, 2, 3, 4],
  launch_at_login: true, sound: true, language: "system",
};
const counting = {
  phase: "counting", secs_left: 23 * 60, paused_until: null,
  done_today: 5, daily_goal: 8, reps: 10, snooze_secs: 300,
};
const reminding = { ...counting, phase: "reminding", secs_left: 0 };
const stats = {
  today: 5, goal: 8, streak: 4, total_squats: 1230,
  // Saturday to Friday.
  week: [0, 0, 8, 9, 8, 11, 5].map((done, i) => ({ date: `2026-10-${10 + i}`, done })),
};

const CAPTIONS = {
  en: {
    info: "Click the tray icon for your countdown",
    reminder: "Squat along when it's time",
    stats: "Watch your streak grow",
    settings: "Make it fit your day",
  },
  tr: {
    info: "Geri sayım için uygulama simgesine tıkla",
    reminder: "Zamanı gelince birlikte squat yap",
    stats: "Serinin büyüdüğünü izle",
    settings: "Gününe göre ayarla",
  },
};
const VIEW_LABELS = {
  settings: { en: "Settings", tr: "Ayarlar" },
  stats: { en: "Stats", tr: "İstatistikler" },
};

/** Runs in the page before the app: answers the app's invoke() calls. */
function mockTauri({ lang, status, settings, stats }) {
  let id = 0;
  window.__TAURI_EVENT_PLUGIN_INTERNALS__ = { unregisterListener() {} };
  window.__TAURI_INTERNALS__ = {
    metadata: { currentWindow: { label: "main" }, currentWebview: { windowLabel: "main", label: "main" } },
    transformCallback: () => ++id,
    convertFileSrc: (p) => p,
    invoke: async (cmd, args) => {
      switch (cmd) {
        case "get_settings": return { settings, lang, dev: false };
        case "set_settings": return { settings: args.settings, lang, dev: false };
        case "get_status": return status;
        case "get_stats": return stats;
        case "plugin:event|listen": return ++id;
        default: return null;
      }
    },
  };
}

const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

const server = await createServer({ server: { port: 1430, strictPort: false } });
await server.listen();
const url = server.resolvedUrls.local[0];
const browser = await puppeteer.launch({ executablePath: browserPath, headless: true });

/** Screenshot of one popup view, as a PNG buffer (680×960). */
async function popup({ lang, theme, view, reminder }) {
  const page = await browser.newPage();
  await page.setViewport({ width: 340, height: 480, deviceScaleFactor: 2 });
  await page.emulateMediaFeatures([{ name: "prefers-color-scheme", value: theme }]);
  await page.evaluateOnNewDocument(mockTauri, {
    lang, status: reminder ? reminding : counting, settings, stats,
  });
  await page.goto(url, { waitUntil: "networkidle0" });
  if (view) await page.click(`button[aria-label="${VIEW_LABELS[view][lang]}"]`);
  if (reminder) {
    // Catch the character mid-squat around rep 4.
    await page.waitForFunction(
      () => document.querySelector(".reps")?.textContent.trim().startsWith("4 "),
      { timeout: 60000 },
    );
    await sleep(700);
  } else {
    await sleep(1300);
  }
  const png = await page.screenshot();
  await page.close();
  return png;
}

/** A 1920×1080 Store image: caption on the left, the popup on the right. */
async function storeImage(png, { caption, theme }) {
  const dark = theme === "dark";
  const page = await browser.newPage();
  await page.setViewport({ width: 1920, height: 1080 });
  await page.setContent(`<!doctype html>
<style>
  body { margin: 0; width: 1920px; height: 1080px; display: flex; align-items: center;
    justify-content: center; gap: 140px;
    background: ${dark ? "radial-gradient(circle at 75% 50%, #3a2724, #1c1716 70%)" : "radial-gradient(circle at 75% 50%, #ffe9e1, #fffaf7 70%)"};
    font-family: "Segoe UI Variable Display", "Segoe UI", system-ui, sans-serif;
    color: ${dark ? "#f5ece9" : "#2a1f1d"}; }
  .text { width: 640px; }
  .brand { display: flex; align-items: center; gap: 18px; font-size: 40px; font-weight: 700; margin-bottom: 36px; }
  .mark { width: 64px; height: 64px; padding: 9px; box-sizing: border-box; border-radius: 18px;
    background: linear-gradient(#ff7a45, #ef5a4c); }
  .mark * { fill: none; stroke: #fff; stroke-width: 3; stroke-linecap: round; stroke-linejoin: round; }
  .mark circle { fill: #fff; }
  h1 { margin: 0; font-size: 76px; line-height: 1.08; letter-spacing: -1px; }
  img { width: 578px; height: 816px; border-radius: 18px;
    box-shadow: 0 40px 100px rgba(60, 20, 10, ${dark ? 0.6 : 0.22}), 0 0 0 1px rgba(0, 0, 0, ${dark ? 0.5 : 0.06}); }
</style>
<div class="text">
  <div class="brand"><svg class="mark" viewBox="0 0 24 24"><circle cx="12" cy="6" r="3"/><path d="M4.5 19.5 12 12l7.5 7.5"/></svg>Kalk</div>
  <h1>${caption}</h1>
</div>
<img src="data:image/png;base64,${Buffer.from(png).toString("base64")}">`);
  await page.evaluate(() => document.fonts.ready);
  const out = await page.screenshot();
  await page.close();
  return out;
}

function save(file, data) {
  mkdirSync(path.dirname(file), { recursive: true });
  writeFileSync(file, data);
  console.log(file);
}

const SHOTS = [
  { name: "info" },
  { name: "reminder", reminder: true },
  { name: "stats", view: "stats" },
  { name: "settings", view: "settings" },
];

try {
  for (const lang of ["en", "tr"]) {
    for (const theme of ["light", "dark"]) {
      for (const shot of SHOTS) {
        const png = await popup({ lang, theme, ...shot });
        // The site has no Turkish settings shot.
        if (!(lang === "tr" && shot.name === "settings")) {
          const dir = lang === "tr" ? "docs/screenshots/tr" : "docs/screenshots";
          save(`${dir}/${shot.name}-${theme}.png`, png);
        }
        // Store: every view in light, plus the reminder in dark.
        if (theme === "light" || shot.name === "reminder") {
          const n = theme === "dark" ? 5 : SHOTS.indexOf(shot) + 1;
          const image = await storeImage(png, { caption: CAPTIONS[lang][shot.name], theme });
          save(`store/screenshots/${lang}/${n}-${shot.name}-${theme}.png`, image);
        }
      }
    }
  }
} finally {
  await browser.close();
  await server.close();
}
