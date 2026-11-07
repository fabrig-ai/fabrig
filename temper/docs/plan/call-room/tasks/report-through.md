+++
id = "report-through"
part = "report"
kind = "feature"
needs = []
files = ["temper/crates/temper-engine-domain-forge/src/domain.rs", "temper/crates/temper-engine-domain-forge/src/result_tests.rs", "temper/crates/temper-engine-domain-forge/src/tests/change.rs", "temper/crates/temper-engine-domain/src/route/forge_route.rs", "temper/crates/temper-engine-domain/src/link.rs", "temper/tests/engine/domain/tests/suite/forge_routes/result.rs", "temper/tests/engine/domain/tests/suite/forge_routes/change_tool.rs", "temper/tests/engine/domain/src/forge_routes.rs"]
effort = "high"
minutes = 120
+++

# A report past 3,584 bytes reaches its requester, and a batch of 32 passes

## Goal

Once a report may pass 3,584 bytes, the path below it has to hold: a landing's result at its bound must still reach a live requester, and a large spec must not push a pull request's body past what a forge takes. This task holds a report through to its requester in a world below the real loop, closes those two holes, and holds a batch of 32 and one more.

Acceptance:

- `a_landed_change_carries_its_producers_report` is a table with rows past 3,584 bytes, at least at a report of 3,585 bytes with a result bound of 10,660, and one at the bound. Each row holds `row.report` whole in the durable row, and the requester's message holding the report whole where it fits, or cut with its `[N bytes cut]` marker.
- A report past the message's room stays whole: a row of `task_report_selectors_render_complete_retained_continuations` (`temper/tests/engine/domain/tests/suite/host_calls.rs`) reads a report past 3,584 bytes back whole by `read_task` with `field=result`, in pieces.
- A landing's result at its bound, with the label and sender its requester reads it under, is within `tasks.result_bytes`: a row asserts it on `link::message`'s words and sender, against the words bound (`temper/crates/temper-engine-domain/src/link.rs`, `inbox_words`; jig's hub refuses more as `MessageRefusal::TooLarge`).
- A pull request's body is at most 65,536 bytes whatever `forge.client.op_bytes` allows, cut with its existing line, and the change's row keeps its words whole: a row of the tests beside `projected_words` (`temper/crates/temper-engine-domain-forge/src/tests/change.rs`) at an `op_bytes` of 131,070 and a body of 131,070.
- In `change_tool.rs`, a `change` of 32 changes at an `authority.batch` of 32 makes one batch, and 33 are refused for the batch, as rows of one test.
- `a_report_at_the_bodys_bound_is_kept_whole_at_the_stocks_result_bound` no longer names a bound that has gone: rename it for what it holds, and it holds the largest report that stays whole beside its evidence, and one byte more cut with its marker.
- Focused and fuzzy tests of each changed package pass.

## Inputs

- `temper/docs/plan/call-room/notes/common.md`; `temper/docs/plan/call-room/notes/contracts.md`, 5.
- `temper/docs/design/domain/agent.md`, 4.4; `temper/docs/design/domain/forge.md`, 8.1.
- `temper/docs/design/protocol/configuration.md`, 13.8, *A batch*.

## Owns and may change

Owns the header's files, and one row in `temper/tests/engine/domain/tests/suite/host_calls.rs`. `StoryLimits` (`temper/tests/engine/domain/src/forge_routes.rs`) may gain the fields a row needs, each defaulting to what the world sets today. `temper-engine-domain` and `forge_route.rs` are written by other plans too: keep the edit there to the bound's one function, so a merge is plain.

Options, each pre-approved:

- **The label's room.** `change_result_bound` (`forge_route.rs`) gives the landing's result its contract's words. Take a constant room from it for the longest label and sender a result can carry: name it in `link.rs`, beside `inbox_words`, and compute it from what those functions write, a task number at its longest included. If a smaller edit does the same, take it.
- **A change's direct result to a run** (a `delegate` with a `change` contract, not the `change` tool) can still reach a live requester at its words' bound. It's rare and not on this path: leave it, and list it under `## Left`.
- **A world's limits for the batch's rows:** `plan_world`'s `limits.authority.batch` is 2 by literal (`temper/tests/engine/domain/src/forge_routes.rs`). Set the figures that row needs through `StoryLimits`: `authority.batch`, `tasks.batch`, `tasks.delegates`, its task room (a batch of n needs 2n available), `tasks.batch_bytes`. If that world can't reach 32 cheaply, use `stock_world` in `temper/tests/engine/domain/tests/suite/fabrig_scale.rs`, whose limits are `standard`'s, and own that one test there.
- **`forge_route.rs`'s reading of `row.report`:** leave its format as it stands.
- **`result_bytes` above `forge.client.op_bytes` in a row's world:** `temper-engine-domain/src/limits.rs` refuses it. Set both.

## Steps

The body's bound, in `change_write`, cited to `temper/docs/design/domain/forge.md`, 8.1:

```rust
/// What a forge takes in a pull request's body, in bytes at most (a forge counts characters).
const PULL_BODY_BYTES: usize = 65_536;
let body = crate::text::fit(&row.body, room.saturating_sub(title.len()).min(PULL_BODY_BYTES), marker);
```

The table, keeping the test's scene and its assertions on `row.report` and on the requester's message:

```rust
for (report, result_bytes) in [(/* the 36-byte report */, 512), (3_585, 10_660), (/* at the bound */, 10_660)] { .. }
```

A row at the reference's figure (131,070) is worth having only if it's cheap: run it once and keep it if it adds under 50 ms of CPU, saying what it costs in a comment. No row reaches a real host. Each pinned rule breaks once by hand: the label's room removed fails the row at the bound; the body's bound removed fails the pull request's row. Report what you saw.

`temper-engine-smith`'s decoding of a large `change` is `reference-room`'s; `jig-room` moves the contract's `body`, and this world hands typed results, so this task needs neither.

`cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-engine-domain-forge -p temper-engine-domain -p temper-engine-domain-world --all-targets -- -D warnings`; `heavy cargo nextest run -p temper-engine-domain-forge -p temper-engine-domain -p temper-engine-domain-world -p temper-protocol-store`, and with `--profile fuzzy` for the packages you changed; `heavy cargo check -p temper-service --tests`. Commit, push.

## Report

What the requester sees at each row: whole, or cut and how much. The label's room and how you sized it. What the batch's rows needed of their world, and their cost. Anything you left, under `## Left`.
