+++
id = "journeys"
part = "journeys"
kind = "feature"
needs = ["tui-launchpad", "tui-walk", "shell-launchpad", "tui-views"]
files = [
  "temper/docs/design/client/tui/journeys/launchpad-chat/**",
  "temper/docs/design/client/tui/journeys/launchpad-drawn/**",
  "temper/crates/temper-fake/src/backends.rs",
  "temper/docs/design/client/tui/journeys.status.md",
]
effort = "high"
minutes = 120
+++

# `launchpad-chat` recorded again, and the new journey `launchpad-drawn`

## Goal

The launchpad as drawn is recorded for the user to take
(`temper/docs/design/client/tui/journeys.md`, 9): `launchpad-chat` again,
on a fresh fake box, and a new journey, `launchpad-drawn`, where the fake
box reaches the launchpad's states: a chat asked with `/change` whose fake
model asks the person a question, the launchpad with that chat and its
need, the walk's prompt, and the launchpad after it.

Acceptance:

1. **`launchpad-chat`** recorded again (`heavy cargo run -p
   temper-client-journeys -- again launchpad-chat`), its walk and setup as
   they are unless the launchpad's words make a `see` wait for words no
   longer drawn (then the nearest words drawn, named in the report); its
   `README.md` names each frame as before, the launchpad as drawn in 01.
2. **`launchpad-drawn`,** new, as 9's row has it: `README.md` (each frame
   named, as `launchpad-chat/README.md` names its), `setup` (as
   `launchpad-chat`'s, `after project-landing`), `walk`: approve; see the
   launchpad; paste `/change Add a CSV export to the report` and Enter;
   see the fake model's question; `/new` and Enter; see "n to walk";
   mark (01); key `n`; see "Next"; mark (02); Enter; see "Nothing else
   needs you."; end (03). Recorded by `again`, its pages printed by it.
3. **The fake model's question** (`temper-fake`'s `backends.rs`,
   `model_document`): one cue more, for a chat whose input holds "Add a
   CSV export to the report": it calls `message` with `form: "question"`,
   the words "csv-export: comma or semicolon?", two choices, "Comma"
   recommended and "Semicolon", then `wait`, as the cue
   `walk-away-mode` does today. Its answer, after the person's, finishes
   with a line. No other cue changes.
4. **Recording flakes** (`notes/common.md`, *Runs*): a re-record that stops
   on a moving screen is retried up to three times, each on a fresh fake
   box; if all fail, widen the mark's wait and say so.
5. **Status:** delete the **Not yet** bullet naming `journeys` in
   `temper/docs/design/client/tui/journeys.status.md`, and, where
   `launchpad-chat/04` now holds the person's opening question, the
   bullet on finding-465's `launchpad-chat/04` too; else leave it. No
   other design line.
6. **Runs,** through `heavy`, counts in the report: `cargo nextest run -p
   temper-client-journeys --lib`; `cargo nextest run -p temper-fake`.

## Inputs

- `temper/docs/plan/tui-whole-t1/notes/common.md`.
- `temper/docs/design/client/tui/journeys.md`, 3 (the recording, 3.3 a
  key's event), 6 (the print), 8.1 and 8.2 (`again`, its walk and setup
  files), 9 (`launchpad-chat`, `launchpad-drawn`); `journeys.status.md`.
- `temper/docs/design/client/README.md`, 6.14 (*The launchpad*, *Walking
  what needs you*, *The TUI's own commands*).
- The code: `temper/tools/journeys/src/main.rs` (`again`),
  `src/tui_again.rs`, `temper-client-shell-tui`'s `journey/walk.rs` (the
  steps a walk has: `see`, `key`, `mark`, `approve`, `end`);
  `temper-fake`'s `backends.rs` (`model_document`, the `walk-away-mode`
  cue); the journeys `temper/docs/design/client/tui/journeys/launchpad-chat/`
  (`walk`, `setup`, `README.md`).

## Owns and may change

Owns the files listed. Not the TUI's crates or the host: a frame that
shows a fault is reported under `## Left` with its page, not repaired
here. Not the browser's journeys.

Options, each decided:

- **The `/change` chat's template asks no question** (its charter has no
  `message` tool with a question form): ask the question from an Ask
  temper chat instead (the box's words, the cue keyed on them), and say
  so.
- **No brief on the fake box:** the launchpad shows Home's headline; the
  journey doesn't wait for a brief. If one shows, 01 holds it with
  " · /brief".

## Steps

1. The cue, with `temper-fake`'s tests green; then `launchpad-drawn`'s
   files and `again` for it; then `again launchpad-chat`.
2. Read every page against 6.14 and the request's frames; the status edit;
   the runs of acceptance 6; `cargo fmt`; `python3 -I -B
   temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-fake
   --all-targets -- -D warnings`; `python3 -I -B
   tools/citations/citations.py --base origin/main`.
3. Commit (why: the launchpad as drawn, recorded for the user to take);
   push.

## Report

As the implementer's text says, with each journey's pages and what each
shows, every retry of a recording, and where a page departs from the
mockup and why.
