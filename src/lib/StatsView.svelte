<script lang="ts">
  import type { Category, DayStatus, Stats } from "./api";
  import { shade } from "./colors.svelte";
  import { longDate, pct, scheduleLabel, WEEKDAYS_LETTER } from "./dates";

  interface Props {
    stats: Stats;
    categories: Category[];
    onedit: (c: Category) => void;
  }

  let { stats, categories, onedit }: Props = $props();

  const rows = $derived(
    categories
      .map((c) => ({ c, s: stats.categories.find((s) => s.categoryId === c.id)! }))
      .filter((r) => r.s),
  );

  const o = $derived(stats.overall);

  const STATUS_LABEL: Record<DayStatus, string> = {
    done: "Cumplido",
    missed: "No cumplido",
    pending: "Pendiente hoy",
    off: "No programado",
    extra: "Extra (día libre)",
    before: "Antes de empezar",
    future: "Próximamente",
  };
  const LEGEND: DayStatus[] = ["done", "extra", "missed", "pending", "off"];

  let tip = $state<{ x: number; y: number; text: string } | null>(null);

  function showTip(e: MouseEvent, text: string) {
    const r = (e.currentTarget as HTMLElement).getBoundingClientRect();
    tip = { x: r.left + r.width / 2, y: r.top, text };
  }

  /** Agrupa el historial (lunes→domingo) en columnas por semana. */
  function weeks<T>(days: T[]): T[][] {
    return Array.from({ length: days.length / 7 }, (_, i) => days.slice(i * 7, i * 7 + 7));
  }

  const plural = (n: number, one: string, many: string) => `${n} ${n === 1 ? one : many}`;
</script>

<div class="stats">
  <section class="tiles">
    <div class="card tile hero">
      <span class="label">Racha perfecta</span>
      <span class="value">🔥 {o.perfectStreak}</span>
      <span class="sub">{o.perfectStreak === 1 ? "día" : "días"} cumpliendo todo · mejor: {o.bestPerfectStreak}</span>
    </div>
    <div class="card tile">
      <span class="label">Hoy</span>
      <span class="value">{o.today.done}<small>/{o.today.target}</small></span>
      <span class="sub">objetivos del día</span>
    </div>
    <div class="card tile">
      <span class="label">Esta semana</span>
      <span class="value">{pct(o.week.done, o.week.target)}<small>%</small></span>
      <span class="sub">{o.week.done} de {o.week.target} cumplidos</span>
    </div>
    <div class="card tile">
      <span class="label">Este mes</span>
      <span class="value">{pct(o.month.done, o.month.target)}<small>%</small></span>
      <span class="sub">{o.month.done} de {o.month.target} cumplidos</span>
    </div>
    <div class="card tile">
      <span class="label">Histórico</span>
      <span class="value">{Math.round(o.completionRate * 100)}<small>%</small></span>
      <span class="sub">de los días programados</span>
    </div>
  </section>

  <section class="cats">
    {#each rows as { c, s } (c.id)}
      {@const col = shade(c.color)}
      <article class="card cat" style:--c={col}>
        <header>
          <span class="icon">{c.icon}</span>
          <div class="title">
            <h3>{c.name}</h3>
            <span class="muted">{scheduleLabel(c.daysMask)}</span>
          </div>
          <button class="btn icon" onclick={() => onedit(c)} aria-label="Editar {c.name}" title="Editar">✎</button>
        </header>

        <div class="numbers">
          <div>
            <span class="big">{s.currentStreak}</span>
            <span class="muted">racha actual</span>
          </div>
          <div>
            <span class="big">{s.bestStreak}</span>
            <span class="muted">mejor racha</span>
          </div>
          <div>
            <span class="big">{Math.round(s.completionRate * 100)}%</span>
            <span class="muted">cumplimiento</span>
          </div>
          <div>
            <span class="big">{s.completedScheduled + s.extraCompletions}</span>
            <span class="muted">{s.extraCompletions ? `total (${s.extraCompletions} extra)` : "total"}</span>
          </div>
        </div>

        <div class="bars">
          {#each [["Semana", s.week], ["Mes", s.month]] as const as [label, p]}
            <div class="bar-row">
              <span class="bar-label">{label}</span>
              <div
                class="track"
                role="img"
                aria-label="{label}: {p.done} de {p.target}"
                onmouseenter={(e) => showTip(e, `${label}: ${p.done} de ${p.target} días objetivo`)}
                onmouseleave={() => (tip = null)}
              >
                <div class="fill" style:width="{pct(p.done, p.target)}%"></div>
              </div>
              <span class="bar-value">{p.done}/{p.target}</span>
            </div>
          {/each}
        </div>

        <div class="heat-wrap">
          <div class="heat-days">
            {#each WEEKDAYS_LETTER as l, i}<span class:hidden={i % 2 === 1}>{l}</span>{/each}
          </div>
          <div class="heat">
            {#each weeks(s.history) as week}
              <div class="col">
                {#each week as d (d.date)}
                  <span
                    class="sq {d.status}"
                    class:today={d.date === stats.today}
                    role="img"
                    aria-label="{longDate(d.date)}: {STATUS_LABEL[d.status]}"
                    onmouseenter={(e) => showTip(e, `${longDate(d.date)} — ${STATUS_LABEL[d.status]}`)}
                    onmouseleave={() => (tip = null)}
                  ></span>
                {/each}
              </div>
            {/each}
          </div>
        </div>
        <footer class="legend">
          {#each LEGEND as st}
            <span><i class="sq {st}"></i>{STATUS_LABEL[st]}</span>
          {/each}
        </footer>
      </article>
    {:else}
      <p class="muted">Crea tu primera categoría para ver métricas.</p>
    {/each}
  </section>
</div>

{#if tip}
  <div class="tooltip" style:left="{tip.x}px" style:top="{tip.y}px">{tip.text}</div>
{/if}

<style>
  .stats {
    display: flex;
    flex-direction: column;
    gap: 20px;
    padding-bottom: 24px;
  }

  .tiles {
    display: grid;
    grid-template-columns: 1.4fr repeat(4, 1fr);
    gap: 12px;
  }

  .tile {
    padding: 16px 18px;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .label {
    font-size: 12px;
    font-weight: 600;
    color: var(--ink-2);
  }

  .value {
    font-size: 30px;
    font-weight: 700;
    letter-spacing: -0.02em;
  }

  .value small {
    font-size: 16px;
    font-weight: 600;
    color: var(--muted);
  }

  .hero .value {
    font-size: 38px;
  }

  .sub {
    font-size: 12px;
    color: var(--muted);
  }

  .cats {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(420px, 1fr));
    gap: 14px;
  }

  .cat {
    padding: 18px;
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .cat header {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .icon {
    width: 40px;
    height: 40px;
    border-radius: 11px;
    display: grid;
    place-items: center;
    font-size: 20px;
    background: color-mix(in srgb, var(--c) 18%, transparent);
  }

  .title {
    flex: 1;
    display: flex;
    flex-direction: column;
  }

  h3 {
    margin: 0;
    font-size: 16px;
  }

  .title .muted {
    font-size: 12px;
  }

  .numbers {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 8px;
  }

  .numbers div {
    display: flex;
    flex-direction: column;
  }

  .big {
    font-size: 22px;
    font-weight: 700;
  }

  .numbers .muted {
    font-size: 12px;
  }

  .bars {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .bar-row {
    display: grid;
    grid-template-columns: 52px 1fr 44px;
    align-items: center;
    gap: 10px;
    font-size: 12px;
  }

  .bar-label {
    color: var(--ink-2);
  }

  .bar-value {
    text-align: right;
    font-variant-numeric: tabular-nums;
    color: var(--ink-2);
  }

  .track {
    height: 10px;
    border-radius: 99px;
    background: var(--surface-3);
    overflow: hidden;
    padding: 2px 0;
  }

  .fill {
    height: 6px;
    min-width: 0;
    border-radius: 0 4px 4px 0;
    background: var(--c);
    transition: width 0.3s ease;
  }

  .heat-wrap {
    display: flex;
    gap: 6px;
  }

  .heat-days {
    display: grid;
    grid-template-rows: repeat(7, 14px);
    gap: 3px;
    font-size: 10px;
    color: var(--muted);
    line-height: 14px;
  }

  .heat-days .hidden {
    visibility: hidden;
  }

  .heat {
    display: flex;
    gap: 3px;
    flex: 1;
  }

  .col {
    display: grid;
    grid-template-rows: repeat(7, 14px);
    gap: 3px;
    flex: 1;
    max-width: 18px;
  }

  .sq {
    display: block;
    border-radius: 3px;
    background: var(--surface-2);
  }

  .sq.done {
    background: var(--c);
  }

  .sq.extra {
    background: color-mix(in srgb, var(--c) 45%, transparent);
  }

  .sq.missed {
    background: color-mix(in srgb, var(--critical) 30%, transparent);
  }

  .sq.pending {
    background: transparent;
    box-shadow: inset 0 0 0 2px var(--c);
  }

  .sq.off {
    background: var(--surface-2);
    opacity: 0.6;
  }

  .sq.before,
  .sq.future {
    background: transparent;
    box-shadow: inset 0 0 0 1px var(--line);
  }

  .sq.today {
    outline: 1.5px solid var(--ink-2);
    outline-offset: 1px;
  }

  .legend {
    display: flex;
    flex-wrap: wrap;
    gap: 12px;
    font-size: 11px;
    color: var(--muted);
  }

  .legend span {
    display: inline-flex;
    align-items: center;
    gap: 5px;
  }

  .legend .sq {
    width: 10px;
    height: 10px;
  }

  .tooltip {
    position: fixed;
    transform: translate(-50%, calc(-100% - 8px));
    background: var(--ink);
    color: var(--surface);
    font-size: 12px;
    padding: 5px 9px;
    border-radius: 7px;
    pointer-events: none;
    white-space: nowrap;
    z-index: 60;
  }

  @media (max-width: 1100px) {
    .tiles {
      grid-template-columns: repeat(3, 1fr);
    }
  }
</style>
