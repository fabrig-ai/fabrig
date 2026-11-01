## 1. `temper-box`: groups, steps, pipe, directories and cache

Read-only survey; no builds or tests run. Paths below are repository-relative; `:N` identifies a source line. Reverse-dependency inventories are file-level lists for the named shapes, including constructors, readers and test fixtures.

**Current group representation.** `temper/crates/temper-forge-setup/src/step.rs:10–58` defines:

```rust
pub enum Stage { Fetch, Prepare, Run }

pub struct Group {
    pub forge: u16,
    pub label: Box<[u8]>,
    pub recorded: bool,
    pub before: u32,
    pub steps: Box<[(Stage, Step)]>,
}
pub struct Steps {
    pub always: Box<[Step]>,
    pub asked: Box<[Group]>,
}
pub enum Kind { Once, Long, Link }

pub struct Step {
    name: Box<[u8]>,
    arguments: Box<[Box<[u8]>]>,
    kind: Kind,
    ready: Option<core::net::SocketAddr>,
    listeners: Box<[core::net::SocketAddr]>,
    started_notice: Box<[u8]>,
}
```

Relevant `Step` methods, in the same file:

```rust
// :62
pub fn new(name: Box<[u8]>, arguments: Box<[Box<[u8]>]>, kind: Kind) -> Self;
// :77
pub fn listening(mut self, addrs: Box<[core::net::SocketAddr]>) -> Self;
// :93
pub fn with_ready(mut self, address: core::net::SocketAddr) -> Self;
```

`listening` excludes port zero (`:78–84`).

**How the document becomes steps.**

1. `temper/crates/temper/src/box_command.rs:41` — `run(directories: &Directories, echo: Echo) -> Result<Exit, String>` obtains the document, then calls `run_configuration`.
2. `temper/crates/temper/src/setup.rs:17` — `check_with(...) -> Result<Deployment, String>` reads named forge/identity/listener configuration, checks `application.setup`, and reads `application.demo`.
3. `temper/crates/temper/src/box_command.rs:63–85` calls:

   ```rust
   temper_fake::steps(
       &checked.setup,
       demo.as_ref(),
       &directories.flags(),
       serve.into_boxed_slice(),
       cfg!(target_os = "linux"),
       first,
   )
   ```

4. `temper/crates/temper-fake/src/steps.rs:21` first calls `temper_forge_setup::steps`, adds fake steps, and adjusts group insertion points.
5. `temper/crates/temper-box/src/host.rs:175` — `Service::new(limits: &Limits, steps: Steps, settings: Settings) -> Result<Self, Unfit>` flattens and retains the complete step set.

There is no later group-definition loading today.

**Owned Forgejo steps.** `temper/crates/temper-forge-setup/src/step.rs:144`:

```rust
pub fn steps(
    setup: &Setup,
    flags: &[Box<[u8]>],
    listeners: Box<[core::net::SocketAddr]>,
    forgejo_listens: bool,
) -> Steps;
```

At `:155–189`, each `Mode::Owned(o)` produces, in order:

| Condition | Command | Stage / kind |
|---|---|---|
| `o.fetch.forgejo || o.fetch.runner` | `temper forge fetch DIRS N` | `Fetch` / `Once` |
| Always | `temper forge prepare DIRS N` | `Prepare` / `Once` |
| Always | `temper forge run DIRS N forgejo` | `Run` / `Long`, readiness at `o.listen` |
| `o.runner.is_some()` | `temper forge run DIRS N runner` | `Run` / `Long` |

With `Entry.asked`, these form:

```rust
Group {
    forge: e.forge,
    label: /* "Forge N (Forgejo)" */,
    recorded: true,
    before: out.len(),
    steps: group.into_boxed(),
}
```

Without `asked`, their steps enter `Steps.always`.

At `:192–200`, legacy managed entries with a file-backed admin token additionally produce an always-running-chain `forge reconcile` once step. Owned and fake entries reconcile inside serve.

At `:202–213`, the final ordinary step is `temper internal serve DIRS --box`, `Kind::Long`.

**Fake groups.** `temper/crates/temper-fake/src/steps.rs:21`:

```rust
pub fn steps(
    setup: &Setup,
    demo: Option<&Demo>,
    flags: &[Box<[u8]>],
    listeners: Box<[core::net::SocketAddr]>,
    forgejo_listens: bool,
    first: Box<[Step]>,
) -> Steps;
```

- Non-demo `Mode::Fake`: `forge prepare DIRS N`, then `fake forge DIRS N`, both in `always` (`:49–62`).
- Demo: one asked group labelled `Playground`, `recorded: false`, containing fake LLM, fake preparation, fake forge (`:73–100`).
- `first` precedes ordinary steps and every asked group (`:34–48`, `:102–104`). Fake-backend peers use this prefix.

**Flattening and dormancy.** `temper/crates/temper-box/src/layout.rs:6–18`:

```rust
pub struct Asked {
    pub forge: u16,
    pub recorded: bool,
    pub stage: Stage,
}
pub struct Placed {
    pub step: Step,
    pub asked: Option<Asked>,
    pub label: Option<Box<[u8]>>,
}
pub fn flatten(steps: Steps) -> Box<[Placed]>;
```

`flatten` inserts each group before its `before` index; groups sharing an insertion point retain setup order. The label is retained on the first step only (`:24–47`).

`temper/crates/temper-box/src/machine.rs:58` has:

```rust
enum Status {
    Dormant,
    Starting,
    Recording { next: Option<usize> },
    Started,
    Failing { why: Why },
    Stopping,
}
```

`Machine::new` constructs every `GroupState` as `Dormant` (`:123–159`). `Machine::start(&mut self, _now: Time, recorded: &[u16], orders: &mut Queue<Order>)` chains recorded groups into startup (`:191–205`). Unrecorded groups remain separate until asked.

**Pipe.** `temper/crates/temper-box/src/pipe.rs:42–80`:

```rust
pub enum Command {
    Start(u16),
    Stop(u16),
    Unknown,
}
pub fn command(line: &[u8]) -> Command;
```

Accepted commands are canonical decimal `start N\n` and `stop N\n`; leading zeros, overflow, missing LF and other spellings are rejected. Zero is accepted.

`temper/crates/temper-box/src/host.rs:960–962` translates them to:

```rust
Input::Ask { forge }
Input::Stop { forge }
```

Replies are `step N fetch|prepare|run`, `started N`, `stopped N`, and `failed N WHY`; formatters are in `pipe.rs:82–106`.

Policy functions:

- `machine.rs:598` — `fn ask(&mut self, forge: u16, orders: &mut Queue<Order>)`
- `machine.rs:622` — `fn stop_group(&mut self, now: Time, forge: u16, orders: &mut Queue<Order>)`
- `machine.rs:652` — `fn stop_group_next(...)`
- `machine.rs:669` — `fn advance_group(...)`

A stop closes the group in reverse, resets its states to dormant, and answers only after settlement (`:638–667`). It does **not** erase its started record or directory.

**`<data>/box/groups`.** **Not found in implementation.** No group-file schema, group-file loader, `remove N` parser variant, or group-directory removal operation exists in the surveyed box/binary/link code. These are additions proposed by reset §3–4.

**Started records.** `temper/crates/temper-box/src/records.rs:28` defines private `Records`; its relevant methods are:

```rust
// :38
pub fn new(steps: &[Placed]) -> Self;
// :59
pub fn recorded(&self) -> &[u16];
// :77
pub fn write(&mut self, forge: u16);
// :84
pub fn tick(&mut self, root: Fd, subs: &mut Queue<Submit>);
// :114
pub fn complete(&mut self, complete: Complete);
```

- It probes only groups with `Asked.recorded == true` (`:42–49`).
- Path construction is `box/started/N` (`:191–192`).
- `box` and `started` directories are created mode `0700`; empty record files mode `0600` (`:99–110`).
- Missing record means dormant; other probe errors produce diagnostics.
- Successful creation/close yields `Input::Written`; failure yields `Input::NotWritten`.
- The implementation records **after the whole asked group completes readiness/startup**, including remaining group steps (`machine.rs:669–705`). Protocol §7.1’s wording “once preparation has exited 0” at `temper/docs/design/protocol/setup.md:1050` is less precise than the implementation.

**Listeners and control descriptors.**

- `temper/crates/temper/src/box_command.rs:125–184` — `open_listeners(...)` opens listeners across `always` and `asked`, including dormant groups; duplicate listeners refuse startup.
- `temper/crates/temper-box/src/host.rs:1119–1145` — `spawn_pipes(...)` gives each child its listeners from fd 3. Serve’s request pipe follows its listeners, then its answer pipe.
- Thus the control descriptors are `3 + listener_count` and `4 + listener_count`, not invariably fd 3/4.

**Forge directory.** It is configured, not automatically derived from the number.

- `Entry.directory: Box<[u8]>`: `temper/crates/temper-forge-setup/src/setup.rs:80`.
- Relative directory resolved against Data: same file `:108–115`.
- Shipped owned forge: `"directory": "forge-3"` at `temper/crates/temper-init/template/temper.json:329`.
- Shipped fake forge: `"directory": "demo/forge"` at `:306`.
- `temper/crates/temper-forge-setup/src/place.rs:6`:

  ```rust
  pub struct Place {
      pub directory: Box<[u8]>,
      pub user: Box<[u8]>,
  }
  pub fn path(&self, file: File) -> Box<[u8]>; // :47
  pub fn work_path(&self) -> Box<[u8]>;         // :74, adds "/forgejo"
  ```

- File names are mapped at `place.rs:47–65`: `forgejo/app.ini`, `state`, `admin_token`, person password/headers, bot token files, user, webhook secret, runner files and locks.
- `temper/crates/temper-forge-setup-shell/src/files.rs:52` — `layout(place: &Place) -> io::Result<()>` creates private `home`, `forgejo`, `forgejo/repositories`, `runner`, `backups`.
- Existing `clear(place: &Place) -> io::Result<()>` (`:39`) removes only an interrupted preparation’s six named entries; it keeps backups/state and is not whole-forge removal.

**Binary cache.**

- `setup.rs:295–307` rewrites `"fetched"` to:
  - `forgejo/VERSION/forgejo`
  - `forgejo-runner/RUNNER_VERSION/forgejo-runner`
- `Setup::read` resolves these against Cache (`setup.rs:133–148`).
- `temper/crates/temper-forge-setup/src/fetch.rs:12`:

  ```rust
  pub struct Pin {
      pub origin: Origin,
      pub path: Box<[u8]>,
      pub digest: [u8; 32],
      pub file: Box<[u8]>,
  }
  pub fn pins(owned: &Owned, machine: &str) -> Result<Box<[Pin]>, Failure>; // :82
  ```

- Architecture, release URL, mirror and destination selection: `fetch.rs:82–135`.
- Existing-file hashing, streamed download and part-file effects: `fetch.rs:139` onward.
- Shell entry: `temper/crates/temper-forge-setup-shell/src/fetch.rs:98`:

  ```rust
  pub fn run(limits: &skein_io::Limits, trust: &Trust, mut machine: Fetch)
      -> Result<(), Failed>;
  ```

- Protocol contract: `temper/docs/design/protocol/setup.md:812–867`: matching existing binary is reused; differing digest refuses overwrite; incomplete `.part` is rewritten.

**Reverse dependencies: `Step` / `Stage` / `Group` / `Steps` and placed group metadata.**

```text
temper/crates/temper-forge-setup/src/lib.rs:24
temper/crates/temper-forge-setup/src/step.rs:10
temper/crates/temper-forge-setup/src/tests/steps.rs:8
temper/crates/temper-fake/src/steps.rs:5
temper/crates/temper-fake/src/tests/steps.rs:6
temper/crates/temper-fake/src/backends.rs:12
temper/crates/temper-fake/src/backends/tests.rs:74
temper/crates/temper-box/src/layout.rs:4
temper/crates/temper-box/src/machine.rs:8
temper/crates/temper-box/src/host.rs:23
temper/crates/temper-box/src/pipe.rs:6
temper/crates/temper-box/src/records.rs:3
temper/crates/temper-box/src/logs.rs:3
temper/crates/temper-box/src/terminal.rs:3
temper/crates/temper-box/src/tests/boundary.rs:30
temper/crates/temper-box/src/tests/fixtures.rs:8
temper/crates/temper-box/src/tests/host.rs:476
temper/crates/temper-box/src/tests/logs.rs:9
temper/crates/temper-box/src/tests/machine.rs:216
temper/crates/temper-box/src/tests/pipe.rs:5
temper/crates/temper-box/src/tests/terminal.rs:36
temper/crates/temper-box/tests/real_loop.rs:19
temper/crates/temper/src/box_command.rs:51
temper/crates/temper/src/fake_backends.rs:70
temper/crates/temper/src/fake_backends/tests.rs:230
```

Direct users in `skein/`, `smith/`, `jig/`: **not found**. The box’s dependency/import restriction is itself tested by `temper-box/src/tests/boundary.rs:4,56`.

**Reverse dependencies: started-record path/semantics.**

```text
temper/crates/temper-box/src/records.rs:191
temper/crates/temper-box/src/host.rs:179
temper/crates/temper-box/src/machine.rs:191
temper/crates/temper-box/src/tests/host.rs:200
temper/crates/temper-box/src/tests/machine.rs:214
temper/crates/temper/tests/support/first_repository.rs:554
temper/crates/temper/tests/qa_all/qa_first_run.rs:110
temper/crates/temper/tests/qa_all/qa_forge_handover.rs:547
```

Directory/setup and cache consumers are included in question 3’s configuration inventory and question 5’s process-test inventory.

**Tests nearest these mechanisms.**

| File and line | Exact test |
|---|---|
| `temper/crates/temper-forge-setup/src/tests/steps.rs:113` | `asked_groups_keep_their_place_and_run_only_their_processes` |
| Same `:162` | `forgejo_takes_its_listener_only_when_the_shell_can_identify_signal_senders` |
| `temper/crates/temper-fake/src/tests/steps.rs:23` | `documents_give_the_box_its_whole_ordered_steps` |
| Same `:194` | `first_steps_precede_fakes_and_both_groups_without_entering_the_playground` |
| `temper/crates/temper-box/src/tests/pipe.rs:28` | `only_canonical_start_and_stop_lines_are_known` |
| `temper/crates/temper-box/src/tests/machine.rs:214` | `asked_groups_follow_their_chains_records_failures_and_stop` |
| Same `:788` | `an_owned_version_gates_the_runner_and_record_on_ask_and_restart` |
| Same `:892` | `an_unrecorded_group_runs_only_when_asked_even_with_a_stale_record` |
| Same `:989` | `a_group_stop_signals_in_reverse_and_answers_only_after_all_closes_then_can_start_again` |
| `temper/crates/temper-box/src/tests/host.rs:200` | `records_are_probed_before_start_and_written_private_or_reported` |
| Same `:543` | `an_unrecorded_group_neither_probes_nor_writes_and_waits_at_every_box_start` |
| Same `:579` | `a_pipe_stop_closes_only_its_group_keeps_the_record_and_can_be_started_again` |
| Same `:721` | `a_dormant_listener_queues_connections_and_is_given_again_after_restart` |

## 2. Who writes the pipe from serve

**`temper-service` constructs and writes commands. `temper` adopts the descriptors.**

`temper/crates/temper-service/src/route/box_link.rs:43` defines:

```rust
pub(crate) struct Link {
    pub(crate) write: Option<Box<[u8]>>,
    ended: bool,
    asked: Option<Ask>,
    held: Option<u16>,
    stop: Option<u16>,
    line_bytes: u32,
    partial: List<u8>,
    skipping: bool,
}
enum Ask { Start(u16), Stop(u16) }
```

Entry points:

```rust
// :57
pub(crate) fn new(line_bytes: u32, _piped: bool) -> Link;
// :70
pub(crate) fn ask(&mut self, forge: u16, request: Token, out: &mut Queue<Out>);
// :90
pub(crate) fn stop(&mut self, forge: u16, out: &mut Queue<Out>);
// :108
pub(crate) fn read(&mut self, input: &[u8], out: &mut Queue<Out>);
// :147
pub(crate) fn closed(&mut self, out: &mut Queue<Out>);
```

`start` and `stop` formatters are `box_link.rs:167` and `:163`. `Out::Write(Box<[u8]>)` is defined at `:22`; `drain(engine: &mut Engine, out: &mut Queue<Out>)` retains it in `engine.box_link.write` at `:302–309`.

Triggers in `temper/crates/temper-service/src/route.rs:15–36`:

```rust
root::Request::StopGroup { forge } => engine.box_link.stop(...),
root::Request::SetUpGroup { forge, request } => engine.box_link.ask(...),
root::Request::SetUp { forge, request, org, repository, playground } => {
    crate::setup::lifecycle::first(...);
    engine.box_link.ask(...);
}
```

Actual stream write: `temper/crates/temper-service/src/system.rs:1039–1083`, `fn box_io(service: &mut Service)`, sends:

```rust
io::Request::Stream {
    stream: pipe.requests,
    down: Down::Send(line),
}
```

`Service::adopt_box(&mut self, requests: Fd, answers: Fd, line_bytes: u32) -> Result<(), BoxRefusal>` is `system.rs:495`.

Binary wiring: `temper/crates/temper/src/lib.rs:575–606`, `fn serve(directories: &Directories, boxed: bool) -> Result<Exit, String>`, determines descriptors after inherited listeners, validates pipes, and calls `service.adopt_box`.

**Reverse dependencies: `Link`, its outputs and pipe adoption.**

```text
temper/crates/temper-service/src/route/box_link.rs:43
temper/crates/temper-service/src/route.rs:15
temper/crates/temper-service/src/engine.rs:19
temper/crates/temper-service/src/system.rs:495
temper/crates/temper-service/src/tests/box_link.rs:1
temper/crates/temper-service/src/tests/qa_box_link.rs:11
temper/crates/temper-service/src/tests/system.rs:176
temper/crates/temper-service/src/tests/fixtures/system.rs:75
temper/crates/temper-service/src/tests/setup.rs:1253
temper/crates/temper-service/src/tests/setup_create.rs:251
temper/crates/temper/src/lib.rs:563
temper/crates/temper/src/tests.rs:468
```

**Reverse dependencies if `StopGroup` / `SetUpGroup` are changed or replaced.** These include exhaustive test-world dispatchers, even where the arm only rejects/ignores the request:

```text
temper/crates/temper-engine-domain-forge/src/boundary.rs:1376
temper/crates/temper-engine-domain-forge/src/demo.rs:110
temper/crates/temper-engine-domain-forge/src/tests.rs:346
temper/crates/temper-engine-domain-forge/src/tests/demo.rs:277
temper/crates/temper-engine-domain/src/boundary.rs:540
temper/crates/temper-engine-domain/src/route/forge_route.rs:2993
temper/crates/temper-engine-domain/src/domain.rs:749
temper/crates/temper-engine-domain/src/decision.rs:616
temper/crates/temper-engine-domain/src/tests/resume.rs:483
temper/crates/temper-engine-domain/src/tests/setup.rs:84
temper/crates/temper-service/src/route.rs:15
temper/crates/temper-service/src/tests/engine.rs:480
temper/crates/temper-service/src/tests/fixtures.rs:745
temper/crates/temper-service/src/tests/setup_github.rs:97
temper/tests/client/src/front.rs:1379
temper/tests/client/src/translate.rs:243
temper/tests/client/src/translate/forge.rs:418
temper/tests/system/src/forge.rs:422
temper/tests/system/src/composed.rs:741
temper/tests/engine/domain/src/direct.rs:207
temper/tests/engine/domain/src/escalation.rs:673
temper/tests/engine/domain/src/forge_routes.rs:1132
temper/tests/engine/domain/src/protocol.rs:294
temper/tests/engine/domain/src/roles.rs:532
temper/tests/engine/domain/src/walking.rs:786
temper/tests/engine/domain/tests/suite/demo.rs:199
temper/tests/engine/forge/src/world.rs:781
temper/tests/engine/forge/tests/suite/simulation.rs:821
temper/tests/engine/protocol-forge/src/protocol.rs:507
temper/tests/engine/protocol-forge/tests/suite/protocol.rs:510
```

Users in the other three trees: **not found**.

**Tests.**

- `temper/crates/temper-service/src/tests/box_link.rs:42` — `an_ask_writes_start_and_sets_up_with_no_step`
- `:49` — `an_ask_is_held_until_started_or_failed_and_a_second_held_ask_is_busy`
- `:224` — `lines_split_and_joined_across_reads_are_read_alike`
- `:279` — `stop_writes_its_line_and_stopped_tells_the_root_nothing`
- `:294` — `stop_is_held_behind_start_and_failed_is_only_logged`
- `:306` — `stop_cancels_a_held_demo_start_and_preserves_another_held_start`
- `temper/crates/temper-service/src/tests/system.rs:172` — `the_box_pipes_carry_no_service_data_and_close_on_drain_or_abort`
- `temper/crates/temper-service/src/tests/qa_box_link.rs:9,63,117` — invalid lines, chunking and byte-bound QA cases.

## 3. `temper-forge-setup`, shell, document inputs and secrets

**Document-owned setup shape.** `temper/crates/temper-forge-setup/src/setup.rs:14–90`:

```rust
pub struct Account {
    pub login: Box<[u8]>,
    pub email: Box<[u8]>,
}
pub struct Runner {
    pub binary: Box<[u8]>,
    pub labels: Box<[Box<[u8]>]>,
    pub capacity: u32,
    pub path: Box<[u8]>,
}
pub struct Owned {
    pub public: Option<Origin>,
    pub fetch: Fetch,
    pub forgejo: Box<[u8]>,
    pub runner: Option<Runner>,
    pub listen: SocketAddr,
    pub person: Account,
}
pub struct Fetch {
    pub forgejo: bool,
    pub runner: bool,
    pub mirror: Option<Mirror>,
}
pub struct Fake {
    pub listen: SocketAddr,
    pub git: Box<[u8]>,
    pub owner: Box<[u8]>,
    pub repositories: Box<[Repository]>,
}
pub enum Mode {
    Managed { admin_token: Option<Box<[u8]>> },
    Owned(Owned),
    Fake(Fake),
}
pub struct Entry {
    pub forge: u16,
    pub directory: Box<[u8]>,
    pub bot: Account,
    pub orgs: Box<[Box<[u8]>]>,
    pub mode: Mode,
    pub asked: bool,
}
pub struct Setup {
    pub entries: Box<[Entry]>,
}
```

`Repository { name: Box<[u8]>, landing: Box<[u8]> }` is at `:57`; `Origin { host: Box<[u8]>, port: u16 }` at `:51`; `Mirror { forgejo: Origin, runner: Origin }` at `:45`.

```rust
// setup.rs:93
pub fn read(
    d: &Document,
    at: u32,
    limits: &Limits,
    directories: &config::Directories,
    problems: &mut Problems,
) -> Option<Self>;

// :161
pub fn entry(&self, number: u16) -> Option<&Entry>;
```

Parsing:

- Owned members: `setup.rs:277–328`.
- Fake members/repositories: `:374–415`.
- Mode forms: `:417–449`.
- Entry members and `"start": "asked"`: `:451–530`. `asked` is accepted only for owned entries.
- String `"managed"` gives `admin_token: None`, no directory.
- Object managed mode gives a configured admin-token **path**, `Some(...)`.
- Path resolution: directory/admin-token/fake git against Data, owned binaries against Cache (`:106–151`).

`temper/crates/temper-forge-setup/src/check.rs:21–46` adds:

```rust
pub struct Members {
    pub forges: Box<[Forge]>,
    pub listeners: Box<[SocketAddr]>,
}
pub struct Checked {
    pub fetch_bytes: u64,
    pub records_only: Box<[u16]>,
    pub setup: Setup,
    pub limits: Limits,
}
```

`checked(document, at, directories, members_value, listener, overrides) -> Result<Checked, Problems>` is at `:181`. It joins setup entries to configured forge numbers, validates listener/API/git correspondence, file-backed credential paths, uniqueness and limits (`:64–169`).

**The shell does not parse `temper.json` itself.** The binary reads/checks it:

- `temper/crates/temper/src/forge.rs:153` — `start(...) -> Result<u8, Refused>`.
- It reopens `directories.read()`, calls `setup::check`, finds `checked.setup.entry(number)`, then supplies typed machine/settings values (`:184–255`).
- `temper/crates/temper-forge-setup-shell/src/prepare.rs:45`:

  ```rust
  pub fn run(limits: &skein_io::Limits, place: &Place, mut machine: Prepare)
      -> Result<(), Failed>;
  ```

- Preparation machine: `temper/crates/temper-forge-setup/src/prepare.rs:172`:

  ```rust
  pub fn new(owned: Owned, place: Place, allowed_host: Option<IpAddr>) -> Self;
  ```

**Given values for reconcile.** `temper/crates/temper-forge-setup/src/given.rs:103`:

```rust
pub fn values(
    entry: &crate::setup::Entry,
    listener: Option<core::net::SocketAddr>,
    limits: &skein_forge_setup::limits::Limits,
) -> skein_forge_setup::given::Given;
```

Copies forge number, bot login/email, owned person as `org_owner`, listener and limits; supplies names `temper`, `temper-admin`/`admin`, `temper-api`, `temper-git`. Current `Given.orgs` and `Given.adopted` are initially empty (`:108–129`); runtime reconcile supplies restored adopted IDs.

**Admin token lifecycle.**

1. Owned offline preparation builds `spec::admin_token(owned: &Owned, place: &Place, name: &[u8]) -> Spec` at `temper/crates/temper-forge-setup/src/spec.rs:163`.
2. It runs `admin user generate-access-token`, with `--raw` and scopes `write:admin,read:user,write:organization,write:repository` (`:168–177`).
3. `prepare::step` takes the command’s first output line and writes `File::AdminToken` (`prepare.rs:616–624`), the private handover file.
4. Serve takes it through `host::Plan::TakeOver { forge, admin_token }`; skein’s `TakeOver` writes `Item::AdminToken` at `skein/crates/skein-forge-setup/src/take_over.rs:101`.
5. Only after the durable write and completed takeover does serve request deletion of the handover file (`temper/crates/temper-service/src/setup/lifecycle.rs:323–327`).
6. On ordinary restart with no handover, `reconcile_local` reads the retained admin record (`lifecycle.rs:899–964`).
7. An asked group’s `started` initiates a fresh handover read (`:1185–1206`).

Runtime file interfaces are `take_read` (`:1208`), `read` (`:1232`), `take_removal` (`:1266`) and `removed` (`:1285`). `settings::Setup` retains `entry` and `handover: Option<Secret>` at `temper/crates/temper-service/src/settings.rs:130`.

**Durable service reconcile writes.**

Skein host boundary, `skein/crates/skein-forge-setup/src/host/boundary.rs:128–141`:

```rust
pub enum Answer {
    Read(Option<Secret>),
    Written,
}
pub enum Need {
    Read(Item),
    Write { item: Item, bytes: Secret },
}
pub enum Event {
    Need { run: Token, need: Need },
    Done { run: Token, outcome: Outcome },
    Stop { run: Token, failure: Failure },
    Closed,
}
```

`temper/crates/temper-service/src/setup.rs:484–499` routes `Read` to retained rows and `Write` to:

```rust
// setup/lifecycle.rs:267
pub(super) fn keep(
    engine: &mut crate::engine::Engine,
    forge: u16,
    item: Item,
    bytes: &[u8],
    purpose: Purpose,
);
```

`keep` encodes the record, calculates its number, and issues `Secrets::Keep { owner: SecretOwner::Application, ... }` (`:267–289`).

On `StoreEvent::Kept`, `setup.rs:523–527` inserts the retained row, then `lifecycle::kept`; `Purpose::Host(run)` answers `host::Answer::Written` only then (`lifecycle.rs:531`). Writes therefore persist incrementally.

**Bot tokens and webhook secret.**

- Bot API/git token reconciliation: `skein/crates/skein-forge-setup/src/reconcile/tokens.rs:37–148`; processes `Item::ApiToken`, then `Item::GitToken`, validates/reuses existing credentials and emits `Need::Write` for new values.
- Bot user record: `skein/crates/skein-forge-setup/src/reconcile.rs:250`.
- Webhook shared secret: `skein/crates/skein-forge-setup/src/access.rs:80–108` reads it; if absent, draws 32 bytes, hex-encodes, writes `Item::WebhookSecret`, waits for `Written`, then configures access/hooks. This machinery is shared by reconcile/adopt/create.
- GitHub does not retain a webhook secret; see question 6.

**Legacy standalone reconcile remains file-backed.**

`temper/crates/temper/src/forge.rs:312`:

```rust
pub fn reconcile_settings(
    named: &temper_service::startup::Named,
    checked: &temper_forge_setup::check::Checked,
    number: u16,
) -> Result<temper_forge_setup_shell::reconcile::Settings, String>;
```

It rejects records-only local forges, directing them to serve (`:333–334`). The standalone shell:

```rust
// temper-forge-setup-shell/src/reconcile.rs:486
pub fn reconcile(settings: Settings, clock: &Clock) -> Result<(), Refused>;
```

- `Need::Read` reads private files (`:269–290`).
- `Need::Write` writes files, converting `User` between decimal file and eight-byte kit value (`:292–308`).
- Successful completion ensures a file-backed webhook secret when a listener exists (`:154–173`, `:314`).
- `files::write_private(path: &Path, bytes: &[u8]) -> io::Result<()>` writes `.partial`, syncs, renames and syncs parent (`temper-forge-setup-shell/src/files.rs:10–22`).

**Credential records.**

`temper/crates/temper-protocol-records/src/lib.rs:23–54`:

| Code | Exact variant |
|---:|---|
| 1 | `Item::AdminToken` |
| 2 | `Item::ApiToken` |
| 3 | `Item::GitToken` |
| 4 | `Item::User` |
| 5 | `Item::WebhookSecret` |
| 6 | `Item::AppId` |
| 7 | `Item::AppSlug` |
| 8 | `Item::AppKey` |
| 9 | `Item::Accepted` |
| 10 | `Item::Origin` |

```rust
pub fn number(forge: u16, item: Item) -> u64 {
    (u64::from(FAMILY_FORGE) << 48)
        | (u64::from(forge) << 16)
        | u64::from(item.code())
}
pub fn parse(number: u64) -> Option<(u16, u16, Item)>;
```

`FAMILY_FORGE = 1`; `CARRIED = 1 << 63`. `Pasted` is not a durable `Item` variant.

**Format discrepancy:** `temper/docs/design/protocol/credentials.md:137` describes a one-byte version followed by kit bytes. Current code uses generated `v1::Stored` schema records:

- `temper/crates/temper-protocol-records/schema/secrets-v1.schema:7–30`
- `temper/crates/temper-protocol-records/src/codec.rs:14–30`

```rust
pub fn encode(item: Item, bytes: &[u8]) -> Result<Vec<u8>, Unreadable>;
pub fn decode(item: Item, bytes: &[u8]) -> Result<Secret, Unreadable>;
```

The kit boundary still uses little-endian IDs; the generated stored installation list is big-endian (`codec.rs:97–104`). A removal/change must use the codec, not assume the prose byte layout.

`Rows` and access construction are in `temper-protocol-records/src/forge.rs:53–119,165`; the service filters rows to configured forge numbers in `temper-service/src/forge/records.rs:17–37`. Unconfigured values are discarded from memory and their forge numbers logged.

Whole-forge credential erasure: **not found**. Existing `setup::public::erase` explicitly asserts `number >= CARRIED` (`public.rs:370–380`): it erases carried values, not durable forge records.

**Reverse dependencies: setup configuration (`Setup`, `Entry`, `Mode`, `Owned`, `Fake`, their directory/listener/binary fields).**

```text
temper/crates/temper-forge-setup/src/lib.rs:24
temper/crates/temper-forge-setup/src/setup.rs:93
temper/crates/temper-forge-setup/src/check.rs:64
temper/crates/temper-forge-setup/src/given.rs:103
temper/crates/temper-forge-setup/src/step.rs:144
temper/crates/temper-forge-setup/src/fetch.rs:82
temper/crates/temper-forge-setup/src/prepare.rs:172
temper/crates/temper-forge-setup/src/run.rs:84
temper/crates/temper-forge-setup/src/ini.rs:2
temper/crates/temper-forge-setup/src/runner.rs:5
temper/crates/temper-forge-setup/src/spec.rs:49
temper/crates/temper-forge-setup/src/sign_in.rs:4
temper/crates/temper-forge-setup/src/tests/fixtures.rs:98
temper/crates/temper-forge-setup/src/tests/read.rs:6
temper/crates/temper-forge-setup/src/tests/managed.rs:6
temper/crates/temper-forge-setup/src/tests/members.rs:57
temper/crates/temper-forge-setup/src/tests/steps.rs:4
temper/crates/temper-forge-setup/src/tests/fetch.rs:10
temper/crates/temper-forge-setup/src/tests/run.rs:108
temper/crates/temper-forge-setup-shell/src/prepare.rs:5
temper/crates/temper-forge-setup-shell/src/files.rs:8
temper/crates/temper-forge-setup-shell/src/reconcile.rs:11
temper/crates/temper-forge-setup-shell/tests/support/mod.rs:8
temper/crates/temper-forge-setup-shell/tests/suite/prepare.rs:8
temper/crates/temper-fake/src/demo.rs:9
temper/crates/temper-fake/src/steps.rs:5
temper/crates/temper-fake/src/forge.rs:53
temper/crates/temper-fake/src/tests.rs:595
temper/crates/temper-fake/src/tests/demo.rs:8
temper/crates/temper-fake/src/tests/steps.rs:6
temper/crates/temper-service/src/settings.rs:130
temper/crates/temper-service/src/startup.rs:21
temper/crates/temper-service/src/startup/application.rs:1045
temper/crates/temper-service/src/startup/application/demo.rs:66
temper/crates/temper-service/src/startup/application/sign_in/forges.rs:53
temper/crates/temper-service/src/setup/settings.rs:104
temper/crates/temper-service/src/clients/bundle.rs:342
temper/crates/temper-service/src/tests/fixtures.rs:1031
temper/crates/temper-service/src/tests/startup.rs:464
temper/crates/temper-service/src/tests/sign_in_configuration.rs:273
temper/crates/temper-service/src/tests/setup.rs:764
temper/crates/temper-service/src/tests/setup_adoption.rs:3
temper/crates/temper-service/src/tests/setup_admin.rs:19
temper/crates/temper-service/src/tests/setup_scopes.rs:5
temper/crates/temper/src/setup.rs:17
temper/crates/temper/src/box_command.rs:63
temper/crates/temper/src/forge.rs:185
temper/crates/temper/src/forge/tests.rs:18
temper/crates/temper/src/forge_sign_in.rs:9
temper/crates/temper/src/fake.rs:348
temper/crates/temper/src/fake/check.rs:23
temper/crates/temper/src/fake/check/tests.rs:336
temper/crates/temper/src/fake_backends.rs:558
temper/crates/temper/src/fake_backends/import.rs:24
temper/crates/temper/src/fake_backends/tests.rs:756
temper/crates/temper/src/init/tests.rs:326
temper/crates/temper/src/tests.rs:178
temper/tests/one-process/src/restart_control.rs:37
temper/tests/one-process/src/stories.rs:318
temper/tests/one-process/src/fixtures.rs:138
temper/tests/real-loop/src/fixtures.rs:59
temper/tests/client-browser/src/configuration.rs:88
```

Additional **document constructors/readers**, without direct Rust type names:

```text
temper/crates/temper-init/template/temper.json:303
temper/crates/temper-init/src/tests/documents.rs:160
temper/crates/temper-fake/src/backends.rs:448
temper/crates/temper-fake/src/backends/tests.rs:313
temper/docs/examples/owned.json:247
temper/crates/temper/tests/fixtures/cluster/engine.json:209
temper/crates/temper/tests/support/github/engine.json:196
temper/crates/temper/tests/support/configuration.rs:69
temper/crates/temper/tests/support/owned.rs:28
temper/crates/temper/tests/support/case.rs:214
temper/crates/temper/tests/support/first_repository.rs:484
temper/crates/temper/tests/support/forgejo_elsewhere.rs:160
temper/crates/temper/tests/support/csv_repair.rs:121
temper/crates/temper/tests/support/account_file_qa.rs:31
temper/crates/temper/tests/support/account_move.rs:46
temper/crates/temper/tests/support/account_move_qa.rs:51
temper/crates/temper/tests/process.rs:2110
temper/crates/temper/tests/suite/startup.rs:338
temper/crates/temper/tests/suite/forge.rs:363
temper/crates/temper/tests/suite/fake.rs:450
temper/crates/temper/tests/suite/box_listeners.rs:54
temper/crates/temper/tests/qa_all/qa_adoption.rs:203
temper/crates/temper/tests/qa_all/qa_first_repository.rs:19
temper/crates/temper/tests/qa_all/qa_forge_handover.rs:32
temper/crates/temper/tests/qa_all/qa_forge_setup.rs:45
temper/crates/temper-service/src/tests/fixtures/startup.rs:15
```

**Reverse dependencies: durable forge-record API/rows.**

```text
temper/crates/temper-protocol-records/src/lib.rs:23
temper/crates/temper-protocol-records/src/codec.rs:14
temper/crates/temper-protocol-records/src/forge.rs:53
temper/crates/temper-service/src/forge/records.rs:5
temper/crates/temper-service/src/forge/tokens.rs:150
temper/crates/temper-service/src/forge.rs:471
temper/crates/temper-service/src/engine.rs:22
temper/crates/temper-service/src/store.rs:336
temper/crates/temper-service/src/setup.rs:15
temper/crates/temper-service/src/setup/lifecycle.rs:12
temper/crates/temper-service/src/setup/public.rs:188
temper/crates/temper-service/src/startup/checks.rs:1021
temper/crates/temper-service/src/lib.rs:14
temper/crates/temper-service/src/tests/fixtures.rs:1059
temper/crates/temper-service/src/tests/limits.rs:292
temper/crates/temper-service/src/tests/engine.rs:548
temper/crates/temper-service/src/tests/workers.rs:71
temper/crates/temper-service/src/tests/startup.rs:1448
temper/crates/temper-service/src/tests/forge.rs:594
temper/crates/temper-service/src/tests/setup.rs:6
temper/crates/temper-service/src/tests/setup_lifecycle.rs:548
temper/crates/temper-service/src/tests/setup_adoption.rs:124
temper/crates/temper-service/src/tests/setup_admin.rs:145
temper/crates/temper-service/src/tests/setup_scopes.rs:103
temper/crates/temper-service/src/tests/setup_bound_webhook.rs:47
temper/crates/temper-service/src/tests/setup_reachable.rs:92
temper/crates/temper-service/src/tests/setup_github.rs:329
temper/crates/temper-client-domain/src/forge/forges.rs:803
temper/testing/temper-github-app/src/lib.rs:4
temper/tests/codecs/tests/suite/records.rs:4
temper/tests/codecs/tests/suite/fuzzy/records.rs:3
```

The client-domain consumer uses the **pasted-value** codec. Direct consumers of these temper-specific types in `skein/`, `smith/`, `jig/`: **not found**. Skein owns the reusable setup machines and `item::Item`; those are distinct from temper’s persisted-record facade.

The in-memory first-repository `Intent` is declared at `temper-service/src/setup/lifecycle.rs:53`; its direct constructors/readers are that file (`:35,67,1330`) and `temper-service/src/setup/public.rs:89`. `first(...)` stores it only in `Local.first` (`lifecycle.rs:1340–1342`); persistence of this intent is **not found**.

**Nearest tests.**

- `temper-service/src/tests/setup.rs:376` — `written_is_answered_only_after_kept_and_the_routing_rows_follow_each_keep`
- `:410` — `a_failed_keep_stops_the_engine_and_never_answers_written`
- `:1274` — `takeover_waits_for_kept_done_and_correlated_removal_before_restored_reconcile`
- `:1317` — `restart_always_takes_a_present_handover_even_when_the_record_differs`
- `:1353` — `an_asked_forge_takes_its_handover_at_started_and_connects`
- `temper-service/src/tests/setup_create.rs:302` — `reconcile_then_create_then_connect_makes_the_first_repository_and_bot_workflow`
- `:341` — `no_first_intent_reconciles_and_connects_without_create`
- `temper-forge-setup-shell/tests/suite/prepare.rs:65` — `an_interrupted_first_preparation_discards_both_headers_before_starting_again`
- `temper-forge-setup-shell/tests/suite/reconcile.rs:43,57,72` — webhook-secret creation/reuse and no-listener behavior.
- `temper-forge-setup-shell/tests/suite/fetch.rs:80` — `local_tls_fetch`.

## 4. `temper-fake`, ports and fake backends

**Fake forge.** `temper/crates/temper-fake/src/forge.rs:43`:

```rust
pub struct Document {
    listen: SocketAddr,
    admin_token: Box<[u8]>,
    owner: Box<[u8]>,
    repositories: Box<[Repository]>,
    github: Option<github::Github>,
}
```

```rust
// :53
pub fn fake(fake: &temper_forge_setup::setup::Fake, admin_token: &[u8]) -> Self;
// :82
pub fn decode(bytes: &[u8]) -> Result<Self, Refusal>;
// :181
pub fn listen(&self) -> SocketAddr;
// :374
pub fn peer(
    document: &Document,
    token: &[u8; 40],
    seed: u64,
) -> Result<
    (skein_fake_peers::forge::Peer, alloc::rc::Rc<core::cell::RefCell<Forge>>),
    skein_fake_peers::Error,
>;
```

`peer` selects GitHub when `document.github` exists; otherwise builds a Forgejo peer and seeds its setup store (`:379–394`). Peer memory is recreated each launch.

Deployment form: `temper/crates/temper/src/fake.rs:348–366` reads setup, selects `Mode::Fake`, reads that entry’s handover and calls `Document::fake`.

Standalone form: `fake.rs:338–346` decodes `--document` and creates/reuses deterministic seeded credential output.

**Fake LLM.** `temper/crates/temper-fake/src/llm.rs:20`:

```rust
pub struct Document {
    listen: SocketAddr,
    control: Option<SocketAddr>,
    key: Box<[u8]>,
    added_keys: Box<[Box<[u8]>]>,
    root: Option<Box<[u8]>>,
    models: Box<[Model]>,
    scripts: api::Scripts,
    rules: api::Rules,
}
pub fn decode(bytes: &[u8]) -> Result<Self, Refusal>; // :31
pub fn configuration(document: &Document, key: &[u8], seed: u64)
    -> Configuration;                              // :143
```

`ROUTE = b"/v1/responses"` at `:140`. Deployment form gets its script/listener from `application.demo.llm`; standalone form decodes a document (`temper/src/fake.rs:371–399`).

Demo shape: `temper-fake/src/demo.rs:47` retains `llm`, `provider: u32`, `forge: u16`, `project: u32`; `read(document: &Json, at: u32, facts: &Facts<'_>) -> Result<Option<Demo>, Refusal>` is at `:82`. It validates the fake forge reference, provider endpoint, project and listener uniqueness.

**Ports.** There are configured ports, not hardwired ports in the peers.

`temper/crates/temper-init/src/lib.rs:33`:

```rust
pub struct Ports {
    pub clients: NonZeroU16,
    pub owned: NonZeroU16,
    pub forge: NonZeroU16,
    pub llm: NonZeroU16,
    pub webhooks: NonZeroU16,
}
```

Preferred init ports are `[8367, 8368, 8369, 8370, 8371]`, respectively (`temper/crates/temper/src/init.rs:209–214,272–274`). Init reserves alternatives when occupied. The ordinary fake forge therefore prefers 8369; ordinary fake LLM 8370.

Box keeps fixed listeners, including dormant fake listeners. Fake process adoption is at `temper/src/fake.rs:402–459`, covering forge, LLM and OAuth.

**`box --fake-backends` exists.** The “Missing” list in `temper/docs/design/draft/fake-backends.md:63` is stale against current code.

- Command variant: `temper/crates/temper/src/lib.rs:51`, `Command::BoxFakeBackends { directories, bundle, echo }`.
- Flag parsing: `temper/crates/temper/src/fake_backends.rs:39`.
- Entry: `:67`, `run(source: &Directories, bundle: Option<&Path>, echo: Echo) -> Result<Exit, String>`.
- Sandbox: `<source-data>/box/fake-backends/{config,data}`; Cache is shared (`:223–233`).
- Required effective documents: `temper.json`, `agent.json`, `github.json`, `oauth.json`, `llm.json`, `forgejo.json` (`:23`).
- Fresh initialization, source compatibility checks, lock and `prepared` marker: `:472–530,611–612`.
- Four distinct loopback peer ports are reserved dynamically (`:568–575`), excluding ordinary listeners and callback port 1455 (`:289–313`).
- Retained restart reuses effective documents and checks their ports (`:526–530`).

`temper/crates/temper-fake/src/backends.rs:24`:

```rust
pub struct Peers {
    pub github: SocketAddr,
    pub oauth: SocketAddr,
    pub llm: SocketAddr,
    pub forgejo: SocketAddr,
}
pub const SEED: u64 = 7;
pub fn steps(config: &[u8], peers: &Peers) -> Box<[Step]>; // :38
```

Always-running prefix (`:41–56`):

```text
fake github  -> temper fake forge --document .../github.json --seed 7
fake issuer  -> temper fake oauth --document .../oauth.json --seed 7
fake model   -> temper fake llm   --document .../llm.json --seed 7
fake forgejo -> temper fake forge --document .../forgejo.json --seed 7
```

Each is `Kind::Long` with a held listener. The prefix survives stopping the playground group.

`backends::rewrite(temper: &[u8], agent: &[u8], paths: &Paths, peers: &Peers) -> Result<Documents, Refusal>` is at `backends.rs:350`. It rewrites GitHub forge 2, OAuth registration, both model endpoints, worker/agent paths; adds managed Forgejo 4 (`:448–494`).

**Seeded fake GitHub.** `backends.rs:601–608` generates:

- owner/account `fake-person`, account ID 3;
- repository `widget`, landing `main`;
- App ID 1, slug `temper-fake`, bot ID 2, public test-key index 0;
- setup callback `/forges/2/github/setup`;
- installation ID 1, repositories `"all"`.

`temper-fake/src/forge/github.rs:43` decodes the GitHub extension; `:207–302` constructs the GitHub peer, App, permissions, repositories and installations. The box writes its public test App key to the effective data directory at `temper/src/fake_backends.rs:604–608`; this survey did not read that output.

**Reverse dependencies: fake document/demo/peer configuration and fake-backend shapes.**

```text
temper/crates/temper-fake/src/demo.rs:47
temper/crates/temper-fake/src/steps.rs:21
temper/crates/temper-fake/src/forge.rs:43
temper/crates/temper-fake/src/forge/github.rs:43
temper/crates/temper-fake/src/llm.rs:20
temper/crates/temper-fake/src/wire.rs:71
temper/crates/temper-fake/src/backends.rs:24
temper/crates/temper-fake/src/backends/tests.rs:9
temper/crates/temper-fake/src/tests.rs:160
temper/crates/temper-fake/src/tests/demo.rs:40
temper/crates/temper-fake/src/tests/steps.rs:177
temper/crates/temper-fake/src/tests/github.rs:25
temper/crates/temper-fake/src/tests/observations.rs:18
temper/crates/temper/src/setup.rs:14
temper/crates/temper/src/box_command.rs:78
temper/crates/temper/src/fake.rs:335
temper/crates/temper/src/fake/tests.rs:683
temper/crates/temper/src/fake/control.rs:8
temper/crates/temper/src/fake_backends.rs:20
temper/crates/temper/src/fake_backends/import.rs:155
temper/crates/temper/src/fake_backends/tests.rs:230
temper/crates/temper/src/init/tests.rs:328
temper/crates/temper/src/tests.rs:190
temper/crates/temper/tests/suite/fake_opening.rs:82
temper/crates/temper/tests/suite/fake.rs:545
temper/tests/real-loop/src/green_light.rs:47
temper/tests/real-loop/src/repair.rs:49
temper/tools/twin/src/document.rs:10
temper/tools/twin/tests/suite/cli.rs:65
temper/tools/twin/tests/suite/frames.rs:147
temper/tools/twin/tests/suite/frames/round_trip.rs:104
```

`forge::FILES` additionally feeds `temper/crates/temper/tests/process.rs:2633` and `temper/crates/temper/tests/support/first_repository.rs:222`.

**Reverse dependencies: init `Ports`.**

```text
temper/crates/temper-init/src/lib.rs:33
temper/crates/temper-init/src/template.rs:18
temper/crates/temper-init/src/tests/fixtures.rs:15
temper/crates/temper-init/src/tests/documents.rs:85
temper/crates/temper/src/init.rs:209
temper/crates/temper/src/fake_backends/import.rs:155
temper/crates/temper-fake/src/backends/tests.rs:23
temper/crates/temper-service/src/tests/stock.rs:18
temper/crates/temper-service/src/tests/welcome_policy.rs:25
temper/crates/temper/tests/support/owned.rs:832
temper/crates/temper/tests/support/first_repository.rs:715
```

Direct users of these temper-specific shapes in the other three trees: **not found**.

## 5. Process-tier tests and how they start a box

**Tier selection.** `.config/nextest.toml:180` selects binary `process` in temper packages. `tools/tiers/packages.toml:43` lists package `temper`. Its entry is `temper/crates/temper/tests/process.rs`; `:2–3` includes `tests/fake_backends/mod.rs`.

The setup-shell `suite`, box `real_loop`, binary `suite`, and `qa_all` cases are separate test binaries, not this process-tier selection.

**Box startup helpers.** `temper/crates/temper/tests/support/boxed.rs`:

```rust
// :170
pub fn run(
    directory: &Path,
    forgejo: Option<SocketAddr>,
    plan: Plan,
    deadline: Duration,
) -> Outcome;

// :185
pub fn run_command(
    command: Command,
    forgejo: Option<SocketAddr>,
    plan: Plan,
    deadline: Duration,
) -> Outcome;
```

`run` executes the shipped `temper` binary with `box` plus configuration directory flags (`:171–177`). `run_command` wraps it with `dying_with_test`, then uses `Binary::start_with_tree(..., Mode::Pipes, ..., Expectation::Supervise, false)` (`:187–189`).

- `run_echo_steps`: same command plus `--echo steps` (`:158–166`).
- `run_story`: starts box and a real client/person driver (`:369` onward).
- `run_ctl_peers`: starts box plus ctl and optional independent peers (`:856–902`); all public ctl box helpers converge here.
- Fake-backend flags: `temper/crates/temper/tests/fake_backends/mod.rs:35–48`.
- Fake-backend `Walk` creates isolated config/data/cache and a provisioned fixture bundle (`walk.rs:27–36`).
- Owned walks prepopulate cache (`walk.rs:39–42`).
- `support::forge_land::cache_forgejo_binaries` copies test-supplied pinned binaries into release-version cache paths (`support/forge_land.rs:577–588`).
- `forgejo_setup_steps` sends `forge setup 3` only on its first run, then waits for settled forge state (`:592–609`).

**Direct setup/box process cases.** All functions below are in `temper/crates/temper/tests/process.rs`.

| Line | Exact test |
|---:|---|
| 140 | `the_box_finishes_its_restart_policy_after_a_refused_restore` |
| 1077 | `forge_prepare_makes_an_owned_forge_then_remakes_nothing` |
| 1112 | `a_prepared_forge_runs_forgejo_and_its_runner_once_each` |
| 1138 | `an_older_recorded_forgejo_is_dumped_then_upgraded` |
| 1165 | `an_older_recorded_runner_has_its_files_prepared_again_without_a_dump` |
| 1183 | `reconcile_converges_an_owned_forgejo_and_a_second_run_writes_nothing` |
| 1217 | `temper_box_prints_its_link_and_stops_its_processes` |
| 1316 | `temper_box_restarts_a_killed_forgejo_while_the_engine_serves` |
| 1527 | `a_fresh_init_s_first_chat_reads_its_forge` |
| 1600 | `a_first_run_answers_on_the_demo` |
| 1606 | `a_csv_change_passes_ci_and_a_blocking_agent_review_on_the_owned_forge_then_lands_and_survives_restart` |
| 1783 | `a_first_run_plays_the_demo_then_sets_up_a_forgejo_here_with_no_restart` |
| 1929 | `a_persons_plan_change_lands_on_the_owned_forge_and_is_found_after_a_restart` |
| 2100 | `a_first_run_answers_on_the_demo_and_a_second_finds_it_running` |
| 2104 | `two_first_runs_one_after_the_other_choose_their_own_ports` |
| 2192 | `a_first_run_ctl_demo_start_answers_running_twice` |
| 2224 | `a_second_forge_ends_the_playground_for_good_closing_its_chat_and_account` |
| 3830 | `a_default_box_with_a_sign_in_registration_starts_without_dns` |
| 3919 | `a_persons_github_app_connects_pushes_once_and_is_found_after_an_engine_restart` |
| 4059 | `an_owned_forgejo_signs_its_member_in_through_temper` |
| 4066 | `a_forgejo_elsewhere_connected_as_its_admin_makes_the_first_repository_lands_a_chats_change_and_is_found_after_a_restart` |
| 4073 | `a_story_box_and_its_steps_die_when_the_test_process_is_killed` |
| 4553 | `playground_setup_adopts_widget_and_the_worker_reads_its_source_on_the_owned_forge` |
| 4795 | `a_restart_after_a_cut_at_listening_answers_ctl` |
| 4800 | `a_restart_with_another_box_sharing_its_cache_answers_ctl` |

Other process stories also use the same owned setup/box helpers: review exhaustion (`:1437`), gate repair (`:1705`), failed coder (`:2039`), inbox/landing story (`:4618`), and the account/review restart family (`:4779–4805`).

The direct prepare/reconcile cases run `forge prepare`, `forge run` and `forge reconcile` through `support/owned.rs:63,451,456,468`; they do not all start a box.

**Fake-backend process cases.**

| File:line | Exact test |
|---|---|
| `temper/crates/temper/tests/fake_backends/mod.rs:114` | `the_serve_child_reads_retained_effective_documents_across_restart` |
| `temper/crates/temper/tests/fake_backends/oauth.rs:232` | `standalone_oauth_signs_in_and_answers_the_first_model_chat` |
| `temper/crates/temper/tests/fake_backends/walk.rs:128` | `forgejo_elsewhere_and_a_key_reach_a_first_chat_on_the_fake_box` |
| Same `:145` | `chatgpt_sign_in_and_github_one_click_reach_a_first_chat_on_the_fake_box` |
| Same `:150` | `github_one_click_and_a_key_reach_a_first_chat_on_the_fake_box` |
| Same `:155` | `githubs_own_app_and_a_key_reach_a_first_chat_on_the_fake_box` |
| `temper/crates/temper/tests/fake_backends/walk/here_key.rs:39` | `forgejo_here_and_a_key_reach_a_first_chat_on_the_fake_box` |
| `temper/crates/temper/tests/fake_backends/walk/program_here.rs:44` | `a_program_and_forgejo_here_reach_ready_on_the_fake_box` |
| Same `:91` | `a_program_and_imported_account_survive_a_box_restart` |
| `temper/crates/temper/tests/fake_backends/walk/adoption.rs:28` | `context_preserves_the_writer_and_a_refused_setup_preserves_the_first_key` |
| `temper/crates/temper/tests/fake_backends/closed.rs:7` | `a_closed_chat_keeps_its_turns_and_replays_its_first_close` |
| Same `:89` | `a_cancel_replays_after_the_task_ends_and_a_new_key_is_refused` |

`fake_backends/mod.rs:183` also includes `qa.rs`, whose cases cover sandbox isolation, retained edits/secrets, source changes, partial preparation and second-box refusal (`qa.rs:237,328,359,380,400,413`).

**Test-support reverse dependencies relevant to changing setup/directories/cache.**

```text
temper/crates/temper/tests/support/boxed.rs:158
temper/crates/temper/tests/support/owned.rs:21
temper/crates/temper/tests/support/forge_land.rs:577
temper/crates/temper/tests/support/forge_land/rung.rs:604
temper/crates/temper/tests/support/first_repository.rs:484
temper/crates/temper/tests/support/restart.rs:6
temper/crates/temper/tests/support/csv_repair.rs:121
temper/crates/temper/tests/support/landing_by_turn.rs:40
temper/crates/temper/tests/support/forgejo.rs:297
temper/crates/temper/tests/support/forgejo_sign_in.rs:170
temper/crates/temper/tests/support/forgejo_elsewhere.rs:160
temper/crates/temper/tests/support/playground.rs:1
temper/crates/temper/tests/fake_backends/mod.rs:35
temper/crates/temper/tests/fake_backends/walk.rs:27
temper/crates/temper/tests/fake_backends/walk/github.rs:19
temper/crates/temper/tests/fake_backends/walk/chatgpt.rs:1
temper/crates/temper/tests/fake_backends/walk/elsewhere.rs:1
temper/crates/temper/tests/fake_backends/walk/here_key.rs:39
temper/crates/temper/tests/fake_backends/walk/program_here.rs:44
temper/crates/temper/tests/fake_backends/walk/adoption.rs:28
```

Relevant separate QA coverage:

- `temper/crates/temper/tests/qa_all/qa_forge_handover.rs:124,233,420,509,581,608`: handover restart, replacement, fake restart, asked startup/record, removal failure and preparation retry.
- `temper/crates/temper/tests/qa_all/qa_forge_setup.rs:136,174,210,250`: dormant startup, missing-file subsets, unasked missing files, absent box pipes.
- `temper/crates/temper/tests/qa_all/qa_first_run.rs:88,119,212`: held dormant port, playground end, bad cached binary.
- `temper/crates/temper/tests/qa_all/qa_first_repository.rs:43`: adopted/landed task across box restart without setup replay.
- `temper/crates/temper/tests/suite/box_listeners.rs:40,91`: held client port and refusal of occupied port.

**Not found:** a process case killing setup at every step, a whole-forge removal/reset crash-resume case, or a fake-backend GitHub case restarting between App conversion and installation. These are new coverage requested by reset §8.

## 6. GitHub setup: incremental App records and installation

**Current protocol:** `temper/docs/design/protocol/setup.md:3348–3520`, especially §17.2 at `:3396` and installation at `:3429`.

**Plan variants.** `skein/crates/skein-forge-setup/src/host/boundary.rs:17–31`:

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
    // Forgejo/read plans follow.
}
```

**Manifest/conversion path.**

- `temper/crates/temper-service/src/setup/public.rs:157`:

  ```rust
  pub(crate) fn manifest(
      engine: &mut Engine,
      forge: u16,
      request: Token,
      back: Option<Box<[u8]>>,
  );
  ```

  Always stages `Target::Personal` (`:171`).

- Service URLs are constructed from configured client origin/listener and forge number in `temper-service/src/setup/settings.rs:44–60`.
- Routes are `/forges/N/github/manifest`, `/redirect`, `/setup`, decoded at `temper-service/src/setup/routes.rs:34–43`.
- Manifest redirect validates/takes the live page, stages `Plan::Convert { forge, code, state }` (`routes.rs:402–453`).

**Exactly what conversion retains.**

`skein/crates/skein-forge-setup/src/github/convert.rs:76–96` destructures the conversion, drops its webhook secret, decodes PEM to DER, proves App identity and reads its bot.

`skein/crates/skein-forge-setup/src/github/write.rs:44–53` writes sequentially:

```rust
State::Begin   => (Item::AppId,   /* id LE */,   State::AppId),
State::AppId   => (Item::AppSlug, /* slug */,    State::AppSlug),
State::AppSlug => (Item::AppKey,  /* DER key */, State::AppKey),
State::AppKey  => (Item::User,    /* bot LE */,  State::User),
```

Each advances only on `Event::Written` (`:38–40`), which temper answers only after `Secrets::Keep` is durable, as described in question 3.

After conversion completes, temper writes `Item::Origin` (`temper-service/src/setup/lifecycle.rs:365–369`). After that write is durable, `Purpose::Origin` stages `Plan::Install` (`:532–536`).

Thus App rows survive independently as they arrive; there is no all-record commit or rollback.

**Old-draft discrepancy.** `temper/docs/design/draft/github.md:317` says conversion keeps the webhook secret. Current code drops it; `temper/docs/design/protocol/setup.md:3424–3425` and credentials §2.1 agree with code. The records are App ID, slug, DER key, bot user, accepted installations and origin.

**Installation can be requested from retained records.**

`temper/crates/temper-service/src/setup/public.rs:200`:

```rust
pub(crate) fn install(
    engine: &mut Engine,
    forge: u16,
    request: Token,
    back: Option<Box<[u8]>>,
);
```

It stages `Plan::Install { forge }` (`:215–217`), holding behind an internal run if necessary.

Skein installation machine:

```rust
// skein-forge-setup/src/github/install.rs:33
pub fn new(given: &Given, state: &[u8]) -> Result<Self, Refused>;
// :40
pub fn step(machine: &mut Install, event: Event, out: &mut Queue<Need>);
```

It reads **only `Item::AppSlug`** (`:46`) and returns:

```rust
Outcome::Install { state, url }
```

using `<web>/apps/<slug>/installations/new?state=...` (`:59–65`).

A fresh installation link can therefore be generated from retained slug after restart; old in-memory state/page is not required. This does not establish that the new welcome “Go on” behavior is implemented.

**Accepting the browser return.**

- Query contains `installation_id`, `setup_action`, optional `state`: `temper-service/src/setup/routes.rs:87–127`.
- With state, service stages `Plan::Accept { forge, installation, state: Some(state) }` (`:334–356`).
- Without state, only an already accepted installation is admitted as an update; otherwise the visit ends (`:311–318`).
- `setup_action=request` receives an explanatory page (`:300–308`).
- Explicit owner acceptance uses `public::accept(engine, forge, request, installation)` and `state: None` (`public.rs:219–246`).

`skein/crates/skein-forge-setup/src/github/accept.rs:35` constructs `Accept`; it reads `AppId`, `AppKey`, `Accepted` (`:76,86,97`), verifies the installation and writes the complete updated `Item::Accepted` set (`:53–69`).

**Restart behavior today.**

- Record loader: `temper/crates/temper-service/src/forge/records.rs:130–176`.
- GitHub access rebuild: `temper/crates/temper-protocol-records/src/forge.rs:165–238`.
- Complete identity records plus origin can connect with no `Accepted` row; absence gives an empty accepted list (`:233–236`).
- Incomplete records fail access as `NotConnected::Incomplete` (`:109–114`).
- Manifest pages/states are memory only: `temper-service/src/setup.rs:18–74`; protocol explicitly says restart forgets them at `temper/docs/design/protocol/setup.md:3441–3445`.
- Existing App fragments block another manifest/conversion through `has_app`, which tests presence of any `AppId`, `AppSlug`, `AppKey` (`temper-service/src/setup/lifecycle.rs:255–259`; refusal at `setup.rs:335`).
- After connection, GitHub reconcile is scheduled at `setup/lifecycle.rs:624–627`.

**Reverse dependencies.**

- Persisted App records, numbering and row/access shapes: the complete record inventory in question 3.
- Installation service entry/routing: `temper-service/src/route.rs:47` onward; `setup/public.rs:157,200,219`; `setup/routes.rs:34,334,402`; `setup.rs:301,397,484`; `setup/lifecycle.rs:308,506,552,575,589`.
- App write progression is constructed by `skein/crates/skein-forge-setup/src/github/convert.rs:164` and `skein/crates/skein-forge-setup/src/github/paste.rs`; its implementation is `github/write.rs:20`.
- These existing skein plans need no new stored forge schema: that schema and restartable setup intent are **not found** today.

**Existing tests and process proof.**

- `temper/crates/temper-service/src/tests/setup.rs:376`: verifies each write waits for durable `Kept`.
- `temper/crates/temper-service/src/tests/setup_github.rs:277`: `replaying_install_keeps_its_link_after_one_acceptance`.
- Same `:289`: `independent_install_states_each_accept_the_same_set_member`.
- Same `:305`: `paste_replay_and_refused_replacement_keep_the_connection_and_write_counts`.
- `temper/crates/temper-service/src/tests/setup_github_render.rs:110`: `retained_install_link_renders_in_human_and_json_replays`.
- `temper/crates/temper/tests/fake_backends/walk/github.rs:19–30`: actual five-hop manifest/Create/conversion/install/setup chain, asserting statuses `[200, 200, 302, 302, 200]`.
- Same `:33–64`: pasted own-App path, followed by `forge install 2`.
- `temper/crates/temper/tests/process.rs:3919`, implemented by `temper/crates/temper/tests/support/github.rs:53`: connected GitHub App, push and engine restart. It covers restart after connection, not the new interrupted-installation continuation.