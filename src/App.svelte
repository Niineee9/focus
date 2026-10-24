<script lang="ts">
  import { api, type Category, type CategoryInput, type Stats } from "./lib/api";
  import Calendar from "./lib/Calendar.svelte";
  import CategoryModal from "./lib/CategoryModal.svelte";
  import StatsView from "./lib/StatsView.svelte";
  import { shade } from "./lib/colors.svelte";
  import { isScheduled, longDate, monthGrid, MONTHS, scheduleLabel, toISO } from "./lib/dates";

  let today = $state(toISO(new Date()));
  let view = $state<"calendar" | "stats">("calendar");
  let year = $state(new Date().getFullYear());
  let month = $state(new Date().getMonth());
  let selected = $state(toISO(new Date()));
  let filter = $state<number | null>(null);

  let categories = $state<Category[]>([]);
  let done = $state(new Set<string>());
  let stats = $state<Stats | null>(null);
  let error = $state("");
  let modal = $state<{ category: Category | null } | null>(null);

  const statsById = $derived(new Map(stats?.categories.map((s) => [s.categoryId, s]) ?? []));

  async function loadCompletions() {
    const grid = monthGrid(year, month);
    const list = await api.getCompletions(grid[0], grid[grid.length - 1]);
    done = new Set(list.map((c) => `${c.categoryId}|${c.date}`));
  }

  async function refresh() {
    try {
      [categories, stats] = await Promise.all([api.listCategories(), api.getStats()]);
      await loadCompletions();
      error = "";
    } catch (e) {
      error = String(e);
    }
  }

  async function toggle(categoryId: number, date: string) {
    if (date > today) return;
    const key = `${categoryId}|${date}`;
    // Actualización optimista: la X aparece al instante.
    const next = new Set(done);
    next.has(key) ? next.delete(key) : next.add(key);
    done = next;
    try {
      await api.toggleCompletion(categoryId, date);
      stats = await api.getStats();
    } catch (e) {
      error = String(e);
      await refresh();
    }
  }

  function shiftMonth(delta: number) {
    const d = new Date(year, month + delta, 1);
    year = d.getFullYear();
    month = d.getMonth();
    loadCompletions();
  }

  function goToday() {
    const now = new Date();
    year = now.getFullYear();
    month = now.getMonth();
    selected = today;
    loadCompletions();
  }

  async function save(input: CategoryInput) {
    if (modal?.category) await api.updateCategory(modal.category.id, input);
    else await api.createCategory(input);
    modal = null;
    await refresh();
  }

  async function remove() {
    if (!modal?.category) return;
    await api.deleteCategory(modal.category.id);
    if (filter === modal.category.id) filter = null;
    modal = null;
    await refresh();
  }

  // Si la app queda abierta pasada la medianoche, actualiza "hoy".
  $effect(() => {
    const t = setInterval(() => {
      const now = toISO(new Date());
      if (now !== today) {
        today = now;
        refresh();
      }
    }, 60_000);
    return () => clearInterval(t);
  });

  refresh();

  const dayCategories = $derived(
    [...categories].sort(
      (a, b) => Number(isScheduled(b.daysMask, selected)) - Number(isScheduled(a.daysMask, selected)),
    ),
  );
</script>

<div class="app">
  <aside class="sidebar">
    <div class="brand">
      <span class="logo">✕</span>
      <span>Focus</span>
    </div>

    <nav>
      <button class:on={view === "calendar"} onclick={() => (view = "calendar")}>📅 Calendario</button>
      <button class:on={view === "stats"} onclick={() => (view = "stats")}>📊 Métricas</button>
    </nav>

    <div class="section-title">
      <span>Categorías</span>
      <button class="add" onclick={() => (modal = { category: null })} title="Nueva categoría">＋</button>
    </div>

    <ul class="cats">
      <li>
        <button class="cat" class:on={filter === null} onclick={() => (filter = null)}>
          <span class="cat-icon all">✦</span>
          <span class="cat-name">Todas</span>
        </button>
      </li>
      {#each categories as c (c.id)}
        {@const s = statsById.get(c.id)}
        <li>
          <button
            class="cat"
            class:on={filter === c.id}
            style:--c={shade(c.color)}
            onclick={() => {
              filter = filter === c.id ? null : c.id;
              view = "calendar";
            }}
          >
            <span class="cat-icon">{c.icon}</span>
            <span class="cat-name">
              {c.name}
              <small>{scheduleLabel(c.daysMask)}</small>
            </span>
            {#if s}
              <span class="streak" class:hot={s.currentStreak > 0} title="Racha actual">🔥{s.currentStreak}</span>
            {/if}
          </button>
          <button class="edit" onclick={() => (modal = { category: c })} aria-label="Editar {c.name}" title="Editar">✎</button>
        </li>
      {/each}
    </ul>

    {#if stats}
      <div class="today-card">
        <span class="muted">Hoy</span>
        <strong>{stats.overall.today.done} de {stats.overall.today.target} objetivos</strong>
        <div class="mini-track">
          <div
            style:width="{stats.overall.today.target
              ? (stats.overall.today.done / stats.overall.today.target) * 100
              : 0}%"
          ></div>
        </div>
        <span class="muted">Racha perfecta: 🔥 {stats.overall.perfectStreak}</span>
      </div>
    {/if}
  </aside>

  <main>
    {#if error}
      <div class="error" role="alert">{error} <button onclick={() => (error = "")}>✕</button></div>
    {/if}

    {#if view === "calendar"}
      <header class="top">
        <h1>
          <span class="month">{MONTHS[month]}</span>
          <span class="year">{year}</span>
        </h1>
        {#if filter !== null}
          {@const f = categories.find((c) => c.id === filter)}
          {#if f}
            <span class="filter-pill" style:--c={shade(f.color)}>
              {f.icon} {f.name}
              <button onclick={() => (filter = null)} aria-label="Quitar filtro">✕</button>
            </span>
          {/if}
        {/if}
        <span class="spacer"></span>
        <button class="btn" onclick={goToday}>Hoy</button>
        <button class="btn icon" onclick={() => shiftMonth(-1)} aria-label="Mes anterior">‹</button>
        <button class="btn icon" onclick={() => shiftMonth(1)} aria-label="Mes siguiente">›</button>
      </header>

      <div class="cal-layout">
        <div class="cal">
          <Calendar
            {categories}
            {done}
            {year}
            {month}
            {today}
            {filter}
            {selected}
            ontoggle={toggle}
            onselect={(d) => (selected = d)}
          />
        </div>

        <aside class="card day">
          <h2>{longDate(selected)}</h2>
          {#if selected > today}
            <p class="muted">Día futuro: podrás marcarlo cuando llegue.</p>
          {/if}
          <ul>
            {#each dayCategories as c (c.id)}
              {@const key = `${c.id}|${selected}`}
              {@const sched = isScheduled(c.daysMask, selected)}
              <li>
                <button
                  class="day-item"
                  class:checked={done.has(key)}
                  class:unscheduled={!sched}
                  style:--c={shade(c.color)}
                  disabled={selected > today}
                  onclick={() => toggle(c.id, selected)}
                >
                  <span class="box">
                    {#if done.has(key)}
                      <svg viewBox="0 0 20 20"><path d="M5 5 L15 15 M15 5 L5 15" /></svg>
                    {/if}
                  </span>
                  <span class="day-icon">{c.icon}</span>
                  <span class="day-name">
                    {c.name}
                    <small>{sched ? "Objetivo del día" : "Día libre · cuenta como extra"}</small>
                  </span>
                </button>
              </li>
            {:else}
              <li class="muted">Aún no tienes categorías.</li>
            {/each}
          </ul>
          <p class="hint muted">
            {filter === null
              ? "Pulsa un icono del calendario para marcarlo con una ✕."
              : "Con una categoría seleccionada, pulsa cualquier día para marcar o desmarcar."}
          </p>
        </aside>
      </div>
    {:else if stats}
      <header class="top">
        <h1>Métricas</h1>
      </header>
      <div class="scroll">
        <StatsView {stats} {categories} onedit={(c) => (modal = { category: c })} />
      </div>
    {/if}
  </main>
</div>

{#if modal}
  <CategoryModal category={modal.category} onsave={save} ondelete={remove} onclose={() => (modal = null)} />
{/if}

<style>
  .app {
    display: grid;
    grid-template-columns: 270px 1fr;
    height: 100vh;
  }

  .sidebar {
    background: var(--surface);
    border-right: 1px solid var(--line);
    padding: 20px 14px;
    display: flex;
    flex-direction: column;
    gap: 18px;
    overflow-y: auto;
  }

  .brand {
    display: flex;
    align-items: center;
    gap: 10px;
    font-size: 19px;
    font-weight: 700;
    padding: 0 6px;
  }

  .logo {
    width: 32px;
    height: 32px;
    border-radius: 9px;
    background: var(--accent);
    color: #fff;
    display: grid;
    place-items: center;
    font-size: 16px;
  }

  nav {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  nav button {
    text-align: left;
    padding: 9px 10px;
    border-radius: 9px;
    color: var(--ink-2);
    font-weight: 500;
  }

  nav button:hover {
    background: var(--surface-2);
  }

  nav button.on {
    background: var(--surface-2);
    color: var(--ink);
  }

  .section-title {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 6px;
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--muted);
  }

  .add {
    width: 24px;
    height: 24px;
    border-radius: 7px;
    font-size: 14px;
    color: var(--ink-2);
  }

  .add:hover {
    background: var(--surface-2);
  }

  .cats {
    list-style: none;
    margin: -10px 0 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .cats li {
    position: relative;
  }

  .cat {
    width: 100%;
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 7px 8px;
    border-radius: 10px;
    text-align: left;
  }

  .cat:hover {
    background: var(--surface-2);
  }

  .cat.on {
    background: color-mix(in srgb, var(--c, var(--accent)) 14%, transparent);
  }

  .cat-icon {
    width: 32px;
    height: 32px;
    flex: none;
    border-radius: 9px;
    display: grid;
    place-items: center;
    font-size: 16px;
    background: color-mix(in srgb, var(--c) 20%, transparent);
  }

  .cat-icon.all {
    background: var(--surface-2);
    color: var(--ink-2);
  }

  .cat-name {
    flex: 1;
    display: flex;
    flex-direction: column;
    font-weight: 500;
    min-width: 0;
  }

  .cat-name small {
    font-size: 11px;
    font-weight: 400;
    color: var(--muted);
  }

  .streak {
    font-size: 12px;
    font-weight: 600;
    color: var(--muted);
    filter: grayscale(1);
  }

  .streak.hot {
    color: var(--ink);
    filter: none;
  }

  /* El lápiz ocupa el sitio de la racha al pasar el ratón. */
  .cats li:hover .streak {
    opacity: 0;
  }

  .edit {
    position: absolute;
    right: 8px;
    top: 50%;
    transform: translateY(-50%);
    width: 26px;
    height: 26px;
    border-radius: 7px;
    opacity: 0;
    color: var(--ink-2);
    background: var(--surface-3);
  }

  .cats li:hover .edit,
  .edit:focus-visible {
    opacity: 1;
  }

  .today-card {
    margin-top: auto;
    background: var(--surface-2);
    border-radius: 12px;
    padding: 12px 14px;
    display: flex;
    flex-direction: column;
    gap: 6px;
    font-size: 12px;
  }

  .today-card strong {
    font-size: 14px;
  }

  .mini-track {
    height: 6px;
    background: var(--surface-3);
    border-radius: 99px;
    overflow: hidden;
  }

  .mini-track div {
    height: 100%;
    background: var(--good);
    border-radius: 99px;
    transition: width 0.3s;
  }

  main {
    display: flex;
    flex-direction: column;
    padding: 20px 24px;
    gap: 16px;
    min-width: 0;
    min-height: 0;
  }

  .top {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  h1 {
    margin: 0;
    font-size: 26px;
    letter-spacing: -0.02em;
  }

  .month {
    text-transform: capitalize;
  }

  .year {
    color: var(--muted);
    font-weight: 500;
  }

  .spacer {
    flex: 1;
  }

  .filter-pill {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 4px 6px 4px 10px;
    border-radius: 99px;
    background: color-mix(in srgb, var(--c) 16%, transparent);
    border: 1px solid color-mix(in srgb, var(--c) 40%, transparent);
    font-weight: 500;
  }

  .filter-pill button {
    width: 20px;
    height: 20px;
    border-radius: 50%;
    font-size: 11px;
  }

  .filter-pill button:hover {
    background: var(--surface-3);
  }

  .cal-layout {
    flex: 1;
    display: grid;
    grid-template-columns: 1fr 290px;
    gap: 16px;
    min-height: 0;
  }

  .cal {
    min-height: 0;
  }

  .day {
    padding: 18px;
    display: flex;
    flex-direction: column;
    gap: 12px;
    overflow-y: auto;
  }

  .day h2 {
    margin: 0;
    font-size: 16px;
  }

  .day h2::first-letter {
    text-transform: uppercase;
  }

  .day ul {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .day-item {
    width: 100%;
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 10px;
    border-radius: 10px;
    border: 1px solid var(--line);
    text-align: left;
    transition: background 0.15s, border-color 0.15s;
  }

  .day-item:not(:disabled):hover {
    border-color: var(--c);
  }

  .day-item:disabled {
    opacity: 0.5;
  }

  .day-item.checked {
    background: color-mix(in srgb, var(--c) 12%, transparent);
    border-color: color-mix(in srgb, var(--c) 50%, transparent);
  }

  .day-item.unscheduled:not(.checked) {
    border-style: dashed;
  }

  .box {
    width: 24px;
    height: 24px;
    flex: none;
    border-radius: 7px;
    border: 2px solid var(--c);
    display: grid;
    place-items: center;
  }

  .checked .box {
    background: var(--c);
  }

  .box svg {
    width: 14px;
    height: 14px;
    stroke: #fff;
    stroke-width: 3.2;
    stroke-linecap: round;
  }

  .day-icon {
    font-size: 18px;
  }

  .day-name {
    display: flex;
    flex-direction: column;
    font-weight: 500;
  }

  .day-name small {
    font-size: 11px;
    font-weight: 400;
    color: var(--muted);
  }

  .hint {
    margin: auto 0 0;
    font-size: 12px;
  }

  .scroll {
    overflow-y: auto;
    min-height: 0;
    margin-right: -12px;
    padding-right: 12px;
  }

  .error {
    background: color-mix(in srgb, var(--critical) 14%, transparent);
    color: var(--critical);
    padding: 8px 12px;
    border-radius: 9px;
    display: flex;
    justify-content: space-between;
  }
</style>
