# ui-p2-home: contracts

The shapes that cross this plan's tasks, binding. A task builds its half
to them; a change to one is a question. Paths are under
`temper/crates/temper-client-domain/src/`. Design:
`temper/docs/design/client/README.md`, 3.5, *Home on All projects*,
*Home* (*The cards*), *Needs you* and *A page's parts*.

## 1. The address

```rust
// address.rs
pub enum Address { /* … as now … */ AllHome, /* … */ }
```

- `print(&Address::AllHome)` is `/home`; `parse` reads `/home` and
  `/home/` as `AllHome`, any query dropped (as `/chats/7?x=1` is), and the
  address printed again replaces it (`temper/docs/design/client/browser.md`, 2.3).
- It is a project page in the open project's frame, as `Inbox` is: every
  match that lists `Inbox` with the frame's pages lists `AllHome` beside
  it; the rail's "Home" is current on it; going to it opens and moves no
  project (`Pages::project` unchanged).

## 2. Home on All projects

```rust
// pages/all_home.rs
//! Home on All projects (temper/docs/design/client/README.md, 3.5, Home on All projects).
pub const ACROSS: u32 = 16;

#[derive(Debug, PartialEq, Eq)]
pub struct Facts {
    /// `me`'s projects in its order, the first `ACROSS`.
    pub projects: Box<[Project]>,
    /// Every open card across projects: the unfiltered inbox's with every `approvals` row.
    pub needs: super::home::Needs,
    /// Rows showing across the projects read, at most 7, the newest kept, oldest first.
    pub decided: Box<[Decided]>,
    /// Every row showing across the projects read.
    pub showing: u32,
    /// The newest Away spell whose `off` is after the newest `previous`.
    pub away: Option<Spell>,
}
#[derive(Debug, PartialEq, Eq)]
pub struct Project {
    pub project: u32,
    pub name: Box<[u8]>,
    /// Unknown until this entry's look on it is answered and its document delivered.
    pub status: super::home::Status,
}
#[derive(Debug, PartialEq, Eq)]
pub struct Decided {
    pub project: u32,
    pub line: super::home::DecisionLine,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Spell {
    pub since: Wall,
    pub off: Wall,
}
#[derive(Debug, PartialEq, Eq)]
pub struct Intro {
    /// The newest `previous` among the projects shown.
    pub last_looked: Option<Wall>,
    pub away: Option<Spell>,
    pub landed: Landed,
    pub needs: super::home::Needs,
    pub brief: Option<Brief>,
}
#[derive(Debug, PartialEq, Eq)]
pub enum Landed {
    /// Nothing landed on any project shown.
    Nothing,
    /// None since the last look: the newest `last_landed`.
    Last { at: Wall },
    /// One project with landings since its look.
    In { project: u32, name: Box<[u8]>, count: u64 },
    /// Two projects or more with landings since their looks.
    Across { count: u64, projects: u32 },
}
#[derive(Debug, PartialEq, Eq)]
pub struct Brief {
    /// Newest `at` first, at most four.
    pub lines: Box<[BriefLine]>,
    /// The newest joined brief's `at`.
    pub at: Wall,
}
#[derive(Debug, PartialEq, Eq)]
pub struct BriefLine {
    pub project: u32,
    pub name: Box<[u8]>,
    /// The first sentence of the brief's `lede`.
    pub sentence: Box<[u8]>,
}

/// The page's facts; with no `AllHome` page open, every project's status unknown.
#[must_use]
pub fn facts(root: &Domain, wall: Wall) -> Facts;
/// The intro's one composition.
#[must_use]
pub fn intro(facts: &Facts) -> Intro;
/// The briefs joined; none while no project shown has a brief new to the person.
#[must_use]
pub fn brief(facts: &Facts) -> Option<Brief>;
/// Up to and with the first `.`, `!` or `?` that ends the lede or is followed by a space; the whole lede with none.
#[must_use]
pub fn first_sentence(lede: &[u8]) -> &[u8];
```

- A brief is new to the person by Home's rule, `home::Facts::brief`
  (its `landed` above `seen`, or its `at` after `previous`), on that
  project's status.
- `intro`, `brief` and `first_sentence` are pure: the terminal's
  launchpad (milestone T1) calls `brief` for its framed brief.

## 3. A page's parts

```rust
// pages/parts.rs
//! A page's parts (temper/docs/design/client/README.md, 3.5, A page's parts).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct HomeFacts {
    /// Home on All projects rather than a project's Home.
    pub across: bool,
    /// The projects the person is in, as `me` lists them.
    pub projects: u32,
    /// A brief new to the person on a project shown.
    pub brief: bool,
    /// Open cards waiting.
    pub open: u32,
    /// Decided for you rows showing.
    pub showing: u32,
    /// A project's To begin shows.
    pub to_begin: bool,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct HomeParts {
    pub brief: bool,
    pub needs: bool,
    pub decided: bool,
    pub projects: bool,
    pub ask: bool,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct NeedsFacts {
    /// The inbox has answered.
    pub known: bool,
    pub open: u32,
    pub showing: u32,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct NeedsParts {
    pub list: bool,
    pub done: bool,
    pub decided: bool,
}
#[must_use]
pub const fn home(facts: &HomeFacts) -> HomeParts;
#[must_use]
pub const fn needs(facts: &NeedsFacts) -> NeedsParts;
/// Needs you's facts from the root: its inbox, the approvals, the decisions read.
#[must_use]
pub fn needs_facts(root: &Domain, wall: Wall) -> NeedsFacts;

// pages/all_home.rs and pages/home.rs
impl all_home::Facts { #[must_use] pub fn parts_facts(&self) -> parts::HomeFacts; }
impl home::Facts { #[must_use] pub fn parts_facts(&self) -> parts::HomeFacts; }
```

The rules, each part's alone:

| Part | `true` when |
|---|---|
| `HomeParts::brief` | `brief && !to_begin` |
| `HomeParts::needs` | `open > 0 && !to_begin` |
| `HomeParts::decided` | `showing > 0 && !to_begin` |
| `HomeParts::projects` | `across && projects >= 2` |
| `HomeParts::ask` | `!to_begin` |
| `NeedsParts::list` | `known && open > 0` |
| `NeedsParts::done` | `known && open == 0` |
| `NeedsParts::decided` | `showing > 0` |

## 4. Acts

```rust
// pages.rs
pub enum Act { /* … as now … */ AskWith { words: Box<[u8]> }, /* … */ }
```

- `AskWith { words }` with words opens `Address::Ask { project }` on the
  project in view (a project's Home's project; on `AllHome` the open
  project, else `me`'s first) with the words in its composer, unsent, as
  if typed there; with empty words it does nothing.
- Closed rows of the wide Needs you open with today's
  `needs::Act::Open { object }`.
- Overturn stays `away::Act::Overturn { project, number }`, accepted on
  `AllHome` and on `Inbox` addresses as on `Home { project }`, the row
  found among that page's decisions.

## 5. The view

- Styles, appended to `look.rs` and named in `NAMES`:
  `temper-need-card`, `temper-need-from`, `temper-need-acts`,
  `temper-need-more`, `temper-need-row`, `temper-need-tell`,
  `temper-needs-intro`, `temper-needs-head`, `temper-chip`,
  `temper-chip-good`, `temper-chip-signal`, `temper-projects`,
  `temper-ask-line`, `temper-decided-head`; a task adds the ones it
  draws, and no other task adds them again.
- Regions named "Needs you", "Decided for you" and "Projects", as the
  design names them; the Ask line's field is named "Ask temper".
- `temper-client-view` draws Home on All projects in
  `src/pages/all_home.rs`, from `all_home::Facts`, by a function the
  tests can call with hand-made facts:
  `pub(crate) fn draw(b: &mut Builder, facts: &all_home::Facts, w: &Words, bindings: &mut Bindings, times: &mut Times)`,
  `main` reading the facts from the root and calling it.
- Home's cards, the same on a project's Home and on All projects, are one
  function of `src/pages/home_cards.rs`:
  `pub(crate) fn cards(b: &mut Builder, root: &Domain, needs: &[needs::Need<'_>], across: bool, w: &Words, bindings: &mut Bindings, times: &mut Times)`.
