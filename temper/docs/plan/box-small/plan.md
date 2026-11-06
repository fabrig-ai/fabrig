# box-small: the plan

From `inputs/request.md`: three small things the user met walking a real box. The trunk is `box-small/integration`. `walk = false`: the user's walking a box now.

## Parts and tasks

| Item | Part | Task | Needs |
|---|---|---|---|
| 1 | `printed-words` | `printed-words` | none |
| 2 | `copy-part` | `copy-part`, skein's clipboard | none |
| 2 | `copy-button` | `copy-button`, temper's Copy | `copy-part`, `printed-words` |
| 3 | `link-hours` | `link-hours` | none |

The request asks for three tasks. Item 2 is two, because it asks for the generic mechanism in skein first, and a branch stays in its tree.

## What the cut decided

- **Item 1 changes no wire.** The answer's text, `ProgramEnvironmentAnswer.environment`, stays one opaque string, composed by `temper`, and gains the binary's word. The service and the domain don't move; only `temper`'s `printed.rs` and the view's `command` do. That keeps it off `first-usable-model`'s and `sign-in-new-tab`'s schema lines. The field's name stays `environment`, renaming it being a wire change nobody asked for.
- **Item 3 is one task across two trees.** The request asks for three tasks, and jig's part is one value whose only reader outside fixtures is this footer. Its branch says so.
- **The clipboard is a part of skein's,** where `skein/docs/design/ui-web.md`, section 15 had it an application's own code until a second application writes one. The request says a generic mechanism goes in skein first, so section 15 changed.
- **Copy is on the welcome's block alone,** as the request has it: every `temper-welcome-cmd`, of which there is one producer. ConnectModel's command is `temper-flow-code` and has none. Say if it should.
- **The state is the root's, in a new file.** `copy.rs` holds it, with a field and a few one-line hooks elsewhere, which keeps it off `ui-walk-3`'s and `sign-in-new-tab`'s hunks on `connect_model.rs` and `pages.rs`. Two reserved look rows are reused, keeping off the look table's tail, where the other drafts add.

## What isn't proved

No temper browser story presses Copy: no story in that tier reaches the Claude panel today, and the user's walking a box. Skein's story holds the part in Chromium, and the host test holds that the press's turn asks it. The one line between them, in `carry`, is wasm-only and read, not run.

## Overlaps

- `first-usable-model` (queue): `temper/crates/temper/src/lib.rs`, the call site; `temper-client-view`'s `pages/connect_model.rs`, `command`; `temper/docs/design/client/README.md`, 4.6; and its `program_login` says the binary as well. As the request says, by catch-up: `notes/contracts.md` writes how the two meet, in whichever order they land.
- `ui-walk-3` (priority): `printed-words` touches `command` and one word row of `temper-client-view`. `copy-button` touches `temper.css` (one line and three rules), `look.rs` (two reserved rows), `words.rs` (three rows), `welcome_model.rs` (one call), `host.rs` and `lib.rs` of the web shell, and new files otherwise.
- `sign-in-new-tab`: `copy-button` adds beside its tab on `boundary.rs`, `domain.rs`, `route.rs`, `host.rs`, `lib.rs` and `journey/*`; `copy-part` adds beside its `tab_fixture` on `shell-web/src/lib.rs` and `browser_stories.rs`. `printed-words` adds a test and a fixture line to `tests/connect_model.rs`.
- `chat-reading-c/page-reading`, `debt-27/dead-api`: other files of `skein-ui-web`; `copy-part` adds a module and touches theirs in `lib.rs` alone.

## Critical path

`copy-part` and `printed-words`, then `copy-button`. `link-hours` runs beside them and waits for nothing. Slices are numbered as the request's items are; each can land by itself.
