# Plan: playground-reset-r6

The draft `temper/docs/design/draft/playground-reset.md`, sections 3 to 7, cut by a lead and
five Claude designers, built by Codex implementers. Binding shapes:
`notes/contracts.md`; standing rules: `notes/common.md`; where the code stands: `status.md`.

## Parts (the draft's order, section 7)

| Part | What | Designers (spec prefix) |
|---|---|---|
| `fs` | Forges in the store (3): records, numbering, the box's groups by kind, `temper init` naming no forge, limits as counts, skein's forges at runtime | fe, sv, bx, cl |
| `rf` | Removing a forge (4), with a set up started over (6.4) | fe, sv, bx, cl |
| `pd` | Deleting a project, jig's (5) | pd |
| `pg` | The playground on them (6.1-6.3): records, reset, what goes | fe, sv, bx, cl |

Crate families, one designer each, one writer per crate at a time: `pd-` jig; `fe-` store,
forge connector, root and their worlds; `sv-` skein's forge layers and temper-service; `bx-`
box, setup steps, fakes, init, the binary and the process tier; `cl-` the client wire,
domain, view, tools and the client world.

## Released

| Batch | Tasks |
|---|---|
| 1 | `sv-skein-connection`, `sv-skein-setup` (T7, skein only) |
| 2 | `bx-pipe`, `bx-steps`, `cl-wire` |

## Critical path (as it stands)

`fe-surface` → the forge connector from records → `RemoveForge` → the playground's reset
(engine) → the service's routing of each → the client's pages → the process-tier stories.
jig's `pd` runs beside it; the reset's engine task needs jig's `RemoveProject` surface.
Updated with each batch.
