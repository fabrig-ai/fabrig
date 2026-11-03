# wasm32-temper-3: plan

Draft: `temper-playground` joins fab's wasm32 gate list, third try (`inputs/request.md`).
fab's gate step `clippy-wasm32` reads `.config/gate/wasm32.txt` (plan wasm32-list). The second
try, plan wasm32-temper-2, measured `temper-playground` red on rustls's `UnixTime::now` (E0599);
plan wasm32-rustls has since stopped the workspace's rustls turning on `std`.
`temper/docs/design/playground.md`, 9 puts the crate's wasm32 build in the gate with clippy's
lints as errors.

## Parts and tasks

One part, one task, released in the first and only push.

- **playground-list** (slice 1): `tasks/playground-list.md`, no needs. Runs `heavy cargo clippy
  --target wasm32-unknown-unknown -p temper-playground -- -D warnings` at its base. Green: adds
  `temper-playground` right after `skein-playground`, clippies the whole list once, and narrows
  `temper/docs/design/playground.status.md`'s Not yet 9 to the web shell's wasm32 build with
  `playground`. Red: changes nothing, and reports the first error, then every warning of a
  second run without `-D warnings`.

## Decisions

- **The request's one task, and no more.** The second try added two status tasks, one for skein
  and one for smith, to close their entries on the portable crates and `smith-agent-shell`
  missing from the gate's wasm32 clippy, which `temper-playground`'s wasm32 tree brings in
  (`cargo tree --target wasm32-unknown-unknown -e normal -p temper-playground` lists 99
  workspace crates, among them `skein-fake-machine`, `skein-kv`, `skein-tls` and
  `smith-agent-shell`). Plan wasm32-more's task gate-more (not merged) now owns those exact
  entries (`skein/docs/design/testing.status.md` 6, 10, `world.status.md` 8, `STATUS.md`;
  `smith/docs/design/shell.status.md` 2.5, `STATUS.md`): it clippies those crates by name,
  lists the green ones and rewrites each entry from its measurements. Two plans editing those
  lines would not merge, so this plan leaves them to gate-more. Until gate-more lands, those
  entries name as missing crates the gate lints through `temper-playground`; gate-more closes
  them when its crates are green. Should gate-more record one of them red alone while it is
  green inside `temper-playground`'s build, that is gate-more's report to make.
- **The status item narrows, it does not close.** Section 9 names two wasm32 builds for the gate:
  `temper-playground`, and the web shell with its feature `playground`. Only the first closes;
  entry 9 keeps the web shell's. Temper's `STATUS.md` keeps its clauses, which stand for the
  playground's other entries and for the client crates still outside the list.
- **`temper/docs/design/client/browser.status.md` 4.3 is left as it is**: plans
  playground-browser and wasm32-more both edit it.
- **Whole list once more after the line goes in:** features unified across seven packages can
  differ from `temper-playground` alone. A red in a crate outside `temper-playground`'s wasm32
  tree keeps the line and is reported (wasm32-list's rule); a red inside it takes the line out.
- **A red changes nothing**, as the request says: no fix in this plan.
  `temper-client-shell-web` stays out (plan playground-browser adds it; `temper-playground`'s
  wasm32 tree does not hold it).
- No design section is written: `playground.md`, 3 and 9 already say it.

## Overlaps with the drafts beside this one

- **wasm32-more** (integration): its task gate-more (not merged) appends its green packages to
  `.config/gate/wasm32.txt` after `skein-playground`, where this plan puts `temper-playground`.
  The two appends meet at the file's end and will not merge without a hand where fab merges
  them; whichever lands second keeps both. Unavoidable: the request places the line. The spec
  pre-approves a base that already holds gate-more's lines. No other file of gate-more's is
  touched (above).
- **playground-browser** (integration): its trunk puts `temper-client-shell-web` before
  `skein-playground` and rewrites `playground.status.md`'s entries 3, 6, 7 and 8; checked with
  `git merge-file` against this plan's two edits: both merge cleanly. Its task status-nine (not
  merged, after four of its tasks) edits entry 9 from what its trunk holds and leaves words
  another plan removed; if it runs before main's edit reaches its trunk, the two meet on 9's
  lines. Unavoidable: the request asks for the status edit, and this task runs first.
- **public-reading-build**, **configuring-p3** and **wasm32-rustls**: no listed file touched; the
  queued drafts (v0.1.0-ladder, sign-in-wait, reference-walks) and fabrig-scale list no files
  yet.

## Critical path

playground-list alone: about 30 implementer-minutes, most of them two wasm32 clippy builds
(`temper-playground` alone, then the whole list), or one denied and one survey when red.
