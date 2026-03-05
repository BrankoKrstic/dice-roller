# dice-roller

A Leptos + Axum web app for dice expression rolling with a custom dice DSL.

## Current status

Implemented so far:
- Dice roller UI with two input modes (builder and expression)
- Builder mode includes clickable dice cards and adv/dis shortcuts
- Expression mode supports free-form DSL input
- Local roll feed with timestamp, total, expression, and roll breakdown
- `/chance` page with calculator UI controls (simulation backend not wired yet)
- Full lexer/parser/interpreter pipeline for the dice DSL with unit tests
- SSR server + hydration setup via `cargo-leptos`

Not implemented yet:
- Server API endpoints (files exist but are currently empty)
- Persistent storage / rooms / auth pages (files exist but are currently placeholders)
- Real-time multiplayer roll feed

## Tech stack

- Rust 2024
- Leptos 0.8 (`leptos`, `leptos_router`, `leptos_meta`, `leptos_axum`)
- Axum 0.8 (SSR server)
- Stylance + SCSS modules (component styling)
- `cargo-leptos` for SSR/hydrate build flow
- Playwright scaffold for end-to-end tests

## Routes

- `/` -> Roller page
- `/chance` -> Chance calculator page (UI scaffold)
- Any other path -> Not found page

## Dice DSL (supported today)

Core:
- Arithmetic: `+ - * /` and parentheses
- Dice: `d4 d6 d8 d10 d12 d20 d% dF`
- Implicit one die: `d20` == `1d20`

Modifiers (implemented in parser/interpreter):
- `adv`, `dis` (d20 only, single-die expressions)
- `k` (keep) and `d` (drop) with conditions (`kh2`, `dl1`, `d>=5`, etc.)
- `r` (reroll), optional `timesN` cap (`r<=2times3`)
- `ex` (explode), optional `timesN` cap (`ex=6times2`)
- `u` (unique)
- `c` (count), with optional condition (`c>=5`)
- `s` / `sa` (sort descending / ascending)
- `minN`, `maxN`

Examples:
- `2d10 + 1d6 + 5`
- `4d6kh3`
- `2d6r<=3times2 + 1`
- `d20adv + 7`
- `4dFmin0max1`

## Prerequisites

- Rust toolchain (`cargo`, `rustup`)
- `cargo-leptos`
- `stylance-cli`

Install helper tools:

```bash
cargo install cargo-leptos --locked
cargo install stylance-cli
```

## Development

Start CSS + Leptos watch mode:

```bash
./watch.sh
```

This runs:
- `stylance --watch .`
- `cargo leptos watch`

Default local server address comes from `Cargo.toml` metadata:
- `127.0.0.1:3000`

## Tests

Run Rust tests:

```bash
cargo test
```

Current status in this workspace:
- 31 Rust tests passing (DSL lexer/parser/interpreter coverage)

## Production build

```bash
./build.sh
```

This runs:
- `stylance .`
- `cargo leptos build --release`

Artifacts:
- Server binary in `target/release/`
- Site assets in `target/site/`

## Deploying without Rust toolchain

After `./build.sh`, copy:
- server binary (from `target/release/`)
- `target/site/` directory

Expected runtime env vars (adjust as needed):

```bash
export LEPTOS_OUTPUT_NAME="dice-roller"
export LEPTOS_SITE_ROOT="site"
export LEPTOS_SITE_PKG_DIR="pkg"
export LEPTOS_SITE_ADDR="127.0.0.1:3000"
export LEPTOS_RELOAD_PORT="3001"
```

Then run the server binary.
