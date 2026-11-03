# wasm32-list: plan

Draft: fabrig's wasm32 gate list (`inputs/request.md`). fab's gate has a `clippy-wasm32` step
that reads `.config/gate/wasm32.txt`; fabrig has no such file, so the step checks nothing.

## Parts and tasks

- **gate-list** (slice 1): `tasks/gate-list.md`, no needs, released in the first push. Adds the
  list with the request's six packages, runs the gate's command over all six, and closes
  `skein/docs/design/playground.status.md`'s Not yet 8 and its clause in skein's `STATUS.md`.

## Decisions

- **One task, as the request asks.** The list and the status it closes travel in one branch, so
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

## Overlaps with the drafts beside this one

None found. No listed file of reference-m10, configuring-p2-build or playground-temper is
touched; `temper/docs/design/playground.status.md` (playground-temper's) is not edited here.

## Critical path

gate-list alone: about 20 implementer-minutes, most of them the wasm32 build of six packages.
