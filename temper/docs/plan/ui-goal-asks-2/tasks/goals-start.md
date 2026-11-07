+++
id = "goals-start"
part = "start"
kind = "feature"
needs = []
files = [
  "temper/crates/temper-client-domain/src/pages/start_goal.rs",
  "temper/crates/temper-client-domain/src/pages/goal.rs",
  "temper/crates/temper-client-domain/src/tests/pages/start_goal.rs",
  "temper/crates/temper-client-domain/src/tests/pages/goal.rs",
  "temper/crates/temper-client-view/src/pages/start_goal.rs",
  "temper/crates/temper-client-view/src/pages/goals.rs",
  "temper/crates/temper-client-view/src/tests/start_goal.rs",
  "temper/crates/temper-client-view/src/tests/goals.rs",
  "temper/crates/temper-client-view/src/tests/goal.rs",
  "temper/crates/temper-client-shell-terminal/src/tests/golden/**",
  "temper/crates/temper-client-shell-terminal/src/tests/fixtures/{goals,task}.rs",
  "temper/tools/gallery/**",
]
effort = "high"
minutes = 110
+++

## Goal

Three things a person trips over when starting and following a goal. Goals' "Start one from a draft" goes to `/chats`, where the design has it open the Start goal sheet with a Draft field. The sheet starts a goal with a budget of 0, which holds before any model runs. And a goal says "2 landed" when a person task and an advisor finished and nothing landed. Build the design's sheet, say why when nothing funds a goal, and count landings where the rows can tell.

Acceptance:

1. Goals' "Start one from a draft" opens the sheet with no draft, for the project in the address: heading "Start a goal", no lead paragraph, a "Draft" field with its placeholder, then Name, which follows the Draft's file without `.md` until the person edits it. Start stays disabled while Name or Draft is empty.
2. `Act::OpenStart { draft: None, chat: None }` opens that sheet; a draft with no chat, or a chat with no draft, opens nothing. A sheet given a draft keeps today's heading, lead and fixed draft: `Act::StartDraft` changes nothing there.
3. Starting from it sends the same `Ask::Create` a chat's sheet sends but for its words, which `start_goal::words` makes with no `Chat:` line: `Build PATH.` and `Draft: PATH`, the path trimmed, and `Asks: always` if chosen. It writes no `GoalFrom` fact and goes to the goal's page. In the client world that goal delegates a person task and the person's answer wakes it, as a chat's does.
4. While `creating.within.budget` is 0 and the way's `Template.budget` isn't `TemplateBudget::Amount(_)`, the sheet, given a draft or not, draws `StartGoalNoFundsHeld` where the party's `pool` has `numbers.reserved` above 0, otherwise `StartGoalNoFunds`, as `.sheet-error` above the foot. Start is disabled and `Act::StartGoal` sends nothing. It clears when `Me` next delivers a budget. With no `creating`, the sheet says what it says now.
5. `goal::line` follows `notes/contracts.md`, *A landing*. On a goal's page the bar's name and the fold count landings alone; `Line::Done` rows stand with the unlanded ones, reading "done" and their end in the clock form, and take no bar class.
6. A Goals row's headline and a goal's page's say "N of M tasks done.". `Counts.done` stays every delegate ended done.

## Inputs

- `temper/docs/design/client/README.md`, 5.14, all of it; 3.5, *Goals*, the rows' headline and its filters; 5.8, the header's `<h1>` and *Tasks*; section 11, *A goal row's count of landings*.
- `temper/docs/design/client/greenfield/` (`goals.html`, `app.js`'s `startSheet`): the look.
- `temper/docs/plan/ui-goal-asks-2/notes/contracts.md`: *New words*, *A landing*, *Not changed*.
- `temper/docs/plan/ui-goal-asks-2/notes/common.md`, which gives you your **Not yet** entry in `temper/docs/design/client/README.status.md`, the second of the two naming this work: remove it. Its **Design departs** entry stays.
- **Not in scope:** the rail's More ways → Goal, which 5.3 has opening a new draft chat. Leave it.

## Owns and may change

**Owns:** the files in the header.

**Readers and neighbours, each for the stated edit only:**
- `temper/crates/temper-client-domain/src/pages.rs`, line 502: `Act::StartDraft { words }` reaches the sheet, leaving `Act::Away(_)` as it stands.
- `temper/crates/temper-client-view/src/frame.rs`, near line 85: `(PageAct::StartDraft { .. }, skein_ui::Act::Input { value })`.
- `temper/crates/temper-client-view/src/words.rs`: your words, as `notes/contracts.md` has them.
- `temper/crates/temper-client-view/src/pages/goal.rs`: the Tasks section and its `line_words` and `row`, and the word at line 397. `question-whole` changes `band` at the same time, so keep off it.
- `temper/crates/temper-client-shell-web/temper.css`, under `/* ==== ui: goals ==== */`: `button.temper-sheet-link { border: 0; background: none; padding: 0; cursor: pointer; }`, in the link's look.
- `temper/tests/client/tests/suite/chat_draft.rs`: one new test at the file's end.
- `temper/tests/client/tests/suite/task_pages.rs` (near 161) and `b2.rs` (near 52, 115), which read "N landed of M": their delegates' results become a change's in their fixture if they are to land, or their expected words move, whichever keeps what each story checks. Name which in the report.
- `temper/crates/temper-client-domain/src/tests/pages/goal.rs`, `line_uses_the_notes_precedence_in_every_case`: a row for each.
- `temper/tools/gallery/src/{worlds,tests}.rs`, `temper/tools/gallery/` prints and `temper-client-shell-terminal` goldens, which say "landed": a sample's delegates end with a change's result so that "6 landed of 9" stands. "N of M tasks landed." moves to "done". A headline in a sample's own report ("Six of nine tasks landed; …") is that report's words and stays. Regenerate by their tools.
- `temper/docs/design/client/journeys/**` recordings: `temper-client-journeys again`, if a focused replay check goes stale.

**Options, each pre-approved:**
- If sharing `delegation.rs`'s test of a landed change as one function is plainer than a second pattern, share it; `temper-client-domain/src/pages/delegation.rs` may change for that alone.
- If the Goals link must stay `Role::Link` for `crate::link`'s `b.link`, it can't: it moves nowhere, so it's a `Role::Button` in `look::SHEET_LINK`. `tests/goals.rs` near 772 and 795 change with it.
- If `temper-client-view/src/limits.rs` prices the sheet's fields by count, one more `text` field goes there.

## Steps

**1. Failing tests first,** as rows of what exists:

- `temper-client-domain/src/tests/pages/start_goal.rs`: `missing_me_does_not_open` over the four shapes of `draft` and `chat`; `the_default_follows`' neighbour for Name following the Draft and stopping once edited; `start_needs_name_and_draft` over a blank path; `create_sends_and_goes_to_its_goal` over no chat and no draft, with a row in `follow_adds_none`'s words; `refused_stays_open_with_jigs_words`' neighbour, a table of `within.budget`, `Template.budget` and `pool.numbers.reserved` against whether it sends.
- `temper-client-view/src/tests/start_goal.rs`: `no_draft_opens_no_sheet` becomes the sheet of acceptance 1, in both shells; `a_draft_names_the_sheet_and_its_name` holds a given draft has no Draft field; the two funds lines and the disabled Start.
- `temper-client-view/src/tests/goals.rs`, near 772: the button and its `Act::OpenStart`.
- `temper-client-view/src/tests/goal.rs`: `landed_rows_fold_under_n_landed_with_their_end` gets a person task's `Words` result and an advisor's `Report`: "1 landed of 3", a fold of 1, and two "done" lines; `otherwise_it_counts_what_landed` says "done".

**2. `pages/start_goal.rs`:**

```rust
/// Nothing to fund a goal, and why (client/README.md, 5.14).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Unfunded { Held, Empty }
#[must_use]
pub fn unfunded(me: &c::Me, project: u32, way: Option<u32>) -> Option<Unfunded>;
```

`StartGoal` gains a private `named: bool`, set by `name`; `pub(super) fn draft_field(&mut self, words: Box<[u8]>)`, doing nothing while `given` is set or a request is out. `open` takes `(None, None)` to a blank sheet, for `super::conversation_project(pages)`. `send` trims the path, and returns at `unfunded`.

**3. Views:** `pages/goals.rs`, the button. `pages/start_goal.rs`: the lead only with `given`; the field before Name; the line above the foot, in the look `failure` uses; the disabled Start.

**4. `pages/goal.rs` (domain):** `Line::Done`; `line`; `counts`' arm. **`pages/goal.rs` (view):** the count, the bar's arms, `row`, `line_words` with `GoalDoneAt`, and the word.

**5. The world,** `temper/tests/client/tests/suite/chat_draft.rs`, a new test at its end beside its siblings' fixture: open the Goals page, press to `Act::OpenStart { draft: None, chat: None }`, `Act::StartDraft` a path, `Act::StartGoal`, assert the goal's words; drive its person task as `a_goal_from_the_sheet_asks_its_person_who_answers_under_needs_you` does and assert the answer wakes it. Share that test's driving as a helper if that's plainer.

**6.** `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy` naming each package changed, `--all-targets -- -D warnings`; `heavy cargo nextest run -p temper-client-domain -p temper-client-view -p temper-client-shell-terminal -p temper-client-gallery -p temper-client-world`. `goal::line` changes what a shared item means, so `temper-client-view`'s readers of it, Home's among them, run. Commit and push.

## Report

Name the rules broken by hand and what failed, which stories changed fixture or words, and whether a journey was re-recorded.
