# ui-goal-asks: a goal a person started can ask them

A blocker on the happy path, from the user's journey on a fresh box at `d893a93cc1`. Chat T3 wrote a draft holding four open questions. The user pressed "Start goal…", way "Follow the away switch", Away off, which made goal T4. Then:

- `decided` answered, four calls out of four: "Ask: your person is here or this kind waits for them. Ask with message, as a question."
- `message`, form question, at targets 3 (its chat), 1 and 2: "Authority refused: task reference. Your brief's [Requested by ...] line names your requester; [This task: N] is you."
- `delegate`, a person task: "Authority refused: missing executor permission", at the task's `authority.delegation.kinds`, the project's `policy.ceiling.delegation.kinds` and `rules.ceiling.delegation.kinds`.
- `propose`: "Authority refused: tool family propose".
- It landed its draft and declared failure, with the questions in its result: "The draft landed but no code was written; I am ending this run because I can't get four open questions to you".

**Why, by reading:** the sheet's goal is a person's Create (`temper/docs/design/client/README.md`, 5.14), so its requester is the person. `authorize_tool` (`jig/crates/jig-domain-core/src/routing/mod.rs`, near 166) counts a requester only as `Party::Task(target)`, so a person's task has no target that reaches them. `temper/docs/design/domain/agent.md`'s "the goal asks its requester, the chat" holds only for a goal a chat calls. By reading, not run: "Always ask" and an agreed goal dead-end the same way on their first question, and so does a goal started from Goals.

## The ruling: the way jig and the design already have

A person task addressed to its requester (jig's `domain/tasks.md`, 8.5; `people.md`, section 8; `temper/docs/design/reference-code-production.md`, near 327). So charter 1's goal gets the person executor at those three places, narrowed as far as jig allows to a task addressed to its own requester, and its paragraph and `decided`'s answer (`temper-engine-smith/src/answers.rs`, near 476) say to use it when the requester is a person. The answer comes back as that task's result, and wakes the goal. Not a change to `message`, and not a chat as a sheet goal's requester, which wouldn't cover a goal started from Goals.

## What this plan makes true

1. **A goal started from the sheet or from Goals asks its person, whichever way it's working.** A goal a chat calls keeps asking its chat.
2. **The question shows under Needs you with its context and choices,** as a goal's question form draws them, with no extra step, and answering it there wakes the goal, carrying what they said.
3. **The refusal's hint is corrected:** "[Requested by person 1]" led the agent to try task 1. A requester that's a person says what to do.
4. **Failing tests first, at the lowest tier:** a person-started goal's ask is authorised and a stranger's task isn't; the answer reaches its goal. A story has a goal started from the sheet ask, its person answer under Needs you, and the goal carry on, written and compiled, not run, in the existing goals story if it can be a step of one: no new browser test.
5. `agent.md` and the help's goal pages say so.

## Limits

If this turns out to need a new design, stop with a `Question:`. Two or three tasks. Focused and fuzzy alone: no browser, walk or render check, by the user's rule that a plan is complete when it lands. The goal's charter is temper/migration's by seam 4, and it isn't running; its next plans replace that text, so keep the change small, and keep the words in one paragraph. `stock.rs` is also being moved by `model-catalogue-2`: whichever lands second catches up.

## Sources

- `jig/docs/design/domain/tasks.md`
- `temper/crates/temper-engine-smith/src/answers.rs`

## For every task of this plan

- **A plan is complete when it lands (the user, 11-06).** Walks, render checks, screenshots and throwaway browser drivers follow landing, outside the plan, and file findings back to fab: don't ask for them, or run them.
- **Only focused and fuzzy run in a plan (the user, 11-06).** Never `tools/tiers/run`, a browser, process or live tier, one of their stories, or a tier's CPU figure: "we run optimistically and repair if after-landing flags". A new story for those tiers is written and compiled (`cargo check --tests`), not run. This overrides anything below or in a spec. A look's render check by a throwaway driver isn't a tier.
- **Tests stay cheap (temper/tiers, 11-06):** a new case of one behaviour is a row of an existing table test, not a new test (skein's `docs/foundation/testing-strategy.md`, 8), and a story goes beside its siblings' fixture. temper's focused and fuzzy suites go to 30 and 120 CPU seconds in the gate, and a plan that adds cost gets rejected there.
- **Decide, don't ask.** The user is asleep and takes fabrig/ui's judgement. A stored format, a protocol or a public interface may change freely: no compatibility, no `Question:`; note it in `plan.md`. A small edit a task needs just outside its file list (a re-export, a `mod` line, a design line, a golden) is made and named in the report, not parked. A `Question:` is only for what truly blocks the task.
- **`temper.css`:** put new rules under this plan's marker comment, `/* ==== ui: AREA ==== */` (chats, needs, goals, changes, settings, welcome, phone), never at the end of the file, so that parallel plans merge cleanly. Remove replaced rules where they stand.
- **Goldens, prints and the gallery's `names.txt`/`tests.rs`:** they conflict across plans; on a catch-up, regenerate rather than hand-merge, and keep `names.txt` sorted as it is.
- **Reach for the mock, not the old page:** `temper/docs/design/client/greenfield/`; where old tests walk a part the mock removes, change them with it.
- **Generated files** (`docs/help/reference/*`, prints, goldens, counted baselines) that your change makes stale are yours to regenerate in the same task; don't park on them.
- **Recording flakes:** a re-record that stops on "a key mark caught a moving page" is retried up to three times, each on a fresh fake box; if all fail, widen the mark's wait in the recording driver and say so. A wire change makes journey recordings stale: re-record them in that task.
- **Help pages:** `docs/help/` pages a change grows past their bound are that task's to shorten, losing no facts, and never raise the bound.
- **A test's oracle or helper that your change's correct behaviour trips is yours to repair in the same task,** keeping what it checks for everything else, and named in the report; don't park on it.
- **Before calling a focused or fuzzy failure "outside the change",** run it once on the pristine base. If main passes and your branch fails, it's yours.
