+++
id = "reference-relay"
part = "reference"
kind = "docs"
needs = ["relay-foreman", "foreman-charter"]
files = ["temper/docs/examples/reference/settings.json", "temper/crates/temper/src/tests/reference_settings.rs", "temper/docs/design/protocol/configuration.status.md"]
effort = "medium"
minutes = 40
+++

# reference-relay: the reference's coordinator, coding agent and foreman for questions

## Goal

The reference's settings file carries this milestone's editions (`configuration.md`, 13.9):
the coordinator makes its changes with questions `foreman`, answers, amends, makes a change
a question waits on, asks its person and decides holds; the coding agent asks its change
where it cannot choose; the foreman's charter 6 adds the reference's rules to the stock's.
The texts are `notes/settings.md`'s; a test reads them.

Acceptance:

- `temper/docs/examples/reference/settings.json`: charter 5's instructions edited as
  `notes/settings.md` says (the cut paragraph's one insertion; the question paragraph
  replaced by its two), charter 2's gaining its paragraph, and a fourth item, charter 6,
  as that note gives it. Every other paragraph, and every member not named there, stays as
  the trunk has it, including what other plans' tasks have written since this note.
- `temper/crates/temper/src/tests/reference_settings.rs`: `ITEMS` 4; charter 6 decodes with
  model `gpt-6.1-sol`, `modify` and `agents` false, the workers, and instructions starting
  with `include_bytes!("../../../temper-service/src/stock/foreman.txt")`; charter 5's
  instructions contain `"questions": "foreman"` and `decide`; charter 2's contain `in the
  question form`.
- `configuration.status.md`: remove the **Not yet** sentence "13.9: the reference's charter
  6, its coordinator's instructions for questions and holds, and its coding agent's for
  asking."

## Inputs

- `temper/docs/design/protocol/configuration.md`, 13.9 (the settings file's charters 5, 2
  and 6; the rules and policy 1 on the box, role 0); `temper/docs/design/domain/forge.md`,
  8.11 (what the coordinator answers).
- `temper/docs/plan/reference-m5/notes/common.md`; `notes/settings.md`.

## Owns and may change

Owns the files in the header.

- If the trunk's charter 5 text no longer holds the cut sentence or the question paragraph
  as `notes/settings.md` quotes them (another plan rewrote them), make the same insertion in
  the sentence that now makes the changes, and put the two paragraphs where the question
  paragraph stood or, gone, before the goal's end paragraph; say what you found.
- If the settings file's decode refuses a fourth charter for `tasks.charters` in the limits
  the test's form uses, report the bound and stop: the limits are `foreman-charter`'s.

## Steps

1. The file's edits; the test.
2. Narrow: `heavy cargo nextest run -p temper --lib -E 'test(/reference_settings/)'` (check
   the count), then `heavy cargo nextest run -p temper-service --lib -E
   'test(/reference_landing/)'`.
3. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p temper
   --all-targets -- -D warnings`. Commit ("Teach the reference's agents to ask and answer")
   and push.

## Report

As the implementer's text says, with what of the trunk's charter 5 text you found.
