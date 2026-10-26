# protocol: code organization

The crates this plan adds, what each holds and links, who owns each
interface other sessions use, where tests and fixtures go, what no
increment does, and the size limits. The design is `protocol.md`
(section 5 lists the crates) and the documents it reads in its section
2. Where a name here and the design differ, the design wins and the
difference is a question (development.md, 8.7).

## 1. Names

- **Steps** are `temper-NN.k` (`temper-08.2`); branches
  `protocol/temper-NN-k`; the commit trailers `Plan: protocol` and
  `Step: temper-NN.k`.
- **The design's names, in code too:** the routing, the forge component,
  the git component, temper's parts, the identities' table, the API
  token, the throwaway Forgejo, a transcript, locate, a keyed adoption.
  A type named for one of them carries that name (`IdentityTable`, not
  `SecretStore`).
- **The four version constants,** each `pub const` of its crate, `1`
  in this plan: `temper_protocol_worker_wire::PAYLOAD_VERSION: u32`,
  `temper_protocol_store::STORE_VERSION: u32`,
  `temper_protocol_client_wire::CLIENT_VERSION: u32`, and
  `temper_protocol_client_wire::FORGE: u16`, the forge connector's
  number, `1`. A version moves at most once in this plan (protocol.md,
  section 9); a moving increment says so in its commit body.
- **The environment variables:** `TEMPER_FORGEJO` (the throwaway
  Forgejo's binary), `TEMPER_FORGEJO_RUNNER` (forgejo-runner, Actions
  facts only), `TEMPER_LIVE_MODEL` (the live tier's model, optional),
  `TEMPER_RUN_LOGS` (where the live script writes its private account
  file and its logs), `SKEIN_CODEGEN_WRITE=1` (a drift test writes the
  generated code and goldens again, as jig's client wire's does).
  temper's code and tests read no other variable but cargo's
  (`CARGO_TARGET_TMPDIR`, `CARGO_BIN_EXE_temper`, `CARGO_MANIFEST_DIR`)
  and `HOME` (`scripts/proof-live.sh` only).
- **Words no code uses** (the Pulse block of `README.md`): `legacy`,
  `compat`, `shim`, `hack`, `workaround`, `TODO`, `FIXME`. Nothing takes
  their place: code says what it does.

## 2. Crates this plan adds

Kind: **step** is step code under the subset (programming-model.md,
section 10; the workspace's `[lints]`, `clippy.toml`, `#![cfg_attr(not(test),
no_std)]` and `#![forbid(unsafe_code)]` at its root); **ordinary** is
ordinary Rust with a `[lints]` table and a `clippy.toml` of its own,
copied from the named precedent. A world is ordinary, as every crate
under `tests/` is, with `tests/clippy.toml`.

| Path | Package | Kind | Holds | Made by |
|---|---|---|---|---|
| `crates/temper-protocol-worker-wire` | same | step | temper's parts on jig's channel and a git identity's value: one schema, generated code, goldens, `PAYLOAD_VERSION` (channel.md) | 03.1 |
| `crates/temper-protocol-client-wire` | same | step | the forge family's client records: `Adopt`, its outcome, `FORGE`, `CLIENT_VERSION` (client.md, 4.6) | 03.2 |
| `crates/temper-protocol-store` | same | step | temper's families in jig's store: key encoders, one schema, translations, sizes, the interval, `STORE_VERSION` (store/README.md) | 04.1 |
| `crates/temper-forge-forgejo` | same | step | Forgejo's documents, both sides: requests as sized encoders, answers as decoders over skein-json's tokens, error bodies; the server's side for the fake forge (forge.md, 2 and 3) | 06.1 |
| `crates/temper-worker-protocol-git` | same | step | the git component (git.md) | 07.1 |
| `crates/temper-service` | same | step | the engine's and the worker's services and their one-process composition, startup from configuration's bytes, temper's profile `small` (protocol.md, sections 5 to 8) | 08.1 |
| `crates/temper-engine-protocol-forge` | same | step | the forge component (forge.md) | 13.1 |
| `crates/temper-client-domain` | same | step | the client root (client.md, section 3) | 11.1 |
| `crates/temper-client-view` | same | step | the view root and the forge's view (client.md, section 5) | 11.2 |
| `crates/temper-shell-terminal` | same | ordinary, precedent jig's `crates/jig-shell-native/clippy.toml`; its modules `route`, `render`, `command` and `limits` are step code (client.md, section 2), held to the subset as section 6 below says | the native client, the client's routing, the terminal | 12.1 |
| `crates/temper` | same, binary `temper` | ordinary, precedent jig's `testing/jig-test/{Cargo.toml,clippy.toml}` | the four commands (protocol.md, section 4); the process and live tiers in `tests/` | 10.3 |
| `testing/temper-throwaway-forgejo` | same | ordinary, precedent `tests/clippy.toml` | starting, provisioning and removing the throwaway Forgejo, for the tools and the process tier (domain/testing.md, 2.6) | 05.1 |
| `testing/temper-fake-forge-protocol` | same | step | the fake forge's protocol layer over its domain (forge.md, section 9) | 14.3 |
| `testing/temper-fake-git` | same | ordinary, precedent `tests/clippy.toml` | the fake git program, a skein-world hosted program on the fake machine's files (domain/testing.md, 4.3) | 17.1 |
| `tools/forgejo-conformance` | same, binary | ordinary, precedent `tests/clippy.toml` | Forgejo's transcripts and facts (forge.md, section 10); replaces the Python tool | 05.2 |
| `tools/git-transcripts` | same, binary | ordinary, precedent `tests/clippy.toml`, `[lints]` as 05.1's | git's transcripts (git.md, section 9) | 05.4 |
| `tools/live-login` | same, binary | ordinary | reads the borrowed Codex login for `scripts/proof-live.sh` (domain/testing.md, 2.7) | 15.4 |
| `tests/codecs` | `temper-codecs-world` | world | drift and fuzzy tests of the three codec crates and of `temper-forge-forgejo`'s decoders | 03.1 |
| `tests/worker/protocol-git` | `temper-worker-protocol-git-world` | world | the transcripts' tests of git's builders and parsers (07.2) and the git component's machine world (07.4) (git.md, section 9) | 07.2 |
| `tests/engine/protocol-forge` | `temper-engine-protocol-forge-world` | world | the forge component's machine world (13.5) and the forge's protocol world (14.4) | 13.5 |
| `tests/client` | `temper-client-world` | world | temper's client joined to temper's engine root (domain/testing.md, 6.3) | 11.3 |
| `tests/one-process` | `temper-one-process-world` | world | the one-process world (domain/testing.md, 2.5) | 17.2 |

**Data, not crates:** `tools/forgejo-conformance/transcripts/` and
`facts.txt` (05.2, 05.3); `tools/git-transcripts/transcripts/` (05.4);
`scripts/proof-one-process.sh` (10.4) and `scripts/proof-live.sh`
(15.4), each at most 20 lines of `sh` that run cargo; `.config/nextest.toml`'s
profiles `process` and `live` (10.4).

**Retired** by 11.4, in one increment: `crates/temper-web-domain`,
`crates/temper-web-view`, `testing/temper-fake-person`,
`tests/web/domain`. **Removed** by 05.2: `tools/forgejo-conformance/*.py`.

## 3. Who links what

- **Codec crates** link `skein-lib` and `skein-codec`, and, for the store
  crate only, `skein-kv`, `jig-protocol-store` and
  `temper-engine-domain-forge` (its translations' domain side), with the
  forge's three children whose types `Stored` and `Key` hold,
  `temper-engine-domain-forge-client`, `temper-engine-domain-forge-change`
  and `temper-engine-domain-forge-issues` (decisions.md, ruling 101). No
  codec crate links an engine or worker root.
- **`temper-forge-forgejo`** links `skein-lib` and `skein-json` only: no
  temper domain type (forge.md, section 2).
- **`temper-engine-protocol-forge`** links `skein-lib`, `skein-io`,
  `skein-http`, `skein-tls`, `skein-json`, `temper-forge-forgejo` and
  `temper-engine-domain-forge-client`.
- **`temper-worker-protocol-git`** links `skein-lib`, `skein-io` and
  `temper-worker-domain-checkout`. It decodes no channel record and reads
  no configuration (git.md, section 2).
- **`temper-client-domain`** links `skein-lib` and `jig-domain-client`;
  **`temper-client-view`** adds `jig-view` and `temper-client-domain`.
  Neither links a wire, the link, smith or an engine crate (client.md,
  section 2).
- **`temper-service`** links what protocol.md, section 5's table says.
  It never links `jig-protocol-charter` or `jig-protocol-tools` (llm.md,
  section 7); `temper-engine-smith` takes their place.
- **Tools and test support** may link `serde_json` (decisions.md, 10);
  no step crate does. No crate adds any other dependency from outside
  skein, smith and jig.
- **Keeping the pins.** Adding to any `Cargo.toml` a crate of jig, smith
  or skein that `Cargo.lock` does not hold yet moves that repository's
  git source to its newest commit, and the others with it. The increment
  that adds one then runs, in this order, and commits the lock:

  ```
  cargo update -p jig-domain-core --precise d91a89c45553f3f259b4d95d34a75164a9715877
  cargo update -p smith-host-domain --precise dfc0fe5700c628d7bce7da3d2ee918e9c1fbe46d
  cargo update -p skein-lib --precise b5d3a2c3363dbbd1db8dae933cd87a643a18b3f2
  ```

  The check, before merging: `grep -oE 'forgetic/(jig|smith|skein)\.git#[0-9a-f]+' Cargo.lock | sort -u`
  prints exactly three lines, ending `#d91a89c45553f3f259b4d95d34a75164a9715877`,
  `#dfc0fe5700c628d7bce7da3d2ee918e9c1fbe46d` and
  `#b5d3a2c3363dbbd1db8dae933cd87a643a18b3f2`. (The planner ran these
  three commands at the base after adding 30 such crates, and the check
  held.)

## 4. Shared interfaces and their owners

The owner defines it; every other session uses it by name. A sketch in
a session file shows it; landed code wins where they differ, unless the
design asks for the change.

| Interface | Owner | Users |
|---|---|---|
| the root relaying an undecodable host call's error with no commit | 01.1 | 08.3 |
| `temper_engine_domain::Parts.preparation` and the root's fact of a failed preparation | 01.2 | 08.2, 18.3 |
| the worker's parts by tag: `wire::Work.saved` as `wire::SavedRepository { tag, from_saved, landing }`, `wire::Preparation` naming a tag | 01.3 | 03.1 (field for field), 09.1, 18.3 |
| `api::Read::Locate { owner, name }`, `api::Answer::Located { repository }`; the fake forge domain's repository ids | 02.1 | 06.1, 13.3, 14.2 |
| the root's keyed application request (`PartyInput::Application`), its outcome token and `Delivery::Application` | 02.2 | 08.5b, 15.1a, 15.2 (the joint world's front) |
| `forge::Adoption` (no repository id), the adoption's outcome and its forge reasons, `NoSuchForge` among them | 02.3 (no such forge: 02.2) | 03.2 (field for field), 15.1a |
| `Stored::Application`, `Key::Application` (02.4); `forge::Limits.applications` (02.2) | 02.2, 02.4 | 04.4b, or 02.4 itself by README's rule "a durable record's codec follows its record" |
| `temper_protocol_worker_wire::{PAYLOAD_VERSION, v1::*}` | 03.1 | 08.3, 09.1a, 10.2, 18.3 |
| `temper_protocol_client_wire::{FORGE, CLIENT_VERSION, v1::*}` | 03.2 | 08.5b, 12.1, 15.1a |
| the `tests/codecs` world (drift, fuzzy, the transcripts' tests of `temper-forge-forgejo`) | 03.1 | 03.2, 04.2 to 04.5, 06.x |
| `temper_protocol_store::{STORE_VERSION, keys, records, application, interval}` | 04.1 to 04.5 | 08.2, 18.1 |
| the transcripts' layout and masking (`tools/forgejo-conformance/transcripts/<case>/{request,response}`; `tools/git-transcripts/transcripts/<case>/{argv,exit,stdout,stderr}`) | 05.2, 05.4 | 06.x, 07.2, 14.3, 17.1 |
| `temper_throwaway_forgejo::{Forgejo, Instance}` | 05.1 | 05.2 to 05.4, 14.5, 15.3 (the process support) |
| `temper_forge_forgejo`'s module layout and its conventions (paths, percent-encoding, times, ids, the client's and the server's sides) | 06.1 | 06.2 to 06.5, 13.x, 14.3 |
| `temper_worker_protocol_git::{Component, Request, Event, Limits, Settings, Identity}` | 07.1, 07.3a | 09.1a (`Identity`), 09.2 |
| `temper_worker_protocol_git::{invocation, output}` (the builders, the parsers) | 07.1, 07.2 | 17.1 |
| `temper_worker_protocol_git_world::{io::Below, transcripts}` | 07.2 (`transcripts`), 07.4 (`io::Below`) | 09.2, 17.1 |
| the engine world's forge stand (`ForgeStand`, `Driver::forge`) | 02.2 | 15.2 (a constructor taking the scenario's repositories) |
| `temper_service::settings::Agent` | 08.1 | 09.1b, 10.1b |
| `temper_service::settings::*` (the typed settings startup builds), `temper_service::limits` (the profile `small` and its overrides), the token prefixes of section 7 | 08.1 | 09.x, 10.x, 14.1, 17.2, 18.1 |
| `temper_service::{Service, iterate}` | 08.2d (the engine), 09.3 (one process) | 10.3, 17.2 |
| `temper_service::store::route` (the store's routing) | 08.2a | 18.1 |
| `temper_service::translate::{items, parts, identity}` (engine) | 08.3 | 18.3 |
| `temper_service::translate::{workspace, work, preparation, value}` (worker) | 09.1a | 18.3 |
| `temper_engine_smith::charter` (the charter, with the resume rule the system world keeps today) | 08.3 | 18.3 |
| `temper_service::system` (one process), `temper_service::startup::{named, startup}` | 09.3, 10.1b | 10.3, 14.1, 16.5, 17.2 |
| `crates/temper/tests/support/` (the process tier's support: `Scratch`, `configuration`, `serve`, `printed_link`), `scripts/proof-one-process.sh`, the nextest profiles `process` and `live` | 10.4 | 12.2, 15.3, 15.4, 16.4, 16.5, 16.6 |
| `temper_client_domain::Domain`, `temper_client_view::View` | 11.1, 11.2 | 11.3, 12.1, 15.1a, 15.1b, 16.x |
| `tests/client`'s `App` and `Front` (jig's `jig-client-world` kit) | 11.3 | 11.4, 15.2 |
| `temper_shell_terminal::client::NativeClient` | 12.1 | 12.2, 15.3, 16.3, 17.2 |
| `temper_engine_protocol_forge::{Component, Request, Event, Limits}` | 13.1 | 13.5, 14.1 (wired), 14.4, 14.5 |
| the fake forge domain's Actions runs and jobs | 14.2 | 14.3 |
| `testing/temper-fake-forge-protocol` | 14.3 | 14.4, 17.2 |
| `testing/temper-fake-git` | 17.1 | 17.2 |
| the engine worlds' store driver (jig's store component over skein-kv's memory mode) | 18.1 | 18.2 |

## 5. Tests and fixtures

- **Where tests go.** A step crate's unit tests in `src/tests.rs`, its
  fixtures in `src/tests/fixtures.rs`, made by the first increment that
  tests the crate and extended by the others, never redefined. A world's
  in `tests/<name>.rs`, its fuzzy ones in `tests/fuzzy_<name>.rs`, its
  fixtures in its `src/fixtures.rs`. The process tier's in
  `crates/temper/tests/process.rs`, the live tier's in `live.rs`, their
  support in `crates/temper/tests/support/mod.rs`.
- **One test per behaviour; a behaviour's cases are rows** of one table
  test that names each failing row (testing-strategy.md, section 8). A
  test costs the suite about 8 ms summed before its work: fixtures are
  constants or built once per test; limits are test-sized.
- **Ceilings, per test** (development.md, 5.6): a unit test 20 ms; a
  focused world test 0.5 s; a one-process story 1 s; a fuzzy test about
  5 s; generated codec tests 200 ms. The process tier's tests aim at 5
  minutes together, under the `process` profile's 10-minute timeout;
  the live tier's at most 15 minutes.
- **The new worlds' share of the suites** (domain/testing.md, 6.1: "together at
  most 4 s of the default suite and 25 s of the fuzzy suite"), as guides
  on the summed time of their tests, focused and fuzzy:
  `tests/client` 1.0 s and 5 s; `tests/engine/protocol-forge` 1.0 s and
  8 s; `tests/worker/protocol-git` 0.5 s and 3 s; `tests/one-process`
  1.0 s and 6 s; the terminal's machine world
  (`crates/temper-shell-terminal/tests/`) 0.3 s and 2 s; `tests/codecs`
  outside this share (codec tests are step tests). A world past its
  share is made cheaper before more stories land in it; past twice it,
  a question.
- **Transcripts are read as files** from the tools' directories by path
  from `env!("CARGO_MANIFEST_DIR")`, raw bytes, never re-recorded by a
  test. Golden text of the terminal's renderer is
  `crates/temper-shell-terminal/src/tests/golden/<page>.txt`, changed
  only by hand (client.md, 6.8).
- **Scratch directories** of tests are made under
  `env!("CARGO_TARGET_TMPDIR")`, never under `/tmp`; the tools take
  their output and scratch directory as an argument.
- **No thread in a test,** even for independent seeds: a test that would
  run several seeds in parallel is several tests.
- **Secrets in tests:** the worlds with a protocol layer check that no
  value their fakes issued (an LLM access token, a git identity's token,
  an API token) appears in a root's events or requests, a fact, a store
  record or a log (credentials.md, section 7): 14.4's protocol world for
  the API token, 17.4's referee for all three, 07.4's referee for a git
  identity's value in an argument.

## 6. The plan's "Don't" list

No increment of this plan:

- adds a type beside an existing one for the same thing: it changes the
  type in place, and its users in the same increment;
- adds a field or predicate that its counterpart across a boundary
  lacks: a record mirrors its domain type field for field;
- adds a test that checks several behaviours under one name, a lint
  exception in a test, or an expected value computed by the code under
  test;
- uses the words of section 1's list in code;
- adds a retry below the domain (protocol.md, section 5);
- puts a secret's value in a domain type, a fact, a store record, a log
  or a `Debug` output but skein-lib's `Secret`'s length;
- links `jig-protocol-charter` or `jig-protocol-tools` into
  `temper-service`;
- changes jig, smith or skein, moves a pin (section 3), or edits
  `docs/design/`;
- writes Python, or commits a script beyond the two `scripts/proof-*.sh`
  (a test may write a throwaway script into its scratch directory);
- starts, stops, configures or reads the user's own Forgejo
  (`~/.local/bin/forgejo`, `~/.local/bin/forgejo-server`, which is
  Forgejo 7.0.12, `~/.local/state/forgejo`), or any Forgejo but the one
  `TEMPER_FORGEJO` names, whose `--version` must be 16.0.5;
- writes to `~/.codex`, reads its refresh token, or signs anything in;
- writes under `/tmp`;
- spawns a thread in a test;
- adds an `unreachable!` arm for a variant the service can reach: an arm
  the service reaches is routed;
- adds more than 40 lines to a file that was past 800 lines at the base
  (`README.md`, "Large files"): new code goes in a new module beside it;
- in `temper-shell-terminal`'s step modules (`route`, `render`, `command`,
  `limits`), uses `Vec`, `String`, `format!`, `std::`, a closure or a
  `loop`/`while`: the check is `bash -O nullglob -c "grep -nE 'Vec<|String|format!|std::|\|[a-z_, ]*\||\bloop\b|\bwhile\b' crates/temper-shell-terminal/src/{route,render,command,limits}.rs crates/temper-shell-terminal/src/{route,render,command}/*.rs"`,
  which prints nothing; and each of those modules starts with
  `#![deny(clippy::indexing_slicing, clippy::arithmetic_side_effects,
  clippy::as_conversions, clippy::unwrap_used, clippy::panic,
  clippy::wildcard_enum_match_arm)]`, so clippy holds the rest of the
  subset there that the crate's ordinary lints leave out.

## 7. Token prefixes

Every component's tokens carry a prefix in their top four bits, as jig's
reference service gives its own (findings/jig-service.md, section 1):
`1` the workers' listener, `2` the workers' connections, `3` accounts
and each worker (`(3 << 60) + ((i + 1) << 48)`), `4` the clients'
component, `8` to `10` the store's kernel operations: jig's, kept. This
plan adds `5` the forge component (14.1) and `6` each worker's git
component (`(6 << 60) + ((i + 1) << 48)`, 09.2). `7` and `11` to `15` stay
free. 08.1 writes these as constants in `temper_service::tokens`, with a
test that no two prefixes meet.

## 8. Size limits

- **A file:** at most 800 lines, generated code under `src/generated/`
  and golden files aside. **A test function:** at most 40 lines; no
  command counts it: the hourly check's sampled test reads it
  (development.md, 14.3).
- **An increment:** the Size its file gives, in hand-written production
  lines (tests, generated code, goldens and transcripts aside). It lands
  within twice its Size without asking; a codec or document increment
  (03.x, 04.x, 06.x, and 02.4's family part), whose code is a schema,
  translations and decoders, within 2.5 times it (decisions.md, ruling
  106). The parts of a split increment count together against the
  original increment's Size. Past that, it asks before landing, with the
  count by file. The count:
  `git diff --numstat origin/main...HEAD -- crates testing tools ':!*/tests.rs' ':!*/tests/*' ':!*/generated/*' ':!*/golden/*' ':!*/transcripts/*'`,
  its first column summed.
- **An increment above about 600 lines** of production code is split by
  its session before it starts: two increments, the second needing the
  first, named `NN.ka` and `NN.kb` in its report.
