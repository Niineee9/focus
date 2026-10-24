# Focus

Calendario de hábitos de escritorio: categorías con días objetivo, marcas con ✕, rachas y métricas.

**Stack:** Tauri 2 · Rust (SQLite vía `rusqlite`, `chrono`) · Svelte 5 + TypeScript + Vite.

## Uso

```sh
npm install
npm run tauri dev      # desarrollo
npm run tauri build    # instalador (.msi / .exe) en src-tauri/target/release/bundle
cd src-tauri && cargo test   # tests de rachas y métricas
```

Los datos se guardan en `%APPDATA%\com.focus.habits\focus.db`.

## Cómo cuentan las rachas

- Solo cuentan los **días programados** de cada categoría; los días libres no rompen la racha.
- Si hoy toca y aún no lo marcas, la racha sigue viva hasta que termine el día.
- Marcar un día libre cuenta como **extra** (suma al total, no a la racha).
- **Racha perfecta:** días seguidos en los que cumpliste *todas* las categorías programadas.

## Estructura

- `src-tauri/src/db.rs` — esquema SQLite, CRUD de categorías y marcas.
- `src-tauri/src/stats.rs` — cálculo puro de rachas, porcentajes e historial (con tests).
- `src-tauri/src/lib.rs` — comandos Tauri expuestos a la interfaz.
- `src/App.svelte` — barra lateral, calendario y panel del día.
- `src/lib/` — `Calendar`, `StatsView`, `CategoryModal` y utilidades.
