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

  async function save() {
    const saved = await invoke<SettingsView>("set_settings", { settings: $state.snapshot(s) });
    s = saved.settings;
    onsaved(saved);
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

  <div class="group">
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

  <button class="reset" class:confirming={confirmingReset} onclick={resetStats}>
    {confirmingReset ? t("resetConfirm") : t("resetStats")}
  </button>
</div>

<style>
  .settings {
    display: flex;
    flex-direction: column;
    gap: 2px;
    overflow-y: auto;
    min-height: 0;
    margin: 0 -4px;
    padding: 0 4px;
  }
  .note {
    margin: 0 0 6px;
    padding: 6px 8px;
    border-radius: 8px;
    background: var(--accent-soft);
    font-size: 12px;
  }
  .row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    min-height: 34px;
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
  .group {
    border-top: 1px solid var(--line);
    border-bottom: 1px solid var(--line);
    padding: 2px 0 8px;
    margin: 4px 0;
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
    border-radius: 6px;
    padding: 3px 6px;
  }
  input[type="checkbox"] {
    width: 16px;
    height: 16px;
    accent-color: var(--accent);
  }
  .days {
    display: flex;
    gap: 4px;
  }
  .day {
    flex: 1;
    padding: 4px 0;
    border-radius: 6px;
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
    margin-top: 8px;
    padding: 6px;
    border-radius: 8px;
    color: var(--muted);
    font-size: 12.5px;
  }
  .reset:hover,
  .reset.confirming {
    background: var(--accent-soft);
    color: var(--accent);
  }
</style>
