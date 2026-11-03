+++
id = "checkout-heads-engine"
part = "checkout"
kind = "feature"
needs = ["checkout-heads"]
files = ["temper/crates/temper-engine-domain-forge/src/items.rs", "temper/crates/temper-engine-domain/src/boundary.rs", "temper/crates/temper-engine-domain/src/route/forge_route.rs", "temper/crates/temper-service/src/translate.rs", "temper/crates/temper-service/src/tests/translate.rs"]
effort = "medium"
minutes = 45
+++

# checkout-heads-engine: the forge's start with heads merged in, down to the worker

## Goal

The forge connector can name a producer's start as its landing branch with heads merged in,
and the engine carries it to the worker's `Start::Merged` (contract C4's start half). Nothing
chooses it yet: task `early-route` does.

Acceptance:

- `temper_engine_domain_forge::items::Start::Merged { branch, heads }` and
  `temper_engine_domain::ForgeStart::Merged { branch, heads }` as C4 has them.
- `run_workspace` (`temper-engine-domain/src/route/forge_route.rs`) maps the one to the
  other, the heads in order, with the repository's `save`, `push` and `landing` as for a
  `Base` start.
- The service's translation (`temper-service/src/translate.rs`, `start()`) encodes
  `ForgeStart::Merged` as the wire's `Start::Merged`, the heads in order as full 32-byte ids.
- Tests: in `temper-service/src/tests/translate.rs`, beside the `Merge` cases,
  `a_merged_start_carries_its_branch_and_heads_in_order` (encode, then decode as the
  worker's routing does, equal to the worker's `Start::Merged`); in the root's own tests, or
  the forge connector's, `a_merged_item_maps_to_a_merged_forge_start_saving_as_a_base`
  (place it where `run_workspace`'s existing `Merge` mapping is tested; if none is, a test in
  `temper-engine-domain`'s `src/tests/` beside its route tests).
- Every test of `temper-engine-domain-forge`, `temper-engine-domain` and `temper-service`
  passes.

## Inputs

- `temper/docs/design/protocol/channel.md`, section 4 (`start`'s row).
- `temper/docs/design/domain/worker.md`, 4.1 (where to start).
- `temper/docs/plan/reference-m6/notes/common.md`; `notes/contracts.md`, C3 and C4.

## Owns and may change

Owns the files in the header: in `boundary.rs`, `ForgeStart` alone (task `checkout-heads`
added `Preparation::Conflicted` there); in `forge_route.rs`, `run_workspace`'s mapping
alone; in `items.rs`, the enum alone (`workspace_item`'s choice is `early-route`'s).

- Reader and neighbour files: every exhaustive match on `items::Start` or `ForgeStart`
  (`temper-engine-domain-forge/src/tests.rs`, the root's tests, `temper/tests/engine/domain/**`),
  the new arm only.

## Steps

1. The two variants, the mapping and the encoding; the tests above.
2. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p
   temper-engine-domain-forge -p temper-engine-domain -p temper-service --all-targets -- -D
   warnings`; the focused tests of those packages. Commit, push.

## Report

As the implementer's text says.
