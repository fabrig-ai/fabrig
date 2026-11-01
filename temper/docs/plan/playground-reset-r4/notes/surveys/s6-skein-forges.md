Read-only survey completed. Nothing built or tested. Paths below are relative to the repository root; `:N` identifies a source line. “Needed” describes deductions from the current implementation, not an existing API.

## 1. `skein-forge-connection`

### Settings and construction

`skein/crates/skein-forge-connection/src/settings.rs:8`:

```rust
pub struct Settings {
    pub forges: List<Forge>,
    pub webhooks: Option<crate::hooks::Webhooks>,
    pub word: Box<[u8]>,
    pub user_agent: Box<[u8]>,
}
```

`skein/crates/skein-forge-connection/src/settings.rs:21`:

```rust
pub struct Forge {
    pub number: u16,
    pub kind: Kind,
    pub address: Addr,
    pub transport: Transport,
    pub access: Option<Access>,
    pub log_origins: List<LogOrigin>,
}
```

Related types in the same file:

| Location | Exact shape |
|---|---|
| `:37` | `LogOrigin { origin: skein_forge::github::redirect::Origin, address: Addr }` |
| `:46` | `Access { credential: skein_forge::credential::Credential, user: u64, webhook_secret: Option<Secret> }` |
| `:57` | `Kind::{Forgejo, GitHub}` |
| `:66` | `Transport::Plaintext { name: Option<Box<[u8]>> }` |
| `:70` | `Transport::Tls { server_name: skein_tls::Name, trust: skein_tls::Config }` |

The constructor is at `skein/crates/skein-forge-connection/src/component.rs:124`:

```rust
pub fn new(
    settings: Settings,
    limits: &Limits,
    shared: &Shared,
) -> Result<Self, Unusable>
```

Its forge-related fields, at `component.rs:102`, are:

```rust
settings: Settings,
starts: List<Start>,
github: List<Option<Box<crate::installations::GitHub>>>,
forgejo_tables: Tables,
github_tables: skein_forge::github::plans::tables::Tables,
own_tokens: List<Option<Token>>,
own_plans: List<Option<OwnPlan>>,
```

Construction at `component.rs:125`:

1. Calls `limits::check`.
2. Allocates the four parallel lists with `limits.forges`.
3. Appends one entry to each for every `settings.forges` entry.
4. Chooses `Start::Unconnected` when access is absent; otherwise the dialect’s initial state.
5. Creates `GitHub::new(limits.installations)` for each GitHub forge and a diagnostic-origin queue bounded by `limits.log_origins`.
6. Reserves one extra connection slot per **configured GitHub forge**.
7. Gives each dialect’s repository/user tables zero capacity if that dialect is absent at construction.

The last two points are construction-dependent:

```rust
// component.rs:165
github_tables: ...::Tables::new(
    if has_github { limits.repositories } else { 0 },
    if has_github { limits.users } else { 0 },
),

// component.rs:176
connections: Map::with_capacity(
    limits.connections.checked_add(reserved).expect("checked connections")
),
```

### Per-forge state

`Start`, at `skein/crates/skein-forge-connection/src/start.rs:9`:

```rust
pub enum Start {
    Unconnected,
    Checking { next: Duration },
    Settings { next: Duration },
    Listing { next: Duration },
    Minting { next: Duration },
    Identity { next: Duration },
    Waiting { until: Time, next: Duration },
    Ready { cap: u32 },
    Stopped(Stopped),
}
```

`OwnPlan`, at `component.rs:36`, has:

```rust
Forgejo(skein_forge::forgejo::plans::start::Plan),
GitHubInstallations(skein_forge::github::plans::start::Installations),
GitHubMinting(skein_forge::github::plans::start::Minting),
GitHubIdentity(skein_forge::github::plans::start::Identity),
GitHubLogPage(Box<skein_forge::github::plans::start::LogPage>),
GitHubLogDelivery(Box<skein_forge::github::plans::start::LogDelivery>),
```

`GitHub`, at `skein/crates/skein-forge-connection/src/installations.rs:355`, owns:

- `jwt: Jwt`;
- `puller: crate::puller::Puller`;
- `accepted: Set<u64>`;
- `known: Map<u64, Accepted>`;
- `rows: Map<u64, Row>`;
- installation-read staging, generation and retry state;
- selected installation, minting work and held token;
- connect generation, identity-check and notification state;
- `said_origins`, bounded per forge.

Its constructor is `GitHub::new(cap: u32) -> Self`, at `installations.rs:381`.

Installation `Row`, at `installations.rs:11`, contains Full/Contents token clocks and revisions, account, accepted/suspended status, API token, permissions and expiry data.

**Existing forgetting is not forge removal.** `GitHub::forget(&mut self)`, at `installations.rs:409`, invalidates credentials/work but deliberately retains comparison metadata. `Puller::pause(&mut self)`, at `puller.rs:45`, drops current polling work while retaining `newest` and `validator`.

### Position versus number

Forge numbers are externally supplied `u16`s. No forge-number allocator was found in this crate.

**Persistent parallel-list positions:** `starts`, `github`, `own_tokens`, `own_plans` correspond exactly to the order of `settings.forges`.

| File and locations | Positional use |
|---|---|
| `component.rs:131` | Constructor appends all parallel entries together. |
| `component.rs:224` | `begin` walks positions and initializes starts. |
| `component.rs:337` | `shed` clears access/GitHub work/plans by position. |
| `component.rs:354` | `offer_pullers` walks GitHub positions. |
| `component.rs:362` | `refused_hint` matches number/source, then uses position. |
| `component.rs:383` | `installations` finds a number, then updates its positional GitHub entry. |
| `component.rs:400` | `connect` finds number → position, then changes access/start/own work. |
| `component.rs:535` | `own_closed` finds own token → position. |
| `component.rs:594` | `start_heard` updates positional settings/start/own token. |
| `component.rs:624` | `drive_own` dispatches each position by dialect. |
| `component.rs:632` | `drive_forgejo(at: u32, ...)`. |
| `component.rs:753` | `call_connections` identifies reserved GitHub own connections by position. |
| `component.rs:766` | `open_call(at: u32, ...)`. |
| `component.rs:769` | `open_call_origin(at: u32, origin: Option<u32>, ...)`. |
| `component.rs:799` | `open_own(at: u32, ...)`. |
| `component.rs:830` | Public `start(forge)` finds number → slice position. |
| `component/calls.rs:66` | `drive_calls` finds repository forge number → position. |
| `component/calls.rs:123` | `assign(..., at: u32, ...)`; reserved-slot checks at `:157`. |
| `component/calls.rs:234` | `pump_call(..., at, ...)`. |
| `component/github.rs:33` | GitHub state-machine helpers use `at: u32`; see functions below. |
| `component/github_calls.rs:83` | GitHub call-routing helpers use `at: u32`. |
| `component/github_logs.rs:26` | Log redirect follows number → forge position; `:35` separately indexes `log_origins`. |
| `component/puller.rs:28` | Delivery-log helpers use forge positions. |
| `limits.rs:378` | Enumeration position is used only to check earlier entries for duplicate numbers. |

The positional GitHub helpers are:

```text
skein/crates/skein-forge-connection/src/component/github.rs:
  33 github_token_refused       75 github_state
  96 github_phase             113 github_wait
 171 github_stop              179 github_select
 236 github_progress          333 github_identified
 359 github_minted            414 github_error
 449 github_begin             512 github_send
 609 github_heard             683 local_output_pending
 692 github_local             751 github_idle
 806 github_ready             847 github_retire_unsent
 871 drive_github

skein/crates/skein-forge-connection/src/component/github_calls.rs:
  83 github_route             159 drive_github_call
 217 github_bot_permission    231 begin_github_call
 266 pump_github_call         336 send_github_call
 375 hear_github_call         426 release_github_call
 444 github_call_step         484 github_discovered

skein/crates/skein-forge-connection/src/component/puller.rs:
  28 github_log_active         40 github_log_selected
  55 github_log_local          77 github_log_failed
  97 github_log_error         105 github_log_delivery
 121 github_logged
```

**Number-keyed state:**

- `Waiting.repository: api::Repository`: `component.rs:46`.
- `Connection.forge: u16`: `connection.rs:117`.
- `Hooked.forge: u16`: `hooks/connection.rs:57`.
- Webhook selection scans `settings.forges` for `configured.number`: `hooks/connection.rs:143`.
- Repository tables use `api::Repository`; user tables use `(u16, u64)`: see question 4.
- Installation maps use installation IDs inside the positional GitHub state.

`Connection.origin: Option<u32>` and `Route.origin: Option<u32>` are **log-origin positions**, not forge positions: `connection.rs:118`, `component/github_calls.rs:27`.

### Limits

`skein/crates/skein-forge-connection/src/limits.rs:7` defines `Limits`; relevant bounds are:

```rust
pub forges: u32,
pub connections: u32,
pub repositories: u32,
pub users: u32,
pub installations: u32,
pub log_origins: u32,
```

- Too many entries: `check_settings`, `limits.rs:375` → `Unusable::Forges { forges: u32 }`.
- Duplicate number: `limits.rs:378` → `Unusable::Duplicate { forge: u16 }`.
- `forges` is a nonzero capacity; an **empty configured list is valid**. Test: `src/tests.rs:329`, `no_configured_forge_is_valid`.
- `connections` must equal `Shared.calls`; repository capacity is checked against `Shared.repositories`.
- `Forges { forgejo: u32, github: u32 }`, at `limits.rs:87`, counts configured dialects through `Forges::of`.
- Derived capacity validation uses those counts at `limits.rs:282`; GitHub-specific validation runs only when a GitHub forge is configured, at `limits.rs:296`.

### Public API

Root reexports are at `skein/crates/skein-forge-connection/src/lib.rs:16`. Public functions/methods:

```rust
// component.rs
Component::new(settings: Settings, limits: &Limits, shared: &Shared)
    -> Result<Self, Unusable>;                                      // 124
Component::begin(&mut self, env: &Env<Limits>,
    up: &mut Queue<Event>, io: &mut Queue<skein_io::Request>);        // 217
Component::down(&mut self, env: &Env<Limits>, request: Request,
    up: &mut Queue<Event>, io: &mut Queue<skein_io::Request>);        // 232
Component::up(&mut self, env: &Env<Limits>, event: skein_io::Event,
    up: &mut Queue<Event>, io: &mut Queue<skein_io::Request>);        // 447
Component::fire(&mut self, env: &Env<Limits>,
    up: &mut Queue<Event>, io: &mut Queue<skein_io::Request>);        // 552
Component::next_deadline(&self) -> Option<Time>;                     // 573
Component::diagnostics(&self) -> Diagnostics;                       // 812
Component::reclaim(&mut self);                                      // 815
Component::forgejo_tables(&mut self) -> &mut Tables;                 // 820
Component::github_tables(&mut self)
    -> &mut skein_forge::github::plans::tables::Tables;              // 824
Component::start(&self, forge: u16) -> Option<Start>;                 // 829

// settings.rs
owned_bytes(settings: &Settings) -> Option<u64>;                     // 111

// limits.rs
Forges::of(settings: &Settings) -> Option<Forges>;                   // 94
check(l: &Limits, s: &Shared, settings: &Settings)
    -> Result<(), Unusable>;                                       // 205
page_size(listing: Listing, cap: u32, l: &Limits, s: &Shared) -> u32; // 524
documents(l: &Limits) -> documents::Limits;                          // 557
http(l: &Limits, tls: bool, io: &skein_io::Limits)
    -> skein_http::client::Limits;                                  // 596
hook_http(l: &Limits) -> skein_http::server::Limits;                  // 607
hook_documents(l: &Limits) -> documents::Limits;                     // 619

// start.rs
heard(start: Start, heard: Heard, user: u64, now: Time,
    limits: &Limits) -> Start;                                      // 32

// grow.rs
Grow::new(first: u32, cap: u32) -> Grow;                             // 25
Grow::write(&mut self, piece: &[u8]) -> Result<(), Full>;             // 30
Grow::finish(self) -> Box<[u8]>;                                    // 51
Tail::new(first: u32, max: u32) -> Self;                             // 71
Tail::write(&mut self, piece: &[u8]);                               // 75
Tail::dropped(&self) -> bool;                                      // 90
Tail::finish(self) -> Box<[u8]>;                                    // 95
```

`documents` above is the source alias for `skein_forge::forgejo::limits`.

`Request`, at `boundary.rs:59`, has exactly:

```rust
Connect { forge: u16, access: Access },
Installations { forge: u16, accepted: Box<[u64]> },
Call { call: Token, repository: api::Repository, op: api::Op },
Refused(Answered),
RefusedHint(crate::hooks::Hinted),
Returned(Answered),
Offer,
Close,
Abort,
```

The corresponding public boundary types are `Account`, `Answered`, `Change`, `Event`, `Fact`, `StartRead`, `Stopped`, `Value`; definitions are at `boundary.rs:9,16,27,43,91,117,132,147`.

Public hook data types are `Webhooks { listener: Addr }` and `Hinted { connection: Token, hint: api::Hint }`, at `hooks.rs:10,15`.

**Add/remove/disconnect API: not found.** `Connect` only supplies/replaces access for an already configured forge; an unknown number is ignored (`component.rs:399`).

### What runtime addition/removal needs

**Addition:**

- Admit and validate a new complete `Forge`, including duplicate/count, transport, origins, access and dialect-specific checks.
- Insert its settings and all four parallel state entries together.
- Ensure settings storage can accept it: temper currently constructs `Settings.forges` with capacity equal to the initial list length (`temper-service/src/forge.rs:52`).
- Supply usable tables when adding the first forge of a previously absent dialect.
- Supply the extra own-connection capacity required by a newly added GitHub forge.
- Start only the new forge. Calling existing `begin` again reinitializes existing starts and begins the listener again.

**Removal:**

- Stop admission immediately, remove access and stop start/mint/log work.
- End or suppress calls selected by `Waiting.repository.forge`.
- Abort that forge’s active and idle connections, including own connections and log-origin connections.
- Remove held/refused answers and webhook/log hints. `Held.answer: Answered` has no forge/repository field (`component.rs:96`, `boundary.rs:43`), so held-answer cleanup needs retained association.
- Drop the whole GitHub state; `forget()` alone retains metadata.
- Keep positional lists aligned, or replace the positional representation.
- Handle settling I/O and already queued events after removal without accessing a vanished position.
- Purge dialect lookup-table entries if removal must reclaim all forge-associated cached data; existing table APIs only drop one repository.

Whole-component `Close`/`Abort` and `shed` are at `component.rs:307,311,326`; they do not provide selective removal.

### Reverse dependencies

The positional-state table above lists implementation readers of the fields that registration/removal must change. Additional `Settings`/`Forge` readers and constructors are:

```text
skein/crates/skein-forge-connection/src/settings.rs:111
skein/crates/skein-forge-connection/src/connection.rs:157
skein/crates/skein-forge-connection/src/connection/tests.rs:42
skein/crates/skein-forge-connection/src/hooks.rs:134
skein/crates/skein-forge-connection/src/hooks/connection.rs:140
skein/crates/skein-forge-connection/src/limits.rs:94
skein/crates/skein-forge-connection/src/limits/tests.rs:341
skein/crates/skein-forge-connection/src/tests.rs:177
skein/crates/skein-forge-connection/src/tests/fixtures.rs:97
skein/crates/skein-forge-connection/src/tests/connection.rs:467
skein/crates/skein-forge-connection/src/tests/github_start.rs:554
skein/crates/skein-forge-connection/src/tests/github_logs.rs:32
skein/crates/skein-forge-connection/src/tests/puller.rs:75
skein/crates/skein-forge-connection/src/tests/hooks.rs:25
skein/crates/skein-forge-connection/src/tests/github_installation_loss.rs:776
skein/crates/skein-forge-connection/src/tests/github_calls.rs:753
skein/crates/skein-forge-connection/src/tests/github_files.rs:14

skein/tests/forge/machine/src/fixtures.rs:95
skein/tests/forge/machine/src/machine.rs:168
skein/tests/forge/machine/src/tls.rs:14
skein/tests/forge/machine/src/hooks.rs:49
skein/tests/forge/machine/src/joint.rs:78
skein/tests/forge/machine/src/memory.rs:211
skein/tests/forge/machine/tests/suite/machine.rs:412
skein/tests/forge/machine/tests/suite/github_start.rs:18
skein/tests/forge/machine/tests/suite/github_job_log.rs:40
skein/tests/forge/machine/tests/suite/github_installations.rs:16
skein/tests/forge/machine/tests/suite/github_unknown_repository.rs:33
skein/tests/forge/machine/tests/suite/github_joint.rs:125
skein/tests/forge/machine/tests/suite/hooks.rs:63
skein/tests/forge/machine/tests/suite/github_renewal.rs:18
skein/tests/forge/machine/tests/suite/github_installation_loss.rs:26
skein/tests/forge/machine/tests/suite/github_calls.rs:30

skein/tests/forge/protocol/src/fixtures.rs:34
skein/tests/forge/protocol/src/github_fixture.rs:24
skein/tests/forge/protocol/src/referee.rs:38
skein/tests/forge/protocol/src/world.rs:489
skein/tests/forge/protocol/src/github.rs:167
skein/tests/forge/conformance/src/component/hosted.rs:138

temper/tests/engine/protocol-forge/src/fixtures.rs:77
temper/tests/engine/protocol-forge/src/protocol.rs:317
temper/crates/temper-service/src/forge.rs:42
temper/crates/temper-service/src/setup/settings.rs:8
temper/crates/temper-service/src/startup.rs:749
temper/crates/temper-service/src/system.rs:169
temper/crates/temper-service/src/tests/system.rs:448
temper/crates/temper-service/src/tests/fixtures/forge.rs:239
temper/crates/temper-service/src/tests/forge.rs:381
temper/crates/temper-service/src/tests/setup.rs:493
temper/crates/temper-service/src/tests/setup_adoption.rs:460
```

Every direct `Component::new` caller:

```text
skein/crates/skein-forge-connection/src/component/tests.rs:9,46
skein/crates/skein-forge-connection/src/component/github.rs:1020,1137,1222,1252
skein/crates/skein-forge-connection/src/limits/tests.rs:343
skein/crates/skein-forge-connection/src/tests.rs:830
skein/crates/skein-forge-connection/src/tests/fixtures.rs:7,159
skein/crates/skein-forge-connection/src/tests/hooks.rs:28
skein/crates/skein-forge-connection/src/tests/github_calls.rs:756
skein/tests/forge/machine/src/machine.rs:191
skein/tests/forge/machine/src/hooks.rs:53
skein/tests/forge/machine/src/joint.rs:170
skein/tests/forge/machine/src/memory.rs:233
skein/tests/forge/machine/tests/suite/github_joint.rs:126
skein/tests/forge/protocol/src/world.rs:489
skein/tests/forge/protocol/src/github.rs:167
skein/tests/forge/conformance/src/component/hosted.rs:169
temper/tests/engine/protocol-forge/src/protocol.rs:317
temper/crates/temper-service/src/forge.rs:237
temper/crates/temper-service/src/tests/system.rs:452
temper/crates/temper-service/src/tests/fixtures/forge.rs:245
```

Other direct component-state/API test readers:

```text
skein/crates/skein-forge-connection/src/tests/free_rate.rs:168
skein/crates/skein-forge-connection/src/tests/github_installations.rs:298
skein/crates/skein-forge-connection/src/tests/abort.rs:60
skein/crates/skein-forge-connection/src/tests/github_unknown_repository.rs:17
skein/crates/skein-forge-connection/src/tests/github_writes_heap.rs:65
skein/crates/skein-forge-connection/src/tests/github_renewal.rs:65
skein/crates/skein-forge-connection/src/tests/reads.rs:638
skein/crates/skein-forge-connection/src/installations/tests.rs:65
skein/tests/forge/protocol/src/driver.rs:4
skein/tests/forge/machine/src/joint/github.rs:228
skein/tests/forge/machine/tests/suite/memory.rs:6
```

`Forges` counting/derived-capacity consumers additionally include:

```text
skein/crates/skein-forge-connection/src/tests.rs:307
skein/tests/forge/machine/src/memory.rs:5
skein/tests/forge/machine/tests/suite/memory.rs:8
```

Direct consumers in `smith/` and `jig/`: **not found**.

### Tests and worlds

| Package / binary | Construction coverage |
|---|---|
| `skein-forge-connection` / library tests | `src/tests.rs:171` unusable settings; `:293` duplicates; `:307` dialect counts; `:329` empty list; `:542` forge-count boundaries; `:725` dialect-dependent validation; `:829` GitHub construction; `:834` accessless GitHub. |
| Same | `src/component/tests.rs:28`, `forgejo_starts_leave_the_github_reserved_slot_free`. |
| Same | `src/tests/connection.rs:464,484,511`: unknown forge, isolation between forges, idle connection reassignment. |
| Same | `src/tests/github_start.rs:112,552,587,626`: startup, reserved connection, replacement, close during startup. |
| Same | `src/limits/tests.rs:337` onward: constructor capacity checks. |
| `skein-forge-machine-world` / `suite` | Root `skein/tests/forge/machine/tests/suite/main.rs`; construction/start coverage in `machine.rs:10`, `github_start.rs`, `access.rs`, `github_joint.rs`, `memory.rs`; worlds and fixtures listed above. |
| `skein-forge-protocol-world` / `suite` | Root `skein/tests/forge/protocol/tests/suite/main.rs`; `protocol.rs`, `github.rs`, `owner.rs`, `fake_protocol.rs`; worlds listed above. |
| `forgejo-conformance` / executable | `skein/tests/forge/conformance/src/main.rs:1`, `--component` mode; real component construction in `src/component/hosted.rs:169`. This is a recorder executable, not a `suite` test binary. |
| `temper-engine-protocol-forge-world` / `suite` | `temper/tests/engine/protocol-forge/tests/suite/protocol.rs`; component built by its protocol world at `src/protocol.rs:317`. |

Runtime forge-add/remove tests: **not found**.

## 2. `skein-forge-setup`

### Host, settings and limits

`skein/crates/skein-forge-setup/src/host.rs:24`:

```rust
pub struct Host {
    settings: Settings,
    crypto: CryptoRng,
    states: states::States,
    runs: Map<Token, Run>,
    connections: Map<Token, Connection>,
    plan_out: Queue<reconcile::Need>,
    closing_tokens: List<Token>,
    next_token: u64,
    wake: Option<Time>,
    closing: bool,
    ended: bool,
}
```

At `host.rs:38`:

```rust
pub fn new(
    limits: &Limits,
    settings: Settings,
    crypto: CryptoRng,
) -> Result<Self, Unusable>
```

It validates all settings, retains them, and allocates global run/connection/state capacities. It does **not** instantiate one persistent plan per forge.

`skein/crates/skein-forge-setup/src/host/settings.rs:7`:

```rust
pub struct Settings {
    pub forges: List<Forge>,
    pub user_agent: Box<[u8]>,
}

pub struct Forge {
    pub kind: Kind,
    pub address: skein_io::Addr,
    pub transport: Transport,
}

pub enum Kind {
    Forgejo(Given),
    GitHub(github::given::Given),
}
```

The forge number is inside the dialect’s `Given`; accessor signatures:

```rust
Forge::number(&self) -> u16;                            // settings.rs:26
Forge::forgejo(&self) -> Option<&Given>;                // :33
Forge::github(&self) -> Option<&github::given::Given>;  // :40
```

`Transport`, at `host/settings.rs:77`, has the same two shapes as the connection transport: `Plaintext { name }`, `Tls { server_name, trust }`.

`host::Limits`, at `skein/crates/skein-forge-setup/src/host/limits.rs:7`:

```rust
pub struct Limits {
    pub forges: u32,
    pub runs: u32,
    pub states: u32,
    pub token_bytes: u32,
    pub head_bytes: u32,
    pub headers: u32,
    pub body_bytes_in: u32,
    pub depth: u32,
    pub chunk: u32,
    pub string_bytes: u32,
    pub body_bytes: u32,
    pub inner: u32,
    pub target_bytes: u32,
    pub io: skein_io::Limits,
    pub tls: skein_tls::client::Limits,
}
```

`limits::check`, private to the host, starts at `host/limits.rs:145`. Count/duplicate checks are at `:175,178`. It also checks each dialect’s `Given`, page bounds and transport demands.

The design explicitly fixes settings for the host’s lifetime: `skein/docs/design/forge-setup.md:863`, section 6.5.

### Plans

`skein/crates/skein-forge-setup/src/host/boundary.rs:17`:

```rust
pub enum Plan {
    Manifest { forge: u16, target: Target },
    Convert { forge: u16, code: Secret, state: Box<[u8]> },
    Paste { forge: u16, app: NonZeroU64 },
    Install { forge: u16 },
    Accept {
        forge: u16,
        installation: NonZeroU64,
        state: Option<Box<[u8]>>,
    },
    Reconcile { forge: u16, adopted: Box<[u64]> },
    Check { forge: u16 },
    TakeOver { forge: u16, admin_token: Secret },
    Adopt { forge: u16, repository: NonZeroU64 },
    Create {
        forge: u16,
        org: Box<[u8]>,
        repository: Box<[u8]>,
        files: Box<[File]>,
    },
    Reachable {
        forge: u16,
        installation: Option<u64>,
        page: NonZeroU32,
    },
}
```

`File { path: Box<[u8]>, content: Box<[u8]> }` is at `host/boundary.rs:12`.

`Plan::forge`, `writes`, `name`, `dialect` are host-private helpers at `host/boundary.rs:45,61,77,93`.

- GitHub-only: `Manifest`, `Convert`, `Paste`, `Install`, `Accept`.
- Forgejo-only: `Check`, `TakeOver`, `Adopt`, `Create`.
- Both: `Reconcile`, `Reachable`.
- Only `Reachable` is classified as non-writing.

`Host::down` at `host.rs:65` allows one writing run and one non-writing run per forge, subject to global `runs`.

### What is per forge versus per run

**Fixed per forge:**

- endpoint and transport;
- Forgejo `Given`, at `src/given.rs:8`: forge number, bot login/email, admin/API/git item names, orgs/owner, adopted IDs, listener, marker word and plan limits;
- GitHub `Given`, at `src/github/given.rs:6`: forge number, `read_only`, word, web/deployment/callback URLs and plan limits.

**Per run:** `host/run.rs:159`:

```rust
pub(super) struct Run {
    pub bounds: Bounds,
    pub issued_state: Option<(Purpose, Box<[u8]>)>,
    pub forge: u16,
    pub writes: bool,
    pub machine: Machine,
    pub deadline: Time,
    pub pending: Option<reconcile::Need>,
    pub owner: Option<Owner>,
    pub send: Option<Send>,
    pub connection: Option<Token>,
    pub retry: bool,
    pub retry_at: Option<Time>,
    pub backoff: Duration,
    pub backoff_max: Duration,
    pub answered: bool,
}
```

`Machine`, at `host/run.rs:69`, contains the twelve concrete plan machines, including separate Forgejo/GitHub reconcile variants.

`given_for(forge: &Given, adopted: &[u64]) -> Given`, at `host/run.rs:29`, clones the fixed Forgejo `Given` and replaces `adopted` from the `Reconcile` request.

Concrete constructors:

```rust
Reconcile::new(given: &Given) -> Result<Self, Refused>;             // reconcile.rs:157
Create::new(given: &Given, org: Box<[u8]>, repository: Box<[u8]>,
    files: Box<[File]>) -> Result<Self, Refused>;                   // create.rs:54
Adopt::new(given: &Given, repository: u64) -> Result<Self, Refused>; // adopt.rs:33
Check::new(forge: u16) -> Self;                                    // check.rs:34
TakeOver::new(given: &Given, admin_token: Secret)
    -> Result<Self, Refused>;                                    // take_over.rs:41
Reachable::new(given: &Given, installation: Option<u64>, page: u32)
    -> Result<Self, Refused>;                                    // reachable.rs:68
Reachable::github(given: &github::given::Given,
    installation: u64, page: u32)
    -> Result<Self, github::given::Refused>;                      // reachable.rs:86
```

The concrete machines copy their forge identity and required values; they do not retain references to host settings. Examples: `reconcile.rs:139,163`, `create.rs:35,63`, `adopt.rs:23,38`.

**Callback state is global but forge-bound:** `host/states.rs:20`:

```rust
struct Entry {
    state: Box<[u8]>,
    forge: u16,
    purpose: Purpose,
    birth: Time,
    expiry: Time,
}
```

`States` currently has `new`, `issue`, `take`, `clear`, `prune`; no per-forge clearing operation.

**Lookup uses numbers, not persistent configuration positions.** `host.rs:421`:

```rust
fn configured(settings: &Settings, number: u16) -> &Forge {
    for forge in &settings.forges {
        if forge.number() == number {
            return forge;
        }
    }
    unreachable!("owner names a configured forge")
}
```

Thus an unknown or removed forge’s new plan currently panics. The same lookup is used when sending a run at `host.rs:356`.

### Host boundary and runtime change requirements

Public host entry points:

```rust
Host::down(&mut self, env: &Env<Limits>, request: Request,
    up: &mut Queue<Event>, io: &mut Queue<skein_io::Request>); // host.rs:54
Host::up(&mut self, env: &Env<Limits>, event: skein_io::Event,
    up: &mut Queue<Event>, io: &mut Queue<skein_io::Request>); // :166
Host::fire(&mut self, env: &Env<Limits>,
    up: &mut Queue<Event>, io: &mut Queue<skein_io::Request>); // :211
Host::next_deadline(&self) -> Option<Time>;                    // :215
```

`Request`, at `host/boundary.rs:111`:

```rust
Run { run: Token, plan: Plan },
Answer { run: Token, answer: Answer },
Close,
Abort,
```

`Event`, at `:138`:

```rust
Need { run: Token, need: Need },
Done { run: Token, outcome: Outcome },
Stop { run: Token, failure: Failure },
Closed,
```

**Add/remove API: not found.**

Needed:

- Add a checked `host::Forge` under the count/uniqueness and dialect validations currently performed only by `Host::new`.
- Make unknown/removed-forge requests reject safely.
- Remove or stop every `Run` with that `Run.forge`, abort its connections, discard pending owner needs and secret-bearing machine state.
- Clear callback states for that forge.
- Handle already emitted owner writes/answers through temper’s staging layer; removing the host’s run alone cannot retract those effects.
- Preserve settling connection accounting. A host connection stores `run: Token`, not forge number (`host/connection.rs:28`); removal must retain or establish the run → connection association.
- Existing `Close`/`Abort` operate on the whole host (`host.rs:118,154`).

Existing `Plan` and concrete-plan field shapes need not change merely to add registration/removal.

### Reverse dependencies

Direct `Host::new` callers:

```text
skein/crates/skein-forge-setup/src/host/tests.rs:61
skein/crates/skein-forge-setup/src/tests/host.rs:116
skein/crates/skein-forge-setup/src/tests/github_host.rs:31
skein/crates/skein-forge-setup/src/tests/github_states.rs:54
skein/crates/skein-forge-setup/src/tests/github_send.rs:475
skein/crates/skein-forge-setup/src/tests/github_memory.rs:335
skein/tests/forge/setup/src/host.rs:215
skein/tests/forge/setup/src/memory.rs:234
temper/crates/temper-service/src/setup.rs:439
```

`host::Settings`, `host::Forge`, host bounds and per-forge registration readers/constructors:

```text
skein/crates/skein-forge-setup/src/host.rs:38
skein/crates/skein-forge-setup/src/host/settings.rs:23
skein/crates/skein-forge-setup/src/host/limits.rs:145
skein/crates/skein-forge-setup/src/host/run.rs:35
skein/crates/skein-forge-setup/src/host/connection.rs:72
skein/crates/skein-forge-setup/src/host/connection_tests.rs:21
skein/crates/skein-forge-setup/src/host/tests.rs:59
skein/crates/skein-forge-setup/src/tests/host.rs:48,79,94
skein/crates/skein-forge-setup/src/tests/fixtures.rs:450
skein/crates/skein-forge-setup/src/tests/github_host.rs:63
skein/crates/skein-forge-setup/src/tests/github_states.rs:339
skein/crates/skein-forge-setup/src/tests/github_send.rs:460
skein/crates/skein-forge-setup/src/tests/github_memory.rs:325
skein/tests/forge/setup/src/fixtures.rs:200,220,258,267
skein/tests/forge/setup/src/host.rs:176,188,202
skein/tests/forge/setup/src/memory.rs:169,190,209
skein/tests/forge/setup/tests/suite/host.rs:389,414
skein/tests/forge/setup/tests/suite/github.rs:15
skein/tests/forge/setup/tests/suite/hooks.rs:250
temper/crates/temper-service/src/setup.rs:19,82,97
temper/crates/temper-service/src/setup/settings.rs:8,40,81,85,102
temper/crates/temper-service/src/tests/setup.rs:491,519,1035
temper/crates/temper-service/src/tests/setup_adoption.rs:252,417,463
```

`Plan` readers/constructors—the enum-dependent files if its variants change:

```text
skein/crates/skein-forge-setup/src/host.rs:65
skein/crates/skein-forge-setup/src/host/boundary.rs:44
skein/crates/skein-forge-setup/src/host/settings.rs:47
skein/crates/skein-forge-setup/src/host/run.rs:35
skein/crates/skein-forge-setup/src/given.rs:174
skein/crates/skein-forge-setup/src/tests/host.rs:177
skein/crates/skein-forge-setup/src/tests/github_host.rs:176
skein/crates/skein-forge-setup/src/tests/github_states.rs:39
skein/crates/skein-forge-setup/src/tests/github_send.rs:149
skein/crates/skein-forge-setup/src/tests/create_bounds.rs:11
skein/tests/forge/setup/src/host.rs:251
skein/tests/forge/setup/src/fixtures.rs:577
skein/tests/forge/setup/src/memory.rs:134
skein/tests/forge/setup/tests/suite/host.rs:98
skein/tests/forge/setup/tests/suite/github.rs:3
skein/tests/forge/setup/tests/suite/github_convert.rs:134
skein/tests/forge/setup/tests/suite/github_install.rs:21
skein/tests/forge/setup/tests/suite/github_paste.rs:28
skein/tests/forge/setup/tests/suite/github_reconcile.rs:192
skein/tests/forge/setup/tests/suite/fuzzy/github_setup.rs:3
temper/crates/temper-service/src/setup.rs:295
temper/crates/temper-service/src/setup/public.rs:42
temper/crates/temper-service/src/setup/routes.rs:354,451
temper/crates/temper-service/src/setup/lifecycle.rs:228
temper/crates/temper-service/src/tests/setup.rs:100
temper/crates/temper-service/src/tests/setup_scopes.rs:68
temper/crates/temper-service/src/tests/setup_adoption.rs:544
temper/crates/temper-service/src/tests/setup_admin.rs:108
temper/crates/temper-service/src/tests/setup_lifecycle.rs:132
temper/crates/temper-engine-domain-forge/src/tests.rs:1733
```

`States` and its forge-bound entries are read/constructed by:

```text
skein/crates/skein-forge-setup/src/host.rs:43,344
skein/crates/skein-forge-setup/src/host/run.rs:177,355
skein/crates/skein-forge-setup/src/host/states.rs:31
skein/crates/skein-forge-setup/src/tests/github_states.rs:36
skein/crates/skein-forge-setup/src/tests/github_memory.rs:1
```

Direct consumers in `smith/` and `jig/`: **not found**.

### Tests and worlds

| Package / binary | Relevant files and tests |
|---|---|
| `skein-forge-setup` / library tests | `src/tests/host.rs:325`, `the_host_refuses_unusable_settings_naming_each`; `:389`, empty host; `:975`, mixed dialects; `:1004`, duplicate number across dialects; `:1020`, `forges_counts_both_kinds`. |
| Same | `src/tests/host.rs:674,711,724`: parallel picker/writer runs, separate forges and global run capacity. |
| Same | `src/tests/github_states.rs:254,270,370`: restart state loss, zero state bound, state capacity across forges. |
| Same | `src/tests/github_host.rs:58,368,390`: forge-bound states, per-forge write exclusion and deadlines. |
| Same | `src/host/tests.rs:53` onward; `src/host/connection_tests.rs`; `src/tests/github_memory.rs`; `src/tests/github_send.rs`. |
| `skein-forge-setup-world` / `suite` | Root `skein/tests/forge/setup/tests/suite/main.rs`; construction in `src/host.rs:215`, `src/memory.rs:234`; fixtures in `src/fixtures.rs:200`. |
| Same | `tests/suite/host.rs:17,313,340,388`: host reconcile, picker alongside writer, admission, several runs finishing together. |
| Same | `tests/suite/github.rs`, `github_convert.rs`, `github_install.rs`, `github_paste.rs`, `github_reconcile.rs`, `heap.rs`, `hooks.rs`, `fuzzy/github_setup.rs`. |
| `temper-service` / library tests | `src/tests/setup.rs`, `setup_adoption.rs`, `setup_lifecycle.rs`, `setup_admin.rs`, `setup_scopes.rs`. |

Runtime host-forge-add/remove tests: **not found**.

## 3. `skein-forge-client`

### Config, writer and runtime writer admission

`skein/crates/skein-forge-client/src/identity.rs:37,48`:

```rust
pub struct Writer {
    pub forge: u16,
    pub author: u64,
}

pub struct Config {
    pub namespace: Box<[u8]>,
    pub writers: Box<[Writer]>,
}
```

Relevant public signatures:

```rust
// domain.rs
Domain::new(l: &Limits) -> Domain;                             // 38
Domain::seeded(l: &Limits, seed: u64) -> Domain;                // 42
Domain::configured(l: &Limits, seed: u64, config: crate::Config)
    -> Result<Domain, api::Error>;                            // 47
Domain::bind_deployment(&mut self, id: [u8; 16],
    limits: &Limits) -> bool;                                 // 55

pub fn writer(d: &mut Domain, env: &Env<Limits>,
    writer: Writer, out: &mut Queue<Request>) -> bool;         // 153

pub fn step(d: &mut Domain, env: &Env<Limits>,
    event: Event, out: &mut Queue<Request>);                  // 164
pub fn resume(d: &mut Domain, env: &Env<Limits>,
    out: &mut Queue<Request>);                               // 248
pub fn fire(d: &mut Domain, env: &Env<Limits>,
    out: &mut Queue<Request>);                               // 282
```

`writer(...)` already supports a previously absent forge:

- new writer → insert and emit `Request::Save { record: Stored::Writer(writer) }`;
- same author → return `true`, emit nothing;
- different author, author zero, unconfigured client or full writer table → `false`.

Implementation: `domain.rs:153`; admission: `identity.rs:97`.

Configuration validation at `identity.rs:53` checks:

- nonempty namespace, at most `u16::MAX` bytes;
- writer count at most `Limits.repositories`;
- namespace plus fixed writer allocation at most `Limits.op_bytes`;
- nonzero authors and unique forge numbers.

There is **no client `Limits.forges` field**. Writer-table capacity is `Limits.repositories`, including runtime gifts.

### Per-forge and global working state

`Domain`, at `domain.rs:24`:

```rust
pub(crate) config: Option<crate::Config>,
pub(crate) writers: Map<u16, u64>,
pub(crate) calls: calls::Calls,
pub(crate) alarms: Deadlines<Alarm>,
pub(crate) outbox: outbox::Outbox,
pub(crate) keep: keep::Keep,
```

Configured writers are copied into the number-keyed map at `domain.rs:76`. Runtime gifts update that map; they do not append to `Config.writers`.

There is no parallel array indexed by configured-forge position.

| State | Definition | Key / forge association |
|---|---|---|
| Authenticated writers | `domain.rs:26` | `Map<u16, u64>` |
| Outbox execution copies | `outbox.rs:25` | `entries: Map<u64, Writing>` |
| Outbox entry | `boundary.rs:107` | `Entry.repository: Repository`; number is the entry key |
| Write lane | `outbox.rs:49` | `Lane { repository: Repository, number: Option<u64> }` |
| Logical calls | `calls.rs:15` | `Call.repository: Repository` |
| Emitted calls | `calls.rs:39` | `flights: Map<Token, Id<Call>>` |
| Watched objects | `keep.rs:22` | `live: Map<Resource, Live>` |
| Repository scans | `keep.rs:24` | `repos: Map<Repository, Repo>` |
| Retry/keep deadlines | `domain.rs:14` | `ReadRetry(Id<Call>)`, `Entry(u64)`, `Resource(u64)`, `Repository(Repository)` |

`Calls` also owns four priority queues and one **client-wide** rate budget (`calls.rs:39,50`), not one budget per configured forge.

The outbox holds execution copies; durable entries belong to its parent (`outbox.rs:4`). `Entry` is:

```rust
pub struct Entry {
    pub number: u64,
    pub task: u64,
    pub repository: Repository,
    pub effect: Effect,
    pub start: Option<Position>,
    pub attempt: Option<Attempt>,
    pub failures: u32,
    pub queued: bool,
}
```

Thus outbox entries are associated with a forge through `entry.repository.forge`; there is no per-forge outbox collection.

Writer persistence types, at `boundary.rs:192,202`:

```rust
Stored::Writer(Writer)
Key::Writer(u16)
```

`Event::Writer { resource: Resource, taken: bool }`, at `boundary.rs:257`, marks a resource’s write slot. It is **not** authenticated-writer registration/removal.

### Removing a forge’s writer and flights at once

**Removal API: not found.**

Existing operations do not implement immediate removal:

- `Event::Withdraw { entry }` only withdraws before the effect has an attempt. `outbox::withdraw` returns immediately when `entry.attempt.is_some()` (`outbox.rs:248`).
- `calls::cancel_entry`, `cancel_resource`, `cancel_repository` remove queued calls only (`calls.rs:110`).
- Watch replacement marks objects inactive and retains busy state until outstanding reads settle (`keep.rs:328,351`).
- `Event::Settle` is whole-client graceful settling, not per-forge erasure.

A new removal operation needs to remove the writer, its execution entries, logical calls/flights, priority-queue references, relevant alarms and watched/scan state in one decision, with corresponding parent-store erasures.

**Late-answer handling must change or be coordinated with connection removal.** Current `calls::answered`, at `calls.rs:360`:

```rust
let id = d.calls.flights.remove(&token)
    .expect("exactly one terminal per call token");
```

Erasing flights and later delivering their old `Event::Answered` currently panics. Tokens are separately sequenced, so removal must also preserve non-reuse and define how old terminals are suppressed/ignored.

**Temper’s current writer-erasure route is incomplete for this operation:**

- Writer saves are translated to temper’s own `Stored::Writer` at  
  `temper/crates/temper-engine-domain-forge/src/domain.rs:1521`.
- Client erases are currently translated uniformly to `Key::Client(key)` at `:1530`.
- `skein_forge_client::Key::Writer` under that wrapper is forbidden by  
  `temper/crates/temper-protocol-store/src/keys.rs:298`:

```rust
skein_forge_client::Key::Writer(_) =>
    unreachable!("the connector keeps the client's writer as Stored::Writer"),
```

A removal emitting `Request::Erase { key: Key::Writer(forge) }` therefore also needs temper’s drain to translate it to temper’s top-level `Key::Writer(forge)`.

`Config`, `Writer` and `Entry` field shapes need not change to implement this; the lifecycle and erasure behavior does.

### Reverse dependencies

`Config`/`Writer` definitions, direct construction, field reading, persistence and restoration:

```text
skein/crates/skein-forge-client/src/lib.rs:31
skein/crates/skein-forge-client/src/identity.rs:53,82,97,113,135
skein/crates/skein-forge-client/src/domain.rs:25,47,55,76,153,193
skein/crates/skein-forge-client/src/boundary.rs:198,208
skein/crates/skein-forge-client/src/keep/restore.rs:3
skein/crates/skein-forge-client/src/measure.rs:24
skein/crates/skein-forge-client/src/tests.rs:522
skein/crates/skein-forge-client/src/tests/fixtures.rs:142
skein/crates/skein-forge-client/src/tests/fixtures/keep.rs:65
skein/crates/skein-forge-client/src/tests/keep.rs:153
skein/crates/skein-forge-client/src/tests/writer.rs:8,19,23,50

skein/tests/forge/client/src/world.rs:233,385,816
skein/tests/forge/client/tests/suite/memory.rs:44
skein/tests/forge/protocol/src/world.rs:805,877

temper/crates/temper-service/src/forge.rs:454
temper/crates/temper-service/src/tests/fixtures.rs:397
temper/crates/temper-service/src/tests/engine.rs:27
temper/crates/temper-service/src/tests/store.rs:205
temper/crates/temper-service/src/tests/setup_lifecycle.rs:698

temper/crates/temper-engine-domain/src/translate.rs:27
temper/crates/temper-engine-domain/src/domain.rs:270,1215
temper/crates/temper-engine-domain/src/tests/resume_fixture.rs:343

temper/crates/temper-engine-domain-forge/src/boundary.rs:852
temper/crates/temper-engine-domain-forge/src/domain.rs:492,523,1121,1487,1523,4875
temper/crates/temper-engine-domain-forge/src/lib.rs:240,248
temper/crates/temper-engine-domain-forge/src/reconcile.rs:210,239
temper/crates/temper-engine-domain-forge/src/restore.rs:247
temper/crates/temper-engine-domain-forge/src/adoption.rs:93,104,288
temper/crates/temper-engine-domain-forge/src/capabilities.rs:30
temper/crates/temper-engine-domain-forge/src/landing.rs:888
temper/crates/temper-engine-domain-forge/src/tests.rs:128,1178,1203,1219,1255,1274,1427
temper/crates/temper-engine-domain-forge/src/tests/limits.rs:62
temper/crates/temper-engine-domain-forge/src/tests/connect.rs:14,325
temper/crates/temper-engine-domain-forge/src/tests/adoption.rs:304,407
temper/crates/temper-engine-domain-forge/src/tests/restore_fit.rs:89,574
temper/crates/temper-engine-domain-forge/src/tests/panel.rs:143,275,292
temper/crates/temper-engine-domain-forge/src/tests/reconcile.rs:226,235,673

temper/crates/temper-protocol-store/src/records.rs:78,135
temper/crates/temper-protocol-store/src/records/client.rs:180
temper/crates/temper-protocol-store/src/keys.rs:176,209,300
temper/crates/temper-protocol-store/src/tests/records.rs:769,1197,1210,1227
temper/crates/temper-protocol-store/src/tests/fixtures.rs:173
temper/crates/temper-protocol-store/src/tests/keys.rs:115

temper/crates/temper-fake/src/check.rs:94
temper/crates/temper-fake/src/tests/check.rs:197
temper/crates/temper/src/fake/check/tests.rs:280

temper/tests/engine/forge/src/world.rs:107,296,337,507
temper/tests/engine/forge/tests/suite/memory.rs:54
temper/tests/engine/forge/tests/suite/writer.rs:7,25
temper/tests/engine/protocol-forge/src/protocol.rs:134
temper/tests/engine/domain/src/walking.rs:352
temper/tests/engine/domain/tests/suite/reconcile.rs:79
temper/tests/codecs/tests/suite/fuzzy/store.rs:75
temper/tests/client/tests/qa_all/qa_forges.rs:108
```

The public `skein_forge_client::writer(...)` function itself is called by:

```text
skein/crates/skein-forge-client/src/tests/writer.rs:25
skein/tests/forge/client/src/world.rs:234
temper/crates/temper-engine-domain-forge/src/domain.rs:1488
```

All implementation files reading/mutating the private working state that immediate removal must coordinate:

```text
skein/crates/skein-forge-client/src/domain.rs:97
skein/crates/skein-forge-client/src/identity.rs:83
skein/crates/skein-forge-client/src/calls.rs:84
skein/crates/skein-forge-client/src/outbox.rs:172
skein/crates/skein-forge-client/src/outbox/answer.rs:12
skein/crates/skein-forge-client/src/outbox/find.rs:20
skein/crates/skein-forge-client/src/outbox/queue.rs:27
skein/crates/skein-forge-client/src/outbox/retry.rs:19
skein/crates/skein-forge-client/src/outbox/stop.rs:25
skein/crates/skein-forge-client/src/keep.rs:269
skein/crates/skein-forge-client/src/keep/answer.rs:13
skein/crates/skein-forge-client/src/keep/echo.rs:96
skein/crates/skein-forge-client/src/keep/poll.rs:7
skein/crates/skein-forge-client/src/keep/restore.rs:4
skein/crates/skein-forge-client/src/tests/fixtures/outbox.rs:83
skein/crates/skein-forge-client/src/tests/fixtures/keep.rs:85
```

Direct consumers in `smith/` and `jig/`: **not found**.

### Tests and worlds

| Package / binary | Relevant coverage |
|---|---|
| `skein-forge-client` / library tests | `src/tests.rs:522`, `deployment_config_and_decoded_key_frames_are_checked`. |
| Same | `src/tests/writer.rs:46`: runtime writer saves a row; `:54`: same writer; `:65`: conflicting author; `:78`: zero/full/unconfigured refusal; `:97`: effects require a writer; `:109`: restoration; `:120`: failed restoration. |
| Same | `src/tests/contracts.rs:74,85,92,430`: namespace binding restrictions. |
| Same | `src/tests/outbox.rs`, `recovery.rs`, `settle.rs`, `keep.rs`, `queue.rs`, `budget.rs`: existing flight/outbox/working-set lifecycle. |
| `skein-forge-client-world` / `suite` | Root `skein/tests/forge/client/tests/suite/main.rs`; construction/runtime writer in `src/world.rs:233,816`; `tests/suite/recovery.rs:99`, `memory.rs:44`, plus `simulation.rs`, `queue.rs`. |
| `skein-forge-protocol-world` / `suite` | `skein/tests/forge/protocol/src/world.rs:877`; exercises configured client with the connection component. |
| `temper-engine-forge-world` / `suite` | `temper/tests/engine/forge/tests/suite/writer.rs:7,25`: runtime writer persistence and writerless restart restoration. World at `src/world.rs:337`. |
| `temper-protocol-store` / library tests | `src/tests/records.rs:769`: writer golden round trips; `:1197` onward: writer persistence through connector decisions. |

Immediate forge removal, writer erasure and erased-flight late-terminal tests: **not found**.

## 4. Other skein crates with a fixed forge list

**No other fixed forge registry was found.**

| Crate / candidate | Finding |
|---|---|
| `skein-forge` | No configured-forge list. It holds neutral API types, documents, plans, credentials and shared bounded lookup tables. |
| `skein-forge-routing` | **Not found**: neither crate nor `skein-forge/src/routing.rs`. Routing belongs to consumers. |
| Forge credentials | No forge registry. `skein/crates/skein-forge/src/credential.rs:6` defines one credential value. |
| Fake-forge crates | Model individual fake services/forges and their provider state; no additional deployment forge registry was found. |

`skein-forge`’s forge-associated types:

```rust
// skein/crates/skein-forge/src/api.rs:58
pub struct Repository {
    pub forge: u16,
    pub repository: u32,
}

// skein/crates/skein-forge/src/credential.rs:6
pub enum Credential {
    Token(Secret),
    App { id: u64, key: Secret },
}
```

Credentials themselves do not carry a forge number; their owning connection/settings do.

The two table types retain dynamically discovered forge-associated rows:

```rust
// forgejo/plans/tables.rs:27
pub struct Tables {
    repositories: Map<api::Repository, RepositoryEntry>,
    users: Map<(u16, u64), UserEntry>,
    clock: u64,
}

// github/plans/tables.rs:34
pub struct Tables {
    repositories: Map<api::Repository, RepositoryEntry>,
    users: Map<(u16, u64), UserEntry>,
    clock: u64,
}
```

Both expose:

```rust
new(repositories: u32, users: u32) -> Self;
repository(&mut self, key: api::Repository) -> Option<&Repository>;
keep_repository(&mut self, key: api::Repository, row: Repository);
drop_repository(&mut self, key: api::Repository);
user(&mut self, forge: u16, id: u64) -> Option<&[u8]>;
keep_user(&mut self, forge: u16, id: u64, login: Box<[u8]>);
```

Locations:

- Forgejo: `skein/crates/skein-forge/src/forgejo/plans/tables.rs:34,42,48,64,68,74`.
- GitHub: `skein/crates/skein-forge/src/github/plans/tables.rs:42,51,57,77,81,87`.

Per-forge purge and individual user deletion: **not found**.

If table purge is added, current table readers/constructors are:

```text
skein/crates/skein-forge/src/forgejo/plans/plan.rs:26
skein/crates/skein-forge/src/forgejo/plans/turn.rs:6
skein/crates/skein-forge/src/forgejo/plans/actions.rs:7
skein/crates/skein-forge/src/forgejo/plans/contents.rs:7
skein/crates/skein-forge/src/forgejo/plans/reads.rs:6
skein/crates/skein-forge/src/forgejo/plans/reads/keep.rs:2
skein/crates/skein-forge/src/forgejo/plans/reads/done.rs:2
skein/crates/skein-forge/src/forgejo/plans/writes.rs:9

skein/crates/skein-forge/src/github/plans/plan.rs:2
skein/crates/skein-forge/src/github/plans/failure.rs:2
skein/crates/skein-forge/src/github/plans/reads.rs:5
skein/crates/skein-forge/src/github/plans/reads/people.rs:2
skein/crates/skein-forge/src/github/plans/reads/done.rs:2
skein/crates/skein-forge/src/github/plans/writes.rs:5
skein/crates/skein-forge/src/github/plans/writes/reviewers.rs:2
skein/crates/skein-forge/src/github/plans/tests.rs:1
skein/crates/skein-forge/tests/suite/github_items.rs:13
skein/crates/skein-forge/tests/suite/github_plans.rs:14

skein/crates/skein-forge-connection/src/component.rs:22,161,165,820,824
skein/crates/skein-forge-connection/src/component/github_calls.rs:15,35
skein/crates/skein-forge-connection/src/tests/fixtures.rs:185
skein/crates/skein-forge-connection/src/tests/reads.rs:638
skein/crates/skein-forge-connection/src/tests/puller.rs:141
skein/crates/skein-forge-connection/src/tests/github_calls.rs:301
skein/crates/skein-forge-connection/src/tests/github_unknown_repository.rs:129
skein/crates/skein-forge-connection/src/tests/github_writes_heap.rs:65
skein/tests/forge/machine/src/memory.rs:487
skein/tests/forge/github/src/runner.rs:72
```

These tables have capacity construction, not per-forge construction. Relevant tests are `skein-forge` library tests in `src/github/plans/tests.rs`, its integration binary `suite` rooted at `skein/crates/skein-forge/tests/suite/main.rs`, and the connection/machine tests listed above.

## 5. Temper construction and consumers of new APIs

### Production construction chain

| Surface | Construction / caller |
|---|---|
| Connection `Settings` and `Forge` | `temper/crates/temper-service/src/forge.rs:42`, `pub fn settings(...)`; constructs `Forge` at `:84`, `Settings` at `:103`. |
| Connection validation | `temper/crates/temper-service/src/forge.rs:436`, `prepare`; calls `limits::check` at `:442`. Startup wrapper at `src/startup.rs:1066`. |
| Connection component | `temper/crates/temper-service/src/forge.rs:218`, `Layer::new`; calls `Component::new` at `:237`. |
| Connection layer’s caller | `temper/crates/temper-service/src/system.rs:217`. |
| Existing access replacement | `temper/crates/temper-service/src/forge.rs:478`, `connect`; sends `Request::Connect` at `:488,505`, `Installations` at `:501`. |
| Setup Forgejo `Given` | `temper/crates/temper-service/src/setup/settings.rs:104`, `forgejo`; literal at `:142`. |
| Setup GitHub `Given` | `temper/crates/temper-service/src/setup/settings.rs:49`. |
| Setup host `Forge` | `temper/crates/temper-service/src/setup/settings.rs:40,81`. |
| Setup host `Limits` and `Settings` | `temper/crates/temper-service/src/setup/settings.rs:85,102`, in `fixed`. |
| Setup layer | `temper/crates/temper-service/src/setup/settings.rs:163`, `prepare`; calls `Layer::new` at `:172`; definition at `src/setup.rs:97`. |
| Setup layer’s caller | `temper/crates/temper-service/src/system.rs:215`. |
| Setup `Host` | `temper/crates/temper-service/src/setup.rs:432`, `begin`; calls `host::Host::new` at `:439`, after records load and any required webhook bind. |
| Client `Config` | `temper/crates/temper-service/src/forge.rs:454`, `config`; literal at `:465`, initially with **empty writers**. Called by `src/system.rs:197`. |
| Root carries client config | `temper/crates/temper-service/src/system.rs:228`; `temper-engine-domain/src/translate.rs:27`. |
| Root constructs forge connector | `temper/crates/temper-engine-domain/src/domain.rs:275`; validation also constructs it at `:1220`. |
| Forge connector constructs skein client | `temper/crates/temper-engine-domain-forge/src/domain.rs:492`, `Domain::new`; calls `skein_forge_client::Domain::configured` at `:582`. |
| Runtime authenticated writer | `temper/crates/temper-engine-domain-forge/src/domain.rs:1121`; wrapper `pub fn writer(...) -> bool` at `:1487`, calls skein at `:1488` and mirrors writer into temper’s map at `:1491`. |

Connection preparation’s exact signature:

```rust
// temper/crates/temper-service/src/forge.rs:42
pub fn settings(
    forges: &[settings::Forge],
    runtime: List<ForgeRuntime>,
    webhooks: Option<skein_forge_connection::hooks::Webhooks>,
    found: &mut Problems,
) -> Option<skein_forge_connection::settings::Settings>
```

Setup preparation:

```rust
// temper/crates/temper-service/src/setup/settings.rs:8
pub(super) fn fixed(
    config: &crate::settings::Engine,
    forges: &skein_forge_connection::Settings,
) -> Result<(host::Settings, host::Limits), Problems>
```

Forge connector construction:

```rust
// temper/crates/temper-engine-domain-forge/src/domain.rs:492
pub fn new(
    l: &Limits,
    seed: u64,
    config: skein_forge_client::Config,
    identities: &[crate::GitIdentity],
    forges: &[crate::Configured],
    under_box: bool,
) -> Result<Domain, skein_forge::api::Error>
```

### Temper layers that must participate

Adding/removing only the skein component’s entry would leave fixed temper state:

| File | Fixed/cached state |
|---|---|
| `temper/crates/temper-service/src/forge.rs:202` | `Layer.held: Map<u16, Box<[u8]>>`, prepared in `Layer::new`; runtime down/begin gating. |
| `temper/crates/temper-service/src/setup.rs:18` | Pending host settings, `served`, `unserved`, staged plans, runs, public callers, paste associations, file API tokens, erases, pages and waiting/held setup requests. |
| `temper/crates/temper-service/src/setup/lifecycle.rs:228` | Plan classification and lifecycle; constructs adoption/reconcile/takeover/create plans at `:753,804,894,964,1018,1427`. |
| `temper/crates/temper-service/src/setup/public.rs:42` | Person-facing setup staging and requests. |
| `temper/crates/temper-service/src/setup/routes.rs:354,451` | GitHub callback plans. |
| `temper/crates/temper-service/src/forge/records.rs:11` | `Rows.configured: List<u16>`; `Rows::new` at `:17`; rejects unconfigured forge records at `:29`. |
| `temper/crates/temper-service/src/forge/tokens.rs:28` | `Tokens.forges: Map<u16, Forge>`; populated only from configured GitHub forges in `Tokens::new` at `:36`. |
| `temper/crates/temper-service/src/startup/application.rs:1291` | Builds the connector’s `Box<[Configured]>`. |
| `temper/crates/temper-engine-domain-forge/src/domain.rs:507` | Copies configured forges, git identities and writers; owns additional `entries` and `flights` at `:555,556`. |
| `temper/crates/temper-engine-domain-forge/src/domain.rs:1509` | Client-output translation, including the writer-erasure issue described in question 3. |

Temper’s existing domain `ended` handling is not removal: service gating checks it in `temper/crates/temper-service/src/forge.rs:12,478`.

### One-shot setup path

This path constructs a pure reconcile machine, not `host::Host`:

```text
temper/crates/temper-forge-setup/src/given.rs:103
  pub fn values(
      entry: &crate::setup::Entry,
      listener: Option<core::net::SocketAddr>,
      limits: &skein_forge_setup::limits::Limits,
  ) -> skein_forge_setup::given::Given

temper/crates/temper-forge-setup-shell/src/reconcile.rs:206
  Reconcile::new(&settings.given)
```

Its tests are in:

```text
temper/crates/temper-forge-setup/src/tests/given.rs:37
temper/crates/temper-forge-setup-shell/tests/suite/reconcile.rs
temper/crates/temper-forge-setup-shell/tests/suite/exchange.rs
```

The first uses the `temper-forge-setup` library test binary; the latter two belong to the `temper-forge-setup-shell` integration binary `suite`. The production consumers of new runtime registration/removal APIs are the service and connector layers above; this one-shot path does not hold a forge registry.