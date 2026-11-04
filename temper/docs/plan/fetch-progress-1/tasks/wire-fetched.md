+++
id = "wire-fetched"
part = "row"
kind = "interface"
needs = []
files = ["temper/crates/temper-client-protocol-wire/schema/client-wire-v1.schema", "temper/crates/temper-client-protocol-wire/src/generated/v1.rs", "temper/crates/temper-client-protocol-wire/golden/**", "temper/crates/temper-client-protocol-wire/src/tests.rs"]
effort = "medium"
minutes = 30
+++

# The wire's `SettingUp` carries how far the fetch has gone

## Goal

The client protocol's `SettingUp` gains an optional `fetched`, in place, with every literal given `fetched: None`. Nothing fills it yet: `link-fetched` does on the server's side and `client-fetched` reads it.

Acceptance:

- the schema as `temper/docs/plan/fetch-progress-1/notes/contracts.md`, section 5, has it; `CLIENT_VERSION` unchanged;
- the generated code and goldens are the generator's, and the drift test passes;
- a `SettingUp` with `fetched`, with a total and without, round-trips;
- every literal compiles, with `fetched: None`.

## Inputs

- `temper/docs/design/client/README.md`, 4.6, `ForgeRow`.
- `temper/docs/plan/fetch-progress-1/notes/contracts.md`, section 5, and `notes/common.md`.

## Owns and may change

Owns the files in the header.

May also change, each `SettingUpParts` literal alone, adding `fetched: None`:

- `temper/crates/temper-service/src/family.rs` and `src/tests/family.rs`
- `temper/crates/temper-client-domain/src/tests/forges.rs`
- `temper/crates/temper-client-view/src/tests/connect_forge.rs`, `welcome.rs`, `model_forge.rs`, `forges.rs`
- `temper/crates/temper-client-tools/src/tests/forges.rs`
- `temper/crates/temper-client-shell-cli/src/tests/fixtures/setup.rs`
- `temper/tools/gallery/src/worlds.rs`

Pre-approved:

- `generated/v1.rs` and the goldens are written by the generator alone, never by hand. On a merge conflict later, regenerate.
- `to-begin-store` and `repository-copies-1` change this schema on their trunks; `SettingUp`'s lines are not theirs. Expect nothing but a regeneration.
- If a golden outside `golden/**` nests `SettingUp`, regenerate it with the same command and say so.

## Steps

1. The schema's three lines, placed by `SetUpStep`.
2. Regenerate: `heavy env SKEIN_CODEGEN_WRITE=1 cargo nextest run -p temper-codecs-world --test suite -E 'test(drift::)'`.
3. The literals above; `rg 'SettingUpParts \{' temper` finds them all.
4. In `src/tests.rs`, beside the present `SettingUp`: `a_setting_up_carries_its_fetch`, with a total and without.
5. `heavy cargo nextest run -p temper-codecs-world --test suite -E 'test(drift::)'` with no variable: it compares.
6. `cargo fmt`; the old-forms check; `heavy cargo clippy -p temper-client-protocol-wire --all-targets -- -D warnings`; `heavy cargo nextest run -p temper-client-protocol-wire`; `heavy cargo check --tests -p temper-service -p temper-client-domain -p temper-client-view -p temper-client-tools -p temper-client-shell-cli -p temper-client-gallery`. Commit and push.

## Report

As the implementer's text has it, with the goldens that changed.
