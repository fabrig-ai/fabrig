+++
id = "status-nine"
part = "browser"
kind = "docs"
needs = ["native-walk-a", "one-process-replay", "bundle-playground", "browser-walk"]
files = [
  "temper/docs/design/playground.status.md",
  "temper/docs/design/STATUS.md",
]
effort = "medium"
minutes = 20
+++

# playground.status.md's 9, as the plan's work leaves it

## Goal

Four tasks of this plan close the gaps of `temper/docs/design/playground.status.md`'s **Not yet**
9 ("the native walk, the wasm32 builds in the gate, a log replayed and the browser tier's walk"),
and two edits of one line by parallel branches would not merge; so this task, after them, edits
the entry once, from what the trunk holds.

Acceptance:

- In `temper/docs/design/playground.status.md`, **Not yet** 9, remove the words for each gap the
  trunk closes, in place, never reflowed (`notes/common.md`), and leave the rest word for word:
  - "the native walk": closed when `temper/tests/playground/tests/suite/walk.rs` walks story A
    (`native-walk-a`);
  - "a log replayed": closed when the kept log replays under the one-process world's referee
    (`temper/tests/one-process/tests/suite/replay.rs`, `one-process-replay`);
  - "the browser tier's walk": closed when `temper/tests/client-browser/tests/browser_all/
    browser_playground.rs` exists (`browser-walk`);
  - "the wasm32 builds in the gate": closed when `.config/gate/wasm32.txt` holds both
    `temper-playground` and `temper-client-shell-web` (the second linting the playground's page,
    `notes/contracts.md`, C5); plan `wasm32-temper` adds the first, and may have removed these
    words already: leave what you find.
  When all four are closed, 9 goes from the entry. 10 (the bar) stays: Firefox and Safari by hand,
  the bars the user sets, and the release.
- `temper/docs/design/STATUS.md`'s summary paragraph, where it names `temper-playground`'s lag,
  stays true: edit its words only if 9's removal makes them false, in place.
- The date lines today's; `python3 -I -B tools/citations/citations.py --base origin/main` clean.

## Inputs

- `temper/docs/plan/playground-browser/notes/common.md`; `DESIGN.md`, 2.
- `temper/docs/design/playground.md`, 9 and 10.

## Owns and may change

- The two files in `files`, as above, and nothing else. If a gap's task has not landed (its file
  above is missing), leave its words and list it under `## Left`.

## Steps

1. Read the trunk's files named above; edit the entry; the citations check.
2. Commit ("Close what the playground's browser plan built in playground.status.md's 9"); push.

## Report

As the implementer's text says, and also: the entry before and after, quoted.
