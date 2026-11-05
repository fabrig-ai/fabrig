+++
id = "needs-kinds"
part = "needs-page"
kind = "feature"
needs = ["needs-page"]
files = [
  "temper/crates/temper-client-view/src/pages/ask.rs",
  "temper/crates/temper-client-view/src/pages/needs.rs",
  "temper/crates/temper-client-view/src/tests/needs.rs",
  "temper/tools/gallery/src/needs.rs",
]
effort = "high"
minutes = 70
+++

# Needs you: a proposal, a held task, funds, a person task

## Goal

*Needs you* draws every kind of card, by `temper/docs/design/client/README.md`, 3.5, *By its kind* and
*SHORT*, and the gallery shows them.

Acceptance:

- **A proposal:** its question line the reason's first sentence (up to and including the first `. `,
  or the first line, or the whole; the batch's heading where a batch has no reason of its own); its
  facts and rows as `.ctx` beneath, through `crate::confirm::facts_at` and
  `crate::cards::Place::Inbox`; Accept (primary, no "recommended") and Reject (plain, a disclosure with
  its reason's textarea as `confirm::card_controls` has it), each through `Act::Decide` and the
  confirmation; the pair's first "Open" where `source_task` names a task, left out for a party's.
- **A held task:** its title as the question line, its words and facts as `.ctx`, Release (primary)
  and Leave held (plain, a disclosure with its reason's textarea, since `Decision::LeftHeld` needs a
  reason), through `Act::Decide`; "Open"; "Later". Kind "held".
- **Funds:** a held task whose `hold.reason` is `HoldReason::Budget { spent, limit }`: kind "funds", its
  question line "S of L used." with plain numbers, as the task page's `Word::BudgetLeft` has its money
  (no `$`: the contract's units aren't dollars), its title as owner on the mono line, then as a held
  task. No "Add $10".
- **A person task:** its title, its spec through `crate::cards::specification`, its acts (Take, Hand
  back) as plain buttons, "Open", "Later". Kind "a person task".
- **An act's outcome:** a sent `card.key` or `card.outcome` draws as `confirm::card_controls` has it,
  in place of the card's acts, sending at once, with no Undo and no answered line.
- **SHORT** for each kind, in `ask::read`, and "Next: TITLE · SHORT" over them; on a funds card "S of L
  used".
- Gallery: `needs-proposal`, `needs-held`, `needs-funds`, in `names.txt` directly after `needs-done`,
  in that order and before `needs-plain-words`, each with its `HOLDS` row. A person task is tested, with
  no gallery state.

## Inputs

- `temper/docs/plan/ui-needs/notes/common.md`, `notes/contracts.md`.
- `temper/docs/design/client/README.md`, 3.5, *Needs you*, *By its kind*, *SHORT*; 3.6.
- jig's `HoldReason`: `jig/crates/jig-client-domain/src/contract/documents.rs`.
- `temper/docs/design/client/greenfield/data.js`, for a proposal's and funds' words.

## Owns and may change

Owns `files`. May also change:

- `temper-client-view`: `src/words.rs` (`funds`, `NeedsFundsUsed` and the short form, each a word),
  `src/look.rs`, `src/limits.rs`, `src/confirm.rs` and `src/cards.rs`, `src/cards/proposal.rs` for
  `pub(crate)` or a drawing split, the terminal's print the same.
- `temper.css`, under the marker: the `.ctx` facts' look, the plain and primary acts.
- The gallery: `tools/gallery/names.txt`, `src/samples.rs`, `src/tests.rs`,
  `tools/gallery/src/worlds.rs` only if a builder is to be shared: `worlds.rs:824` has no Escalation or
  PersonTask entry, so make the fixtures in `src/needs.rs`.

Options, each with its default:

- A proposal's reason with no sentence end: the whole, its first line.
- `HoldCategory::Exceeded` isn't read here: the card is funds by `HoldReason::Budget`.
- A held task with no `hold` on its page: "held", with no facts.

## Steps

1. `ask.rs`'s `Kind` and `read`; the kinds' arms in `needs.rs`. Tests in `src/tests/needs.rs`: each
   kind's parts in order and acts; Reject's and Leave held's reason; Open with and with no task; funds
   by reason, and a held task of any other reason not funds; SHORT and the next-up line over a mix; the
   confirmation still opens over the page; a card sent shows its outcome and no acts; terminal unchanged.
2. Gallery, CSS, render against `needs.html` where the mock draws the kind, and `README.md`'s words
   where it doesn't.
3. `heavy cargo nextest run -p temper-client-view --lib`; `-p temper-client-gallery`; `-p
   temper-client-world`. fmt, clippy and the checks of the implementer's text. Commit and push.

## Report

As the implementer's text asks, and each difference from the mock, with its reason.
