# ui-forge-setup: the shapes that cross the tasks

These bind. The reasons are `skein/docs/design/forge-setup.md`, 4.1, 4.3, 5.2, 5.6, 5.8 and 6.4, and `skein/docs/design/forge-github.md`, section 11.

## The record (task `made-record`)

`temper-protocol-records`:

- Schema, `schema/secrets-v1.schema`, in place, at each enum's end so that no existing golden moves:

  ```
  record AppMade { value: u64 }
  enum Item { ... origin: Origin  app_made: AppMade }
  enum PastedItem { ... pem: Pem  app_made: AppMade }
  ```

- `Item::AppMade`, `code()` **12**. Code 11 stays `Pasted`'s and `parse` still refuses it: a row under 11 stays unreadable (`temper/docs/design/protocol/credentials.md`, 2.1).
- `AppMade` is never required: `access` connects a GitHub forge with or without it. A row whose bytes aren't an `AppId`'s form (8 bytes, little-endian, never 0) is `Unreadable`.
- A removal erases in code order, `AppMade` last, after `Origin`.

## The App's account (task `setup-answers`)

`skein/crates/skein-forge/src/github/document.rs`:

```rust
/// The account that owns an App (forge-github.md, section 11).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct AppOwner {
    pub id: u64,          // never 0
    pub login: Box<[u8]>, // never empty
    pub kind: Account,    // the installation's enum, User or Organization
}
pub struct App {
    pub id: u64,
    pub slug: Box<[u8]>,
    pub owner: Option<AppOwner>,
    pub permissions: Box<[Permission]>,
    pub events: Box<[Box<[u8]>]>,
}
```

| `GET /app`'s `owner` | `App::owner` |
|---|---|
| `{"id":7,"login":"ada","type":"User"}` | `Some`, `Account::User` |
| `{"id":7,"login":"acme","type":"Organization"}` | `Some`, `Account::Organization` |
| absent, or `null` | `None` |
| lacking `id`, `login` or `type`, as `{"id":9,"slug":"acme"}` | `None` |
| `id` 0, an empty `login`, or another `type`, as `"Bot"` | `None` |
| a field of the wrong JSON kind, as `"id":"7"` | `InvalidAnswer`, as anywhere |

The dialect's server half writes `"owner":{"id":..,"login":"..","type":".."}` after `slug` when `Some`, and nothing when `None`, in both `app` and `app_identity`. `Conversion`'s `owner` and `owner_id`, and its node `Node::AppOwner`, stay as they are: `GET /app`'s owner is its own node.

The fact, in `forge-github.md`, section 13, and `cases::NAMES`: `app-owner-id-login-type`.

## The plans' answers (task `setup-answers`)

`skein/crates/skein-forge-setup`:

```rust
// plan.rs
pub enum Outcome {
    // ...the others, as they are, but:
    /// The App's missing wanted access, possibly empty, and its account (5.6 and 5.8).
    Lacking { lacking: Box<[crate::github::Lacking]>, owner: Option<AppOwner> },
    /// The App a manifest made, and its account (5.2).
    Made { owner: Option<AppOwner> },
    /// The given person's user id; none when the account is gone (4.3, step 7).
    Person { id: Option<NonZeroU64> },
}
pub enum Stage { /* ... */ Person }

// item.rs
pub enum Item { /* ... */ AppMade }
pub fn app_made(bytes: &[u8]) -> Option<NonZeroU64>; // 8 bytes little-endian, never 0

// forgejo/given.rs
pub struct Given { /* ... */ pub person: Option<Box<[u8]>> } // checked by `login`
pub enum Place { /* ... */ Person }
```

- `AppOwner` is `skein_forge::github::document::AppOwner`, used by name: the kit adds no type of its own.
- `Plan::Reconcile { forge, adopted }` doesn't change: the login is a given value, on Forgejo's `Given` alone (4.1), so a GitHub forge can't be given one and temper's three `Reconcile` sites stay as they are.
- A Forgejo `Reconcile` with no login given ends `Done(Nothing)`, as today. Given one, it ends `Done(Person { id })`.
- `Convert` ends `Done(Made { owner })`. GitHub's `Reconcile` and `Paste` end `Done(Lacking { lacking, owner })`. `owner` is what that run's `GET /app` read, moved out of the `App`.
- `Convert`'s writes: `AppMade`, `AppId`, `AppSlug`, `AppKey`, `User`. `AppMade`'s bytes are `AppId`'s. `Paste` writes no `AppMade`, so `github/write.rs`'s `Write` stays the four items both share and `Convert` writes the mark before it starts one.
- The plans decide nothing from any of these: the owner's type, the mark's equality and the link are temper's, in the next plan.
