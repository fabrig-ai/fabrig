# report-in-finish: a producer's report goes in finish, and is told so at once

From the first goal on a real deployment, 2026-11-05. A producer sent its report with `message` to its own change, which answered, as the design says, with a notice (`takes_no_messages` in `temper/crates/temper-engine-domain/src/route/forge_route.rs`):

```
Relays questions. Use read_task on this change; message target=this change, form=answer, question=number. Words do not answer.
```

The notice arrived between the `message` and the `finish`, so smith refused the `finish`: `Finish was not accepted because messages arrived` (`smith/crates/smith-domain-run/src/prompt.rs`). The run took another turn to read words meant for someone else. Up to three times in one goal, 1:19 to 2:18 each, at a coding model's prices.

Smith's rule is right, and stays: a run reads what arrived before it finishes. Don't special-case a notice in smith or jig.

## The change

1. **The producer's own charter says it.** In the stock producer's instructions (`temper/crates/temper-service/src/stock/`), where a run is told to report: the report goes in `finish`; `message` to its change is only the question form. The fewest words that hold, in the charter's own voice. If a stock bound or a figure (`tests/stock_brief.rs`) moves, say what by.
2. **The notice fits who asked.** When the sender is the change's own producing task and its form isn't a question, the words say what that run should do: its report goes in `finish`, and a question uses the question form. Words for a requester stay as they are, and `a_question_from_another_task_names_the_enabled_route` stays. Check by `temper/docs/design/domain/forge.md`, 8.1 and 8.11, and write the case in.
3. **If it's cheap, sooner than a notice:** a producer's plain words to its own change can be refused by the `message` call itself, with those same words, so no second message arrives and the `finish` isn't refused. Prefer this if the tool's admission already reads the sender and target and it stays small; otherwise leave it out and say why in the report. Don't widen a protocol for it: a change there is a `Question:`.

## Tests

A failing test first, in `temper/tests/engine/domain` (`forge_routes/questions.rs`): a producer messaging its change plain words sees the words for it, then a `finish` goes through once read.

## Size

One task, in temper. fabrig/ui has priority, and `repair-green` and `brief-failure` are working in `forge_route.rs`'s neighbours: keep hunks small, and expect catch-ups. Budgets and timeouts are the user's (skein's `docs/foundation/testing-strategy.md`, 8.1).

## Walk

`walk = false`: fabrig/qa sees it in the next goal's activity.
