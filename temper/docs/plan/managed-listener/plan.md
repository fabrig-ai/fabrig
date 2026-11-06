# managed-listener: the plan

The user's word of 2026-11-06 (`inputs/request.md`): a Forgejo connected as its admin, on this host, gets its webhook; one elsewhere keeps polling. One task, as the request asks.

## Parts and tasks

- **`listener`** (`temper/docs/design/protocol/setup.md`, 5.5), slice 1, `walk = false`: ops proves delivery at the next deployment of dev.fabrig.ai.
  - **`managed-hook`:** the step tests first, `runtime.rs`'s `listener`, the secret handed to a connected forge, the process story, the help and the status.

Critical path: `managed-hook`, alone.

## What this cut wrote

The design, with this push: `temper/docs/design/protocol/setup.md`, 1, 5.3, 5.4, 5.5, 5.6, 10, 13.4 and 16; `temper/docs/design/protocol/forge.md`, 5; `skein/docs/design/forge-setup.md`, 4.5; `temper/docs/design/draft/github.status.md`, 12.2 moved from **Open** to **Design departs**. `setup.status.md` and `temper/docs/design/STATUS.md` say **Not yet**, and the task closes them.

## What the survey found past the request

- **A secret drawn at a first adoption never reached the connection.** The request's two lines leave it open. A managed forge connected with no repository has no secret at its reconcile, so no path on the listener (`skein/docs/design/forge-connection.md`, 6.1). The first adoption's plan draws one and makes the hook, and every delivery then answers `404` until the next start. The task hands the access again when the secret is kept (`setup.md`, 5.4). An owned forge with no first repository has the same gap and gets the same fix. Reconcile drawing the secret with no repository would be simpler, but it is a skein plan's change in a second branch, and the request asks one task.
- **`setup_admin.rs:215` does not turn round.** Its `config()` has no listener. The check the request means is `a_managed_adoption_grants_write_on_a_private_repository_without_a_hook`, whose `access_host(false)` configures one.
- **A forge on loopback connected before this landing** gets its hooks at its next start: reconcile's step 6 draws the secret and makes them, before it connects. That is the path ops' proof takes.
- **A delivery's failure reaches no task by construction:** skein's Forgejo plans read no hook's deliveries, so nothing comes back to temper. The task's step test pins it, with the walks on the fake, whose hooks deliver nothing.
- **The story's throwaway Forgejo probably allows the listener,** its `app.ini` being temper's own for `127.0.0.1`, so its hook may deliver. The story does not assert delivery either way.

## Overlaps

- `temper/crates/temper-docs/src/tests/rules.rs`: `plainer-panels/help-says-so` also adds to it. Unavoidable, since the page rules live there; a new function, so a small merge at most.
- `temper/docs/design/STATUS.md` gained a line on main since this branch's base, away from this clause.
- `temper/crates/temper/tests/process.rs`, which `box-link-first` is changing, stays untouched: the story keeps its name.
- This draft is not priority; fabrig/ui goes first, as the request says. Its plan options are fab's and unchanged.
