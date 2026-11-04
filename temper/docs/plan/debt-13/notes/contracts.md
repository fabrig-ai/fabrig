# debt-13: the shapes that cross tasks

Binding for this plan's tasks. The design sections are the contract; this note gives the names they build to.
A task builds only the parts its spec names. Every name here is new unless the note says it stands at base.

## 1. skein-forge-connection: one entry per forge

`skein/docs/design/forge-connection.md`, 7.1. In `skein-forge-connection/src/component/entry.rs` (new), all
`pub(crate)`:

```rust
/// One configured forge (forge-connection.md, 7.1).
pub(crate) struct ForgeEntry {
    pub base: Base,
    pub dialect: Dialect,
}

/// What every forge has, whichever its dialect.
pub(crate) struct Base {
    pub forge: Forge,            // settings::Forge, as the owner configured it
    pub start: Start,            // start::Start
    pub own: Option<Token>,      // the forge's own connection
}

/// A forge's dialect, and what only that dialect has.
pub(crate) enum Dialect {
    Forgejo { plan: Option<skein_forge::forgejo::plans::start::Plan> },
    GitHub(Box<GitHubForge>),
}

pub(crate) struct GitHubForge {
    pub state: crate::installations::GitHub,
    pub plan: Option<OwnPlan>,   // OwnPlan keeps its GitHub variants, spelled as at base
}
```

- `Settings` stays the owner's public input, unchanged. `Component::new` splits it: each `Forge` goes into an
  entry, and the rest into `settings::Owner { webhooks, word, user_agent }` (`pub(crate)`).
- `Component` becomes `{ forges: List<ForgeEntry>, common: Common }`. `Common` (`component.rs`, `pub(crate)`)
  holds every other field of today's `Component`, `owner: Owner` among them.
- A step on one forge is a method of `Common` (or a free function) that takes the entry, or after
  `forge-dialect` takes `(&mut Base, &mut GitHubForge)` for GitHub's and `(&mut Base, plan)` for Forgejo's. It
  never takes a position.
- `Component::add`, `Component::remove` and every public method of `Component` keep their signatures and what
  they do; `remove` is `List::swap_remove`, so the order forges are driven in stays what it is.
- `connection::Use` (`pub(crate)`): `Calls`, `Own` (a Forgejo forge's own connection) or `Reserved` (a GitHub
  forge's own connection, outside the calls' count). `Connection::role: Use` carries what `call_connections`
  and `assign` find today by scanning every forge's own token. It changes where the entry's `own` changes.

## 2. skein-forge-connection: a call's dialect

`skein/docs/design/forge-connection.md`, 7.1. `Waiting` loses `plan`, `route`, `discovery` and `discovered`, and
gains `side: Side`:

```rust
pub(crate) enum Side {
    Unbegun,
    Forgejo(Box<ForgejoCall>),
    GitHub(Box<GitHubCall>),
}
pub(crate) struct ForgejoCall { pub plan: Plan }
pub(crate) struct GitHubCall {
    pub plan: Option<github::plans::plan::Plan>,
    pub route: Option<Route>,
    pub discovery: Option<Discovery>,
    pub discovered: Option<api::Answer>,
}
```

`Side` is chosen and matched against the forge's `Dialect` in one place, `drive_calls`: `Unbegun` begins in the
forge's dialect; a variant that isn't its forge's finishes the call `api::Error::Unavailable`, as a call to a
forge that has gone. Below that place a function reads its own variant by a method of `Waiting` that answers
`Option`: `fn forgejo(&mut self) -> Option<&mut ForgejoCall>`, `fn github(&mut self) -> Option<&mut GitHubCall>`.

## 3. skein-forge-connection: a connection with no dialect

`skein/docs/design/forge-connection.md`, 8.1.

- `Connection` keeps its stack (link, HTTP client, tokenizer, the raw and tail bodies), holds no dialect's type
  and no `Kind`, and `heard`, `Heard`, `Exchange`, `Refusal`, `Request`, `Decoder` and `Body` leave it.
- `Connection::step(&mut self, env, io, held: bool) -> Moved` is one movement of today's `pump` loop. `Moved`
  (`pub(crate)`) names what the holder must read: `Within` (a movement, nothing to read), `Nothing` (stop),
  `Head(http::Response)`, `Token(tokenizer::Event)`, `Finished { reuse, raw }`, with `raw` the raw or tail body
  and whether it was cut.
- A failure or a malformed answer is kept on the connection, `Ended::{Failed(Failure), Malformed(..)}`, and
  `take_ended` hands it to the holder after what it has heard.
- `connection::forgejo::Exchange` (`src/connection/forgejo.rs`) and `connection::github::Exchange`
  (`src/connection/github.rs`) hold the request, the decoder, what was heard and the finished body, each in its
  own dialect's types. Each has:

  ```rust
  pub fn new() -> Self;
  pub fn start(&mut self, c: &mut Connection, env, forge: &Forge, user_agent: &[u8], ...) -> Result<(), its Refusal>;
  pub fn pump(&mut self, c: &mut Connection, env, io);   // the 64 movements of today's pump, in the same order
  pub fn heard(&mut self, c: &mut Connection) -> Option<its Heard>;
  pub fn ready(&self, c: &Connection) -> bool;
  ```

- The holder keeps its exchange: `Dialect::Forgejo` and `GitHubForge` gain `exchange`, as `ForgejoCall` and
  `GitHubCall` do. A holder's exchange is made new each time it takes a connection.

## 4. jig: what the roots share

`jig/docs/design/domain/root.md`, 3 and 5; `jig/docs/design/domain/hosts.md`, 8.

```rust
// jig-domain-link
/// How a root takes an event (jig/docs/design/domain/root.md, 3).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Class { Admitted, New, Mixed }
impl Class {
    /// A child's `new_work`, as a class.
    #[must_use]
    pub const fn of(new_work: bool) -> Self;
}

// jig-domain-host
impl LinkUp {
    /// Calls start new work; other reports settle hosted work (domain/hosts.md, 6).
    #[must_use]
    pub const fn new_work(&self) -> bool;
}

// jig-domain-tasks
impl Party {
    /// The numeric party label carried by a host message (domain/hosts.md, 9).
    #[must_use]
    pub fn sender(self) -> Box<[u8]>;
}

// jig-domain-core
impl Core {   // the type that holds `commit_header` and `cold_header` at base
    /// The deployment row this decision writes, if any (engine.md, 5.4): a writing decision's, or a cold one's.
    #[must_use]
    pub fn decision_header(&mut self, wrote: bool) -> Option<Record>;
}

// skein-lib
impl<L> Env<L> {
    /// Whether a wall-clock expiry has come, on this step's clocks.
    #[must_use]
    pub fn wall_due(&self, expiry: Wall) -> bool;
}

// jig-domain-hub-link
pub fn job_assignment(job: JobStart) -> host::JobAssignment;
pub fn settled_answer(answer: core::SettledAnswer) -> Result<host::Relayed, core::SettledAnswer>;
pub fn brief_plan(task: u64, sections: List<brief::Planned>, budget: u32, deadline: Time) -> core::Event;
pub fn no_workspace(task: u64, attempt: u64) -> core::Event;
impl Link { pub fn complete_brief(&mut self, brief: Token, order: Box<[brief::GatherPlaced]>) -> core::Event; }
/// The core's `Now`s that are the link's; any other comes back.
pub fn now(value: core::Now) -> Result<Event, core::Now>;

// jig-protocol-charter
/// A root's own part of a run's hand-over (domain/hosts.md, 8).
pub struct Given<'a> {
    pub tools: Box<[Tool]>,
    pub endpoints: &'a [EndpointName],
    pub deployment: &'a [u8; 16],
    pub limits: &'a Limits,
    pub answered: u32,
    pub messages: u32,
}
pub fn hosted(assignment: &jig_domain_hub_link::Assignment, given: Given<'_>) -> Option<host::Assignment>;
```

The `pub use` of `Class` at each root's `admission` keeps its readers' paths. Field and type names in these
signatures stand at base unless this note names them new; where a type's path differs from the sketch, the
base's path wins and the report says so.
