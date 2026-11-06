+++
id = "service-fixture"
part = "service"
kind = "feature"
needs = []
files = [
  "temper/crates/temper-service/src/tests/fixtures.rs",
  "temper/crates/temper-service/src/tests/fixtures/**",
  "temper/crates/temper-service/src/tests/setup.rs",
  "temper/crates/temper-service/src/tests/stock.rs",
  "temper/crates/temper-service/src/tests/stock_brief.rs",
  "temper/docs/plan/limits-fixtures/figures/service-fixture.md",
]
effort = "high"
minutes = 120
+++

# service-fixture: what `temper-service`'s tests build before they check anything

## Goal

`temper-service`'s 1,111 tests read 46.3 CPU s, and its `setup` module alone 24.9 s at a 77 ms median. Profile the shared fixture in `tests/fixtures.rs`, and fix what it repeats and what it overpays for, at the cause. What `setup` tests do beyond the fixture is `setup-steps`, after this.

**Acceptance:**

- `figures/service-fixture.md` gives `Ir` before and after, by test name, for: the cheapest and a median test of `setup::`; the cheapest and a median test of `tests::stock::`; and the cheapest and a median test of the rest of the binary. Each has what it built before its first assertion.
- The median `setup` test's `Ir` falls to at most half. If the profile shows that can't be done, the figures say what stops it, with the counts.
- The causes table has every cause over the stop rule: fixed, or with what fixing it would take.
- Every `temper-service` test passes, with the base's count plus any you add. None is removed or weakened, and no fixture's value changes (`notes/common.md`).

## Inputs

- `temper/docs/plan/limits-fixtures/notes/common.md`: the rules, how to profile, your figures
- `temper/docs/plan/limits-fixtures/inputs/request.md`, items 1 to 3 and `For every task of this plan`
- `skein/docs/foundation/testing-strategy.md`, 8 and 8.1

## Owns and may change

- **Owns:** `temper/crates/temper-service/src/tests/fixtures.rs` and `tests/fixtures/**`; the fixture helpers of `tests/setup.rs` (`service`, `routing_service`, `routing_service_at`, `github_config`, `load_routing`); the helpers of `tests/stock.rs` and `tests/stock_brief.rs` (`generated`, `single_model`, `without_projects`, `engine`); `figures/service-fixture.md`.
- **Leave to `setup-steps`:** the rest of `tests/setup.rs` and `tests/setup_*.rs`, and `src/setup/**`.
- **May change, where the profile puts the cause:** product code in `temper/crates/temper-service/src/**`, `temper/crates/temper-init/src/**`, `temper/crates/temper-protocol-store/src/**` and `jig/crates/jig-protocol-config/src/**`, small, with its own focused test.
- **May change, call sites only:** any `temper/crates/temper-service/src/tests/*.rs`, a line where a test can take a cheaper fixture it already has.
- **Options, each pre-approved:**
  - **If `engine()`'s parse is the repeat,** first stop the fixtures calling it again: pass the settings or limits down through private helpers beside the public ones. If it's still large, keep one parsed value per process and clone it. `jig_protocol_config::Engine` and `settings::Engine` derive `Debug` alone, so adding `Clone` to them and what they hold is allowed. If a member can't be `Clone`, don't force it: stay with passing down, and say so.
  - **If `completed_maps`'s `ready()` is the repeat,** first remove the second and third builds (below). If it's still the largest, write the start's mark into the maps directly, and add one test in `tests/fixtures.rs`'s neighbours that holds what you write equal to what a real start leaves. If they can't be held equal cheaply, keep the real start, and say so.
  - **If `Service::prepared`, `limits::engine`, `jig_protocol_config::pass`, `Store::over` or `temper_protocol_store::application` is the cost,** that's the product's cost too, and a person's box pays it. Fix it there if small. If not small, a line in the causes table with what it would take.
  - **If `generated()` is `tests::stock`'s cost,** `temper_init::files` runs for each call, and `fixtures::with` reparses the document for each edit. Build it once in a test, or keep it per process and copy the bytes, since it returns `Box<[u8]>`.

## Steps

**From reading, not found (`notes/common.md`):**

- `fixtures::engine()` (`tests/fixtures.rs:64`) parses `ENGINE`, builds the stock through `crate::stock::defaults` and runs `jig_protocol_config::pass`, for each call. `store_limits` (:272) calls it, `memory_store` (:278) and `records_store` (:289) call it twice each, and `start_with_worker` (:487) and `configured_engine` (:1297) call it.
- `records_store` (:289) calls `completed_maps` (:283), which calls `ready()` (:643): a whole started service and its restart. It then builds `memory_store` and `memory_store_for`.
- `configured_prepared` (:1374) calls `records_store`. `routing_service_at` (`tests/setup.rs:40`) calls `configured_service`, then `fixtures::replace_store(&mut service, fixtures::records_store())`, which builds it again. `tests/setup.rs`'s `service` (:8) calls `routing_service` and then `fixtures::ready()`.
- So a `setup` test seems to parse the engine about a dozen times and to start a service three times before its first assertion. The profile says how much of it counts.

**The work:**

1. Choose the tests and take the before figures, at your branch's start.
2. Read the inclusive annotations and write down what each test builds.
3. Fix the largest cause, by the options above. Run the narrowest test that checks it (`heavy cargo nextest run -p temper-service --lib -E 'test(=NAME)'`), and read its `Ir` again.
4. Repeat until the stop rule (`notes/common.md`).
5. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-service --all-targets -- -D warnings`, naming any other package you changed.
6. `heavy cargo nextest run -p temper-service`. A product change also runs the focused tests of what reads it: `temper-engine-domain-world`, `temper-client-world`, `temper-conformance-world` and `temper-system-world` for `temper-service`, with `heavy cargo check -p PKG --tests` for jig's other readers of what you changed.
7. Write the figures, commit and push.

## Report

The counts before and after, the causes fixed and not fixed, what you changed outside the owned files, and whether `setup`'s median met its half.
