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
5. **withdrawn-forms** (skein, smith, jig): being cut (survey running).
6. **jig-wire-compat** (jig): being cut (survey running).

## Critical path

`remove-lab-browser` (~100 minutes, up to ~200 with the buffer), then fab's end check. The
questions hold only their own parts.

## Overlaps

- `versions-in-place` edits `temper/docs/design/domain/forge.md` at line ~422 only; the ladder's
  `finding-67` and `finding-75` edit it near lines 1171-1216: the same file, no shared hunk.
- `remove-lab-browser` touches no file an open ladder or sign-in-wait branch changes (checked
  against `origin/v0.1.0-ladder/*` on 2026-11-02).

## Calls made

- **Item 4 asks nothing:** the request asks only if the design does not settle the values; it
  does (11, in place), and closing the gap moves no stored format.
- **Item 1 and item 3 ask:** item 1 as the request expected; item 3 because its survey found the
  wire's shape for a new connected Forgejo, the client's offer and two rules undecided, and a
  wire is the user's (AGENTS.md, 7).
- **Item 2's reach:** besides the crate, the look's reads and the mapping back serve only the
  lab (their status entries say they await removal with it), and `Pages::verbatim` in
  `skein-fake-person-browser` is called only by the lab: all go in one skein task.
