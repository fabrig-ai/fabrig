+++
id = "client-world"
part = "client-world"
kind = "feature"
needs = ["view-words", "domain-fixture"]
files = [
  "temper/tests/client/src/fixtures.rs",
  "temper/tests/client/src/fixtures/**",
  "temper/tests/client/src/front.rs",
  "temper/tests/client/src/front/**",
  "temper/tests/client/src/app.rs",
  "temper/tests/client/src/component.rs",
  "temper/docs/plan/limits-fixtures/figures/client-world.md",
]
effort = "high"
minutes = 90
+++

# client-world: what the joined client world builds before a story starts

## Goal

`temper-client-world::suite` reads 24.3 CPU s over 384 tests at a 49 ms median, with the highest floor of the request's table: 23 ms. It joins the client root and its view to the real engine, so `view-words`' and `domain-fixture`'s savings reach it. Profile what's left, and fix it at the cause.

**Acceptance:**

- `figures/client-world.md` gives `Ir` before and after, by test name, for the binary's cheapest test and a median test, each with what it built before its first assertion.
- The causes table has every cause over the stop rule: fixed, or with what fixing it would take. Say how much of the floor is the loader.
- Every test of `temper-client-world` passes, in the focused suite and in `--profile fuzzy`, and a seed still replays. No fixture's value changes.

## Inputs

- `temper/docs/plan/limits-fixtures/notes/common.md`: the rules, how to profile, your figures
- `temper/docs/plan/limits-fixtures/inputs/request.md`, items 1 and 3 and `For every task of this plan`
- `temper/docs/plan/limits-fixtures/figures/view-words.md` and `figures/domain-fixture.md`, what those tasks found
- `skein/docs/foundation/testing-strategy.md`, 8 and 8.1
- `temper/docs/design/domain/testing.md`, 6.3

## Owns and may change

- **Owns:** `temper/tests/client/src/fixtures.rs` and `src/fixtures/**`, `src/front.rs` and `src/front/**`, `src/app.rs`, `src/component.rs`; `figures/client-world.md`.
- **Leave to other plans:** `temper/tests/client/tests/suite/**`, which `look-round-13` and `model-catalogue-2` are changing, but for one line where a test can take a cheaper fixture it already has.
- **May change, where the profile puts the cause, small, with a focused test:** `jig/testing/jig-client-kit/src/**`, `jig/testing/jig-client-test-domain/src/**`, `skein/testing/skein-fake-person/src/**`, `skein/testing/skein-world/src/**`, `temper/tests/engine/domain/src/forge_stand.rs`, `temper/tests/engine/domain/src/direct.rs`, `temper/crates/temper-client-domain/src/**`, `temper/crates/temper-client-view/src/**` outside its words, `skein/crates/skein-ui/src/**`.
- **Options, each pre-approved:**
  - **If `Joint::with_engine`'s double start is the cost** (below): `restored_forges` runs a whole first start and `Driver::cold` then opens it. `domain-fixture` will have made these cheaper, so measure before you touch them. A world that needs no first-start rows beyond a few may give them to the store directly, with one test that holds them equal to the real start's, or else it keeps the real start, and you say so.
  - **If `Truth::after(core_rows(&engine))` or `Composition::new` is the cost,** a build that's only read by some stories may wait for its first use.
  - **If `World::new`, `decision_words` or `App::new` is the cost,** the same care. A value it needs once may be built once, and `fixtures::phrasebooks` (`src/fixtures.rs:186`) already keeps a book by size.
  - **If a harness's fixed iterations are the cost** (`iterations: 4096` is a bound; check it's not also the work), stop once nothing more can happen, with the bound kept, as long as the world's order and a seed's replay stay the same.
  - **If the cost is in the binary and not in any test,** (relocations, as `notes/common.md` says), a line in the causes table with its share, and leave it.

## Steps

**From reading, not found (`notes/common.md`):**

- `fixtures::world` (`src/fixtures.rs:205`) calls `Joint::new` (`src/front.rs:479`), which builds `engine_limits`, `Store::with_limits` and two `walking::config`s.
- `with_engine_with_refused_hello` (`src/front.rs`) builds `Transport`, `Composition::new` and `restored_forges` (`:145`, a whole first start and forge rows), then `Driver::cold`, `ForgeStand::with`, `settle`, the worker's hello, `settle` and `Truth::after(core_rows(&engine))`.
- `world_with` (`src/fixtures.rs:337`) adds `World::new` with `decision_words` (`:322`), and each client adds `App::new` (`src/app.rs`): `phrasebooks`, `view_bounds`, `domain::unlinked` and `View::new`.

**The work:**

1. Choose the tests and take the before figures, at your branch's start. Use `-p temper-client-world --test suite`.
2. Read the inclusive annotations, and split each test's `Ir` among the pieces above.
3. Fix the largest cause. Run the narrowest test (`heavy cargo nextest run -p temper-client-world --test suite -E 'test(=NAME)'`), and read its `Ir` again.
4. Repeat until the stop rule (`notes/common.md`).
5. `cargo fmt`; `python3 -I -B <tree>/scripts/old_forms.py` for each tree you touched; `heavy cargo clippy -p temper-client-world --all-targets -- -D warnings`, naming any other package you changed.
6. `heavy cargo nextest run -p temper-client-world`, and `heavy cargo nextest run --profile fuzzy -p temper-client-world`. A change outside this package also runs that package's own tests, and its readers by `cargo tree -i PKG --depth 1`, with `heavy cargo check -p PKG --tests` for those in another tree.
7. Write the figures, commit and push.

## Report

The counts before and after, the causes fixed and not fixed, how much of the floor is the loader, and how much of the world's cost `view-words` and `domain-fixture` had already taken.
