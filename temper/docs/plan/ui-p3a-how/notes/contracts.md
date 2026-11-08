# ui-p3a-how: the shapes that cross tasks

The run's binding calls. The design is `temper/docs/design/client/README.md`,
5.13, *The page* and *Its parts*, and `browser.md`, 2.5, *How work runs' look*.

## C1. `temper_client_domain::pages::how` (`how-parts`)

A new module, `pub mod how;` in `pages.rs`, beside `pub mod home;`. Pure but for
`facts`, which reads the root.

```rust
//! How work runs' parts and lanes (temper/docs/design/client/README.md, 5.13).
use jig_client::contract as c;
use skein_lib::List;
use temper_client_protocol_wire::v1 as w;

/// The project's `ways` read, as the page sees it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Read {
    Reading,
    Ended,
    /// `current`: the index of the stock way the project runs, none for its own.
    Shown { current: Option<u32> },
}

/// What How work runs' parts are decided from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Facts {
    /// `me` lists Ask temper's template (`chats::ASKING_TEMPLATE`) in the project.
    pub asking: bool,
    pub read: Read,
    /// Kind 7 read, with Goals shown.
    pub goals: bool,
    /// The goal's template's budget: the template `forge::ways::default_way`
    /// names, as `me` gives it; none with no such template or kind 7 unread.
    pub budget: Option<c::TemplateBudget>,
}

/// The heading's way.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Head {
    /// "How work runs in PROJECT.": read not answered, or its watch ended.
    Reading,
    /// The stock way at this index of the read.
    Way(u32),
    /// "A way of its own".
    Own,
}

/// The path section's state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Path {
    Reading,
    Ended,
    Shown,
}

/// Budgets' one row's figure.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Budget {
    Amount(u64),
    UpTo(u64),
}

/// What How work runs draws (README.md, 5.13, *Its parts*).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Parts {
    pub head: Head,
    /// "Ask temper to change this".
    pub ask: bool,
    pub path: Path,
    /// Budgets' row, while one is set: Goals shown and the template's budget
    /// `Amount(n)` or `AsAllowed { cap: Some(n) }`; none for `Given` or no cap.
    pub budget: Option<Budget>,
}

#[must_use]
pub fn parts(facts: &Facts) -> Parts;

/// The facts of the open project's How work runs, from `me`, the forge's kind 7
/// (`Side::ways`) and the `StockWays` page held for the address.
#[must_use]
pub fn facts(root: &crate::Domain, project: u32) -> Facts;

/// A lane of the path's table.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lane {
    You,
    Agents,
    Code,
}

/// A column of the path's table, one a stop, in the path's order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Column {
    /// `stop` indexes `path.stops()`; a person's in You, an agent's in Agents,
    /// code's in Code.
    Stop { stop: u32, lane: Lane },
    /// A branch's bar, in no lane.
    Bar { stop: u32 },
}

/// A column a stop of `path`, in its order, every stop placed once.
#[must_use]
pub fn lanes(path: &w::BranchPath) -> List<Column>;
```

Every `match` lists every variant; no `_`. `facts` reads `Read` from
`root.pages().stock_ways()`'s `state()` (`Loading` is `Reading`, `Ended` is
`Ended`, `Shown(panel)` is `Shown { current: *panel.current() }`), and is
`Reading` with no `StockWays` held.

## C2. The ways watched on How work runs (`how-parts`)

`temper-client-domain`'s `pages.rs`: `StockWays` is opened on `Settings { project
}` as on `Ask { project }` (the `went` block that makes it), and left on leaving
either. `Act::WayTab` and `Act::WayStop` are heard while the address is `Ask {
project }` or `Settings { project }` of their `project`. `Act::UseWay`,
`Act::KeepWay`, the toast and `configure_stock_ways` are unchanged. Nothing else
in `pages.rs` changes.

## C3. Style names (`how-look` adds them to `look.rs` and styles them; `how-page` sets them)

`temper-client-view`'s `tests/look.rs` holds every class `temper.css` selects to a
name of `NAMES`, so the names and their rules land together, in `how-look`: each
a `pub const` beside the ways' (`WAYS_*`), its name appended to `NAMES`.

| Constant | Class | On |
|---|---|---|
| `HOW_WIDE` | `temper-how-wide` | Settings' column (`SETTINGS_COL`) while How work runs is its tab |
| `HOW_WAY` | `temper-how-way` | the path section's head, at its right: "the way: NAME" |
| `HOW_SCROLL` | `temper-how-scroll` | the slot holding the table |
| `HOW_PATH` | `temper-how-path` | the table |
| `HOW_HEAD` | `temper-how-head` | a column header (the stop's step, or a bar's words) |
| `HOW_LANE` | `temper-how-lane` | a row header ("You", "Agents", "Code") |
| `HOW_PERSON` | `temper-how-person` | a person's stop's button |
| `HOW_AGENT` | `temper-how-agent` | an agent's stop's button |
| `HOW_CODE` | `temper-how-code` | code's stop's button |
| `HOW_BAR` | `temper-how-bar` | a bar's column header and its three cells |
| `HOW_LINE` | `temper-how-line` | the authority row's one cell |
| `HOW_ROW` | `temper-how-row` | Budgets' row |

The ways' tabs, axis, panel, line, card, act and refusal keep their `WAYS_*`
classes on How work runs (C4). The tree, inside Settings' body
(`SETTINGS_BODY`), for the path section:

```
section (SETTINGS_SECTION)
  head (SETTINGS_SECTION_HEAD): heading level 2 "The path of a change", text "the way: NAME" (HOW_WAY)
  tabs "Ways of working" (WAYS_TABS) and the axis paragraph (WAYS_AXIS)      -- C4's `tabs`
  group named the shown way's name (WAYS_PANEL)
    paragraph: its line (WAYS_SAYS), where it has one
    slot (HOW_SCROLL)
      table "The path of a change" (HOW_PATH)
        row: column header "" (HOW_HEAD), a column header a column (HOW_HEAD; a bar's HOW_BAR too)
        row: row header "You" (HOW_LANE), a cell a column; in its lane a button (HOW_PERSON)
        row: row header "Agents", cells; buttons HOW_AGENT
        row: one cell spanning every column (HOW_LINE): "authority: no merge tool above this line"
        row: row header "Code", cells; buttons HOW_CODE
      (a bar's three cells carry HOW_BAR)
    the open stop's card (WAYS_ZOOM ...)                                       -- C4's `card`
    the act (WAYS_ACTS > WAYS_USE)                                             -- C4's `act`
    the refusal (WAYS_REFUSED)                                                 -- C4's `refused`
```

A stop's button carries `State::Current` while its card is open, and binds
`Act::WayStop { project, stop }`.

## C4. `temper-client-view`'s `pages/ways.rs` (`ways-shared`)

What Ask temper's page draws of the ways today, moved whole into a module both
pages call, `pub(crate) mod ways;` in `pages.rs`:

```rust
//! A project's stock ways, drawn on Ask temper's page and How work runs
//! (temper/docs/design/client/README.md, 3.5 and 5.13).

/// The tabs control "Ways of working", a tab a way, "now" on the current one, and
/// the axis line under it.
pub(crate) fn tabs(b: &mut Builder, ways: &StockWays, w: &Words, bindings: &mut Bindings, project: u32);
/// The open stop's card: heading, `who`, "Open NAME ›" where the stop opens a
/// charter, a gate or a rule, and its facts.
pub(crate) fn card(b: &mut Builder, stop: &wire::Stop, w: &Words, bindings: &mut Bindings, project: u32);
/// The shown way's act, "Use this" or "Keep it", where `ways.act(index)` gives one.
pub(crate) fn act(b: &mut Builder, ways: &StockWays, index: u32, w: &Words, bindings: &mut Bindings, project: u32);
/// The refusal of the last act, where there is one.
pub(crate) fn refused(b: &mut Builder, ways: &StockWays, w: &Words);
```

`Bindings` is `crate::frame::Bindings`; `StockWays` is
`temper_client_domain::forge::stock_ways::StockWays`; `wire` is
`temper_client_protocol_wire::v1`. Ask temper's page draws exactly what it drew
before.

## C5. How work runs in the view (`how-page`)

`temper-client-view`'s new `pages/how.rs`, `pub(crate) mod how;`, called from
`settings::how_work_runs` and `settings::open` alone:

```rust
/// The heading and lede of How work runs (README.md, 5.13, *Its heading and lede*).
pub(crate) fn head(root: &Domain, w: &Words, project: u32) -> (Box<[u8]>, Box<[u8]>);
/// "Ask temper to change this", where `parts.ask`.
pub(crate) fn ask(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut Bindings, project: u32);
/// The section "The path of a change" (C3's tree).
pub(crate) fn path(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut Bindings, project: u32);
/// Budgets' section, where `parts.budget`.
pub(crate) fn budgets(b: &mut Builder, root: &Domain, w: &Words, project: u32);
```
