<script lang="ts">
  import type { Category } from "./api";
  import { shade } from "./colors.svelte";
  import { fromISO, isScheduled, monthGrid, WEEKDAYS_SHORT } from "./dates";

  interface Props {
    categories: Category[];
    done: Set<string>;
    year: number;
    month: number;
    today: string;
    filter: number | null;
    selected: string;
    ontoggle: (categoryId: number, date: string) => void;
    onselect: (date: string) => void;
  }

  let { categories, done, year, month, today, filter, selected, ontoggle, onselect }: Props = $props();

  const grid = $derived(monthGrid(year, month));
  const focused = $derived(categories.find((c) => c.id === filter) ?? null);

  const isDone = (id: number, date: string) => done.has(`${id}|${date}`);

  /** Categorías que aparecen en el día: las programadas y las cumplidas como extra. */
  function visible(date: string) {
    return categories.filter((c) => isScheduled(c.daysMask, date) || isDone(c.id, date));
  }

  function isPerfect(date: string) {
    if (date > today) return false;
    const due = categories.filter((c) => isScheduled(c.daysMask, date) && c.createdAt <= date);
    return due.length > 0 && due.every((c) => isDone(c.id, date));
  }

  function chipState(c: Category, date: string) {
    if (isDone(c.id, date)) return isScheduled(c.daysMask, date) ? "done" : "extra";
    if (date > today) return "future";
    if (date === today) return "pending";
    // Antes de crear la categoría no es un fallo: se puede registrar a posteriori.
    return date < c.createdAt ? "open" : "missed";
  }

  const LABEL: Record<string, string> = {
    done: "cumplido",
    extra: "cumplido (extra)",
    future: "programado",
    pending: "pendiente hoy",
    missed: "no cumplido",
    open: "sin registrar",
  };

  function cellClick(date: string) {
    onselect(date);
    if (focused && date <= today) ontoggle(focused.id, date);
  }
</script>

<div class="calendar">
  <div class="head">
    {#each WEEKDAYS_SHORT as d}<span>{d}</span>{/each}
  </div>
  <div class="grid">
    {#each grid as date (date)}
      {@const inMonth = fromISO(date).getMonth() === month}
      {@const future = date > today}
      <div
        class="cell"
        class:out={!inMonth}
        class:today={date === today}
        class:selected={date === selected}
        class:future
        class:perfect={inMonth && !focused && isPerfect(date)}
        role="button"
        tabindex="0"
        onclick={() => cellClick(date)}
        onkeydown={(e) => (e.key === "Enter" || e.key === " ") && cellClick(date)}
      >
        <span class="num">{fromISO(date).getDate()}</span>
        {#if inMonth && !focused && isPerfect(date)}
          <span class="badge" title="Día perfecto: todo cumplido">✓</span>
        {/if}

        {#if !inMonth}
          <!-- Días de otros meses: solo el número. -->
        {:else if focused}
          {@const st = chipState(focused, date)}
          {@const scheduled = isScheduled(focused.daysMask, date)}
          {#if st === "done" || st === "extra"}
            <svg class="big-x" class:extra={st === "extra"} viewBox="0 0 100 100" aria-label="Cumplido">
              <path d="M24 24 L76 76 M76 24 L24 76" stroke={shade(focused.color)} />
            </svg>
          {:else if scheduled && st === "missed"}
            <span class="missed-dot" title="No cumplido"></span>
          {:else if scheduled}
            <span class="due-ring" style:--c={shade(focused.color)} title={LABEL[st]}></span>
          {/if}
          {#if !scheduled && st !== "extra"}<span class="rest">descanso</span>{/if}
        {:else}
          <div class="chips">
            {#each visible(date) as c (c.id)}
              {@const st = chipState(c, date)}
              <button
                class="chip {st}"
                style:--c={shade(c.color)}
                title="{c.name}: {LABEL[st]}"
                aria-label="{c.name}: {LABEL[st]}"
                disabled={future}
                onclick={(e) => {
                  e.stopPropagation();
                  onselect(date);
                  ontoggle(c.id, date);
                }}
              >
                {#if st === "done" || st === "extra"}
                  <svg viewBox="0 0 20 20"><path d="M5 5 L15 15 M15 5 L5 15" /></svg>
                {:else}
                  <span class="emoji">{c.icon}</span>
                {/if}
              </button>
            {/each}
          </div>
        {/if}
      </div>
    {/each}
  </div>
</div>

<style>
  .calendar {
    display: flex;
    flex-direction: column;
    height: 100%;
    min-height: 0;
  }

  .head {
    display: grid;
    grid-template-columns: repeat(7, 1fr);
    gap: 6px;
    margin-bottom: 6px;
  }

  .head span {
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--muted);
    padding-left: 10px;
  }

  .grid {
    flex: 1;
    display: grid;
    grid-template-columns: repeat(7, 1fr);
    grid-template-rows: repeat(6, 1fr);
    gap: 6px;
    min-height: 0;
  }

  .cell {
    position: relative;
    background: var(--surface);
    border: 1px solid var(--ring);
    border-radius: 12px;
    padding: 8px 10px;
    display: flex;
    flex-direction: column;
    gap: 6px;
    min-height: 0;
    overflow: hidden;
    cursor: pointer;
    transition: border-color 0.15s, transform 0.1s;
  }

  .cell:hover {
    border-color: var(--line-strong);
  }

  .cell.out {
    opacity: 0.4;
  }

  .cell.future {
    background: transparent;
  }

  .cell.selected {
    border-color: var(--accent);
    box-shadow: 0 0 0 1px var(--accent);
  }

  .cell.perfect {
    background: color-mix(in srgb, var(--good) 8%, var(--surface));
  }

  .num {
    font-size: 13px;
    font-weight: 600;
    color: var(--ink-2);
    width: 24px;
    height: 24px;
    display: grid;
    place-items: center;
    border-radius: 50%;
    margin: -2px 0 0 -4px;
  }

  .today .num {
    background: var(--accent);
    color: #fff;
  }

  .badge {
    position: absolute;
    top: 8px;
    right: 9px;
    font-size: 11px;
    font-weight: 700;
    color: var(--good-text);
  }

  .chips {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }

  .chip {
    width: 34px;
    height: 34px;
    border-radius: 10px;
    display: grid;
    place-items: center;
    background: color-mix(in srgb, var(--c) 14%, transparent);
    border: 1.5px solid color-mix(in srgb, var(--c) 35%, transparent);
    transition: transform 0.1s, background 0.15s;
  }

  .chip:not(:disabled):hover {
    transform: scale(1.08);
    background: color-mix(in srgb, var(--c) 26%, transparent);
  }

  .chip .emoji {
    font-size: 17px;
    line-height: 1;
  }

  .chip svg {
    width: 20px;
    height: 20px;
    stroke: #fff;
    stroke-width: 3;
    stroke-linecap: round;
  }

  .chip.done {
    background: var(--c);
    border-color: var(--c);
  }

  .chip.extra {
    background: color-mix(in srgb, var(--c) 60%, transparent);
    border-color: transparent;
  }

  .chip.missed {
    background: transparent;
    border-style: dashed;
    border-color: color-mix(in srgb, var(--critical) 55%, transparent);
  }

  .chip.missed .emoji {
    opacity: 0.45;
    filter: grayscale(1);
  }

  .chip.pending {
    background: color-mix(in srgb, var(--c) 22%, transparent);
    border: 2px solid var(--c);
  }

  .chip.future {
    background: color-mix(in srgb, var(--c) 7%, transparent);
    border-color: transparent;
  }

  .chip.future .emoji {
    opacity: 0.45;
  }

  .big-x {
    position: absolute;
    inset: 18% 14% 6%;
    width: 72%;
    height: 76%;
    animation: draw 0.25s ease-out;
  }

  .big-x path {
    fill: none;
    stroke-width: 11;
    stroke-linecap: round;
  }

  .big-x.extra {
    opacity: 0.5;
  }

  .missed-dot {
    margin: auto;
    width: 10px;
    height: 10px;
    border-radius: 50%;
    background: var(--critical);
    opacity: 0.7;
  }

  .due-ring {
    margin: auto;
    width: 26px;
    height: 26px;
    border-radius: 50%;
    border: 2px dashed var(--c);
    opacity: 0.6;
  }

  .rest {
    margin: auto;
    font-size: 11px;
    color: var(--muted);
  }

  @keyframes draw {
    from {
      transform: scale(0.6);
      opacity: 0;
    }
  }
</style>
