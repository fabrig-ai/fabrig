+++
id = "jig-kits-close"
part = "jig-kits"
kind = "feature"
needs = ["temper-kit-names", "skein-world-readers"]
files = ["Cargo.toml", "Cargo.lock", "jig/testing/jig-people-kit/**", "jig/testing/jig-tasks-kit/**", "jig/tests/people/Cargo.toml", "jig/tests/people/src/**", "jig/tests/tasks/Cargo.toml", "jig/tests/tasks/src/**", "temper/tests/engine/domain/Cargo.toml", "temper/tests/engine/domain/tests/suite/referee.rs", "temper/tests/engine/domain/tests/suite/panel_before_change.rs", "jig/docs/design/domain/testing.md", "jig/tests/boundary/src/tests/kits.rs", "jig/tests/boundary/src/tests/graph.rs", "jig/docs/development/debt/D031-worlds-as-kits.md", "jig/docs/design/domain/testing.status.md", "jig/docs/design/README.md", "jig/docs/design/client/domain/testing.md", "jig/docs/design/client/domain/client.md", "jig/docs/design/client/domain/root.md", "jig/docs/design/client/domain/README.md", "jig/docs/design/protocol/client.md", "jig/docs/design/domain/engine.md", "jig/docs/design/domain/hosts.md", "jig/docs/design/domain/root.md", "jig/docs/design/domain/connectors.md"]
effort = "medium"
minutes = 45
+++

# `PENDING` goes, and D031 goes

## Goal

Jig D031's **Paid when**: the boundary's list is empty, so it goes, with what only it needed.

Temper's last two edges go first, by `temper/docs/plan/debt-24/notes/kits.md`'s recipe, as the rest of this part's did:

- **`jig-people-kit`** holds `referee` alone: `git mv jig/tests/people/src/referee.rs` to the kit. `jig-people-world`'s `src/lib.rs` drops `pub mod referee;` and adds `pub use jig_people_kit::*;`, so `crate::referee` and its suite's paths hold.
- **`jig-tasks-kit`** holds the tasks world's whole library, as `world` reads every other module: `lib.rs`'s items and the `src/` modules move, and `jig-tasks-world`'s `src/lib.rs` is its `//!` lines and `pub use jig_tasks_kit::*;`.
- Each kit is made as `temper/docs/plan/debt-24/tasks/kit-packages.md` says: a `description`, the inherited fields, `[lints]` by `tools/lints/lints.py --set harness`, the `ordinary` `clippy.toml`, a root member and workspace dependency, the lock by `cargo metadata`. A pure move in its own commit.
- `temper/tests/engine/domain` swaps its two dev-dependency lines for the kits and renames the crate paths in `tests/suite/referee.rs` and `tests/suite/panel_before_change.rs`, nothing else; `rustfmt` may rewrap. Every test stays, with its name, its assertions and its seeds: `heavy cargo nextest list` counts the same for `jig-people-world`, `jig-tasks-world` and `temper-engine-domain-world`.
- `jig/docs/design/domain/testing.md`, 7's list of kits gains both: the people's referee (`jig-people-kit`), the tasks' world and referees (`jig-tasks-kit`), in the list's words and nothing else.

Acceptance:

- `PENDING`, its `pending` parameter and `pending_world_dependency` go from `kits.rs`, and `graph.rs`' call with them. `world_breaches` names any `-world` edge not on `WORLD_KITS`. The rule's fixture tests stay, each reading a breach; a row that only tested a pending entry goes.
- **It must bite:** add `jig-core-world` to one kit's dependencies, and watch it fail, then restore.
- The Measure's `awk` reads 0, and `rg -n '^jig-[a-z-]*-world\b' jig temper -g Cargo.toml` reads nothing, but `smith-agent-process-world` isn't that pattern, and neither is `skein-world`.
- `jig/docs/development/debt/D031-worlds-as-kits.md` is deleted. `jig/docs/design/domain/testing.status.md`'s **Not yet** on 7 goes. `python3 -I -B tools/citations/citations.py --all` reads 0, and `python3 -I -B tools/debt/debt.py | rg 'D031'` shows no jig row.
- **Design's names.** The files in the header name these worlds. A sentence that says a world holds a suite stays true. A sentence that says a world's library offers something, or that another package runs on it (`client/domain/testing.md`: "`jig-client-world`, on which an application's client worlds run"; the `README.md` row's own), names the kit, one word each.
- `heavy cargo nextest run -p jig-boundary -p jig-people-world -p jig-tasks-world` passes, and `heavy cargo nextest run -p temper-engine-domain-world -E 'test(/^referee::/) | test(/^panel_before_change::/)'` selects and passes the two files' tests.

## Inputs

- `temper/docs/plan/debt-24/notes/common.md`
- `temper/docs/plan/debt-24/notes/kits.md`
- `temper/docs/plan/debt-24/tasks/kit-packages.md`
- `jig/docs/development/debt/D031-worlds-as-kits.md`
- `jig/docs/design/domain/testing.md`, 7

## Owns and may change

- **Owns:** the files in the header. This departs from the common note on design for those files, in a package's name alone, and for `testing.md`, 7's list, in its two kits alone.
- **In `temper/tests/engine/domain`:** the two dev-dependency lines and the crate's name in a path, nothing else.

## Steps

The two kits and temper's edges; the list; the break; the docs; the deletion; the checks. `cargo fmt`; `(cd jig && python3 -I -B scripts/old_forms.py)`; `python3 -I -B tools/lints/lints.py`; `heavy cargo clippy -p jig-boundary -p jig-people-kit -p jig-tasks-kit --all-targets -- -D warnings`. Commit and push.

## Report

The common note's items; the break's failure; the Measure; the docs changed. Across D031's tasks, the kits' net Rust lines, as the plan says they can't fall. Under `## Left`, for fabrig/debt: the tests of jig's referees in temper's `tests/suite/referee.rs` that copy jig's own suites (`authority_referee_rejects_a_person_without_a_grant` observes what `rejects_an_observer_routed` does); they stay here, as no test goes in this task.

## Answers

2026-11-05. `temper/tests/engine/domain` still depends on `jig-people-world` and `jig-tasks-world`, as used libraries: extract kits and migrate temper, or exempt the two edges and narrow the acceptance? Extract and migrate. An exemption would contradict D031's **Paid when** and `jig/docs/design/domain/testing.md`, 7, which name only `skein-world` and `smith-agent-process-world`, and the plan's own line that temper's edges go. `jig-people-kit` (`referee`) and `jig-tasks-kit` (the library) are pure moves by the kits note, with both worlds keeping a re-export. No test goes or changes; rewriting `panel_before_change.rs` off the tasks world would risk its fixture, and removing `referee.rs`'s copies is fabrig/debt's, reported under `## Left`. This task is widened as above because its acceptance can't hold otherwise; the commits already on your branch stand.
