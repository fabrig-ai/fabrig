# diagrams: the plan

The request: build `skein/docs/design/draft/diagrams.md`, its cases the acceptance, byte for byte (`inputs/request.md`). Focused and fuzzy alone: the cases' scenes are checked as bytes, and temper/ui looks after landing.

## Parts and tasks

| Part | Task | Needs | What it does |
|---|---|---|---|
| scene-height | `scene-height` | | `Scene::set_height`, the SVG in the figure's flow, the page's lazy scene, `skein-ui.css` |
| flow | `flow-data` | | `skein_ui::flow`: owner, kind, tokens, data |
| temper-flow | `temper-flow-look` | | `temper.css`'s rules for a flow's tokens |

More follow in the next batch.

## Critical path

As it stands: `flow-data`, then the reader and the layout.

## Decided

- **The design is written with this cut:** `skein/docs/design/ui.md`, 3.3, 3.5, 10.1, 11.2, 11.5, 15, 17, 18; `ui-web.md`, 8, 12; `ui-text.md`, 5; temper's `client/README.md`, 3.5, `client/browser.md`, 2.5, `domain/agent.md`, 8.
