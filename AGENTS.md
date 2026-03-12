# Repository Guidelines

## Project Structure & Module Organization
Core Rust code lives in `src/`. Use `src/client/` for Leptos UI pages, components, and client-side utilities; `src/server/` for Axum handlers, services, DB code, and server-only structures; `src/dsl/` for the dice lexer, parser, and interpreter; and `src/shared/` for data types and utilities used across both sides. Static assets live in `public/`, SCSS and the Stylance bundle live in `style/`, and browser tests belong in `end2end/`.

## Build, Test, and Development Commands
Run `./watch.sh` for local development; it starts `stylance --watch .` and `cargo leptos watch` against `127.0.0.1:3000`. Run `cargo test` to execute Rust unit tests, especially for the DSL pipeline. Use `./build.sh` for a production build; it bundles styles and runs `cargo leptos build --release`. For end-to-end coverage, install the `end2end/` dependencies and run `npx playwright test` from that directory.

## Coding Style & Naming Conventions
Follow standard Rust formatting and keep code `cargo fmt` clean. Prefer small, focused modules that match the existing split between `client`, `server`, `dsl`, and `shared`. Use `snake_case` for files, modules, and functions; `CamelCase` for types and Leptos components; and keep SCSS module names aligned with their component, for example `nav_bar.rs` with `nav_bar.module.scss`.
Shared utility classes belong in `style/main.scss` and must use the `g-` prefix with single-dash names only, for example `g-panel-strong` or `g-button-action`. Page- and component-specific classes belong in matching `*.module.scss` files beside their Rust owners and should be referenced through Stylance imports rather than plain string class names.

## Testing Guidelines
Add unit tests close to the behavior they validate, especially under the DSL parser/interpreter modules. Name tests by behavior, for example `parses_advantage_roll` or `rerolls_with_cap`. When adding UI or route behavior that spans client and server boundaries, add or extend Playwright coverage in `end2end/`.

## Commit & Pull Request Guidelines
Recent history uses short imperative commit subjects like `add dark mode toggle` and `format`. Keep commits focused and descriptive in that style. Pull requests should explain the user-visible change, list any new commands or env vars, and include screenshots or short recordings for UI changes. Link related issues when applicable.

## Architecture Status
The roller UI, dice DSL, SSR wiring, styling pipeline, `/chance` simulation UI, and `/reference` guide are active code paths. Auth is also wired end to end enough for local use: `src/server/api/auth.rs`, `src/server/services/auth.rs`, and the login/register pages back the cookie-based session flow.

The rooms surface is active on the client, but still local-first: `/rooms` and `/room/:roomId` render seeded room summaries, roster state, and append-only local room activity without real persistence, membership approval actions, or multiplayer sync. Some server-facing areas remain scaffolds only, especially `src/server/api/roll.rs`, `src/server/api/router.rs`, and parts of `src/server/services/rooms.rs` / `src/server/services/presets.rs`. Verify behavior before extending those modules, and document any newly activated paths in `README.md`.
