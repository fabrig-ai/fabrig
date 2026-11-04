+++
id = "status-words"
part = "status"
kind = "docs"
needs = ["temper-names"]
files = [
  "temper/docs/design/client/browser.status.md",
  "temper/docs/design/playground.status.md",
]
effort = "medium"
minutes = 10
+++

# Two of temper's status files stop naming the pipeline, once their rewrite has landed

## Goal

fabrig names nothing of whoever runs its development (`AGENTS.md`, 1). Two of temper's status
files call the gate's wasm32 list "fab's gate list". The playground-browser draft is rewriting
the very entries that hold the phrase (its `browser-walk` and `status-nine` tasks), and its
versions still say it. To keep off its lines, this task changes a file only where that rewrite
has already reached your base, and then only the phrase: "fab's gate list" → "the gate's list".
Where it has not, the file stays as it is and the inventory keeps its entry.

Acceptance:

- For each of the two files, the test of `## Steps`, 1 decides:
  - **rewritten at your base:** every "fab's gate list" in it now says "the gate's list", and
    any other name the check counts in it is changed by `notes/contracts.md`, 3. Nothing else
    changes: not the date line, not a gap, not a label.
  - **still this plan's base wording:** the file is unchanged.
- For each changed file, the check of `notes/common.md` ("Checking your files are clean"), with
  its entry deleted, passes (1 selected); the inventory is then restored and not committed.
- `python3 -I -B tools/citations/citations.py --all` prints `citations: 0 broken in HEAD`.
- With neither file changed, the task is done with no commit: push the branch as it stands (at
  its base), and report `Done:` with both files left and why.

## Inputs

- `temper/docs/plan/outward-names/notes/common.md`: the standing rules.
- `temper/docs/plan/outward-names/notes/contracts.md`, 3: the wording.
- `DESIGN.md`, 2: a status file records the gaps as they stand; a wording change moves no gap.

## Owns and may change

- **The two status files of `files`**, only the words holding a name, and only under the
  "rewritten" outcome.
- **Departures from the implementer's text, with their reasons:** this task edits
  `docs/design/` status files, which the implementer's text leaves to designers: the request
  asks the lab's names out of them (`inputs/request.md`, Files), and the change is a phrase in a
  sentence, never a gap or a date. No Rust changes, so no `cargo fmt`, clippy or nextest of a
  tree's package; no old-forms run (no code). The one heavy command is the check above.
- Nothing else: not `tests/boundary/outward.txt`, not any other `docs/design/` file.

## Steps

1. For each file, find its lines holding the phrase: `rg -n "fab's gate list" <file>`. The file
   is **still this plan's base wording** when every such line appears unchanged in
   `git show 444de312a3:<file>` (main had browser.status.md's "  fab's gate list
   (`.config/gate/wasm32.txt`) leaves it out; its first" and playground.status.md's "    fab's
   gate list (`.config/gate/wasm32.txt`) holds neither"); otherwise it is **rewritten**. A file
   without the phrase and with no other name is unchanged and needs nothing.
2. Under "rewritten", change the phrase, rewrapping within its entry only if the line grows past
   its neighbours'.
3. For each changed file, the check, as `notes/common.md` says; restore the inventory.
4. Commit; citations; push.

## Report

As the implementer's text says, and: each file's outcome with the lines that decided it; each
changed line before and after; the check's selected count; anything under `## Left` (a file
left at base wording goes there: the playground-browser draft's rewrite had not landed).
