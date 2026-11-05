# producer-failure: the plan

From `inputs/request.md`. This file grows with each push. `walk = false`: the
comparison's rerun is the proof.

## Parts and tasks

| Task | Part | Needs | Effort, minutes | What it lands |
|---|---|---|---|---|
| `failed-producer-holds` | `failed-producer` | | high, 60 | item 1: the hold, the fresh producer on release, the title default |
| `stock-after-asking` | `stock-words` | | medium, 15 | item 3: one sentence in `coding.txt` |

Item 2 (an asker's `finish`) is still being cut.

## Critical path

Each task stands alone, so the path is `failed-producer-holds`.

## Overlaps not avoided

- `repair-green` changes `temper-engine-domain-forge-change/src/domain.rs` and
  `forge.status.md`. Item 1 has to change the same file; its edits stay in `producing`
  and `released_state`.
- `ui-chat-draft`'s `chat-stock` changed `temper-service/src/tests/stock.rs` and
  `stock_brief.rs`. `stock-after-asking` adds one test there.
