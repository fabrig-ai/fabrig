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
minutes = 30
+++

# temper-playground in fab's wasm32 gate list, when its wasm32 clippy is green

## Goal

The design puts `temper-playground`'s wasm32 build in the gate with clippy's lints as errors
(`temper/docs/design/playground.md`, 9); fab's gate list `.config/gate/wasm32.txt` does not hold
it yet. The first try (plan wasm32-temper) measured it red in jig; plan wasm32-jig has since
scoped jig's size expectations to the host. Measure it at your base. Green: add its line and
narrow the status entry that records it missing. Red: change nothing, and report every error
and warning the build shows, the first one first, for temper to fix.

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

  `temper-client-shell-web` is not added: plan playground-browser adds it. Leave that crate's
  code alone.
- `heavy cargo clippy --target wasm32-unknown-unknown -p temper-playground -- -D warnings`
  passes, and so does the gate's command over the whole list (step 3), but for the cases under
  `Pre-approved`.
- `temper/docs/design/playground.status.md`'s date line is today's (`date -u +%F`), and its
  lines 28-30, now

  ```text
      in the one-process world, ignored (4). The wasm32 builds in the gate,
      so no gate checks `temper-playground`'s build for wasm32, which
      proves 3's product crates. A log replayed, and the browser tier's
  ```

  read, as two lines:

  ```text
      in the one-process world, ignored (4). The web shell's wasm32 build
      with `playground` in the gate. A log replayed, and the browser tier's
  ```

  Line 31 (`    walk.`) stays. Section 9 names two wasm32 builds for the gate:
  `temper-playground`, and the web shell with its feature `playground` (entry 3 says that
  feature is not built); only the first closes. Nothing else in the file changes.
- `python3 -I -B tools/citations/citations.py --base origin/main` prints no finding this branch
  adds.

Acceptance, when red (step 1 fails): no file changes; your branch is pushed at its base; the
report's first line is `Done: temper-playground fails wasm32 clippy; the list is unchanged`,
and its `## Left` gives first the first error of step 1 (the lint or error code, the message,
the crate, the file and line, and the item), then every error and warning of step 1b, one line
each, grouped by crate. No fix: the request gives them to temper.

## Inputs

- `temper/docs/plan/wasm32-temper-2/notes/common.md`: the standing rules, the gate's step, and
  this plan's departure from the implementer's text for status files.
- `temper/docs/plan/wasm32-temper-2/inputs/request.md`: the request and its check.
- `temper/docs/design/playground.md`, 3 ("The crates") and 9 ("Testing", its item "The wasm32
  builds"). Read only.
- `temper/docs/design/playground.status.md`: the entry this task narrows.

## Owns and may change

- `.config/gate/wasm32.txt`: one line, `temper-playground`, appended after `skein-playground`;
  the other lines stay as they are. Its siblings in `.config/gate/` stay as they are.
- `temper/docs/design/playground.status.md`: the date line and the edit above.
- Nothing else. Temper's `docs/design/STATUS.md` keeps its clauses: "the playground in the
  browser" stands for the crate's other open entries, and "the client's crates in the gate's
  wasm32 clippy" for `temper-client-domain`, `temper-client-view` and `temper-client-shell-web`.
  `temper/docs/design/client/browser.status.md` stays as it is: plan playground-browser edits
  its 4.3. skein's and smith's status files are this plan's tasks skein-status and
  smith-status, which start after this one.

Pre-approved:

- If the list at your base holds lines beyond the seven above (another plan landed one), keep
  them where they are, put `temper-playground` right after `skein-playground`, and name every
  listed package in step 3's command.
- If `playground.status.md`'s entry 9 at your base differs from the lines quoted above, change
  only the words that say `temper-playground`'s wasm32 build is not in the gate, so the entry
  keeps the web shell's build with `playground`; if no words say it, leave the file and say so.
- If step 1 fails: the red acceptance above, with step 1b. Commit nothing.
- If step 1 passes and step 3 fails in a crate outside `temper-playground`'s wasm32 tree
  (`cargo tree --target wasm32-unknown-unknown -e normal -p temper-playground --prefix none`
  does not list it): keep the list and the status edit, and report `Done:` with each failing
  lint, its crate, file and line under `## Left`: that code is not this task's.
- If step 3 fails in a crate of `temper-playground`'s wasm32 tree (features unified across the
  list differ from it alone): remove the line again in a new commit, leave the status file
  unchanged, and report `Done: temper-playground fails wasm32 clippy beside the list; the list
  is unchanged`, with the first error as in the red case under `## Left`.
- If the wasm32 target is missing on the builder, follow the note: `Failed:`, nothing
  installed.

## Steps

1. From the repository root, at your base, run through `heavy`, once:
   `heavy cargo clippy --target wasm32-unknown-unknown -p temper-playground -- -D warnings`.
   On failure, read its log around the first `error` (`grep -n -m5 -A12 '^error'`), never
   whole, then go to 1b.

   1b. Red only: run once more without denying warnings, so clippy goes past the first failing
   crate and shows every warning at once:
   `heavy cargo clippy --keep-going --target wasm32-unknown-unknown -p temper-playground`.
   Read its log by `grep -n -E '^(warning|error)|^ *--> '`, never whole, list each under the
   report's `## Left` (crate, lint, file:line, item), push your branch at its base and stop
   after the report.
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
