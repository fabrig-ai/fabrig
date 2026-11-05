# start-refuses: a release at a start withdraws its stale proposals without a panic

From the redeploy of 2026-11-05: a restart with a pending settings proposal and changed defaults crash-loops. The request (`inputs/request.md`) has the backtrace.

## Parts and tasks

| Part | Task | Needs | What |
|---|---|---|---|
| `server-walk` | `server-walk` | | the walk's two additions, each with its help test |

The fix's part follows in the next push, once its design is checked.

Critical path, as it stands: `server-walk`.

## What was decided

- **The walk does not say a restart is required for the sign-in meta.** The request says so; `temper/docs/design/protocol/sign-in.md`, 9.7 and three tests say the meta comes when the forge is made. The walk says what all of them support, and the report carries the difference to the user.

## Overlaps

`temper/docs/help/server.md` and `temper/crates/temper/src/tests/help.rs` are on no other draft's list.
