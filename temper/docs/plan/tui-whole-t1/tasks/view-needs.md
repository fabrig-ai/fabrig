+++
id = "view-needs"
part = "needs"
kind = "interface"
needs = []
files = [
  "temper/crates/temper-client-view/src/shared/needs.rs",
  "temper/crates/temper-client-view/src/shared/question.rs",
  "temper/crates/temper-client-view/src/shared.rs",
  "temper/crates/temper-client-view/src/tests/shared.rs",
  "temper/docs/design/client/README.status.md",
]
effort = "high"
minutes = 75
+++

# The view says each need's mark and how the terminal walks it

## Goal

The TUI's launchpad marks a need "?", "$", "✓" or "•" and walks them one
prompt after another; the view's shared surface says which kind each is
and what its prompt offers, so the TUI derives nothing of the root's
again. Add `shared::need_prompts` and its types, exactly as
`notes/contracts.md`, section 2, has them; change nothing the browser
draws.

Acceptance:

1. **`src/shared/needs.rs`,** new, with `NeedMark`, `NeedPrompt`,
   `NeedAct` and `need_prompts` as the contract has them, re-exported
   from `src/shared.rs` in one line beside the others. `need_prompts`
   reads `root.pages().needs().needs(inbox, root.forge().approvals())`
   for the page's inbox (an empty box without one), in that order, and
   for each need:
   - `object`: the card's `object`; an approval's
     `c::ObjectKey::Task(row.task)`;
   - `project`: the card's `project`; an approval's `Some(row.project)`;
   - `mark`: by the view's `pages::ask` reading of a card (`Kind::Question`
     → `Question`, `Kind::Funds` → `Money`, any other → `Other`, every
     kind listed, no `_`); `Need::Approval` → `SignOff`;
   - `line`: `need_line(need, w)`, unchanged;
   - `act`: `Answer(question)` for a card `shared::question` would offer,
     built by one per-card function that `question` now calls too (its
     checks and its parse unchanged: a question group, no pending key, a
     project, a page, `Answer` admitted); `Review { words, address }` for
     an approval whose `yours` is `Yours::Counts` and whose
     `pages::approval::address(row, forges)` is some, `words` the
     browser's `Word::ApprovalReview` with the forge's kind as
     `pages/approval.rs` says it there; `Web` for any other.
2. `NeedKind`, `NeedLine`, `need_line` and `question` keep their shapes;
   `question` returns what it returns today for every input (a change of
   its inner shape only).
3. **Tests,** a failing test first (`notes/common.md`): in
   `src/tests/shared.rs`, one table test `need_prompts_say_each_kind`,
   a row each: a question with two parsed choices, the second
   recommended (`Question`, `Answer` with both choices and `primary`);
   a question not admitted for `Answer` (`Question`, `Web`); a task
   held on its budget (`Money`, `Web`); a task held otherwise (`Other`,
   `Web`); a proposal of settings (`Other`, `Web`); an approval the
   person's review counts on (`SignOff`, `Review` with its words and the
   forge's address); an approval with `Yours::NoAccount` (`SignOff`,
   `Web`); the order, a card later, an approval, a card (cards first,
   later last, as `needs()` orders them); no inbox (none). Build the
   cards and rows as `src/tests/needs.rs` and `src/tests/shared.rs`
   build them today. The existing `shared` parity rows stay green
   unchanged.
4. **Status:** delete the **Not yet** bullet naming `view-needs` in
   `temper/docs/design/client/README.status.md`. No other design line.
5. **Runs,** through `heavy`, counts in the report: `cargo nextest run -p
   temper-client-view --lib -E 'test(/shared::/)'`, then `cargo nextest
   run -p temper-client-view` once; `cargo check -p temper-client-tui
   --tests`, `cargo check -p temper-client-shell-terminal --tests` and
   `cargo check -p temper-client-shell-web --tests` (the view's direct
   dependents).

## Inputs

- `temper/docs/plan/tui-whole-t1/notes/common.md`; `notes/contracts.md`,
  section 2.
- `temper/docs/design/client/README.md`, 6.14, *Needs you* and *Walking
  what needs you*; 3.5, *Needs you* (the kinds and an approval's card).
- The code: `temper-client-view`'s `src/shared.rs`, `src/shared/words.rs`
  (`NeedKind`, `NeedLine`, `need_line`), `src/shared/question.rs`
  (`Question`, `question`), `src/pages/home.rs` (`need_words`, about line
  1150: how a need's kind is read today), `src/pages/ask.rs` (`read`,
  `Kind`), `src/pages/approval.rs` (`address`, `question`, the review's
  words, about line 174); `temper-client-domain`'s `src/pages/needs.rs`
  (`Need`, `needs`), `src/forge/approvals.rs` (`Row`, `Yours`); tests
  `src/tests/shared.rs`, `src/tests/needs.rs`.

## Owns and may change

Owns the files listed. Not `src/pages/home.rs` or `src/shared/words.rs`:
temper/ui's plans in flight (`ui-p2-home`, `ui-reaches`) change them; read
the kinds there, don't move them. Not `temper-client-domain`.

Options, each decided:

- **`pages::ask` or `pages::approval` items private to their module:** if
  `needs.rs` needs one that is private, make it `pub(crate)` with no other
  change, and name it in the report.
- **The forge's kind words:** if the approval card's "Review on FORGE ↗"
  is built inside a function that also does more, factor the words out
  into a `pub(crate)` function in `pages/approval.rs`, the card calling
  it, and say so; that file then joins this task's.

## Steps

1. The failing table, seen failing; then `needs.rs` and the refactor of
   `question`; the narrowest test after each.
2. The status edit; the runs of acceptance 5; `cargo fmt`; `python3 -I -B
   temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-client-view
   --all-targets -- -D warnings`; `python3 -I -B
   tools/citations/citations.py --base origin/main`.
3. Commit (why: the terminal marks and walks each need by the view's
   reading of it); push.

## Report

As the implementer's text says, with each row and the rule broken by hand
for it (for instance funds read as `Other`), and any item made
`pub(crate)`.
