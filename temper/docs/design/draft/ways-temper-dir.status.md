# ways-temper-dir: status

2026-11-28. Where the design lags or departs from [ways-temper-dir.md](ways-temper-dir.md).

- **Design lags:**
  - 1: `../protocol/temper-dir.md`, 3.1: `ways.toml` holds no `[defaults]` ([ways.md](ways.md), 5.3).
  - 4: the hooks' lags are [hooks.status.md](hooks.status.md)'s.
  - 5: nothing takes a project's ways or charters from its repository revision, so no start names a way, and none waits for a first good read (`../domain/forge/temper-dir.md`, 3).
  - 6: `../protocol/temper-dir.md`, 8: the judgement checks no way's tasks, executors, contracts, guardrails or recipes, and no charter's `settings` or bounds.
  - 7: `../domain/forge/adoption.md`: a project's creation and adoption write nothing into its repository.
- **Design departs:**
  - 6: a hook that is a symbolic link is judged where it runs, not before it lands: the forges' listings give no link's target (`../protocol/temper-dir.md`, 5). A link anywhere else temper reads is refused before it lands. Goal 21394's designer decided, 2026-11-28, keeping the rule where the forge can tell it.
  - 6: a fault the turn's candidate shows, where each covered member's own head passed, holds those members for a person rather than repairing them, since no one member's diff shows it (`../domain/forge/temper-dir.md`, 4). Goal 21394's designer decided, 2026-11-28.
  - 6: a change that only moves a file out of `.temper/` lands unchecked, since the forges' comparisons give a renamed file at its new path alone; the read after its landing judges the tree, and the last good revision holds (`../domain/forge/temper-dir.md`, 8). Goal 21394's designer decided, 2026-11-28.
