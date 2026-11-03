# wasm32-temper: plan

Draft: `temper-playground` joins fab's wasm32 gate list (`inputs/request.md`). fab's gate step
`clippy-wasm32` reads `.config/gate/wasm32.txt` (plan wasm32-list); `temper-playground` landed
with plan playground-temper. `temper/docs/design/playground.md`, 9 puts its wasm32 build in the
gate with clippy's lints as errors.

## Parts and tasks

- **playground-list** (slice 1): `tasks/playground-list.md`, no needs, released in the first and
  only push. Runs `heavy cargo clippy --target wasm32-unknown-unknown -p temper-playground -- -D
  warnings` at its base. Green: appends `temper-playground` after `skein-playground`, runs the
  gate's command over the whole list, and narrows `temper/docs/design/playground.status.md`'s
  Not yet 9 to the web shell's wasm32 build with `playground`. Red: changes nothing and reports
  the first error, file and line for temper to fix.

## Decisions

- **One task, as the request says.** The list and the status it narrows travel in one branch, so
  the status never says the gate checks `temper-playground` before the list does.
- **The status item narrows, it does not close.** Section 9 names two wasm32 builds for the gate:
  `temper-playground`, and the web shell with its feature `playground`, which is not built
  (Not yet 3). Only the first closes; entry 9 keeps the web shell's. Temper's `STATUS.md` keeps
  its `temper-playground` clause, which stands for the crate's other open entries (4, 6 to 10).
- **The whole list is clippied once more after the line goes in:** the gate's step clippies the
  listed packages the selection holds together, and features unified across seven packages can
  differ from `temper-playground` alone. A red there in another package keeps the line and is
  reported (wasm32-list's rule); a red in `temper-playground` takes the line out again.
- **A red changes nothing**, as the request says: no fix in this task, the first error reported
  for temper. `temper-client-shell-web` stays out (plan playground-browser fixes it).
- No design section is written: `playground.md`, 9 already says it.

## Overlaps with the drafts beside this one

- **playground-browser** (cut in flight): its request names "`temper-playground`'s wasm32 build
  in the gate" (9's browser part) and `temper-client-shell-web` joining `.config/gate/wasm32.txt`
  "along with `temper-playground`". Both touch this task's two files: the list's last line and
  `playground.status.md`'s entry 9 (lines 11-12). Unavoidable, since the request asks for both
  edits. This task is small and released first; playground-browser's tasks that edit those
  lines meet it where fab merges, and find `temper-playground` already listed.
- No file of configuring-p2-build's merged tasks is touched; the queued drafts (v0.1.0-ladder,
  sign-in-wait, reference-walks) and public-reading-build list no files yet.

## Critical path

playground-list: about 20 implementer-minutes, most of them two wasm32 clippy builds.
