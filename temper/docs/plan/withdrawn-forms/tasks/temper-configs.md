+++
id = "temper-configs"
part = "temper-configs"
kind = "migration"
needs = []
files = [
  "temper/crates/temper-init/template/agent.json",
  "temper/crates/temper-init/src/tests/documents.rs",
  "temper/docs/examples/owned/agent.json",
  "temper/crates/temper/tests/support/github/agent.json",
  "temper/crates/temper/tests/fixtures/cluster/agent.json",
  "temper/crates/temper/tests/support/configuration.rs",
  "temper/crates/temper/tests/suite/forge_land.rs",
  "temper/crates/temper-fake/src/backends.rs",
  "temper/crates/temper-fake/src/wire.rs",
  "temper/tests/client-browser/src/configuration.rs",
  "temper/tests/real-loop/src/fixtures.rs",
  "temper/tests/real-loop/src/llm.rs",
  "temper/tests/real-loop/tests/suite/account_key.rs",
  "temper/tests/one-process/src/fixtures.rs",
]
effort = "medium"
minutes = 30
+++

# temper writes no withdrawn form: no endpoint `identity`, no `account_id`

## Goal

temper's agent configurations (the shipped template, the owned example, the fixtures and the
test code that writes one) carry no endpoint `"identity"`, and its fake and test code makes
skein-llm credentials with `skein_llm::Credential::key(..)`, never naming the field
`account_id`. Nothing changes in behaviour: smith's agent reads a missing `identity` as plain
(`smith/crates/smith-agent-shell/src/config/document.rs:176`, `#[serde(default)]`;
`config/endpoints.rs:207`, `None | Some("plain")`), and `Credential::key` builds the same value
(`skein/crates/skein-llm/src/types.rs:429`). This is the first of the plan's five removals: once
it lands, smith's agent can refuse `identity` and skein can drop `account_id` without touching
temper's configurations.

Acceptance:

- `rg -n '"identity"' temper --glob '!**/docs/plan/**'` finds no agent-configuration endpoint
  key: what is left is jig's and temper's own identity vocabulary (for example
  `temper-client-tools/src/jig/render/document/read_turns.rs:82`, `bounds.rs:19`,
  `temper-protocol-worker-wire/src/tests.rs:135`, `tests/real-loop/src/case/oracle.rs:9333`),
  which this task does not touch.
- `rg -n 'account_id' temper --glob '!**/docs/plan/**' --glob '*.rs'` finds no skein-llm
  `Credential` literal (the forge's `account_id` in `tests/engine/protocol-forge` is a GitHub
  account and stays).
- The tests in Steps pass, each count not zero; clippy is clean.

## Inputs

- `temper/docs/plan/withdrawn-forms/notes/common.md`.
- `temper/AGENTS.md`.
- `smith/docs/design/protocol/agent.md`, 4: an endpoint's identity profile is withdrawn,
  "written by none of smith's"; it goes when its last host has moved. temper is such a host.
- `skein/docs/design/llm.md`, 2.6: `Credential::bearer` and `Credential::key` make one; the
  account id is the withdrawn Codex route's alone.
- `temper/docs/help/box.md`, step 1 (read only): its `jq` builds Claude's endpoint as
  `$openai + {...}`, so it inherits whatever init's template writes; it names no `identity`
  and stays unchanged.

## Owns and may change

Agent configurations, each loses its endpoint's `"identity": "plain"` and nothing else (keep the
surrounding fields, their order and formatting):

- `temper/crates/temper-init/template/agent.json` (two endpoints, ~28 and ~56);
- `temper/docs/examples/owned/agent.json` (~28); `temper-init`'s tests include it
  (`src/tests/fixtures.rs:6`, `AGENT`) and compare it with the template;
- `temper/crates/temper-init/src/tests/documents.rs` (`demo_agent`, ~332);
- `temper/crates/temper/tests/support/github/agent.json` (~27);
- `temper/crates/temper/tests/fixtures/cluster/agent.json` (~27);
- `temper/crates/temper/tests/support/configuration.rs` (~13);
- `temper/crates/temper/tests/suite/forge_land.rs`
  (`a_claude_configuration_replaces_openai_and_keeps_the_demo`, ~380: the expected Claude
  endpoint, which `box.md`'s `$openai + {...}` copies from the template);
- `temper/crates/temper-fake/src/backends.rs` (`program_endpoint`, ~314: the agent's tail starts
  at `"connect"`);
- `temper/tests/client-browser/src/configuration.rs` (~32);
- `temper/tests/real-loop/src/fixtures.rs` (~27).

skein-llm credentials, each `skein_llm::Credential { value: V, scheme: Scheme::Key, account_id:
.. }` becomes `skein_llm::Credential::key(V)` (drop an import of `Scheme` or `Credential` the
change leaves unused):

- `temper/crates/temper-fake/src/wire.rs` (~91);
- `temper/tests/real-loop/src/llm.rs` (~72, ~129);
- `temper/tests/real-loop/tests/suite/account_key.rs` (~29);
- `temper/tests/one-process/src/fixtures.rs` (~375).

Pre-approved options:

- If another temper file writes an endpoint `"identity"` in an agent configuration (in JSON or
  in a Rust string or `json!` literal), remove it the same way and list it in the report. If one
  writes `"identity"` with a value other than `"plain"`, stop and ask: that would be a behaviour
  change.
- If another temper file builds a skein-llm `Credential` by its fields, switch it to
  `Credential::key` or `Credential::bearer` by its scheme and list it.
- If a golden or rendered expectation (for example a help page or a `temper-init` facts golden)
  holds the template's bytes, regenerate it with `SKEIN_CODEGEN_WRITE=1` on its test and list
  it; never edit `temper/docs/help/**` by hand.
- Leave these to later tasks of this plan, and do not touch them: `temper-worker-domain`'s
  `Cap::AccountId` arm and its test (smith's channel at v6 removes the variant),
  `temper/docs/design/domain/worker.md`'s and `protocol/channel.md`'s and
  `protocol/channel.status.md`'s lines on the account id and the withdrawn value, and every
  `"dialect"` value (jig's task).

## Steps

1. Edit the agent configurations, then the credentials, as above.
2. Narrow tests, one at a time:
   - `heavy cargo nextest run -p temper-init --lib`
   - `heavy cargo nextest run -p temper-fake --lib`
   - `heavy cargo nextest run -p temper --test suite -E 'test(/forge_land/)'`
   - `heavy cargo nextest run -p temper --lib`
   - `heavy cargo nextest run -p temper-real-loop-world --test suite -E 'test(/account_key/)'`
   - `heavy cargo nextest run -p temper-one-process-world --test suite` (if the whole suite is
     long, filter to the stories that build `fixtures.rs`'s peer and say which)
   - `heavy cargo check -p temper --tests` (the `process` binary reads `support/` and the cluster
     fixture) and `heavy cargo check -p temper-client-browser-world --tests`
3. `cargo fmt`; `python3 -I -B scripts/old_forms.py` from `temper/`;
   `heavy cargo clippy -p temper-init -p temper-fake -p temper -p temper-real-loop-world -p temper-one-process-world -p temper-client-browser-world --all-targets -- -D warnings`.
4. The acceptance's two `rg`.
5. One commit ("temper writes no endpoint identity and no credential account id: the withdrawn
   forms' first removal"), pushed.

## Report

As the implementer's text says, and also: each file changed, by the edit (identity or
credential); any file found beyond the list; what the two `rg` leave, by file.
