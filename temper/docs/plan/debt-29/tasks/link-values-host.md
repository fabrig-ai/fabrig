+++
id = "link-values-host"
part = "link-values"
kind = "migration"
needs = ["link-readers-temper", "link-values-fleet"]
files = ["jig/crates/jig-domain-host/**", "jig/crates/jig-protocol-engine-workers/**", "jig/crates/jig-protocol-worker-link/**", "jig/crates/jig-protocol-worker-agents/**", "jig/crates/jig-protocol-charter/**", "jig/crates/jig-domain-hub-link/**", "jig/tests/core/**", "jig/tests/host/**", "jig/tests/worker-link/**", "jig/tests/engine-workers/**", "jig/tests/ops/**", "jig/tests/channel/**", "jig/testing/**", "jig/examples/ops/domain/**", "jig/tests/boundary/**", "jig/docs/design/domain/root.status.md", "jig/docs/development/debt/D006-link-values-two-paths.md", "Cargo.lock"]
effort = "medium"
minutes = 70
+++

# `jig-domain-host` re-exports no link value, and D006 is paid

## Goal

The last part of jig's D006. `jig-domain-host/src/lib.rs`'s two `pub use jig_domain_link` lines go, and jig's readers of its 21 names through it name `jig_domain_link`: 390 uses at the cut. `host::AgentOperation` becomes `jig_domain_link::Operation`.

Acceptance:

- The measure reads 0: `rg -n 'pub use jig_domain_link' jig/crates | wc -l`.
- `rg -n 'pub (use|type) .*\b(SourceName|GrantTable|LoginState|ModelBound|SignOutResult|SourceFailure|LoginMethod|CountKind|JobBound|JobKind)\b' jig temper -g '*.rs'` finds only `jig-domain-link`'s own exports, as the item's **Paid when** asks. Name any other.
- `rg -n 'AgentOperation' jig temper -g '*.rs'` finds nothing, unless a file keeps it as its own `use .. as`, which you name.
- `jig/docs/development/debt/D006-link-values-two-paths.md` is deleted, and `jig/docs/design/domain/root.status.md`'s sentence citing it is gone, leaving **Not yet**'s other entries as they are.
- The net is at most +130 lines.

## Inputs

- `temper/docs/plan/debt-29/notes/link-values.md`, whole.
- `jig/docs/development/debt/D006-link-values-two-paths.md`.
- `jig/docs/design/domain/root.md`, 2, and `root.status.md`.
- `temper/docs/plan/debt-29/notes/common.md`.

## Owns and may change

Owns the `use` lines and paths of these values in jig, each manifest's dependency line, the item, and that one status sentence. Nothing else of those files. `Cargo.lock` follows by cargo.

Start by deleting the two lines: the compiler then names every reader, and the packages the cut saw are `jig-protocol-engine-workers` (87 uses), `jig-protocol-worker-link` (76), `jig-host-world` (67), `jig-protocol-worker-agents` (48), `jig-worker-link-world` (39), `jig-test-service` (33), `jig-test-worker` (31), `jig-engine-workers-world` (28), `jig-channel-kit` (18), `jig-test-domain` (16), `jig-core-world`, `jig-domain-hub-link`, `jig-ops-domain`, `jig-ops-world`, `jig-core-kit`, `jig-channel-world`, `jig-protocol-charter` and `jig-test`.

- **The boundary's lists:** `engine_worker_channels_link_only_their_boundary_configuration_and_foundation` and `worker_link_links_only_its_wire_host_and_channel_vocabulary` in `jig/tests/boundary/src/tests/graph.rs` list exact dependencies, and each gains `jig-domain-link`, as `jig/docs/design/domain/root.md`, 2 allows a shared leaf value crate. The tests' names stay. Report each list you change.
- **`jig-domain-host` depends on `skein-lib` and `jig-domain-link` alone** (`host_hub_links_only_foundation_and_link_values`), and that stays.
- **The hub carries smith's payloads as bytes:** none of this touches them. `jig-protocol-worker-agents` links smith; its smith paths stay.

## Steps

Follow the note. Keep a private `use jig_domain_link::{..};` at `jig-domain-host`'s root for its own `crate::` readers. Then what the note's **What runs** lists. Commit, push.

## Report

As your text says, with the searches' readings, the lists you changed, and the net lines, and the three branches' net together if you can read them from the trunk.
