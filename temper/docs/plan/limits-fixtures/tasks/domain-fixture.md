+++
id = "domain-fixture"
part = "engine-world"
kind = "feature"
needs = []
files = [
  "temper/tests/engine/domain/src/forge_routes.rs",
  "temper/tests/engine/domain/src/fixtures.rs",
  "temper/tests/engine/domain/src/store.rs",
  "temper/tests/engine/domain/src/walking.rs",
  "temper/tests/engine/domain/src/direct.rs",
  "temper/tests/engine/domain/src/oracle/**",
  "temper/docs/plan/limits-fixtures/figures/domain-fixture.md",
]
effort = "high"
minutes = 120
+++

# domain-fixture: what the engine world builds before a story starts

## Goal

`temper-engine-domain-world::suite` reads 52.9 CPU s over 893 tests, with a floor of 14 ms where a process costs 2 to 3 ms. Its `forge_routes` reads 31.2 s at a 55 ms median, and `root_routes` 3.4 s at 25 ms. Profile what every test builds before its first assertion: the world, its store and its first start. Fix it at the cause, in the product where the product pays it too. What a `forge_routes` story repeats after that is `forge-routes-steps`.

**Acceptance:**

- `figures/domain-fixture.md` gives `Ir` before and after, by test name, for: the binary's cheapest test; the cheapest and a median test of `forge_routes::`; and the cheapest and a median test of `root_routes::`. Each has what it built before its first assertion.
- The causes table has every cause over the stop rule: fixed, or with what fixing it would take. Say how much of the floor is the loader.
- A product change has its own focused test in its crate.
- Every test of `temper-engine-domain-world` passes, in the focused suite and in `--profile fuzzy`, and a seed still replays. A world's first start stays the real first start, unless the option below holds it equal.
- `temper-client-world`, `temper-conformance-world`, `temper-system-world` and `temper-service` compile and pass, since they read this crate.

## Inputs

- `temper/docs/plan/limits-fixtures/notes/common.md`: the rules, how to profile, your figures
- `temper/docs/plan/limits-fixtures/inputs/request.md`, items 1 to 3 and `For every task of this plan`
- `skein/docs/foundation/testing-strategy.md`, 8 and 8.1
- `temper/docs/design/domain/testing.md`, 6.1

## Owns and may change

- **Owns:** `temper/tests/engine/domain/src/forge_routes.rs` (the world's construction: `World::configured_settings` and what it calls), `src/fixtures.rs`, `src/store.rs`, `src/walking.rs`, `src/direct.rs`, `src/oracle/**`; `figures/domain-fixture.md`.
- **Leave to `forge-routes-steps`:** `World::until`, `tick`, `collect`, `flush` and the rest of a story's stepping, and `tests/suite/forge_routes/**`.
- **May change, where the profile puts the cause:** product code, small, with its own focused test: `temper/crates/temper-engine-domain/src/**`, `temper/crates/temper-service/src/store/**`, `temper/crates/temper-protocol-store/src/**`, `jig/crates/jig-domain-core/src/**`, `jig/crates/jig-domain-tasks/src/**`, `jig/crates/jig-protocol-store/src/**`, `jig/testing/jig-test-profile/src/**`, `jig/testing/jig-test-store-files/src/**`, `skein/crates/skein-kv/src/**`, `skein/crates/skein-fake-forge-domain/src/**`. Other plans are changing `temper-engine-domain/src/route/**` and `jig-protocol-store` right now, so keep a change there small and say so.
- **May change, call sites only:** `temper/tests/engine/domain/tests/suite/**`, a line where a test can take a cheaper fixture it already has.
- **Options, each pre-approved:**
  - **If building the configuration twice is the cost** (`std::array::from_fn` in `configured_settings`, `:321`), build the common part once and derive the second, as long as both stay equal to what they are.
  - **If `first_start_over`'s cold start is the cost,** look first at why a first start is dear: that's a person's first start too. A table sized past its use, a repeat encoding or a scan in the core, the root or the store is the fix.
  - **If it's still the largest after that,** and most worlds use one of a few configurations, you may give a first start's rows directly to the store for those worlds, with one test that holds them equal to a real `first_start_over`. If that can't be held cheaply and simply, keep the real start, and give the counts.
  - **If `Domain::new` or `Store::with_limits` allocates for its limits** (`Store::new` uses `widest()`, the world's largest limits), a table built at first use, or sized by what's in it, is the fix: in the product, where the product pays it.
  - **If `fake::Domain::new` or `fake::repository` is the cost,** it's a fake and no person pays it: fix it in `skein-fake-forge-domain`, small, or give the world less of it.
  - **If the cost is in the binary and not in any test** (relocations, as `notes/common.md` says), a line in the causes table with its share, and leave it.

## Steps

**From reading, not found (`notes/common.md`):**

- `World::configured_settings` (`src/forge_routes.rs:287`) builds two configurations and limits through `std::array::from_fn` (`:321`): `walking::limits`, `walking::config` and, with a change, grants and charters.
- It then builds `forge_world::fake_config`, `fake::Domain::new` and `fake::repository` (`:809`), then `restored_over(Store::with_limits(&store_limits), bootstrap, &limits)` (`:921`), which in `src/fixtures.rs:227` runs `first_start_over`, a real `Driver::cold` to its two commits, and then `store.seed`.
- `Store::with_limits` (`src/store.rs:128`) opens the real store over memory, `Domain::new` builds the root, and the world starts with `Restart`, so a story reads the first start back through the store.
- `root_routes` and others build worlds through `walking` and `direct.rs`: read which of these they share.

**The work:**

1. Choose the tests and take the before figures, at your branch's start. Use `-p temper-engine-domain-world --test suite`.
2. Read the inclusive annotations. Split each test's `Ir` into the loader, configuration, fake, first start, opening the store, `Domain::new` and the story.
3. Fix the largest cause. Run the narrowest test (`heavy cargo nextest run -p temper-engine-domain-world --test suite -E 'test(=NAME)'`), and read its `Ir` again.
4. Repeat until the stop rule (`notes/common.md`).
5. `cargo fmt`; `python3 -I -B <tree>/scripts/old_forms.py` for each tree you touched; `heavy cargo clippy -p temper-engine-domain-world --all-targets -- -D warnings`, naming any other package you changed.
6. `heavy cargo nextest run -p temper-engine-domain-world`, `heavy cargo nextest run --profile fuzzy -p temper-engine-domain-world`, then `heavy cargo nextest run -p temper-client-world -p temper-conformance-world -p temper-system-world -p temper-service`. A change in a jig or skein crate also runs that crate's own tests and `heavy cargo check -p PKG --tests` for its direct dependents, each named.
7. Write the figures, commit and push.

## Report

The counts before and after, the causes fixed and not fixed, how much of the floor is the loader, and any product change, with what reads it.
