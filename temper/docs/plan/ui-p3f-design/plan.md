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

Later pushes add the rest of `goal` (the forge's value, the kind declared and
proposed, ctl), the part `path` (the derivation and the root's read) and the part
`stock` (template 4's stock and reference values).

**Critical path, so far:** `goal-wire`.
