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
| 3 | `fe-surface`, `bx-compose` |
| 4 | `cl-choices`, `cl-ctl`, `cl-forges`, `cl-welcome` |
| 5 | `cl-pg`, `cl-pg-goes`, `fe-kept-root`, `fe-make`, `fe-playground-forge`, `fe-playground-root`, `fe-records`, `fe-remove`, `pd-acts`, `pd-vocabulary` |
| 6 | `pd-accounts-uses`, `pd-people-purge`, `pd-store-reads`, `pd-tasks-erase`, `pd-test-connector` |
| 7 | `bx-doc`, `bx-fixtures`, `bx-harness`, `bx-init`, `bx-walks`, `sv-family`, `sv-playground`, `sv-registry`, `sv-registry-setup`, `sv-removal`, `sv-resolve`, `sv-seed-kit`, `sv-service-fixtures`, `sv-startup-machine`, `sv-startup-refuse`, `sv-world-client-browser`, `sv-world-one-process`, `sv-world-real-loop`, `cl-world-forges` |
| 8 | `bx-story-github`, `bx-story-playground`, `bx-story-reset-cut`, `bx-story-setup-cut`, `bx-unforge`, `cl-journeys`, `cl-world-pg`, `fe-config-drop`, `fe-demo-gone`, `fe-make-git`, `fe-world-remove`, `fe-world-reset`, `pd-client-view`, `pd-client-wire`, `pd-conformance`, `pd-core-decide`, `pd-core-erase`, `pd-core-release` |

## Critical path

63 tasks, 5995 implementer-minutes estimated. By the specs' estimates the longest chain is
1080 minutes:

`fe-surface` → `sv-family` → `sv-startup-machine` → `sv-registry` → `sv-registry-setup` →
`sv-resolve` → `sv-removal` → `sv-playground` → `sv-service-fixtures` → `sv-startup-refuse`
→ `fe-config-drop` → `fe-make-git`

temper-service has one writer at a time. Close behind: the binary's chain to `bx-unforge`
(1055: `bx-pipe` → `bx-compose` → `bx-doc` → `bx-init` → `bx-harness` → `bx-fixtures` →
`bx-walks` → `bx-story-playground` → the three later stories at once → `bx-unforge`), the
client to `cl-journeys` (950), and jig's core to `pd-conformance` (680). The playground's
end-to-end tasks (`bx-story-playground`, `cl-world-pg`, `fe-world-reset`) wait on jig's
`pd-core-release`.

## Notes

- `notes/contracts.md`: the binding shapes (J jig, T temper, G generated docs).
- Each designer's `notes/<prefix>-progress.md`: its graph, its calls, the lead's answers;
  `notes/fe-surface.md` and `notes/cl-wire.md`: shared shapes.
