# Rooms UI Design

**Date:** 2026-03-11

## Goal

Design the `/rooms` and `/room/:roomId` routes as branded, UI-only surfaces that fit the existing session-ledger aesthetic, reuse current shared components, and rely on local stub data until room logic is implemented.

## Context

The current app already has a clear visual identity:

- Parchment-toned shell with editorial framing
- Serif display typography with mono metadata accents
- Panel-based layouts with restrained green and copper accents
- Existing `RollEditor` and `RollFeed` components on the home route

The current `/rooms` and `/room/:roomId` pages are placeholders. This work should replace those placeholders with realistic room surfaces without adding backend behavior.

## Design Direction

Use a `Ledger Stage` direction.

- The experience should feel table-first rather than admin-first.
- `/rooms` should read like a board of active tables, not a settings page.
- `/room/:roomId` should center active play: room identity, composer, and shared roll activity.
- Pending joiners should use `soft presence`, appearing as a secondary list rather than a dominant moderation queue.

## Scope

### In Scope

- UI-only redesign of `/rooms`
- UI-only redesign of `/room/:roomId`
- Protected client-side routing for `/room/:roomId`
- Reading the client route parameter for local stub lookup
- New page-local layout and presentational components as needed
- Local stub data and placeholder copy
- Reuse of existing shared roll components
- SSR coverage for the new room route markup

### Out of Scope

- Real room creation or join flows
- Server-backed route loading or fetches
- Presence synchronization
- Approval actions
- Real-time room activity
- New shared server/client contracts

## Route Design

### `/rooms`

The rooms index should feel like a live staging board.

#### Top Section

Use a prominent hero-style panel with two actions:

- Primary CTA: `Create a room`
- Secondary flow: `Join by room ID`

The join experience should live inside the same surface, using:

- A signal-backed room ID input
- Stub helper text
- A single join button

This section should communicate that rooms are active tables, not just stored records.

Placeholder behavior:

- `Create a room` is a visible primary CTA rendered as a disabled button for this pass
- the disabled create control should include adjacent helper copy that clearly says room creation wiring comes later
- the join button stays disabled whenever the trimmed room ID value is empty, including whitespace-only input
- when the room ID input is non-empty, the join action trims surrounding whitespace, URL-encodes the value for navigation, and routes to `/room/<room-id>` without changing case or implying server validation
- arbitrary non-seeded room IDs should land on the room page's local not-found state

#### Joined Rooms Section

Below the launch area, render only already joined rooms as `active table` cards.

Each card should include:

- Room name
- Room ID
- Live user count
- Short “who is here” preview text
- Recent activity stub text
- Dominant `Enter room` action

Cards should feel like snapshots of a live table rather than neutral list rows.

If the joined-room list is empty, render a branded empty state that points back to the create/join controls rather than leaving a blank section.

### `/room/:roomId`

The room detail page should feel like entering a single active table.

#### Main Column

The main column should contain:

1. Room header
2. Dice editor surface
3. Roll feed

This order preserves the table-first priority:

- identify the table
- compose the next roll
- read the shared ledger

The room page must be reachable from the joined-room cards and registered in the app router as a protected route alongside `/rooms`, reusing the same authenticated-access expectation and redirect behavior as the existing `/rooms` route.

#### Supporting Rail

The supporting rail should contain:

- `Live in room` roster
- `Waiting for approval` roster

The pending list must remain visibly secondary:

- lighter styling
- smaller emphasis
- no dominant moderation framing

## Component Boundaries

Keep boundaries local and focused.

### Reuse

- Reuse [`src/client/components/roll_editor.rs`](/home/nox/code/projects/dice-roller/src/client/components/roll_editor.rs)
- Reuse [`src/client/components/roll_feed.rs`](/home/nox/code/projects/dice-roller/src/client/components/roll_feed.rs)

### Page-Local Components

If the pages grow beyond a clean single file, split them into small page-local presentational units.

Recommended candidates:

- `RoomLaunchPanel`
- `JoinRoomPanel`
- `JoinedRoomCard`
- `LiveRosterCard`
- `PendingRosterCard`
- `RosterMemberRow`

These components should remain UI-only and depend on plain stub data structures.

## Stub Data Model

Keep all temporary data in one client-pages-local stub source for this pass. The preferred boundary is a helper module such as `src/client/pages/room_stubs.rs`, and it should feed both routes without becoming a broader shared contract.

For this UI pass, the seeded room list is the joined-room list. There is no second dataset for discoverable rooms, invitations, or membership lookup.

### Canonical Seeded Room Stubs

Define one canonical page-local room stub shape keyed by room ID exactly as seeded in the local data. The `/rooms` page should derive its summary cards from this source, and `/room/:roomId` should derive its detail view from the same source.

Each seeded room should contain enough fields to support both views, including:

- room title
- room id
- status or room note
- live users
- pending users
- recent activity line
- `DiceRollFeed`-compatible seeded activity data

Seeded activity entries should include stable unique roll IDs so `RollFeed` keyed rendering remains predictable.

Each roster entry should use a minimum shared stub shape so both room rosters and joined-room preview copy derive from the same fields:

- display name
- short presence note or role label
- lightweight status label when needed

### Derived Room Summary Stubs

The `/rooms` page can derive summary card fields such as:

- live user count
- short live-user preview text
- recent activity line

The `/rooms` page should render this list as already-joined rooms only. No broader discoverable-room dataset is needed for this UI pass.

Resolve room detail stubs by decoding the route `roomId`, trimming surrounding whitespace, and matching that value against the local seeded room list. If no local room matches, render a room-specific local not-found state inside the room shell with a short explanation, the attempted room ID in the copy, and a link back to `/rooms`.

The unknown-room state should replace the normal room-content body while preserving the surrounding page shell and back-navigation affordance.

The seeded activity data must stay compatible with the existing `DiceRoll` and `DiceRollFeed` structures so the page can reuse the current room-feed component without adapter churn later.

The goal is to make future wiring straightforward without prematurely defining shared API contracts.

## Interaction Rules

All interaction remains placeholder-level only.

Allowed stub interactions:

- room ID input updates local signal state
- join button stays disabled when the trimmed room ID input is empty
- when the room ID input is non-empty, the join action URL-encodes the trimmed value before routing to `/room/<room-id>` without claiming that access or membership was granted
- helper copy near the join control should state that room validation and membership wiring are still pending
- local room cards link to room detail routes
- room page surfaces display static presence and seeded activity
- the room page composer remains active and local: submitting through `RollEditor` should append new rolls only to the mounted room page's local `DiceRollFeed` state, with no mutation of the canonical seeded room stubs and no cross-route shared client store
- locally appended room rolls must include a client-local unique roll ID plus placeholder attribution and timestamp metadata, following the same basic shape already used by the home-page local feed
- the create-room CTA remains an honest placeholder control with no persistence and no fake success path

Do not add fake async loading, simulated sockets, or placeholder approval business logic. The pages should feel real without pretending the system is wired.

### Empty States

Include lightweight empty states for:

- no joined rooms on `/rooms`
- no pending users on `/room/:roomId`
- unknown room ID on `/room/:roomId`

## Responsive Behavior

### `/rooms`

- On larger screens, use a wide launch panel followed by a grid or stacked board of active-room cards.
- On smaller screens, collapse into a single-column ledger stack.
- The create CTA and join controls must remain immediately visible without squeezing text or controls.
- Joined room metadata should stack into short readable rows on mobile.

### `/room/:roomId`

- Keep the main play column first in both source and visual order.
- Move the presence rail below the main column at narrower widths.
- Preserve clear spacing between room header, editor, roster, and roll feed.

## Styling Direction

Stay inside the existing brand system rather than creating a new one.

### Reuse Global Language

Prefer existing global classes and tokens:

- `g-panel`
- `g-panel-strong`
- `g-section-label`
- `g-section-title`
- `g-section-summary`

### New Page Styling

Add page-local styling in:

- [`src/client/pages/rooms.module.scss`](/home/nox/code/projects/dice-roller/src/client/pages/rooms.module.scss)
- [`src/client/pages/room.module.scss`](/home/nox/code/projects/dice-roller/src/client/pages/room.module.scss)

Visual references:

- ledger cards
- attendance slips
- restrained decorative rules
- subtle gradients and paper-like surfaces
- serif headline, sans body, mono metadata balance

Avoid dashboard-like neon, heavy control chrome, or generic utility-app layouts.

## Testing Strategy

Add SSR tests for both routes to verify the new structure renders.

It is acceptable to extract page-local helper functions or small render seams so SSR tests can inject stub room lists, empty states, and room IDs directly without depending on full router or auth setup. Router verification only needs registration coverage for the new protected room route, not end-to-end auth-flow testing.

### `/rooms` Assertions

Confirm rendered HTML includes:

- create room framing
- join room framing
- joined or active room section content
- empty-state framing when the joined-room list is empty

### `/room/:roomId` Assertions

Confirm rendered HTML includes:

- live roster framing
- pending roster framing
- dice composer framing
- room activity framing
- local room not-found framing for an unknown room ID

If the implementation extracts pure helpers for local room lookup or stub join-state decisions, add unit tests for those helpers. DOM interaction tests are not required for this pass; the required behavior should stay simple and local enough to verify primarily through SSR coverage and small helper tests.

Required helper-level verification should cover:

- join input normalization and disabled-state decisions
- local room lookup by decoded and trimmed `room_id`
- room-local roll submission appending a new entry into the local `DiceRollFeed`

## Implementation Notes

- Follow existing Leptos page patterns from [`src/client/pages/home.rs`](/home/nox/code/projects/dice-roller/src/client/pages/home.rs)
- Keep files focused; split only if it improves readability
- Use page-local stub data instead of introducing cross-cutting abstractions
- Preserve the current editorial shell and spacing rules from [`style/main.scss`](/home/nox/code/projects/dice-roller/style/main.scss)

## Success Criteria

This design is successful if:

- `/rooms` clearly supports create, join-by-id, and joined-room browsing
- `/room/:roomId` clearly supports active play and soft-presence side information
- both routes feel consistent with the current session-ledger brand
- the work remains UI-only and easy to wire later
