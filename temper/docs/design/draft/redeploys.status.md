# redeploys: status

2026-11-13. Where the design lags or departs from [redeploys.md](redeploys.md).

Sections 3.1, 3.2, 3.3's interim and 3.4 are designed (`../protocol/wires.md`; jig's `protocol/channel/wires.md`; skein's `data/codec/command.md`, 2, the wire rule).

Sections 3.3 and 4 are designed (milestone R4): skein's `data/codec/bridges.md` (reverses, floors, the breaking declaration, the bridge) and `data/codec/README.md`, 7.3 (a kept version's bounds); jig's `protocol/channel/bridges.md` (the engine's statement, terms, translation in the channel's components, what cannot go down) and `domain/engine/fleet.md`; smith's `protocol/README.md`, 8 (`smith-pieces`) and `protocol/charter.md`, 2 and 5 (the host tools' `contract`); `../protocol/wires.md`, sections 3, 6 and 7.

Of this draft, sections 4.4, 6.1 and 6.2 are designed and built (milestone R2): `../domain/worker/README.md`, sections 2 and 5, jig's `domain/hosts/hub.md`, sections 6, 7 and 9, and smith's `domain/run.md`, section 9.

Sections 6.3 and 6.4 are designed, in `../domain/worker/checkouts.md`: stores kept across a restart and made once, blob-less, workspaces not kept, and preparations a few at a time, `checkout.preparations` of them, 1 by default.

Section 8 is designed in jig's `protocol/client/README.md`, 6: a client
wire's version is its schema's `family` line and moves with its layout,
held by its layouts' list and two checks in `fabrig-boundary`; a client of
another version is refused by name at the hello, and one that can't read
a record of jig's is out of date too, `unreadable` (jig's
`client/domain/client/watches.md`, 2). A page out of date asks its
person to reload (`../client/view.md`, 1), and the terminal says to run
the engine's temper. The clients' wires keep no kept text and no bridge;
temper's side of section 8 is `../protocol/wires.md`, section 9.
The rest remains below.

Section 5 is designed (milestone R5). 5.1 in `../protocol/wires.md`, 10: the engine keeps each worker's last opening since its start (jig's `protocol/store/records.md`, 3, *The workers' openings*), and `upgrade --check` judges each configured worker by it, with the function the opening judges by (jig's `protocol/channel/wires.md`, 3), and the engine's own wires, `temper-host-tools`, by the stored wires. 5.2 in `../domain/root.md`, 8.2 and jig's `domain/engine/restart.md`, 1, *Settling before the stop*: no run or job placed and no procedure stepped while the engine settles. 5.3 in `../protocol/configuration/README.md` (the workers' grace) and jig's `domain/engine/fleet.md` (the engine's grace above the stop bound). 5.4 is the workers' admission of preparations, section 6.4's.

Section 9 is designed and built: jig's `domain/tasks/lifecycle.md`, 6 gives infrastructure failures a class and a bound of their own.

Section 7 is designed (milestone R6) in `../redeploy.md`: `temper-redeploy`, run by the deployments' operator from a description of the deployment it keeps, stages the build on every host, stops the engine settled, runs the new build's check over the stopped deployment, and does what it says: the engine alone, then the workers in turn when asked, or every part together; a build whose check patched is backed up before the swap. Its referee reports each worker's agreed versions, the window against the workers' grace, and the tries spent and tasks held since the redeploy began (7.5). The drills on dev.fabrig.ai and staging.fabrig.ai follow landing, each a redeploy its operator starts.

- **Design lags:**
  - None.
- **Design departs:**
  - 4.2: a git identity's value is translated by jig's workers component at each send, by temper's bridge, rather than encoded by temper for each channel; the engine keeps its two newest generations in its current version alone (`../protocol/wires.md`, 6; `../protocol/channel.md`, 6). Because one place then translates everything an assignment carries, so one that cannot go down is refused whole before anything is sent (the designer, 2026-11-13).
  - 4.2: a call's name and a delivery's evidence, the family `smith-pieces`, have no reverse: every move of it is breaking, its floor its current (smith's `protocol/README.md`, section 8). Because the engine keeps a delivery's evidence as bytes without their version, so no reader could tell which version to translate from (the designer, 2026-11-13).
  - 4.3: the fleet does not place by agreed versions ahead of a try: the engine's component tries each assignment's translation, refuses one that cannot go down as *untranslated*, and the fleet then keeps that task off that channel (jig's `protocol/channel/bridges.md`, 6.1). Because whether an assignment goes down depends on its values, which only its translation judges; it costs a pause per task and channel, and no try (the designer, 2026-11-13).
  - 3.1, 4.2: a job kind's request and result are no wire between temper's builds: its spec is `temper-worker-wire`'s `JobSpec`, and its request and result files are the project's tooling contract, which the worker carries unread; a kind's meaning to a worker is fixed by its number (`../protocol/wires.md`, section 8). Because `../domain/worker/jobs.md` has the worker never read them, so no build of temper's reads another's; the goal of 2026-11-12 decided it.
  - 3.1: the host tools' contract's layout is each declaration's name, effect and input schema; what a tool means, and its description, move the version by its author's hand (`../protocol/wires.md`, section 7). Because a description's wording is no layout a gate can judge; the goal of 2026-11-12 decided it.
  - 6.1: a job whose delivery began when the cancel after the drain reaches it lets the delivery settle and answers as it ends, finished or failed, rather than being left for the next attempt to recover; only a delivery the stop bound cuts is left to it (jig's `domain/hosts/hub.md`, 9). The delivery settles in any case, so its outcome is known and nothing need be run again (the designer, 2026-11-12).
  - 7.1: the check runs after the engine's settled stop, not before it, and a check saying a worker may not stay starts the engine again on its old build before every part goes together, so that each worker drains to a live engine (`../redeploy.md`, 4.2 and 4.5). Because `upgrade --check` reads a stopped deployment's directory, whose kept openings and records only a stopped engine leaves whole; the old engine's restart costs seconds, where a worker drained with no engine would abandon its turns at its stop bound (the designer, 2026-11-13).
  - 7.2: the workers go in turn when the operator asks (`--workers`), never because a coming build's check would say its floor passes them, and the engine always goes first (`../redeploy.md`, 4.4 and 9). Because the tool judges only the build it deploys, whose own check sends a worker it would pass to every part together; a worker first saves nothing the engine's first doesn't (the designer, 2026-11-13).
  - 7.5: no try spent "because of the redeploy" is judged as no run failing, and no task held for failures, of any class but the run's own between the redeploy's start and the referee's read (`../redeploy.md`, 6.2). Because no read of the engine tells what caused a failure, and every class but a run's own report can follow from a redeploy (the designer, 2026-11-13).
  - 6.1: in one process, `temper-box serve`, the worker does not drain: it is stopped after the engine has settled its claims, so a turn it let end could not be committed, and it cancels at once as before (`../protocol/README.md`, section 6; the designer, 2026-11-12).
- **Open:** none.
