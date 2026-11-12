# redeploys: status

2026-11-12. Where the design lags or departs from [redeploys.md](redeploys.md).

Of this draft, sections 4.4, 6.1 and 6.2 are designed and built (milestone R2): `../domain/worker/README.md`, sections 2 and 5, jig's `domain/hosts/hub.md`, sections 6, 7 and 9, and smith's `domain/run.md`, section 9.

Sections 6.3 and 6.4 are designed, in `../domain/worker/checkouts.md`: stores kept across a restart and made once, blob-less, workspaces not kept, and preparations a few at a time, `checkout.preparations` of them, 1 by default.

The client wire's version rule (8) is designed in jig's
`protocol/client/README.md`, 6: the overlay changes its wire from 19 to 20,
with another version refused. The rest remains below.

Section 9 is designed and built: jig's `domain/tasks/lifecycle.md`, 6 gives infrastructure failures a class and a bound of their own.

- **Design lags:**
  - 3.1: jig's `protocol/README.md`, section 7, jig's `protocol/channel/README.md`, 4.1, smith's `protocol/README.md`, section 8, skein's `data/codec/README.md`, section 5, and `../protocol/README.md`, section 7 still change the wires in place before v0.1.0, keep no older version and translate nothing. `../protocol/channel.md`, section 3 versions temper's payloads by one number; `../domain/worker/jobs.md` gives jobs' requests and results no version; the host tools have no versioned contract.
  - 3.2: jig's `protocol/channel/README.md`, 4.1 states terms from the current codec, and its kinds' table is outside any schema.
  - 3.2, 5.1: `../protocol/binaries.md` has `versions` print the wires' numbers alone, and `upgrade --check` compare them.
  - 3.4: jig's `domain/tasks/lifecycle.md`, 2 makes an assignment its host found invalid a permanent failure, whatever the cause.
  - 4.1: skein's `data/codec/command.md`, section 3 writes forward patches alone, and no floor.
  - 4.2: `../protocol/channel.md`, section 6 encodes a git identity's value once, for every worker.
- **Design departs:**
  - 6.1: a job whose delivery began when the cancel after the drain reaches it lets the delivery settle and answers as it ends, finished or failed, rather than being left for the next attempt to recover; only a delivery the stop bound cuts is left to it (jig's `domain/hosts/hub.md`, 9). The delivery settles in any case, so its outcome is known and nothing need be run again (the designer, 2026-11-12).
  - 6.1: in one process, `temper-box serve`, the worker does not drain: it is stopped after the engine has settled its claims, so a turn it let end could not be committed, and it cancels at once as before (`../protocol/README.md`, section 6; the designer, 2026-11-12).
  - 6.1: the engine's grace rises with the drain, 240 seconds in jig's profiles, above the 186-second stop bound the drain's default gives (jig's `domain/engine/fleet.md`, *The grace is derived from that bound*; the user, 2026-11-12). This is 5.3's rule, met before 5.3's own figures.
- **Open:** none.
