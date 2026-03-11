# Rooms UI Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build branded, UI-only `/rooms` and `/room/:roomId` routes that reuse the existing roll components, use local room stubs, and stay aligned with the approved session-ledger design.

**Architecture:** Keep the temporary room data in one page-local stub module under `src/client/pages/`, derive both the rooms index and room detail views from that source, and keep all behavior local to the mounted page. Reuse `RollEditor` and `RollFeed` directly, add only page-scoped helpers and styling, and mirror the existing `/rooms` protected-route behavior for `/room/:roomId`.

**Tech Stack:** Rust 2024, Leptos 0.8, `leptos_router`, Stylance SCSS modules, existing `DiceRoll` / `DiceRollFeed` utilities, SSR unit tests via `cargo test`

---

## Chunk 1: Shared Room Stubs, Routing, And `/rooms`

### Task 1: Create The Shared Page-Local Room Stub Module

**Files:**
- Create: `src/client/pages/room_stubs.rs`
- Modify: `src/client/pages/mod.rs`

- [ ] **Step 1: Wire the new stub module into the pages module tree**

Update `src/client/pages/mod.rs` to declare `mod room_stubs;` before adding any tests to `src/client/pages/room_stubs.rs`, so the new file is compiled during the TDD cycle.

- [ ] **Step 2: Write the failing unit tests for room lookup and join normalization**

Add tests in `src/client/pages/room_stubs.rs` for:
- trimmed empty join input disables navigation
- non-empty join input produces a URL-encoded room target while preserving the user-entered case after trimming
- seeded room lookup finds a known room by decoded-and-trimmed room ID with exact seeded-ID matching
- seeded room lookup returns `None` for an unknown room ID

Expected test names:
- `join_target_is_none_for_blank_input`
- `join_target_url_encodes_trimmed_room_id`
- `find_room_by_id_matches_seeded_room`
- `find_room_by_id_returns_none_for_unknown_room`

- [ ] **Step 3: Run the new unit tests to confirm they fail**

Run: `cargo test join_target_is_none_for_blank_input -- --nocapture`

Expected: FAIL because the new helper functions and room-stub types are not implemented yet.

- [ ] **Step 4: Define the canonical room and roster stub structs**

In `src/client/pages/room_stubs.rs`, define:
- the canonical room stub shape
- the shared roster-entry shape

Include fields for:
- room title
- room id
- status or room note
- live users
- pending users
- recent activity line
- roster display name
- roster presence note or role label
- optional lightweight status label

- [ ] **Step 5: Add the seeded room data and activity entries**

Populate:
- a page-local seeded room list
- seeded activity data that is already compatible with `DiceRoll` / `DiceRollFeed`
- stable unique roll IDs for seeded activity entries

Keep the module page-local; do not move it into `src/shared/`. Export only the items needed by sibling page modules.

- [ ] **Step 6: Add the summary and join-target helpers**

Implement:
- helpers for deriving room summaries from the canonical list
- a helper that trims join input and returns an encoded room route target only when the trimmed value is non-empty, preserving the entered case

- [ ] **Step 7: Add the room lookup helper**

Implement a helper that decodes and trims a route room ID before matching the seeded room list exactly, without case normalization.

- [ ] **Step 8: Run the room stub tests to confirm they pass**

Run: `cargo test room_stubs -- --nocapture`

Expected: PASS for the new helper tests in `src/client/pages/room_stubs.rs`.

- [ ] **Step 9: Commit the stub-module foundation**

```bash
git add src/client/pages/room_stubs.rs src/client/pages/mod.rs
git commit -m "add room ui stubs"
```

### Task 2: Register The Room Detail Route And Module Wiring

**Files:**
- Modify: `src/client/pages/mod.rs`

- [ ] **Step 1: Add the new page-local module wiring**

Update `src/client/pages/mod.rs` to:
- import `room::RoomPage`

- [ ] **Step 2: Add the protected `/room/:roomId` route**

Mirror the existing `/rooms` protected route pattern:
- use `ProtectedRoute`
- guard on the same auth condition
- redirect unauthenticated users to `/`

Leave unknown-room local not-found rendering to Chunk 2; this chunk only registers the route and auth gate.

- [ ] **Step 3: Verify the app compiles with the new route registration**

Run: `cargo test room_stubs -- --nocapture`

Expected: PASS and successful compilation with the new route path in `src/client/pages/mod.rs`.

- [ ] **Step 4: Commit the route wiring**

```bash
git add src/client/pages/mod.rs
git commit -m "wire room detail route"
```

### Task 3: Replace The `/rooms` Placeholder With The Active-Tables UI

**Files:**
- Modify: `src/client/pages/rooms.rs`
- Modify: `src/client/pages/rooms.module.scss`
- Test: `src/client/pages/rooms.rs`

- [ ] **Step 1: Write the failing SSR tests for the rooms page**

Add SSR tests in `src/client/pages/rooms.rs` that assert the rendered page includes:
- create-room framing
- join-by-ID framing
- active/joined rooms framing
- empty-state framing when no joined rooms are injected
- a disabled join action when the trimmed room ID is blank or whitespace-only
- a populated join control that exposes the encoded room-detail target for a trimmed non-empty room ID
- an `Enter room` action on joined-room cards that points at the matching room-detail route
- room name content on joined-room cards
- room ID content on joined-room cards
- live user count on joined-room cards
- who-is-here preview text on joined-room cards
- recent activity stub text on joined-room cards

Expected test names:
- `rooms_page_renders_launch_and_join_sections`
- `rooms_page_renders_joined_room_cards`
- `rooms_page_renders_empty_state_when_no_joined_rooms`
- `rooms_page_exposes_encoded_join_target_for_trimmed_room_id`
- `rooms_page_links_joined_room_cards_to_room_detail`

Use a page-local render seam or helper arguments so tests can inject seeded or empty room lists without depending on auth or full router setup.

- [ ] **Step 2: Run one rooms-page test to confirm it fails**

Run: `cargo test rooms_page_renders_launch_and_join_sections -- --nocapture`

Expected: FAIL because `/rooms` still renders the placeholder content.

- [ ] **Step 3: Implement the new `/rooms` page layout**

Rewrite `src/client/pages/rooms.rs` to render:
- a top launch panel with a disabled create CTA and helper copy
- a signal-backed join-by-ID input with disabled-state logic for trimmed empty input
- a join flow that exposes the encoded `/room/<room-id>` target for trimmed non-empty input
- join-control helper copy that states room validation and membership wiring are still pending
- joined-room cards derived from `room_stubs.rs`
- `Enter room` actions that link each card to its room-detail route
- room name on each joined-room card
- room ID, live user count, who-is-here preview text, and recent activity text on each joined-room card
- an empty state for zero joined rooms

Use honest placeholder copy; do not imply persistence or real membership changes.

- [ ] **Step 4: Implement the `/rooms` page styling**

Update `src/client/pages/rooms.module.scss` to support:
- the hero launch surface
- the join controls
- the active-table card layout
- compact metadata rows and status styling
- the empty-state treatment

Reuse existing global panel and section classes where possible instead of recreating the app shell.

- [ ] **Step 5: Run the targeted rooms-page tests**

Run: `cargo test rooms_page_ -- --nocapture`

Expected: PASS for the new `/rooms` SSR tests.

- [ ] **Step 6: Commit the `/rooms` page implementation**

```bash
git add src/client/pages/rooms.rs src/client/pages/rooms.module.scss
git commit -m "build rooms index ui"
```

## Chunk 2: `/room/:roomId`, Local Feed Behavior, And Verification

### Task 4: Add Local Room Feed Helpers For Appended Rolls

**Files:**
- Modify: `src/client/pages/room_stubs.rs`
- Test: `src/client/pages/room_stubs.rs`

- [ ] **Step 1: Write the failing helper test for local room roll appends**

Add a unit test in `src/client/pages/room_stubs.rs` for local room-roll creation or append behavior.

Expected test names:
- `appended_room_roll_has_unique_id_and_local_metadata`
- `appending_room_roll_does_not_mutate_seeded_room_feed`

The assertion should check:
- a unique client-local roll ID is generated
- placeholder attribution is present
- timestamp metadata is present
- the new entry can be added to a `DiceRollFeed` without replacing existing seeded items
- appending through page-local cloned feed state does not mutate the canonical seeded room stub data

- [ ] **Step 2: Run the new helper test to confirm it fails**

Run: `cargo test appended_room_roll_has_unique_id_and_local_metadata -- --nocapture`

Expected: FAIL because the local roll helper does not exist yet.

- [ ] **Step 3: Implement the local room-roll helper**

In `src/client/pages/room_stubs.rs`, add a pure helper that follows the same basic local-feed shape as `src/client/pages/home.rs`:
- build and return a `DiceRoll` entry for a submitted room roll
- generate a unique client-local roll ID
- apply placeholder attribution
- format the timestamp

Keep feed ordering consistent with the existing home-page local feed behavior.

- [ ] **Step 4: Run the stub-module test set**

Run: `cargo test room_stubs -- --nocapture`

Expected: PASS for all room-stub helper tests, including the new roll-append coverage.

- [ ] **Step 5: Commit the local feed helper**

```bash
git add src/client/pages/room_stubs.rs
git commit -m "add local room feed helpers"
```

### Task 5: Replace The `/room/:roomId` Placeholder With The Room Table UI

**Files:**
- Modify: `src/client/pages/room.rs`
- Modify: `src/client/pages/room.module.scss`
- Test: `src/client/pages/room.rs`

- [ ] **Step 1: Write the failing SSR tests for the room detail page**

Add SSR tests in `src/client/pages/room.rs` that assert:
- a known room renders the room header with room title, room ID, and status/note
- a known room renders the shared room shell with back navigation to `/rooms`
- a known room renders the live roster, pending roster, roll editor framing, and room activity framing
- a known room renders at least one seeded live-roster member name from `room_stubs.rs`
- a known room renders at least one seeded activity entry from `room_stubs.rs`
- a known room preserves the table-first main-column order of room header, dice editor, then roll feed before the supporting rail content
- a room with no pending users renders the pending-roster empty state
- an unknown room renders the preserved room shell with a not-found state, the attempted room ID in copy, and a link back to `/rooms`

Expected test names:
- `room_page_renders_table_first_layout_for_known_room`
- `room_page_renders_not_found_state_for_unknown_room`

Use a page-local render seam so tests can inject a known room or unknown room result without depending on `ProtectedRoute`.

- [ ] **Step 2: Run one room-page test to confirm it fails**

Run: `cargo test room_page_renders_table_first_layout_for_known_room -- --nocapture`

Expected: FAIL because `/room/:roomId` still renders the placeholder hero.

- [ ] **Step 3: Implement the room detail layout**

Rewrite `src/client/pages/room.rs` to:
- decode and trim the route parameter
- look up the room via `room_stubs.rs`
- render a shared room shell with back navigation to `/rooms`
- render the table-first main column with room header, `RollEditor`, and `RollFeed`
- render the supporting rail with live users and softly-emphasized pending users
- replace the main room body with the local not-found state when the room ID is unknown

- [ ] **Step 4: Wire local room-feed state into the reused roll components**

In `src/client/pages/room.rs`:
- clone the matched room stub's seeded `DiceRollFeed` into page-local `RwSignal<DiceRollFeed>` state
- pass that feed into `RollFeed`
- provide local placeholder `loading_more` and `load_older_rolls` props to `RollFeed`, matching the current local-only pattern from `src/client/pages/home.rs`
- use the local room-roll helper when `RollEditor` submits a new expression
- append new room rolls only to the mounted page state

- [ ] **Step 5: Implement the room-detail styling**

Update `src/client/pages/room.module.scss` to support:
- the room header
- the two-column table-first layout
- the presence rail cards
- roster rows and status treatments
- the room-shell not-found state
- responsive collapse to a single-column stack

- [ ] **Step 6: Run the targeted room-page tests**

Run: `cargo test room_page_ -- --nocapture`

Expected: PASS for the new `/room/:roomId` SSR tests.

- [ ] **Step 7: Commit the room detail page implementation**

```bash
git add src/client/pages/room.rs src/client/pages/room.module.scss
git commit -m "build room detail ui"
```

### Task 6: Run Final Verification For The Rooms UI Pass

**Files:**
- Verify only

- [ ] **Step 1: Run the focused room-page and stub tests together**

Run:
- `cargo test room_stubs -- --nocapture`
- `cargo test rooms_page_ -- --nocapture`
- `cargo test room_page_ -- --nocapture`

Expected: PASS for room stub, `/rooms`, and `/room/:roomId` tests added in this plan.

- [ ] **Step 2: Run the full Rust test suite**

Run: `cargo test`

Expected: PASS with the existing DSL tests plus the new rooms UI coverage.

- [ ] **Step 3: Inspect the final diff before handoff**

Run: `git status --short`

Expected: no unexpected uncommitted changes from the rooms UI work remain. Ignore unrelated pre-existing workspace changes if present.
