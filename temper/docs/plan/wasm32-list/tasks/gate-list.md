+++
id = "gate-list"
part = "gate-list"
kind = "feature"
needs = []
files = [
  ".config/gate/wasm32.txt",
  "skein/docs/design/playground.status.md",
  "skein/docs/design/STATUS.md",
]
effort = "medium"
minutes = 20
+++

# fabrig's wasm32 gate list: six packages, and skein-playground's status closed

## Goal

fab's gate has a `clippy-wasm32` step: it reads fabrig's `.config/gate/wasm32.txt` and runs
`cargo clippy --target wasm32-unknown-unknown -p <each listed package the selection holds> --
-D warnings`. fabrig has no such file yet, so the step checks nothing. Add it with the six
packages the request lists (measured green at main `943c9871ec`), which puts
`skein-playground` in the gate's wasm32 clippy (`skein/docs/design/playground.md`, 8), and
close the status entry that records it missing.

Acceptance:

- `.config/gate/wasm32.txt` exists and holds exactly these seven lines, in this order, ending
  with one newline:

  ```text
  # Packages the gate clippies for wasm32-unknown-unknown (fab's clippy-wasm32 step).
  skein-ui
  skein-ui-web
  skein-ui-example-view
  skein-ui-example-shell-web
  jig-client-shell-web
  skein-playground
  ```

  `temper-client-shell-web` is not in it: it fails `clippy::too_many_lines` at
  `temper/crates/temper-client-shell-web/src/lib.rs:131` (`turn`, 102/100), temper's to fix
  before it joins the list. Leave it out, and leave that code alone.
- `heavy cargo clippy --target wasm32-unknown-unknown -p skein-ui -p skein-ui-web -p
  skein-ui-example-view -p skein-ui-example-shell-web -p jig-client-shell-web -p
  skein-playground -- -D warnings` passes: what the gate's step runs on a selection that holds
  all six.
- `skein/docs/design/playground.status.md` reads, whole:

  ```text
  # playground: status

  <today, date -u +%F>. Where skein's code lags or departs from [playground.md](playground.md).

  No gap.
  ```

- `skein/docs/design/STATUS.md`'s summary paragraph drops the sentence `` `skein-playground`
  still lacks the gate's wasm32 clippy (fab's gate list; `playground.md`, 8); `ui-web.md` has no
  gap. `` (lines 12-13), so that line 12 reads `` (`fake-forge.md`). json, tls and http's
  collector `` and continues as it does; and the list of documents with no gap (lines 18-22)
  becomes `` `README.md`, `channel.md`, `codec.md`, `kv.md`, `world.md`, `fake-checkout.md`,
  `fake-llm.md`, `forge-client.md`, `forge-connection.md`, `forge-setup.md`,
  `llm-connection.md`, `oauth.md`, `oauth-issuer.md`, `playground.md`, `ui-text.md`,
  `ui-web.md` and `browser.md` have no gap. ``, wrapped at about 72 columns as the paragraph
  is. Its date line is today's. Nothing else in the file changes; its index already lists
  `playground.md`.
- `python3 -I -B tools/citations/citations.py --base origin/main` prints no finding this branch
  adds.

## Inputs

- `temper/docs/plan/wasm32-list/notes/common.md`: the standing rules, and this plan's
  departure from the implementer's text for status files.
- `temper/docs/plan/wasm32-list/inputs/request.md`: the request, its list and its check.
- `skein/docs/design/testing.md`, 6 (the gate's steps, clippy for wasm32 among them) and 10 (the
  portable target, checked in the gate); `skein/docs/design/ui-web.md`, 14 (the UI kit's and
  its example's wasm32 builds in the gate); `skein/docs/design/playground.md`, 8 (clippy over
  `skein-playground` for wasm32 in the gate). Read only.

## Owns and may change

- `.config/gate/wasm32.txt`: created, as above. It sits beside `always.txt` and
  `read-by-tests.txt`, fab's other gate lists, which stay as they are.
- `skein/docs/design/playground.status.md` and `skein/docs/design/STATUS.md`: the edits above.
- Nothing in fabrig describes `.config/gate/`'s lists (`rg -n 'config/gate|always\.txt'
  --glob '!**/docs/plan/**'` finds none; `tools/tiers/design.md`, 9 describes
  `.config/fixture-readers.toml`, which is not one of them), so no document gains a description
  of the new list. If that `rg` does find a description, add one sentence there naming
  `wasm32.txt` and what reads it, in that text's words, and say so in the report.

Pre-approved:

- If the clippy command fails in `skein-playground`, stop before editing the status files:
  commit and push the list, and report `Question:` with the lint, its file and line, and two
  options: fix it in this task (when it is one item, local to `skein-playground`, whose fix
  changes no public item), or keep `skein-playground` out of the list and its Not yet open.
  Recommend the first where it holds.
- If it fails in one of the other five packages only (the trunk moved since `943c9871ec`), keep
  the list as above, close the status entry as above, and report `Done:` with each failing lint,
  its package, file and line under `## Left`: the gate's list is the request's, and that code
  is not this task's.
- If the wasm32 target is missing on the builder, follow the note: `Failed:`, nothing
  installed.

## Steps

1. Create `.config/gate/wasm32.txt` as above; commit (the list fab's clippy-wasm32 step reads,
   with the six packages measured green; `temper-client-shell-web` out until temper fixes
   `turn`'s length).
2. Run the clippy command above through `heavy`, once. Read its log's tail on failure, then
   follow the pre-approved options.
3. Edit the two status files as above; commit (skein-playground is in the gate's wasm32 clippy
   now; the status records no gap).
4. `python3 -I -B tools/citations/citations.py --base origin/main`; `git log --stat` of your
   commits shows the three files alone.
5. Push your branch.

## Report

As the implementer's text says, and also: the clippy command's outcome and wall time from
heavy's output, the host it ran on, the citations check's output, and the `rg` result of the
description check.
