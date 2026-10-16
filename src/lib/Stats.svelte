<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { fmtDay, t, weekdayOf } from "./i18n.svelte";
  import type { Stats } from "./types";

  let stats = $state<Stats | null>(null);
  let hovered = $state<number | null>(null);

  $effect(() => {
    invoke<Stats>("get_stats").then((s) => (stats = s));
  });

  const CHART_H = 96;
  /** Top of the y scale: the goal, or the best day if it beat the goal. */
  const max = $derived(stats ? Math.max(stats.goal, ...stats.week.map((d) => d.done)) : 1);
  const y = (n: number) => (n / max) * CHART_H;
</script>

{#if stats}
  <div class="stats">
    <div class="tiles">
      <div class="tile">
        <span class="tile-label">{t("today")}</span>
        <span class="tile-value">{stats.today}<small>&nbsp;/ {stats.goal}</small></span>
      </div>
      <div class="tile">
        <span class="tile-label">{t("streak")}</span>
        <span class="tile-value">{t("streakDays", { n: stats.streak })}</span>
      </div>
      <div class="tile">
        <span class="tile-label">{t("totalSquats")}</span>
        <span class="tile-value">{stats.total_squats.toLocaleString()}</span>
      </div>
    </div>

    <figure>
      <figcaption>{t("last7")}</figcaption>
      <div class="plot" style:height="{CHART_H}px">
        <div class="goal" style:bottom="{y(stats.goal)}px">
          <span>{t("goal", { n: stats.goal })}</span>
        </div>
        {#each stats.week as day, i (day.date)}
          {@const today = i === stats.week.length - 1}
          <!-- The whole column is the hover target, not just the bar. -->
          <div
            class="col"
            role="img"
            aria-label={t("dayBreaks", { day: fmtDay(day.date), n: day.done })}
            onpointerenter={() => (hovered = i)}
            onpointerleave={() => (hovered = null)}
          >
            {#if today && day.done > 0}
              <span class="value" style:bottom="{y(day.done) + 2}px">{day.done}</span>
            {/if}
            <div class="bar" class:dim={hovered !== null && hovered !== i} style:height="{y(day.done)}px"></div>
            {#if hovered === i}
              <div class="tip" class:right={i > 3}>
                {t("dayBreaks", { day: fmtDay(day.date), n: day.done })}
              </div>
            {/if}
          </div>
        {/each}
      </div>
      <div class="axis">
        {#each stats.week as day, i (day.date)}
          <span class:today={i === stats.week.length - 1}>{weekdayOf(day.date)}</span>
        {/each}
      </div>
    </figure>
  </div>
{/if}

<style>
  .stats {
    display: flex;
    flex-direction: column;
    gap: 14px;
  }
  .tiles {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 6px;
  }
  .tile {
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: 8px 10px;
    background: var(--surface);
    border: 1px solid var(--line);
    border-radius: 12px;
  }
  .tile-label {
    font-size: 11.5px;
    color: var(--muted);
  }
  .tile-value {
    font-size: 18px;
    font-weight: 700;
    font-variant-numeric: tabular-nums;
    white-space: nowrap;
  }
  .tile-value small {
    font-size: 12px;
    font-weight: 600;
    color: var(--muted);
  }

  figure {
    margin: 0;
  }
  figcaption {
    font-size: 12px;
    font-weight: 600;
    margin-bottom: 18px;
  }
  .plot {
    position: relative;
    display: flex;
    gap: 2px;
    border-bottom: 1px solid var(--line);
  }
  .goal {
    position: absolute;
    left: 0;
    right: 0;
    border-top: 1px dashed var(--muted);
    opacity: 0.6;
    pointer-events: none;
  }
  .goal span {
    position: absolute;
    right: 0;
    bottom: 2px;
    font-size: 10.5px;
    color: var(--muted);
  }
  .col {
    position: relative;
    flex: 1;
    display: flex;
    align-items: flex-end;
    justify-content: center;
  }
  .bar {
    width: 60%;
    background: var(--chart);
    border-radius: 4px 4px 0 0;
    transition: opacity 0.15s;
  }
  .bar.dim {
    opacity: 0.45;
  }
  .value {
    position: absolute;
    font-size: 11.5px;
    font-weight: 700;
    font-variant-numeric: tabular-nums;
  }
  .tip {
    position: absolute;
    bottom: calc(100% + 4px);
    left: 0;
    z-index: 1;
    padding: 4px 8px;
    border-radius: 6px;
    background: var(--text);
    color: var(--bg);
    font-size: 11.5px;
    white-space: nowrap;
    pointer-events: none;
  }
  .tip.right {
    left: auto;
    right: 0;
  }
  .axis {
    display: flex;
    gap: 2px;
    margin-top: 4px;
  }
  .axis span {
    flex: 1;
    text-align: center;
    font-size: 11px;
    color: var(--muted);
  }
  .axis .today {
    color: var(--text);
    font-weight: 700;
  }
</style>
