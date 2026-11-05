# debt-17: the seventeenth debt round

Ten small items (`inputs/request.md`), one part each. This file grows with each push. Every item was re-measured at this
plan's base and is still open. D003 reads 86, not the item's 97.

## Parts and tasks

| Item | Part | Task | Needs | Effort | Min | What it does |
|---|---|---|---|---|---:|---|
| hearth D008 | `hearth-path` | `hearth-path` | | medium | 15 | the fixtures move into `suite/memory/` |
| skein D034 | `codec-once` | `codec-once` | | medium | 15 | `build.rs`, `names.rs`, the `#[path]` go |
| skein D044 | `terminal-namespace` | `terminal-namespace` | | medium | 20 | namespace 4 of `Token::tagged`, the same bit |

Still to come in later pushes: skein D003, D005, D006, D008, D019, D026 and jig D027, each surveyed first.

## Critical path

Nothing here waits on anything yet: 20 minutes.

## Decisions

- **hearth D008 goes now.** The item says it waits for hearth's memory migration, which holds both files; the request sets
  that aside unless a `#[path]` needs `hearth-service-builder/src/lib.rs` or a `MAX_OUT` form. Neither does.
- **The terminal keeps bit 62, as namespace 4.** Every raw token stays the same. `temper-client-shell-terminal` keeps its own
  hand tag at bit 61, and `temper-box`, `temper-client-shell-cli` and smith keep theirs: the item's measure is skein's, and
  this plan stays out of temper's client work.
- **smith D003 stops waiting on skein D044.** `Token::tagged` is the lane the smith item asked for; its text says so, and it
  stays open.

## Overlaps

`hearth-memory-3`, a cut in flight beside this plan, may touch the config memory suite. `hearth-path` goes first and is a
small move, so whichever lands second merges it.
