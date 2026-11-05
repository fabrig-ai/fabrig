# producer-failure: a producer's failure holds at producing, and a question outlives no one's finish

Priority: these stopped a comparison run on a real deployment, 2026-11-05, and it waits on them. Reproduce each first in the lowest tier. Causes below are from reading, not reproduced: say so if they're wrong.

## 1. A delivered producer that finishes `failure` opens a pull request

A producer delivered, then called `finish` with form `failure`. The change went to `opening`, not held at producing, so partial work would open as a pull request. Here the title was empty: Forgejo answered 422 `[Title]: Required`, and the change held on `open pull request: empty operation`.

From reading: in `producing()` in `temper/crates/temper-engine-domain-forge-change/src/domain.rs`, `Status::Failed` is checked only in `Early::Started` and `Early::Spent`, and `keep_produced_title` (`route.rs`) fires only for `Done(Change)`. `temper/docs/design/domain/forge.md`, 8.4 and `temper/docs/help/change.md` say a failure holds at producing.

**The change:** a failed producer holds at producing, whatever it delivered, with its failure's words on the hold, by 8.4. Its pushed work stays on its branch. No pull request opens. Also check the same shape in a repair and a resolution, and in `opening` itself: an empty title should never reach a forge, whichever route gets it there. A failing test first, in the forge world: `deliver`, then `finish` failure.

## 2. An asker's `finish` drops its open question

A producer asked its change a question, a foreman was started, and then the producer called `finish`. The foreman was cancelled, the question reached no one, and the requester's inbox said only `escalation`.

By `forge.md`, 8.11 an agent that asked is kept by its own question, and its run parks until the answer comes (jig's `jig/docs/design/domain/tasks.md`, 5.7). So a run that has asked has no reason to `finish` until it's been answered.

**The change, by recommendation:** a `finish` from a task with its own question still open is refused, with words saying so and what to do: `Your question is still open: call wait; the answer reaches this run.` It's the same shape as smith's refusal when messages arrive, and `report-in-finish`'s refusal at `message`. Settle by `tasks.md` whether this belongs in jig, as a rule of any task with an open question, or in temper's forge, and name the case in the design. If the design has a reason that such a finish must end the task (a task's own deadline, a cancel, an account's end), keep those: only the agent's own `finish` is refused. If this rule is wrong or much larger than it reads, raise a `Question:` at once with your recommendation, and keep item 1 moving: **nothing waits on this but the rerun.** Whichever way it's settled, no question may silently reach no one: a question left when its asker ended says so to whoever was waiting.

A failing test first: a producer asks, then finishes: the finish is refused; it waits; the answer resumes it; its finish goes through.

## 3. The stock words on how a run ends after asking

Stock coding instructions (`temper/crates/temper-service/src/stock/`) should say how to end after asking: `wait`, not `finish`. A run read "end your run" as `finish` with `failure`. Fix the words where they say so, the fewest that hold; if `tests/stock_brief.rs` moves, say what by.

## Size

Two tasks, perhaps three, in temper and maybe jig (hearth's manifests name no jig crate). `repair-green` is in `temper-engine-domain-forge-change` too; expect catch-ups. fabrig/ui has priority: leave its files alone. No protocol or store change beyond what's named; formats change in place before v0.1.0. Budgets and timeouts are the user's (skein's `docs/foundation/testing-strategy.md`, 8.1).

## Walk

`walk = false`: the comparison's rerun is the proof.
