<!--
  The squatting character, drawn side-on as a stick figure whose joints are
  posed from a single squat depth (0 = standing, 1 = bottom of the squat).
  Driven from JS rather than CSS keyframes so the rep counter stays in sync.
-->
<script lang="ts">
  interface Props {
    /** Seconds per squat. */
    period?: number;
    /** Stop standing up after this many reps; omit to loop forever. */
    reps?: number;
    /** Called with the rep count each time the figure reaches the bottom. */
    onrep?: (count: number) => void;
  }
  let { period = 3, reps, onrep }: Props = $props();

  const reducedMotion =
    typeof matchMedia === "function" && matchMedia("(prefers-reduced-motion: reduce)").matches;

  let depth = $state(0);

  $effect(() => {
    if (reducedMotion) return;
    const ms = period * 1000;
    let start: number | undefined;
    let counted = 0;
    let frame = requestAnimationFrame(function step(now) {
      start ??= now;
      const t = (now - start) / ms;
      // A rep counts at the bottom, half way through each cycle.
      const bottoms = Math.floor(t + 0.5);
      if (bottoms > counted) {
        counted = bottoms;
        onrep?.(counted);
      }
      if (reps !== undefined && t >= reps) {
        depth = 0;
        return;
      }
      depth = (1 - Math.cos(2 * Math.PI * t)) / 2;
      frame = requestAnimationFrame(step);
    });
    return () => cancelAnimationFrame(frame);
  });

  const rad = (deg: number) => (deg * Math.PI) / 180;
  const lerp = (a: number, b: number) => a + (b - a) * depth;

  // Facing right; angles in degrees from vertical.
  const pose = $derived.by(() => {
    const ankle = { x: 44, y: 90 };
    const shin = rad(lerp(4, 36));
    const knee = { x: ankle.x + 21 * Math.sin(shin), y: ankle.y - 21 * Math.cos(shin) };
    const thigh = rad(lerp(4, 80));
    const hip = { x: knee.x - 21 * Math.sin(thigh), y: knee.y - 21 * Math.cos(thigh) };
    const lean = rad(lerp(3, 36));
    const shoulder = { x: hip.x + 24 * Math.sin(lean), y: hip.y - 24 * Math.cos(lean) };
    const head = { x: shoulder.x + 10 * Math.sin(lean), y: shoulder.y - 10 * Math.cos(lean) };
    // Arms hang down standing and reach forward at the bottom, for balance.
    const arm = rad(lerp(20, 92));
    const hand = { x: shoulder.x + 19 * Math.sin(arm), y: shoulder.y + 19 * Math.cos(arm) };
    return { ankle, knee, hip, shoulder, head, hand };
  });

  const pt = (p: { x: number; y: number }) => `${p.x.toFixed(2)},${p.y.toFixed(2)}`;
</script>

<svg viewBox="0 0 100 100" aria-hidden="true">
  <ellipse class="shadow" cx="48" cy="93" rx={lerp(17, 21)} ry="2.6" />
  <polyline
    class="limb"
    points="{pt(pose.ankle)} {pt(pose.knee)} {pt(pose.hip)} {pt(pose.shoulder)}"
  />
  <line class="limb" x1="40" y1="91" x2="53" y2="91" />
  <line
    class="limb"
    x1={pose.shoulder.x}
    y1={pose.shoulder.y}
    x2={pose.hand.x}
    y2={pose.hand.y}
  />
  <circle class="head" cx={pose.head.x} cy={pose.head.y} r="7.5" />
  <circle class="eye" cx={pose.head.x + 3.2} cy={pose.head.y - 1} r="1.1" />
</svg>

<style>
  svg {
    width: 100%;
    height: 100%;
    overflow: visible;
  }
  .limb {
    fill: none;
    stroke: var(--accent);
    stroke-width: 6;
    stroke-linecap: round;
    stroke-linejoin: round;
  }
  .head {
    fill: var(--accent);
  }
  .eye {
    fill: var(--bg);
  }
  .shadow {
    fill: var(--accent);
    opacity: 0.15;
  }
</style>
