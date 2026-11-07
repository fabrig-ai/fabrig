# ui-forge-identity: the shapes that cross the tasks

These bind. The reasons are `temper/docs/design/draft/forge-identity.md`, sections 3, 4, 7, 8 and 9, and the design sections each part cites.

## The link (task `link-record`)

`temper-engine-domain-forge`, `src/boundary.rs`:

```rust
/// A forge's link to a box's one person: on this forge, that party is that user
/// (protocol/forge.md, 2.1; domain/forge.md, 4.6).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Link {
    pub user: u64,   // the forge's user, never 0
    pub person: u64, // jig's party, never 0
}

pub struct ForgeRecord {
    // ...every field as it is, then, last:
    /// The forge's link to a box's person, or none (protocol/forge.md, 2.1).
    pub person: Option<Link>,
}
```

- **Stored in place,** in `temper-protocol-store`'s `schema/store-v11.schema`, `record Forge`, a last field `person: option ForgeLink`, with `record ForgeLink { user: u64 person: u64 }`, and no version bump. A decoded `ForgeLink` with either number 0 is refused, as a malformed row.
- **The connector's read:** `temper_engine_domain_forge::Domain::link(&self, forge: u16) -> Option<Link>`, `#[must_use]`: the kept forge's record's `person`, `None` for an ended or unknown forge.
- **Its inputs,** new variants of `temper_engine_domain_forge::Event`, at the enum's end:

  ```rust
  /// The deployment's sole owner, handed by the root at each start (domain/root.md, 9.1).
  Owner { party: Option<u64> },
  /// What a setup run read of the forge's person (domain/forge.md, 4.6).
  PersonOf { forge: u16, answer: PersonAnswer },
  ```

  ```rust
  #[derive(Clone, Copy, PartialEq, Eq, Debug)]
  pub enum PersonAnswer {
      /// The person's user on this forge: a Forgejo `Reconcile`'s `Person { id: Some }`, or a
      /// manifest-made App's `User` account.
      Is(u64),
      /// The person's Forgejo account is gone: a `Reconcile`'s `Person { id: None }`.
      Gone,
      /// The App isn't the box's person's: no `AppMade` equal to its `AppId`, or an account
      /// that isn't a `User`.
      NotTheirs,
  }
  ```

- **Its note,** a new `Request` variant, and the root's fact for it:

  ```rust
  // temper-engine-domain-forge
  #[derive(Clone, Copy, PartialEq, Eq, Debug)]
  pub enum LinkNote { Gone, Writer, Replaced, Cleared }
  Request::LinkNoted { forge: u16, note: LinkNote },

  // temper-engine-domain, boundary.rs
  Fact::ForgeLink { forge: u16, note: temper_engine_domain_forge::LinkNote },
  ```

  The root's `forge_route::outputs` turns the request into the fact by `diagnostics::emit`; temper-service writes it as one log line, as it writes the other facts: `forge N's person: their account is gone, the link kept` (`Gone`), `forge N's person: refused, temper's writer` (`Writer`), `forge N's person: replaced` (`Replaced`), `forge N's person: cleared` (`Cleared`).

`link-record` declares these and handles both events as nothing; `link-decide` gives them their bodies.

## The lookup the other readers use (task `link-lookup`)

`temper-engine-domain`, `src/route/forge_route.rs`, signatures as they are, meanings as `temper/docs/design/domain/forge.md`, section 4 now says:

```rust
pub(super) fn forge_person(domain: &Domain, project: u32, forge: u16, user: u64) -> Option<(u64, people::Role)>;
pub(super) fn person_forge_user(domain: &Domain, project: u32, forge: u16, person: u64) -> Option<u64>;
pub(super) fn person_forges(domain: &Domain, project: u32, person: u64) -> Box<[super::PersonForge]>;
```

and, replacing `human_forge_identity`, per forge:

```rust
/// Whether a party counts as a person for approvals on a forge of a project: known
/// there, as a user who isn't its retained writer (domain/forge.md, 4 and 8.9).
pub(super) fn forge_human(domain: &Domain, project: u32, forge: u16, person: u64) -> bool;
```

`temper-engine-domain`, `src/route/kinds.rs`, made `pub(super)`:

```rust
/// The project's members with `role` who count as people on `forge` (domain/forge.md, 8.9).
pub(super) fn eligible_people(domain: &Domain, project: u32, forge: u16, role: u32) -> u32;
```

The `approval` part's read builds on these two alone: a watcher's review could count on a change's forge when `forge_human` holds for them there, and the rule can be met when `eligible_people` reaches each approval's count.

## The approval read (tasks `approval-wire`, `approval-waits`, `approval-read`, `approval-client`, `approval-card`)

The reasons are `temper/docs/design/domain/forge.md`, 8.7 and 8.12; `temper/docs/design/client/README.md`, 3.5, 4.6 and 5.13; `temper/docs/design/client/forge.md`, 4.1.

### The subject

`temper-engine-domain-forge`, `src/views.rs`, exported from its `lib.rs`, and re-exported by the root's `boundary.rs` as `temper_engine_domain::APPROVALS_SUBJECT`, as `TRIALS_SUBJECT` is:

```rust
/// The changes waiting on a person's approval, the party's own scope (domain/forge.md, 8.12).
pub const APPROVALS_SUBJECT: u64 = (1 << 62) | (1 << 37);
```

`changes_mark` answers `[changes_subject(project), status_subject(project), APPROVALS_SUBJECT]`.

### The forge's part (task `approval-waits`)

`temper-engine-domain-forge`, a new `src/approvals.rs`, exported from `lib.rs`:

```rust
/// A live change whose merge waits on a person's approval, before the people are joined
/// (domain/forge.md, 8.12).
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ApprovalWait {
    pub task: u64,
    pub project: u32,
    pub forge: u16,
    pub owner: Box<[u8]>,
    pub name: Box<[u8]>,
    pub pull: u64,
    pub title: Box<[u8]>,
    /// When its review facts at its last head were first kept.
    pub since: skein_lib::Wall,
    /// Whether its reviews were read whole.
    pub whole: bool,
    /// The changes after it in its landing branch's order (domain/forge.md, section 9).
    pub behind: u32,
    /// Its enforcing approvals still short, in its panel's order, each with its people counted.
    pub short: Box<[crate::PanelApproval]>,
}

impl Domain {
    /// Every live change in state `First` or `Landing`, with a pull request, whose review facts
    /// are at its last head, with an approval still short and none refused; oldest `since`
    /// first, then by task. Allocates; changes nothing.
    #[must_use]
    pub fn approval_waits(&self, now: skein_lib::Wall, limits: &Limits) -> Box<[ApprovalWait]>;
}
```

and, changed in place, `Domain::keep_review_facts(&mut self, wall: skein_lib::Wall, task: u64, facts: ReviewFacts) -> bool`: the same facts again answer `false`; facts at the head already kept keep its `since`; at another head `since` is `wall`. Facts are dropped, and `[APPROVALS_SUBJECT, task]` marked, at `Event::Committed` for an entry whose effect's `write` is `Write::Merge`.

### The root's part (task `approval-read`)

`temper-engine-domain`, a new `src/route/approvals.rs` (`mod approvals;` in `route.rs`), its types re-exported from `boundary.rs`:

```rust
/// Whether the watcher's review would count for the approval a row waits on (domain/forge.md, 8.12).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum YourReview {
    Counts,
    /// The watcher doesn't count as a person on the change's forge (`forge_human`).
    NoAccount,
    /// Fewer members count as people there with the role than the approval asks (`eligible_people`).
    TooFew,
    /// The approval's role isn't the watcher's: an owner seeing the wait.
    OtherRole,
}

/// A change waiting on a person's approval, as one watcher sees it (domain/forge.md, 8.12).
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ApprovalRow {
    pub task: u64,
    pub project: u32,
    /// The outermost goal above the change's task, as `route/public.rs`'s `public_goal` finds it.
    pub goal: Option<u64>,
    pub forge: u16,
    pub owner: Box<[u8]>,
    pub name: Box<[u8]>,
    pub pull: u64,
    pub title: Box<[u8]>,
    pub since: skein_lib::Wall,
    /// The approval it waits on: 0 for the deployment's rules, else the project.
    pub scope: u32,
    pub rule: u32,
    pub role: u32,
    pub required: u32,
    pub counted: u32,
    pub behind: u32,
    pub whole: bool,
    pub yours: YourReview,
}

/// One watcher's approvals document (client/README.md, 4.6).
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Approvals {
    pub rows: Box<[ApprovalRow]>,
    /// The rows past the bound.
    pub more: u32,
}

/// The watcher's approvals (domain/forge.md, 8.12).
pub(super) fn document(domain: &Domain, env: &Env<Limits>, party: u64) -> Approvals;
/// One wait as `party` sees it, or none (domain/forge.md, 8.12).
pub(super) fn row(domain: &Domain, wait: &forge::ApprovalWait, party: u64) -> Option<ApprovalRow>;
```

`Delivery` gains `Approvals { watcher: Token, document: Box<Approvals> }`, after `Trials`. `crate::room::approval_rows(limits: &Limits) -> u32` bounds the rows: at most 32, and as many as `result_bytes.max(transcript_bytes)` holds at the largest row. `ForgeReadAnswer::ProposalItems` becomes `ProposalItems { items: Box<[(forge::settings::SettingKey, forge::settings::Edit)]>, owned_approval: bool }`.

`temper-service`, `src/family.rs`:

```rust
/// The watcher's approvals document, each row's kind and files-page address from configuration,
/// as a panel's pull request's are (client/README.md, 4.6; client/forge.md, 4.1).
#[must_use]
pub fn approvals(document: root::Approvals, forges: &crate::forges::Forges, limits: &w::Limits) -> Option<Box<[u8]>>;
```

### The wire (task `approval-wire`)

`temper-client-protocol-wire`, `schema/client-wire-v1.schema`, in place, no version bump:

```
# A change waiting on a person's review (client/README.md, 4.6; domain/forge.md, 8.12).
enum ApprovalYours { counts, no_account, too_few, other_role }
record ApprovalRow { task: u64 project: u32 goal: option u64 forge: u16 kind: Kind owner: bytes 1024 name: bytes 1024 pull: u64 address: bytes 2048 title: bytes 256 since: u64 scope: u32 rule: u32 role: u32 required: u32 counted: u32 behind: u32 whole: bool yours: ApprovalYours }
versioned record Approvals { rows: list 32 ApprovalRow more: u32 }
```

`Watch` gains `approvals`, last. `ProposalItemsAnswer` gains a last field, `owned_approval: bool`. `src/lib.rs` gains `approvals(&v1::Approvals) -> Box<[u8]>` and `decode_approvals(&[u8], &v1::Limits) -> Option<v1::Approvals>`, beside `changes_panel` and `decode_changes_panel`.

### The client (tasks `approval-client`, `approval-card`)

`temper-client-domain`, a new `src/forge/approvals.rs`:

```rust
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Yours { Counts, NoAccount, TooFew, OtherRole }

/// A change waiting on a person's review on its forge (client/README.md, 4.6; client/forge.md, 4.1).
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Row {
    pub task: u64,
    pub project: u32,
    pub goal: Option<u64>,
    pub forge: u16,
    pub kind: super::forges::Kind,
    pub owner: Box<[u8]>,
    pub name: Box<[u8]>,
    pub pull: u64,
    /// The files page on the forge's web address, empty for none.
    pub address: Box<[u8]>,
    pub title: Box<[u8]>,
    /// Of the type `pages::inbox::waiting_since` answers.
    pub since: Wall,
    pub scope: u32,
    pub rule: u32,
    pub role: u32,
    pub required: u32,
    pub counted: u32,
    pub behind: u32,
    pub whole: bool,
    pub yours: Yours,
}

/// The session's approvals document: no rows until delivered, and none once refused or ended.
pub struct Approvals { /* the implementer's */ }
impl Approvals {
    #[must_use] pub fn rows(&self) -> &[Row];
    #[must_use] pub fn more(&self) -> u32;
}
impl Side {
    #[must_use] pub fn approvals(&self) -> &approvals::Approvals;
    /// Whether the proposal's items read answered `owned_approval`; false until it has.
    #[must_use] pub fn owned_approval(&self, proposal: u64) -> bool;
}
```

`src/pages/needs.rs`:

```rust
/// One open card of Needs you: jig's inbox's, or a change waiting on a review on the forge
/// (client/README.md, 3.5).
#[derive(Clone, Copy, Debug)]
pub enum Need<'a> {
    Card(&'a super::inbox::Card),
    Approval(&'a crate::forge::approvals::Row),
}
impl Need<'_> {
    /// The card's object, or `ObjectKey::Task` of the change.
    #[must_use] pub fn object(self) -> c::ObjectKey;
}
impl Session {
    /// The open cards merged: the inbox's in its order, then the approvals' rows the inbox's
    /// filter takes, those marked later last; no rows until the inbox is known.
    #[must_use] pub fn needs<'a>(&self, inbox: &'a Inbox, approvals: &'a Approvals) -> Box<[Need<'a>]>;
    /// Changed in place: the shown one among `needs`.
    #[must_use] pub fn shown<'a>(&self, inbox: &'a Inbox, approvals: &'a Approvals) -> Option<Need<'a>>;
    /// Changed in place: the rail's count with the project's approvals; cut where either is.
    #[must_use] pub fn count(&self, client: &Client, approvals: &Approvals, project: u32) -> Count;
}
```

`Session::open(&self, inbox) -> Box<[&Card]>` stays, jig's cards alone, for its readers that need nothing more (`pages.rs`, Resume's question check).
