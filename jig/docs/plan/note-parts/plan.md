# note-parts: the plan

jig's notes as an LLM's working area: `note` takes a line range, so that a body past what one call holds, about 10.6 KB on a box, is built
in parts up to `notes.body_bytes`. As the request says: `jig/docs/plan/note-parts/inputs/request.md`.

## The parts

Design first, in the cut itself: jig's `domain/engine.md`, 10 and 16, `client/domain/contract.md`, 6, `draft/notes-memory.md`, 5, and
temper's `protocol/configuration.md`, 13.8, `domain/agent.md`, 8.3, `client/tools.md`, 5.2. The shapes that cross tasks are in
`notes/contracts.md`.

| Part | Task | Needs | Minutes | Says |
|---|---|---|---|---|
| `note-range` | `note-range` | | 85 | jig: machine, core, store, jig's tool and words; pure tests and world refusals |
| `task-notes` | `task-notes` | | 60 | temper ctl: `task notes`, the pane's read, which the proof needs and ctl lacks |

## The critical path

`note-range` (85), then the rest as it is released. `task-notes` runs beside it. The end check waits on `ui-chat-draft`'s landing.
