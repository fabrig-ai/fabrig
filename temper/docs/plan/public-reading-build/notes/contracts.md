# public-reading-build: contracts

The shapes that cross this plan's tasks: build to them exactly. The design they come from is
jig's `jig/docs/design/domain/people.md`, section 13, `jig/docs/design/protocol/client.md`,
17.5 and section 19, `jig/docs/design/client/domain/contract.md`, section 12, and temper's
`temper/docs/design/client/forge.md`, section 11. A name below that a task creates is that
task's; every other task builds on it as written. Private helpers are each task's own.

## 1. Limits (`public-limits`)

- `jig_protocol_engine_clients::Limits` (`limits.rs`) gains `pub public: PublicLimits`:

  ```rust
  /// Public reading's admission (jig/docs/design/domain/people.md, 13.5).
  pub struct PublicLimits {
      pub reads: u32,        // clients.public.reads, in flight
      pub rate: u32,         // clients.public.rate, reads per second
      pub rate_burst: u32,   // clients.public.rate_burst
      pub bytes: u64,        // clients.public.bytes, answer bytes per second
      pub bytes_burst: u64,  // clients.public.bytes_burst
      pub answer_bytes: u32, // clients.public.answer_bytes, one answer's ceiling
      pub max_age: u32,      // clients.public.max_age, seconds, at most 60
  }
  ```
  `PublicLimits::OFF` is every field zero, and `PublicLimits::on(&self) -> bool` is
  `self.rate > 0`.
- `jig_protocol_engine_clients_http::Limits` gains `pub public_calls: u32`.
- `jig_domain_core`'s `Limits` (`vocabulary.rs`) gains `pub public: PublicIndexLimits`
  (`goals`, `ended`, `lines`, `depth`, each `u16`; keys `core.public.goals` and so on), with
  `PublicIndexLimits::OFF`, every field zero.
- jig's root `Limits` (`jig/testing/jig-test-domain/src/lib.rs`) and temper's
  (`temper/crates/temper-engine-domain/src/limits.rs`) gain `pub public_watermark: u32` and
  `pub public_reads: u32`, set where their watermark is set (`jig-test-service`'s and
  `jig-test-profile`'s limits, `temper-service`'s `limits.rs`): `public_watermark` half the
  watermark, `public_reads` the clients' `public.reads`.
- Every profile's public figures are zero. The relations of
  `jig/docs/design/protocol/configuration.md`, section 6, are checked by jig-protocol-config
  at start once `clients.public.rate > 0`; with a rate of zero only `max_age <= 60` is.
- `jig_client_protocol_wire::public_bounds(index: &PublicIndexBounds, projects: u32) -> [u32; 3]`
  (section 3) gives jig's three kinds' largest answers; the start checks each against
  `answer_bytes`.

## 2. Visibility (`visibility-surface`, `visibility-wire`, `visibility-core`)

- `jig_domain_people::Ask::SetVisibility { project: u32, public: bool }`, appended last, and
  `jig_domain_people::Stored::Public { project: u32 }`, appended last (`visibility-surface`).
- `jig_client_domain::contract::Ask::SetVisibility { project: u32, public: bool }`, appended
  last, its project typed as `RemoveProject`'s is (`visibility-wire`).
- jig-protocol-store's people family, `schema/people-v14.schema` in place, as its recent
  changes are (commit `95f342a4aa`): its `Ask` enum gains
  `set_visibility: AskSetVisibility { project: u32 public: bool }`, appended last, and a row
  `StoredPublic { project: u32 }` under a key of its own, present while the project is public
  and erased when it is made private or deleted, so a store written before holds none and
  reads every project private.
- The secrets' store keeps the public identifiers' key: `jig_protocol_store::SecretOwner`
  gains `PublicNames`, one row (number 0), its value 32 bytes, in `secrets-v1.schema` in place
  where its record needs one.
- The client wire v19, in place (as configuring-p2-build's `settings-wire` did): `Ask` gains
  `set_visibility` (`project`, `public`), appended last; `MeProject` gains
  `public: option u64`, the project's public identifier while it is public.
- `jig_client_domain::contract::MeProject` gains `pub public: Option<u64>`; the core's
  `FrameProjectRole` gains `pub public: bool`; the clients' component fills `MeProject.public`
  from it through its public names (section 4), `None` until `public-component`.
- Until `visibility-core`, the people child refuses `SetVisibility` with the refusal it gives
  an act it does not take for that party (the implementer names it in the report).

## 3. jig's public records (`public-wire`)

`jig/crates/jig-client-protocol-wire/schema/public-v1.schema`, family `jig-public-wire 1`,
module `jig_client_protocol_wire::public_v1` (`src/generated/public_v1.rs`), goldens under
`golden/public/`, registered in `jig/tests/codec-families.rs`. `PUBLIC_VERSION: u16 = 1`.
Every identifier is `u64`; every time is `u64` seconds since the epoch.

```
record PublicProjects { projects: list u64, max_age: u32 }
record PublicBoard { project: u64, live: list PublicGoal, live_folded: u32, ended: list PublicGoal }
record PublicGoal { line: PublicLine, tree: list PublicLine, folded: u32 }
record PublicCard { line: PublicLine, lineage: list u64, delegates: list PublicLine, folded: u32 }
record PublicLine { task: u64, parent: option u64, goal: bool, priority: option u32,
                    executor: PublicExecutor, phase: PublicPhase, hold: option PublicHold,
                    made: u64, changed: u64, active: option u64, ended: option u64,
                    spent: u64, budget: u64, delegates: PublicCounts }
record PublicCounts { working: u32, waiting: u32, held: u32, closing: u32, ended: u32 }
record PublicApplication { tasks: list u64, record: bytes }
enum PublicExecutor { agent, procedure, person, job }            # unit variants
enum PublicPhase { ... }    # one unit variant per jig_client_domain::contract::Phase variant
enum PublicHold { ... }     # one unit variant per kind of hold a held task's line names
```

List bounds come from `PublicIndexBounds { goals, ended, lines, depth, projects }` (the
crate's own `u16`/`u32` struct, filled from section 1's limits by its callers), each list
within its bound, so `public_bounds` is exact. Kind numbers: *projects* 1, *board* 2, *card*
3. The schema's syntax is skein-codegen's as the client schema uses it; where the sketch
above departs from that syntax, the syntax wins and the report says how.

Text golden: `schema/public-v1.fields`, one line per reachable record field and enum
variant, with its type, tag and default, in the schema's order.

## 4. The core and the component (`public-vocabulary`)

In `jig-domain-core`, a module `public` (`src/public.rs`), its types re-exported at the
crate root:

```rust
pub struct PublicRead { pub owner: u16, pub version: u16, pub kind: u16,
                        pub project: Option<u64>, pub task: Option<u64> }
pub enum PublicAnswer { Document { decided: skein_lib::Time, document: PublicDocument },
                        Unknown, Busy }
pub enum PublicDocument { Projects(PublicProjects), Board(PublicBoard), Card(PublicCard),
                          Application(PublicApplication) }
pub struct PublicProjects { pub projects: Box<[u32]> }
pub struct PublicBoard { pub project: u32, pub live: Box<[PublicGoal]>, pub live_folded: u32,
                         pub ended: Box<[PublicGoal]> }
pub struct PublicGoal { pub line: PublicLine, pub tree: Box<[PublicLine]>, pub folded: u32 }
pub struct PublicCard { pub line: PublicLine, pub lineage: Box<[u64]>,
                        pub delegates: Box<[PublicLine]>, pub folded: u32 }
pub struct PublicLine { pub task: u64, pub parent: Option<u64>, pub goal: bool,
                        pub priority: Option<u32>, pub executor: PublicExecutor,
                        pub phase: PublicPhase, pub hold: Option<PublicHold>,
                        pub made: u64, pub changed: u64, pub active: Option<u64>,
                        pub ended: Option<u64>, pub spent: u64, pub budget: u64,
                        pub delegates: PublicCounts }
pub struct PublicCounts { pub working: u32, pub waiting: u32, pub held: u32,
                          pub closing: u32, pub ended: u32 }
pub struct PublicApplication { pub tasks: Box<[u64]>, pub record: Box<[u8]> }
// PublicExecutor, PublicPhase, PublicHold: as section 3's enums, unit variants.
```

Numbers here are internal (project `u32`, task `u64`); times are whole seconds. The
vocabulary gains:

- `Event::PublicRead { reply: ReplyTo, read: PublicRead }`, from the clients' component
  through the root, with no sign-in and no party. The component's `owner` is 0 for jig or a
  family's number plus one; the root maps a family to the connector that serves it, as it
  maps an application read's, so the `owner` the core sees is 0 or that connector's number
  plus one;
- `Ask::Public { reply: ReplyTo, kind: u16, version: u16, project: u32, task: Option<u64>,
  public: Box<[u64]> }`, the core's ask of the connector that declared the kind, `public` the
  project's public tasks, ascending;
- `Event::PublicAnswered { reply: ReplyTo, answer: Option<PublicApplication> }`, the
  connector's answer, back to the core (`None` is unknown);
- `Held::PublicRead { reply: ReplyTo, answer: PublicAnswer }`, the core's answer once
  durable, which the root hands to the clients' component.

Every connector answers `Ask::Public` with `Event::PublicAnswered { answer: None }` until its
application builds public kinds; the core answers every `Event::PublicRead` unknown until
`public-reads-core`; each root hands `Event::PublicRead` back busy until it admits it.

The clients' component (`jig-protocol-engine-clients`):

- `SecretOwner::PublicNames` in its boundary; at start it asks for the key, draws 32 bytes
  from its `CryptoRng` and keeps them when there is none, and answers public reads busy until
  it holds the key.
- `public::Names` (`src/public/names.rs`): `fn project(&self, n: u32) -> u64`,
  `fn task(&self, n: u64) -> u64`, `fn number(&self, id: u64, of: Of) -> u64` with
  `enum Of { Project, Task }`: a 4-round Feistel permutation over two 32-bit halves, the
  round function the first 4 bytes of `skein_lib::CryptoRng::new(key, stream)` with
  `stream = (domain << 40) | (round << 32) | half`, domain 1 for projects and 2 for tasks.
- `Settings` gains `pub public: List<PublicKind>`, every kind it serves, jig's three
  included: `PublicKind { owner: u16, kind: u16, first: u16, last: u16, task: bool,
  bound: u32 }`, `owner` 0 for jig and one more than a family's number for an application's
  (`jig/docs/design/protocol/client.md`, 17.5).
- The component's message: `PublicRead { owner, version, kind, ids: Box<[u64]> }` (at most
  two ids), answered `PublicReply::{Document { bytes: Box<[u8]>, decided: Time }, Unknown,
  Busy, NotServed { record: Box<[u8]> }}`, which the HTTP end maps to 17.5's answers.

## 5. temper's public records (`forge-public-wire`)

`temper/crates/temper-client-protocol-wire/schema/public-v1.schema`, family
`temper-public-wire 1`, module `temper_client_protocol_wire::public_v1`, goldens under
`golden/public/`, text golden `schema/public-v1.fields`, as section 3. Records:
`PublicChange`, `PublicLandings`, `PublicLanding` and their parts, as
`temper/docs/design/client/forge.md`, 11.1 and 11.3 list them. A task is a `u16`, its place in
the answer's `PublicApplication.tasks`; an address is the forge's `text`, within the panels'
address bound; a commit is as the panels carry one. Kind numbers: *change* 1, *landings* 2,
*landing* 3; owner: one more than `number(Owner::Forge)`.
`temper_client_protocol_wire::public_bounds(&Bounds) -> [u32; 3]` gives each kind's largest
answer, which `temper-service` declares to jig's component.

## 6. An exchange's cache and an answer's headers (`fetch-headers`)

From `skein/docs/design/ui-web.md`, section 10:

- `skein_ui_web::fetch::Exchange` gains `pub cache: Cache` and
  `pub keep: &'static [&'static str]`, with `pub enum Cache { Browser, NoStore }`
  (`NoStore` is fetch's `RequestCache::NoStore` on wasm32); `fetch::Report::Answered` gains
  `headers: Vec<(String, String)>`, only the kept names, lower case, each value within the
  exchange's `bound`, in the order `keep` names them. The bridge's and the session log's
  encodings carry both.
- `jig_client_shell_web::adapter::Asked::Exchange` gains `cache: fetch::Cache` and
  `keep: &'static [&'static str]`; `Heard::Answered` gains `headers: Vec<(String, String)>`.
  Every existing exchange passes `Cache::Browser` and `&[]`.
- jig's public reads name the headers they keep in
  `jig_client_protocol_http::PUBLIC_KEEP: &[&str] = &["cache-control", "age", "retry-after"]`
  (`public-http`), and send with `Cache::NoStore` (`public-web`).
