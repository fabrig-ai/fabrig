+++
id = "jig-kits-close"
part = "jig-kits"
kind = "feature"
needs = ["temper-kit-names", "skein-world-readers"]
files = ["jig/tests/boundary/src/tests/kits.rs", "jig/tests/boundary/src/tests/graph.rs", "jig/docs/development/debt/D031-worlds-as-kits.md", "jig/docs/design/domain/testing.status.md", "jig/docs/design/README.md", "jig/docs/design/client/domain/testing.md", "jig/docs/design/client/domain/client.md", "jig/docs/design/client/domain/root.md", "jig/docs/design/client/domain/README.md", "jig/docs/design/protocol/client.md", "jig/docs/design/domain/engine.md", "jig/docs/design/domain/hosts.md", "jig/docs/design/domain/root.md", "jig/docs/design/domain/connectors.md"]
effort = "medium"
minutes = 20
+++

# `PENDING` goes, and D031 goes

## Goal

Jig D031's **Paid when**: the boundary's list is empty, so it goes, with what only it needed.

Acceptance:

- `PENDING`, its `pending` parameter and `pending_world_dependency` go from `kits.rs`, and `graph.rs`' call with them. `world_breaches` names any `-world` edge not on `WORLD_KITS`. The rule's fixture tests stay, each reading a breach; a row that only tested a pending entry goes.
- **It must bite:** add `jig-core-world` to one kit's dependencies, and watch it fail, then restore.
- The Measure's `awk` reads 0, and `rg -n '^jig-[a-z-]*-world\b' jig temper -g Cargo.toml` reads nothing, but `smith-agent-process-world` isn't that pattern, and neither is `skein-world`.
- `jig/docs/development/debt/D031-worlds-as-kits.md` is deleted. `jig/docs/design/domain/testing.status.md`'s **Not yet** on 7 goes. `python3 -I -B tools/citations/citations.py --all` reads 0, and `python3 -I -B tools/debt/debt.py | rg 'D031'` shows no jig row.
- **Design's names.** The files in the header name these worlds. A sentence that says a world holds a suite stays true. A sentence that says a world's library offers something, or that another package runs on it (`client/domain/testing.md`: "`jig-client-world`, on which an application's client worlds run"; the `README.md` row's own), names the kit, one word each.
- `heavy cargo nextest run -p jig-boundary` passes.

## Inputs

- `temper/docs/plan/debt-24/notes/common.md`
- `jig/docs/development/debt/D031-worlds-as-kits.md`
- `jig/docs/design/domain/testing.md`, 7

## Owns and may change

- **Owns:** the files in the header. This departs from the common note on design for those files, in a package's name alone.

## Steps

The list; the break; the docs; the deletion; the checks. `cargo fmt`; `(cd jig && python3 -I -B scripts/old_forms.py)`; `heavy cargo clippy -p jig-boundary --all-targets -- -D warnings`. Commit and push.

## Report

The common note's items; the break's failure; the Measure; the docs changed. Across D031's tasks, the kits' net Rust lines, as the plan says they can't fall.
