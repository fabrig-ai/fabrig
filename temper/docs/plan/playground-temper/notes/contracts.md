# playground-temper: contracts

The run's binding calls for the shapes that cross tasks (`temper/docs/design/playground.md`,
3 to 5). A task builds its half to this note; names inside a module are its implementer's, and
its report lists the public ones its readers use.

## C1. The crate

- `temper/crates/temper-playground`, package `temper-playground`, a library on every target
  (`playground.md`, 3). Its `[dependencies]` are product crates alone (`*/crates/*`), on every
  target; test kits only in `[dev-dependencies]`. It is ordinary Rust
  (`skein/docs/foundation/programming-model.md`, 10.2), as `temper-fake-git` is: its `clippy.toml`
  is `temper/crates/temper-fake-git/clippy.toml`'s bans, its header "Ordinary Rust
  (programming-model.md, 10.2) that the playground ships: the world's composition and its
  stories". Its `[lints]` are the one-process world's table.
- Root `Cargo.toml`: its member line goes right after `"temper/crates/temper-fake",`, its
  workspace dependency right after `temper-fake = { path = ... }`, so that no other task's lines
  meet it.
- Modules, each a task's (the task named owns it):

  | Module | What | Task |
  |---|---|---|
  | `documents` | the deployment's document and every value the one-process world takes from test kits today (jig's channel world's engine and worker documents, files, seeds and agent configuration; smith's process world's profile, declarations and limits; skein's LLM world's config and limits; the hosted git's identity, config and table), as the playground's own | `playground-documents` |
  | `fakes` | the fakes' places and connection limits: the fake LLM peer's config and limits, the fake forge's table, setup peer and limits | `playground-documents` |
  | `machine` | `Machine`, `Rooted<H>`, `SignalHost<H>`, `fresh` | `playground-processes` |
  | `service` | `ServiceHost<H>` and the trait `Observe` | `playground-processes` |
  | `programs` | `Process<H>`, the trait `Harness`, the factories and `programs` | `playground-processes` |
  | `story` | `Start`, `Fakes`, `Links` | `playground-processes` |
  | `stories` | `Story` and story A's documents | `story-a` |

## C2. The harness's seam

The composition's processes and the harness's share one process type, so the world's program
registry and its factories (`fn` pointers) stay as they are:

```rust
pub trait Harness: 'static {
    /// What the harness keeps beside the service: captures, cuts, the wire (C2's `Observe`).
    type Service: Observe;
    /// The harness's own processes: a person, a terminal client, a hosted ctl.
    type Process: skein_shell::Host;
    /// Edits the deployment's document before its bytes start the service.
    fn configure(_document: &mut serde_json::Value) {}
}

pub trait Observe {
    fn started(root: skein_io::kernel::Fd, inherited: &skein_sim_world::Inherited) -> Self;
    // and the hooks `ServiceHost::iterate` calls where the one-process world's capture, wire,
    // commits, cut and reserve act today (temper/tests/one-process/src/world.rs, ~83-187)
}

pub enum Process<H: Harness> {
    Service(..), Agent(..), GitRoute(..), Git(..), Shell(..), Peer(..), Forge(..),
    Other(H::Process),
}

/// No harness: the playground's world (`playground.md`, 4).
impl Harness for () { type Service = (); type Process = Never; }
pub enum Never {}

pub fn service_roots() -> Vec<skein_sim_world::StartupRoot>;
pub fn programs<H: Harness>() -> Vec<(skein_sim_world::HostedProgram<Process<H>>,
                                      skein_sim_world::StartupRoots)>; // agent, git, git route, sh
pub fn world(start: Start) -> skein_sim_world::World<Process<()>, Machine>;
```

Each world registers `programs()` with its own `host_roots`, the one-process world on
`skein_world::World`, the playground's on `skein_sim_world::World`, and adds its own programs
(the one-process world's `/world/temper-ctl`).

## C3. A world's start

```rust
pub struct Start {
    pub seed: u64,                 // the visit's one seed (`playground.md`, 4)
    pub wall: skein_lib::Wall,     // the world's wall at its start
    pub deployment: Box<[u8]>,     // the deployment's document, as `temper serve` reads it
    pub fakes: Fakes,              // built, not documents: a story's documents or a harness's code
    pub links: Links,
}
pub struct Fakes { pub llm: skein_fake_peers::llm::Peer, pub forge: temper_fake_git::Typed /* and the forge peer's setup */ }
pub enum Links {
    /// The harness takes the service's links itself (the one-process world's referee, today).
    Keep,
    /// The service's host drains its links each iteration and writes each as one line to its
    /// standard output, as `temper serve`'s host does (`playground.md`, 4).
    Print,
}
```

- **One seed:** `documents::seeds(seed)` derives the service's, the agents' and the fakes' seeds
  from `Start::seed`; the simulator's is `Start::seed`. A harness that keeps seeds of its own
  (the one-process world's, so that its stories stay as they are) sets them through
  `Harness::configure`.
- **No box:** the composition lays on the fake disk, before the service starts, the files a
  handover read names (the playground's setup entry's admin token, the fake LLM's key) and
  answers each `take_handover_read` from the disk at its path, bounded by its `max`; a removal
  is answered `Ok(())`. A harness whose stories answer otherwise does so through `Observe`.

## C4. Stories as data

```rust
pub struct Story {
    pub name: &'static str,           // "a"
    pub description: &'static str,    // what the visitor is expected to do, which the frame shows
    pub deployment: &'static [u8],    // names the scripted provider and the fakes' places
    pub llm: &'static [u8],           // temper-fake's LLM document of its own, with `pace`
    pub forge: &'static [u8],         // temper-fake's forge document of its own, with `pace`
}
pub const A: Story;
pub fn all() -> &'static [Story];
impl Story { pub fn start(&self, seed: u64, wall: skein_lib::Wall) -> Start; }
```

- Each document is a file under `temper/crates/temper-playground/stories/<name>/`
  (`deployment.json`, `llm.json`, `forge.json`, `description.txt`), compiled in by
  `include_bytes!` (`playground.md`, 5.1); `Story::start` decodes the fakes' by
  `temper_fake::llm::Document::decode` and `temper_fake::forge::Document::decode` and builds
  `Fakes` as `temper fake` builds its peers, with `Links::Print`.
- The pace is the documents' `pace` (`temper/docs/design/protocol/setup.md`, 8.4: the LLM's
  `{"reply_ms"}`, the forge's `{"ci_ms"}`, each at most 60000), which `fake-pace` makes
  `temper-fake` read.
