# ui-forge-reviewer: contracts

The run's binding calls: names and signatures that cross the two tasks. The design is `temper/docs/design/domain/forge.md`, section 4 (*Who a forge's user is*), and `temper/docs/design/protocol/store.md`, section 2 (*Provider 0's subject*).

## The subject (`forge-subject` makes it)

A new module of the forge's crate, `temper/crates/temper-engine-domain-forge/src/forge_user.rs`, exported from its `lib.rs`:

```rust
/// jig's provider whose subjects are forge users.
pub const FORGE_PROVIDER: u16 = 0;

/// The key of `user` on `forge`: `FORGE_PROVIDER`, with a subject of 10 bytes,
/// the forge's number then the user's, each big-endian.
#[must_use]
pub fn user_key(forge: u16, user: u64) -> jig_domain_people::IdentityKey;

/// The forge and user a key names; `None` for another provider's key or a
/// subject of another form.
#[must_use]
pub fn key_user(key: &jig_domain_people::IdentityKey) -> Option<(u16, u64)>;
```

Nothing else in temper's production code writes or reads the form. `user_key(1, 7).subject` is exactly `[0, 1, 0, 0, 0, 0, 0, 0, 0, 7]`.

## The lookup (`forge-subject` makes it)

In `temper/crates/temper-engine-domain/src/route/forge_route.rs`, beside each other, where `human_forge_identity` stands:

```rust
/// Who a forge's user is in a project: the one lookup, by forge and user
/// (domain/forge.md, 4). The party whose key is that pair's subject, with their
/// role there, else unknown.
pub(super) fn forge_person(domain: &Domain, project: u32, forge: u16, user: u64) -> Option<(u64, people::Role)>;

/// The user a party is on a forge: the one `forge_person` answers them for, else unknown.
pub(super) fn person_forge_user(domain: &Domain, project: u32, forge: u16, person: u64) -> Option<u64>;

/// Whether a party counts as a person for approvals in a project: known on a forge
/// of its repositories, as a user who isn't that forge's retained writer
/// (domain/forge.md, 4 and 8.9).
pub(super) fn human_forge_identity(domain: &Domain, project: u32, person: u64) -> bool;
```

- `forge_person` is `domain.core.people.role_for_identity(forge::user_key(forge, user), project)`, and stays one function: the draft's later link answers first in it (`temper/docs/design/domain/forge.md`, section 20). Don't build the link, or leave a stub or a comment about it.
- `person_forge_user` reads the party's key with `key_user`, answers `None` if it names another forge, and otherwise answers the user only if `forge_person` answers this same party.
- `human_forge_identity` walks `domain.forge.readable_repositories(project)` and answers true at the first forge where `person_forge_user` answers a user for whom `is_writer(forge, user)` is false. A project with no repository answers false: eligibility is only asked of a project with a writable repository (`route/kinds.rs`, `covers_project`).
- Eligibility stays per project: per repository's forge is the draft's section 7, a later plan.

## The inventory's person (`inventory-forges` makes it)

In `temper/crates/temper-engine-domain/src/route/host_route.rs`, exported where `ProjectPerson` is (`route.rs`, `lib.rs`):

```rust
pub struct ProjectPerson {
    pub person: u64,
    pub role: u32,
    /// Empty for someone adoption seeded who has not signed in.
    pub login: Box<[u8]>,
    /// Their account on each forge of the project's repositories, each forge once,
    /// in the order the repositories first name it (domain/agent.md, 8.1).
    pub forges: Box<[PersonForge]>,
    /// The person this chat's task was requested by.
    pub you: bool,
}

/// What the lookup answers for one person on one forge.
pub struct PersonForge {
    /// The forge's host, as the inventory's repository lines name it.
    pub host: Box<[u8]>,
    /// Their user there, unknown as `None`.
    pub user: Option<u64>,
    /// That user is temper's retained writer on this forge, who never counts as a person.
    pub writer: bool,
}
```

`forge_user` and `writer` leave `ProjectPerson`. In `forge_route.rs`, beside the lookup:

```rust
/// A person's account on each forge of the project's repositories (domain/agent.md, 8.1).
pub(super) fn person_forges(domain: &Domain, project: u32, person: u64) -> Box<[PersonForge]>;
```

The line (`temper/crates/temper-engine-smith/src/settings.rs`, `project_people_rows`), after `person N login: role R` and `, requester`:

| The person | What follows |
|---|---|
| no forge row has a user, whatever the number of rows | `, forge identity unknown: cannot count for approvals`, the words of today |
| some forge row has a user | for each row in order, `, forge HOST user N`, then `, temper's own forge writer: never counts as a person` where it's the writer; or `, forge HOST unknown: cannot count for approvals there` |

So, on two forges: `person 4 ada: role 1, forge forge.example:3847 user 3, forge github.com unknown: cannot count for approvals there`. The charter's words (`temper/crates/temper-service/src/stock/configuring.txt`) stay true and don't change.
