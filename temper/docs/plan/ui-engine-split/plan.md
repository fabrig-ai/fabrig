# ui-engine-split: the plan

M2 of `temper/docs/design/draft/deployment-binaries.execution.md`: the
service's split on the engine's side and `temper-engine`, the engine's
binary (`inputs/request.md`; the draft, sections 4 and 9.2). Design
written by this cut:

- `temper/docs/design/protocol/README.md`, section 4's opening, 4.1 (the
  deployment's binaries: tables, `--version`, refusing another's
  document, the closure test, shared crates, features), 4.2
  (`temper-engine`), section 5's rows, 15.1's opening;
- `temper/docs/design/release.md`, 3 (`--version` for each binary,
  `temper-version`);
- `temper/docs/design/protocol/setup.md`, 7.4 and 11;
- `temper/docs/design/domain/agent.md`, 4.3 (the fitter's crate);
- `temper/docs/design/client/browser.md`, 2.6 (the bundle in
  `temper-engine`);
- `temper/docs/design/README.md`, sections 1 and 9;
- their status files, the draft's status and M2's row in the execution
  file.

## Parts and tasks

| Part | Task | Request item | Crates (files) | Needs | Minutes |
|---|---|---|---|---|---|
| `shared-brief` | `brief-crate` | `fit_brief` moves | new `temper-protocol-brief`; `temper-engine-smith`, `temper-service` call sites | | 45 |
| `engine-version` | `version-crate` | `--version` | new `temper-version` | | 60 |
| `engine-versions` | `jig-levels` | `versions` | `jig-protocol-store` (`levels`) | | 25 |
| `shared-shell` | `shell-crate` | the binary's help | new `temper-shell`; `temper` (`commands.rs`, `directories`, `help`, `link_file`) | | 90 |
| `service-split` | `engine-limits` | the engine's limits read no worker crate | `temper-service` (limits mirrors, `nearest`, the engine's modules) | `brief-crate` | 120 |
| `service-split` | `worker-feature` | an engine-alone service composes without the worker's crates | `temper-service` (`worker` feature), its dependents' manifests | `engine-limits` | 120 |
| `engine-shell` | `engine-forge` | `forge …`, `internal forge-child` | new `temper-engine-shell` (forge commands, their rows); `temper` | `shell-crate` | 120 |
| `engine-shell` | `engine-serve` | `serve` | `temper-engine-shell` (serve, host, bundle, runtimes, `WorkerHalf`); `temper` | `engine-forge` | 180 |
| `engine-binary` | `engine-binary` | the package and binary, its step tests, `versions` | new `temper-engine` | `engine-serve`, `version-crate`, `jig-levels`, `engine-limits` | 120 |
| `engine-closure` | `engine-closure` | the engine's half of the boundary test | `fabrig-boundary` (`tests/binaries.rs`) | `engine-binary`, `worker-feature` | 60 |

The first push releases the first four, which need nothing. The rest
follow in later pushes.

## Decisions

- **The split is a feature, not crates.** `temper-service` gains a
  `worker` feature, on by default and wherever the worker runs (the box,
  the worlds), off in `temper-engine`'s own build. Without it the
  worker's types are uninhabited stand-ins, so the service's system
  module and every caller compile unchanged both ways, and a document
  with a worker member is refused at startup. A crate split would cost
  about ten times as much: the worker side reaches into the service's
  limits, settings, tokens and translation, `Service` shares its io
  between the halves, and 13 files outside the crate would change paths.
  Draft section 8 already relies on a binary's own invocation keeping its
  features. The cost: the gate builds the workspace whole, so the
  feature-off service is compiled only by `temper-engine`'s own tests and
  the release build. That is the draft's accepted "a break shows at that
  build". Recorded in `protocol/README.md`, 4.1.
- **The engine's limits mirror the worker's.** The engine sizes its wire
  to the worker by the checkout's and git component's limits; it takes
  them in structs of its own, with the same fields and override names,
  which the worker's half converts. An engine-alone document carries
  every application member and must parse the same in every binary.
- **`temper-protocol-records`' `forge` feature needs no change for M2.**
  Only `temper-service`'s engine half turns it on, with the test kits and
  worlds; the client takes the records without it. The worker's half
  names no record. M3 takes it off the worker alone's build when the
  engine's half becomes optional.
- **The fitter goes to a new crate,** `temper-protocol-brief`, on
  `smith-domain-run` and `skein-lib`. No existing crate fits: jig's brief
  crate would take smith and temper's cut lines; smith's run crate would
  take temper's tool names; the worker wire is a codec crate.
- **"smith's agent", for the closure test,** is smith's agent process
  and its machinery: `smith-agent-`, `smith-local-`,
  `smith-inline-agent`, `smith-protocol-channel`, `-events`, `-llm`,
  `-machine`, `-program`, `smith-events` and `smith-domain*`. smith's
  host half, `smith-host-`, counts as the worker's domain. The engine
  keeps smith's codecs and shared translation (`smith-charter`,
  `-channel`, `-transcript`, `smith-protocol-translate`). It switches
  from `smith-protocol-channel` to `smith-protocol-translate` for the
  items it reads.
- **Exceptions: `smith-domain`, `-run`, `-session`, `-tools`.** They are
  smith's agent domain, sans-io, but their vocabulary (the charter's run,
  `Start`, `Event`, `Transcript`) is what jig's charter and the engine's
  translation speak. Cutting them would move that vocabulary into a smith
  crate of its own, a change of smith's that gives the engine no io or
  process less. They are named in the test, with that reason.
- **`upgrade` is dropped from this plan.** `temper upgrade --check` and
  `--back` exist in no binary. Their jig mechanisms (rehearsing and going
  back, jig's `protocol/store.md`, 10.5) aren't built: no
  `Request::Rehearse`, `Request::Back` or `BackRefusal`, and no rehearsing
  root. Building them is several hundred lines across jig and temper,
  partly jig's. The design places `upgrade` in `temper-engine`'s table,
  and `protocol/README.status.md` holds it **Not yet**, waiting on jig's
  10.5.
- **`versions` is built** in `temper-engine` alone, over jig's `levels`
  (a jig task), its format's versions and `temper-service`'s `WIRES`. Its
  lines are designed in 4.2. The box's `temper versions` stays as
  designed and **Not yet**: the draft moves the command to the engine's
  binary, and M5 decides the box's table.
- **`--version` is a shared crate,** `temper-version`, whose build script
  reads git, linked by binaries' packages alone. A commit then relinks
  those binaries and nothing they link. M1, M3 and M4 can link it for
  their own binaries. The box's `temper --version` stays `release.md`'s
  **Not yet**: no milestone asks for it, and its package also holds the
  library every test of `temper` links.
- **What `temper-engine` refuses.** A worker alone's document is refused
  naming `temper-worker serve`, as the request says. A standalone
  document is refused naming the box's `temper serve`, as draft section 4
  says ("each refusal names the binary that would run it"). The same
  holds for each forge command that reads the document. The shape is
  read before any check, so a broken worker document still names the
  right binary.
- **The bundle stays in the box's `temper` too.** The request's "the only
  binary to embed the browser bundle" is read within the deployment set
  (draft section 4, "The web client is the engine's"). The box's
  `temper` links all of the engine, serves the browser today, and keeps
  every command (draft 9.4).
- **The engine's lines name it** (`temper-engine serve: …`,
  `run temper-engine forge prepare …`). The moved code takes the
  binary's name, and the box passes `temper`, so the box's lines and
  every test reading them stay as they are.
- **The shared code is three crates, not one binary's package.**
  `temper-shell` holds what every binary uses (table, directories, help,
  link file). `temper-engine-shell` holds the engine's commands and
  serve's host, which the box also links. serve's worker half (agent
  preflight, runtime, program sources, the launcher's placement, the
  program-login line) stays in the box's `temper` and comes in through
  `serve::WorkerHalf`. `temper-engine` gives none.
- **No stored format, protocol or public interface of the product
  changes.** Only Rust APIs move, with `temper`'s paths kept by
  re-exports.

## What M3 takes from this split

- `temper-service`'s `worker` feature, and the stand-in pattern for the
  other half: M3 makes the engine's half optional, `records`' `forge`
  feature with it, so that a worker alone builds without the engine's
  crates.
- `temper-protocol-brief`: the worker's route already calls it, so the
  worker needs no `temper-engine-smith` for the fitter. The standalone
  start's `temper_engine_smith::room::host_part` read
  (`temper-service/src/startup.rs`) is the engine's, behind the engine's
  half.
- The limits' mirrors and their conversion in the worker's half.
- `temper-engine-shell::serve::WorkerHalf`: the worker's half of a start,
  which the box's `temper` implements from `agent.rs`, `claude_code.rs`,
  `worker_alone.rs` and `program_login`. `temper-worker serve` needs the
  host without the engine's half: `Host` names the engine's domain, so
  M3 splits `temper-engine-shell`'s serve host or moves it to a crate of
  no side.
- `temper-shell` (table, directories, help) for `temper-worker`'s table.
  `printed` and `program_environment` move to `temper-engine-shell`, the
  engine's program environment. `program_login`'s line needs `printed`'s
  `path_word`, so M3 moves `printed` again to a crate both link.
- `tests/boundary/tests/binaries.rs`: M3 adds the worker's row.

## Overlaps

- **`tui-box-front`** (integration) changes `temper/src/lib.rs`,
  `commands.rs`, `src/tests/commands.rs` and
  `docs/help/reference/commands.md`. `shell-crate`, `engine-forge` and
  `engine-serve` must edit `lib.rs` and `commands.rs` to move code out.
  They keep `commands.rs`'s rows in place, except the rows they replace
  by shared constants, and touch `lib.rs`'s `mod` lines and moved
  functions alone. A catch-up conflict there is expected.
- **`client-binary`** (M1) moves the TUI's link-file helpers into a
  client crate, while `shell-crate` moves `link_file` into
  `temper-shell`, which M1's client can link. Whichever lands second
  follows the first's place. M1 also writes `protocol/README.md`,
  section 4, for its binary. This plan added 4.1 (rules common to every
  binary) and 4.2 (the engine), so M1's goes beside them.
- **`agent-binary`** (M4) may write the boundary test's agent row first,
  in the same new file. Whichever lands second adds its row to the
  other's table.
- **`goal-call-faults`** changes `temper-engine-smith`'s `calls.rs`,
  `answers.rs`, `src/tests.rs` and `lib.rs`. `brief-crate` changes
  `lib.rs`'s `pub use start::…` line alone, and its `start.rs`.
- **`look-round-22`** changes `client/browser.status.md`. This plan
  appended one entry at its end.

## Critical path

`shell-crate` (90) → `engine-forge` (120) → `engine-serve` (180) →
`engine-binary` (120) → `engine-closure` (60): 570 minutes. Beside it,
`brief-crate` (45) → `engine-limits` (120) → `worker-feature` (120)
meets it at `engine-closure`.
