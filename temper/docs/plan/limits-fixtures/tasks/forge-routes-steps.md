+++
id = "forge-routes-steps"
part = "engine-world"
kind = "feature"
needs = ["domain-fixture"]
files = [
  "temper/tests/engine/domain/src/forge_routes.rs",
  "temper/tests/engine/domain/src/store.rs",
  "temper/tests/engine/domain/src/lib.rs",
  "temper/tests/engine/domain/src/protocol.rs",
  "temper/tests/engine/domain/tests/suite/forge_routes.rs",
  "temper/tests/engine/domain/tests/suite/forge_routes/**",
  "temper/docs/plan/limits-fixtures/figures/forge-routes-steps.md",
]
effort = "high"
minutes = 120
+++

# forge-routes-steps: what a `forge_routes` story repeats beyond its fixture

## Goal

`forge_routes` is the suite's largest single set: 425 tests, 31.2 CPU s at a 55 ms median. `domain-fixture` has made the world's start cheaper. Profile what most stories still repeat, which reading suggests is the walk from a started world to an adopted repository and a running change, and each tick's cost. Fix it at the cause: in the product where a person's box pays it too (request, item 2).

**Acceptance:**

- `figures/forge-routes-steps.md` gives `Ir` before and after, by test name, for: the cheapest and a median test of `forge_routes::`, and a story through `change_world`. Each says what it spends beyond its fixture: sign-in and adoption, the change's walk, and its own steps. Use `figures/domain-fixture.md`'s `forge_routes::` tests where they fit.
- The causes table has every cause over the stop rule: fixed, or with what fixing it would take.
- A product change has its own focused test, and changes no behaviour.
- Every test of `temper-engine-domain-world` passes, in the focused suite and in `--profile fuzzy`, and a seed still replays. `temper-client-world`, `temper-conformance-world` and `temper-system-world` pass.

## Inputs

- `temper/docs/plan/limits-fixtures/notes/common.md`: the rules, how to profile, your figures
- `temper/docs/plan/limits-fixtures/inputs/request.md`, items 2 and 3 and `For every task of this plan`
- `temper/docs/plan/limits-fixtures/figures/domain-fixture.md`, what the fixture's task found
- `skein/docs/foundation/testing-strategy.md`, 8 and 8.1
- `temper/docs/design/domain/testing.md`, 6.1

## Owns and may change

- **Owns:** in `temper/tests/engine/domain/src/forge_routes.rs`, a story's stepping (`World::send`, `flush`, `collect`, `tick`, `until`, `sign_in`, `adopt_with_checks`, `change_world_funded`) and everything `domain-fixture` left; `src/store.rs`'s `apply`, `tick` and `request`; `src/lib.rs`'s `poll`; `src/protocol.rs`; a test's own setup lines in `tests/suite/forge_routes.rs` and `tests/suite/forge_routes/**`; `figures/forge-routes-steps.md`.
- **Leave to `goal-delivery`:** `tests/suite/forge_routes/goal_delivery.rs`, which that plan is changing.
- **May change, where the profile puts the cause, small, with a focused test:** `temper/crates/temper-engine-domain/src/**`, `temper/crates/temper-engine-domain-forge/src/**`, `temper/crates/temper-service/src/store/**`, `temper/crates/temper-protocol-store/src/**`, `jig/crates/jig-domain-core/src/**`, `jig/crates/jig-domain-tasks/src/**`, `jig/crates/jig-domain-people/src/**`, `jig/crates/jig-protocol-store/src/**`, `skein/crates/skein-forge-client/src/**`, `skein/crates/skein-kv/src/**`, `skein/crates/skein-codec/src/**`, `skein/crates/skein-fake-forge-domain/src/**`.
- **`limits-worlds` profiles the same engine, store and fake from the real loop.** Before you fix a shared cause in a product crate, `git fetch origin` and read `git log origin/limits-worlds/integration -- FILE`. If it's fixed there, take no second fix: say so in your causes table, and measure what's left. If you fix it first, the merge sorts it out.
- **Options, each pre-approved:**
  - **If a tick's fixed work is the cost** (`tick_with_reply_cut`, `:1762`, and `Domain::reclaim`, `temper/crates/temper-engine-domain/src/domain.rs:631`, which runs after each input as well as each tick), make the idle case cheap: a scan or copy that finds nothing is the product's cause, since a box pays it each turn too.
  - **If the store's round trip is the cost** (`store.rs`, `apply`, `tick`: real encoding, then the world's typed `rows` mirror, and `inflight` holds a clone of each write), remove what's copied or encoded twice. The real store stays real: rows still go through its codec.
  - **If `until(Until::Ready)` is the cost** (each `tick` moves the clock 100 ms, up to 200 times), find why a state takes many ticks, and leave the clock's meaning alone: a story's times are what it checks.
  - **If a story's walk to adoption is the largest,** and it's the same for many, that's work every one repeats (`adopt` has 59 uses, and `change_world` 56 in its forms). It's real work on real code, so make that code cheaper. Don't give a story a shortcut unless a test holds what it leaves equal to the real walk's rows, and say so.
  - **If `oracle_running` or `crate::oracle::config::copy` shows,** it's the world's: fix it here.
  - **Keep every story's checks.** A story's own steps are what it tests; a cause that's only there is a line in your causes table.

## Steps

**From reading, not found (`notes/common.md`):**

- `World::until` (`src/forge_routes.rs:1848`) runs `tick` up to 200 times, and `tick_with_reply_cut` (`:1762`) applies one store transaction, then `poll` (`src/lib.rs:33`, up to two `release`s and a `Resume`), `collect`, `flush`, `fire`, `release`, `collect`, `flush` and both `reclaim`s.
- `send` (`:1122`) flushes, and each input then runs `try_step`, `release`, `collect` and `reclaim` (`:1145`).
- `change_world_funded` (`:1992`) goes through `adopt` (`sign_in`, then `Until::Ready`, `SignedIn` and `Adopted`), then hello, the ask and `Until::Assigned`, and delegates a change. Most of these stories go through it or through `adopt`.

**The work:**

1. Choose the tests and take the before figures, at your branch's start. Use `-p temper-engine-domain-world --test suite`.
2. Read the inclusive annotations. Find the share of `tick`, `flush`, `Store::apply`, `reclaim` and the root's step. Then read the self annotation for what's under them: repeated encoding, clones, scans.
3. Fix the largest cause. Run the narrowest test (`heavy cargo nextest run -p temper-engine-domain-world --test suite -E 'test(=NAME)'`), and read its `Ir` again.
4. Repeat until the stop rule (`notes/common.md`).
5. `cargo fmt`; `python3 -I -B <tree>/scripts/old_forms.py` for each tree you touched; `heavy cargo clippy -p temper-engine-domain-world --all-targets -- -D warnings`, naming any other package you changed.
6. `heavy cargo nextest run -p temper-engine-domain-world`, `heavy cargo nextest run --profile fuzzy -p temper-engine-domain-world`, then `heavy cargo nextest run -p temper-client-world -p temper-conformance-world -p temper-system-world`. A change in a product crate also runs that crate's own tests, and its readers by `cargo tree -i PKG --depth 1`, with `heavy cargo check -p PKG --tests` for those in another tree.
7. Write the figures, commit and push.

## Report

The counts before and after, the causes fixed and not fixed, anything `limits-worlds` had already fixed, and any product change, with what reads it.
