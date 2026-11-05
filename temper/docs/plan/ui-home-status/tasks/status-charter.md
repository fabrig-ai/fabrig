+++
id = "status-charter"
part = "status-charter"
kind = "feature"
needs = []
files = [
  "temper/crates/temper-service/src/stock.rs",
  "temper/crates/temper-service/src/stock/status.txt",
  "temper/crates/temper-service/src/tests/stock.rs",
  "temper/crates/temper-service/src/tests/stock_brief.rs",
  "temper/docs/help/charter.md",
  "temper/docs/help/settings.md",
]
effort = "medium"
minutes = 60
+++

# Stock charter 9, the status agent's

## Goal

The stock holds the status agent's charter: `temper/docs/design/protocol/configuration.md`, 13.4, its row. Nothing runs it yet: `status-run` does.

Acceptance:

- `stock/status.txt` holds, exactly: "You write the few lines at the top of a project's home page, from the facts in your task. Write one sentence on what landed, starting with the count. Then a blank line, then at most three plain sentences on what stands out: what finished, what is stuck and why. Say only what the facts say. No greetings, no lists, no advice."
- `checked()` has it, number 9: charter 1's models, 4 turns, 300 s, a reserve of 1 turn and 60 s, `inspect`, `modify`, `shell` and `agents` false, `tools` `Tools(0)`.
- `ceiling`'s `delegation.kinds` gains `authority::Executor::Charter(9)`, so the forge's procedure can delegate it, and `tasks.charters` and `authority.executors` floors hold in every configuration the tests start.
- `temper/crates/temper-service/src/tests/stock_brief.rs`'s host-part figures gain charter 9's row, and `every_stock_charter_admits_its_whole_maximal_brief` passes.
- `protocol/configuration.status.md` keeps its second *Not yet* entry for charter 9 and loses `13.4: charter 9, the status agent's: plan ui-home-status.` once this is in.

## Inputs

- `temper/docs/plan/ui-home-status/notes/common.md`.
- `temper/docs/design/protocol/configuration.md`, 13.4; `temper/docs/design/domain/forge.md`, 5.7.
- `temper/crates/temper-service/src/stock.rs`: `foreman`, the model; the `tools` match in `charter`.

## Owns and may change

Owns, as `files` lists them. May also change `temper/docs/design/protocol/configuration.status.md`, as above.

Readers, each one number and any comment that counts them ("Five slots hold the stock charters"): `tasks.charters`, 5 to 6, in `temper/crates/temper/tests/support/configuration.rs`, `forgejo.rs`, `github_land.rs`, `case.rs`, `github/engine.json`, and `temper/crates/temper/tests/fixtures/cluster/engine.json`; `authority.executors`, 8 to 9, in `temper/crates/temper/tests/suite/configuration.rs`, `support/configuration.rs` and both `engine.json`; `temper/tests/engine/domain/tests/suite/configuring/fixture.rs`'s stock list; `temper-engine-smith/src/tests/settings.rs`, if it pins the stock.

Options, each pre-approved:

- **Charter 9 taken** by a reference's or a test's settings where you look: leave theirs. A test's own configuration overrides the stock by its number, which is what `stock.rs`'s tests say. Report it.
- **The host part's figures** come from running the test, which prints the row it wants: copy what it prints, not by hand. If `tools_for` declares a host tool whatever `Tools(0)` says, that is the design's status, `protocol/configuration.status.md`, 3; leave `tools_for`.
- **The reference's rules** (`temper/crates/temper/src/tests/reference_settings.rs`, 13.9): leave them; the status line says so.
- **`stock.rs:174`'s test,** which says charters 5 and 7 aren't stock, stays as it is.

## Steps

1. **`stock.rs`:** `STATUS_TOOLS`, `status(llm)` beside `foreman`, the array, the match, `ceiling`.
2. **Tests,** `tests/stock.rs`: `the_status_charter_is_small_and_reaches_nothing`, its numbers and flags; `the_status_charters_words_are_the_requests`, against the file; `the_rules_delegate_to_the_status_charter`.
3. **`docs/help`:** `charter.md` and `settings.md`'s stock lists; regenerate `docs/help/reference/*` if a test says they moved.
4. **Check,** the implementer's text, with `temper-engine-domain-world` and the `temper` package's tests that start a stock configuration (`heavy cargo check -p temper --tests`, then those that name `charters`), then push.

## Report

As `notes/common.md`, Your report, with the figures.
