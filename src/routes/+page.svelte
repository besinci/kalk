<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { openUrl } from "@tauri-apps/plugin-opener";
  import Settings from "$lib/Settings.svelte";
  import Squatter from "$lib/Squatter.svelte";
  import { fmtClock, fmtLeft, fmtSpan, setLang, t, weekday } from "$lib/i18n.svelte";
  import type { Outcome, SettingsView, Status } from "$lib/types";

  const STUDY_URL = "https://doi.org/10.1111/sms.14628";

  let status = $state<Status | null>(null);
  let settings = $state<SettingsView | null>(null);
  let showSettings = $state(false);
  const reminder = $derived(status?.phase === "reminding" || status?.phase === "asking");
  /** Reps the character has done so far in the open reminder. */
  let repsShown = $state(0);

  function applySettings(view: SettingsView) {
    settings = view;
    setLang(view.lang);
  }

  $effect(() => {
    invoke<SettingsView>("get_settings").then(applySettings);
    invoke<Status>("get_status").then((s) => (status = s));
    const unlisten = listen<Status>("status", (e) => {
      if (e.payload.phase === "reminding" && status?.phase !== "reminding") {
        repsShown = 0;
        showSettings = false;
      }
      status = e.payload;
    });
    return () => {
      unlisten.then((f) => f());
    };
  });

  /** e.g. "Mon, Tue, Wed, Thu, Fri · 09:00–18:00" */
  const activeHours = $derived.by(() => {
    if (!settings) return "";
    const { days, start, end } = settings.settings;
    return `${days.map(weekday).join(", ")} · ${start}–${end}`;
  });

  function answer(outcome: Outcome) {
    invoke("answer", { outcome });
  }

  function hide() {
    invoke("hide_popup");
  }

  function quit() {
    invoke("quit_app");
  }

  function onKeydown(e: KeyboardEvent) {
    if (e.key !== "Escape" || reminder) return;
    if (showSettings) showSettings = false;
    else hide();
  }
</script>

<svelte:window onkeydown={onKeydown} oncontextmenu={(e) => e.preventDefault()} />

<main>
  <header>
    {#if showSettings}
      <div class="brand">
        <button class="icon" title={t("back")} aria-label={t("back")} onclick={() => (showSettings = false)}>
          <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M15 5l-7 7 7 7" /></svg>
        </button>
        <span>{t("settings")}</span>
      </div>
    {:else}
      <div class="brand">
        <svg class="mark" viewBox="0 0 24 24" aria-hidden="true">
          <circle cx="12" cy="6" r="3" />
          <path d="M4.5 19.5 12 12l7.5 7.5" />
        </svg>
        <span>Kalk</span>
      </div>
    {/if}
    <div class="actions">
      {#if !showSettings && !reminder}
        <button
          class="icon"
          title={t("settings")}
          aria-label={t("settings")}
          disabled={!settings}
          onclick={() => (showSettings = true)}
        >
          <svg viewBox="0 0 24 24" aria-hidden="true">
            <circle cx="12" cy="12" r="3" />
            <path
              d="M12 2v3M12 19v3M4.2 4.2l2.1 2.1M17.7 17.7l2.1 2.1M2 12h3M19 12h3M4.2 19.8l2.1-2.1M17.7 6.3l2.1-2.1"
            />
          </svg>
        </button>
      {/if}
      {#if !reminder}
        <button class="icon" title={t("close")} aria-label={t("close")} onclick={hide}>
          <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M6 6l12 12M18 6 6 18" /></svg>
        </button>
      {/if}
    </div>
  </header>

  {#if showSettings && settings}
    <Settings view={settings} onsaved={applySettings} />
  {:else}
    <section class="hero">
      <div class="figure" class:big={status?.phase === "reminding"} aria-hidden="true">
        {#if status?.phase === "reminding"}
          <!-- Squat along: the counter follows the character. -->
          {#key status.phase}
            <Squatter period={2.4} reps={status.reps} onrep={(n: number) => (repsShown = n)} />
          {/key}
        {:else}
          <Squatter />
        {/if}
      </div>
      {#if status?.phase === "reminding" && repsShown > 0}
        <span class="reps" aria-live="polite">{repsShown} / {status.reps}</span>
      {/if}
    </section>

    {#if status?.phase === "reminding"}
      <section class="prompt">
        <h1>{t("remindTitle", { reps: status.reps })}</h1>
        <p>{t("remindBody", { reps: status.reps })}</p>
        <div class="buttons">
          <button class="primary" onclick={() => answer("done")}>{t("done")}</button>
          <button class="secondary" onclick={() => answer("snoozed")}>
            {t("snooze", { d: fmtSpan(status.snooze_secs) })}
          </button>
          <button class="secondary" onclick={() => answer("skipped")}>{t("skip")}</button>
        </div>
      </section>
    {:else if status?.phase === "asking"}
      <section class="prompt">
        <h1>{t("askTitle")}</h1>
        <p>{t("askBody", { reps: status.reps })}</p>
        <div class="buttons">
          <button class="primary" onclick={() => answer("done")}>{t("yes")}</button>
          <button class="secondary" onclick={() => answer("missed")}>{t("no")}</button>
        </div>
      </section>
    {:else}
      <section class="next">
        {#if status?.phase === "paused" && status.paused_until}
          <span class="label">{t("pausedUntil")}</span>
          <span class="value">{fmtClock(status.paused_until)}</span>
          <button class="link" onclick={() => invoke("resume_breaks")}>{t("resumeNow")}</button>
        {:else if status?.phase === "off_hours"}
          <span class="label">{t("offHours")}</span>
          <span class="value">{t("off")}</span>
          <span class="today">{activeHours}</span>
        {:else}
          <span class="label">{t("nextBreakIn")}</span>
          <span class="value">{status ? fmtLeft(status.secs_left) : "–"}</span>
        {/if}
        <span class="today">{t("doneToday", { n: status?.done_today ?? 0 })}</span>
      </section>

      <section class="why">
        <h2>
          {t("whyTitle", {
            reps: settings?.settings.reps ?? 10,
            mins: settings?.settings.interval_mins ?? 45,
          })}
        </h2>
        <p>{t("whyBody")}</p>
        <button class="link" onclick={() => openUrl(STUDY_URL)}>{t("readStudy")}</button>
      </section>
    {/if}
  {/if}

  <footer>
    <button class="quiet" onclick={quit}>{t("quit")}</button>
  </footer>
</main>

<style>
  :global(:root) {
    --bg: #fffaf7;
    --surface: #ffffff;
    --text: #2a1f1d;
    --muted: #7d6b66;
    --line: #f0e3dd;
    --accent: #ef5a4c;
    --accent-2: #ff7a45;
    --accent-soft: #ffe9e1;
    color-scheme: light dark;
  }
  @media (prefers-color-scheme: dark) {
    :global(:root) {
      --bg: #1c1716;
      --surface: #262020;
      --text: #f5ece9;
      --muted: #a9958f;
      --line: #3a302e;
      --accent: #ff6e5e;
      --accent-2: #ff8a55;
      --accent-soft: #3a2724;
    }
  }

  :global(html, body) {
    margin: 0;
    height: 100%;
    background: var(--bg);
    color: var(--text);
    font: 14px/1.45 -apple-system, BlinkMacSystemFont, "Segoe UI Variable", "Segoe UI", system-ui,
      sans-serif;
    -webkit-font-smoothing: antialiased;
    user-select: none;
    overflow: hidden;
  }

  main {
    height: 100%;
    box-sizing: border-box;
    padding: 14px 16px 12px;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }
  .brand {
    display: flex;
    align-items: center;
    gap: 8px;
    font-weight: 700;
    font-size: 16px;
    letter-spacing: 0.2px;
  }
  .mark {
    width: 22px;
    height: 22px;
    padding: 3px;
    border-radius: 7px;
    background: linear-gradient(var(--accent-2), var(--accent));
    fill: #fff;
    stroke: #fff;
    stroke-width: 3;
    stroke-linecap: round;
    stroke-linejoin: round;
  }
  .mark path {
    fill: none;
  }
  .actions {
    display: flex;
    gap: 2px;
  }

  button {
    font: inherit;
    color: inherit;
    border: 0;
    background: none;
    cursor: pointer;
  }
  .icon {
    width: 30px;
    height: 30px;
    border-radius: 8px;
    display: grid;
    place-items: center;
    color: var(--muted);
  }
  .icon:hover:not(:disabled) {
    background: var(--accent-soft);
    color: var(--text);
  }
  .icon:disabled {
    opacity: 0.4;
    cursor: default;
  }
  .icon svg {
    width: 18px;
    height: 18px;
    fill: none;
    stroke: currentColor;
    stroke-width: 2;
    stroke-linecap: round;
  }

  .hero {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 4px;
    padding: 6px 0;
  }
  .figure {
    width: 112px;
    height: 112px;
    box-sizing: border-box;
    padding: 14px;
    border-radius: 50%;
    background: var(--accent-soft);
    transition:
      width 0.25s,
      height 0.25s;
  }
  .figure.big {
    width: 140px;
    height: 140px;
  }
  .reps {
    font-weight: 700;
    font-variant-numeric: tabular-nums;
    color: var(--accent);
  }

  .next {
    display: flex;
    flex-direction: column;
    align-items: center;
    background: var(--surface);
    border: 1px solid var(--line);
    border-radius: 14px;
    padding: 10px;
  }
  .label,
  .today {
    color: var(--muted);
    font-size: 12px;
  }
  .value {
    font-size: 28px;
    font-weight: 700;
    font-variant-numeric: tabular-nums;
    color: var(--accent);
    line-height: 1.2;
  }

  .prompt {
    text-align: center;
  }
  .prompt h1 {
    margin: 0 0 4px;
    font-size: 22px;
    color: var(--accent);
  }
  .prompt p {
    margin: 0 0 14px;
    color: var(--muted);
  }
  .buttons {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .primary,
  .secondary {
    padding: 10px;
    border-radius: 10px;
    font-weight: 600;
  }
  .primary {
    background: linear-gradient(var(--accent-2), var(--accent));
    color: #fff;
  }
  .primary:hover {
    filter: brightness(1.05);
  }
  .secondary {
    background: var(--surface);
    border: 1px solid var(--line);
  }
  .secondary:hover {
    background: var(--accent-soft);
  }

  .why h2 {
    margin: 0 0 4px;
    font-size: 14px;
  }
  .why p {
    margin: 0 0 6px;
    color: var(--muted);
    font-size: 12.5px;
  }
  .link {
    padding: 0;
    color: var(--accent);
    font-weight: 600;
    font-size: 12.5px;
  }
  .link:hover {
    text-decoration: underline;
  }

  footer {
    margin-top: auto;
    display: flex;
    justify-content: center;
    border-top: 1px solid var(--line);
    padding-top: 8px;
  }
  .quiet {
    color: var(--muted);
    font-size: 12px;
    padding: 4px 10px;
    border-radius: 6px;
  }
  .quiet:hover {
    background: var(--accent-soft);
    color: var(--text);
  }
</style>
