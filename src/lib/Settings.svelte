<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { t, weekday } from "./i18n.svelte";
  import type { Settings, SettingsView } from "./types";

  interface Props {
    view: SettingsView;
    /** Called with the saved (and sanitised) settings after every change. */
    onsaved: (view: SettingsView) => void;
  }
  let { view, onsaved }: Props = $props();

  // A local copy the form edits, taken once on open and then replaced with
  // whatever Rust saved.
  // svelte-ignore state_referenced_locally
  let s = $state<Settings>($state.snapshot(view.settings));
  let confirmingReset = $state(false);
  /** Briefly swaps the "saved automatically" hint for a "Saved" tick. */
  let justSaved = $state(false);
  let savedTimer: ReturnType<typeof setTimeout> | undefined;

  async function save() {
    const saved = await invoke<SettingsView>("set_settings", { settings: $state.snapshot(s) });
    s = saved.settings;
    onsaved(saved);
    justSaved = true;
    clearTimeout(savedTimer);
    savedTimer = setTimeout(() => (justSaved = false), 1500);
  }

  function toggleDay(day: number) {
    s.days = s.days.includes(day) ? s.days.filter((d) => d !== day) : [...s.days, day];
    save();
  }

  async function resetStats() {
    if (!confirmingReset) {
      confirmingReset = true;
      return;
    }
    confirmingReset = false;
    await invoke("reset_stats");
  }

  const unit = $derived(view.dev ? t("sec") : t("min"));
</script>

<div class="settings">
  {#if view.dev}
    <p class="note">{t("devNote")}</p>
  {/if}

  <section>
    <h3>{t("breaksSection")}</h3>
    <div class="card">
      <label class="row">
        <span>{t("interval")}</span>
        <span class="field">
          <input type="number" min="1" max="240" bind:value={s.interval_mins} onchange={save} />
          <span class="unit">{unit}</span>
        </span>
      </label>
      <label class="row">
        <span>{t("reps")}</span>
        <span class="field">
          <input type="number" min="1" max="100" bind:value={s.reps} onchange={save} />
          <span class="unit"></span>
        </span>
      </label>
      <label class="row">
        <span>{t("dailyGoal")}</span>
        <span class="field">
          <input type="number" min="1" max="50" bind:value={s.daily_goal} onchange={save} />
          <span class="unit">{t("breaks")}</span>
        </span>
      </label>
      <label class="row">
        <span>{t("idle")}</span>
        <span class="field">
          <input type="number" min="1" max="120" bind:value={s.idle_mins} onchange={save} />
          <span class="unit">{unit}</span>
        </span>
      </label>
      <label class="row">
        <span>{t("snoozeFor")}</span>
        <span class="field">
          <input type="number" min="1" max="60" bind:value={s.snooze_mins} onchange={save} />
          <span class="unit">{unit}</span>
        </span>
      </label>
    </div>
  </section>

  <section>
    <h3>{t("activeHoursSection")}</h3>
    <div class="card">
      <label class="row">
        <span>{t("activeHours")}</span>
        <input type="checkbox" bind:checked={s.active_hours} onchange={save} />
      </label>
      {#if s.active_hours}
        <div class="row">
          <span class="field">
            <input type="time" aria-label={t("from")} bind:value={s.start} onchange={save} />
            {t("to")}
            <input type="time" aria-label={t("to")} bind:value={s.end} onchange={save} />
          </span>
        </div>
        <div class="days">
          {#each [0, 1, 2, 3, 4, 5, 6] as day (day)}
            <button
              class="day"
              class:on={s.days.includes(day)}
              aria-pressed={s.days.includes(day)}
              onclick={() => toggleDay(day)}>{weekday(day)}</button
            >
          {/each}
        </div>
      {/if}
    </div>
  </section>

  <section>
    <h3>{t("general")}</h3>
    <div class="card">
      <label class="row">
        <span>{t("launchAtLogin")}</span>
        <input type="checkbox" bind:checked={s.launch_at_login} onchange={save} />
      </label>
      <label class="row">
        <span>{t("sound")}</span>
        <input type="checkbox" bind:checked={s.sound} onchange={save} />
      </label>
      <label class="row">
        <span>{t("language")}</span>
        <select bind:value={s.language} onchange={save}>
          <option value="system">{t("system")}</option>
          <option value="en">English</option>
          <option value="tr">Türkçe</option>
        </select>
      </label>
    </div>
  </section>

  <button class="reset" class:confirming={confirmingReset} onclick={resetStats}>
    {confirmingReset ? t("resetConfirm") : t("resetStats")}
  </button>
</div>

<p class="saved" class:flash={justSaved} aria-live="polite">
  {#if justSaved}
    <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M5 12.5l4.5 4.5L19 7.5" /></svg>
    {t("saved")}
  {:else}
    {t("autoSaved")}
  {/if}
</p>

<style>
  .settings {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 16px;
    overflow-y: auto;
    min-height: 0;
    margin: 0 -4px;
    padding: 2px 4px 4px;
  }
  h3 {
    margin: 0 0 6px 4px;
    font-size: 11.5px;
    font-weight: 600;
    letter-spacing: 0.04em;
    text-transform: uppercase;
    color: var(--muted);
  }
  .card {
    background: var(--surface);
    border: 1px solid var(--line);
    border-radius: 12px;
    padding: 0 12px;
  }
  .note {
    margin: 0;
    padding: 6px 8px;
    border-radius: 8px;
    background: var(--accent-soft);
    font-size: 12px;
  }
  .row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    min-height: 42px;
  }
  .row + .row {
    border-top: 1px solid var(--line);
  }
  .field {
    display: flex;
    align-items: center;
    gap: 6px;
    color: var(--muted);
  }
  .unit {
    min-width: 3.4em;
  }
  input[type="number"] {
    width: 52px;
  }
  input[type="number"],
  input[type="time"],
  select {
    font: inherit;
    color: var(--text);
    background: var(--surface);
    border: 1px solid var(--line);
    border-radius: 8px;
    padding: 5px 8px;
  }
  input[type="checkbox"] {
    width: 16px;
    height: 16px;
    accent-color: var(--accent);
  }
  .days {
    display: flex;
    gap: 4px;
    padding: 4px 0 12px;
  }
  .day {
    flex: 1;
    padding: 6px 0;
    border-radius: 8px;
    border: 1px solid var(--line);
    font-size: 11.5px;
    color: var(--muted);
  }
  .day.on {
    background: var(--accent);
    border-color: var(--accent);
    color: #fff;
  }
  .reset {
    padding: 8px;
    border-radius: 8px;
    color: var(--muted);
    font-size: 12.5px;
  }
  .reset:hover,
  .reset.confirming {
    background: var(--accent-soft);
    color: var(--accent);
  }
  .saved {
    margin: -4px 0 0;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 4px;
    font-size: 12px;
    color: var(--muted);
    transition: color 0.2s;
  }
  .saved.flash {
    color: var(--accent);
    font-weight: 600;
  }
  .saved svg {
    width: 14px;
    height: 14px;
    fill: none;
    stroke: currentColor;
    stroke-width: 2.5;
    stroke-linecap: round;
    stroke-linejoin: round;
  }
</style>
