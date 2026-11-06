+++
id = "fleet-bounds"
part = "bounds"
kind = "interface"
needs = []
files = ["jig/crates/jig-domain-link/src/lib.rs", "jig/crates/jig-domain-fleet/src/**", "jig/crates/jig-protocol-worker-wire/**", "jig/crates/jig-protocol-worker-link/src/translate.rs", "jig/crates/jig-protocol-engine-workers/src/translate.rs", "jig/crates/jig-domain-core/src/settings/fleet_needs.rs"]
effort = "high"
minutes = 120
+++

# A worker's hello declares its endpoints, not its models

## Goal

The fleet stops needing a list of model names from a worker, so that a model can be added while a worker is connected: a hello names each endpoint the agent serves and the largest window and output a charter may name on it, and a model's need is judged against its endpoint's bound. smith's configuration keeps its list for now; `temper`'s preflight folds it into the new bound, and `smith-limits` replaces that.

Acceptance:

- `rg -n 'ModelBound' jig temper smith -g '*.rs' -g '*.schema'` finds nothing but `generated` output of a removed type, which regenerates away.
- A need on a served endpoint within its bound is admitted, whatever its name; a need past it reports `AgentShortfall::Window` or `Output`; a need on no served endpoint is `Model`, and only makes the charter ineligible on that host, as an undeclared model did.
- The fleet keeps a host's whole evidence with no catalogue given it: `Domain.catalogue` and `Domain::new`'s catalogue parameter go, and a hello's endpoints are kept within `CATALOGUE_MODELS`.
- The workers' wire changes in place, its generated code and goldens with it.
- Focused tests of every package touched pass, and `heavy cargo check --workspace --tests` compiles.

## Inputs

- `temper/docs/plan/model-catalogue-2/notes/contracts.md`, 5.
- `jig/docs/design/protocol/channel.md`, 5.1, hello's row and the refusals under it.
- `jig/docs/design/domain/hosts.md`, the agent's bounds and "judges its connected hosts afresh".
- `temper/docs/plan/model-catalogue-2/notes/common.md`.

## Owns and may change

Owns the header's files.

Readers and neighbours, for call sites and literals only, keeping what each test checks:

- `jig/crates/jig-domain-core/src/lib.rs` (`:602`, the fleet's catalogue argument goes), `settings/charters.rs:35` (the two `const _` assertions go, or the one still true stays), `src/tests/**`.
- The shared fixtures, which absorb most of it: `jig/testing/jig-test-profile/src/fixtures.rs`, `jig/testing/jig-test/src/lib.rs`, `jig/testing/jig-channel-kit/src/{fixtures,processes}.rs`, `jig/testing/jig-test-domain/src/**`, `jig/testing/jig-test-service/src/tests/**`, `jig/tests/fleet/src/fixtures.rs`, `jig/tests/fleet/tests/**`, `jig/tests/engine-workers/src/fixtures.rs`, `jig/tests/worker-link/src/fixtures.rs`, `jig/tests/one-process/src/fixtures.rs`.
- temper, by the required-field exception, each edited in the type path and value alone: `temper/crates/temper/src/agent.rs`, `temper/crates/temper-service/src/startup/checks.rs` (`model`, `:846-862`), `temper/crates/temper-service/src/tests/{fixtures,fixtures/agent,fixtures/startup,first_model}.rs`, `temper/crates/temper-playground/src/documents.rs`, `temper/tests/engine/domain/src/protocol.rs`, `temper/tests/engine/domain/tests/suite/**`.

Options, each pre-approved:

- Until `smith-limits`, `temper/src/agent.rs` makes one `EndpointBound` per endpoint with the largest window and output of its models.
- `checks.rs`'s `model` becomes: the charter's model's endpoint has a bound holding its window and output. If `What::Agent` needs no other wording, keep it.
- A test that asserted a model's name was refused or undeclared now asserts on its endpoint; where that loses its point because the rule is gone, delete it and name it in the report.
- `AgentShortfall` keeps its variants and the client's wire and words stay as they are.

## Steps

`jig-domain-link`: `EndpointBound`, `Bounds.endpoints`. `ModelNeed` gains `endpoint`, from `fleet_needs::model` reading `Model.endpoint`. `needs.rs`: `eligibility`, `declarations`, `charter_shortfall`, `evidence` and `allows_models` read the bound of `need.endpoint`; `empty_evidence` empties `endpoints`. `CATALOGUE_NAME_BYTES` goes if nothing reads it. The schema: `record EndpointBound { endpoint: u32 window: u32 output: u32 }`, `endpoints: list 64 EndpointBound`; regenerate with the generator the crate's README or build names, never by hand.

Tests, in `jig/crates/jig-domain-fleet/src/tests/needs.rs`, each breaking its rule once by hand:

- `a_model_on_a_served_endpoint_is_admitted_by_any_name`
- `a_need_past_its_endpoints_window_or_output_falls_short`
- `a_need_on_no_served_endpoint_leaves_the_charter_ineligible`
- `evidence_keeps_every_endpoint_a_hello_declared`

`cargo fmt`; `python3 -I -B jig/scripts/old_forms.py` and temper's; `heavy cargo clippy` on the changed packages, `--all-targets -- -D warnings`; `heavy cargo nextest run` on them and their direct dependents; `heavy cargo check --workspace --tests`. Commit and push.

## Report

The tests deleted and why. Whether a hello's bound changed any refusal's words.
