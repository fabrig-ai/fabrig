# ui-goal-asks

A goal a person started can ask them. The request: `inputs/request.md`. The ruling holds and needs no new design: a person task addressed to its requester, which jig and the design already have.

## Parts and tasks

- **asks** (`temper/docs/design/domain/agent.md`, 8.2): `ask-its-person`. The person executor in the stock authority at its three places, a chat's goal without it, charter 1's paragraph, `decided`'s Ask, the refusal's hint, the help's goal page, and the tests of who may be asked.
- **answers** (`temper/docs/design/client/README.md`, 3.5): `answer-under-needs`. A person task in words reads and answers as a question under Needs you, on Home and in a goal's band, and "Answer" sends on the terminal's inbox. Today's card drew only its title and spec, and neither surface could send its answer. Its story has a sheet goal ask, its person answer, and the goal carry on.

Each is one task with `needs = []`; they meet at `notes/contracts.md`.

## Critical path

Two tasks side by side: `answer-under-needs`, about 90 minutes, is the longer; `ask-its-person` about 60. The story needs no stock change, since its fixture names the kinds itself.

## What the cut decided

- **The narrowing.** jig's executor permission is a role's, so "its own requester" narrows to its requester's role: both ceilings name roles 0, 1 and 2, a role's authority its own alone, and a goal a chat calls none. A person of the same role can be addressed; the charter's words alone say otherwise. A permission naming the requester would be a new jig kind, so a new design: `agent.md`, section 10, holds it.
- **Not a root rail.** temper refusing a `delegate` by its address would have the root decide what jig promises (`jig/AGENTS.md`).
- **Agreed goals.** A person's `Create` never sets `asks`, so the owed-question rail, which only a `message` answers, holds no person-started goal. `shapes::set_agreed_goal_from` stays unsent; `agent.md`, section 10, says what sending it would need.
- **The card's kind and title.** It says "a question", and its line's TITLE is "Task N", its requester, since its own title is the question and the card reads no second task. In the inbox's filter it stays a person task.
- **The story's tier.** No browser story starts a goal, so it can't be a step of one: it is in the client world's `chat_draft.rs`, beside `a_goal_starts_from_a_document`, and runs in focused. No browser test is added or run.

No stored format, protocol or public interface changes.

## Overlaps not avoided

Whichever lands second catches up.

- `temper/crates/temper-service/src/stock.rs`, `tests/stock.rs`: `model-catalogue-2/stock-models` (named by the request), `limits-fixtures/service-fixture`, `ui-small-1/small-words`.
- `temper/crates/temper-service/src/stock/chat.txt`: `ui-small-1/small-words`.
- `temper/crates/temper-engine-smith/src/answers.rs`: `ui-box-slots/box-words`.
- `temper/tests/client/src/front.rs`, a few lines: `limits-fixtures/client-world`.
- `temper/docs/design/client/README.md`, `README.status.md`: `look-round-13/look-proof`, `ui-model-line/model-line`, among others, at other sections.
