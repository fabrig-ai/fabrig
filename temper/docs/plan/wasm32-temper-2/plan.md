# wasm32-temper-2: plan

Draft: `temper-playground` joins fab's wasm32 gate list, second try (`inputs/request.md`).
fab's gate step `clippy-wasm32` reads `.config/gate/wasm32.txt` (plan wasm32-list). The first
try, plan wasm32-temper, measured `temper-playground` red in jig (`jig-domain-people`
`boundary.rs:225`); plan wasm32-jig has since scoped jig's size expectations to the host.
`temper/docs/design/playground.md`, 9 puts the crate's wasm32 build in the gate with clippy's
lints as errors.

## Parts and tasks

All three are released in the first and only push.

- **playground-list** (slice 1): `tasks/playground-list.md`, no needs. Runs `heavy cargo clippy
  --target wasm32-unknown-unknown -p temper-playground -- -D warnings` at its base. Green:
  appends `temper-playground` after `skein-playground`, clippies the whole list once, and
  narrows `temper/docs/design/playground.status.md`'s Not yet 9 to the web shell's wasm32 build
  with `playground`. Red: changes nothing, and reports the first error, then every warning of a
  second run without `-D warnings`.
- **skein-status** (slice 1): `tasks/skein-status.md`, needs playground-list. If the list holds
  the line, closes the entries that record skein's remaining portable crates out of the gate's
  wasm32 clippy: `testing.status.md` 6, 10's clause, `world.status.md` 8 (the file says `No
  gap.`) and `STATUS.md`'s sentence. Otherwise changes nothing.
- **smith-status** (slice 1): `tasks/smith-status.md`, needs playground-list. If the list holds
  the line, closes `shell.status.md` 2.5 and `STATUS.md`'s clause on `smith-agent-shell`.
  Otherwise changes nothing.

## Decisions

- **The request's one task is playground-list**, as the first try's was, with one addition: a
  red reports every warning, from a second run that does not deny them, so a third try is cut
  from the whole list rather than from one site.
- **The status item narrows, it does not close.** Section 9 names two wasm32 builds for the gate:
  `temper-playground`, and the web shell with its feature `playground`, which is not built
  (Not yet 3). Only the first closes; entry 9 keeps the web shell's. Temper's `STATUS.md`
  keeps its clauses, which stand for the playground's other entries and for the client crates
  still outside the list.
- **The line brings about 90 workspace crates into the gate's wasm32 clippy.** clippy lints
  every workspace crate a build compiles, and the trees' status files count the list's
  coverage that way (`skein/docs/design/testing.status.md`, 6, 10). `cargo tree --target
  wasm32-unknown-unknown -e normal -p temper-playground` lists 99 workspace crates where the
  six listed packages list 21. Those include every portable crate skein's status records
  missing (the fake machine, the fakes a world hosts, the key-value store, JSON, the channel,
  TLS) and `smith-agent-shell` with what it depends on in smith. Those entries would be false
  once the line lands, so the survey gives them to two tasks, one per tree (a branch stays in
  its tree). Each checks the list first and the crates by `cargo tree` (builds nothing); a red
  playground-list leaves them at their base. jig's `protocol/browser.status.md` 15
  (`jig-client-test-shell-web`) stays: no listed crate links it.
- **`temper/docs/design/client/browser.status.md` 4.3 is left as it is**, though
  `temper-client-protocol-wire` is in `temper-playground`'s wasm32 tree: plan
  playground-browser rewrites that entry on its trunk (task shell-turn-split, with
  `temper-client-shell-web` joining the list, which links all three crates 4.3 still names).
  Narrowing it here would put a second edit on the same lines; until playground-browser
  lands, 4.3 names one crate more than is missing.
- **Whole list once more after the line goes in:** features unified across seven packages can
  differ from `temper-playground` alone. A red in a crate outside `temper-playground`'s wasm32
  tree keeps the line and is reported (wasm32-list's rule); a red inside it takes the line out.
- **A red changes nothing**, as the request says: no fix in this plan.
  `temper-client-shell-web` stays out (plan playground-browser adds it).
- No design section is written: `playground.md`, 9, `skein/docs/design/testing.md`, 6 and 10,
  `world.md`, 8 and `smith/docs/design/shell.md`, 2.5 already say it.

## Overlaps with the drafts beside this one

- **playground-browser** (integration): its trunk adds `temper-client-shell-web` to
  `.config/gate/wasm32.txt` before `skein-playground`; this plan appends after it. Checked with
  `git merge-file`: the two merge cleanly. Its task status-nine (not merged) and its trunk's
  `playground.status.md` edit entry 9, whose lines playground-list narrows; its trunk's copy
  is older than main's (playground-temper's retirement rewrote the file), so the two meet where
  fab merges in any case. Unavoidable: the request asks for the status edit.
- **public-reading-build** (integration): task public-status (not merged) edits
  `skein/docs/design/STATUS.md`, whose summary paragraph skein-status edits. Unavoidable: the
  sentence skein-status removes is in that paragraph. Its trunk's copy also predates main's
  wasm32 sentence.
- No file of configuring-p2-build's merged tasks is touched; the queued drafts (v0.1.0-ladder,
  sign-in-wait, reference-walks) list no files yet.

## Critical path

playground-list (about 30 implementer-minutes, most of them two wasm32 clippy builds, or one
denied and one survey when red), then skein-status and smith-status at once (10-15 each, no
build). About 45 minutes end to end.
