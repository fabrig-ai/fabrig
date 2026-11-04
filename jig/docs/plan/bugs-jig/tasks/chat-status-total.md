+++
id = "chat-status-total"
part = "chat-status"
kind = "feature"
needs = []
files = ["jig/crates/jig-client/src/facts.rs", "jig/crates/jig-client/src/tests/facts.rs", "jig/crates/jig-client-view/src/words/words/status.rs", "jig/docs/design/client/domain/client.status.md"]
effort = "medium"
minutes = 30
+++

# chat_status reads any chip

## Goal

`jig_client::facts::chat_status` `.expect`s a chip's `ended_at` and `retry`, both optional on the
wire and in the contract. The link refuses such a record (`chip_phase_fields`,
`jig/crates/jig-client-protocol-link/src/translate/facts_part1.rs`, and its test in
`src/tests/documents.rs`), so this abort is reached only past the link; but `chat_status` is a
public helper over public contract types, and it must not abort on them.

Acceptance:

- `ChatStatus::Retrying`'s `notice` is `Option<RetryNotice>`.
- An ended chip with no `ended_at` reads `ChatStatus::Ended { at: details.phase_since }`.
- A chip backing off with no `retry` reads `ChatStatus::Retrying { until, notice: None }`, and the
  view says "Retrying at" alone for it.
- The new test fails on the old code by the two `expect`s, and passes. `facts.rs` holds neither.
- The tests of `jig-client`, `jig-client-view` and `jig-client-test-view` stay green, and
  `temper-client-view` and `temper-client-tools` still compile with their tests.

## Inputs

- `jig/docs/plan/bugs-jig/notes/common.md`.
- `jig/docs/design/client/domain/client.md`, 19 (the status table and the paragraph after it);
  `jig/docs/design/protocol/client.md`, 3 ("Documents are decoded whole").
- The code: `jig/crates/jig-client/src/facts.rs`, `jig/crates/jig-client/src/tests/facts.rs` and
  `facts_chip` in `tests/fixtures.rs`; `jig/crates/jig-client-view/src/words/words/status.rs`.

## Owns and may change

- `jig/crates/jig-client/src/facts.rs`: `ChatStatus::Retrying`, the two reads.
- `jig/crates/jig-client/src/tests/facts.rs`: the `backing off` row's expected value, one new test.
- `jig/crates/jig-client-view/src/words/words/status.rs`: only the `Retrying` arm, which matches
  the notice when there is one. Plan `debt-3` works across `jig-client-view/src`: touch nothing
  else there.
- `jig/docs/design/client/domain/client.status.md`: this plan's **Not yet** entry for 19.
- temper's readers match `ChatStatus::Retrying { .. }` and need no edit. If one does not compile,
  fix its pattern and say so.
- Link behaviour is settled: `chip_phase_fields` keeps refusing these records. Leave it.

## Steps

1. The test in `tests/facts.rs`, `a_chip_without_its_phase_facts_still_reads`:
   - `facts_chip(ChipPhase::Ended, None)`, then clear `ended_at` in its details: assert
     `ChatStatus::Ended { at: PHASE_SINCE }`;
   - `facts_chip(ChipPhase::Active(RunStage::BackingOff { until: RETRY_AT }), None)`, then clear
     `retry`: assert `ChatStatus::Retrying { until: RETRY_AT, notice: None }`.
   See it fail on each `expect`, then make the change, with the `backing off` row's expected
   value now `Some(RETRY_NOTICE)`.
2. `heavy cargo nextest run -p jig-client --lib -E 'test(=tests::facts::a_chip_without_its_phase_facts_still_reads)'`,
   then `heavy cargo nextest run -p jig-client -p jig-client-view -p jig-client-test-view`.
3. `heavy cargo check -p temper-client-view -p temper-client-tools --tests`.
4. `cargo fmt`; old forms; `heavy cargo clippy -p jig-client -p jig-client-view --all-targets -- -D warnings`;
   the citations check; commit and push.

## Report

As the implementer's text asks, with how the test failed on the old code.
