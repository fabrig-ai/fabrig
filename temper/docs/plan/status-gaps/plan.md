# status-gaps: the plan

From `inputs/request.md` (the director, 2026-11-02): six code gaps the normalized status files
record. One part per item, each in its own tree; they share nothing and run in parallel. Low
priority beside the ladder's repairs. The status files are the spec; each task updates the
design and status text its change makes true (`notes/common.md`).

## Parts and tasks

1. **readiness-eintr** (skein, `skein/docs/design/shell.md, 7.4`), slice 3: **a question**
   (`readiness-eintr`). shell.md 10 leaves the rule open; the request asks first. Recommended:
   complete all four `Other(EINTR)` as the ring does. The part's task is cut once answered.
2. **lab-browser** (skein, `skein/docs/design/browser.md, 5.1`), slice 1.
   - `tasks/remove-lab-browser.md`, high, ~100 minutes, needs nothing: deletes
     `skein/testing/skein-lab-browser` (workspace member, `Cargo.lock`, `.config/nextest.toml`'s
     `browser-skein` filter, `tools/tiers/packages.toml`, `tools/tiers/design.md`), the browser
     kit's `Look`/`Looked` with their tests, `skein_ui_web::mapping::person`,
     `Pages::verbatim`, the four `clippy.toml` lines naming `Look`; deletes `lab-browser.md` and
     its status, removes browser.md 5.1 (number left unused) and every look line, ui-web.md
     14.2's mapping back, testing.md's and README.md's lines, and the status entries.
3. **forgejo-connect** (temper, `temper/docs/design/protocol/forge.md, 2.6`), slice 4: **a
   question** (`forgejo-connect`). The design leaves the wire's shapes, the client's offer, where
   the login is kept, a reconnect by another user and the paste's layout open. Recommended: the
   designer settles them, then a later cut (about seven tasks).
4. **versions** (temper, `temper/docs/design/protocol/README.md, 9`), slice 2.
   - `tasks/versions-in-place.md`, medium, ~15 minutes, needs nothing, docs only: the design
     settles both versions at 11 until the first release (`client/README.md` 4.6, "It stays at
     11"; `protocol/README.md` 9; `protocol/store.md` 4), the code holds 11 and the schema has
     changed in place at v11 since 2026-10-31; the "Departs" entries record history. They go,
     with `domain/forge.md`'s stale "(`STORE_VERSION` 3)". No stored format moves, so no
     question.
5. **withdrawn-forms** (skein, smith, jig, temper; `skein/docs/design/llm.md, 4.6`), slice 6:
   **a question** (`withdrawn-forms`). The survey found smith's product still running on the
   withdrawn forms (its codex preset and default account, the ChatGPT account-id sign-in,
   Anthropic's `claude-code` identity), unrecorded by any status file; the removal makes smith's
   channel v6 (a protocol), stops dialect-1 charters and transcripts from resuming, deletes
   recordings that cannot be captured again, and needs a jig dialect rule the design leaves
   open. No order keeps each branch in one tree (exhaustive matches, `wildcard_enum_match_arm`
   denied). Recommended: remove all as the designs say, in a later cut of five ordered tasks,
   smith's product after sign-in-wait lands.
6. **jig-compat** (jig, temper's readers; `jig/docs/design/protocol/client.md, 10`), slice 5.
   - `tasks/remove-program-and-seen.md`, high, ~120 minutes, needs nothing: removes
     `TaskWaitRefused::Program` from jig's domain core, client contract, `client-v19` (in place,
     goldens regenerated), translations, scripted engine and words, and temper's arm, words and
     renderer (forced: the build breaks without them); a stored top-level `Seen` is dropped,
     told `Dropped` and erased at start; the people domain skips a kept `Seen` answer and
     erases it at the sweep. Seven status entries, `client.md` 11's sentence and temper's
     `client/README.md` 3.5 program causes updated.

## Critical path

`remove-program-and-seen` (~120 minutes, up to ~240 with the buffer) beside
`remove-lab-browser` (~100), then fab's end check. `versions-in-place` has landed. The three
questions hold only their own parts.

## Overlaps

- `versions-in-place` edits `temper/docs/design/domain/forge.md` at line ~422 only; the ladder's
  `finding-67` and `finding-75` edit it near lines 1171-1216: the same file, no shared hunk.
- `remove-lab-browser` touches no file an open ladder or sign-in-wait branch changes (checked
  against `origin/v0.1.0-ladder/*` on 2026-11-02).
- `remove-program-and-seen` regenerates `jig-client-protocol-wire/src/generated/v19.rs`, which
  the ladder's `finding-4` also regenerates (from `client-v19-part1.schema`), and edits
  `client/domain/contract.status.md`, which `finding-4` also edits: unavoidable, since the
  removal is that schema's; the generated file is regenerated, never merged by hand.
- `withdrawn-forms`, once answered, meets sign-in-wait in
  `smith/crates/smith-local-shell/src/credentials/product.rs`: its smith product task waits
  for that draft to land.

## Calls made

- **Item 4 asks nothing:** the request asks only if the design does not settle the values; it
  does (11, in place), and closing the gap moves no stored format.
- **Item 6 asks nothing:** the user dropped the arm (2026-11-02); the wire changes in place
  before the baseline (jig `protocol/README.md`, 7); a stored top-level `Seen` follows the
  design's own rules: a record the client does not read is dropped and told (`client.md`, 11),
  and its key is erased so that `Seen` alone never warns (`browser.md`, 10); the people domain
  skips and sweeps a kept `Seen` answer as it does an expired one, with no format change.
  temper's lines are deleted in the same branch because the build breaks without them.
- **Item 5 asks:** a protocol version, stored data and a product's routes are the user's.
- **Item 1 and item 3 ask:** item 1 as the request expected; item 3 because its survey found the
  wire's shape for a new connected Forgejo, the client's offer and two rules undecided, and a
  wire is the user's (AGENTS.md, 7).
- **Item 2's reach:** besides the crate, the look's reads and the mapping back serve only the
  lab (their status entries say they await removal with it), and `Pages::verbatim` in
  `skein-fake-person-browser` is called only by the lab: all go in one skein task.

## 11-03: the three asked items leave this plan

Answered unattended by agents/loop (the user asleep; the director's word to take the recommended
options and prefer what can be undone):

- `readiness-eintr`: (a), the readiness backend completes an interrupted Socket, Accept, Connect
  and Open `Other(EINTR)` as the ring does. Its own plan, `readiness-eintr`.
- `forgejo-connect`: (a), temper's designer settles the five open points first (the suggested
  settlement as a start), then a later cut builds it. Its own plan, `forgejo-connect`.
- `withdrawn-forms`: deferred, option (c), not the recommended (a). (a) deletes skein-llm's Codex
  recordings, which cannot be captured again, moves smith's channel to v6 and ends stored
  dialect-1 charters and transcripts: hard to undo, so the user's when awake. Nothing waits on it.

A direct plan's part left undesigned by a question never gets a design call once answered, so
the answered items start afresh as their own plans rather than here.
