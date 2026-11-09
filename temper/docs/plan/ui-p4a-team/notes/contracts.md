# ui-p4a-team: contracts

The run's binding calls: the shapes that cross tasks. A task builds its half
to this note and needs no other task's code but what its `needs` name. The
design they follow is `temper/docs/design/client/README.md`, 3.5, *Team*,
and `temper/docs/design/client/browser.md`, 2.3, 2.5 (*Team's look*) and
7.2.

## T1. The address (`team-surface`)

In `temper/crates/temper-client-domain/src/address.rs`, after `Rule`:

```rust
pub enum Address {
    // ... every variant today, then:
    Team { project: u32 },
}
```

| Address | Path |
|---|---|
| `Team { project }` | `/projects/<project>/team` |

- A project's page, in the project's frame, as `Changes { project }` is:
  `address_project` gives its project; the standing inbox watch is open on
  it (`pages.rs`, the `showing` match); choosing another project keeps the
  page, `Team { project: chosen }`.
- Every other exhaustive `match` on `Address` takes `Team` in `Changes`'
  arm, but the TUI's (`temper-client-tui`'s `view.rs` and
  `view/launchpad.rs`, `temper-client-shell-tui`'s `host.rs`), where it
  takes `Builders`' arm, and the view's rail (`frame.rs`), where no entry
  is current on it until `team-rail`.

## T2. Team's model (`team-surface`; bodies of `shown` and `people` are `team-reads`')

`temper/crates/temper-client-domain/src/pages/team.rs`, `pub mod team` in
`pages.rs`:

```rust
//! Team: a project's people and what waits on each
//! (temper/docs/design/client/README.md, 3.5, Team).
use jig_client::contract as c;
use skein_lib::List;

/// Other people whose waits are read, one `tasks{query}` watch each.
pub const OTHERS: u32 = 8;

/// What waits on one person in the project, by kind; each count a floor while `cut`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Waits {
    pub questions: u32,
    pub reviews: u32,
    pub proposals: u32,
    /// Escalations: a held task that reached them.
    pub held: u32,
    pub person_tasks: u32,
    pub cut: bool,
}
// an explicit `Default`, every count 0, `cut` false
impl Waits {
    /// The five counts summed, saturating.
    #[must_use]
    pub const fn total(&self) -> u32;
}

/// One person of the project as Team draws them.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Person {
    pub party: u64,
    /// The person reading.
    pub you: bool,
    /// The member's name, or its login where the name is empty.
    pub name: Box<[u8]>,
    pub role: c::HeldRole,
    /// What waits on them; `None` while unread or past `OTHERS`: no line is drawn.
    pub waits: Option<Waits>,
}

/// Team's facts: plain values its reads give.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Facts {
    /// The project's members of kind person, the reader first, then in the document's order, each once.
    pub people: List<Person>,
    /// The project's document was cut: its members may be more.
    pub cut: bool,
    /// The project's triager, as its line names it; no read gives it yet.
    pub triager: Option<Box<[u8]>>,
    /// The project's standing roles; no read gives them yet.
    pub standing_roles: Option<u32>,
}

/// What Team draws (README.md, 3.5, Team, Its parts). A person's line is
/// drawn while their `waits` is `Some`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Parts {
    /// People: a person is listed.
    pub people: bool,
    /// The triager's line: a triager is read.
    pub triager: bool,
}

/// What Team's page shows of the project.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Shown {
    /// No answer yet: the intro, and "People" with no line under it.
    Reading,
    /// The project's document answered.
    Read(Facts),
    /// A refused or failed read: "This project's people can't be read now."
    Unreadable,
}

#[must_use]
pub fn parts(facts: &Facts) -> Parts;

/// Team's facts from the project's document, the reader's party, what waits on
/// the reader, and what waits on others by party; pure.
#[must_use]
pub fn gather(project: &c::Project, you: u64, yours: Option<Waits>, others: &[(u64, Waits)]) -> Facts;

/// Team's page for the project: `Reading` until its document is read for it.
#[must_use]
pub fn shown(root: &crate::Domain, project: u32) -> Shown;

/// The project's people counted, for the rail's Team; `None` until its document is read.
#[must_use]
pub fn people(root: &crate::Domain, project: u32) -> Option<u32>;
```

`gather`'s rules, which the fuzz test holds:

- `people` holds each member of kind `Person` once, by `party`, the first
  kept; the member whose party is `you` first, then the others in the
  document's order. A `Service` is never listed.
- `name` is the member's `name.bytes`, or its `login.bytes` where the name
  is empty.
- `waits` is `yours` for the reader; for another, the `Waits` of the first
  pair in `others` naming its party, else `None`. A pair naming no listed
  person is dropped.
- `cut` is the document's `cut`; `triager` and `standing_roles` are `None`.

`parts`' rules: `people` while `facts.people` isn't empty; `triager` while
`facts.triager` is `Some`.

In `team-surface`, `shown` answers `Reading` and `people` `None` for every
project: no read is made yet. `team-reads` gives them their bodies, under
the same signatures.

## T3. Team's reads (`team-reads`)

A page state in `pages/team.rs`, held by `Pages` beside `roles` (the model
is `pages/roles.rs`):

```rust
#[derive(Debug)]
pub struct Team { /* the open project's document read, its outcome, the others' watches */ }
```

- **The project's document:** on `Team { project }`, jig's
  `Read::Project { project }` once on opening, and again on each `me`
  delivery (`Told::Changed` for `pages::me_handle()`) while none is
  pending; its outcome kept, `ReadValue::Project` or unreadable (a
  `ReadNone`, `Told::Ended`); dropped, with its handle, when the address
  leaves Team or Team's project. `team-rail` widens "on `Team`" to every
  project page, for the rail; nothing else of it changes.
- **The reader's waits,** `yours`: from the standing inbox watch
  (`needs::handle()`), each `needs` row that `shows_in(project)` and was
  not answered here, by its `group()`: `Questions`, `Proposals`,
  `Escalations` (held), `PersonTasks`; and each of the person's approvals'
  rows (`forge::approvals`) with `row.project == project`, as reviews;
  `cut` as `needs::Session::count` says it. `None` while the inbox watch
  has delivered nothing.
- **Others' waits:** once the document is read, one watch
  `Subject::Tasks { query, window: limits.client.domain.rows }` for each
  person member but the reader, in `gather`'s order, at most `OTHERS`,
  `query` naming `project: Some(project)` and `requester:
  Some(Requester::Person(party))`, every other key `None`, the order Chats'
  query uses. A delivered page counts its live lines whose `waits_on` is
  `WaitsOn::Party { whom: Whom::Person(party), kind, .. }`, one each by
  `kind` (`Question`, `Proposal`, `Escalation` held, `PersonTask`), `cut`
  while the page says there are more. A watch not yet delivered gives no
  pair. Closed on leaving; a change of the members closes and opens them.
- `shown(root, project)`: `Read(gather(..))` once the document is read
  for `project`, with `you` the `me` party's number; `Unreadable` after a
  refusal or a failed read; else `Reading`. `people(root, project)`: the
  number of `gather`'s people while the document is read for `project`.

## T4. The view (`team-page`)

`temper/crates/temper-client-view/src/pages/team.rs`, `pub(crate) mod team`
in `pages.rs`:

```rust
//! Team's page (temper/docs/design/client/README.md, 3.5, Team;
//! browser.md, 2.5, Team's look).
pub(crate) fn main(b: &mut Builder, root: &Domain, w: &Words, project: u32);
/// The main column from what Team shows; `main` calls it with
/// `team::shown(root, project)` and the project's name.
pub(crate) fn draw(b: &mut Builder, w: &Words, project_name: &[u8], shown: &team::Shown);
```

`root.rs` draws `main` for `Address::Team { project }` beside the other
project pages' branches. The view tests call `draw` with hand-made `Shown`
values, so they need no read.

## T5. The rail's count (`team-rail`)

ui-p1-rail's `pages::rail::facts` gives `members: team::people(root,
project)`; `standing_roles` stays `None`. The rail's "Team" entry and the
switcher's "Team for PROJECT" are drawn by `parts.team`, which
`rail::parts` already decides.
