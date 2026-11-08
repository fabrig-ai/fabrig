# tui-whole-t1: contracts

The shapes that cross this plan's tasks. Each section is binding for the
specs that cite it; a later batch adds sections, never changes one.

## 1. Printed views (`tui-views`)

`temper-client-tui`, `src/view/printed.rs`, new, the one place a view is
printed (`temper/docs/design/client/README.md`, 6.14, *A chat*, *a view*):

```rust
/// What the chat page holds for one view address, resolved at read time.
#[derive(Debug)]
pub(crate) enum Printed {
    Asking(Box<[u8]>),          // the address
    Failed(Box<[u8]>),          // the address
    Drawn(Box<w::View>),        // temper_client_protocol_wire::v1::View
}
pub(crate) fn read(address: &[u8], shown: Option<&Shown>) -> Printed;
pub(crate) fn draw(b: &mut Builder, printed: &Printed, words: &Words);
```

`Shown` is `temper_client_domain::pages::chat::Shown`. `view/chat.rs`'s
`Entry` gains one variant, `Viewed(Box<[Piece]>)`, with
`pub(crate) enum Piece { Text(Box<[u8]>), View(Printed) }`, for committed
agent text holding at least one view block; `Entry::Agent` keeps its shape
for text without one and for the words being written. A later task that
draws a chat's document (T2) calls `printed::read` and `printed::draw`.

No new TUI word, look token or theme entry: the marks are the module's
glyph constants, painted by existing tokens (`tui-signal`, none,
`tui-hint`, `tui-faint`), so the launchpad's tasks own `words.rs`,
`look.rs` and `theme.rs` alone.

## 2. The view's needs (`view-needs`)

`temper-client-view`, `src/shared/needs.rs`, new, re-exported from
`shared` (`pub use needs::{NeedAct, NeedMark, NeedPrompt, need_prompts};`
in `src/shared.rs`), for `temper/docs/design/client/README.md`, 6.14,
*Needs you* and *Walking what needs you*:

```rust
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum NeedMark { Question, Money, SignOff, Other }

#[derive(Debug)]
pub struct NeedPrompt {
    /// A card's `object`; a sign-off's `c::ObjectKey::Task(row.task)`.
    pub object: c::ObjectKey,
    pub project: Option<u32>,
    pub mark: NeedMark,
    /// As `need_line` says it.
    pub line: NeedLine,
    pub act: NeedAct,
}

#[derive(Debug)]
pub enum NeedAct {
    /// A question the person may answer, as `shared::question` builds one.
    Answer(Question),
    /// The browser's "Review on FORGE ↗" and the forge's review page.
    Review { words: Box<[u8]>, address: Box<[u8]> },
    /// Decided on the web.
    Web,
}

pub fn need_prompts(root: &Domain, w: &Words) -> Box<[NeedPrompt]>;
```

- **Which and in what order:** `root.pages().needs().needs(inbox,
  root.forge().approvals())` with the page's inbox, none without one, in
  that order, all of them.
- **`mark`:** `Question` where the view's `ask::read` says
  `Kind::Question`; `Money` where it says `Kind::Funds` (a task held on
  its budget); `SignOff` for `Need::Approval`; `Other` for any other.
- **`act`:** `Answer` for a card that `shared::question` would offer (the
  same checks, by one per-card function both call); `Review` for an
  approval whose `yours` is `Yours::Counts` and whose
  `pages::approval::address` is some, its words `Word::ApprovalReview`
  with the forge's kind as the browser's card says it; `Web` otherwise.
- `NeedKind`, `NeedLine`, `need_line` and `question` keep their shapes and
  meaning.

## 3. The TUI's surface (`tui-surface`)

`temper-client-tui`, every name below exported where the crate exports
its kind today (`lib.rs`).

**Events** (`src/event.rs`), `Event` gains:

```rust
Own { command: Own },
Open { address: Box<[u8]> },
Walk { object: jig_client::contract::ObjectKey, step: WalkStep },
EndWalk,

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Own { Change, Ask, Brief, Needs, Open }
impl Own {
    pub const ALL: [Own; 5] = [Own::Change, Own::Ask, Own::Brief, Own::Needs, Own::Open];
    /// "/change", "/ask", "/brief", "/needs", "/open".
    pub const fn name(self) -> &'static [u8];
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum WalkStep { Choose { words: Box<[u8]> }, Open { address: Box<[u8]> }, Next }
```

**State** (`src/state.rs`), `Told` gains `Walk { by_key: bool }`,
`Walked { object: ObjectKey }`, `EndWalk`, `Brief` and `NotOpened {
address: Option<Box<[u8]>> }`; `Tui` gains:

```rust
pub fn walk(&self) -> Option<&Walk>;          // the walk while it stands
pub const fn brief_shown(&self) -> bool;      // `/brief`'s note
pub fn not_opened(&self) -> Option<&[u8]>;    // the opener's note's address

#[derive(Debug)]
pub struct Walk { /* private */ }
impl Walk {
    pub fn shown(&self, object: &ObjectKey) -> bool; // taken, opened or passed
    pub const fn by_key(&self) -> bool;              // opened by `n`
    pub const fn taken(&self) -> bool;               // a prompt was acted on
}
```

`Walk { by_key }` starts a walk with nothing shown; `Walked` marks one
shown and sets `taken`; `EndWalk` ends it; `Went` and `Cleared` end it
and clear `/brief`'s and the opener's notes; `Typed` with a value ends
it. The walk holds at most `limits::WALK` objects (64, at least the
needs a page reads).

**Words** (`src/words.rs`), the TUI's own phrases this plan adds, all of
them, numbered from 90 in this order:

| Word | English |
|---|---|
| `KindChat` | ", a chat" |
| `KindGoal` | ", a goal" |
| `ChatNumber` | "/chats " NUMBER |
| `WalksThem` | "n walks them" |
| `MoneyNeed` | "$" |
| `SignOffNeed` | "✓" |
| `StartHead` | "Start" |
| `ChangeStart` | "one change, run as " NAME "'s way says" |
| `ChangeCommand` | "One change, run as the project's way says" |
| `BriefCommand` | "The brief, whole" |
| `NeedsCommand` | "Walk what needs you" |
| `OpenCommand` | "Open this on the web" |
| `BriefMore` | " · /brief" |
| `BoxStart` | "Ask for a change, or ask temper.  / for commands" |
| `ToWalk` | "n to walk" |
| `Next` | "Next" |
| `OpenOnWeb` | "Open it on the web" |
| `WalkDone` | "Nothing else needs you." |
| `OpenAt` | "Open " NAME " in your browser." |
| `KeyWalk` | "n walks what needs you" |

"/ask"'s words are `Word::AskTemper`; "?" and "•" are `Word::Question`
and `Word::OtherNeed`. No other task adds a word: one it lacks is a
question.

**The launchpad's rows** (`src/view/launchpad.rs`), the shapes
`tui-launchpad` fills:

```rust
pub(crate) struct LeftOff {
    pub task: u64,
    pub project: Box<[u8]>,   // the row's project's name in `me`
    pub title: Box<[u8]>,
    pub kind: LeftKind,
    pub at: Box<[u8]>,        // the view's clock of its last activity
    pub state: Box<[u8]>,     // `shared::chat_state`
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum LeftKind { Chat { number: u32 }, Goal }
```

**The view's keys and focus** (`src/view.rs`), filled in `signed()`
from the launchpad's part and the root:

```rust
pub fn walk_key(view: &View) -> Option<Event>;           // `n`: Own { command: Needs } while walkable
pub fn chat_number(view: &View, number: u32) -> Option<u64>; // `/chats N`'s chat
pub fn focus(view: &View) -> Option<&[u8]>;              // `/open`'s whole address
pub fn offered_own(root: &Domain, tui: &Tui) -> Box<[Own]>;  // in Own::ALL order
```

- *walkable:* the launchpad with a project, its box empty, no list
  standing, and `root.pages().needs().needs(..)` not empty.
- *`event()`:* the box's Enter with the selected row a goal makes
  `Event::Open { address }`, the goal's page (`Address::Task { task }`).
- *focus:* `http://` + `tui.address()` + `address::print(..)` of the
  selected row's page (`Chat { task }` or `Task { task }`), else the
  launchpad's `Home { project }`; on a chat its `Chat { task }`; none
  signed out.
- *`offered_own`:* on the launchpad with a project `Change`, `Ask`,
  `Brief` while `home::facts(..).brief()` is some, `Needs` while
  walkable but for the box, `Open`; on a chat `Open`; else none.

## 4. The launchpad's needs rows and the walk's prompt (`tui-launchpad`, `tui-walk`)

`Launchpad.needs` becomes `Box<[NeedRow]>`:

```rust
pub(crate) struct NeedRow {
    pub mark: NeedMark,       // temper_client_view::shared
    pub project: Box<[u8]>,
    pub words: Box<[u8]>,     // `line.words`
}
```

`tui-walk`'s reader, `view/foot/lists.rs`:

```rust
pub(crate) fn walk(root: &Domain, tui: &Tui, words: &Words) -> Option<Options>;
```

the first of `shared::need_prompts` that `tui.walk()` has not shown, as
6.14 has it; its items bind `Event::Walk { object, step }`, and
"Tell it something else" `Event::Aside { object }`; its escape
`Event::EndWalk`. It stands before the commands' list and the chat's
question in `read`'s order.

## 5. The host (`shell-launchpad`)

`temper-client-shell-tui`'s `host.rs` carries 6.13's table's new rows:

- `Own { Change | Ask }`: the box's words become "/change " or "/ask "
  (`Act::ChatWords`, its write count moved); `Own { Brief }`:
  `Told::Brief`; `Own { Needs }`: `Told::Walk { by_key }`, `true` from the
  `n` key (`tui::walk_key`, read before the key reader as
  `tui::arrow` is), else `false`; `Own { Open }`: `Duties.open =
  tui::focus(view)`.
- `Open { address }`: `Duties.open`; an `Opened { opened: false }` for a
  page, not the approval's tab, tells `Told::NotOpened { address }`.
- `Walk { object, step }`: `Choose` makes `needs::Act::Choose { object,
  words }`, `Open` opens as `Open`, `Next` nothing; each then
  `Told::Walked { object }`. `EndWalk`: `Told::EndWalk`.
- `Send` on the launchpad: `/change WORDS` (the root to `NewChat {
  draft: false }`, `Act::ChatWords`, `Act::StartChat`), `/ask WORDS`
  (`Act::ChatWords`, `Act::StartChat` on Home), `/chats N`
  (`tui::chat_number`, `Act::Go`), each before the root's commands; the
  walk's question set aside, `needs::Act::OpenReply`, `Act::CardWords`,
  `needs::Act::SendReply` for its object, then `Told::Walked`.
- Typing while a walk stands that `by_key` opened and nothing `taken`:
  the box's words are "n" and the typed value.
