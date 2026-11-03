+++
id = "tools-surface"
part = "charter-tools"
kind = "interface"
needs = []
files = ["jig/crates/jig-domain-core/src/runs/state.rs", "jig/crates/jig-domain-core/src/tests/run_policy_tools.rs"]
effort = "medium"
minutes = 60
+++

# tools-surface: a charter's host families, named

## Goal

jig's `RunPolicy` carries the charter's `tools`, its host-family mask (contract C1), so that
the stores, the wires, temper's settings file and temper's offered tools can each build on it
in their own task. Every literal and decoder of a `RunPolicy` fills `RunPolicy::EVERY_FAMILY`;
no stored, wired or golden byte moves, and no run is offered anything it is not offered today.

Acceptance:

- `RunPolicy.tools: jig_domain_tasks::Tools` exists, last member, with C1's doc comment;
  `RunPolicy::EVERY_FAMILY` exists with C1's doc comment.
- Every struct expression of `RunPolicy` in every tree names `tools`, as the table below says;
  every exhaustive destructuring of one names `tools: _` (an encoder) or binds it where the
  code passes the whole policy on.
- One unit test in `jig-domain-core`, `run_policy_tools::every_family_is_the_identity`: for
  a few authorities' tools (0, 1, 512 | 1024, `u64::MAX`), `RunPolicy::EVERY_FAMILY.0 & t == t`.
- `heavy cargo check --workspace --all-targets` is clean; the focused tests of
  `jig-domain-core`, `jig-protocol-store`, `jig-protocol-config`, `jig-protocol-charter`,
  `jig-protocol-engine-clients`, `jig-client-protocol-link`, `temper-settings-form` and
  `temper-engine-smith` pass unchanged (but for the pre-approved price fixes below).

## Inputs

- `jig/docs/design/domain/engine.md`, 7.1 (a run's charter: "the tools of each host family
  its charter names and its authority holds").
- `temper/docs/plan/configuring-p3/notes/contracts.md`, C1 (binding names).
- `temper/docs/plan/configuring-p3/notes/common.md`.

## Owns and may change

- Owned: `jig/crates/jig-domain-core/src/runs/state.rs` (the member and the constant); the new
  test file `jig/crates/jig-domain-core/src/tests/run_policy_tools.rs` and its `mod` line in
  the crate's test module (`jig/crates/jig-domain-core/src/tests.rs`).
- Neighbour files, **the new member alone** (the stated cross-tree allowance for a required
  field, each line edited only to name `tools`), in every tree the compiler names. The sites
  on the trunk at the cut:

| Where | The edit |
|---|---|
| every `RunPolicy { .. }` struct expression in tests, fixtures and testing crates (`jig-domain-core/src/tests.rs`, `jig-domain-hub-link/src/tests.rs`, `jig-protocol-charter/src/tests.rs`, `jig-protocol-engine-clients/src/tests/settings*.rs`, `jig-protocol-store/src/tests/settings.rs`, `jig/testing/*`, `jig/tests/engine-clients/src/fixtures/receiving.rs`, `jig/tests/ops/src/conformance/scripts.rs`, `temper-client-tools/src/tests/jig/settings.rs`, `temper-engine-domain/src/tests*.rs`, `temper-engine-smith/src/tests.rs`, `temper/tests/engine/domain/src/walking.rs`) | `tools: RunPolicy::EVERY_FAMILY` (by whatever path the file names `RunPolicy`) |
| every decoder that builds a `RunPolicy` from a record, a wire value or a file (`jig-protocol-store/src/codec/settings.rs`, `jig-protocol-config/src/charters.rs`, `jig-protocol-engine-clients/src/translate/settings*.rs`, `jig-client-protocol-link/src/translate/settings.rs`, `jig-client-domain/src/contract/settings.rs`, `temper-client-tools/src/jig/settings_adapter.rs`, `temper-settings-form/src/{settings_file_values,view,wire}.rs`) | `tools: RunPolicy::EVERY_FAMILY`, with the line `// Its format carries no mask yet (configuring-p3, contracts C1).` once per file |
| `temper-service/src/stock.rs`, `charter()` | `tools: jig_domain_core::RunPolicy::EVERY_FAMILY` (`stock-configuring` gives each stock charter its own) |
| every exhaustive destructuring (`let RunPolicy { .. } = ..`, a match) in an encoder | `tools: _` |

  The generated codecs (`jig-client-protocol-wire/src/generated/v19.rs`,
  `jig-protocol-store/src/generated/settings_v1.rs`) hold their own generated types, not
  `RunPolicy`: never edit them; their translators are the decoder rows above.
  `smith/tests/tools` has a `call_timeout:` of smith's own type: not a `RunPolicy`, leave it.
- Options, pre-approved:
  - If a test's fixed price or size (a `size_of`, a worst-case figure, a memory budget in a
    test) fails only because `RunPolicy` grew by its 8 bytes, correct that expectation and its
    comment alone, the comment naming `RunPolicy.tools`; list each. A production bound,
    limit, profile figure or budget that fails is not covered: stop and ask.
  - If a `RunPolicy` is built by a `..base` update or by a helper, give the helper the field
    once and leave its callers.
  - A `Hash`, `Ord` or `Debug` output in a golden text that moves because of the new member:
    update that golden, and list it.

## Steps

1. `state.rs`: the member and the constant, as C1 gives them, the constant in an
   `impl RunPolicy` block beside the struct.
2. Build with `heavy cargo check --workspace --all-targets`; give each site the compiler
   names its row's edit, nothing else of the file changed.
3. The unit test, in `jig-domain-core/src/tests/run_policy_tools.rs`.
4. `cargo fmt`; `python3 -I -B jig/scripts/old_forms.py` and `temper/scripts/old_forms.py`
   (no count rises); `heavy cargo clippy -p jig-domain-core -p jig-protocol-store
   -p jig-protocol-config -p jig-protocol-charter -p jig-protocol-engine-clients
   -p jig-client-protocol-link -p jig-client-domain -p temper-settings-form
   -p temper-client-tools -p temper-engine-smith -p temper-engine-domain -p temper-service
   --all-targets -- -D warnings`, adding any other package whose file you edited; the focused
   tests of the packages in the acceptance (`heavy cargo nextest run -p ...`, one command).
5. Commit ("jig: a charter's host families, carried by RunPolicy (engine.md, 7.1)") and push.

## Report

As the implementer's text says, and: each file edited outside `state.rs`, with its row; every
price or golden corrected, with its old and new figure; every encoder or decoder that met
`RunPolicy` other than the table's (a later task carries the mask through it).
