# ui-engine-split: the contracts between tasks

The shapes that cross tasks, binding. The design is
`temper/docs/design/protocol/README.md`, 4.1, 4.2 and section 5's rows;
this note pins the names the tasks build to. A task builds its half to
this note and never waits on another's half but where its spec's `needs`
says so.

## 1. `temper-protocol-brief` (task `brief-crate`)

A step crate, `temper/crates/temper-protocol-brief`, on `smith-domain-run`
and `skein-lib` alone:

```rust
pub fn fit_brief(charter: &mut run::Charter, workspace: Option<&run::Workspace>, limits: &run::Limits);
pub fn fit_brief_bytes(charter: &mut run::Charter, bytes: u64);
```

`run` is `smith_domain_run`. Moved from `temper-engine-smith/src/start.rs`
with their private helpers, unchanged. `temper-engine-smith` no longer
exports them; its own start calls the crate.

## 2. `temper-version` (task `version-crate`)

`temper/crates/temper-version`, its library `#![no_std]` with `alloc`, its
build script `build/main.rs` ordinary `std` Rust:

```rust
pub const VERSION: &str;   // the workspace's version, CARGO_PKG_VERSION
pub const COMMIT: &str;    // twelve hex digits, `-dirty` appended, or `unknown`
pub fn line(name: &str) -> alloc::string::String; // "{name} {VERSION} ({COMMIT})", no newline
```

Only a binary's package depends on it (`temper/docs/design/release.md`, 3).

## 3. jig's `levels` (task `jig-levels`)

`jig_protocol_store::levels(sets: &PatchSets) -> jig_protocol_store::codec::component::Levels`,
at the crate's root, as `jig/docs/design/protocol/store.md`, 10.5 sketches
it: the binary's patch level of each owner, `None` before its first patch.

## 4. `temper-shell` (task `shell-crate`)

`temper/crates/temper-shell`, ordinary Rust on the `binary` profile, on
`skein-shell`, `skein-lib`, `jig-protocol-config` and `temper-docs`.
Modules:

- `commands`: `Who`, `Row` and `Argument`, as `temper/src/commands.rs` has
  them today, and

  ```rust
  pub struct Table { pub binary: &'static str, pub rows: &'static [Row] }
  impl Table {
      pub fn find(&self, words: &[Box<[u8]>]) -> Option<&'static Row>;
      pub fn unknown(&self, words: &[Box<[u8]>]) -> String;   // "{binary}: no command{given}; {binary} --help lists them\n"
      pub fn forms(&self, row: &Row) -> String;
      pub fn refused(&self, row: &Row, why: &str) -> String;
      pub fn help(&self) -> String;                            // `{binary} --help`
      pub fn row(&self, row: &Row) -> String;                  // `{binary} COMMAND --help`
      pub fn reference(&self) -> String;                       // the `commands` page's text for this table
  }
  pub const DIRECTORIES: Argument;  // the `[--config …] [--data …] [--cache …]` argument
  pub const HELP: Row;              // the `help` row, every binary's
  ```

  Each renders what `temper`'s functions of the same names render today,
  `temper` replaced by `binary`: with `binary` `"temper"` and `temper`'s
  rows, byte for byte the same.
- `directories`: `SETTINGS`, `Directories` (its fields and its methods
  `jig`, `document`, `flags`, `open` and `read`), `Refusal`,
  `EnvironmentEntry`, `resolve`, `private_root` and `make_private`, each
  `pub`. `Directories::program_environment` stays in `temper`, a free
  function `temper::directories::program_environment(directories,
  environment, working, word)`, since it reads `printed`, which stays
  there until `engine-serve` moves both into `temper-engine-shell`
  (section 6): the program environment is the engine's, which its
  clients read.
- `help`: `Request`, `request` and `answer`, as `temper/src/help.rs`.
- `link_file`: as `temper/src/link_file.rs`.

`temper`'s library keeps every path it has: `temper::directories` is a
module re-exporting `temper_shell::directories::*` beside
`program_environment`; `help` and `link_file` are `pub use` of the shell's;
`temper::commands` keeps `COMMANDS` and its rows, holds `pub const TABLE:
Table = Table { binary: "temper", rows: &COMMANDS }`, and keeps `find`,
`unknown`, `forms`, `refused`, `help`, `row` and `reference` as one-line
wrappers over `TABLE`, so their callers don't change.

## 5. `temper-service`'s split (tasks `engine-limits`, `worker-feature`)

- **After `engine-limits`:** no module of `temper-service` outside
  `worker.rs`, `worker/`, `worker_route.rs`, `worker_git.rs`,
  `worker_settings.rs`, `identities.rs`, `program_sources.rs`,
  `startup/worker.rs` (new) and `tests/` names `temper_worker_*`,
  `jig_protocol_worker_agents`, `jig_protocol_worker_link`,
  `smith_host_*`, `smith_protocol_program` or `smith_protocol_channel`.
  `startup::nearest` is public:

  ```rust
  /// The document's nearest shape (protocol/README.md, 15.1), read from its
  /// members alone; `None` when the bytes are no JSON document whose root is an object.
  pub fn nearest(input: &[u8]) -> Option<Shape>;
  ```

- **After `worker-feature`:** `temper-service`'s `[features]` are
  `default = ["worker"]` and `worker = [the worker-only dependencies]`; the
  root's `[workspace.dependencies]` entry says `default-features = false`;
  every dependent but `temper-engine-shell` and `temper-engine` says
  `features = ["worker"]`. Without the feature, these names stay and
  compile, so that code written against them builds both ways:
  `Worker`, `WorkerRuntime` and `settings::Agent` (uninhabited), the
  `Service` methods `reuse_workspaces` and `dump_llm` (doing nothing),
  and `startup::with_setup_given` and the other startup functions, with
  their worker parameters. Only with it:
  `startup::{worker_part, worker_alone, WorkerPart}`, `program_sources`,
  `Service::configure_program_sources` and `Service::worker_domain`
  (its type is the worker's root's; `engine-limits` moves the method
  into an `impl Service` of `worker.rs`).

## 6. `temper-engine-shell` (tasks `engine-forge`, `engine-serve`)

`temper/crates/temper-engine-shell`, ordinary Rust on the `binary`
profile. It depends on `temper-service` through the workspace's entry,
with no feature of its own.

- `forge` (task `engine-forge`): the forge's commands, as
  `temper/src/forge.rs`, `forge_sign_in.rs` and `setup.rs` hold them:

  ```rust
  pub fn run(binary: &str, directories: &Directories, arguments: &[OsString], ssl_cert_file: Option<PathBuf>) -> u8;
  pub fn configure(row: &Row, arguments: &[Box<[u8]>], environment: &[EnvironmentEntry], working: &[u8]) -> Result<Command, String>; // the `forge` and `internal forge-child` rows' arguments
  pub enum Command { Forge { directories, arguments, ssl_cert_file }, ForgeChild { parent, signal, program, arguments } }
  pub fn command(binary: &str, command: Command) -> Result<u8, String>;
  pub const FORGE: Argument;
  pub const FETCH: Row; pub const PREPARE: Row; pub const RUN: Row; pub const REMOVE: Row;
  pub const SOURCE: Row; pub const SIGN_IN: Row; pub const FORGE_CHILD: Row;
  pub fn reads_document(row: &Row) -> bool; // every forge row but REMOVE and FORGE_CHILD
  ```

  `binary` is the running binary's name, `temper` or `temper-engine`: the
  lines and hints these commands print name it (`temper-engine forge
  fetch: …`, `run temper-engine forge prepare …`). `temper`'s table holds
  these constants in place of its own rows of the same words.
- `serve` (task `engine-serve`): serve's host and start, as
  `temper/src/lib.rs` (`serve`, `prepare*`, `operations_in*`,
  `prepare_in*` and their helpers), `host.rs`, `restart_fault.rs`,
  `bundle.rs`, `github_logs.rs`, `google_runtime.rs` and
  `accounts_runtime.rs` hold them:

  ```rust
  /// The worker's half of a start, which a binary that holds the worker gives.
  pub trait WorkerHalf {
      /// A worker alone's service, whole.
      fn alone(&self, directories: &Directories, bytes: &[u8], owned: &mut Vec<Fd>) -> Result<Service, String>;
      /// A standalone document's worker inputs to startup, before the service is made.
      fn start(&self, named: &startup::Named, directories: &Directories, owned: &mut Vec<Fd>) -> Result<WorkerStart, String>;
      /// After startup: the worker's program sources, its launcher placed, workspace reuse and the LLM dump.
      fn configure(&self, service: &mut Service, agent: &settings::Agent, named: &startup::Named, directories: &Directories) -> Result<(), String>;
  }
  pub struct WorkerStart {
      pub agent: settings::Agent,
      pub runtime: WorkerRuntime,
      pub program_login: Option<Box<[u8]>>,
  }
  pub mod printed;                 // as temper/src/printed.rs
  pub fn program_environment(directories: &Directories, environment: &[EnvironmentEntry], working: &[u8], word: &[u8]) -> Box<[u8]>;
  pub fn run(binary: &str, directories: &Directories, boxed: bool, worker: Option<&dyn WorkerHalf>) -> Result<Exit, String>;
  pub fn prepare_in(directories: &Directories, worker: Option<&dyn WorkerHalf>) -> Result<Service, String>;
  pub fn prepare_in_with(directories: &Directories, given: &stock::Given, worker: Option<&dyn WorkerHalf>) -> Result<Service, String>;
  pub fn operations_in(directories: &Directories, config: &[u8], worker: Option<&dyn WorkerHalf>) -> Result<u32, String>;
  pub fn boxed_operations_in(directories: &Directories, config: &[u8], worker: Option<&dyn WorkerHalf>) -> Result<u32, String>;
  pub fn operations_in_with(/* as temper's today */, worker: Option<&dyn WorkerHalf>) -> Result<u32, String>;
  pub use host::{Host, ending};
  pub mod bundle;
  ```

  The names bind; an argument list may change to what the moved code
  needs, said in the report. Given no `WorkerHalf`, a document with a
  worker member is refused as `temper-service` refuses it. `binary` names
  the binary in serve's lines (`temper-engine serve: …`). `temper` gives
  its own `WorkerHalf` (its `agent.rs`, `claude_code.rs`,
  `worker_alone.rs` and `program_login` line stay in `temper`) and keeps
  `temper::prepare_in`, `prepare_in_with`, `operations_in`,
  `boxed_operations_in`, `operations_in_with`, `Host`, `service_ending`,
  `bundle`, `github_logs` and `temper::directories::program_environment`
  as wrappers or re-exports, and `printed` as a re-export, so that no
  reader outside `temper/src` changes.

## 7. `temper-engine` (task `engine-binary`)

`temper/crates/temper-engine`, a binary package on the `binary` profile:
`src/main.rs` and its modules, no library; `tests/suite/main.rs`, one test
binary, running the built binary through `skein_scratch::paths::binary("temper-engine")`.
Its table, `ENGINE: Table = Table { binary: "temper-engine", rows: &ROWS }`,
holds `HELP`, its own `serve` row, `versions`, the forge rows and
`FORGE_CHILD`, in that order by `Who`.

## 8. The boundary test (task `engine-closure`)

`tests/boundary/tests/binaries.rs`, in `fabrig-boundary`:

```rust
/// A binary's package, the crates its closure must not hold, and its exceptions.
struct Binary { package: &'static str, excluded: &'static [&'static str], exceptions: &'static [(&'static str, &'static str)] }
const BINARIES: &[Binary] = &[Binary { package: "temper-engine", … }];
```

An `excluded` entry ending in `-` is a prefix; any other, a crate's name.
Each exception is `(crate, reason)`. A plan adding a binary adds its row.
