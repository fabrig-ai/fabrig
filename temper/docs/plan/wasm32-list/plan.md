# wasm32-list: plan

Draft: fabrig's wasm32 gate list (`inputs/request.md`). fab's gate has a `clippy-wasm32` step
that reads `.config/gate/wasm32.txt`; fabrig has no such file, so the step checks nothing.

## Parts and tasks

- **gate-list** (slice 1): `tasks/gate-list.md`, no needs, released in the first push. Adds the
  list with the request's six packages, runs the gate's command over all six, and closes
  `skein/docs/design/playground.status.md`'s Not yet 8 and its clause in skein's `STATUS.md`.
- **wasm32-status** (slice 2): `tasks/wasm32-status.md`, needs gate-list, released in the second
  push. Status text only: records as Not yet, in skein's, jig's, temper's and smith's status
  files and `STATUS.md` summaries, the crates the design names for the gate's wasm32 clippy that
  the list does not hold. The lead's addition beyond the request (below); dropping its branch
  leaves gate-list whole.

## Decisions

- **The request's work is one task, gate-list.** The list and the status it closes travel in one branch, so
  the status never says the gate checks `skein-playground` before the list is on the trunk.
- **The check runs all six,** not only `skein-playground`: it is one `heavy` command, and it is
  what the gate's step runs on a selection that holds them all. A red in one of the other five
  (the trunk moved since `943c9871ec`) keeps its line and is reported, since a list that drops
  a red package would hide the regression the gate is there to catch.
- **No description of the list:** nothing in fabrig describes `.config/gate/`'s lists
  (`always.txt`, `read-by-tests.txt`); `tools/tiers/design.md`, 9 describes
  `.config/fixture-readers.toml` only. The task re-checks and adds one sentence only if it finds
  one.
- **`temper-client-shell-web` stays out** (fails `clippy::too_many_lines` at `turn`), as the
  request says; temper's to fix.
- **The status files say what the gate checks (wasm32-status).** With the list, the gate's
  wasm32 clippy is a concrete set of six packages, while the design names more for it in every
  tree (skein's `testing.md` 6 and 10, `lib.md` 2, `shell.md` 14, `world.md` 8; jig's
  `protocol/browser.md` 15 and 18, `protocol/README.md` 11; temper's `client/browser.md` 4.3;
  smith's `shell.md` 2.5) and their status files record no gap. Status records the gaps as they
  stand (`DESIGN.md`, 2), so a docs task records each as Not yet; the entries' labels were
  checked with `tools/citations/citations.py` on a throwaway commit (0 broken). It needs
  gate-list because both edit skein's `STATUS.md` summary. Already recorded, and left alone:
  temper's `playground.status.md` 9 (playground-temper's file).
- **Extending the list is a question** (`wasm32-more` in `plan.toml`): adding packages lengthens
  the gate's step and only `temper-client-shell-web` is measured (red). Default taken: six, the
  gaps recorded; recommended: a follow-up draft measures each named crate and adds the green.

## Overlaps with the drafts beside this one

None found. No listed file of reference-m10, configuring-p2-build or playground-temper is
touched; `temper/docs/design/playground.status.md` (playground-temper's) is not edited here.
wasm32-status edits `temper/docs/design/STATUS.md`, `jig/docs/design/STATUS.md` and
`smith/docs/design/STATUS.md` by one clause each; no draft beside this one lists them, but the
queued drafts (v0.1.0-ladder, sign-in-wait, reference-walks) list no files yet, so a textual
meeting there is possible; the spec pre-approves placing an entry by section order when its
anchor sentence moved.

## Critical path

gate-list (about 20 implementer-minutes, most of them the wasm32 build of six packages), then
wasm32-status (about 20, text only): about 40 minutes.
