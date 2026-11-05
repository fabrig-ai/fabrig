# ui-away: the plan

From `inputs/request.md`: the greenfield mock's Away works for real. The switch, a goal's host tool `decided`,
Home's away form, Overturn and Settings' section. This file grows with each push.

## How it sits on the plans before it

The request needs `ui-home-status`, and a plan of the settings area if one is queued; the rulings build on
`ui-goal-page`'s Start goal sheet. None has landed as this is cut. `ui-home-status` and `ui-settings` are in the
merge queue, `ui-goal-page` still has `goal-proof` to run.

- **`after = ["ui-home-status", "ui-settings", "ui-goal-page"]`,** beside fab's own options: this trunk may carry
  their unlanded commits, and the option holds its end check and landing until they've landed.
- **`home-base`** merges `ui-home-status` into this trunk, or main once it has landed.

## Parts and tasks

| # | Task | Needs | Effort, minutes | What it lands |
|---|---|---|---|---|
| 1 | `home-base` | | high, 60 | `ui-home-status`' merged code on this trunk |

The rest of the plan follows this push.

## Critical path

As it stands, `home-base`. Everything else waits on it.
