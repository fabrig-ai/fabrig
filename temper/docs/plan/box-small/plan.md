# box-small: the plan

From `inputs/request.md`: three small things the user met walking a real box. The trunk is `box-small/integration`. `walk = false`: the user's walking a box now.

## Parts and tasks

| Item | Part | Task | Needs |
|---|---|---|---|
| 1 | `printed-words` | `printed-words` | none |
| 3 | `link-hours` | `link-hours` | none |

Item 2, the copy button, comes in the next push.

## What the cut decided

- **Item 1 changes no wire.** The answer's text, `ProgramEnvironmentAnswer.environment`, stays one opaque string, composed by `temper`, and gains the binary's word. The service and the domain don't move; only `temper`'s `printed.rs` and the view's `command` do. That keeps it off `first-usable-model`'s and `sign-in-new-tab`'s schema lines. The field's name stays `environment`, renaming it being a wire change nobody asked for.
- **Item 3 is one task across two trees.** The request asks for three tasks, and jig's part is one value whose only reader outside fixtures is this footer. Its branch says so.

## Overlaps

- `first-usable-model` (queue): `temper/crates/temper/src/lib.rs`, the call site; `temper-client-view`'s `pages/connect_model.rs`, `command`; `temper/docs/design/client/README.md`, 4.6; and its `program_login` says the binary as well. As the request says, by catch-up: `notes/contracts.md` writes how the two meet, in whichever order they land.
- `ui-walk-3` (priority): `temper-client-view/src/**`, touching `command` and one word row only.
- `sign-in-new-tab/elsewhere-goes`: `temper-client-view/src/tests/connect_model.rs`, an added test and one fixture line.

## Critical path

The two tasks run at once. Neither waits for the other.
