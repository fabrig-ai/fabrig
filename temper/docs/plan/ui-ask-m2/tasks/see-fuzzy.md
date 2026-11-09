+++
id = "see-fuzzy"
part = "stories"
kind = "feature"
needs = ["see-stories"]
files = ["temper/tests/engine/domain/tests/suite/fuzzy/see.rs", "temper/tests/engine/domain/tests/suite/main.rs", "temper/tests/engine/domain/tests/suite/configuring/asking_person.rs", "temper/docs/design/domain/agent.status.md"]
effort = "high"
minutes = 120
+++

## Goal

The fuzzy test of addresses (`temper/docs/design/domain/agent.md`,
8.4.1, *Its tests*): seeded `see` calls over `see-stories`' two-project
world, so that no line a run may write makes an answer pass its bound,
lose its frame, change the store, or say anything of the other project.

Acceptance:

- `temper/tests/engine/domain/tests/suite/fuzzy/see.rs`, in the world's
  `fuzzy` module, one test,
  `seeded_addresses_stay_bounded_framed_and_in_their_project`, at most 64
  seeds, each making a handful of calls on `asking_person::world()` (built
  once a seed, or once for all seeds if its answers stay independent):
  - lines from the grammar of 8.4.1's table, every address form, numbers
    drawn from project 1's tasks, project 2's tasks, 0, 999999 and
    `u64::MAX` and past it, turns `A.T` from real and made-up pairs;
  - mutations: a word dropped, doubled, upper-cased or replaced by a
    random ASCII word; runs of spaces and tabs; the line cut to 512 and 513
    bytes; `more` followed by a token from a real answer's `more:` line,
    one with its offset changed to a seeded value, and a random word;
  - each answer: within the call's answer allowance; an error, or ending
    with its `see:` line or `more: TOKEN` then its `see:` line; never
    holding `ELSEWHERE` (project 2's titles); the store's rows unchanged
    after every call; the world never panics.
- The test's CPU seconds measured and reported; within the fuzzy suite's
  share (a few seconds); if more, fewer calls a seed before fewer seeds.
- `temper/docs/design/domain/agent.status.md`: the **Not yet** entry
  *Its tests*, "no fuzzy test of addresses exists", is deleted.

## Inputs

- `temper/docs/plan/ui-ask-m2/notes/common.md`.
- `temper/docs/design/domain/agent.md`, 8.4.1 (the table, *Bounded*, *The
  token*, *Then what to read next*, *Its tests*);
  `skein/docs/foundation/testing-strategy.md`, section 8 (seeds replay).
- `see-stories`' world and table,
  `temper/tests/engine/domain/tests/suite/configuring/asking_person.rs`;
  the world's other fuzzy tests, `tests/suite/fuzzy/*.rs`, for the seeded
  shape; `temper-engine-smith`'s `see.rs` (`token`, `untoken`).

## Owns and may change

- **Owns:** the new fuzzy file and its `mod` line in `main.rs`'s `fuzzy`
  module; the status entry.
- **Reader files:** `asking_person.rs`, to make its world builder
  reachable from the fuzzy module (`pub(crate)`), nothing else.
- **Options, pre-approved:** a failing seed is a fault of the code under
  test: fix it in `see`'s rendering (`temper-engine-smith`'s `see.rs` and
  `see/`) or routing (`temper-engine-domain`'s `route/see.rs` and
  `route/see/`), and add its line as a row of `see-stories`' table; a fault
  anywhere else is a `Question:` naming the seed and the line.

## Steps

1. The generator and one seed; then 64.
2. Break the bound once (render a part past 6,144 bytes) to see the test
   fail on it, and restore.
3. The status entry; measure.
4. `cargo fmt`; `heavy cargo clippy -p temper-engine-domain-world
   --all-targets -- -D warnings`; `heavy cargo nextest run -p
   temper-engine-domain-world --profile fuzzy --test suite -E
   'test(/fuzzy::see/)'`. Commit and push.

## Report

As the implementer's text asks; the seeds' count, the calls made, and the
test's CPU time.
