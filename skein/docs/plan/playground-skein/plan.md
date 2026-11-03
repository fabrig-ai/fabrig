# playground-skein: the plan

skein's playground kit, `skein/docs/design/playground.md`, sections 2 to 8, cut for fab by
one lead call (2026-11-03). The shapes that cross tasks are `notes/contracts.md`; the
standing rules `notes/common.md`.

## Design written by this cut

Compatible detail only, in `skein/docs/design/playground.md`:

- 5: a speed in thousandths (1000 is 1×, 0 paused; a world starts at 1×); a slice's posts in
  order, its clock last, so what the page heard before a clock is all the world said up to
  it.
- 6.1 (new): the log's text, version 1: its first line, an input's line and its text forms,
  the `end` or `cut` line, and the digest of what the world said.
- 4: fetch's reports by construction, the headers a same-origin fetch sends, and what the
  jar reads of a `Set-Cookie` (`Max-Age`, not `Expires`).
- 6 and 6.2 (new): the replay's loop is skein-playground's, the harness's run's steps without
  its verdicts (a product crate names no harness, and a visit's world never settles), stepping
  as the slices step so its step counts are the browser's; the referee observes through
  `Turns`.
- 7: the worker's files in the bundle (`boot-worker.js`, `<worker>.js`,
  `<worker>_bg.wasm`, shared snippets) and the refusals.

## Parts and tasks

| Part | Task | Needs | Batch |
|---|---|---|---|
| ui-web | `ui-web-surface`: the bridge's messages, the platform's two duties, the modules laid down | | 1 |
| bundle | `bundle-worker`: a bundle's worker module | | 1 |
| playground | `playground-crate`: the crate and its world, laid down | | 1 |
| ui-web | `ui-web-log`: the session log, its text and its keeping | ui-web-surface | 2 |
| ui-web | `ui-web-worker-page`: the worker's platform, its pure half | ui-web-surface | 2 |
| playground | `playground-slices`: the slices' loop | playground-crate, ui-web-surface | 2 |
| visitor | `playground-visitor`: the visitor and its world | playground-crate, ui-web-surface | 2 |
| ui-web | `ui-web-worker-web`: the worker's platform, its page half (wasm32) | ui-web-worker-page, ui-web-log | 3 |
| memory | `playground-memory`: the bridge in memory, the meeting point natively | playground-slices, playground-visitor, ui-web-worker-page, ui-web-log | 3 |
| replay | `playground-replay`: a session log replayed natively | playground-slices, playground-visitor, ui-web-log | 3 |
| playground | `playground-worker`: the worker's parts (wasm32) | playground-slices | 3 |
| status | `status`: the status files, once built | every task | 3 |

The plan is whole: playground.md's sections 2 to 8 are each built by a task above. Section 8's
browser tier is the application's (temper's `playground-temper`), and its wasm32 gate line is
fab's.

## One writer per file

`skein-ui-web` and `skein-playground` each take two tasks at once in a later wave, on
disjoint files: the crate's first task (`ui-web-surface`, `playground-crate`) lays down every
module the later tasks fill, so none of them edits another's file. This departs from one
writer per crate, for the parallelism the kit's two families need; each spec lists its files.

## Overlaps with the drafts beside this one

- The root `Cargo.toml` (`playground-crate`'s three lines): other plans may add members;
  the lines are local.
- `jig/crates/jig-client-shell-web/src/tests/fixtures.rs` (`ui-web-surface`: the two trait
  methods of the fake platform): no other draft lists it.
- Nothing else of this plan's files is in another draft's list.

## Critical path

`playground-crate` (40) → `playground-visitor` (180) → `playground-replay` (120) → `status`
(25): about six hours of implementer time. Beside it, `ui-web-surface` (75) →
`ui-web-log` (120) → `ui-web-worker-web` (90), and `playground-slices` (120) →
`playground-memory` (90) and `playground-worker` (75). The visitor is the longest task; the
memory task, the meeting point of both real sides natively, waits for it.

## Waves

1. `ui-web-surface`, `bundle-worker`, `playground-crate`.
2. `ui-web-log` and `ui-web-worker-page` (skein-ui-web, disjoint files); `playground-slices`
   and `playground-visitor` (skein-playground, disjoint files).
3. `ui-web-worker-web`; `playground-worker` once the slices land; `playground-memory` and
   `playground-replay` once the visitor lands.
4. `status`.

## Outside this repository

- The gate's wasm32 clippy list is fab's (`skein/docs/design/testing.md`, 10 names "the
  playground's kit"; `playground.md`, 8): `skein-playground` joins it once
  `playground-crate` lands.
