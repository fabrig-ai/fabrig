+++
id = "confirm-sheet"
part = "confirm-sheet"
kind = "feature"
needs = []
files = [
  "temper/crates/temper-client-view/src/confirm.rs",
  "temper/crates/temper-client-view/src/cards.rs",
  "temper/crates/temper-client-view/src/look.rs",
  "temper/crates/temper-client-view/src/words.rs",
  "temper/crates/temper-client-view/src/tests/confirm.rs",
  "temper/crates/temper-client-view/src/tests/look.rs",
  "temper/crates/temper-client-domain/src/acts.rs",
  "temper/crates/temper-client-domain/src/tests/acts.rs",
  "temper/crates/temper-client-shell-web/temper.css",
  "temper/tools/gallery/**",
]
effort = "high"
minutes = 170
+++

# Every confirmation is one sheet, and a closed chat ends on one line

## Goal

Land `close-chat`'s markup, words and rules: `confirm.rs` draws every confirmation as the mock's sheet, with its
own title and verbs, saying only what will change; a sheet stays until its ask is answered, and says why above
its acts when it fails; a closed chat ends on one quiet line. What closing does doesn't change.

Acceptance, each held by a test named below:

1. **The sheet:** every row of `temper/docs/design/client/README.md`, 3.6, *The sheet*'s table draws its title,
   its lines and its two acts, the keep first, then the act with `look::CONFIRM_DO`. No confirmation of
   `confirm.rs` says "Confirm", "Cancel" as its keep, "Dangerous action", "Close chat now" or a count of done,
   failed or cancelled tasks.
2. **What it changes:** every line of *What it changes*, each only where it holds, no count at zero, one alone
   keeping the hairline, none at rest. A delegate's row offers the other fate, and pressing it changes the row
   (`Act::DelegateFate`, unchanged).
3. **It stays until answered:** the press sends, once; a done answer closes it; any other outcome leaves it open
   saying why above its acts, the chat's chosen fates and `restarting` kept; a press there sends again; the keep
   closes it at any time with an ask left in flight; a press that sends nothing closes it.
4. **A press stays under the finger:** the acts' buttons keep their elements when a why arrives above them and
   when the page updates behind.
5. **Closed:** the end line says "Closed at 21:05" (`look::ENDED`) between two hairlines, the bar's status
   "Closed" with no time, and neither "Applied" nor the closing's "Decision made" is drawn. "Failed 21:05" and
   "Cancelled 21:05" keep their words.
6. **The terminal:** the same lines as text and the acts by their names; its prints keep their meaning.
7. **The stylesheet:** the request's rules are in `temper.css` under `/* ==== ui: chats ==== */`, and the old
   `.terminal-dialog` rules no sheet reads are gone.

## Inputs

- `temper/docs/plan/look-round-10/notes/common.md`, and the request it names, whole.
- `temper/docs/design/client/README.md`, 3.6: *Said before it is sent*, *The sheet*, *Closed ends on one line*;
  `browser.md`, *A press a render can lose*.
- `temper/docs/design/draft/look-requests.md`, 3, steps 1 to 6.

## Owns and may change

Owns the files in the header. May also change:

- **`temper-client-domain/src/pages.rs`** (`confirmation`, `told`, `task_confirmation`'s literal),
  **`pages/chat.rs`** (`close`, `prepare_close`), **`pages/task.rs`** (`send`'s return, the goal's toast for an
  outcome the sheet holds), and their tests under `src/tests/`: this task's lines only. `tests/acts.rs`,
  `tests/pages/task.rs`, `goal.rs`, `chat.rs`: an assertion `confirmation().is_none()` straight after a press
  moves to after the answer.
- **`temper-client-view/src/pages/chat.rs`:** `closed_status`, `act_status`, `hidden_line` alone; its test
  files `tests/chat.rs`, `tests/goal.rs`, `tests/needs.rs`, `tests/home.rs`, `tests/fixtures.rs`: a word, a name.
- **`pages/task.rs`:** only the progress and outcome lines a sheet now holds.
- **Readers of the old names and words,** a name or sentence alone: `temper/tests/client/src/cards.rs`,
  `temper/tests/client/tests/suite/{task,open_chat,chat,inbox,change_panel_edges,configuring_chat}.rs`,
  `temper/tests/client-browser/tests/browser_all/browser_pages.rs`, `temper/crates/temper/tests/support/person.rs`,
  `temper-client-shell-terminal/src/tests/**`.
- **`temper/docs/design/client/browser.md`:** the gallery's table, a row for each new state.
- **`temper/docs/design/client/journeys/**`:** only by regenerating, if the journeys' test finds a page changed.

Options, each pre-approved:

- **`Word::Confirm` and `Word::Cancel` stay** where others read them (`accounts/words.rs`, forges); only
  `confirm.rs` stops using them. A word no one reads goes (`CloseNow`, `CancelDelegate`, `TakeDelegate`,
  `DelegateCancel`'s old text).
- **The closing's "Decision made":** `Activity::Decided` has only a revision, and jig's `control.rs` appends one
  for each history row, so drop the last one in a chat whose end is `end_closed`. If that needs more than a few
  lines, `hidden_line` hides every `Decided` on such a chat; say so.
- **A sheet's progress or why lacks a rule of the request's:** write it beside the request's, as
  `dialog.temper-welcome-sheet[open] > [role=alert]` is written, and name it. The proof accounts for it.
- **`look::ENDED`'s old rule at `temper.css` line 1508** gives way to the request's where they meet.
- **The phone's `dialog > button:nth-last-child(2) { min-height: 44px; }`** points at `.temper-confirm-acts >
  button` instead, keeping a phone's target; the proof names the difference from the overlay's 38 px.
- **Pause, Resume and a chat's Stop stay at once,** sending no sheet, and a reject's reason disclosure stays in
  its card.

Leave: jig's words and `jig-client-view/src/tree/confirmation.rs` (jig's own presentation keeps its marks),
`look-round-6`'s conversation drawing, every sheet that isn't `confirm.rs`'s (`start_goal.rs`, `home.rs`,
`forge/connect.rs`), though the rules they read are yours to keep working.

## Steps

### 1. The domain: `acts.rs`, `pages.rs`

`Confirm` gains what the sheet reads, and one rule replaces the three closes on the press:

```rust
pub struct Confirm {
    pub object: ObjectKey,
    pub revision: Option<contract::Revision>,
    pub act: ActKind,
    pub again: Option<bool>,
    /// The ask this sheet sent, until it is answered (client/README.md, 3.6, *The sheet*).
    pub sent: Option<jig_client::Key>,
    /// Its last answer other than done, which the sheet says above its acts.
    pub failed: Option<jig_client::Outcome>,
}
```

- `Act::Confirm`, on all three paths (`pages.rs`'s task path, its closing path, `acts::step`): with `open.sent`
  set it does nothing. Otherwise it clears `failed` and sends. `acts::send`, `Chat::close` and `task::Task::send`
  return the `Key` they sent, or `None`: `Some` goes in `open.sent`, and `None` closes the sheet, but for
  `acts::send`'s reject without a reason, which keeps it as it does today.
- `Chat::close` clones `closing` rather than taking it.
- At the top of `pages.rs`'s `told`, on `Told::Answered { key, outcome }` with `open.sent == Some(key)`: `sent`
  clears; done (`seen_answered`) closes the sheet, and on a chat clears `closing`, while the chat's `Show
  Configure` and `restarting = false` move under done alone; anything else goes in `failed`. Then the told goes
  on as today, so that a kept-open card and a closed sheet's page both have the outcome they have now.
- A `card.key` left by a failed ask mustn't stop the press sending again: clear it on that answer, if `acts::told`
  doesn't already.
- `Act::Cancel` stays as it is, and leaves an ask in flight to answer the page.

### 2. The words: `words.rs`

New words, with ids the way their neighbours have them, exact bytes from the request (`’` is `\xe2\x80\x99`):

| Word | Says |
|---|---|
| `CloseTitle` | Close this chat? |
| `CloseCannotReopen` (new text) | You can still read it, but it can’t be reopened. |
| `CloseChat` | Close chat |
| `KeepIt` / `KeepGoing` / `KeepHeld` / `NotYet` | Keep it / Keep going / Keep it held / Not yet |
| `CloseGrace`, `CloseGraceNanos` (new text) | The run under way gets N seconds to finish, then stops. (nanoseconds likewise) |
| `CloseRunNow` | The run under way stops now. |
| `DelegateNumber` | T`N` |
| `DelegateCancel`, `DelegateMove` (new text, a Name) | ` TITLE is cancelled.` / ` TITLE moves to you.` |
| `TakeInstead` / `CancelInstead` | Take it instead / Cancel it instead |
| `TasksBelow`, `TasksBelowOne` | N tasks below it are cancelled. / 1 task below it is cancelled. |
| `ProposalsWithdrawn`, `ProposalWithdrawn` | N proposals are withdrawn. / 1 proposal is withdrawn. |
| `EffectsWithdrawn` | Changes not yet begun are withdrawn. Those under way settle. |
| `WhatClosingDoes`, `WhatCancellingDoes` | What closing does / What cancelling does (the list's name) |
| `StartOverTitle` | Start over? |
| `StartOverQuestion` (new text) | A new configuring chat starts. This one stays in Chats. |
| `GoalStopTitle` (new text, no hole) | Stop this goal? |
| `StopGoal` | Stop goal |
| `CancelTaskTitle`, `CancelTask` | Cancel this task? / Cancel task |
| `StopTaskTitle`, `StopTask` | Stop this task? / Stop task |
| `AcceptTitle`, `RejectTitle`, `ReleaseTitle` | Accept this proposal? / Reject this proposal? / Release this task? |
| `TimeClosed`, `TimeEnded`, `TimeEndedReport` (new text) | Closed at CLOCK (and its report, after it, as today) |

The acts "Start over", "Accept", "Reject" and "Release" are today's `StartOver`, `Accept`, `RejectProposal` and
`NeedsRelease`.

### 3. The view: `confirm.rs`, `look.rs`, `cards.rs`

- `look.rs`: `CONFIRM` (`temper-confirm`), `CONFIRM_FATES`, `CONFIRM_ACTS`, `CONFIRM_DO`, next free tokens; rows in
  `tests/look.rs`.
- `confirm.rs` draws through one set of small functions, in the request's node order, `02.html`'s shape:

```rust
/// Each confirmation's sheet (client/README.md, 3.6, *The sheet*).
struct Sheet { title: Word, keep: Word, does: Word }
fn sheet_open(b: &mut Builder, words: &Words, title: &[u8]);           // Role::Dialog, look::CONFIRM, name, Heading 2
fn sheet_why(b: &mut Builder, root: &Domain, words: &Words);           // open.sent's progress, open.failed's outcome_words
fn sheet_acts(b: &mut Builder, bindings: &mut Bindings, words: &Words, sheet: &Sheet); // keyed Role::Slot, look::CONFIRM_ACTS
fn fates(b: &mut Builder, /* ... */);                                  // Role::List, look::CONFIRM_FATES, its name
```

- `main`'s card path takes its `Sheet` from an exhaustive match on `ActKind`: Accept, Reject and Release draw;
  the rest return, as an act no sheet serves does today. `main`'s task path and `close` use the table's rows.
- Under `sheet_open`: the lead, then `fates`, then the card's facts as today (`proposal_at`, `decision`, the
  reason's paragraph), then `sheet_why`, then a Release's uncertain pair (`Word::Again`, `Word::DoNotRepeat`) in a
  slot of its own, then `sheet_acts`.
- `fates` takes `close_choices`' place, a pure reading of the operations and `jig_client::facts::effect`, the
  run first, each delegate's `Role::Item` keyed by its task, with `b.text(Run::Strong, ...)` then `b.text(Run::Plain,
  ...)` and its button `Act::DelegateFate` for the other fate; the title from `chat.task`'s `delegates` by number,
  with no title if it has gone. `TasksBelow`'s N is `waiting + running + held + closing` less the delegate rows
  listed, saturating. With no line to draw, draw no list. A cancel of a task with no delegates uses the same
  function, with `WhatCancellingDoes`.
- `cards::decision` drops `dangerous` (`jig_client::facts::DecisionFacts`' field is public) and no longer says a
  Cancel's impact: the list says it.
- Guards, so that what the sheet says isn't said twice while it is open for that object: `chat.rs`'s `act_status`,
  and only the outcome and progress lines of `controls_at` and `pages/task.rs`.

### 4. A closed chat: `pages/chat.rs`

- `closed_status` says `Word::ChatClosed`, with no clock; drop what that leaves unused.
- `act_status`: a `Sending::Close` that was accepted (`super::chats::accepted`) says nothing.
- `Decided`: as the option above.

### 5. The rules: `temper.css`

Move the overlay's rules whole under the marker, but `.temper-look-backdrop`. Then take out the `.terminal-dialog`
block's rules at lines 310 to 325 that no sheet reads. `SHEET`, `WELCOME_SHEET` and `forge/connect.rs` still lean on
`dialog`'s frame (position, inset, z-index, `max-height`) and on `dialog textarea`: keep those, check by a throwaway
render of `chat-draft/17.html` and a welcome sheet at 1440 and 420 that they don't move, and name what stayed.

### 6. The gallery

Add states the proof needs, in `samples.rs`, `tests.rs` and `names.txt`, and rows in `browser.md`: `chat-closing-work`
(02), `chat-closing-failed`, `chat-closed` (03), `chat-closing-start-over`, `task-cancelling`, `task-stopping`,
`proposal-accepting`, `proposal-rejecting`, or the name a state of that kind already has. Keep `chat-closing`,
`goal-stop-dialog` and `task-releasing`, with their holds' new words. Where a world can't reach one (a chat with a run
and two delegates, say), add what the world lacks, in the gallery's own fixtures.

### 7. Tests

- `temper-client-domain/src/tests/acts.rs`: `the_sheet_stays_until_its_ask_is_answered`,
  `a_failed_ask_keeps_the_sheet_and_presses_send_again`, `the_keep_closes_it_with_an_ask_in_flight`,
  `a_press_that_sends_nothing_closes_the_sheet`, `a_second_press_in_flight_sends_nothing`.
- `tests/pages/chat.rs`: `a_failed_close_keeps_its_fates_and_restarting`, `a_done_start_over_shows_configure`.
- `temper-client-view/src/tests/confirm.rs`, on the sheet: `each_confirmation_has_its_title_and_acts` (the design's
  table, one row each, read from the tree), `a_chat_at_rest_says_only_it_cant_be_reopened`,
  `a_chat_at_work_lists_what_closing_changes_in_order`, `no_count_at_zero_and_one_alone_keeps_the_list`,
  `a_delegates_row_offers_the_other_fate`, `a_failed_confirmation_says_why_above_its_acts`,
  `no_sheet_says_dangerous_or_done_counts`.
- `a_why_arriving_keeps_the_acts_elements`: `skein_ui::diff` between the sheet before and after, with no removal of
  its buttons in the patches; and the same with a section arriving on the page behind.
- `tests/chat.rs`: `a_closed_chat_ends_on_one_line`: the end line, the bar, no "Applied", no "Decision made"; replacing
  the table at `tests/chat.rs:217` by the grace's lines.
- `temper-client-world`: `open_chat.rs`'s Close with the new words; a story in `chat.rs` where the engine refuses the
  close, the sheet saying why, and "Keep it" then returning to the chat. `temper-client-browser-world`'s `browser_pages.rs`
  press, by its new name.
- The terminal: `temper-client-shell-terminal` holds one print of the at-work sheet: the lines as text, the acts by name.

A change of meaning runs its readers' tests: `Confirm` closes on its answer now, so besides the focused tests of the
packages you change, run `temper-client-domain`, `temper-client-view`, `temper-client-shell-terminal`,
`temper-client-shell-web`, `temper-client-world` and `temper-client-gallery`, and `heavy cargo check -p
temper-client-browser-world --tests`, with its one press by `-E`. Then `cargo fmt`, `old_forms.py`, `clippy` as your text
says. Take this task's **Not yet** in `README.status.md` down to what the proof still owes, and commit and push.

## Report

As your text says, plus: what stayed of the `.terminal-dialog` block and why; each rule you wrote that the overlay
lacks; which gallery state stands for each row of the design's table; how the closing's "Decided" was dropped.
