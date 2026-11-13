# squash-in: execution

2026-11-13. How [squash-in.md](squash-in.md) gets built. It designs nothing: each milestone writes its own design from the draft as it runs, and lands whole. Plans use the focused and fuzzy suites alone.

## 1. The milestones

| Milestone | Draft parts | Owner | Needs | Its walk | State |
|---|---|---|---|---|---|
| S1, a change's commit and main into a goal's branch: the squash commit's subject, body and trailers; a change bringing main's commits merged; a goal asking for main's tip, clean without an agent, conflicting as a resolution; a repair after landing from the goal's branch's tip | 2, 3, 4, 6, 7 | temper/migration | `merge-queue.md`'s M1 (`combine`) | after landing, on the staging deployment: a goal's changes squash into its branch and its branch merges into main; a catch-up asked and made | built by goal 2176; its walk to come |
| S2, the stock charters: the coordinator's, the designer's and the coding agent's words, as the draft gives them | 5, 7 | temper/migration | S1 | none: the charters' tests | built by goal 2231 |

## 2. Order

- **S1 after `merge-queue.md`'s M1,** which builds the squash it refines; it can start as soon as M1 lands.
- **S2 after S1:** the charters describe what S1 built. A project may change its own instructions sooner, since agents' commits on their own branches are already theirs.
