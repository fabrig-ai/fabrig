+++
id = "playground-list"
part = "playground-list"
kind = "feature"
needs = []
files = [
  ".config/gate/wasm32.txt",
  "temper/docs/design/playground.status.md",
]
effort = "medium"
minutes = 20
+++

# temper-playground in fab's wasm32 gate list, when its wasm32 clippy is green

## Goal

fab's gate step `clippy-wasm32` reads `.config/gate/wasm32.txt` and runs `cargo clippy --target
wasm32-unknown-unknown -p <each listed package the selection holds> -- -D warnings`. The design
puts `temper-playground`'s wasm32 build in the gate with clippy's lints as errors
(`temper/docs/design/playground.md`, 9); the list does not hold it yet. Measure it at your base;
green, add its line and narrow the status entry that records it missing; red, change nothing and
report the first error for temper to fix.

Acceptance, when green (step 1 passes):

- `.config/gate/wasm32.txt` holds exactly these eight lines, in this order, ending with one
  newline:

  ```text
  # Packages the gate clippies for wasm32-unknown-unknown (fab's clippy-wasm32 step).
  skein-ui
  skein-ui-web
  skein-ui-example-view
  skein-ui-example-shell-web
  jig-client-shell-web
  skein-playground
  temper-playground
  ```

  `temper-client-shell-web` is not in it: it stays out until plan `playground-browser` fixes it.
  Leave that crate's code alone.
- `heavy cargo clippy --target wasm32-unknown-unknown -p temper-playground -- -D warnings`
  passes, and so does the gate's command over the whole list (step 3), but for the case under
  `Pre-approved`.
- `temper/docs/design/playground.status.md`'s date line is today's (`date -u +%F`), and its
  lines 11-12, now

  ```text
    documents' bounds, measured. 9: the native walk, the wasm32 builds in the
    gate, a log replayed and the browser tier's walk. 10: the bar.
  ```

  read, as three lines:

  ```text
    documents' bounds, measured. 9: the native walk, the web shell's wasm32
    build with `playground` in the gate, a log replayed and the browser
    tier's walk. 10: the bar.
  ```

  Section 9 names two wasm32 builds for the gate: `temper-playground`, and the web shell with
  its feature `playground` (entry 3 says that feature is not built); only the first closes.
  Nothing else in the file changes.
- `python3 -I -B tools/citations/citations.py --base origin/main` prints no finding this branch
  adds.

Acceptance, when red (step 1 fails): no file changes; your branch is pushed at its base; the
report's first line is `Done: temper-playground fails wasm32 clippy; the list is unchanged`, and
its `## Left` gives the first error: the lint or error code, the message, the file and line, and
the item. No fix: the request gives it to temper.

## Inputs

- `temper/docs/plan/wasm32-temper/notes/common.md`: the standing rules, and this plan's
  departure from the implementer's text for the status file.
- `temper/docs/plan/wasm32-temper/inputs/request.md`: the request and its check.
- `temper/docs/design/playground.md`, 3 ("A library on every target and a module on wasm32")
  and 9 ("The wasm32 builds, in the gate"). Read only.
- `temper/docs/design/playground.status.md`: the entry this task narrows.

## Owns and may change

- `.config/gate/wasm32.txt`: one line, `temper-playground`, appended after `skein-playground`;
  the other seven lines stay as they are. Its siblings `always.txt` and `read-by-tests.txt` stay
  as they are.
- `temper/docs/design/playground.status.md`: the date line and the edit above.
- Nothing else. Temper's `docs/design/STATUS.md` keeps its clause on `temper-playground`
  (`playground.md`, `domain/testing.md`): it stands for the crate's other open entries.
  `client/browser.status.md` 4.3 names temper's client crates, not this one; skein's and jig's
  status entries on the list name their own trees' crates. None changes.

Pre-approved:

- If step 1 fails: the red acceptance above. Commit nothing.
- If step 1 passes and step 3 fails in one of the other six packages only (the trunk moved
  since they were measured), keep the list and the status edit as above and report `Done:`
  with each failing lint, its package, file and line under `## Left`: that code is not this
  task's.
- If step 3 fails in `temper-playground` (the whole list's features unified differ from it
  alone): remove the line again in a new commit, leave the status file unchanged, and report
  `Done: temper-playground fails wasm32 clippy beside the list; the list is unchanged`, with
  the first error as in the red case under `## Left`.
- If the wasm32 target is missing on the builder, follow the note: `Failed:`, nothing
  installed.

## Steps

1. From the repository root, at your base, run through `heavy`, once:
   `heavy cargo clippy --target wasm32-unknown-unknown -p temper-playground -- -D warnings`.
   On failure, read its log around the first `error` (`grep -n -m5 -A12 '^error'`), never
   whole, then follow the red acceptance and stop after the report.
2. Append `temper-playground` to `.config/gate/wasm32.txt`; commit (temper-playground's wasm32
   clippy is green, so fab's clippy-wasm32 step checks it).
3. Run the gate's command over the whole list, once:
   `heavy cargo clippy --target wasm32-unknown-unknown -p skein-ui -p skein-ui-web -p
   skein-ui-example-view -p skein-ui-example-shell-web -p jig-client-shell-web -p
   skein-playground -p temper-playground -- -D warnings`. On failure, follow `Pre-approved`.
4. Edit `temper/docs/design/playground.status.md` as above; commit (temper-playground's wasm32
   build is in the gate; the web shell's with `playground` is not).
5. `python3 -I -B tools/citations/citations.py --base origin/main`; `git log --stat` of your
   commits shows the two files alone.
6. Push your branch.

## Report

As the implementer's text says, and also: each clippy command's outcome and wall time from
heavy's output, the host it ran on, and the citations check's output.
