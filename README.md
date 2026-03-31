# dice-roller

A Leptos + Axum web app for dice rolling, room-focused play, authentication, and a custom dice DSL.

## Current status

Implemented so far:
- Home roller with Dice Bench and Expression Editor input modes
- Builder mode includes clickable dice cards and adv/dis shortcuts
- Local roll feeds with timestamp, total, expression, and roll breakdown
- `/chance` Probability Ledger with a worker-backed simulation flow
- `/reference` Expression Editor Guide sourced from the parser-supported syntax
- Email/password auth endpoints plus login/register pages and cookie-backed session checks
- Server-side room persistence service with migrations for rooms, memberships, rolls, and room-roll links
- Protected `/rooms` board backed by persisted room summaries, room ID join routing, and live member counts
- Protected `/room/:roomId` detail view with live/pending roster panels, persisted room activity, membership moderation, and SSE-driven updates
- Full lexer/parser/interpreter pipeline for the dice DSL with unit tests
- SSR server + hydration setup via `cargo-leptos`

Still stubbed or local-only:
- Dedicated standalone roll API outside the room/preset surfaces
- Advanced multiplayer features beyond the current SSE room presence and roll stream
- Expanded end-to-end browser coverage for the authenticated room flows

## Tech stack

- Rust 2024
- Leptos 0.8 (`leptos`, `leptos_router`, `leptos_meta`, `leptos_axum`)
- Axum 0.8 (SSR server)
- libSQL / Turso-compatible storage
- Stylance + SCSS modules (component styling)
- `cargo-leptos` for SSR/hydrate build flow
- Playwright scaffold for end-to-end tests

## Routes

- `/` -> Roller page
- `/chance` -> Probability Ledger simulation page
- `/reference` -> Expression Editor syntax guide
- `/login` -> Login page
- `/register` -> Registration page
- `/rooms` -> Protected rooms board
- `/room/:roomId` -> Protected room detail page
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
- 43 Rust tests passing

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
export TURSO_DATABASE_URL="/absolute/path/to/dice-roller.db"
export JWT_SECRET="replace-me"
export AUTH_COOKIE_SECURE="false"
```

Optional runtime env vars:

```bash
export TURSO_AUTH_TOKEN="required for remote libsql/turso URLs"
export JWT_EXP_SECONDS="604800"
```

Notes:
- `TURSO_DATABASE_URL` can point at a local SQLite/libSQL file or a remote `libsql://`/`https://` URL.
- `TURSO_AUTH_TOKEN` is only required for remote database URLs.
- Set `AUTH_COOKIE_SECURE="true"` when serving over HTTPS.

Then run the server binary.
