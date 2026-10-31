+++
id = "cl-wire"
part = "client"
kind = "interface"
needs = []
files = [
  "temper/crates/temper-client-protocol-wire/**",
  "temper/tests/codecs/tests/suite/client_wire.rs",
  "temper/tests/codecs/tests/suite/fuzzy/client_wire.rs",
  "temper/crates/temper-client-domain/**",
  "temper/crates/temper-client-view/**",
  "temper/crates/temper-client-tools/**",
  "temper/crates/temper-client-shell-cli/**",
  "temper/crates/temper-client-shell-terminal/**",
  "temper/crates/temper-client-shell-web/**",
  "temper/tests/client/**",
  "temper/tools/gallery/**",
  "temper/docs/design/client/journeys/welcome-set-up/**",
  "temper/docs/design/client/journeys/welcome-playground/**",
]
effort = "high"
minutes = 90
+++

## Goal

Put contract 5 on the client's wire, `temper-client-protocol-wire`, in
place at family version 12: `SetUp` naming no forge, `Connect` with an
optional forge, the answers naming the forge's number, `RemoveForge`,
`StartPlayground` (today `start_demo`), `EndPlayground`, the read
`playground` and its answer (running or resetting), and each forge row's
`finished`. Every reader of the wire compiles and behaves as today
through narrow bridges; the engine and client behavior of the new
requests is later tasks'.

Acceptance:

- `temper-client-protocol-wire`'s unit tests (new and renamed below)
  pass: each new request and outcome round-trips; `remove_forge` with
  forge 0 and `connect` with `Some(0)` are refused by
  `decode_requesting`; a running playground answer with an empty cue is
  refused by `decode_playground_answer`; a resetting one round-trips.
- `temper-codecs-world`'s `drift::client_wire_schema_matches_generated_code_and_goldens`
  passes against the regenerated code and goldens, and
  `client_wire::setup_and_every_first_repository_state_round_trip_at_version_twelve`
  (renamed from `..._at_version_eleven`) and the fuzzy client wire tests pass.
- The focused tests of every package listed in Steps 6 pass unchanged in
  meaning, but for the tests of an ended or not-started demo answer
  (Steps 5).

## Inputs

- `temper/docs/plan/playground-reset-r2/notes/common.md`.
- `temper/docs/plan/playground-reset-r2/notes/contracts.md`, 5 (this
  task's contract), 2 (the forge's number: from 1, never 0).
- `temper/docs/design/draft/playground-reset.md`, 4, 6.1-6.3, 6.4.
- `temper/docs/design/client/README.md`, 4.6 (the family's records).
- `skein/docs/design/codec.md`, the schema grammar section the schema's
  header names, if a form below is in doubt (`option u16` is the form of
  today's `option u64`).
- `temper/docs/design/client/journeys.md`, 8 and 8.1 (a recording gone
  stale; `again`).

## Owns and may change

Owns `temper-client-protocol-wire` whole (schema, `src/generated/v1.rs`,
`src/lib.rs`, its tests, `golden/`), and
`temper/tests/codecs/tests/suite/{client_wire.rs,fuzzy/client_wire.rs}`.

May change, **call sites only** (a renamed type or variant, a new field
in a literal, a match arm for a new variant, a removed field dropped),
each to keep today's behavior:

- `temper-service` (fr-'s crate: the lines naming the wire only):
  `src/family.rs`, `src/clients.rs`, `src/clients/*.rs`,
  `src/startup/checks.rs`, `src/startup/application/demo.rs`,
  `src/translate.rs`, `src/assignment.rs`, `src/route.rs`,
  `src/worker_route.rs`, `src/workers.rs`, and its tests that build wire
  records (`src/tests/family.rs`, `system.rs`, `setup_github.rs`,
  `translate.rs`, `fixtures/*.rs`, `clients/demo_tests.rs`, and any other
  `src/tests/*.rs` the compiler names). The bridges, exactly:
  - `w::Request::SetUp` (no forge): the forge is the first
    `crate::settings::Forge` in `forges` whose `mode` is
    `temper_engine_domain_forge::Mode::Owned`; none, forge `0` (the
    domain refuses an unknown forge as today).
  - `w::Request::Connect` with `forge: Some(n)`: `n`; with `None`: the
    first `settings::Forge` of the kind `how` gives (`GithubApp`,
    `GithubManifest`: `Kind` GitHub; `ForgejoAdmin`: Forgejo, mode not
    owned and not fake); none, `0`.
  - `w::Request::StartPlayground`: today's `forge::Ask::StartDemo`.
  - `w::Request::RemoveForge(_)` and `w::Request::EndPlayground`:
    `kit::FromApplication::Undecodable { to: reply_to }`, as an
    undecodable request is answered today. `rm-service` and `pg-reset`
    replace these two arms.
  - The read `w::Read::Playground`: a demo whose state is
    `Some(forge::DemoState::Running)` answers `PlaygroundState::Running`
    with today's members; any other state, or none, answers no record
    (no playground).
  - Answers: `SetUpAnswer::SettingUp` → `Outcome::SettingUp(ForgeNumber
    { forge: 0 })`; Connect's and Install's `Connecting` →
    `Outcome::Connecting(ForgeNumber { forge: 0 })`; their `Link(address)`
    → `Link { forge: 0, address }`. `0` is "not numbered yet": the
    domain's answers carry no number until `fr-domain`; the `fr-` task
    that carries it replaces the zeros. Comment each zero so.
  - Forge rows: `finished` is `true` when the row's connection is
    `forge::Connection::Connected { .. }` and its `repositories` is not
    empty, else `false`.
- The client crates, all mine and run by no other task now:
  `temper-client-domain`, `temper-client-view`, `temper-client-tools`,
  `temper-client-shell-cli`, `-shell-terminal`, `-shell-web`,
  `temper/tests/client` (`temper-client-world`), `temper/tools/gallery`.
  Bridges:
  - The client's `forge::family::set_up` drops its `forge` parameter if
    nothing else reads it there; its callers keep their own slot state
    (which forge they watch) as today.
  - The client's Connect sends `forge: Some(n)` with today's `n`.
  - Decoding answers: `SettingUp(_)`, `Connecting(_)` and `Link(_)` map
    to today's client answers, the number ignored; `Removing`,
    `RemoveRefused`, `Resetting` and `EndRefused` join the arms that
    answer `None` (not this request's answer) in every decoder that
    matches `w::Outcome` exhaustively (client-domain `forge/family.rs`,
    `forge/github.rs`, `forge/admin.rs`, `forge/demo.rs`,
    `temper-client-tools` `forges.rs`, `forges/connection.rs`,
    `forges/repositories.rs`, the shells' matches).
  - Decoding the playground answer into the client's own
    `forge::demo::DemoState` (client-domain's type, unchanged here):
    `running` → `Running`; `resetting` → `Ended` (never sent before
    `pg-reset`; `cl-playground` replaces the client's vocabulary).
    `finished` on a row is read into nothing yet.
  - `temper/tests/client`'s own translation (`src/translate/forge.rs`)
    and demo shim (`src/front.rs`, `src/front/account_acts.rs`) follow
    the service's bridges above.
- The two committed journeys, only if the journeys test reports them
  stale (Steps 7).
- `temper/tests/codecs/tests/suite/fuzzy/store.rs` and any other codecs
  test: not yours; if one reads a client wire name the compiler names,
  change that name only.

Not yours: the engine domains (`temper-engine-domain*`), the store, the
binary, `docs/design/`, `docs/help/`. If a bridge above needs a domain
change, stop and ask.

## Steps

1. **The schema**, `schema/client-wire-v1.schema`, in place (the file
   name and module `v1` stay):
   - `family temper-client-wire 12`.
   - `record SetUp { org: bytes 1024 repository: bytes 1024 project:
     SetUpProject playground: bool }` (no `forge`).
   - `record Connect { forge: option u16 how: How }`: none makes a new
     forge of the kind `how` gives; some goes on with a kept one
     (playground-reset.md, 6.4).
   - `record RemoveForge { forge: u16 }`.
   - `enum Request { adopt: Adopt, set_up: SetUp, start_playground,
     return_: Return, connect: Connect, install: Install, reachable:
     Reachable, accept: Accept, remove_forge: RemoveForge,
     end_playground }` (start_demo renamed in place, the two new ones
     last). Update the comment above `GitHubApp` that names start_demo.
   - `DemoState` goes. `record PlaygroundRunning { project: u32 forge:
     u16 provider: u32 owner: bytes 1024 name: bytes 1024 cues: list 8
     text 256 }`; `enum PlaygroundState { running: PlaygroundRunning,
     resetting }`; `enum Read { playground }`; `versioned record
     PlaygroundAnswer { state: PlaygroundState }` in place of
     `DemoAnswer`; no playground stays the read's empty answer.
   - `ForgeRow` gains `finished: bool` last: connected with its first
     repository adopted (playground-reset.md, 6.3).
   - Outcomes: `record ForgeNumber { forge: u16 }`; `record Link {
     forge: u16 address: bytes 2048 }`; `enum RemoveRefusal {
     not_offered, unknown }`, `record RemoveRefused { why: RemoveRefusal
     }`; `enum EndRefusal { not_offered }`, `record EndRefused { why:
     EndRefusal }`; `enum Outcome` keeps its order, `setting_up:
     ForgeNumber` and `connecting: ForgeNumber` in place, and appends
     `removing, remove_refused: RemoveRefused, resetting, end_refused:
     EndRefused`. Comment the new ones with their draft section (4, 6.2).
2. **Generate:** remove every `golden/enum_*` and `golden/record_*` file
   (the generator's; keep the hand-written ones such as
   `connect_forgejo_admin_*.bin`), then
   `SKEIN_CODEGEN_WRITE=1 heavy cargo nextest run -p temper-codecs-world --test suite -E 'test(=drift::client_wire_schema_matches_generated_code_and_goldens)'`,
   then the same without the variable to see it pass. `git add` the
   regenerated `src/generated/v1.rs` and goldens.
3. **`src/lib.rs`:** `CLIENT_VERSION = 12`; `limits()` takes the
   renamed and new fields (`playground_running_owner`/`_name` from
   `field_bytes`, `_cues` `CUES`, `_cues_item` `CUE_BYTES`; the link's
   address `ADDRESS_BYTES`; the generator names them, follow it);
   `demo_answer`/`decode_demo_answer` become `playground_answer` and
   `decode_playground_answer` (the empty-cue refusal on running only);
   `decode_requesting` refuses `remove_forge` with forge 0 and `connect`
   with `Some(0)` (contracts.md, 2: numbers start at 1), and its match
   names the new variants. Module docs cite
   `temper/docs/design/draft/playground-reset.md, 4 and 6` where the new
   records are.
4. **The crate's tests** (`src/tests.rs`, `src/tests/*.rs`): rename
   `the_demo_read_and_answer_round_trip_with_nonempty_cues` →
   `the_playground_read_and_answer_round_trip_with_nonempty_cues`,
   `demo_cues_obey_their_count_and_text_bounds` →
   `playground_cues_obey_their_count_and_text_bounds`,
   `demo_start_outcomes_round_trip_at_each_boundary` →
   `playground_start_outcomes_round_trip_at_each_boundary`;
   `demo_states_round_trip_at_smallest_and_ceiling` becomes
   `playground_states_round_trip_running_and_resetting`; add
   `a_remove_forge_and_the_playground_end_round_trip`,
   `a_zero_forge_is_refused_in_remove_and_connect`,
   `a_connect_without_a_forge_round_trips`,
   `removal_and_end_outcomes_round_trip` (removing, each refusal,
   resetting), `each_numbered_answer_carries_its_forge` (setting_up,
   connecting, link), `a_forge_row_round_trips_finished_either_way`.
   `limits_take_each_bound_from_its_source_field` covers the new fields.
5. **Readers:** make every package in Steps 6 compile with the bridges
   above. A test that builds a `DemoState::Ended` or `NotStarted` wire
   answer: if it tests the client's retirement or its halves (they go,
   playground-reset.md 6.3), build it as no answer where that keeps the
   test's point, else delete it, and list each deleted test in the
   report; any other demo test builds a running answer. Renamed tests
   keep their names unless they name the wire's `demo` read.
6. **Checks:** `cargo fmt`; `heavy cargo clippy -p temper-client-protocol-wire -p temper-codecs-world -p temper-service -p temper-client-domain -p temper-client-view -p temper-client-tools -p temper-client-shell-cli -p temper-client-shell-terminal -p temper-client-shell-web -p temper-client-world -p temper-client-gallery --all-targets -- -D warnings`;
   the focused tests, one `heavy cargo nextest run` over the same
   packages plus `temper-client-journeys` and `temper-client-browser-world`
   (its focused profile only; it builds against the shells).
7. **Journeys:** if `temper-client-journeys`'
   `the_committed_journeys_are_their_print` reports `welcome-set-up` or
   `welcome-playground` stale (the recordings carry version 11 bytes),
   record each again with `heavy cargo run -p temper-client-journeys --
   again NAME` (journeys.md, 8.1: a fresh fake box and headless
   Chromium; the bridges keep the walk). If `again` refuses (the walk
   changed) or cannot run on the builder, stop: report it as the first
   line, `Question:`, with the refusal, and push nothing.
8. Old forms: `python3 -I -B scripts/old_forms.py` from `temper/`;
   lower with `--write` what your change lowers. Commit (small commits:
   schema and generated; lib and tests; readers; journeys) and push.

## Report

- Each bridge as built, file and line, so `fr-`, `rm-`, `pg-` and the
  client tasks find and replace them; any bridge that could not be
  built as said and what you did instead.
- Each test deleted or rebuilt in Steps 5, by package and name.
- Whether the journeys went stale and were recorded again.
- The generator's limit names for the new fields.
