# ui-box-children: the plan

2026-11-07. From `inputs/request.md`: `red-70`, a box that leaves a process behind it. After-landing's `a_person_asks_claude_code_to_land_a_change_on_the_owned_forge` failed at `21a9819cc3` on `no surviving children`, its box having exited 0 on its `SIGTERM`.

## What is known, read not run

The story's evidence holds process ids alone (skein-world's `tree_status().live_at_leader_exit`), so no log names the survivor. This cut's read-only survey ranked the candidates by how they're spawned (`setup.md`, 7.1 and 7.2; skein's `io.md`, 6):

- **A run's agent or its `claude`, left by serve.** serve's worker spawns each run's `temper internal agent` in a group of its own, with no parent-death guard, and the agent spawns `claude` in another. The box sends serve `SIGTERM` twice, `box.grace_ms` apart, then closes it, killing serve's group alone.
- **A coding run's own child.** Claude Code starts its Bash tool's commands and the MCP relay itself; whether they leave its group isn't in this repository. The request makes that temper/migration's: a `Question:`.
- **A step's or Forgejo's group member still dying.** skein-io's close kills a group and reaps its leader without waiting for the group to empty: skein's, a `Question:`.
- **A runner job or a Forgejo `git` child,** less likely, since the ChatGPT twin shares them.

So the one task first makes the failure name its survivor, then reproduces it.

## Parts and tasks

| Part | Task | Needs | Request item | Crates |
|---|---|---|---|---|
| `survivor` | `box-survivor` | none | 1, 2, 3 | `temper` (test support), and where the cause lies: `temper-box`, `temper-service`, `temper-forge-setup`, `temper-forge-setup-shell` |

- **`box-survivor`:** a recorder in the live story's judge (`tests/support/forge_land/survivors.rs`) notes each descendant alive at the box's exit, with its parent, group, session and command line, and the stop's rows for its parent and group leader, through `live::notice`'s filter; then the story by name, alone, through `proof-live.sh` with `heavy`, at most three runs; then the fix at its lowest place, a failing focused test first, by the spec's options A to E, or a `Question:` where the cause is a coding run's child, skein's or smith's. The recorder stays, so a later red names its survivor.

One task, as the request has it: the fix depends on what the run shows, and one writer keeps the diagnosis and the fix together.

## Critical path

`box-survivor`, about 240 implementer-minutes, much of it the live runs' wall time (up to 40 minutes each; a run with a survivor waits out the story's deadline before its cleanup). The plan lands when it has merged and the trunk's end check is green.

## Decisions

- **The recorder lives in the live story's judge,** not in skein-world: it reads `/proc` for the pids skein-world already gives, once, at the box's exit, so no skein change and no change to the story's check.
- **The live run goes through `temper/scripts/proof-live.sh forge-land-claude` with `TEMPER_WRAPPER=heavy`,** a fresh store and a private `TMPDIR`: the script is the story's runner (`testing.md`, 2.7), checks the held login by `stat` and stops what's left under the store. Running it departs from `testing.md`, 2.7's *never run by an implementer*, on the director's word in the request and `AGENTS.md`, 5's repair rule.
- **A cause outside temper is a question,** since a branch stays in its tree: skein's close, smith's program stop, or a coding run's own child (the request).
- **No held login in the slot is a question too,** after the recorder is pushed: sign-ins are the user's, and the task never looks for one.
- **No deadline widens:** a fix that bounds serve's drain or the runner's shutdown keeps them inside the box's two graces.
- **The status entry** (`temper/docs/design/protocol/setup.status.md`, **Not yet:** 7.2, *Its children*) was written by this cut; the task removes it on a fix. No stored format, protocol or wire changes are expected.
- **Nothing dropped:** the request's three items are the task's steps. The greenfield mockup's sources bear on no part of this request.

## Overlaps

None with the drafts beside this one: `per-wake` is jig's, `look-round-17` temper's client view and `temper.css`, `tui-connected` skein's fail-stop, shell terminal, screen and simulator. A cause in skein's `skein-shell` process code would meet `tui-connected/fail-stop-restore`'s `skein-shell/src/lib.rs`; that is a question here, not an edit.
