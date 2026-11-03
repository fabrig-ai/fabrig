# forgejo-connect: the contracts

The run's binding calls for the shapes that cross tasks. A task builds its half to this note; a
change to a shape here is a question, never a local choice. Design: `temper/docs/design/protocol/
credentials.md`, 2.1 and 2.2; `protocol/forge.md`, 2.1, 2.2 and 2.6; `protocol/setup.md`,
section 4; `client/README.md`, 4.6 and section 8; `client/forge.md`, 5.4; `client/tools.md`, 5.5.

## C1. The carried bundle (`temper-protocol-records`)

`schema/secrets-v1.schema`, in place at `temper-secrets` 1 (credentials.md, 2.2):

```
versioned record Pasted { items: list 3 PastedItem }
```

`PastedItem` and `Pem` stay as they are. The crate's API (`src/codec.rs`, re-exported from
`lib.rs`):

```rust
/// The carried value's aggregate bound, jig's (protocol/credentials.md, 2.2).
pub const PASTED_BYTES: usize = 4096;
/// The most one item may hold in a bundle of that item alone: PASTED_BYTES less the
/// framing of a one-item list (version, count, variant, length).
pub const PASTED_PAYLOAD: usize = PASTED_BYTES - /* the codec's framing, computed */;
/// One bundle of `(code, bytes)` items: a code of protocol/credentials.md 2.1 (1 to 10)
/// or 11 for a pasted PEM; each code at most once, each value nonempty, at most three
/// items and PASTED_BYTES in all; anything else is Unreadable.
pub fn encode_pasted(items: &[(u16, &[u8])]) -> Result<Secret, Unreadable>;
/// Each item's code and value, in the bundle's order. An empty list, a code twice, an
/// empty value, or bytes the codec refuses are PastedError::Unreadable; another version
/// PastedError::Version.
pub fn decode_pasted(bytes: &[u8]) -> Result<Box<[(u16, Secret)]>, PastedError>;
```

`PastedError` and its two messages stay. The codes a request takes are the routing's to check
(credentials.md, 2.2): `ForgejoTokens` 2 and 3, and 5 when given; a Forgejo's `Replace` 2, 3 or
both, and 5 when given; `GitHubApp` and a GitHub `Replace` 11 alone; `ForgejoAdmin` 1 alone.

## C2. The wire (`temper-client-protocol-wire`, `client-wire-v1.schema`, in place at 11)

```
enum ForgeAsk { github, owned, managed: ManagedForge, connected: ManagedForge }
record ForgejoTokens { }
enum How { github_app: GitHubApp, github_manifest: GitHubManifest, forgejo_admin: ForgejoAdmin, forgejo_tokens: ForgejoTokens }
record Replace { forge: u16 }
enum Request { ..., remove_forge: RemoveForge, start_playground, end_playground, replace: Replace }
```

New variants go last, so no existing variant's tag moves. If skein's codegen refuses an empty
record, `forgejo_tokens` is a unit variant (`How::ForgejoTokens`), and the task says so in its
report: its dependants build on what the trunk holds. No new `Outcome`, `ForgeRefusal` or
`ForgeRow` field: a `Replace` is answered as a `Connect` is (`connecting`, `connect_refused`,
`unknown`), and the client reads Replace's and the tokens' Connect's offers from a row's `kind`,
`mode` and `connection` (client/forge.md, 5.4, the actions' table).

## C3. The forge domain's surface (`temper-engine-domain-forge`, `src/boundary.rs`)

```rust
pub enum ForgeAsk {
    GitHub,
    Owned,
    Managed { place: Place, git_program: Option<Box<[u8]>> },
    /// A Forgejo of the person's connected with its bot's tokens (protocol/setup.md, section 4).
    Connected { place: Place, git_program: Option<Box<[u8]>> },
}
pub enum How {
    ForgejoAdmin { first: Option<FirstAsk> },
    GitHubApp { app: core::num::NonZeroU64, read_only: bool },
    GitHubManifest { back: Option<Box<[u8]>> },
    /// A connected Forgejo's bot's tokens, carried (protocol/setup.md, section 4).
    ForgejoTokens,
}
pub enum Ask {
    /* ...as today... */
    /// A GitHub App's new key or a connected Forgejo's new tokens, carried (client/README.md, 4.6).
    Replace { forge: u16 },
}
```

- `Ask::Replace`'s answer is `Answered::Connect(ConnectAnswer)`, as a `Connect`'s, so the root
  and the wire map it as they map a connect's.
- The forge domain's request to the root, beside `Request::Connect { forge, request, how,
  carried }`:

  ```rust
  /// Run a Replace: Check on a connected Forgejo, Paste with the kept App id on GitHub
  /// (protocol/forge.md, 2.2).
  Replace { forge: u16, request: Token, carried: jig_domain_core::Carried },
  ```

  and the root's `Request` (`temper-engine-domain/src/boundary.rs`) gains the same variant,
  forwarded as `Request::Connect` is (`route/forge_route.rs`).
- The routing's event to the forge domain, beside `Event::ConnectEnded`:

  ```rust
  /// A connected Forgejo's Check found the user its tokens answer as: the connector keeps its
  /// login as the record's bot (protocol/forge.md, 2.2).
  BotLogin { forge: u16, login: Box<[u8]> },
  ```

## C4. A Check's end, between the routing and the forge domain

On a connected Forgejo's `Check` (`skein_forge_setup` `Plan::Check { forge }`), for a `Connect`
with `How::ForgejoTokens` and for a Forgejo's `Replace` alike (protocol/forge.md, 2.2 and 2.6):

1. The run reads `ApiToken` and `GitToken` from the bundle's items first, else from the forge's
   rows (a `Replace` of one token reads the other's kept row; a `Connect` takes both pasted),
   then `User` from the forge's rows; its `Need::Write(User)`, when none is kept, is kept as any
   run's write is.
2. `Done(Outcome::Identity { id, login })`: the routing keeps each pasted item under the forge's
   family-1 number, `ApiToken`, then `GitToken`, then `WebhookSecret`, each `Kept` before the
   next, then `Origin` (Forgejo, the record's `api` address).
3. The routing sends the forge domain `Event::BotLogin { forge, login }`, then `Event::ConnectEnded
   { forge, request, connected: true, why: None, stop: None }`.
4. The forge domain, at `BotLogin` for a kept, not ended Forgejo of the connected mode: when the
   record's `bot.login` differs and the login fits `Bot.login`'s 255 bytes, it sets it,
   `forges::save`s the record and pushes `Request::ForgeKept`, as the `read_only` rewrite does
   (`connect.rs`, `send`); otherwise nothing. At `ConnectEnded` it does what it does for any
   connect's end (the carried row's erase decided, `ended` cleared).
5. The routing, at `ForgeKept`, takes the record's login into its forge's state; at the erase's
   `Erased`, builds the forge's access from its records, puts the `GitToken` value in the
   identities' table at its next generation under the record's login, then `Request::Connect`;
   the component's `Connected` goes to the root as `Connection::Connected { id: User, login: the
   record's bot login }`.
6. A `Stop` keeps nothing more: `Event::ConnectEnded { connected: false, why: Some(C5's words),
   stop: None }`, and the forge goes back to the connection it had.

## C5. A Check's stops in words (the routing's `why`, at most 256 bytes)

| `Failure` (subject, why) | `why` |
|---|---|
| `git`, `OtherUser` | `the git token answers as another user than the API token` |
| `api`, `OtherUser` | `the tokens answer as another user than LOGIN, this forge's: connect that user as a new forge` (LOGIN the record's bot login) |
| `api`, `BotIsAdmin` | `the API token is a site admin's: give the tokens of a user that is not one` |
| `api`, `Refused(_)` | `Forgejo refused the API token` |
| `git`, `Refused(_)` | `Forgejo refused the git token` |
| any other | the words the routing gives today for that `Why` on a paste's stop |

A bundle whose items are not the request's (C1's last paragraph), or one the codec refuses:
`the pasted values are not a Forgejo's tokens`, before any plan; another version:
`PastedError::Version`'s message.
