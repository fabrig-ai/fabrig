# hearth-speed: the plan

hearth's focused suite under its 30 CPU s, aiming for 27 on the queue's figure, with every assertion and case kept. The design is `hearth/docs/design/domain/testing.md`, 9.1, which doesn't change. The standing rules, the aim, the base commit and the method are in `notes/common.md`.

## Parts and tasks

| Part | Tasks | Needs |
|---|---|---|
| `before` | `figures-before` | nothing |
| `profiles` | `dev-profiles` | nothing |
| `git-setup` | `git-setup` | nothing |

The rest of the plan follows in the next push. The critical path as it stands is `dev-profiles`, about 60 implementer-minutes.

## What was decided

- **No machine here can profile,** because a call builds nothing, so the first task does. The other two don't wait for it: each measures itself, on its own slot.
- **The aim is a ratio.** No task can run the queue, and a builder reads higher than the queue (41.7 against 31.7). So a task aims at 85% of its own machine's figure at the base commit, which is 27 / 31.7.
- **Every "before" is at one pinned commit,** the trunk's base, so that it doesn't depend on which task merged first.

## Overlaps

No draft beside this one lists a file this plan changes so far. `debt-26/hearth-spot` changes `hearth-protocol-builder`'s source, which a profile line builds differently but no task here edits.
