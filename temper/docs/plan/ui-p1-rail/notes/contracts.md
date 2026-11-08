# ui-p1-rail: contracts

The run's binding calls: the shapes that cross tasks. A task builds its half
to this note and needs no other task's code but what its `needs` name. The
design they follow is `temper/docs/design/client/README.md`, 3.5 (*Work*,
*Settings and Deployment*, *Goals*, *Changes*) and 5.3 (*The rail*, *The
rail's parts*, *Projects in the rail*, *All projects*), and
`temper/docs/design/client/browser.md`, 2.3, 2.5 and 7.2.

## C1. Addresses (`addresses`)

In `temper/crates/temper-client-domain/src/address.rs`, re-exported from the
crate's root beside `ChatsFilter` and `GoalsFilter`:

```rust
pub enum Address {
    // ... every variant today, then:
    Work { project: u32, filter: WorkFilter },
    Members,
    Limits,
}

/// Work's own filters; Goals and Changes are Work's filters at their own addresses.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WorkFilter {
    All,
    Drafts,
}
```

| Address | Path |
|---|---|
| `Work { project, filter: All }` | `/projects/<project>/work` |
| `Work { project, filter: Drafts }` | `/projects/<project>/work?show=drafts` |
| `Members` | `/deployment/members` |
| `Limits` | `/deployment/limits` |

Every other address keeps its path. `Members` and `Limits` are no project's
pages, as `Accounts` and `Forges` are; `Work` is a project's.

## C2. Work's reads (`addresses`)

Work's five addresses are `Work { project, .. }` under either filter,
`Goals { project, .. }` and `Changes { project }`. On any of them `Pages`
holds all three of `goals()`, `changes()` and `chats()` open for that
project, as `Goals`, `Changes` and `Chats { filter: All }` open each today.
A move among Work's five addresses of one project keeps the three open and
reads nothing again (a Goals filter's change sets `goals.filter`, as today).
Any other move closes them, as today. `Chats { .. }` keeps opening its chats
alone.

## C3. The rail's parts (`rail-parts`)

`temper/crates/temper-client-domain/src/pages/rail.rs`, `pub mod rail` in
`pages.rs`:

```rust
/// What the rail's parts are decided from (README.md, 5.3, The rail's parts).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[expect(clippy::struct_excessive_bools, reason = "independent facts, one per rule")]
pub struct Facts {
    pub projects: u32,               // the projects `me` lists
    pub needs: u32,                  // pages().needs().count(..).count
    pub needs_cut: bool,             // its `cut`
    pub members: Option<u32>,        // the open project's members; None: not read
    pub standing_roles: Option<u32>, // the open project's standing roles; None: not read
    pub asking: bool,                // `me` lists template 5 for the project
    pub away: bool,                  // Domain::away_shown(project, wall)
    pub drafts: bool,                // pages::drafts_offered(root, project)
    pub beginning: bool,             // pages::home::beginning(root, project, wall)
}

/// What the rail and the tab bar show.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[expect(clippy::struct_excessive_bools, reason = "one switch per part of the rail")]
pub struct Parts {
    pub all_projects: bool,  // the switcher's "All projects" row
    pub needs_you: bool,     // the "Needs you" entry
    pub count: u32,          // its count; drawn while needs_you and more than zero
    pub cut: bool,           // the count is a floor ("9+")
    pub work: bool,
    pub team: bool,
    pub ask: bool,           // "Ask temper" in the rail, the tab bar and the new box's menu
    pub how: bool,           // the hairline and "How work runs"
    pub away: bool,
    pub new: New,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum New {
    Hidden,        // To begin's cut
    DraftOrChange, // "+ New draft or change" and its menu
    Change,        // "+ New change" alone
}

#[must_use]
pub fn parts(facts: &Facts) -> Parts;
#[must_use]
pub fn facts(root: &crate::Domain, project: u32, wall: skein_lib::Wall) -> Facts;
/// Whether the page is one across projects (README.md, 5.3, All projects).
#[must_use]
pub fn across(root: &crate::Domain) -> bool;
```

The rules are 5.3's table. With `beginning`: `needs_you`, `work`, `team`,
`how` false, `new` Hidden; `ask` and `away` by their own rules;
`all_projects` by its own. `facts` gives `members` and `standing_roles` as
`None`: no read gathers them yet (README.status.md). `across` is true on
`Inbox` with no project, by `if let`, and false elsewhere. "Home", "Deployment"
and the person's menu have no switch: they always show.

## C4. Work's list (`work-list`)

`temper/crates/temper-client-domain/src/pages/work.rs`, `pub mod work` in
`pages.rs`. Plain facts in, places out; no `Domain`, no `Client`, no
settings:

```rust
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Facts {
    pub goals: List<Goal>,     // every goal row read, live and ended
    pub changes: List<Change>, // every queued row, then every landed row the document holds
    pub chats: List<Chat>,     // the person's chats read, live and closed
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Goal { pub task: u64, pub live: bool, pub activity: Wall }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Change { pub task: u64, pub at: Where, pub belongs: Belongs }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Where {
    Queued { place: u32, since: Wall }, // its queue's place from 1, its `ready_since`
    Landed { at: Wall },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Belongs { Goal(u64), Chat(u64), Nothing }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Chat { pub task: u64, pub activity: Wall, pub draft: bool }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Item { Goal(u64), Change(u64), Chat(u64), Draft(u64) }
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Card {
    pub item: Item,
    pub changes: List<u64>, // the changes on the card: on their way by place, then landed newest first
}
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Work {
    pub in_flight: List<Card>,
    pub drafts: List<Card>,
    pub landed: List<u64>, // changes landed in the seven days before `wall`, newest first
    pub filters: Filters,
}
/// What each of Work's filters holds; a filter is drawn while its count is more than zero.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Filters { pub drafts: u32, pub goals: u32, pub changes: u32, pub chats: u32 }

#[must_use]
pub fn place(facts: &Facts, wall: Wall) -> Work;
```

The rules are 3.5's *What goes where*, made exact:

- **In flight** holds a `Goal` card for each live goal; a `Chat` card for each
  chat in `facts.chats` with a queued change that belongs to it; and a
  `Change` card for each queued change that belongs to no live goal of
  `facts.goals` and to no chat of `facts.chats`. A queued change belonging to
  a live goal is on that goal's card; one belonging to a listed chat on that
  chat's card. A goal's or a chat's card also carries its landed changes.
- **Order:** newest activity first, a goal's and a chat's `activity`, a
  `Change` card's `since`; on a tie the lower task number first.
- **Drafts** holds a `Draft` card for each chat with `draft`, newest `activity`
  first, its `changes` empty.
- **Landed** holds each landed change with `wall - 7 days <= at <= wall`,
  newest first, the lower task number first on a tie.
- **Filters:** `drafts` is the drafts' count; `goals` the goals' count, live
  and ended; `changes` the queued and landed changes' count; `chats` the
  chats of `facts.chats` that any change, queued or landed, belongs to.
- A task number appears at most once among `facts`' lists of one kind; the
  function keeps the first and drops a repeat.

The view finds a card's words by its task number in the page states it reads
(`goals()`, `changes()`, `chats()`); `work::facts(root, project)`, which gathers
`Facts` from them, is `work-page`'s.

## C5. Members (`deployment`)

`temper/crates/temper-client-domain/src/pages/members.rs`: on `Members`, one
jig `Read::Project { project }` at a time for each project `me` lists, in its
order, again when `me`'s projects change; each project's outcome kept as
unread, refused, or its `Project`. `Limits` opens the same `deployment` watch
`Builders` opens, under the same rule (a role in the deployment), and its
page reads it from `Pages::builders()`.

## C6. Words and styles across the view's tasks

- **Phrase keys** (`temper-client-view/src/words.rs`, `Word::id`): each task
  takes new keys from its own range alone, so that tasks beside each other
  and the plans beside this one never meet: `rail` 4600 to 4649,
  `work-page` 4650 to 4749, `deployment` 4750 to 4799. A task adds each
  variant in the four places the file keeps it, beside its area's words
  (the rail's beside `RailNewAll`, Work's beside the Goals words, Deployment's
  beside the Builders words), never at a list's end.
- **Style tokens** (`look.rs`, `NAMES`): `addresses` appends the reserved
  names `temper-reserved-646` to `temper-reserved-675`. A task takes a
  reserved slot of its range by renaming that one line and adding its
  constant beside its area's: `rail` 646 to 649, `work-page` 650 to 671,
  `deployment` 672 to 675. A task that needs more says so under `## Left`
  and reuses an existing style.
- **`temper.css` markers:** `rail` adds `/* ==== ui: rail ==== */` before the
  switcher's marker, `work-page` adds `/* ==== ui: work ==== */` before Goals',
  and `deployment` writes under `/* ==== ui: settings ==== */`.
