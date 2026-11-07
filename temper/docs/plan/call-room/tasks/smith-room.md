+++
id = "smith-room"
part = "agent"
kind = "feature"
needs = []
files = ["smith/crates/smith-domain-run/src/limits.rs", "smith/crates/smith-agent-service/src/profile/tests.rs"]
effort = "high"
minutes = 75
+++

# Smith's charter allowance and outcome follow the payload, and T is measured

## Goal

A run's charter allowance and its outcome are fixed at 65,536 and 4,096 today, so a whole spec is cut and a report stops at 3,584 whatever the call holds. Both follow `tool_payload`. This task also measures what that and a larger output do to the agent's largest turn, T, against smith's ceiling of 33,554,432: the reference's figure is picked by this measurement.

Acceptance:

- `run_bytes` is the larger of 65,536 and twice `tool_payload`; `outcome_bytes` the larger of 4,096 and `tool_payload` (`notes/contracts.md`, 2).
- A table test in `smith/crates/smith-agent-service/src/profile/tests.rs` derives at each row's `tool_payload` and `output`, and pins each row's T, or its refusal, measured: 10,662 and 8,192; 63,740 and 48,000; 131,072 and 98,500. Each derived T is within the ceiling, or the row names the check that refuses.
- `temper init`'s T stays 16,803,810: `every_stock_charter_admits_its_whole_maximal_brief` (`temper-service`) and `the_shipped_agent_fits_standards_page_with_derived_carriers` (`temper-real-loop-world`) pass unedited.
- Focused and fuzzy tests of each changed package and of the readers below pass.

## Inputs

- `temper/docs/plan/call-room/notes/common.md`; `temper/docs/plan/call-room/notes/contracts.md`, 1 and 2.
- `smith/docs/design/protocol/limits.md`, 3.10 and section 4 (`turn-in-channel`).
- `temper/docs/design/protocol/configuration.md`, 13.8.

## Owns and may change

Owns the header's files.

Readers (survey of `run_bytes` and `outcome_bytes`): the fixtures that set them by literal keep their literals; these derive from `standard()` (`tool_payload` 40,960, so `run_bytes` 81,920, `outcome_bytes` 40,960), and may move: `smith/crates/smith-agent-service/tests/suite/sink_memory.rs` (a line ceiling of 19,855,104 and heap ceilings of 31,000,000 and 5,000,000), `smith/tests/agent/tests/suite/memory_returns.rs`, `smith/crates/smith-local-service/src/tests.rs`. You may change those figures, and any other pin in `smith/**` that only follows the derivation.

Options, each pre-approved:

- **A heap ceiling or line ceiling at `standard()` moves:** measure before and after, set the ceiling to what the derivation now implies, and say why it moved in the report. If it moves by more than the derivation explains, stop and report it as a `Question:`.
- **A pin in `smith-local-*` moves:** the local product takes the larger charter and outcome with `standard()`. Regenerate the pin and name it. Its own 65,536 of local text (`smith-local-shell/src/settings/resolve/local.rs`) stays.
- **The row at 131,072 and 98,500 is refused:** pin the refusal in its row, naming the check; that is the measurement, and the task is done. `reference-room` then takes 48,000. If the row at 63,740 and 48,000 is refused too, report `Question:` with both refusals and T's terms.
- **The two-model endpoints:** the existing test gives its second endpoint a window of 177,523, which an output of 98,500 refuses on `fresh-window`. Init's demo has a window of 1,048,576 today (`temper/crates/temper-init/template/agent.json`): take that for every row.
- **`HostInput::CAPACITY`** isn't yours: `large-argument` moves it. Derivation doesn't read it.

## Steps

In `smith/crates/smith-domain-run/src/limits.rs`, `derive`:

```rust
run_bytes: u64::from(inputs.tool_payload).saturating_mul(2).max(1 << 16),
outcome_bytes: u64::from(inputs.tool_payload).max(4096),
```

Say on `Limits`' two fields what they follow (`smith/docs/design/protocol/limits.md`, 3.10).

`forty_eight_calls_per_response_derive_with_two_models` becomes the table; keep its name's rule, its `expect` and its printed line, and take the rows from `notes/contracts.md`, 1:

```rust
for (tool_payload, output, turn) in [
    (10_662, 8_192, /* measured */),
    (63_740, 48_000, /* measured */),
    (131_072, 98_500, /* measured */),
] { /* selected.declared.tool_payload, each endpoint's limits.output; derive; assert the row */ }
```

Each row also asserts `limits.domain.run.run_bytes` and `outcome_bytes`. This test is arithmetic: no cost to state beyond that. Break each formula once by hand and see the row fail; report it.

This changes what a shared item means, so run its readers' tests: `heavy cargo nextest run -p smith-domain-run -p smith-agent-service -p smith-local-service -p smith-host-domain -p smith-agent-world -p smith-agent-process-world -p jig-protocol-worker-agents -p temper-service -p temper-real-loop-world`. `cargo fmt`; `python3 -I -B smith/scripts/old_forms.py`; `heavy cargo clippy -p smith-domain-run -p smith-agent-service --all-targets -- -D warnings`, naming any other package you changed. Commit, push.

## Report

T at each row, what moved it and what each term contributed if you read them; which row, if any, was refused and by what. Every pin you moved, old and new, and why. That `temper init`'s T did not move.
