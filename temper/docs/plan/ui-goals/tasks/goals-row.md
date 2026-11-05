+++
id = "goals-row"
part = "goals-row"
kind = "feature"
needs = ["goals-domain", "goals-look"]
files = [
  "temper/crates/temper-client-view/src/pages/goals.rs",
  "temper/crates/temper-client-view/src/words.rs",
  "temper/crates/temper-client-view/src/tests/goals.rs",
  "temper/crates/temper-client-view/src/tests.rs",
]
effort = "high"
minutes = 75
+++

# A goal's row, and the project's ways

## Goal

`goals::row` draws one goal as the mock's `goalRow`, and `goals::ways` names the project's ways. `ui-home-status` and
`ui-settings` build on both: keep the names.

Acceptance:

- `row` and `ways` as `temper/docs/plan/ui-goals/notes/contracts.md`, The view, gives them.
- The whole row is one link to `Address::Task { task, filter: PlanFilter::All }`, shaped as the contract's HTML in the browser.
- **The top:** the chip's title; `6 / 9 · ` from `delegates_done` and `delegates_total`, left out with its " · " at no delegates;
  the state's word: "needs funds", "needs you", "paused by you", "planning", "running", "landed " with the end in the clock
  form ("landed 01:12", "landed Mon 18:40"), "landed" alone with no end, "stopped", "failed".
- **The line:** the report's headline, or with no report: "Planning from the draft." planning; "N of M tasks landed." running or
  paused; "Landed."; "Stopped by you."; "Failed.". Needs takes running's, or planning's with no delegates.
- `row-ended` on landed, stopped and failed; each state's token, `failed` taking `state-stopped`.
- **The terminal:** one numbered link reading "TITLE · 6 / 9 · needs funds", then the headline as one run of words, by `w.shell`
  (`pages/home.rs`'s `step` is the pattern): a mark drawn by style isn't there.
- **Ways:** the project's templates with `tracked` set, in number order; `goal`: "Task by task", "The same plan; each task lands
  on its own once checked."; `goal for a draft`: "Plan, tasks, land whole", "A plan from the draft, its tasks on a goal branch,
  landed as one change once all are checked."; any other its own name and no line.

## Inputs

- `temper/docs/plan/ui-goals/notes/common.md`, `notes/contracts.md`.
- `temper/docs/design/client/README.md`, 3.5, *Goals*; 5.12.
- The mock's `goalRow` (`temper/docs/design/client/greenfield/app.js`).
- `temper/crates/temper-client-view/src/words.rs`, `Times`; `pages/home.rs`, `pages/chats.rs`'s `row`.

## Owns and may change

Owns, as `files` lists them, with `pages.rs`'s `pub mod goals` (was `pub(crate) mod`). A new word as the file's neighbours: the enum,
`ALL`, its number, its English; `PHRASES`, `SEGMENTS` and `text_bytes` as `english_phrase_declarations_match_the_vocabulary` asks.

Options, each pre-approved:

- The clock is a `Name` hole, as `pages/chat.rs`'s closed words have it, through `times.say(TimeForm::Clock, …)`.
- If `Word::Landed` has no hole, leave it and add `GoalLanded`, "landed {clock}".
- A title `cut` reads as it stands.

## Steps

1. **`row`,** with its words; **`ways`.**
2. **Tests,** `tests/goals.rs`, as `tests/chats.rs` reads a tree:
   - `a_row_is_one_link_to_its_task`.
   - `each_state_says_its_word_and_carries_its_token`: every row of the table.
   - `its_count_is_left_out_with_no_delegates`.
   - `its_headline_is_the_reports_or_its_states`: each fallback.
   - `an_ended_row_is_marked`.
   - `a_row_reads_in_the_terminal`.
   - `ways_name_their_templates_in_number_order`.
3. **Check,** the implementer's text, then push.

## Report

As `notes/common.md`, Your report, with the words' numbers.
