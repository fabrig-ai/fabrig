# configuring-words: the shapes that cross tasks

Binding for every task of this plan.

## 1. skein's permission answer (`permission-login`)

```rust
// skein-forge/src/api.rs
/// A forge user's permission on a repository (skein/docs/design/forge.md, 3.3).
pub struct UserPermission {
    pub user: u64,
    /// The login the forge names them by; empty where the permission is read without one.
    pub login: Box<[u8]>,
    pub permission: Permission,
}
// in Answer, in place of Permission(Permission):
Permission(UserPermission),
```

`Read::Permission { user }` is unchanged. `Collaborator` is unchanged.

The store's record, in place in `temper-protocol-store/schema/store-v11.schema` (no new version, before v0.1.0): `record PermissionValue { permission: Permission user: u64 login: <the schema's bounded bytes form> }`.

## 2. `read_permission`'s text (`permission-login`)

- With a login: `Write; forge user 3, login ada`
- With none: `Write; forge user 3`
- The words before `;` are today's: `No permission`, `Read`, `Write`, `Admin`.

`read_collaborators`' text is unchanged (`User 3` lines). Its description gains that `read_permission` names each.

## 3. A read that finds nothing (`settings-reads`)

`SettingsAnswer::Unknown` renders, still as an error, followed by the page's `end`:

```
not read: no such setting at that revision. This says nothing about the settings: read the inventory and name what it lists; project-ways and project-questions take project and no number.
```

## 4. The stock of a project's ways and questions (`settings-reads`)

```rust
// temper-engine-domain-forge, beside the forge's settings values
/// The value of a project's kind 6 or 7 with none stored
/// (temper/docs/design/protocol/configuration.md, 13.4).
pub fn project_stock(kind: u32) -> Option<settings::SizedValue>;
```

Kind 6 is `Value::Questions(Route::None)`; kind 7 is `Value::Ways` with `drafts`, `goals`, `changes` and `away` false and `way` 0; any other kind is `None`.

## 5. The inventory's people (`project-people`)

```rust
// temper-engine-domain/src/route/host_route.rs, exported beside RepositoryIdentity
/// One person of the caller's project, for an authorized inventory
/// (temper/docs/design/domain/agent.md, 8.1).
pub struct ProjectPerson {
    pub person: u64,
    pub role: u32,
    /// Empty for someone adoption seeded who has not signed in.
    pub login: Box<[u8]>,
    /// Provider 0's subject, where their identity is the forge's.
    pub forge_user: Option<u64>,
    /// temper's retained forge writer, who never counts as a person.
    pub writer: bool,
    /// The person this chat's task was requested by.
    pub you: bool,
}
/// An owned forge's person, the login made for whoever runs the box.
pub struct ForgePerson {
    pub host: Box<[u8]>,
    pub login: Box<[u8]>,
}
// HostDelivery::Settings gains:
    people: Box<[ProjectPerson]>,
    forge_people: Box<[ForgePerson]>,
```

```rust
// temper-engine-domain-forge's boundary.rs, Machine gains:
    /// `application.machine.owned.person`'s login; empty with no owned machine.
    pub owned_person: Box<[u8]>,
```

The text, after `project repositories:`, each block only when it has a row. Parts in this order, a part left out when it does not apply; a person is by their number first:

```
project people:
  person 1 owner: role 0, you
  person 4 ada: role 1, forge user 3
  person 5: role 2, forge user 7
  person 6: role 2, forge user 2, temper's own forge writer: never counts as a person
forge people:
  forge 127.0.0.1:3000 person ada
```

Only `Kind::Person` parties are listed. Both blocks are for an authorized inventory of the caller's project alone: setting, revision and history reads carry neither, as for repositories. A forge's person is listed once per host, for a readable repository's forge whose mode is `Owned` with `owned_person` not empty.
