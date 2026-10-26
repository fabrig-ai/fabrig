# Session 10: configuration's startup, `temper serve` and `temper agent`, the process tier's support

`temper-service` gets the startup that turns configuration's bytes into
the one process (configuration.md; protocol.md, section 12, step 3), and
temper gets its binary. The startup reads jig's parts with
`jig-protocol-config` and temper's `application` member with
`jig-protocol-config`'s readers, by configuration.md, 10's format rule;
it runs every check of configuration.md, 2 and 7 and channel.md, 8 before
anything is made; it names each problem by its path, one line each. The
binary `crates/temper` is jig's reference binary (`testing/jig-test` at
the pin) copied, with `serve` and `agent`; `open` and `sign-in` are 16's.
The session ends with the process tier's support: temper's configuration
written by the test, the binary under skein-world's end-to-end harness,
skein's fake LLM peer, nextest's `process` and `live` profiles and
`scripts/proof-one-process.sh`, proved by the binary printing its link
and draining on a signal. 12.2 then runs the first story through it.

## Read first

- `configuration.md` (all); `protocol.md`, sections 4 and 8;
  `credentials.md`, section 3; `channel.md`, section 8; `git.md`,
  section 8; `domain/testing.md`, 2.6 and 6.1.
- `decisions.md`, 5, 8, 9 and 13; `code-organization.md`, sections 1 to
  8.
- `findings/jig-service.md`, sections 1 ("Payload version", "Where
  application-specific parts plug in"), 2 and 3, Discrepancies 2, 3, 4,
  6 and 9.
- jig at the pin: `testing/jig-test/{Cargo.toml, clippy.toml,
  src/{main.rs, lib.rs, host.rs, tests.rs}, tests/{process.rs,
  support/mod.rs}}`; `testing/jig-test-service/src/startup.rs`
  (`parse` :104, `check_files` :276, `engine_startup` :304,
  `worker_specs` :432, `named` :515, `startup` :533);
  `crates/jig-protocol-config/src/{read.rs, problem.rs, files.rs,
  fits.rs, forms.rs:28 (secret), projects.rs:336 (pattern)}`;
  `tests/channel/src/fixtures.rs:100-160` (`WORKER_DOCUMENT`,
  `engine_document`, `files`); `.config/nextest.toml`;
  `scripts/proof-one-process.sh`.
- skein at the pin: `testing/skein-world/src/end_to_end.rs` (`Binary`,
  `Command`, `Mode`, `Streams`); `crates/skein-shell/src/startup.rs`
  (`read_file`, `read_private_file`, `Unread`), `ring/roots.rs:11`
  (`make_root`), `ring.rs:1595` (`open_root`); `crates/skein-lib/src/{writer.rs,
  decimal.rs}`.
- The session files `temper/08-service-engine.md` (08.1's `settings`,
  `limits`, `tokens`; 08's engine constructor), `temper/09-service-worker.md`
  (09.1b's `Worker` and `WorkerRuntime`, 09.2's `path`, 09.3's
  `system`), `temper/03-wire-codecs.md` (03.1's `Bounds`, `limits`,
  `largest`), `temper/07-git-component.md` (07.1's `Limits`, `check`).

## What the design asks

- **One document, three members** (configuration.md, 2): `engine`
  (jig's), `workers` (jig's), `application` (temper's); "Nothing else at
  the top level". "Unknown, missing, duplicate or out-of-bound members
  refuse the start, naming the path."
- **What standalone requires of jig's parts** (configuration.md, 2):
  "`workers` has exactly one entry, whose `engine` is
  `{"in_process": "workers"}`"; "`engine.hosts.listener` is
  `[{"in_process": "workers"}]`, and `engine.hosts.workers` names that
  one worker"; "`engine.clients.listener` is a socket address on
  loopback"; "`engine.connectors` is exactly `[1]`"; "the worker's
  `agent` paths are absolute".
- **Reading, in order** (configuration.md, 2): the file within the
  reference binary's bound; jig's parts by `jig-protocol-config`,
  `application` "with `jig-protocol-config`'s readers, so every problem
  names its path the same way (`application.forges[0].user`)"; "the
  checks across the whole"; "every private file the document names, each
  within its bound ..., then each identity's whole value against the
  channel's bound on a grant's value"; "the agent's configuration".
  "Every problem found is reported, one line each on stderr, and `temper
  serve` exits 1 before io starts."
- **temper's members** (configuration.md, 3 and 4): `forges` (number,
  kind, api, transport, git, api_token, user, identity), `identities`
  (number, token, login, author), `landing`, `charter`, `limits`,
  `workers` (root, git); `webhooks` "Present, it refuses the start as
  not built".
- **The format rule** (configuration.md, 10): member names are field
  names; a fieldless variant is a snake_case string; a variant with
  fields an object of one member; durations `_ms`; "an absent option is
  `null`, never a missing member"; a map keyed by a number is a list of
  objects; jig's types as jig writes them.
- **Secrets' files** (configuration.md, 6): "A secret file holds its
  bytes and nothing else but a final newline"; "Read at start, each
  within its bound, with skein's `read_private_file`": a file readable by
  others or reached through a link "refuses the start, naming it".
- **Limits** (configuration.md, 7): "One profile, `small`"; "Overrides by
  dotted name"; "Limits checked against each other at start, each failure
  naming both limits"; "One worst case per service ... or the start is
  refused, saying by how much". The channel's (channel.md, 8): within
  jig's ceilings; "the engine's within the worker's"; "The remote fits
  the worker's names".
- **The agent's configuration** (configuration.md, 5): "every endpoint of
  `engine.llm.endpoints` must be in the agent's configuration under the
  same name with the same account, and every model a charter names must
  be declared there, or the start is refused, naming it."
- **The binary** (protocol.md, 4): "`temper serve CONFIG.json`" and
  "`temper agent AGENT.json`" (`smith_agent_shell::run`); "Its stdout
  carries only the sign-in links"; "A first termination signal drains it
  ...; a second turns the drain into an abort"; exit 0 after a drained
  shutdown, 1 on any failure, "the reason on stderr".
- **The process tier** (domain/testing.md, 2.6): "the real `temper` binary,
  `temper serve` on a scratch directory, with a configuration the test
  writes"; skein's fake LLM peer on loopback; "nextest profile `process`,
  run by `scripts/proof-one-process.sh`, which builds the binary first".

## Choices this plan makes

- **10.1 is two increments** (code-organization.md, 8): the reference's
  startup (626 lines) plus temper's member parser and the problems'
  lines pass 600 lines. 10.1a lands the `application` parser, temper's
  problems and their lines; 10.1b lands `named` and `startup`. "10.1's
  startup" (16.5's and 17's Needs) means 10.1b landed.
- **temper's problems wrap jig's.** jig's `What` is closed and has no word
  for standalone's shape, a limit against another or a part's bytes; so
  `startup::Problems` holds `Problem { path: jig's Path, what }` with
  `What::Jig(jig's What)` beside temper's reasons (the sketch), capped at
  jig's `PROBLEMS` (32). 08's and 09's constructors return jig's
  `Problems`; `Problems::jig` takes them in under their path's prefix.
- **The lines** are step code (`startup::lines`), written with skein-lib's
  `Writer` and `Decimal`: `PATH: WORDS\n` per problem, members joined by
  `.` and indexes as `[N]`, then `and N more\n`. The binary prints each
  line after `temper serve: `. temper's words, exactly: `Standalone`
  "standalone takes one worker, joined to the engine by the in-process
  name \"workers\""; `Loopback` "must be a socket address on loopback";
  `Connectors` "must be [1], the forge connector"; `Absolute` "must be an
  absolute path"; `NotBuilt` "reserved: not built yet"; `Profile` "the
  only profile is \"small\""; `Override(Unknown)` "no limit of temper's
  has this name"; `Override(Range)` "outside its type";
  `Against { other }` "must agree with OTHER"; `Past { bytes, bound }`
  "N bytes, past its bound of M"; `Agent` "not in the agent's
  configuration under the same name and account"; jig's `What`s in words
  of jig's `protocol/configuration.md` for the refusal, one arm each
  (`Missing` "missing", `Unknown` "unknown member", `Duplicate`
  "duplicate", `Bound` "past its bound", `Unreadable` "not read",
  `Memory { over }` "the worst case passes memory_bytes by N bytes"; the
  others as the author words them, one table).
- **Two passes, as the reference's.** `named` parses everything and runs
  every check that needs no file's bytes; `startup` parses again with
  the files' bytes. temper's members are first `application::Members`
  (files by path), then `resolve` gives 08.1's `Forge` and `Identity`
  with their values; a secret's final newline is dropped by
  `jig_protocol_config::secret` (`forms.rs:28`).
- **`named` returns jig's `File`s** (`files.rs`: `at`, `path`, `max`,
  `private`), temper's beside jig's, every `at` from the document's root,
  so a refused read names the member. The binary reads a private file
  with `read_private_file` and another (a charter's instructions, a trust
  file) with `read_file`, by `private`; the reference read every one as
  private (findings/jig-service.md, 3).
- **Bounds of temper's files:** an identity's token, the grant bound
  (`jig_protocol_worker_wire::CEILINGS.grant_value`, 8,192, until the
  whole value's check in 10.2); an API token, `API_TOKEN_BYTES` (4,096)
  until 14.1's `forge_component.token_bytes` replaces it (14.1 removes the
  constant); a trust file, 1,048,576 (the configuration's bound).
- **No `service` member** (findings/jig-service.md, Discrepancy 4): the
  accounts', workers' and clients' components are always made; a
  document with `service` is refused as an unknown member.
- **The worker's root** (configuration.md, 4; Discrepancy 6): the binary
  resolves `application.workers[0].root` against its working directory
  into an absolute path, refuses it when it exists and is a link, makes
  it with `skein_shell::make_root(path, 0o700)`, then refuses it unless
  `std::fs::symlink_metadata` says a directory owned by the effective
  user with no group or other bits; `WorkerRuntime { root, path }` (09.2)
  carries the opened root and that path.
- **The agent's profile is the binary's to parse,** as the reference's:
  `smith_agent_shell::config::read`, then `jig-test`'s `agent()`
  (`lib.rs:73-116`) copied into 09's `settings::Agent`; 10.2 checks its
  names against jig's engine part.
- **Its own tests' scratch:** `CARGO_TARGET_TMPDIR` exists only for
  integration tests, so the binary's file tests are
  `crates/temper/tests/startup.rs` (in the focused suite), and
  `src/tests.rs` keeps the tests that touch no file (decision 13).
- **The process tier reads no environment variable but section 1's**
  (code-organization.md, 1): the support passes the binary an empty
  environment (jig's passed `PATH`), takes its deadline as an argument
  (jig's read `NEXTEST_PROFILE`), and finds git at `/usr/bin/git`,
  refusing to run without it, naming it.
- **The process tier's configuration** is jig's channel world's, as jig's
  support makes it (`support/mod.rs:34-65`): `engine_document()`,
  `WORKER_DOCUMENT` and `files(false)` (dev-dependency
  `jig-channel-world`), patched as jig's are, with `service` removed,
  `engine.connectors` `[1]`, `application` temper's (one forge whose API
  is `127.0.0.1:9` over plaintext, which no call reaches until 15.3; one
  identity; no landing rule; charter 1; profile `small`; the worker's
  root `worker-root` and git `/usr/bin/git`), its secret files written
  `0o600`.

## Rules that bind it

- "**Scratch directories** of tests are made under
  `env!("CARGO_TARGET_TMPDIR")`, never under `/tmp`"; "No other variable
  is read by temper's code or tests" than section 1's
  (code-organization.md, sections 5 and 1).
- "**The process and live tiers run only where a Proof line or 'Done
  when' says,** once; never in the gate"
  (README, Rules). The default and fuzzy filters leave `process` and
  `live` out; `scripts/check.sh` compiles them with clippy and runs
  neither.
- "**Keeping the pins.** Adding to any `Cargo.toml` a crate of jig, smith
  or skein that `Cargo.lock` does not hold yet ... The increment that
  adds one then runs, in this order," the three `cargo update --precise`
  commands, and the check prints the three pinned revisions
  (code-organization.md, section 3).

## Increments

### 10.1a The `application` member and the problems' lines

Scope: `crates/temper-service/src/startup/{problems.rs,
application.rs}` (step code): temper's `Problem`, `Problems`, `What` and
`lines`; `application::parse` reading `application` at its token with
`jig_protocol_config::read`'s `Object`, `text`, `integer`, `boolean`,
`list` (configuration.md, 3, 4 and 10): `forges` (at least 1, at most
the forge client's `repositories`; `number` unique; `kind` `"forgejo"`;
`api` jig's address form, never in-process; `transport` `"plaintext"`
or `{"tls": {"server_name", "trust"}}`; `git` an `http` or `https` URL
with no user, query, fragment or final `/`, `http` only to a loopback
host; `api_token` a path; `user` above 0; `identity` naming one
identity, each identity named by one forge), `identities` (as many as
`forges`, at most jig's `core.accounts.credentials`; `number` unique;
`login` not empty, no `:` or control byte; `author.name` and
`author.email` not empty, no NUL, newline, `<` or `>`), `landing`
(`deployment` and `projects`, each rule's members of
`temper_engine_domain`'s `LandingRule`, its `pattern` read as jig's
`projects.rs:336` reads one; a project at most once and one of
`engine.projects`; each approval's `role` a role of its project's
policy, of every project's for a deployment rule; `kind` 4), `charter`
(one of `engine.charters`), `limits` (`profile` `"small"`, then each
`overrides` member through 08.1's `limits::apply`), `workers` (one per
top-level worker: `root`, `git` absolute), `webhooks` refused as
`NotBuilt`; `application::files` and `resolve`.
Needs: 09.3's one process (the session's start); 08.1's `settings` and
`limits`.
Files: `crates/temper-service/{src/{lib.rs, startup.rs,
startup/problems.rs, startup/application.rs, tests.rs,
tests/fixtures.rs, tests/startup.rs}}`. No existing type changes Shape.
Shape: the sketch's `problems` and `application` modules; `startup.rs`
holds `mod problems; mod application;` and re-exports `Problem`,
`Problems`, `What`, `lines`.
Test shape: step tests in `src/tests/startup.rs`; fixtures in
`src/tests/fixtures.rs` gain `APPLICATION: &[u8]` (configuration.md,
10's `application` member, complete, its landing rule's project 1),
`document() -> Box<[u8]>` (08's `ENGINE` fixture as `engine`, one worker as
jig's `WORKER_DOCUMENT` with `engine` `{"in_process":"workers"}` and
absolute agent paths, and `APPLICATION`), `parsed() ->
skein_json::Document`, `files() -> settings::Files` (each token's
bytes). Tests: `the_standalone_example_reads_into_its_members`,
`an_unknown_missing_or_reserved_member_is_named_by_its_path` (rows:
`application.forges[0].usr` unknown, `application.forges[0].user`
missing, `application.webhooks` not built, `application.limits.profile`
`"large"`), `an_override_by_dotted_name_is_set_or_refused` (rows:
`forge.repositories` 3 set; `forge.nothing` unknown; `checkout.name_bytes`
past a `u32` out of range), `a_landing_rule_reads_jigs_pattern_form`,
`a_git_base_with_a_user_or_a_final_slash_is_refused`,
`problems_print_one_line_each_with_their_path`. Under 20 ms each.
Reference test, in full:

```rust
// crates/temper-service/src/tests/startup.rs
#[test]
fn problems_print_one_line_each_with_their_path() {
    let mut problems = Problems::empty();
    let forge = Path::root().member(b"application").member(b"forges").child(Segment::Index(0));
    problems.push(forge.member(b"user"), What::Jig(config::What::Missing));
    problems.push(Path::root().member(b"application").member(b"webhooks"), What::NotBuilt);
    problems.push(
        Path::root().member(b"engine").member(b"memory_bytes"),
        What::Jig(config::What::Memory { over: 4_096 }),
    );
    assert_eq!(
        &*lines(&problems),
        b"application.forges[0].user: missing\n\
          application.webhooks: reserved: not built yet\n\
          engine.memory_bytes: the worst case passes memory_bytes by 4096 bytes\n"
            .as_slice()
    );
}
```

Proof: unit, `cargo nextest run -p temper-service -E
'test(/tests::startup::/)'`, run once: configuration.md, 10's example
reads into its members, and an unknown, missing or reserved member, a
refused override and a malformed `git` are each named by their path on
one line (the six tests above); under 0.5 s. The process layer opens at
10.4.
Don't: write a parser for jig's parts (jig-protocol-config's);
`format!` a line (step code); accept a missing member for an absent
option (`null` only).
Size: at most 450 lines.
Removes: nothing.

### 10.1b `named` and `startup`

Scope: `crates/temper-service/src/startup.rs`: `named` (the reference's
`worker_specs` and `engine_named`: the document's three members and
nothing else, jig's `engine` and `workers[0]` by `jig_protocol_config::
engine` and `worker`, their problems under `engine` and `workers[0]`,
`application` by 10.1a, then jig's `files` and `worker_files` and
`application::files`, each `at` from the root); `startup` (parse again;
`check_files` as the reference's `:276`, each file at most its `max`;
`resolve`; 08.1's `settings::Engine` and `settings::Worker` filled;
08's engine layer made, 09.1b's `Worker::new` with `runtime` and the
`Files` `check_files` read (ruling 127), 09.3's
crate-visible constructor joining them; jig's `Problems` from either
taken in with `Problems::jig`); `API_TOKEN_BYTES`. `lib.rs` exports
`startup::{named, startup, Named}`.
Needs: nothing beyond 10.1a.
Files: `crates/temper-service/src/{lib.rs, startup.rs, system.rs (its
constructor's visibility only), tests/startup.rs, tests/fixtures.rs}`.
No existing type changes Shape.
Shape: the sketch's `startup` module.
Test shape: `src/tests/startup.rs`, fixtures as 10.1a's plus 09.3's
`one_process` kernel answers. Tests:
`named_lists_each_file_with_its_bound_and_privacy`,
`named_returns_the_worker_root_and_the_agent_configuration`,
`a_file_named_but_not_given_refuses_the_start_naming_it`,
`jigs_problems_keep_their_paths_under_engine`,
`startup_makes_the_one_process_from_the_example`. Under 20 ms each.
Reference test, in full:

```rust
#[test]
fn a_file_named_but_not_given_refuses_the_start_naming_it() {
    let mut files = fixtures::files();
    files.entries = fixtures::without(files.entries, b"secrets/forge-1.git-token");
    let refused = startup(
        &fixtures::document(),
        &files,
        &fixtures::agent(),
        &fixtures::seeds(),
        None,
        fixtures::worker_runtime(),
    );
    let Err(problems) = refused else { panic!("a missing token refuses the start") };
    assert_eq!(
        &*lines(&problems),
        b"application.identities[0].token: not read\n".as_slice()
    );
}
```

(`fixtures::without` drops one entry by name; jig's `Unreadable` reads
"not read".)
Proof: unit, `cargo nextest run -p temper-service -E
'test(/tests::startup::/)'`, run once: `named` lists every file with its
bound and privacy and the two paths the binary opens, a missing file
refuses the start naming it, jig's problems keep their paths, and the
example makes the one process (the five tests above); under 0.5 s.
Don't: read a file (the binary's); make a component before every check
has passed; read `service`.
Size: at most 250 lines.
Removes: nothing.

### 10.2 The checks across the document

Scope: `crates/temper-service/src/startup/checks.rs`, called by `named`
(`standalone`, `limits`, the remote) and by `startup` (`channel`,
`identities`, `agent`, `memory`), each pushing and going on:
standalone's requirements (configuration.md, 2), a client listener in
process refused too; the git component's rules (07.1's `check` against
the checkout's limits, each failure at `application.limits` naming the
pair, `git_component.operations` against `checkout.workspaces` and so
on); the channel's (channel.md, 8): each side's 03.1 `Bounds` (the
engine's from the forge's `resources_per_task` and `name_bytes`, the
longest remote and the grant bound; the worker's from the checkout's
`repositories`, `name_bytes`, `conflicts`, `path_bytes`), `payload::limits`
refused naming the limit past a ceiling, each of 03.1's `largest` within
`jig_protocol_worker_wire::CEILINGS` (`workspace_items`,
`work_left_some`, `work_saved_some`, `resource_resource_some`,
`grant_value`), the engine's largest items within the worker's, the
longest remote and the forge's `name_bytes` within the checkout's
`name_bytes`; each identity's whole value with 08.3's
`translate::identity`, past the grant bound refused at
`application.identities[i]`; the agent's names (configuration.md, 5)
through `Endpoints::resolve` and the agent's model bounds; one io's
sockets holding the engine's, the worker's link, `slots × 4` agent
entities and `git_component.operations × 4` git entities (09's question,
its default); the two worst cases against `memory_bytes` with
`jig_protocol_config::fits`, each "by how much".
Needs: nothing beyond 10.1b; 07.1's `check` and 03.1's `limits` and
`largest` (both landed before 09).
Files: `crates/temper-service/src/{startup.rs, startup/checks.rs,
tests/checks.rs, tests/fixtures.rs}`, and `system.rs` for one
`pub(crate) fn startup_worst_case`, which `Service::new` calls too
(decisions.md, ruling 130). No existing type changes Shape.
Shape: the sketch's `checks` module.
Test shape: `src/tests/checks.rs`, fixtures as 10.1a's (rows patch
`fixtures::document()` with `fixtures::with(document, path, json)`, made
here: the member at a path replaced, its bytes spliced where skein-json's
tokens put it). Tests: `standalone_shape_is_required_of_jigs_parts`
(rows: two workers; a worker's `engine` a socket; `hosts.listener` a
socket; `clients.listener` in process; `clients.listener`
`10.0.0.1:7100`; `connectors` `[1, 2]`; an agent program `temper`;
each with its path and `What`),
`a_remote_past_the_checkouts_names_refuses_the_start`,
`the_engines_items_past_the_workers_refuse_the_start`,
`an_identity_past_the_grant_bound_is_refused_naming_it`,
`an_endpoint_or_model_missing_from_the_agent_is_named`,
`git_component_limits_against_the_checkout_name_both`,
`a_worst_case_past_memory_bytes_says_by_how_much`. Under 20 ms each.
Reference test, in full:

```rust
// crates/temper-service/src/tests/checks.rs
#[test]
fn a_remote_past_the_checkouts_names_refuses_the_start() {
    let document = fixtures::with(
        &fixtures::document(),
        b"application.limits.overrides",
        br#"{"checkout.name_bytes": 300}"#,
    );
    let Err(problems) = named(&document) else { panic!("a remote past 300 bytes refuses the start") };
    assert_eq!(
        &*lines(&problems),
        b"application.limits.overrides.checkout.name_bytes: must agree with the longest remote\n".as_slice()
    );
}
```

(The longest remote under the fixture's forge, `https://forge.example`
and `name_bytes` 256: 21 + 1 + 256 + 1 + 256 + 4 = 539 bytes, past 300,
while the forge's 256-byte names fit; `Against` names `the longest
remote` for this check.)
Proof: unit, `cargo nextest run -p temper-service -E
'test(/tests::checks::/)'`, run once: standalone's shape, the remote,
the channel's parts, an identity's value, the agent's names, the git
component's limits and the worst case each refuse a start naming their
paths (the seven tests above); under 0.5 s.
Don't: move a profile number to make a check pass without saying so in
the report (08's question on the profile's numbers); check a forge
component's limit (14.1 adds them among these checks).
Size: at most 500 lines.
Removes: nothing.

### 10.3 The binary: `temper serve` and `temper agent`

Scope: `crates/temper` (ordinary Rust; `Cargo.toml` with `[[bin]] name
= "temper"`, its `[lints]` and `clippy.toml` copied from jig's
`testing/jig-test`): `main.rs`, `lib.rs` (`configure`, `run`, `serve`,
`prepare`), `host.rs`, copied from jig-test (`lib.rs:25-283`, `host.rs`)
with: the usage line `usage: temper serve CONFIG.json | temper agent
AGENT.json`; the configuration read with `read_file` within 1,048,576
bytes; `named`, then each `File` read by its `private` flag, a refused
read printed as `temper serve: AT: PATH: WHY` (AT the member's path,
WHY `absent`, `not a file`, `past N bytes`, `readable by others or
reached through a link`, or the errno); the agent from smith's agent
configuration (jig-test's `agent()`); the accounts' runtime always
(jig-test's `accounts()`); the worker's root (Choices); seeds as
jig-test's; `startup`, its problems as `lines`, each after `temper
serve: `; the deployment directory with `make_root(.., 0o700)`; signals,
the kernel, `drive`. `Host::exit`: 0 for `End::Drained` with every link
written, 1 otherwise; `End::Stopped(Stop::Start(problems))` printed with
`lines` before the exit (an unusable git, 09.2 and 09.3; 14.1 adds
`Stop::Forge`'s line). Every error `run` returns is whole lines, each
beginning with its command (`temper serve: `, `temper agent: `), or
`temper: ` and the usage line; `main` prints it as it is. `temper agent
AGENT.json` is `smith_agent_shell::run`.
Needs: nothing beyond 10.2.
Files: `crates/temper/{Cargo.toml, clippy.toml, src/{main.rs, lib.rs,
host.rs, tests.rs}, tests/startup.rs}`; the workspace `Cargo.toml`
(member `crates/temper`); `Cargo.lock` (it gains `skein-shell`,
`skein-tls`, `skein-oauth-accounts`, `smith-agent-shell`,
`smith-local-service`, `smith-host-protocol` where absent: the pins'
three commands). Shared hotspot: the workspace `Cargo.toml`. No existing
type changes Shape.
Shape: the session's ordinary-Rust sketch of `crates/temper` below.
Test shape: `src/tests.rs` (no file):
`serve_without_a_configuration_path_is_a_usage_error`,
`an_unknown_command_prints_the_usage_line`. `tests/startup.rs` (a test
binary `startup`, in the focused suite; scratch under
`env!("CARGO_TARGET_TMPDIR")`/`startup/<case>`, made fresh by each test):
`a_configuration_file_past_its_bound_is_refused`,
`a_secret_file_readable_by_others_refuses_the_start_naming_it`,
`a_worker_root_reached_through_a_link_is_refused`,
`problems_reach_stderr_one_line_each`. Under 50 ms each. Reference test,
in full:

```rust
// crates/temper/tests/startup.rs
#[test]
fn a_secret_file_readable_by_others_refuses_the_start_naming_it() {
    let directory = scratch("public-token");
    support::configuration(&directory, &support::Forge::Unreached);
    let token = directory.join("secrets/forge-1.git-token");
    fs::set_permissions(&token, fs::Permissions::from_mode(0o644)).expect("public permissions");
    let refused = temper::prepare_in(&directory).err().expect("start refused");
    assert_eq!(
        refused,
        "temper serve: application.identities[0].token: secrets/forge-1.git-token: \
         readable by others or reached through a link"
    );
    assert!(!refused.contains("forge-1-git-token-value"), "no secret in a diagnostic");
}
```

(`tests/startup.rs` declares `mod support;` and uses 10.4's
`configuration`; until 10.4 lands, 10.3 writes `support::configuration`
and `support::Forge` there, and 10.4 grows the module.
`temper::prepare_in(directory)` is `prepare` with the configuration read
from `directory/config.json` and paths resolved against `directory`, a
public function for this test; `prepare` itself calls it with `.`.)
Proof: unit, `cargo nextest run -p temper`, run once: a usage
error, a configuration past its bound, a public token file and a linked
worker root are refused with their lines on stderr (the six tests
above); under 1 s. The process layer opens at 10.4.
Don't: add `open` or `sign-in` (16.4, 16.5), or a placeholder for them;
print a problem as `Debug`; read an environment variable: jig-test's
`SSL_CERT_FILE` (`lib.rs:142`) becomes `Trust::Machine { ssl_cert_file:
None }`, the distribution's paths (code-organization.md, 1).
Size: at most 350 lines.
Removes: nothing.

### 10.4 The process tier's support and its first test

Scope: `crates/temper/tests/support/mod.rs` (after jig-test's): `Scratch`
(under `env!("CARGO_TARGET_TMPDIR")`/`process/<case>`, removed first if
present, removed by `remove()` at a test's end, left for reading when the
test panics); `configuration(directory, &Forge)` (Choices: jig's channel
world's fixtures patched, temper's `application`, the secrets `0o600`,
`root.der`, `agent.json` as jig's `agent_configuration`); `Forge::
Unreached` (15.3 adds the throwaway Forgejo's); `serve(directory) ->
Binary` (`temper serve config.json`, pipes, an empty environment,
stderr up to 65,536 bytes); `fake_llm(scripts) -> skein_fake_peers::llm::Peer`
(`smith_agent_process_world::fake::configured_transport` on
`127.0.0.1:34443` over TLS, as jig's process test makes it);
`printed_link`; the `Process` host enum (binary, stdout reader, person
(12.2), peer) and its `Judge`; `run(...)` with its deadline as an
argument. `tests/process.rs` (`[[test]] name = "process"`) with one test.
`.config/nextest.toml`: the default and fuzzy filters gain `& not
binary(/^(process|live)$/)`; `[profile.process]` (`default-filter =
"binary(=process)"`, `global-timeout = "10m"`, `test-threads = 1`; the
tier's tests aim at 5 minutes together);
`[profile.live]` (`default-filter = "binary(=live)"`, `global-timeout =
"20m"`, `test-threads = 1`, `retries = 0`). `scripts/proof-one-process.sh`
(the sketch below).
Needs: nothing beyond 10.3.
Files: `crates/temper/{Cargo.toml ([[test]] process; dev-dependencies
`skein-world`, `skein-fake-peers`, `skein-tls-world`,
`skein-fake-llm-domain`, `skein-llm`, `smith-agent-process-world`,
`jig-channel-world`, `serde_json`), tests/{process.rs, support/mod.rs,
startup.rs}}`; `.config/nextest.toml`; `scripts/proof-one-process.sh`,
mode 100755 (`git ls-files -s scripts/proof-*.sh` shows 100755);
`Cargo.lock` (the pins' three commands). Shared hotspots:
`.config/nextest.toml`, the workspace `Cargo.toml`.
Shape: the session's ordinary-Rust sketch of the support below.
Test shape: `tests/process.rs`:
`temper_serve_prints_its_link_and_drains_on_a_signal`, aiming at 20 s,
at most 60 s. Reference test, in full:

```rust
// crates/temper/tests/process.rs
mod support;
use skein_io::kernel::{Exit, Signal};
use skein_lib::Duration;

#[test]
fn temper_serve_prints_its_link_and_drains_on_a_signal() {
    let scratch = support::Scratch::new("drain");
    support::configuration(scratch.path(), &support::Forge::Unreached);
    let ended = support::until_link_then(scratch.path(), Signal::Terminate, Duration::from_secs(60));
    let link = support::printed_link(&ended.stdout).expect("one printed line");
    assert!(link.starts_with(b"http://127.0.0.1:"), "a loopback link: {}", String::from_utf8_lossy(link));
    assert!(
        skein_lib::bytes::find(link, b"/jig/v1/sign-in/link?secret=").is_some(),
        "the deployment's sign-in link"
    );
    assert_eq!(ended.exit, Some(Exit::Code(0)), "drained: {}", String::from_utf8_lossy(&ended.stderr));
    scratch.remove();
}
```

Proof: process, `scripts/proof-one-process.sh`, run once:
the real binary, started on a scratch directory from the configuration
the test wrote, prints its sign-in link on stdout and, on SIGTERM,
drains and exits 0 (the test above); under 2 minutes.
Don't: start a Forgejo (15.3's); use `tempfile` or `/tmp`; read
`NEXTEST_PROFILE` or `PATH`.
Size: test code; production 0 lines.
Removes: nothing.

## Interface sketches

`s10.rs` holds the step code's sketch, compiled; the binary and the
support are ordinary Rust, below it, not compiled. 08.1's `settings` and
`limits`, 09's `WorkerRuntime`, `settings::Agent` and `Service` are
imported from their sessions' sketches.

```rust
//! Sketches of session 10: `temper-service`'s startup from configuration's
//! bytes (10.1a, 10.1b) and its checks across the document (10.2), step code
//! (configuration.md, sections 2 to 10). The binary `crates/temper` (10.3)
//! and the process tier's support (10.4) are ordinary Rust: their sketches
//! are in the session file only.
//!
//! Who owns what: startup owns parsing the document and every check of
//! configuration.md, 2 and 7; 08.1 owns the settings it builds; 08 and 09
//! own the layers it makes from them; 09.3 owns the one process it returns.
//! What ends what: a refused start ends in `Problems`, which `temper serve`
//! prints with `lines`, one per line on stderr, before io starts, and exits 1.
#![expect(unused_variables, reason = "a sketch's bodies are unreachable; its parameters are its signature")]

/// `crates/temper-service/src/startup/problems.rs` (10.1a): a refused start's
/// problems, each at its path from the document's root, and their lines.
pub mod problems {
    use crate::s08::limits;
    use alloc::boxed::Box;
    use jig_protocol_config as config;
    use skein_lib::List;

    /// Why a member refuses the start: jig-protocol-config's reasons, and
    /// the ones temper's checks add (configuration.md, 2, 3, 5 and 7).
    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    pub enum What {
        /// jig-protocol-config's, for jig's parts and for its readers' failures.
        Jig(config::What),
        /// Standalone's shape is not met: one worker, joined by `workers`.
        Standalone,
        /// An address that must be a socket address on loopback is not.
        Loopback,
        /// `engine.connectors` is not `[1]`, the forge connector.
        Connectors,
        /// A path that must be absolute is not.
        Absolute,
        /// A reserved member is present (`application.webhooks`): not built.
        NotBuilt,
        /// A profile other than `small`.
        Profile,
        /// An override 08.1's `limits::apply` refuses.
        Override(limits::Refused),
        /// A limit that must agree with another: its dotted name, or `the
        /// longest remote` for the remote's check.
        Against { other: &'static [u8] },
        /// Bytes past their bound: a part, a remote, an identity's value.
        Past { bytes: u64, bound: u64 },
        /// Not in smith's agent configuration under the same name and account.
        Agent,
    }

    /// One member that refuses the start, at its path from the root.
    #[derive(Clone, PartialEq, Eq, Debug)]
    pub struct Problem {
        pub path: config::Path,
        pub what: What,
    }

    /// Every problem found: at most `config::PROBLEMS`, and how many more.
    #[derive(Clone, PartialEq, Eq, Debug)]
    pub struct Problems {
        pub problems: List<Problem>,
        pub past: u32,
    }

    impl Problems {
        /// No problem yet, with room for `config::PROBLEMS`.
        #[must_use]
        pub fn empty() -> Problems {
            unreachable!("sketch")
        }
        /// One more; past the cap, counted in `past`.
        pub fn push(&mut self, path: config::Path, what: What) {
            unreachable!("sketch")
        }
        #[must_use]
        pub fn is_empty(&self) -> bool {
            unreachable!("sketch")
        }
        /// jig's problems (a parser's, 08's or 09's constructors'), each as
        /// `What::Jig`, its path under `prefix` (`engine`, `workers[0]`).
        #[must_use]
        pub fn jig(found: config::Problems, prefix: &config::Path) -> Problems {
            unreachable!("sketch")
        }
    }

    /// stderr's lines: `PATH: WORDS\n` per problem, members joined by `.`
    /// and indexes as `[N]` (`application.forges[0].user: missing`), then
    /// `and N more\n` when `past` is above 0; measured, then written with
    /// skein-lib's `Writer` and `Decimal`.
    #[must_use]
    pub fn lines(problems: &Problems) -> Box<[u8]> {
        unreachable!("sketch")
    }
}

/// `crates/temper-service/src/startup/application.rs` (10.1a): the
/// `application` member, read with jig-protocol-config's readers by
/// configuration.md, 10's format rule. Crate-visible: startup's own.
pub mod application {
    use super::problems::Problems;
    use crate::s08::{limits, settings};
    use alloc::boxed::Box;
    use jig_protocol_config as config;

    /// temper's members as the document writes them (configuration.md, 3
    /// and 4), files by path; `resolve` turns them into 08.1's settings.
    #[derive(Debug)]
    pub struct Members {
        pub forges: Box<[Forge]>,
        pub identities: Box<[Identity]>,
        pub landing: temper_engine_domain::LandingPolicy,
        pub charter: u32,
        pub limits: limits::Limits,
        pub workers: Box<[Worker]>,
    }

    /// One entry of `application.forges`, its token and trust by path.
    #[derive(Debug)]
    pub struct Forge {
        pub number: u16,
        pub kind: settings::Kind,
        pub api: config::Address,
        pub transport: Transport,
        pub git: Box<[u8]>,
        /// The API token's private file.
        pub api_token: Box<[u8]>,
        pub user: u64,
        pub identity: u32,
    }

    /// A forge's `transport`, its trust file by path; `None` is the
    /// machine's roots.
    #[derive(Debug)]
    pub enum Transport {
        Plaintext,
        Tls { server_name: Box<[u8]>, trust: Option<Box<[u8]>> },
    }

    /// One entry of `application.identities`, its token by path.
    #[derive(Debug)]
    pub struct Identity {
        pub number: u32,
        /// The token's private file.
        pub token: Box<[u8]>,
        pub login: Box<[u8]>,
        pub author: settings::Author,
    }

    /// One entry of `application.workers`.
    #[derive(Debug)]
    pub struct Worker {
        /// The worker's root directory, as written.
        pub root: Box<[u8]>,
        /// The git program, an absolute path.
        pub git: Box<[u8]>,
    }

    /// Reads `application` at token `at`, against jig's parsed engine part
    /// (its projects, charters and connectors), pushing every problem.
    pub fn parse(
        document: &skein_json::Document,
        at: u32,
        jig: &config::Engine,
        problems: &mut Problems,
    ) -> Option<Members> {
        unreachable!("sketch")
    }

    /// Each private file temper's members name, at its path in the document:
    /// API tokens and identities' tokens private, trust files not.
    #[must_use]
    pub fn files(members: &Members, api_token_bytes: u32, token_bytes: u32) -> Box<[config::File]> {
        unreachable!("sketch")
    }

    /// The members whose values come from files, as 08.1's settings hold them.
    #[derive(Debug)]
    pub struct Resolved {
        pub forges: Box<[settings::Forge]>,
        pub identities: Box<[settings::Identity]>,
    }

    /// The members with their files' bytes; a file `files` lacks, or one
    /// past its bound, is `What::Jig(Unreadable)` or `Jig(Bound)` at its path.
    pub fn resolve(members: &Members, files: &settings::Files, problems: &mut Problems) -> Option<Resolved> {
        unreachable!("sketch")
    }
}

/// `crates/temper-service/src/startup.rs` (10.1b): the two calls the binary
/// and the worlds make, as jig's reference startup's.
pub mod startup {
    use super::problems::Problems;
    use crate::s08::{service::AccountsRuntime, settings};
    use crate::s09;
    use alloc::boxed::Box;
    use jig_protocol_config as config;

    /// An API token's bound until 14.1's `forge_component.token_bytes`
    /// replaces it (configuration.md, 6).
    pub const API_TOKEN_BYTES: u32 = 4_096;

    /// What `temper serve` reads and opens before `startup`.
    #[derive(Debug)]
    pub struct Named {
        /// Each file the document names, jig's parts' and temper's, at its
        /// path in the document, with its bound and whether it is private.
        pub files: Box<[config::File]>,
        /// jig's `engine.deployment.directory`: the store's, made 0o700.
        pub deployment: Box<[u8]>,
        /// `application.workers[0].root` as written: made 0o700 if absent.
        pub worker_root: Box<[u8]>,
        /// `workers[0].agent.configuration`: smith's agent configuration.
        pub agent: Box<[u8]>,
    }

    /// Parses the document and runs every check that needs no file's bytes;
    /// `temper sign-in` (16.5) checks a document with it.
    pub fn named(document: &[u8]) -> Result<Named, Problems> {
        unreachable!("sketch")
    }

    /// The one process: the document parsed again, the files' bytes taken,
    /// every check of 10.2 run, then 08's engine layer, 09's worker and
    /// 09.3's composition made. Nothing is made when a check fails.
    pub fn startup(
        document: &[u8],
        files: &settings::Files,
        agent: &s09::settings::Agent,
        seeds: &settings::Seeds,
        accounts: Option<AccountsRuntime>,
        runtime: s09::WorkerRuntime,
    ) -> Result<s09::Service, Problems> {
        unreachable!("sketch")
    }
}

/// `crates/temper-service/src/startup/checks.rs` (10.2): the checks across
/// the document (configuration.md, 2, 5 and 7; channel.md, 8). Each pushes
/// its problems and goes on, so one start names them all.
pub mod checks {
    use super::problems::Problems;
    use crate::s03::temper_protocol_worker_wire as payload;
    use crate::s08::{limits, settings};
    use crate::s09;
    use jig_protocol_config as config;

    /// Standalone's requirements on jig's parts: one worker, its `engine`
    /// `{"in_process":"workers"}`; `hosts.listener` exactly that name and
    /// `hosts.workers` that worker; `clients.listener` a loopback socket;
    /// `connectors` `[1]`; the agent's paths absolute.
    pub fn standalone(engine: &config::Engine, workers: &[config::Worker], problems: &mut Problems) {
        unreachable!("sketch")
    }

    /// The limits against each other: the git component's rules (07.1's
    /// `check`), named by both dotted names.
    pub fn limits(limits: &limits::Limits, git: &crate::s07::Limits, problems: &mut Problems) {
        unreachable!("sketch")
    }

    /// The longest remote the routing can make: the longest `git` base, `/`,
    /// an owner and a name of the forge's `name_bytes`, `/` between, `.git`.
    #[must_use]
    pub fn longest_remote(forges: &[settings::Forge], name_bytes: u32) -> Option<u64> {
        unreachable!("sketch")
    }

    /// channel.md, 8: each side's `payload::limits` within jig's ceilings;
    /// each of temper's largest parts within jig's worker wire `CEILINGS`;
    /// the engine's largest items within the worker's; the longest remote,
    /// the forge's names and branches within the checkout's `name_bytes`.
    pub fn channel(
        engine: &payload::Bounds,
        worker: &payload::Bounds,
        forges: &[settings::Forge],
        limits: &limits::Limits,
        problems: &mut Problems,
    ) {
        unreachable!("sketch")
    }

    /// Each identity's whole value (token, login, author), encoded as the
    /// channel encodes it, within the grant bound; named by its index.
    pub fn identities(identities: &[settings::Identity], limits: &payload::v1::Limits, problems: &mut Problems) {
        unreachable!("sketch")
    }

    /// configuration.md, 5: each of `engine.llm.endpoints` in the agent's
    /// configuration under the same name and account; each charter's
    /// models among the agent's model bounds.
    pub fn agent(engine: &config::Engine, agent: &s09::settings::Agent, problems: &mut Problems) {
        unreachable!("sketch")
    }

    /// One worst case per service: the engine's layer and the worker's, each
    /// with jig's sum, against its `memory_bytes` (`config::fits`).
    pub fn memory(engine: u64, engine_memory: u64, worker: u64, worker_memory: u64, problems: &mut Problems) {
        unreachable!("sketch")
    }
}
```

The binary, ordinary Rust, not compiled:

```rust
// crates/temper/src/lib.rs (ordinary Rust, not compiled): jig-test's, copied.
pub const USAGE: &str = "usage: temper serve CONFIG.json | temper agent AGENT.json";
const CONFIG_BYTES: u32 = 1 << 20;

/// The command a line of arguments names; 16.4 and 16.5 add `Open` and `SignIn`.
#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    Serve { configuration: PathBuf },
    Agent { configuration: PathBuf },
}
pub fn configure(args: impl Iterator<Item = OsString>) -> Result<Command, String>;
/// The exit code, or the error's lines, which `main` prints as they are.
pub fn run(command: Command) -> Result<u8, String>;
/// Everything before the loop, in the working directory `directory`:
/// read, `named`, files, the agent, the accounts, the worker's root, seeds,
/// `startup`, the deployment directory adopted. Problems as `lines` after
/// `temper serve: `, joined.
pub fn prepare_in(directory: &Path) -> Result<temper_service::Service, String>;

// crates/temper/src/host.rs: jig-test's `Host`, its `exit` 0 only for a
// drained end with every link written; a `Stop::Start` end's lines printed
// (14.1 adds `Stop::Forge`'s line).
pub struct Host { pub service: temper_service::Service, output: Box<dyn Write>, output_failed: bool }
impl skein_shell::Host for Host { /* iterate, queues, deadlines, exit, drain: jig-test's */ }
```

`scripts/proof-one-process.sh`, at most 20 lines, mode 100755:

```sh
#!/bin/sh
# The process tier (domain/testing.md, 2.6): the binary built first, then
# nextest's process profile, once. Run.
set -eu
cd "$(dirname "$0")/.."
export NEXTEST_DOUBLE_SPAWN=0
cargo build -p temper --bin temper
cargo nextest run -p temper --profile process
```

The process tier's support, ordinary Rust, not compiled:

```rust
// crates/temper/tests/support/mod.rs (ordinary Rust, not compiled)

/// A test's scratch directory under `CARGO_TARGET_TMPDIR`/process/<case>.
pub struct Scratch { path: PathBuf }
impl Scratch {
    /// Removes a leftover of the same case, then makes it.
    pub fn new(case: &str) -> Scratch;
    pub fn path(&self) -> &Path;
    /// At the test's end; a panicking test leaves it for reading.
    pub fn remove(self);
}

/// The forge the configuration names: one no call reaches (10.4), or the
/// throwaway Forgejo's instance and temper's account there (15.3).
pub enum Forge<'a> {
    Unreached,
    Throwaway { instance: &'a temper_throwaway_forgejo::Instance, user: &'a temper_throwaway_forgejo::Account },
}

/// Writes `config.json`, `agent.json`, `root.der`, the charters and every
/// secret file (`0o600`) into `directory`.
pub fn configuration(directory: &Path, forge: &Forge<'_>);
/// `temper serve config.json` in `directory`, pipes, an empty environment.
pub fn serve(directory: &Path) -> skein_world::end_to_end::Binary;
/// skein's fake LLM peer on 127.0.0.1:34443 over TLS with these scripts.
pub fn fake_llm(scripts: Box<[skein_fake_llm_domain::api::Script]>) -> skein_fake_peers::llm::Peer;
/// The first line of stdout, without its newline.
pub fn printed_link(stdout: &[u8]) -> Option<&[u8]>;

/// What a run ended with: the binary's exit, its stdout and stderr.
pub struct Ended { pub exit: Option<Exit>, pub stdout: Vec<u8>, pub stderr: Vec<u8> }
/// Starts the binary, reads stdout until its first line, then sends `signal`
/// and waits for its exit, within `deadline`.
pub fn until_link_then(directory: &Path, signal: Signal, deadline: Duration) -> Ended;

/// The real world's processes, each a `skein_shell::Host`; 12.2 adds the person.
enum Process { Binary(Box<Binary>), Stdout(Box<StdoutReader>), Peer(Box<skein_fake_peers::llm::Peer>) }
```

## Questions to raise when reached

- **The binary's environment.** The support passes an empty one. Default:
  if `temper serve` or `temper agent` needs a variable to start, pass
  `PATH` alone, as jig's support does, and say so in the report; a
  variable of temper's own is a question.
- **An identity token file's bound** before 10.2 checks the whole value.
  Default: `CEILINGS.grant_value` (8,192); the whole value's check is the
  binding one.
- **The worker's root through a link.** configuration.md, 4 refuses one;
  `make_root` follows links. Default: `symlink_metadata` before and after
  `make_root`, both refusing a link.
- **One io in one process** (09's question). Decided by ruling 132:
  the process's io is the engine part's `limits.io` with the worker
  part's added, as jig's reference startup composes them
  (`testing/jig-test-service/src/startup.rs:547-600`: `sockets` and
  `names` summed, `intake`, `receive`, `output` and `sends` the larger);
  10.2's check counts the entities the configuration enables against
  that sum.
- **A problem past jig's 32.** Default: `and N more`, as jig's `past`
  counts them.
- **The unreached forge once 14.1 lands.** The process tier's forge is
  `127.0.0.1:9`, where nothing listens. Default: the forge component's
  start check is answered unavailable and made again (credentials.md, 7:
  "an unanswered check is made again"), and the engine runs; if 14.1
  stops the engine on it, 14.1 raises it with this file's 10.4.

## Done when

- Startup, the checks and the binary pass: `cargo nextest run -p
  temper-service -p temper` passes.
- No `service` member is read: `git grep -n '"service"\|b"service"' --
  crates/temper-service/src crates/temper/src` prints nothing.
- Problems are never printed as `Debug`: `git grep -n ':?}' --
  crates/temper/src ':!crates/temper/src/tests.rs'` prints nothing.
- The suites leave the tiers out: `grep -c 'not binary(/^(process|live)\$/)'
  .config/nextest.toml` prints 2.
- The process tier ran once: `scripts/proof-one-process.sh`
  passed in 10.4's Proof, its log naming
  `temper_serve_prints_its_link_and_drains_on_a_signal`.
- The pins held: `grep -oE 'forgetic/(jig|smith|skein)\.git#[0-9a-f]+'
  Cargo.lock | sort -u` prints exactly the three pinned revisions.
- The gate passed on this session's last landing: `git -C
  ~/src/rust/temper log -1 --grep='^Step: temper-10\.4$'
  --format='%(trailers:key=Gate-Result,valueonly)' origin/main` names
  `scripts/check.sh passed`.
