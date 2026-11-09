# ui-p3f-design: the path's "design" and "close-out"

Milestone P3f of `temper/docs/design/draft/whole-story.execution.md`: a goal's
template says whether its coordinator designs first and whether it makes a
close-out, and the one derivation draws "design" in place of "plan", and a
"close-out" after the landing, from that setting. Request: `inputs/request.md`.
`notes/common.md` holds the rules every task follows; `notes/contracts.md` the
shapes that cross tasks.

## Parts and tasks

| Part | Task | Needs | What it makes run | Minutes |
|---|---|---|---|---|
| `goal` | `tasks/goal-wire.md` | | `TemplateGoal` on the client wire and in the store, `PathStep`'s `design` and `close_out`, `SettingValue`'s `goal`, regenerated, nothing filled | 45 |
| `goal` | `tasks/goal-value.md` | `goal-wire` | the forge's kind 11: `TemplateGoal`, its judgement, `template_goal`, the store's and smith's codecs | 75 |
| `path` | `tasks/goal-path.md` | `goal-wire` | the derivation's "design" and "close-out", `Start::Goal`'s two settings, the service's two steps | 90 |

Later pushes add the rest of `goal` (the kind declared and proposed, ctl), the
root's read in `path`, and the part `stock` (template 4's stock and reference
values).

**Critical path, so far:** `goal-wire` (45) → `goal-value` (75); `goal-path` beside `goal-value`.
