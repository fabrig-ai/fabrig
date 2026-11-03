+++
id = "jig-dialects"
part = "jig-dialects"
kind = "migration"
needs = []
files = [
  "jig/crates/jig-protocol-config/src/llm.rs",
  "jig/crates/jig-protocol-config/src/tests/fixtures.rs",
  "jig/crates/jig-protocol-config/src/tests/llm.rs",
  "jig/crates/jig-protocol-config/src/tests/charters.rs",
  "jig/crates/jig-protocol-charter/src/tests.rs",
  "jig/crates/jig-domain-core/src/tests.rs",
  "jig/crates/jig-domain-hub-link/src/tests.rs",
  "jig/crates/jig-protocol-transcript/src/tests.rs",
  "jig/testing/jig-test-service/src/tests/fixtures.rs",
  "jig/testing/jig-test-service/src/tests/worker.rs",
  "jig/testing/jig-test-worker/src/tests/fixtures.rs",
  "jig/testing/jig-test-domain/src/tests/fixtures.rs",
  "jig/testing/jig-test-profile/src/lib.rs",
  "jig/testing/jig-client-test-records/src/settings.rs",
  "jig/testing/jig-client-scripted-engine/src/codec/turns.rs",
  "jig/testing/jig-client-test-domain/src/lib.rs",
  "jig/testing/jig-test/src/tests.rs",
  "jig/testing/jig-test/tests/support/mod.rs",
  "jig/testing/jig-test/tests/support/live_sign_in.rs",
  "jig/tests/channel/src/fixtures.rs",
  "jig/tests/channel/src/program_runs.rs",
  "jig/tests/channel/tests/suite/source_carry_over.rs",
  "jig/tests/core/tests/settings/engine.json",
  "jig/tests/core/tests/suite/reads.rs",
  "jig/tests/core/src/protocol.rs",
  "jig/tests/core/src/agents_tools.rs",
  "jig/tests/client-browser/src/configuration.json",
  "jig/tests/client-protocol/src/native/config.rs",
  "jig/tests/ops/src/conformance/config.rs",
  "jig/tests/ops/src/conformance/scripts.rs",
  "jig/tests/engine-clients/src/fixtures/receiving.rs",
  "jig/examples/ops/domain/src/tests.rs",
  "jig/docs/design/protocol/llm.status.md",
  "temper/crates/temper-service/src/tests/fixtures.rs",
  "temper/crates/temper-service/src/tests/accounts.rs",
]
effort = "high"
minutes = 90
+++

# jig refuses any endpoint dialect but 2 and 3, and its fixtures write 3

## Goal

jig's configuration takes an endpoint's `dialect` only as a dialect smith serves: 2 (Anthropic's
Messages) or 3 (OpenAI's public Responses). Any other number is refused at the endpoint's
`dialect` (`jig/docs/design/protocol/llm.md`, 4, "Its dialect is one smith serves"). Every jig
fixture, literal and world that writes 0, 1 or 4 moves to 3, so that no jig code writes the
withdrawn Codex dialect, 1, and smith can refuse it. This task also does two edits now that save
smith's channel task from touching jig. The five `smith_channel::v5::X` paths become the root
re-exports, and jig-test's agent configurations drop the endpoint `account` and `identity`.
No stored or wire format changes: no schema, generated code or golden moves.

Acceptance:

- `endpoint` in `jig/crates/jig-protocol-config/src/llm.rs` refuses a dialect of 0, 1 and 4
  with one problem, `What::Form` at `...endpoints[<i>].dialect`, and accepts 2 and 3. The
  table test shows each.
- `rg -n '"dialect"\s*:\s*[0145]\b|dialect:\s*[0145]\b|DIALECT:\s*u32\s*=\s*[0145]\b' jig temper/crates/temper-service --glob '!**/docs/**' --glob '!**/generated/**'`
  finds nothing but the refusal test's own refused values and `llm.rs`'s placeholder in
  `price()` (~351, overwritten at ~282). If it finds anything else, the report says why.
- `rg -n 'smith_channel::v5' jig` finds nothing.
- jig-test's agent configurations write no endpoint `"account"` or `"identity"`.
- jig's `protocol/llm.status.md` entry for 4 is gone; the tests in Steps pass, each count not
  zero; clippy is clean.

## Inputs

- `temper/docs/plan/withdrawn-forms/notes/common.md`.
- `jig/AGENTS.md`; `jig/clippy.toml` (step code: no traits, generics or stored closures), and
  the workspace's `wildcard_enum_match_arm` (a `match` on a `u32` with `_` is fine).
- `jig/docs/design/protocol/llm.md`, 4: the endpoint and its dialect (written by this plan's
  cut). Its status: `jig/docs/design/protocol/llm.status.md`, the bullet "4: an endpoint's
  dialect is read as any count ...".
- `smith/docs/design/protocol/agent.md`, 4: the numbers (`openai` 3, `anthropic` 2, the
  withdrawn `codex` 1), fixed for good.

## The lead's calls (no question needed)

- **Refuse at the configuration's read only.** A charter carries no dialect (`smith_charter::v4`
  sends the endpoint's name alone), and every `EndpointName` table is built from a configuration
  already read (`jig-test-service/src/workers.rs:61-69`). No check at charter encode or decode,
  in the store or on the client wire.
- **`What::Form`, no new variant.** `What` is matched exhaustively in temper
  (`temper-service/src/startup/problems.rs`, `temper-forge-setup/src/problem.rs`,
  `temper-fake/src/read.rs`); `Form` renders as "invalid form" there already. Push the problem
  and keep reading the endpoint, as `program_bindings` does (`llm.rs`, ~123-125), so that its
  models do not cascade into `Refers` problems.
- **Move every 0, 1 and 4 to 3,** not only those the refusal turns red: the fake providers are
  OpenAI-shaped, and nothing in jig's worlds compares a jig endpoint's dialect with a skein
  endpoint. Keep each coherent group together, since `jig-protocol-charter`'s `encode` resolves a
  name only where number and dialect both match (`encode.rs`, ~22-28):
  - the channel world: `src/fixtures.rs` (ENGINE ~50, `endpoints()` ~160),
    `tests/suite/source_carry_over.rs` (~13, ~22, ~82) and `src/program_runs.rs`. Line ~48 is a
    `String::replace` keyed on `"dialect": 1,`: it does nothing silently on a miss, so re-key it
    to `"dialect": 3,` and check that the program endpoint is still built;
  - jig-test-service: `src/tests/fixtures.rs` (ENGINE ~15, ~416, ~551, ~572) with
    `src/tests/worker.rs` (~101);
  - jig-test-profile's `scripted_model` (`src/lib.rs`, ~261) with jig-ops-world's
    `src/conformance/config.rs` (~109) and `jig/examples/ops/domain/src/tests.rs` (~65);
  - jig-protocol-transcript's `DIALECT` (`src/tests.rs`, ~3) with its rendering assertion (~150,
    `b"1"` becomes `b"3"`);
  - jig-protocol-charter's tests: the 4s and 1s become 3, and the wrong-dialect case (~282,
    `(7,3)`) becomes `(7,2)`.

## Owns and may change

jig (owned):

- `jig-protocol-config`: `src/llm.rs` (`endpoint`, ~129-163: read the count, then `2 | 3`
  accept, any other value refused as above; name the two accepted numbers as constants with a
  comment citing smith's `protocol/agent.md`, section 4); `src/tests/fixtures.rs` (~24),
  `src/tests/llm.rs` (~26, ~29, ~214 `(1, 0)` becomes `(1, 3)`, ~278, ~283; a case per refused
  value, 0, 1 and 4, in `llm_configuration_problems_name_the_invalid_fields`, and an
  acceptance of 2 beside 3), `src/tests/charters.rs` (~37). `fixtures::with` panics on a
  missing pattern, so a pattern and its document move together.
- The literals and fixtures in the files list (the values of section 2 of the survey, each 0, 1 or
  4 becoming 3): `jig-domain-core/src/tests.rs` (~370), `jig-domain-hub-link/src/tests.rs`
  (~66), `jig-test-worker/src/tests/fixtures.rs` (~132, ~189), `jig-test-domain/src/tests/fixtures.rs`
  (~189), `jig-client-test-records/src/settings.rs` (~45), `jig-client-scripted-engine/src/codec/turns.rs`
  (~98), `jig-client-test-domain/src/lib.rs` (`DIALECT`, ~29), `jig/tests/core/tests/settings/engine.json`
  (~22), `jig/tests/client-browser/src/configuration.json` (~59), `jig/tests/client-protocol/src/native/config.rs`
  (~10), `jig/tests/engine-clients/src/fixtures/receiving.rs` (~10).
- The root paths: `smith_channel::v5::{CallName, CEILINGS, CallNameParts}` become
  `smith_channel::{...}` (re-exported at `smith/crates/smith-channel/src/lib.rs:38`) in
  `jig/tests/core/tests/suite/reads.rs` (~826-828), `jig/tests/core/src/protocol.rs` (~156-158),
  `jig/tests/core/src/agents_tools.rs` (~89-91), `jig/tests/ops/src/conformance/scripts.rs`
  (~439-441), `jig/testing/jig-test-service/src/tests/fixtures.rs` (~421-423).
- jig-test's agent configurations: drop `"account":1` and `"identity":"plain"` from
  `jig/testing/jig-test/tests/support/mod.rs` (~35-36) and `src/tests.rs` (~142-143), and the line
  `endpoint["identity"]="plain"` from `tests/support/live_sign_in.rs` (~78). smith's agent reads
  both as absent today (`smith-agent-shell/src/config/document.rs:155-157,176`).
- `jig/docs/design/protocol/llm.status.md`: delete the bullet "4: an endpoint's dialect is read as
  any count ..."; the date line today's; the first bullet stays, and with one bullet left the
  list goes back to one line, `- **Not yet:** 1 to 4, 5.3, 6 to 8: ...`.

temper (readers, values only, since jig's refusal turns them red):

- `temper/crates/temper-service/src/tests/fixtures.rs`: the ENGINE endpoint's `"dialect": 1`
  (~15) becomes 3, and `DIALECT` (~6) becomes 3, with its comment saying what is true now
  ("smith's protocol/agent.md, section 4: OpenAI's public Responses dialect"); its readers
  (`fixtures/agent.rs`, `store_route.rs`, `worker.rs`, `fixtures/worker_peers.rs`) name the
  constant and need no edit.
- `temper/crates/temper-service/src/tests/accounts.rs` (~100): the pattern `"dialect": 1,`
  becomes `"dialect": 3,` in both arguments.

Pre-approved options:

- If another jig file, or a temper file reading a jig configuration, writes a dialect of 0, 1 or
  4 and its test turns red, move it to 3 (or to 2 if it pairs with an Anthropic endpoint) and
  list it. A temper value that is a smith transcript's or a domain's dialect, not a jig
  configuration's, stays (for example `temper-client-tools`' settings goldens, `temper/tools/twin`,
  `temper-engine-smith`'s tests): smith's channel task owns those.
- If `jig/tests/client-browser/src/configuration.json` is read only by the `browser_*` binaries,
  move it anyway and say you could not run its tier (`browser-jig`, never in the gate).
- Leave these to smith's channel task: `jig/tests/channel/src/fixtures.rs` (~260) and
  `src/program_runs.rs` (~99) `identity: smith_protocol_llm::IdentityProfile::Plain` (a smith
  struct's field), `program_runs.rs` (~92) `destination.credential.account_id` (skein's), and
  `jig/tests/channel/src/worker_orders/probe.rs`'s "version-five kind table" comment.
- Do not edit `smith/docs/design/**` or `skein/docs/design/**`: their sentences about jig's
  dialect go with the whole withdrawn-forms entries in later tasks of this plan.

## Steps

1. The refusal in `llm.rs`, with its table cases: `heavy cargo nextest run -p jig-protocol-config --lib`.
2. The fixtures, group by group, the root paths and jig-test's configurations.
3. Narrow tests, one at a time, each count not zero:
   - `heavy cargo nextest run -p jig-protocol-charter --lib`
   - `heavy cargo nextest run -p jig-protocol-transcript --lib`
   - `heavy cargo nextest run -p jig-domain-core --lib`
   - `heavy cargo nextest run -p jig-domain-hub-link --lib`
   - `heavy cargo nextest run -p jig-test-service --lib`, `-p jig-test-worker --lib`, `-p jig-test-domain --lib`, `-p jig-ops-domain --lib`
   - `heavy cargo nextest run -p jig-channel-world --test suite`
   - `heavy cargo nextest run -p jig-core-world --test suite`
   - `heavy cargo nextest run -p jig-client-protocol-world --test suite`
   - `heavy cargo nextest run -p jig-ops-world --test suite`
   - `heavy cargo nextest run -p jig-engine-clients-world --test suite`
   - `heavy cargo nextest run -p jig-client-world --test suite`
   - `heavy cargo nextest run -p jig-one-process-world --test suite`
   - `heavy cargo nextest run -p jig-test --test process` and `heavy cargo nextest run -p jig-test --test live -E 'test(=the_live_configurations_are_ones_jig_test_reads) | test(=live_sign_in::tests::a_refused_plan_is_reported_with_the_run_failure)'`
   - temper: `heavy cargo nextest run -p temper-service --lib` and `heavy cargo nextest run -p temper-one-process-world --test suite` (it reads `jig_channel_world::fixtures::engine_document()`)
   - `heavy cargo check -p jig-client-browser-world --tests`
4. A change of meaning (configurations with 0, 1 or 4 are now refused): also
   `heavy cargo check -p temper-service -p temper-fake -p temper-forge-setup --tests`.
5. `cargo fmt`; `python3 -I -B scripts/old_forms.py` in `jig/` and `temper/`;
   `heavy cargo clippy -p jig-protocol-config -p jig-protocol-charter -p jig-protocol-transcript -p jig-domain-core -p jig-domain-hub-link -p jig-test-service -p jig-test-worker -p jig-test-domain -p jig-test-profile -p jig-client-test-records -p jig-client-scripted-engine -p jig-client-test-domain -p jig-test -p jig-channel-world -p jig-core-world -p jig-client-protocol-world -p jig-ops-world -p jig-engine-clients-world -p jig-ops-domain -p temper-service --all-targets -- -D warnings`.
6. The status edit; the acceptance's three `rg`.
7. Commits: the refusal and its tests; the fixtures, paths and configurations (both trees); the
   status. Push your branch.

## Report

As the implementer's text says, and also: each coherent group moved and how you checked that
its names still resolve (the program endpoint in particular); any file beyond the list; what the
acceptance's `rg` leaves, by file.
