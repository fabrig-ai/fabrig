# ui-goal-page: the run's binding shapes

What crosses tasks. A task builds to these; a name here is never another task's to change.

## When a task was made (jig)

On `jig_domain_core`'s document `Task`, the wire's `Task` and `jig_client::contract::Task`, each directly
after `last_activity`:

```rust
/// When the task was made: its record's `created_at` (jig/docs/design/client/domain/contract.md, 5.1).
pub made: Wall,
```

The wire's field is `made: u64`, as `last_activity` is, in nanoseconds. An ended task's document has it too:
it comes through the same function. It has no other reader in jig.

## The goal's page in the domain (`temper-client-domain/src/pages/goal.rs`)

A new module, `pub mod goal` in `pages.rs`. Its pure functions take documents and return data; words are the
view's.

```rust
/// A goal's direct delegates, cancelled ones left out (client/README.md, 5.8).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Counts { pub done: u32, pub total: u32 }

/// What a goal's page reads beside its task page's own. On `task::Task` as `pub goal: Option<Goal>`,
/// `Some` once `shape` is `Shape::Tracked`, read through `Pages::task()`.
#[derive(Debug)]
pub struct Goal {
    /// The newest report's parse (`report::newest`, `report::parse`), none being a normal state.
    pub report: Option<report::Report>,
    /// The goal's own task line, once a page held it.
    pub line: Option<c::TaskLine>,
    // private: handles, the walls last read at, a read out, the pages read.
}
impl Goal {
    #[must_use] pub fn counts(&self, task: &c::Task, ended: Option<&c::TaskPage>) -> Counts;
    /// `goals::state`, with `needs` as given, `report` this report's presence and this count's total.
    #[must_use] pub fn state(&self, task: &c::Task, ended: Option<&c::TaskPage>, needs: bool) -> goals::State;
}
/// `TaskQuery { project, requester, label, tracked: Some(true), order: Number, .. }` from its document.
#[must_use] pub fn own_line_query(task: &c::Task) -> c::TaskQuery;

/// The draft's path and the chat's number, each by a line's start: `Draft: ` and `Chat: T`.
#[must_use] pub fn draft_of(words: &[u8]) -> (Option<Box<[u8]>>, Option<u64>);
/// A path's last segment: "docs/design/draft/away-mode.md" is "away-mode.md".
#[must_use] pub fn file_of(path: &[u8]) -> &[u8];

/// Whether a resource's path ends in `PREFIX/<root>/g<root>`, the two numbers equal.
#[must_use] pub fn holds_branch(task: &c::Task) -> bool;

/// A delegate's words on the goal's page; `None` for a cancelled one, left out.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Line {
    Running,
    WaitsOn(u64),
    WaitsOnTasks(u32),
    WaitsOnYou,
    WaitsOnResources,
    Held,
    Failed,
    Waits,
    Landed,
}
/// `yours`: an open need of the goal's inbox is this delegate's.
#[must_use] pub fn line(chip: &c::Chip, yours: bool) -> Option<Line>;

/// What `<h1>` says with no report.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Fallback {
    Planning { draft: Option<Box<[u8]>> },   // the draft's FILE
    Stopped,
    Landed { first_line: Option<Box<[u8]>> },
    Failed { reason: Box<[u8]> },
    Tasks(Counts),
}
#[must_use] pub fn fallback(state: goals::State, task: &c::Task, counts: Counts) -> Fallback;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Landing { Landed(Option<Wall>), Stopped, Failed, Whole { total: u32 }, Each }
#[must_use] pub fn landing(state: goals::State, task: &c::Task, ended_at: Option<Wall>, counts: Counts) -> Landing;
```

- `Fallback`'s result and reason come from `delegation::facts`, as a delegation's card reads them.
- `Whole { total: 0 }` is the words with no tasks yet.
- `fallback`: a live goal with no tasks yet is `Planning`, whatever else its state says, as `goals::row`'s headline does for needs: never "0 of 0 tasks landed."
- `line`'s order: ended done, `Landed`; failed, `Failed`; cancelled, `None`; running, `Running`; `yours`,
  `WaitsOnYou`; held, `Held`; else `pages::task::row_state`: one dependency `WaitsOn`, several `WaitsOnTasks`,
  holdings `WaitsOnResources`; otherwise `Waits`. A cut dependency list is `WaitsOnTasks` by those listed.
- **The inbox runs inside the page:** once the document says `Shape::Tracked`, `pages.inbox` is
  `Inbox::new(Filter { project: Some(task.project), kind: None, goal: Some(number) }, ..)` with its own handle
  range, closed as the page leaves or the shape stops being tracked. `Pages::cards()` and `cards_mut()` give the
  inbox's cards on such a page, where they gave the task's, so `Act::Decide`, `Act::CardWords`, `Act::Again` and
  `Act::Needs` work as they do on Needs you. A goal's page reads no card of its own (5.11): `task.cards` stays
  empty on it.

## The goal's page's acts (`temper-client-domain`)

```rust
// pages.rs, Act:
Pause,    // Ask::Stop at once (`Sending::Stop`), on a tracked task
Resume,   // Ask::Release naming the hold's revision (`Sending::Resume`), in state paused
// Act::Close on a tracked task opens the confirmation for ActKind::Cancel, as today;
// Act::Stop is an agent's task's alone.

// pages/needs.rs:
pub enum Toasted {
    Answered { object: ObjectKey, label: Label },
    Later,
    Paused,
    Resumed,
    Proposed { to: c::Recipient },
    Refused { outcome: Outcome },
}
pub enum Act { /* … */ Dismiss }     // closes the toast: "Look again"
impl Session { pub(crate) fn say(&mut self, toasted: Toasted, now: Time); }  // 4.5 s, replacing the one up
```

- A goal's outcome is the toast, on its answer: `Sending::Stop` done, `Paused`; `Sending::Resume` done, `Resumed`;
  any of `Stop`, `Resume`, `Cancel`, `Message` not done, `Refused`. A done Cancel or Message says nothing: its
  state and its words show it. `page.outcome` stays an agent's task's.
- The view words `Refused` as `confirm.rs`'s `outcome_words` does, with "Look again" (`needs::Act::Dismiss`) after
  `Refusal::Moved`.
- `Sending` gains `Resume`.

## Start goal (`temper-client-domain/src/pages/start_goal.rs`)

```rust
// pages.rs, Act:
OpenStart { draft: Option<Box<[u8]>>, chat: Option<u64> },
StartDraft { words: Box<[u8]> },
StartName { words: Box<[u8]> },
StartWay { template: u32 },
StartCancel,
StartGoal,

// pages.rs, Pages: `start: Option<start_goal::StartGoal>`, and `pub const fn start(&self)`.
#[derive(Debug)]
pub struct StartGoal {
    pub project: u32,
    /// Given when it opened: the sheet names it, and draws no Draft field.
    pub given: Option<Box<[u8]>>,
    pub chat: Option<u64>,
    /// The Draft field, with no draft given.
    pub draft: Box<[u8]>,
    pub name: Box<[u8]>,
    /// The chosen way; none until `me` has a tracked template.
    pub way: Option<u32>,
    pub pending: Option<Key>,
    pub error: Option<Failure>,
    // private: whether the name was edited, what the send was
}
pub enum Failure { Refused(c::Refusal), NotOffered(jig_client::shapes::NotOffered), Outcome(jig_client::Outcome) }
/// The goal's words (client/README.md, 5.14): NAME, a blank line, "Build PATH.", "Draft: PATH", "Chat: T4".
#[must_use] pub fn words(name: &[u8], path: &[u8], chat: Option<u64>) -> Box<[u8]>;
/// The default: the project's highest-numbered tracked template.
#[must_use] pub fn default_way(me: &c::Me, project: u32) -> Option<u32>;
```

- The view's `goals::ways` keeps its name and gains the template's number, `Way { number: u32, name, line }`.
- Escape: the sheet's "Cancel" carries `skein_ui::Shortcut::ESCAPE`, as the welcome's sheet's does (skein's `ui.md`, 6.1).

- A done `Create` goes to the goal's page: the answer's created task number, as `shapes::chat`'s send reads it
  (`pages/chats.rs`).

## Names and tokens

Tokens live in `look.rs`, positional in `NAMES`, under the marker in `temper.css`:
`#app:has(> .temper-rail) .temper-goal-… { … }`. These names are the view's and the stylesheet's meeting point:

| Const | Name | The mock's |
|---|---|---|
| `GOAL` | `temper-goal` | `.col.wide` |
| `GOAL_HEAD` | `temper-goal-head` | `.goal-head`, its `h1` |
| `GOAL_META` | `temper-goal-meta` | `.meta.mono`, its `b` |
| `GOAL_BYLINE` | `temper-goal-byline` | `.byline` |
| `GOAL_ACTS` | `temper-goal-acts` | `.acts` |
| `GOAL_BTN` | `temper-goal-btn` | `.btn` (with `.small` by `GOAL_SMALL`) |
| `GOAL_SMALL` | `temper-goal-small` | `.small` |
| `GOAL_QUIET` | `temper-goal-quiet` | `.quiet` |
| `GOAL_PRIMARY` | `temper-goal-primary` | `.primary`, `.rec` |
| `GOAL_BAND` | `temper-goal-band` | `.band`, its `p` |
| `GOAL_BAND_TAG` | `temper-goal-band-tag` | `.band .tag` |
| `GOAL_CHOICES` | `temper-goal-choices` | `.choices` |
| `GOAL_TWO` | `temper-goal-two` | `.two` |
| `GOAL_REPORT` | `temper-goal-report` | `.report`, its `strong` |
| `GOAL_TITLE` | `temper-goal-title` | `.section-title` |
| `GOAL_TYPING` | `temper-goal-typing` | `.typing`, its dots |
| `GOAL_SIDE` | `temper-goal-side` | `.side` |
| `GOAL_SECTION` | `temper-goal-section` | the side's `section`, `gap: 6px` |
| `GOAL_FACTS` | `temper-goal-facts` | `.facts` |
| `GOAL_BAR` | `temper-goal-bar` | `.bar`, its `span` |
| `GOAL_BAR_LANDED` | `temper-goal-bar-landed` | `.bar .landed` |
| `GOAL_BAR_RUNNING` | `temper-goal-bar-running` | `.bar .running` |
| `GOAL_LINE` | `temper-goal-line` | `.task-line`, its last `span` |
| `GOAL_LINE_RUNNING` | `temper-goal-line-running` | `.task-line.running` |
| `GOAL_LINE_NEEDS` | `temper-goal-line-needs` | "held", `.state-needs` |
| `GOAL_LANDED_FOLD` | `temper-goal-landed-fold` | the `details`' `summary`, `.link`, 13px, `margin-top: 6px` |
| `GOAL_SENT` | `temper-goal-sent` | `.sent` |
| `GOAL_YOU` | `temper-goal-you` | `.sent .you` |
| `GOAL_COMPOSER_BOX` | `temper-goal-composer-box` | the `max-width: 720px` box |
| `GOAL_COMPOSER` | `temper-goal-composer` | `.composer` and its `textarea`, `button` |
| `GOAL_HINT` | `temper-goal-hint` | `.composer-hint` |
| `GOAL_FIELD` | `temper-goal-field` | `.field`, its `label` |
| `GOAL_TEXT` | `temper-goal-text` | `.text` |
| `SHEET` | `temper-sheet` | `dialog.sheet`, `::backdrop`, its `form`, `h2` |
| `SHEET_MUTED` | `temper-sheet-muted` | `.muted`, 14px |
| `SHEET_FIELDSET` | `temper-sheet-fieldset` | `fieldset`, `legend` |
| `SHEET_OPTION` | `temper-sheet-option` | `.option`, its `input`, `b`, `small` |
| `SHEET_LINK` | `temper-sheet-link` | the ways' `.link`, 13px, muted |
| `SHEET_FOOT` | `temper-sheet-foot` | `.sheet .foot` |
| `SHEET_ERROR` | `temper-sheet-error` | `.sheet-error` |

The state's word keeps ui-goals' `GOALS_STATE_*` tokens.
