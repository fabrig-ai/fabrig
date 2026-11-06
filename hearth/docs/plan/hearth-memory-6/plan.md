# hearth-memory-6: the plan

Round 6, the last of hearth's move to the dynamic memory model: the service, and the end of D001 and D004. The design is `hearth/docs/design/domain/render.md`, 8.4 and 8.5, which this cut wrote first. The shapes that cross tasks are in `notes/contracts.md`.

This file grows with each push. What is below is what has been released.

## Parts and tasks

| Part | Tasks | Needs |
|---|---|---|
| `before` | `figures-before` | nothing |
| `lists` | `hand-lists` | nothing |
| `stage` | `stage` | nothing |
| `orders` | `store-step`, and `stuck-client` to come | nothing |

Still to be released: `queues` (`uncap`, `constants`), `resumed` (`resumed-order`), `close` and `after` (`figures-after`).
