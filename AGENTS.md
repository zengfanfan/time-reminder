# Repository Guidelines

## Project Structure & Module Organization

This is a SvelteKit + Tauri desktop app. Frontend code lives in `src/`: shared components and utilities are in `src/lib/`, app routes are in `src/routes/`, and global styles are in `src/app.css`. Rust/Tauri backend code lives in `src-tauri/`, with commands and reminder logic under `src-tauri/src/`. Static assets such as icons and fonts are in `static/`; packaged Tauri icons are in `src-tauri/icons/`. Tests are split into `tests/unit/` and `tests/integration/`.

## Build, Test, and Development Commands

- `bun run dev`: start the Vite/Svelte development server.
- `bun run build`: build the frontend for production.
- `bun run preview`: preview the built frontend.
- `bun run check`: run `svelte-check` with `jsconfig.json`.
- `bun test`: run all Node tests plus Rust Cargo tests via the package script.
- `bun run test:unit`: run unit tests and Rust tests filtered by `unit_`.
- `bun run test:integration`: run integration tests and Rust tests filtered by `integration_`.
- `bun run tauri`: run Tauri CLI commands.

## Coding Style & Naming Conventions

Use ES modules and Svelte 5 conventions. JavaScript is checked with TypeScript via `jsconfig.json`, so keep exported functions and data shapes clear. Prefer small, focused modules in `src/lib/`; keep Tauri command wrappers centralized, such as `src/lib/reminders.js`. Use `camelCase` for JavaScript variables/functions, `PascalCase` for Svelte components, and Rust `snake_case` for functions and files. Rust formatting is controlled by `rustfmt.toml`.

## Testing Guidelines

Frontend tests use Node's built-in test runner through Bun package scripts. Name test files `*.test.js` and place fast logic tests in `tests/unit/`; tests that mock or exercise command boundaries belong in `tests/integration/`. Rust tests run through Cargo using `src-tauri/Cargo.toml`. Before changing reminder behavior, locale handling, or Tauri commands, run at least the relevant `bun run test:unit` or `bun run test:integration`; run `bun test` before release-oriented changes.

## Commit & Pull Request Guidelines

Recent history uses short, imperative commit messages, for example `hide context menu in reminder page` and `remove unused config`. Keep commits focused on one behavior or fix. Pull requests should include a concise summary, test commands run, linked issue or motivation when available, and screenshots or screen recordings for visible UI changes.

## Security & Configuration Tips

Do not commit generated build folders such as `.svelte-kit/`, `build/`, `node_modules/`, or `src-tauri/target/`. Keep Tauri permissions scoped in `src-tauri/capabilities/default.json`, and avoid adding dependencies with restrictive licenses such as GPL unless explicitly approved.
