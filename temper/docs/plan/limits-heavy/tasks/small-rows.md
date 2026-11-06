+++
id = "small-rows"
part = "rows"
kind = "feature"
needs = []
files = [
  "temper/crates/temper-client-tools/src/tests/jig/turn.rs",
  "temper/crates/temper-client-tools/src/tests/jig/reads.rs",
  "temper/crates/temper-client-tools/src/tests/fixtures/reads.rs",
  "temper/crates/temper-client-tools/src/tests/fixtures/digest.rs",
  "temper/crates/temper-service/src/tests/setup_lifecycle.rs",
  "temper/crates/temper-service/src/tests/accounts_host.rs",
  "temper/docs/plan/limits-heavy/figures/small-rows.md",
]
effort = "medium"
minutes = 60
+++

# Four single tests in the client's tools and the service

## Goal

Four tests the request names outside its table:

- `temper-client-tools`: `tests::jig::turn::a_document_sized_turn_prints_escaped_tool_names_with_the_shipped_limits` 1.1 CPU s and `reads::a_document_at_its_byte_bound_prints_its_maximal_json_escapes` 0.6.
- `temper-service`: `setup::tests::lifecycle_tests::setup_malformed_queries_are_bad_requests` 1.1 and `accounts::tests::expiry_and_native_paste_race_take_at_most_one_redirect` 0.9.

They belong where they are (`plan.md`, *Belonging*). Find why each costs what it does, and fix the cause.

Acceptance:

- All four still exist and assert what they assert today: the same shipped limits, the same sixteen malformed queries and their 400, and all 54 schedules of the race. No sampling.
- `temper/docs/plan/limits-heavy/figures/small-rows.md` holds all four, before and after in instructions, with the cause of each.
- The next cause's gain is under about 0.2 CPU s, or what's left is under *Found and not fixed*.
- A product change has its own focused test, and changes no behaviour.

## Inputs

- `temper/docs/plan/limits-heavy/inputs/request.md`.
- `temper/docs/plan/limits-heavy/notes/common.md`: how to measure, the figures' form and the rules, *Beside this plan* above all.
- `temper/docs/plan/limits-heavy/plan.md`, *Belonging*.
- `skein/docs/foundation/testing-strategy.md`, 8, *A test is a process* and *A slow test is understood, never hidden*.

## Owns and may change

Owns the two test functions in `turn.rs` and `reads.rs` and their fixtures, `setup_malformed_queries_are_bad_requests` and `expiry_and_native_paste_race_take_at_most_one_redirect`, and its figures. Other tests in those files aren't yours to reshape.

Readers: `fixtures::maximally_escaped_json_text` and `document_at_byte_bound` (`tests/fixtures/reads.rs`) and `occurrences` (`tests/fixtures/digest.rs`) are shared with other tests of `temper-client-tools`. Their signatures and what they return stay.

May change, at the size the cause needs:

- `temper-client-tools/src/jig/render/**` for a cause a profile puts there (`model-catalogue-2/configuring-learns` holds `temper-client-tools/src/jig/**`, so keep hunks small), and `skein-lib`'s bytes or JSON writing, where small and clearly theirs.
- `temper-service/src/**` for a cause a profile puts there.

Options, each decided:

- **Writing JSON escapes or building the input one byte at a time is the cost:** fix it where it's written. A person's `temper ctl --json` pays what the renderer pays. The printed bytes stay the same to the byte.
- **`occurrences` scanning the output is the cost:** use the library's own search, as the strategy says. It stays a count of the same thing.
- **`setup_malformed_queries_are_bad_requests` builds a service per row:** build it once per test, as the strategy says, and put each row to it, if a refused query leaves nothing behind for the next one. If one does, group the rows that need a fresh service and say which and why. Every row still reads its 400, and a failure names its row.
- **`service` or `SignInWorld::new` itself is what costs:** those fixtures are shared across their files, and `limits-fixtures` is repairing what `temper-service`'s tests build first. Fix what's this test's own (its loop, a repeated visit). Leave a shared fixture's general cost under *Found and not fixed*, unless the profile puts more than 0.2 CPU s of these two tests on it. Then make the smallest fix, and if a merge meets `limits-fixtures`, keep theirs.
- **The race's 54 schedules are linear in their world's work:** that's what it holds. What's left goes under *Found and not fixed*.

## Steps

1. `cargo nextest list -p temper-client-tools -p temper-service` reads each test's full name. Profile all four, with `--lib`.
2. Fix the causes, largest first, re-measuring after each.
3. Write the figures.
4. The implementer's text's end of a run: `cargo fmt`, old forms, `heavy cargo clippy -p temper-client-tools -p temper-service --all-targets -- -D warnings` with any other package you changed, and the focused tests of those packages and their direct dependents. Commit and push.

## Report

All four's before and after, the causes found and not fixed, and any product change with the test that holds it. Your implementation and check minutes apart.
