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
