+++
id = "trial-charter"
part = "trial-charter"
kind = "feature"
needs = []
files = [
  "temper/crates/temper-service/src/stock.rs",
  "temper/crates/temper-service/src/stock/trial.txt",
  "temper/crates/temper-service/src/tests/stock.rs",
  "temper/crates/temper-service/src/tests/stock_brief.rs",
]
effort = "medium"
minutes = 45
+++

# Charter 10, a trial's, in the stock

## Goal

The stock holds a charter of its own for a trial: no tools, one working turn, asked for one word, charter 1's
models and hosts. Nothing uses it yet: `trial-root` makes its tasks.

Acceptance:

- `stock::defaults` holds charter 10, as `temper/docs/design/protocol/configuration.md`, 13.4, has it: its
  instructions `stock/trial.txt`, exactly `Say one word and nothing else. Use no tools.`; `turns` 2, `time` 120 s,
  `reserve_turns` 1, `reserve_time` 60 s; `resume` false; `inspect`, `modify`, `shell`, `agents` false; `tools`
  `Tools(0)`; every other field as charter 1's, its models among them.
- It is in no delegation: `ceiling`'s and the roles' `delegation.kinds` don't name `Executor::Charter(10)`.
- jig's check passes the stock, and a stock start with its floors holds.
- Its offered host part fits, by `stock_brief.rs`'s table, with its figure printed by the test as the others' are.

## Inputs

- `temper/docs/plan/model-check/notes/common.md`.
- `temper/docs/design/protocol/configuration.md`, 13.4: its row and *Each charter's host families*.
- `temper/docs/design/domain/trial.md`, 2.

## Owns and may change

Owns, as `files` lists them.

May also change, each only for the new charter and named in your report:

- `temper/crates/temper-service/src/tests/checks.rs`: its two expected refusal strings gain
  `charter 10.model: ...`, as `charter 9`'s lines have it.
- `temper/crates/temper/tests/suite/creating.rs`: the count `6` becomes `7`, and `offered`'s expectation if it reads
  the stock's charters.
- `temper/crates/temper/src/tests/reference_settings.rs`: a count that follows.
- `temper/tests/**`: an expected list of the stock's charters, the same one-line way.
- `temper/docs/design/domain/trial.status.md`: from **Not yet**, remove "no charter 10", and only that.

Options, each pre-approved:

- **A floor refuses:** `floors` names the limit, `tasks.charters` under a profile that holds fewer than 7. Raise that
  count to 7 in `jig/crates/jig-protocol-config/src/profile/*.rs` or `temper/crates/temper-init/template/*`, where the
  limit is set, and regenerate what its tool regenerates. Say so.
- **`offered` follows the policy's delegation rather than the stock's charters,** so that charter 10 isn't offered:
  keep it not offered, and change `creating.rs`'s assertion to "every charter but 10". If it follows the stock's
  charters, and a person could be offered it: keep the assertion, and report it under `## Left`; `person-task` and
  `trial-root` don't depend on it, and the design's *In no delegation* still holds at `Create`.
- **`host_part`'s figures move** for all charters: don't. A new charter moves only its own row; if another moves,
  it's a question.

## Steps

```rust
// stock.rs
const TRIAL_TOOLS: jig_domain_tasks::Tools = jig_domain_tasks::Tools(0);
// `charter`'s match gains `10 => TRIAL_TOOLS`.
fn trial(llm: &jig_protocol_config::Llm) -> Charter {
    let mut value = charter(10, 2, Duration::from_secs(120), include_bytes!("stock/trial.txt"), llm);
    value.policy.inspect = false;
    value.policy.modify = false;
    value.policy.agents = false;
    value.policy.resume = false;
    value
}
```

`checked`'s array gains `trial(llm)` after `status(llm)`, so that `charter.spend = run` holds for it.

Tests in `tests/stock.rs`, failing first, beside `the_status_charter_is_small_and_reaches_nothing`:

- `the_trial_charter_is_one_working_turn_and_reaches_nothing`: the whole value against charter 1's, as that test does.
- `the_trial_charters_words_ask_for_one_word`: the words, and `stock/trial.txt`'s bytes in the charter.
- `no_policy_delegates_to_the_trial_charter`, the reverse of `the_rules_delegate_to_the_status_charter`.

In `tests/stock_brief.rs`, a row `(10, false, ...)` in both tables, its workspace `None` as charter 9's: `number == 9`
in `workspace` becomes charters 9 and 10.

The checks: fmt, `old_forms.py` for `temper`, clippy on `temper-service`, then
`heavy cargo nextest run -p temper-service --lib`, and `-p temper --test suite -E 'test(creating)'`. Citations.
Push.

## Report

As the implementer's text asks: each test's bite, and which options you took.
