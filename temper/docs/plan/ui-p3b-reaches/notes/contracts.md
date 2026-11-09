# ui-p3b-reaches: the shapes that cross tasks

The run's binding calls. The design is
`temper/docs/design/client/README.md`, 5.13, *What reaches you*, *Its
parts* and *Its rows*; this note fixes the names.

## 1. The model, `temper-client-domain`'s `pages::how`

`Facts` and `Parts` gain two fields each, after their last ones; nothing
else of them changes:

```rust
pub struct Facts {
    // ... asking, read, goals, roles, budget, as today
    /// Kind 8 read for the project: `root.forge().classes(project)` answers.
    pub classes: bool,
    /// `me` lists two projects or more.
    pub several: bool,
}

pub struct Parts {
    // ... head, ask, path, roles, budget, as today
    /// What reaches you draws its columns: `facts.classes`.
    pub reaches: bool,
    /// Its head says "each project has its own": `facts.several`.
    pub several: bool,
}
```

`facts` gathers them for any address (they need no page of their own):
`classes` is `root.forge().classes(project).is_some()`, `several` is `me`'s
`projects` holding two or more, false with no `me`.

The columns, a second pure function of the project's table:

```rust
use temper_protocol_reaches::{Class, Classes, Subject};

/// A subject's line in one of What reaches you's columns.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Line {
    pub subject: Subject,
    /// The item's place in the table, from 1; 0 for a subject with no item
    /// of its own: a movable kind or a set with none, and every fixed kind.
    pub item: u8,
    /// The line's class: its item's, `AskAway` with no item, `Wait` for a fixed kind.
    pub class: Class,
}

/// What reaches you's three columns (README.md, 5.13, *What reaches you*).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Reaches {
    /// "Agents decide, then tell you": the lines at `Decide`, in the settings order.
    pub decide: Box<[Line]>,
    /// "Asks you": the lines at `AskAway`, then those at `Ask`, each in the settings order.
    pub ask: Box<[Line]>,
    /// "Always waits for you": the fixed kinds in `Kind::ALL`'s order, item 0, `Wait`.
    pub wait: Box<[Line]>,
}

#[must_use]
pub fn reaches(classes: &Classes) -> Reaches;
```

**The settings order,** the proposal card's (`class_objects` in
`temper-client-view/src/cards/proposal.rs`):

1. the movable kinds in `Kind::ALL`'s order: each with its own item (the
   subject's item with empty words), `item` its place and `class` its
   class; with none, `item` 0 and `AskAway`;
2. then the table's other items in the table's order: every item with
   words, and every item of a set, its own included;
3. then each set of `sets`, in its order, with no item of its own: `item`
   0, `AskAway`.

On a table `Classes::check` accepts, every subject is placed exactly once
across the three columns, each movable kind by one line, each item that is
not a kind's own by one line, each set without an item of its own by one
line, each fixed kind by one line. On any other table, an item of a fixed
kind, an item at `Wait` and an item naming a set the table lacks are left
out; nothing panics.

## 2. The view, `temper-client-view`

**The section's tree,** which the gallery's expectations and the client
world's story read, in `Settings { project }`'s main, after the path's
section and before Roles':

- a `Role::Slot` styled `look::SETTINGS_SECTION` (the section opened by
  `pages::settings::section_head`), its `Role::Heading { level: 2 }` named
  "What reaches you", the heading "Limits" focuses
  (`Pages::away_focus`), then a `Role::Text` styled `look::HOW_WAY`: "on
  PROJECT", or "on PROJECT; each project has its own";
- while `Parts::reaches`, a `Role::Slot` styled `look::HOW_REACHES`
  holding, a column with a line each, a `Role::Group` styled
  `look::HOW_REACH`, named its chip's words ("Agents decide, then tell
  you", "Asks you", "Always waits for you"), holding a `Role::Text` styled
  `look::HOW_WHO_AGENT` or `look::HOW_WHO_PERSON` with those words, a
  `Role::List` of one `Role::Item` a line, named the line's words, and a
  `Role::Paragraph` styled `look::HOW_REACH_NOTE`, the small line;
- then the section's rows, as `Settings` draws them today: the
  `Role::Select` "Questions a goal can answer from its plan" and, while
  the switch is shown, the `Role::Select` "Away until".

**Its look slots,** the reserved run after `temper-how-role-more` in
`look::NAMES` (tokens 681 to 685), so no plan appends at the list's end:

| Token | Name | Constant |
|---|---|---|
| 681 | `temper-how-reaches` | `HOW_REACHES` |
| 682 | `temper-how-reach` | `HOW_REACH` |
| 683 | `temper-how-who-agent` | `HOW_WHO_AGENT` |
| 684 | `temper-how-who-person` | `HOW_WHO_PERSON` |
| 685 | `temper-how-reach-note` | `HOW_REACH_NOTE` |

**Its words,** `Word` variants after `HowRoleInstructions` (the enum, its
`ALL` list and the English table), phrase ids 5630 to 5651 in this order:

| Id | Word | English |
|---|---|---|
| 5630 | `HowReaches` | What reaches you |
| 5631 | `HowReachesOn` | on NAME |
| 5632 | `HowReachesEach` | on NAME; each project has its own |
| 5633 | `HowReachDecide` | Agents decide, then tell you |
| 5634 | `HowReachAsk` | Asks you |
| 5635 | `HowReachWait` | Always waits for you |
| 5636 | `HowReachDecideNote` | Listed in Decided for you for 24 hours, to overturn. |
| 5637 | `HowReachAwayAll` | While you’re away, they are decided with the recommended choice and listed. |
| 5638 | `HowReachAwayNone` | They ask even while you’re away. |
| 5639 | `HowReachAwayBut` | While you’re away, all but NAME are decided with the recommended choice and listed. |
| 5640 | `HowReachWaitNote` | Away or not. |
| 5641 | `HowLineWording` | wording and names inside a spec |
| 5642 | `HowLineRetry` | retrying or splitting a task |
| 5643 | `HowLineImplementation` | implementation choices a spec leaves open |
| 5644 | `HowLineTests` | test-only changes |
| 5645 | `HowLineDependency` | a new dependency |
| 5646 | `HowLineDeparture` | a departure from a landed design |
| 5647 | `HowLineSet` | a change to NAME |
| 5648 | `HowLineMoney` | money past a budget |
| 5649 | `HowLineIrreversible` | what can’t be undone: rewriting history, a force-push, deleting unlanded work |
| 5650 | `HowLineRules` | changes to this page |
| 5651 | `HowLineOther` | anything an agent can’t place |

The apostrophes are U+2019, as the page's other words write them. An item
with words is its subject's line through the existing `ReachUnder` ("LINE
under “WORDS”"); `HowReachAwayBut`'s NAME is the `AskAway` lines'
subjects as `shared::object` names them ("changes to vsr’s protocol
core"), joined by ", ".

## 3. The gallery's states

`how-reaches` and `night-how-reaches`, beside `how` to `night-how`;
`settings-away` and `settings-away-you-approve` leave. `how-reaches`'s
kind 8 is the whole story's `how.html`'s classes, in this order:

| Place | Subject | Words | Class |
|---|---|---|---|
| 1 | `wording` | | `Decide` |
| 2 | `retry` | | `Decide` |
| 3 | `implementation` | | `Decide` |
| 4 | `tests` | | `Decide` |
| 5 | set 1 | | `AskAway` |
| 6 | `dependency` | | `Ask` |
| 7 | `departure` | | `Ask` |

with set 1 named "vsr’s protocol core", its patterns `include/vsr.h`,
`src/protocol.*`, `src/transition.*` and `docs/protocol.md`. Its "Asks
you" reads "a change to vsr’s protocol core", "a new dependency", "a
departure from a landed design", then "While you’re away, all but changes
to vsr’s protocol core are decided with the recommended choice and
listed."
