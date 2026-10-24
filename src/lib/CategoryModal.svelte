<script lang="ts">
  import type { Category, CategoryInput } from "./api";
  import { PALETTE, shade } from "./colors.svelte";
  import { ALL_DAYS, WEEKDAYS, WEEKEND, WEEKDAYS_SHORT } from "./dates";

  interface Props {
    category: Category | null;
    onsave: (input: CategoryInput) => Promise<void>;
    ondelete: () => Promise<void>;
    onclose: () => void;
  }

  let { category, onsave, ondelete, onclose }: Props = $props();

  const ICONS = ["🏋️", "🏃", "🧘", "🥗", "💧", "📚", "🇬🇧", "✍️", "🎸", "💤", "🧠", "💊", "🚭", "💰", "🧹", "🙏"];

  // El modal se monta de nuevo en cada apertura: basta el valor inicial.
  // svelte-ignore state_referenced_locally
  let name = $state(category?.name ?? "");
  // svelte-ignore state_referenced_locally
  let icon = $state(category?.icon ?? ICONS[0]);
  // svelte-ignore state_referenced_locally
  let color = $state(category?.color ?? PALETTE[0].light);
  // svelte-ignore state_referenced_locally
  let daysMask = $state(category?.daysMask ?? ALL_DAYS);
  let error = $state("");
  let confirmDelete = $state(false);
  let saving = $state(false);

  const presets = [
    { label: "Todos", mask: ALL_DAYS },
    { label: "Lun–Vie", mask: WEEKDAYS },
    { label: "Fin de semana", mask: WEEKEND },
  ];

  async function submit(e: SubmitEvent) {
    e.preventDefault();
    if (!name.trim()) return (error = "Ponle un nombre");
    if (!daysMask) return (error = "Elige al menos un día");
    saving = true;
    try {
      await onsave({ name: name.trim(), icon, color, daysMask });
    } catch (err) {
      error = String(err);
      saving = false;
    }
  }

  function onkeydown(e: KeyboardEvent) {
    if (e.key === "Escape") onclose();
  }
</script>

<svelte:window {onkeydown} />

<div class="backdrop" onclick={(e) => e.target === e.currentTarget && onclose()} role="presentation">
  <form
    class="card modal"
    onsubmit={submit}
    aria-label={category ? "Editar categoría" : "Nueva categoría"}
  >
    <header>
      <span class="preview" style:background={shade(color)}>{icon}</span>
      <h2>{category ? "Editar categoría" : "Nueva categoría"}</h2>
    </header>

    <label class="field">
      <span>Nombre</span>
      <!-- svelte-ignore a11y_autofocus -->
      <input bind:value={name} placeholder="Ej. Meditación" maxlength="40" autofocus />
    </label>

    <div class="field">
      <span>Icono</span>
      <div class="icons">
        {#each ICONS as i}
          <button type="button" class:on={icon === i} onclick={() => (icon = i)}>{i}</button>
        {/each}
        <input class="custom" bind:value={icon} maxlength="4" aria-label="Icono personalizado" />
      </div>
    </div>

    <div class="field">
      <span>Color</span>
      <div class="swatches">
        {#each PALETTE as p}
          <button
            type="button"
            class:on={color === p.light}
            style:--c={shade(p.light)}
            title={p.name}
            aria-label={p.name}
            onclick={() => (color = p.light)}
          ></button>
        {/each}
      </div>
    </div>

    <div class="field">
      <span>Días objetivo</span>
      <div class="days">
        {#each WEEKDAYS_SHORT as d, i}
          <button
            type="button"
            class:on={(daysMask & (1 << i)) !== 0}
            style:--c={shade(color)}
            onclick={() => (daysMask ^= 1 << i)}>{d}</button
          >
        {/each}
      </div>
      <div class="presets">
        {#each presets as p}
          <button type="button" class:on={daysMask === p.mask} onclick={() => (daysMask = p.mask)}>
            {p.label}
          </button>
        {/each}
      </div>
    </div>

    {#if error}<p class="error">{error}</p>{/if}

    <footer>
      {#if category}
        {#if confirmDelete}
          <button type="button" class="btn danger" onclick={ondelete}>¿Seguro? Se borra el historial</button>
        {:else}
          <button type="button" class="btn danger" onclick={() => (confirmDelete = true)}>Eliminar</button>
        {/if}
      {/if}
      <span class="spacer"></span>
      <button type="button" class="btn" onclick={onclose}>Cancelar</button>
      <button type="submit" class="btn primary" disabled={saving}>Guardar</button>
    </footer>
  </form>
</div>

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.45);
    backdrop-filter: blur(3px);
    display: grid;
    place-items: center;
    z-index: 50;
    animation: fade 0.15s ease-out;
  }

  .modal {
    width: min(460px, calc(100vw - 32px));
    padding: 22px;
    display: flex;
    flex-direction: column;
    gap: 18px;
    animation: pop 0.18s ease-out;
  }

  header {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  h2 {
    margin: 0;
    font-size: 18px;
  }

  .preview {
    width: 40px;
    height: 40px;
    border-radius: 11px;
    display: grid;
    place-items: center;
    font-size: 20px;
  }

  .field {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .field > span {
    font-size: 12px;
    font-weight: 600;
    color: var(--ink-2);
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }

  input {
    height: 38px;
    padding: 0 12px;
    border-radius: 9px;
    border: 1px solid var(--line-strong);
    background: var(--surface-2);
  }

  .icons {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }

  .icons button {
    width: 36px;
    height: 36px;
    border-radius: 9px;
    font-size: 18px;
    background: var(--surface-2);
    border: 1px solid transparent;
  }

  .icons button.on {
    border-color: var(--accent);
    background: var(--surface-3);
  }

  .icons .custom {
    width: 64px;
    height: 36px;
    text-align: center;
  }

  .swatches {
    display: flex;
    gap: 8px;
  }

  .swatches button {
    width: 30px;
    height: 30px;
    border-radius: 50%;
    background: var(--c);
    box-shadow: 0 0 0 2px var(--surface);
  }

  .swatches button.on {
    box-shadow: 0 0 0 2px var(--surface), 0 0 0 4px var(--c);
  }

  .days {
    display: grid;
    grid-template-columns: repeat(7, 1fr);
    gap: 6px;
  }

  .days button {
    height: 36px;
    border-radius: 9px;
    background: var(--surface-2);
    border: 1px solid var(--line);
    font-weight: 500;
    color: var(--ink-2);
  }

  .days button.on {
    background: var(--c);
    border-color: transparent;
    color: #fff;
  }

  .presets {
    display: flex;
    gap: 6px;
  }

  .presets button {
    font-size: 12px;
    padding: 4px 10px;
    border-radius: 99px;
    border: 1px solid var(--line);
    color: var(--ink-2);
  }

  .presets button.on {
    border-color: var(--accent);
    color: var(--ink);
  }

  .error {
    margin: 0;
    color: var(--critical);
  }

  footer {
    display: flex;
    gap: 8px;
  }

  .spacer {
    flex: 1;
  }

  @keyframes fade {
    from {
      opacity: 0;
    }
  }

  @keyframes pop {
    from {
      transform: scale(0.96);
      opacity: 0;
    }
  }
</style>
