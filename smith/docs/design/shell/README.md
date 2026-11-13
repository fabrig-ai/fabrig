# smith's shell and product

Provisional, 2026-10-09, revised 2026-11-02. smith's shell layer: the crates that turn its
services into processes, the root package whose binaries are glue, and
the product `smith`, one process a person or a script runs. It covers
the commands, the settings and their presets, credentials, the
environment commands build in, and how the product composes one
process. The contracts it serves are the domain's (`domain/`) and the
protocol layer's (`protocol/`). This is the only document that holds
profile numbers (`settings.md`, section 2); everything else derives from them
(../protocol/limits.md). What is still open is listed in `README.md`, section 7.

## 1. In one page

- **Libraries in `crates/`, glue in `./src`.** smith is a kit taken as
  libraries (../domain/README.md, section 2). Its binaries are for
  showcase and internal use: the root package's `smith` and
  `smith-agent`, each a few dozen lines that parse, compose and drive
  (section 3).
- **Two shell libraries.** `smith-agent-shell` is the agent process a
  host spawns. `smith-local-shell` is the product: commands, settings,
  presets, credentials, composition and front ends (section 2). Both are
  ordinary Rust under skein's shell bans.
- **skein first.** The loop driver, an append sink, durable replace and
  a secret store, a directory's lock, the OAuth io driver, the LLM
  client's routes and credentials, trust roots, terminal modes and the
  process's resource usage are generic, and land in skein before smith
  uses them (2.4).
- **One process.** The product composes the local domain, which owns
  the inline agent as its child, with the effects: one loop, one ring,
  no channel. The shell builds the agent configuration from presets and
  settings as a value, and writes no file. Spawning an agent stays a
  library capability, and `smith-agent` is the spawned agent for hosts
  that want a prebuilt one, and for the product's runs on a program,
  which the inline agent cannot run (section 5).
- **Five commands.** `smith`, the interactive chat; `smith exec`, one
  headless run; `smith check`, validation; `smith login` and `smith
  logout`, an account's sign-in and sign-out (`commands.md`). A headless run says how it ended in its exit code (`commands.md`, 3).
- **Instrumented as it ships.** `--json` and `--trace` carry the
  versioned event stream (../protocol/events.md). The terminal
  renders the same records, never `Debug` text.
- **Settings are TOML,** layered, strict and bounded, with presets for
  `openai` and `anthropic`, which name no account. A first run needs a
  model and an account of the person's. Every refusal names the
  relationship it breaks (`settings.md`, section 1).
- **The numbers are here.** Each preset's per-model values and the
  `standard` profile's deployment and policy values are defaults that
  the probes of ../benchmarks/README.md validate. A value with thin evidence is marked
  provisional (`settings.md`, section 2).
- **smith's own role.** Without `instructions`, a run's role is the task
  role: one text for chats and tasks, on the topics `settings.md`, section 2.6 lists.
- **An account is a provider and one source,** named in the person's
  settings, several per provider: smith's own sign-in, a key, an
  environment variable, a program's own login, or none. smith keeps its
  records and keys in a credentials directory of its own, reads no other
  program's login, and never copies a refresh token (`credentials.md`).
- **The build environment is declared:** a named core environment,
  writable caches, and warnings for build wrappers whose caches cannot be
  written (section 4).

## 2. Crates

### 2.1 Libraries only

`crates/` holds libraries only: no crate there declares a binary. A
project built on smith takes the layer it needs, and a binary is the
root package's (section 3). The layers are those of skein's echo
(skein's `examples.md`, section 2):

| Crate | Layer | Owns, for the shell |
|---|---|---|
| `smith-agent-profile` | values | the agent's configuration values, `Config`, `Limits` and `ProgramBackend`; the `standard` profile's deployment values and their derivation into every layer's limits, the host's share among them, `host_limits`; the event limits; and a configuration's checks, as step code with its tests (2.6) |
| `smith-agent-service` | service | the agent's `iterate`, on the profile's values, which it re-exports under its own paths (2.6) |
| `smith-local-service` | service | the local host's `iterate`: the local domain, which owns its agent as a child (the inline agent, or `smith-host-domain` for a spawned one), composed with the effects: the LLM and machine components, the chat store, the terminal and the event sink; the local layer's share of the profile, and the host's re-exported as `profile::host_limits` |
| `smith-inline-agent` | domain | an agent capability in its host's process: a run as a composed `smith-domain` that speaks `smith-host-domain`'s vocabulary (../domain/host.md) |
| `smith-events` | codec | the event vocabulary (../protocol/events.md), encoded over skein-json, with its goldens |
| `smith-local-protocol` | protocol | the chat store and delivery records over skein io (../protocol/host.md, section 5.3); the credentials directory's keys and installation identifier, as private files (`credentials.md`) |
| `smith-agent-document` | shell | the agent's document read without the agent: decoded, resolved and checked, its trace's settings, and a program's directory, source and sign-in (2.6) |
| `smith-agent-shell` | shell | the agent process (2.2), re-exporting the document's crate as its `config`, `program` and `trace` |
| `smith-local-shell` | shell | the product (2.3) |

A service owns its entrance limits and admission watermark. The agent's
limits, the profile that derives them and a configuration's checks are
values and functions with nothing of the run in them, so they are
`smith-agent-profile`'s, beneath the service, and tested there; the local
service's share stays with it. Edge bounds must be representable; memory
is measured under scenario ceilings (../protocol/limits.md, section 5). The
shell crates only choose a profile and the values a setting overrides.

### 2.2 The agent shell

`smith-agent-shell` is the agent process a host spawns, as a library:

- **The agent configuration:** the JSON document of ../protocol/agent.md,
  section 4, read whole before the loop, strict and bounded. It names a
  profile and its declared values, the endpoints, the programs a run
  may use (../protocol/programs.md) and the commands' environment. Each endpoint declares its models' `window`, `output`,
  `reasoning_item`, `head` and `idle` (../protocol/limits.md, 2.1); a
  charter's model entries carry the effective `window` and `output`, at
  most the declared ones, and a charter naming a model the configuration
  does not declare is refused (../protocol/charter.md, section 2). It never
  holds a credential. It is `smith-agent-document`'s, which this crate
  re-exports (2.6).
- **Startup,** in the order of skein's `runtime/shell.md`, section 6: the
  configuration and the derived edge limits checked, termination
  signals blocked, the endpoints' names resolved, trust roots loaded,
  the seed and the kernel.
- **The one `Agent` adapter:** a `skein_shell::Host` over
  `smith-agent-service`. The service drains the domain's facts each pass
  into its event sink, an append stream through io, and onto the
  channel, each with its own policy and loss count (../protocol/agent.md,
  section 5). The adapter opens or adopts the workspace's roots, and its
  hook writes only the operator's short lines to an error writer its
  caller supplies (skein's `runtime/shell.md`, section 11). `smith-agent` runs
  it with `skein_shell::drive`, and the worlds host the same adapter
  (../testing.md, section 4).
- **The end** is ../protocol/agent.md, section 6's: its answer sent, the
  channel finished, everything it owns closed, then exit. An agent that
  aborts runs no end. On Linux its programs die with it: it starts each
  through its own binary's `--program-child` mode. On macOS it starts
  them directly, and an abort needs the host's whole tree ended
  (../protocol/programs.md, 5.1).

### 2.3 The local shell

`smith-local-shell` is the product, as a library:

- **Commands:** typed command values and their parser, so that `main`
  holds no parsing and the parser has step tests (`commands.md`).
- **Settings:** the layers, the presets, the defaults and their
  validation (`settings.md`, section 1), and `smith check`, a function of the resolved
  settings and what startup found (`commands.md`, 4).
- **Credentials:** each account's provider and source, the credentials
  directory, a program source's check, and the front ends of sign-in
  and sign-out (`credentials.md`).
- **Composition:** `compose` builds `smith-local-service`: the local
  domain with the inline agent as its child, and the effects, the agent's
  LLM and machine components among them (section 5); or, for a run
  whose account's source is a program, the local domain with
  `smith-host-domain` as its child and a spawned `smith-agent`, whose
  JSON configuration it then generates (`settings.md`, 1.4). Either
  way the agent configuration is built from the presets and the resolved
  settings.
- **The `Local` adapter:** a `skein_shell::Host` over
  `smith-local-service`. The service's event sink writes the records
  through io; the front ends read them in the process (below); the
  adapter's hook writes only short operator lines.
- **Front ends:** the interactive renderer and its plain mode (`commands.md`, 1); the
  headless one, which maps an answer to output and an exit code (`commands.md`, 2, 3
  ). Both read `smith-events` records.

It owns no store and no loop: chats are kept by `smith-local-protocol`
over skein io, sign-in records by skein's OAuth driver as private files
under their lock (`credentials.md`), and the loop is `skein_shell::drive`.

### 2.4 What skein provides first

Each lands in skein, with its design, before its smith consumer
(AGENTS.md):

- **`Host` and `drive`,** moved from `skein-world` to `skein-shell`:
  `drive` is the loop of skein/docs/foundation/programming-model.md, section 2, with a hook
  each iteration for a shell's drains. `skein-world` re-exports `Host`,
  and no shipped binary links a testing crate.
- **An append sink:** a bounded append stream on the ring, to a file or
  an inherited descriptor, with a write deadline, that counts what it
  drops and settles exactly at exit (skein's `runtime/io/files.md`, 1). No thread.
- **Resource usage:** the CPU time and peak resident size of the process
  and of its reaped children, read by the shell at the end for
  `session.ended` (../protocol/events.md, 3.2).
- **Durable replace and a secret store:** a whole file replaced over
  io's create, write, sync and rename, its directory synced; and private
  record files on it, with their permissions and links checked.
- **A directory's lock:** exclusive, taken without waiting, released
  when its root closes or its process ends (skein's `runtime/io/files.md`),
  so that processes sharing a record take turns writing it (`credentials.md`).
- **The OAuth io driver:** the loopback listener, the token exchange over
  skein-http, and refresh before expiry, which `skein-oauth` leaves to
  its caller (skein's `oauth/README.md`); and `localhost` redirects, bound to
  127.0.0.1 and compared exactly. With it, a client id the issuer
  issues, a checked ID token and its subject, revocation, a redirect
  whose port varies, and a record shared across processes, read again
  under its owner's lock before each write (skein's `oauth/README.md`, 2.1 to 2.3, and `oauth/accounts.md`, 4 to 5
  ).
- **The LLM client's routes:** OpenAI's public Responses route and
  Anthropic's Messages with a key, a credential as a value and its
  scheme, and the failure classes for an account's limit, its refusal
  and a field a provider does not take (skein's `llm/README.md`, 2.5, 2.6, 4.7
  and 4.8).
- **Trust roots:** the machine's roots, or one DER file, loaded at
  startup (skein's `net/tls.md`).
- **Terminal modes:** the window's size, as a shell effect, for the
  status line. Raw input, and its restoration at exit, wait for a view
  that reads a key at a time (../protocol/host.md, open questions: a
  richer terminal). The line-mode chat (`commands.md`, 1) leaves the terminal in its
  own line mode.

### 2.5 The hosted face

A host that embeds the agent, rather than spawning `smith-agent`, hands
it everything: an application's world, natively or in a browser's Web
Worker (skein's `testing/world.md` and `skein/docs/design/playground.md`), hosts it so. That
part of `smith-agent-shell` reads nothing of the system, and builds for
`wasm32-unknown-unknown`, the portable target of skein's `runtime/shell.md`,
section 13.

- **Portable:** `Agent::new`, the configuration's decoding and checks
  from its bytes, the derived limits, `Resources`, and the `Agent`
  adapter with its `iterate`: what the host gives is all it uses. The
  configuration comes as a value or as bytes, and its roots, its
  descriptors, its seed and its process ID as values.
- **No current directory and no name resolution.** The host supplies
  base roots by name, each a directory it opened (a worker's, say). A
  root a run asks for, at its start, is either one of them, matched by
  its name as given, or a path beneath one, which the agent opens
  beneath it through io, as the native agent opens one by path; never
  made absolute against a current directory. A path beneath no
  supplied root is refused, naming it, and the run with it. An endpoint
  is the address the configuration gives, never a name to resolve.
- **Native, under `cfg(unix)`:** `run` and everything it reads (the
  configuration's file, trust roots, the system's seed, termination
  signals, the current executable, the kernel); a program's sign-in
  (../protocol/programs.md), with its directories, its owner and its
  command; the effective user; the relay's `main`; and a trace or a
  root opened by path. A host that gives no trace and no programs, as a
  world does, reaches none of it.
- **Fenced by target, not by feature,** at the coarsest grain that holds
  each item, as skein's shell is; `libc` and skein's native shell
  exports are named only there.
- **No consumer changes** for a host: `smith-agent` and the worlds take
  the same `Agent`.
- **Checked** in smith's gate: clippy over `smith-agent-shell`,
  `smith-agent-document`, `smith-agent-profile` and what they depend on
  in smith for `wasm32-unknown-unknown`, warnings denied.

### 2.6 The agent's document without the agent

A host reads its agent's document before it starts the agent: an
application's worker checks it at its own start, derives its host's
limits from it and checks its programs' directories. So the document's
reader is a crate of its own, and nothing of the agent's run is in its
closure:

- **`smith-agent-profile`,** step code, `no_std` with `alloc`, beneath
  `smith-agent-service`. It holds what a configuration is and what it
  derives, none of it the run's:
  - the `profile` module, whole;
  - the configuration's values, `Config`, `Limits` and
    `ProgramBackend`, and the route headroom the profile reserves;
  - `event_limits`;
  - a configuration's checks, `valid`, `effects_valid`, `io_valid`,
    `program_valid` and `program_operations`, with the programs'
    settings, reservations and bounds they compute; the service's run
    reads those from here, never from a copy;
  - `host_limits`, the host's share of the profile (../protocol/limits.md,
    3.6 and 3.8).

  It depends on what the service depended on for these, and on
  `smith-host-domain` for `host_limits`; never on `smith-agent-service`
  or anything that links it.
- **`smith-agent-document`,** ordinary Rust as the shell is, on the
  profile's crate: `config`, the document decoded from its bytes,
  resolved and checked, and read from its file; `trace`, the trace's
  settings it names; `program`, a program's directory, source and
  sign-in (../protocol/programs.md), on Unix; `host_limits`, the profile
  crate's, re-exported. Its fixture, the standard product document, is
  `smith-agent-document-fixtures`' for the shell's tests, and a
  `#[cfg(test)]` copy for its own (../testing.md, 7). The three items of `config` the shell reads beyond
  a document's readers, the effective header, the profile's refusal
  words and a decoded document's list endpoints, are public, each with
  its line of prose. It names neither `smith-agent-service`,
  `smith-agent-shell` nor `smith-local-service`.
- **Their readers don't change:** `smith-agent-service` re-exports the
  profile crate's items under the paths they had
  (`smith_agent_service::profile`, `Config`, `Limits`,
  `ProgramBackend`, `event_limits` and the checks);
  `smith-local-service`, `host_limits` as `profile::host_limits`;
  `smith-agent-shell`, the document's `config`, `program` and `trace`
  modules, and its fixture's product document. Each item is one item,
  so its type is the same wherever it is named.
- **The hosted face** (2.5) holds: the decoding and the checks are
  portable, the file, trust and program parts Unix's, fenced by target
  as before.
- **Tests move with what they test,** and neither crate takes a
  dev-dependency on the service, the shell or a world that links them:
  a test that needs `Service` or `Agent` stays where it is and reads the
  moved items through the re-exports.

## 3. The root package

### 3.1 The manifest

- **The package's manifest.** `smith/Cargo.toml` is the package `smith`,
  a member of the repository's one workspace (`AGENTS.md`, section 1).
  Its binaries:
  - `src/main.rs`, the binary `smith` (`default-run`);
  - `src/bin/smith-agent.rs`, the binary `smith-agent`.

  It has no features (skein/docs/foundation/cli.md, 3). The fake program
  the end-to-end tests run is `smith-fake-claude`'s binary, found beside
  `smith`, and the package takes that crate as a dev-dependency alone
  (../testing.md, 2.3).
- **Nothing is discovered.** `autobins`, `autotests`, `autobenches` and
  `autoexamples` are off, and every target is declared: the binaries
  above, and the tests `end_to_end` (`tests/end-to-end/end_to_end.rs`),
  `live` (`tests/end-to-end/live.rs`) and
  `live_configuration` (`tests/end-to-end/live_configuration.rs`).

  The end-to-end and live tests are the package's own targets, because
  cargo gives `CARGO_BIN_EXE_<name>` only to a package's own tests
  (../testing.md, section 7). The world crates under `tests/` stay packages
  of their own. The gate names `--workspace`.
- **Glue.** `main.rs`:
  1. parses its arguments into a command (2.3);
  2. reads the settings and runs `smith check`'s checks, refusing with
     their exit code (`settings.md`, 1.3);
  3. blocks termination signals, draws the seed, and opens the kernel
     and the roots (skein's `runtime/shell.md`, section 6);
  4. composes the service, runs it with `skein_shell::drive`, and exits
     with the code its outcome maps to (`commands.md`, 3), once the service holds
     nothing (skein's `runtime/shell.md`, section 12).

  `smith-agent.rs` reads the configuration's path, starts
  `smith-agent-shell`, runs it with `drive`, and exits 0 after its
  answer or 1 when it could not answer (../protocol/agent.md, section 6).
- **MIT-licensed.** The root `LICENSE` holds the MIT licence, and every
  manifest, the package's and each crate's, takes the workspace's
  `license = "MIT"`. smith's registration with a provider is an
  open-source program's own (`credentials.md`).
- **Files before the loop.** A shell may read whole files with std before
  its loop runs: the settings, the agent configuration. Once the loop
  runs, every file and process operation is a ring operation
  (skein/docs/foundation/programming-model.md, section 2.1).

### 3.2 Lints

- **The root package has its own `[lints]` table,** not the workspace's:
  the ordinary-Rust table of skein's echo shell (skein/docs/foundation/programming-model.md,
  section 10.2). `unsafe` is forbidden; silent casts, overflow and
  panicking indexing are out; disallowed types and methods are denied.
  The shell crates under `crates/` carry the same table. No crate allows
  the `disallowed_*` lints wholesale.
- **Two `clippy.toml` files, split by layer.** Clippy reads the nearest
  `clippy.toml` above each manifest:
  - `crates/clippy.toml` holds the step crates' rules
    (skein/docs/foundation/programming-model.md, section 10), for every crate below `crates/`
    without its own;
  - the root `clippy.toml` holds the shell bans of skein's echo: `File`,
    `OwnedFd`, std's sockets, `std::process`, `Instant`, `SystemTime`,
    `HashMap`, `HashSet` and `thread::spawn`. It covers the root package
    and its test targets;
  - `smith-agent-shell` and `smith-local-shell` each keep a copy of the
    root file, since the step file sits nearer to them;
  - `tests/clippy.toml` is unchanged, and `benchmarks/` keeps its own
    (../benchmarks/README.md), since a harness starts processes.

## 4. The build environment

- **The commands' environment is the configuration's, whole**
  (../protocol/agent.md, section 3): what `[environment]` inherits and sets,
  and nothing else. Shells and checks see it; search sees none.
- **A named core list.** By default `inherit` names `PATH`, `HOME`,
  `USER`, `LOGNAME`, `LANG`, `LC_ALL`, `LC_CTYPE`, `TZ`, `TERM` and
  `TMPDIR`, each taken from smith's own environment when present. Names
  are listed, never matched by a pattern. A build preset adds its own:
  `rust` adds `CARGO_HOME` and `RUSTUP_HOME`.
- **Writable caches.** A build preset, or `[build].caches`, names the
  directories outside the workspace that builds write: for `rust`,
  `$CARGO_HOME/registry` and `$CARGO_HOME/git`. `smith check` confirms
  each is a directory smith's effective user may write and search, as
  the kernel answers it (skein's `runtime/shell.md`, 6.1, `directory_access`),
  creating and changing nothing. A cache it cannot confirm (absent, not a
  directory, or denied, with the kernel's error) is a warning naming the
  cache and why: a build that never writes there is not stopped by it.
  With contained trees, they are the writable part of the view
  (../protocol/agent.md, section 3).
- **Wrapper warnings.** A build preset names the wrappers its tools may
  be configured with, such as `rustc-wrapper` for `rust`. `smith check`
  warns when a configured wrapper's cache is not among the writable
  caches, or cannot be written, and names the variable or file that
  configured it.
- **Credential-like names.** `smith check` warns when a variable the
  commands would see has a name that suggests a credential.
- **Per-tree limits come with contained trees.** CPU, memory and process
  limits per command tree are theirs, and settings do not stand in for
  them.

## 5. One process

- **The product is one process.** `smith-local-shell` composes
  `smith-local-service`: the local domain, which owns the inline agent
  as its child domain, composed with the effects, the agent's LLM and
  machine components, the chat store, the terminal and the event sink,
  in one protocol layer: one loop, one ring, no channel
  (../domain/host.md, section 9). The agent configuration is a value the
  shell builds from presets and settings, never a file. Credentials are
  lent as entities. The loop is `skein_shell::drive`.
- **A run on a program is spawned.** The inline agent has no program
  backend (../protocol/programs.md, section 2), so a run whose account's
  source is a program runs on a spawned agent: the local domain is
  composed with `smith-host-domain` and its protocol layer, and spawns
  `smith-agent`, found beside `smith`'s own binary, with the generated
  configuration (`settings.md`, 1.4). The agent spawns the program as its child, and
  the run's events, failures and exit codes are the same as inline
  ones. Every other run stays inline.
- **Events come from the domain's owner.** The inline agent's owner
  drains each run's facts every pass into the service's event sink, so
  the event stream is the same in one process as from a spawned agent
  (../protocol/events.md).
- **Stopping:** an interrupt is the run's cancel; a second, or a run past
  its grace, is dropped, and everything below it settles before the
  process ends (../domain/host.md, section 9.2).
- **The end:** after its last word, the process closes everything it
  owns and exits once each has reported closed and io is empty, when
  `drive` returns (skein's `runtime/shell.md`, section 12). The LLM pool closes
  at the answer. A termination signal while closing turns the remaining
  closes into aborts. A host that spawns `smith-agent` derives its exit
  grace from the same bound (`settings.md`, 2.5).
- **The spawned capability.** Spawning an agent is a library capability:
  `smith-host-domain` and `smith-host-protocol` in a host, and
  `smith-agent-shell` in the process. `smith-agent` ships with `smith`,
  for the product's runs on a program and for any host that wants a
  prebuilt agent process, and is the real-process proof of the channel.
  An `isolate` option of the product, which spawns the agent of every
  run, comes with contained trees.

## 6. The world

Within the focused and fuzzy suites' budgets (skein/docs/foundation/testing-strategy.md,
section 8):

- **Step tests in `smith-local-shell`:**
  - each command's arguments to its typed value, and each usage error;
  - each layer's merge, the environment per variable;
  - one refusal per strict rule, naming the file, line and key;
  - each key the repository layer may not set, refused;
  - preset expansion, byte-stable, against goldens;
  - the task role within 4 KiB, each tool it names one of smith's own;
  - one refusal per relationship, naming it;
  - the mapping from every answer to its exit code and its last line on
    standard error, one per class: limited, refused and unauthorized
    among them, and a limited account's line with and without its reset;
  - each source's settings, and each refusal of `settings.md`, 1.2 and 1.3: a name out
    of bounds, an ambiguous account, `--account` of another provider, a
    `program` source on an endpoint without a program, and each setting
    of an earlier version naming what takes its place;
  - account numbers: one name, one number, in any order of the settings,
    the first four bytes of its SHA-256 read big-endian; two account
    names of one number, a `none` account's among them, refused with
    exit 2, naming both.
- **Step tests in `smith-agent-service` and `smith-local-service`:** the
  `standard` profile derives and passes its edge compatibility checks.
- **The local host's simulated world** (../testing.md, section 2.1):
  - one headless story per exit code but 2;
  - events against what the fake provider saw: one `response.completed`
    per response, tool records paired by call id, usage summing to the
    answer's spend, no loss;
  - each source lent: a sign-in's record, refreshed before it lapses;
    a key; an `env` account; a `none` endpoint;
  - two processes on one record: one refreshes under the lock, the
    other uses the newer generation with no refresh; a lock held past
    the lock wait fails the write as busy, and a later grant succeeds;
  - a program account's check at startup, each answer of it, and a run
    on the fake program on a spawned agent (../protocol/programs.md,
    section 8);
  - a limited account, with a reset and without one; a refused account;
    an unauthorized grant refreshed once, and one refused for good: each
    with its exit code and last line;
  - a chat of an earlier version refused, and a chat carried over to
    another account.
- **End to end** (../testing.md, section 2.3), against fakes:
  - `smith exec --json` to an accepted answer, and to needs input;
  - `smith check --json`, passing and failing;
  - a startup refusal of each layer;
  - `smith` on a terminal: a first sign-in and a change committed in
    place, then a second invocation that resumes; and one with
    `--trace`, its records read from the file as it grows;
  - `smith login` and `smith logout` against the fake issuer, reached
    through the settings' moved addresses: a first sign-in that
    registers, a sign-in again with the issued client id, another
    subject refused, a scope declined, a revocation confirmed and one
    not; a key read from standard input, and one from a terminal
    refused;
  - `smith login` on a `program` account, against the fake program;
  - `smith-agent`, spawned by a host in the test's loop.
- **The hosted face** (2.5): an agent hosted with roots supplied and
  no current directory runs its story as before; a configuration naming
  a root the host did not supply is refused at `Agent::new`, naming it;
  and the portable target is checked by clippy in the gate.

## 7. Open questions

- **Program lifetime on macOS** (2.2): ../protocol/programs.md, section 9's
  question. Default: direct launch and the host's whole-tree supervision.
- **The Anthropic window:** whether the preset's window should be the
  models' 1,000,000 tokens once the `history` and `cache-reuse` probes measure
  cost and quality there.
- **Kept traces:** a default place for traces, their rotation and how
  long they are kept.
- **The plan's models:** which of the `openai` preset's models
  `/v1/models` lists for a ChatGPT plan, and whether the small and
  working tiers' `gpt-6-luna` and `gpt-6.1-sol` are among them. Default:
  the preset declares them, and a run on one the plan refuses fails as
  refused.
- **A chat's account between invocations:** a chat resumed with another
  account is carried over (`commands.md`, 1). Default: allowed, said once on the
  status line.
- **A chat's directories between invocations:** a
  chat resumed with other `--dir` directories than it began with.
  Default: the workspace is the resumed invocation's, and the
  transcript's paths into a directory no longer named are read as they
  were written.
- **An `isolate` option,** once contained trees make spawning contain
  more than one process does.
