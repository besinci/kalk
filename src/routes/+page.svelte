<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { openUrl } from "@tauri-apps/plugin-opener";

  const STUDY_URL = "https://doi.org/10.1111/sms.14628";

  type Phase = "counting" | "paused" | "off_hours" | "reminding" | "asking";
  type Outcome = "done" | "skipped" | "snoozed" | "missed";
  interface Status {
    phase: Phase;
    secs_left: number;
    paused_until: string | null;
    done_today: number;
    reps: number;
    snooze_secs: number;
  }

  let status = $state<Status | null>(null);
  const reminder = $derived(status?.phase === "reminding" || status?.phase === "asking");

  $effect(() => {
    invoke<Status>("get_status").then((s) => (status = s));
    const unlisten = listen<Status>("status", (e) => (status = e.payload));
    return () => {
      unlisten.then((f) => f());
    };
  });

  /** "23 min", or "0:42" under a minute. */
  function fmtLeft(secs: number) {
    return secs >= 60 ? `${Math.ceil(secs / 60)} min` : `0:${String(secs).padStart(2, "0")}`;
  }

  function fmtSnooze(secs: number) {
    return secs >= 60 ? `${Math.round(secs / 60)} min` : `${secs} s`;
  }

  function fmtClock(iso: string) {
    return new Date(iso).toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" });
  }

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
    if (e.key === "Escape" && !reminder) hide();
  }
</script>

<svelte:window onkeydown={onKeydown} oncontextmenu={(e) => e.preventDefault()} />

<main>
  <header>
    <div class="brand">
      <svg class="mark" viewBox="0 0 24 24" aria-hidden="true">
        <circle cx="12" cy="6" r="3" />
        <path d="M4.5 19.5 12 12l7.5 7.5" />
      </svg>
      <span>Kalk</span>
    </div>
    <div class="actions">
      <button class="icon" title="Settings (coming soon)" aria-label="Settings" disabled>
        <svg viewBox="0 0 24 24" aria-hidden="true">
          <circle cx="12" cy="12" r="3" />
          <path
            d="M12 2v3M12 19v3M4.2 4.2l2.1 2.1M17.7 17.7l2.1 2.1M2 12h3M19 12h3M4.2 19.8l2.1-2.1M17.7 6.3l2.1-2.1"
          />
        </svg>
      </button>
      {#if !reminder}
        <button class="icon" title="Close (Esc)" aria-label="Close" onclick={hide}>
          <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M6 6l12 12M18 6 6 18" /></svg>
        </button>
      {/if}
    </div>
  </header>

  <section class="hero" aria-hidden="true">
    <!-- Placeholder; the animated squatting character lands in milestone 4. -->
    <div class="figure">
      <svg viewBox="0 0 24 24">
        <circle cx="12" cy="6" r="3" />
        <path d="M4.5 19.5 12 12l7.5 7.5" />
      </svg>
    </div>
  </section>

  {#if status?.phase === "reminding"}
    <section class="prompt">
      <h1>Time for {status.reps} squats!</h1>
      <p>Stand up, squat {status.reps} times, sit back down.</p>
      <div class="buttons">
        <button class="primary" onclick={() => answer("done")}>Done</button>
        <button class="secondary" onclick={() => answer("snoozed")}>
          Snooze {fmtSnooze(status.snooze_secs)}
        </button>
        <button class="secondary" onclick={() => answer("skipped")}>Skip</button>
      </div>
    </section>
  {:else if status?.phase === "asking"}
    <section class="prompt">
      <h1>Welcome back!</h1>
      <p>Did you do your {status.reps} squats?</p>
      <div class="buttons">
        <button class="primary" onclick={() => answer("done")}>Yes</button>
        <button class="secondary" onclick={() => answer("missed")}>No</button>
      </div>
    </section>
  {:else}
    <section class="next">
      {#if status?.phase === "paused" && status.paused_until}
        <span class="label">Paused until</span>
        <span class="value">{fmtClock(status.paused_until)}</span>
        <button class="link" onclick={() => invoke("resume_breaks")}>Resume now</button>
      {:else if status?.phase === "off_hours"}
        <span class="label">Outside active hours</span>
        <span class="value">Off</span>
        <span class="today">Mon–Fri, 09:00–18:00</span>
      {:else}
        <span class="label">Next squat break in</span>
        <span class="value">{status ? fmtLeft(status.secs_left) : "–"}</span>
      {/if}
      <span class="today">Today: {status?.done_today ?? 0} breaks done</span>
    </section>

    <section class="why">
      <h2>10 squats every 45 minutes</h2>
      <p>Better for blood sugar than one 30‑minute walk.</p>
      <button class="link" onclick={() => openUrl(STUDY_URL)}>Read the study →</button>
    </section>
  {/if}

  <footer>
    <button class="quiet" onclick={quit}>Quit Kalk</button>
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
    display: grid;
    place-items: center;
    padding: 6px 0;
  }
  .figure {
    width: 112px;
    height: 112px;
    border-radius: 50%;
    background: var(--accent-soft);
    display: grid;
    place-items: center;
  }
  .figure svg {
    width: 60px;
    height: 60px;
    fill: var(--accent);
    stroke: var(--accent);
    stroke-width: 3;
    stroke-linecap: round;
    stroke-linejoin: round;
    animation: bob 2.4s ease-in-out infinite;
  }
  .figure path {
    fill: none;
  }
  @keyframes bob {
    0%,
    100% {
      transform: translateY(4px);
    }
    50% {
      transform: translateY(-4px);
    }
  }
  @media (prefers-reduced-motion: reduce) {
    .figure svg {
      animation: none;
    }
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
