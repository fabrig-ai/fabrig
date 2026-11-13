# redeploys: status

2026-11-13. Where the design lags or departs from [redeploys.md](redeploys.md).

Sections 3.1, 3.2, 3.3's interim and 3.4 are designed (`../protocol/wires.md`; jig's `protocol/channel/wires.md`; skein's `data/codec/command.md`, 2, the wire rule).

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
temper's side of section 8 is `../protocol/wires.md`, section 8.
The rest remains below.

Section 5 is designed (milestone R5). 5.1 in `../protocol/wires.md`, 9: the engine keeps each worker's last opening since its start (jig's `protocol/store/records.md`, 3, *The workers' openings*), and `upgrade --check` judges each configured worker by it, with the function the opening judges by (jig's `protocol/channel/wires.md`, 3), and the engine's own wires, `temper-host-tools`, by the stored wires. 5.2 in `../domain/root.md`, 8.2 and jig's `domain/engine/restart.md`, 1, *Settling before the stop*: no run or job placed and no procedure stepped while the engine settles. 5.3 in `../protocol/configuration/README.md` (the workers' grace) and jig's `domain/engine/fleet.md` (the engine's grace above the stop bound). 5.4 is the workers' admission of preparations, section 6.4's.

Section 9 is designed and built: jig's `domain/tasks/lifecycle.md`, 6 gives infrastructure failures a class and a bound of their own.

- **Design lags:**
  - 3.2, 4.1: the engine states nothing back to the worker: with each build's floor its current, an accepted channel means every wire at both sides' one version (`../protocol/wires.md`, section 11; jig's `protocol/channel/wires.md`, section 8).
  - 3.3, 4.1, 4.5: skein's `data/codec/command.md` writes forward patches alone, with no floor kept in the repository and no breaking declaration; each build's floor is its current version (`../protocol/wires.md`, 3.2).
  - 4.2: `../protocol/channel.md`, section 6 encodes a git identity's value once, for every worker; a run's charter names no tools' version, and the engine answers only its current host tools' contract (`../protocol/wires.md`, section 6).
  - 4.3: the fleet places by no agreed version: every agreed version is the current.
  - 4.4: the pieces of smith's channel the engine keeps are versioned as `smith-channel` itself, not as wires of their own (`../protocol/wires.md`, section 2).
- **Design departs:**
  - 3.1, 4.2: a job kind's request and result are no wire between temper's builds: its spec is `temper-worker-wire`'s `JobSpec`, and its request and result files are the project's tooling contract, which the worker carries unread; a kind's meaning to a worker is fixed by its number (`../protocol/wires.md`, section 7). Because `../domain/worker/jobs.md` has the worker never read them, so no build of temper's reads another's; the goal of 2026-11-12 decided it.
  - 3.1: the host tools' contract's layout is each declaration's name, effect and input schema; what a tool means, and its description, move the version by its author's hand (`../protocol/wires.md`, section 6). Because a description's wording is no layout a gate can judge; the goal of 2026-11-12 decided it.
  - 6.1: a job whose delivery began when the cancel after the drain reaches it lets the delivery settle and answers as it ends, finished or failed, rather than being left for the next attempt to recover; only a delivery the stop bound cuts is left to it (jig's `domain/hosts/hub.md`, 9). The delivery settles in any case, so its outcome is known and nothing need be run again (the designer, 2026-11-12).
  - 6.1: in one process, `temper-box serve`, the worker does not drain: it is stopped after the engine has settled its claims, so a turn it let end could not be committed, and it cancels at once as before (`../protocol/README.md`, section 6; the designer, 2026-11-12).
- **Open:** none.
