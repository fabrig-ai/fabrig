# pair: execution

2026-11-27. How [pair.md](pair.md) gets built. It designs nothing: the goal writes its own design from the draft as it runs, and lands whole, with its own tests. Plans use the focused and fuzzy suites alone, world tests among them; walks come after landing, on the user's word. Main stays redeployable to dev.fabrig.ai at every landing: nothing changes for work that names no connected producer.

## 1. The goals

| Goal | Draft parts | Needs | State |
|---|---|---|---|
| P1, a connected implementer: smith's Finite run in conversation, its replies parking it, `finish_requested` asking it to finish; a change's producer task with a conversation, its requester's words waking it; a save when it parks with work undelivered; `temper ctl task message --finish`; the stock `pair` charter with a model for sub-agents; a change recipe's producer with `conversation`, and the gallery's Pair entry released; no page changed | 1 to 5 | [ways.execution.md](ways.execution.md), W4 | not started |

## 2. Order

- On the user's approval, after W4 has landed. The pages, "Land this" and the diff, are the UX pass's (W14's, in ways' execution).
