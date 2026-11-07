# model-catalogue-3: the contracts between parts

The run's binding calls. The design sections they implement are skein's `docs/design/llm.md`, 4.9, `llm-connection.md`, 4.3 and `fake-llm.md`, 3.2; smith's `protocol/agent.md`, 6.1 and `protocol/hosts.md`, 2; jig's `domain/accounts.md`, 3.7, `domain/engine.md`, 11.1, 17.11 and 17.12, `domain/hosts.md`, 5.1, `protocol/channel.md`, 5.1, 5.2 and 8, `protocol/store.md`, 4.4, `protocol/configuration.md`, 3, and `client/domain/contract.md`, 3.2 and 5.1; temper's `client/README.md`, 3.5 and `protocol/configuration.md`, 13.1. Each side builds to this note, and neither needs the other's task unless its spec's `needs` says so. Names are binding; bodies are the task's.

## 1. A list's rows (skein)

```rust
// skein-llm/src/list.rs; `pub mod list`
pub struct Listed {
    pub name: Box<[u8]>,
    pub shown: Option<Box<[u8]>>,
    pub window: Option<u32>,
    pub output: Option<u32>,
    pub efforts: Box<[Box<[u8]>]>,   // low to high
}
pub struct Limits { pub body: u32, pub rows: u32 }   // Limits::DEFAULT: 1 MiB, 256
pub struct Unreadable;
pub fn target(provider: Provider) -> &'static [u8];  // b"/v1/models", b"/v1/models?limit=1000"
pub fn parse(provider: Provider, body: &[u8], limits: &Limits) -> Result<Box<[Listed]>, Unreadable>;
```

`Provider` is `skein_llm::Provider`.

## 2. One exchange (skein)

```rust
// skein-llm-connection/src/list.rs; `pub mod list`
pub enum ListAnswer { Listed(Box<[skein_llm::list::Listed]>), Refused { status: u16 }, Unreachable, Unreadable, Deadline }
pub struct Exchange;
impl Exchange {
    pub fn new(token: Token, endpoint: &Endpoint, provider: Provider, credential: Option<Credential>,
               deadlines: Deadlines, limits: &ExchangeLimits) -> Exchange;
    pub fn start(&mut self, now: Time, below: &mut Queue<Lower>);
    pub fn up(&mut self, event: LowerEvent, now: Time, below: &mut Queue<Lower>);
    pub fn fire(&mut self, now: Time, below: &mut Queue<Lower>);
    pub fn next_deadline(&self) -> Option<Time>;
    pub fn cancel(&mut self, below: &mut Queue<Lower>);
    pub fn answer(&mut self) -> Option<ListAnswer>;      // once, after its socket settled
}
```

`Deadlines`, `Endpoint`, `Lower` and `LowerEvent` are the crate's own.

## 3. The fake (skein)

```rust
// skein-fake-llm-protocol's Config, by route
pub list: Option<Box<[u8]>>,     // the body `GET /v1/models` answers; none is a 404
pub list_status: Option<u16>,    // answered in its place
// skein-fake-peers: Peer::set_list(&mut self, list: Option<Box<[u8]>>, status: Option<u16>)
// Observation::List { route: skein_llm::Provider, credential: <as Observation::Credential> }
```

## 4. smith

```rust
// smith-channel's schema: records, with no kind and no place in a channel's order
record ListAsk    { endpoint: u32  credential: option Credential }     // Credential: the channel's grant value's own form
record ListRow    { name: bytes 128  shown: option bytes 64  window: option u32  output: option u32  efforts: list 8 bytes 16 }
enum   ListFailed { refused: u16, unreachable, unreadable, deadline }
enum   ListAnswer { listed: list 256 ListRow, failed: ListFailed }

// smith-agent-shell/src/list.rs, as `relay.rs` is: kernel and hosted glue for one read
pub const FLAG: &str = "--list";
pub struct Process;                     // holds one Exchange, stdin's ask, stdout's answer
pub fn run_decoded(configuration, errors) -> Result<(), String>;

// smith-protocol-program/src/command.rs
impl Command { pub fn with_input(.., input: Secret, output_bytes: u32, ..) -> Self; }

// smith-host-protocol/src/program.rs's sibling, `list.rs`
pub struct ListLaunch { program, arguments, environment, root, directory }  // the agent's own, with FLAG added
pub struct ListRead;   // new, owns, from_io, fire, cancel, next_deadline, answer as ProgramSource
impl ListRead { pub fn new(io_owner, launch: ListLaunch, endpoint: u32, credential: Credential, deadline: Time, below) -> Result<Self, program::Error>;
                pub fn answer(&mut self) -> Option<smith_host_domain::ListOutcome>; }

// smith-host-domain
pub struct ListedModel { name, shown, window, output, efforts }
pub enum ListFailure { Refused { status: u16 }, Unreachable, Unreadable, Deadline, Agent }  // Agent: it could not start, exited without an answer, or its answer doesn't read
pub enum ListOutcome { Listed(Box<[ListedModel]>), Failed(ListFailure) }
```

## 5. The worker path (jig)

```rust
// jig-domain-link/src/lib.rs
pub const LIST_ROWS: u32 = 256;
pub struct ListedModel { pub name: Box<[u8]>, pub shown: Option<Box<[u8]>>, pub window: Option<u32>, pub output: Option<u32>, pub efforts: Box<[Box<[u8]>]> }
pub enum ListFailure { Refused { status: u16 }, Unreachable, Unreadable, Deadline, Agent }
pub enum ListOutcome { Listed(Box<[ListedModel]>), Failed(ListFailure) }

// jig-domain-host's link
LinkDown::ReadList { read: u64, endpoint: u32, grant: Grant }
LinkUp::ListRead   { read: u64, outcome: ListOutcome }
// its boundary, beside Request::Source and Event::SourceAnswered
Request::ReadList  { read: u64, endpoint: u32, grant: Grant }
Event::ListRead    { read: u64, outcome: ListOutcome }
Request::CancelRead { read: u64 }

// jig-domain-core/src/vocabulary.rs
Held::ReadList { channel, read: u64, endpoint: u32, grant: Grant }

// jig-domain-fleet's boundary
Event::ReadList   { read: u64, endpoint: u32, grant: Grant }     // from the core
Event::ListRead   { channel, read: u64, outcome: ListOutcome }   // from the hub's link
Request::ReadList { channel, read: u64, endpoint: u32, grant: Grant }
Request::ListRead { read: u64, outcome: ListOutcome }
```

Worker wire, in place: `read list` is kind `0x018d`, from the responder; `list read` is `0x010a`, from the initiator. `read list`'s grant takes the credential arm alone: a form or value of another arm isn't sent, and `jig-protocol-engine-workers` answers the read `Failed(Agent)` upward in place of sending it, as it does a generation its table no longer holds. A read's number is the accounts child's, never durable: an answer to one nobody holds is dropped.

## 6. The accounts child (jig)

```rust
// jig-domain-accounts/src/records.rs
pub struct ListRecord { pub names: u64, pub read: Option<Wall>, pub tried: Option<Wall>, pub failure: Option<ReadFailure>, pub left_out: u32 }
pub enum ReadFailure { Refused { status: u16 }, Unreachable, Unreadable, Deadline, Agent, Room }
// Stored::Account gains `list: Option<ListRecord>`; none is no read yet, whether or not the account has a list

// jig-domain-accounts/src/providers.rs
pub struct ProviderList { pub keep: Box<[Box<[u8]>]>, pub drop: Box<[Box<[u8]>]> }
// Provider gains `list: Option<ProviderList>`

// jig-domain-accounts/src/boundary.rs
Change::ReadLists { provider: u32 }                                          // answered Answer::Reading { count: u32 }
Request::ReadList { account: u32, read: u64, provider: u32, grant: Grant }
Event::Listed     { read: u64, names: u64, left_out: u32, at: Wall }
Event::ListFailed { read: u64, failure: ReadFailure, at: Wall }

// jig-domain-accounts/src/domain.rs
pub enum ModelKind { Stock, List, Person }
impl Domain {
    pub fn allows(&self, account: u32, number: u32, kind: ModelKind) -> bool;   // accounts.md, 3.7
    pub fn list(&self, account: u32) -> Option<&ListRecord>;
    pub fn has_list(&self, account: u32) -> bool;
    pub fn listed_names(&self, provider: u32, except: u32) -> u64;              // the union of the other accounts' good reads
}
```

`names` is a bit per setting: bit `i` is the setting numbered `i + 1`, as `Models` has it.

## 7. Settings (jig)

```rust
// jig-domain-core/src/settings.rs
pub enum Decider { FirstStart, Release, Party(u64), List { provider: u32 } }

// jig-domain-core/src/settings/catalogue.rs
pub struct CatalogueModel { .., pub first_read: Option<Wall> }
pub const NEW_FOR: Duration;                                    // a week
pub fn matches(pattern: &[u8], name: &[u8]) -> bool;            // `*` and `?`, on the whole name
pub fn kept(name: &[u8], list: &ProviderList) -> bool;
pub struct ListChange { pub items: Box<[Item]>, pub names: u64, pub left_out: u32 }
pub fn from_list(provider: u32, list: &ProviderList, rows: &[ListedModel], others: u64, at: Wall,
                 models: &[CatalogueModel], endpoints: &[EndpointFacts]) -> ListChange;
```

`Edit::Removed` of a model with `source: List` stores `source: Person`. `from_list` withdraws by `Edit::Set` of a whole value, not `Edit::Removed`, so that this doesn't apply to it. A model with `source: List` is withdrawn only when no bit of `names | others` holds it.

## 8. `me`, the accounts document, and the act (jig's client)

```rust
// jig-client-domain/src/contract/llm.rs
pub struct MeModel { .., pub new_until: Option<WallTime> }     // WallTime: the form `made` takes in a task's document
// contract/accounts.rs
pub struct AccountList { pub read: Option<WallTime>, pub tried: Option<WallTime>, pub failure: Option<AccountListFailure>, pub left_out: u32 }
pub enum AccountListFailure { Refused { status: u16 }, Unreachable, Unreadable, Deadline, Agent, Room }
pub struct AccountLine { .., pub list: Option<AccountList> }   // none: this account has no list
// contract/settings.rs
pub enum RevisionDecider { .., List { provider: u32 } }
// contract's account requests, beside `CheckProgram`
Ask::ReadLists { provider: u32 }                                // answered AccountAnswer::Reading { count: u32 }
```

`has_list` says whether `AccountLine.list` is some. A deployment's accounts appear in the deployment's document, as their sources do.

## 9. Marks (jig's core)

```rust
// jig-domain-core/src/documents.rs
pub enum Touch { .., AccountLines(Option<u64>) }     // the `accounts` document and the deployment's; never a Frame
```

- **A read's answer** routes by `routing/lists.rs`, new, not through `route_account`, whose `touch_accounts` marks every Frame. It marks `Touch::AccountLines`. `me` is marked by `settings_revised` where the decision changed the current settings, and by `touch_accounts` where the account's `names` changed.
- **`Timer::Account`:** `routing/mod.rs`'s arm marks as it does unless the firing's outputs are empty or `Request::ReadList` alone, as `Timer::Fleet`'s `if !out.is_empty()` has it.
