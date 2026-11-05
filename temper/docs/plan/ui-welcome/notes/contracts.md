# ui-welcome: the shapes that cross tasks

Binding for every task of this plan. A task that creates one of these builds it exactly so; a task that
reads one reads it so.

## The welcome's domain (`temper_client_domain::pages::welcome`)

`frame` creates: `Step::WelcomeAsked` (`/welcome?at=welcome`), `StepMark`, `step_marks`, `forge_kept`,
`go_on`. `go_on` is every "Continue" and "Go on setting up": Ready with a usable model, else Model.

## The welcome's parts, as `temper-` names

| The mock's | `look` const | Class | First built by |
|---|---|---|---|
| `.forge-status`, `.done` | `WELCOME_FORGE_STATUS`, with `FLOW_DONE` | `temper-welcome-forge-status` | `forge-step` |
| `.spin` | `WELCOME_SPIN` | `temper-welcome-spin` | `forge-step` |
| `.acts` | `WELCOME_ACTS` | `temper-welcome-acts` | `forge-step` |
| `.panel` | `WELCOME_PANEL` | `temper-welcome-panel` | `model-step` |
| `.away-note` | `WELCOME_AWAY` | `temper-welcome-away` | `model-step` |
| `.numbered` | `WELCOME_NUMBERED` | `temper-welcome-numbered` | `model-step` |
| `.field-v` | `WELCOME_FIELD` | `temper-welcome-field` | `model-step` |
| `.btn.quiet` for "All ways" | `FLOW_QUIET`, under `.temper-welcome-acts` | | `model-step` |
| `.facts` | `FLOW_FACTS` | as it stands | `model-step` |
| `.cmd` | `WELCOME_CMD` | `temper-welcome-cmd` | `claude-way` |
| `.checklist .current .mark` | `WELCOME_CURRENT` on the item | `temper-welcome-current` | `ready-step` |

Copy each rule's declarations from `mock.css`, with the welcome's `--gf-*` tokens, under
`/* ==== ui: welcome ==== */`.

## The set up's intent

`setup-intent` creates these in place; `sheet-choices`, `ready-copies` and `home-copies` read them.

The wire, `temper-client-protocol-wire/schema/client-wire-v1.schema`:

```
record FirstEmpty { org: bytes 1024 repository: bytes 1024 playground: bool }
record CopyAddresses { addresses: list FIRST_COPIES of bytes 512 }
enum SetUpFirst { empty: FirstEmpty, copies: CopyAddresses, nothing }
record SetUp { forge: Making first: SetUpFirst }
enum StartsWith { repository, copies: CopyAddresses, nothing }
record ForgeRow { ... first: option FirstRepository starts_with: StartsWith ... }
```

`FIRST_COPIES` is 16, in `temper-client-protocol-wire/src/lib.rs` beside `MAKINGS`, for both schemas. An address is
`source::address`'s form, `https://HOST/OWNER/NAME`, at most 512 bytes (`source::ADDRESS_BYTES`, created here).
`repository` on the row says what it says today: `first`, or none known yet (GitHub, a forge's tokens). `FirstAsk`
and `ForgejoAdmin` are unchanged.

The store, `temper-protocol-store/schema/store-v11.schema`, at 11, in place:

```
record EmptyIntent { org repository label playground }     # today's FirstIntent
record CopyIntent { addresses: list FIRST_COPIES of bytes 512 }
enum FirstIntent { empty: EmptyIntent, copies: CopyIntent, nothing }
```

The engine, `temper-engine-domain-forge`'s `boundary.rs`:

```rust
pub enum FirstIntent { Empty(EmptyIntent), Copies { addresses: Box<[Box<[u8]>]> }, Nothing }
pub enum SetUpFirst { Empty { org, repository, playground }, Copies { addresses }, Nothing }
pub enum StartsWith { Repository, Copies { addresses }, Nothing }
```

The client, `temper_client_domain::forge::forges`:

```rust
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StartsWith { Repository, Copies { addresses: Box<[Box<[u8]>]> }, Nothing }
// Row gains: pub starts_with: StartsWith,
```

## The copies, on the client

`sheet-choices` creates, in `temper_client_domain::forge::making`:

```rust
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CopiesRefused { NoAddress, NotAnAddress(Box<[u8]>), TooMany }
/// Each non-empty line as the greenfield mock's `parseCopies` reads it, then `source::parse`.
/// An address in `source::address`'s form, in the order written.
pub fn parse_copies(text: &[u8]) -> Result<List<Box<[u8]>>, CopiesRefused>;
```

`ready-copies` creates, there too:

```rust
/// The copies of the real forge numbered lowest whose `starts_with` names them, for Ready and Home.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Copies { pub forge: u16, pub addresses: Box<[Box<[u8]>]> }
pub fn copies(rows: &Forges, playground: Option<u16>) -> Option<Copies>;
/// A copy's org and name, the source's: `copy_target` of its address.
pub fn target(address: &[u8]) -> Option<(Box<[u8]>, Box<[u8]>)>;
```

Ready's keys are `welcome-copy-N`, N from 1 in the order given.

`home-copies` reads them; its `Repository` gains the states its spec names.
