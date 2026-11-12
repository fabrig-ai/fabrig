# claude-api: execution

2026-11-11. How [claude-api.md](claude-api.md) gets built. It designs nothing: each milestone writes its own design from the draft as it runs, and lands whole. Plans use the focused and fuzzy suites alone.

## 1. The milestones

| Milestone | Draft parts | Owner | Needs | Its walk | State |
|---|---|---|---|---|---|
| M1, step 1: `anthropic` offering keys beside Claude Code's program, a direct endpoint beside the program's, the route by source (account, source, host and endpoint chosen together), Claude's cache prices, a key's list read directly, the key's words on the pages and in the help, the dialect's tests for what current models refuse | 2 to 7, 8.1, 9 | temper/migration | none | after landing, on a real Claude Console key whose account was made after 2026-08-31: a chat turn and a delegated change on Opus 5.5 through `ctl`, on a deployment with no Claude Code (`claude-api.md`, section 9) | landed; its walk after landing waits for the user's key; in v0.1.0 (`v0.1.0.md`, section 5) |
| M2, step 2: the parts of 8.2 the user picks | 8.2 | temper/migration | M1, and the user's word on which parts | its own, once designed | not started; for v0.1.0 or after, by the user's word once M1 has landed |

## 2. Order and v0.1.0

- **M1 is v0.1.0's,** a part beside the ladder with no rung of its own (`v0.1.0.md`, sections 5, 8 and 9).
- **M2 waits for the user's word,** given once M1 has landed: which of 8.2's parts, and whether in v0.1.0. `v0.1.0.md`, section 10 holds the question until then.
- **M1's walk needs a real key:** a Claude Console organization with credits and a spend limit, which is the user's to provide.
