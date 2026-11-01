+++
id = "cl-wire"
part = "fs"
kind = "interface"
needs = []
files = [
  "temper/crates/temper-client-protocol-wire/**",
  "temper/tests/codecs/tests/suite/client_wire.rs",
  "temper/tests/codecs/tests/suite/fuzzy/client_wire.rs",
  "temper/crates/temper-service/src/family.rs",
  "temper/crates/temper-service/src/tests/family.rs",
  "temper/crates/temper-service/src/clients/demo_tests.rs",
  "temper/crates/temper-service/src/startup/application/demo.rs",
  "temper/crates/temper-service/src/tests/startup.rs",
  "temper/crates/temper-client-domain/**",
  "temper/crates/temper-client-view/src/tests/**",
  "temper/crates/temper-client-tools/**",
  "temper/crates/temper-client-shell-terminal/src/tests.rs",
  "temper/crates/temper-client-shell-terminal/src/tests/**",
  "temper/crates/temper-client-shell-cli/src/tests/**",
  "temper/tests/client/**",
  "temper/tools/gallery/**",
]
effort = "medium"
minutes = 80
+++

## Goal

The client wire's shape of `notes/contracts.md` T8: the playground's renames, the requests
`make_forge`, `remove_forge` and `end_playground`, and the new outcomes, with generated code
and goldens, and every reader's compiler-forced arm, so that later tasks of every designer
build on the shape. Every byte the wire encoded before encodes the same (renamed variants
keep their tags, new ones are appended), and the family's version stays 11 in this task
(`temper/docs/plan/playground-reset-r6/notes/cl-wire.md`, 1, "The version").

- The schema is exactly `notes/cl-wire.md` section 1; generated code and goldens match it
  (the codecs world's drift test passes); no golden file of a removed name is left.
- `decode_requesting` refuses a `make_forge` of `forgejo` with an empty `api` or `git` or an
  empty `Some` `web`, and a `remove_forge` of forge 0; each new request and outcome round
  trips.
- temper-service answers `make_forge`, `remove_forge` and `end_playground` at once
  `not_offered`; `start_playground` and `Read::playground` behave as `start_demo` and
  `Read::demo` did (notes/cl-wire.md, 4).
- The client reads `PlaygroundState::Resetting` as its `DemoState::Ended`; nothing a person
  sees changes.
- Both committed journeys stay their print (`temper-client-journeys`' test passes unchanged,
  no journey file edited).
- The workspace builds; the tests of every package edited pass.

## Inputs

- `temper/docs/plan/playground-reset-r6/notes/common.md`.
- `temper/docs/plan/playground-reset-r6/notes/contracts.md`, T8 (and T4 to T6 for the names
  the new values stand for).
- `temper/docs/plan/playground-reset-r6/notes/cl-wire.md`, whole: the schema text, the
  generated names, the interim behaviour.
- `temper/docs/design/client/README.md`, 4.6 (the family's records) and 4.7 (outcomes).
- `temper/docs/design/client/journeys.md`, 8 (why the version stays): read only.
- skein's codec generator usage as `temper/tests/codecs/tests/suite/drift.rs` prints it.

## Owns and may change

Owned: `temper/crates/temper-client-protocol-wire/` (schema, `src/lib.rs`, `src/tests.rs`,
`src/tests/`, `src/generated/v1.rs`, `golden/`), and the client wire's two files of the
codecs world, `temper/tests/codecs/tests/suite/client_wire.rs` and
`temper/tests/codecs/tests/suite/fuzzy/client_wire.rs` (fe owns that world's store files;
touch nothing else there, not `drift.rs`).

Readers, each edited only as said:

- `temper/crates/temper-service/src/family.rs`: the wire's names renamed (`w::Request::
  StartDemo` → `StartPlayground`, `w::Read::Demo` → `Playground`, `w::DemoAnswer*` →
  `w::PlaygroundAnswer*`, `w::DemoState` → `w::PlaygroundState`, `wire::demo_answer` →
  `wire::playground_answer`); the engine's own names (`forge::Ask::StartDemo`,
  `root::ApplicationAnswer::StartDemo`, `forge::DemoState`) left exactly as the tree has
  them, on their own lines: fe's `fe-surface` renames those concurrently, and the two
  merges must not touch the same lines. Add the new requests' arm as the **first** arm of
  the `match request` in `to_root` (before `w::Request::SetUp`), not beside the StartDemo
  arm, and add the new variants to the `unreachable!("connect or install")` arm's list.
- `temper/crates/temper-service/src/tests/family.rs`, `src/clients/demo_tests.rs`,
  `src/startup/application/demo.rs`, `src/tests/startup.rs`: the same wire renames
  (`w::CEILINGS.demo_answer_*` → `playground_answer_*`), and the new tests below in
  `tests/family.rs`. No engine name changes.
- `temper/crates/temper-client-domain/`: `forge/demo.rs` (`w::Read::Playground`,
  `w::Request::StartPlayground`, the new outcomes in `answer`'s `return None` arm),
  `forge/family.rs` (`decode_playground_answer`; `Resetting` → `forge::demo::DemoState::
  Ended`; new outcomes in each "another operation's outcome" arm), `limits.rs`
  (`w::PlaygroundAnswer::decoded_bound`), `forge/github.rs` and any other exhaustive
  `w::Outcome`/`w::Request` match (new variants into the existing "not this operation"
  arm), and its tests (renames; `w::DemoState::Ended` → `w::PlaygroundState::Resetting`).
  The client's own vocabulary (`forge::Demo`, `forge::demo::DemoState`, `Act::StartDemo`,
  module names) keeps its names: later cl tasks replace it.
- `temper/crates/temper-client-view/src/tests/`, `temper/crates/temper-client-shell-terminal/
  src/tests.rs` and `src/tests/`, `temper/crates/temper-client-shell-cli/src/tests/`,
  `temper/tools/gallery/`, `temper/tests/client/` (src and tests, `qa_all` included):
  renames, `w::DemoState::Ended` → `w::PlaygroundState::Resetting`, and new variants into
  existing exhaustive arms. In `temper/tests/client/src/translate/forge.rs` the new requests
  join the `panic!("client sends no demo start yet")` arm (say "the client sends none yet").
- `temper/crates/temper-client-tools/` (`src/forges.rs`, `src/forges/connection.rs`,
  `src/forges/repositories.rs`, its test fixtures): the rename, and the new variants into
  each existing "not this request/outcome" arm (`None`, or the arm that refuses an
  unexpected outcome). No new command.
- Any other file of the tree the compiler names for these types (common.md, "Allowed").

Options, each pre-approved:

- If the generator names a type or limit member otherwise than `notes/cl-wire.md` 2 says
  (for example `ForgeChoice::GitHub`), use the generator's name and list it in the report.
- If the schema grammar refuses a name (`playground` as an `Outcome` variant beside
  `Read::playground`), stop and report it as a question; never pick another spelling.
- A test whose subject is a renamed item may be renamed with it (`demo_states_round_trip_*`
  → `playground_states_round_trip_*`); its assertions stay.
- If `carried` is `Some` on one of the three new requests in `to_root`, answer
  `Undecodable` instead, as for an undecodable record (temper's clients send none).

## Steps

1. **Schema.** Edit `schema/client-wire-v1.schema` to `notes/cl-wire.md` 1 exactly; keep
   `family temper-client-wire 11`. Replace the `# start_demo: ...` comment with the two
   comments of 1 and, above `PlaygroundAnswer`, "The family's read name and the
   playground's answer (client/README.md, 4.6; temper/docs/design/draft/
   playground-reset.md, 6.1). No playground offered is the read's empty answer, with no
   record."
2. **Generate.** `SKEIN_CODEGEN_WRITE=1 heavy cargo nextest run -p temper-codecs-world --test
   suite -E 'test(=drift::client_wire_schema_matches_generated_code_and_goldens)'`, then
   the same without the variable: it passes. `git rm` the goldens of the old names:
   `golden/enum_demo_state_{ended,not_started,running}_{full,smallest}.bin`,
   `golden/enum_read_demo_{full,smallest}.bin`,
   `golden/enum_request_start_demo_{full,smallest}.bin`,
   `golden/record_demo_answer_{full,smallest}.bin`; `git status` shows no other golden
   removed, and every renamed golden's bytes equal its old name's (check with `cmp`; say so
   in the report).
3. **`src/lib.rs`.** `playground_answer`/`decode_playground_answer` replace the demo's (same
   bodies, the cue check kept); `limits()` names `playground_answer_*` and sets
   `forgejo_choice_api`, `forgejo_choice_git`, `forgejo_choice_web_some` to
   `v1::CEILINGS`' values; `decode_requesting`'s match gains
   - `v1::Request::MakeForge(make)`: for `v1::ForgeChoice::Forgejo(choice)`, None when
     `choice.api()` or `choice.git()` is empty or `choice.web()` is `Some` empty; `Owned` and
     `Github` pass;
   - `v1::Request::RemoveForge(remove)` with `remove.forge() == 0`: None;
   - `v1::Request::StartPlayground | v1::Request::EndPlayground` beside the others that pass.
   The `CUES`/`CUE_BYTES` docs say "the playground's".
4. **Wire tests** (`src/tests.rs`; a new `src/tests/forge_requests.rs` module if it reads
   better), at least:
   - `make_forge_round_trips_each_choice` (owned, github, forgejo with and without web, at
     the ceilings 259/1024/1024 and one past each refused by `ForgejoChoice::new`);
   - `make_forge_with_an_empty_address_is_refused_on_decode` (empty api, empty git,
     `Some(empty)` web: `decode_requesting` None);
   - `remove_forge_round_trips_and_refuses_forge_zero`;
   - `end_and_start_playground_round_trip`;
   - `the_new_outcomes_round_trip` (`made { forge: 1 }`, `made { forge: u16::MAX }`,
     `full`, `not_offered`, `removing`, `playground`, `reset`);
   - `renamed_variants_keep_their_bytes`: `Request::StartPlayground` encodes as tag 2,
     `PlaygroundState::Resetting` as tag 2, `Read::Playground` as tag 0 (the bytes the
     demo's names had).
   The existing demo tests renamed to the playground's names.
   `heavy cargo nextest run -p temper-client-protocol-wire --lib`.
5. **Codecs world.** Rename in `client_wire.rs` and `fuzzy/client_wire.rs`; add a
   `make_forge` forgejo request and a `made` outcome to the fuzzy seeds where the file
   seeds requests and outcomes. `heavy cargo nextest run -p temper-codecs-world --test
   suite -E 'test(/client_wire/) | test(/drift::client_wire/)'` and `heavy cargo nextest
   run -p temper-codecs-world --profile fuzzy --test suite -E 'test(/fuzzy::client_wire/)'`.
6. **temper-service.** In `family.rs::to_root`, the first arm:

   ```rust
   w::Request::MakeForge(_) | w::Request::RemoveForge(_) | w::Request::EndPlayground => {
       // Answered here until the family's translation routes them (notes/cl-wire.md, 4).
       let answer = w::Answer::new(limits, w::AnswerParts { outcome: w::Outcome::NotOffered })
           .expect("fieldless outcome");
       return Err(Box::new(kit::Event::Application(kit::FromApplication::Answered {
           to: reply_to, family: wire::FORGE, record: wire::answer(&answer),
       })));
   }
   ```

   (the comment in the code's own words, citing the draft's section 3 or 4 rather than the
   plan's note if you prefer; no plan path in code). `Read::Playground` answered from the
   demo's settings and state as before: `None` → `NotStarted`, `Running` → `Running`,
   `Ended` → `Resetting`. Tests in `src/tests/family.rs`:
   `make_remove_and_end_playground_are_answered_not_offered_at_once` (each of the three
   decoded from its wire bytes through `to_root` gives `Err(Answered)` whose record decodes
   to `Outcome::NotOffered`, family `wire::FORGE`), and the existing demo read/state tests
   renamed (`Ended` → `Resetting`).
   `heavy cargo nextest run -p temper-service --lib -E 'test(/tests::family::/) |
   test(/clients::demo_tests::/) | test(/tests::startup::/)'`.
7. **Client crates and worlds.** Follow the compiler (`heavy cargo check --workspace
   --tests`), renaming and adding arms as "Owns and may change" says. Then:
   `heavy cargo nextest run -p temper-client-domain --lib`,
   `heavy cargo nextest run -p temper-client-view --lib`,
   `heavy cargo nextest run -p temper-client-tools --lib`,
   `heavy cargo nextest run -p temper-client-shell-terminal --lib`,
   `heavy cargo nextest run -p temper-client-shell-cli --lib`,
   `heavy cargo nextest run -p temper-client-gallery --lib`,
   `heavy cargo nextest run -p temper-client-world --test suite`,
   `heavy cargo nextest run -p temper-client-journeys --lib` (must pass with no journey file
   touched: it proves the asks' bytes are unchanged).
8. **Lints.** `cargo fmt` of the changed packages; `heavy cargo clippy --workspace --tests`
   for the edited packages (`-p` each), and `heavy cargo clippy -p
   temper-client-protocol-wire -p temper-client-domain --target wasm32-unknown-unknown`
   (the gate's wasm clippy, `temper/docs/design/client/browser.md`).
9. `python3 -I -B scripts/old_forms.py` from `temper/`; commit (small commits: schema and
   generated; lib and tests; readers) and push your branch.

## Report

- Each generated name that differs from `notes/cl-wire.md` 2.
- The `cmp` check of the renamed goldens.
- Every file edited outside the wire crate, by package, and which were compiler-forced.
- Under `## Left`: design text still naming `start_demo`, `Read::demo`, `DemoAnswer` or
  `DemoState` of the wire (`temper/docs/design/client/README.md` 4.6 and others you met).
