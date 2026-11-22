# repairs-briefs: status

2026-11-22. Where the tree's code lags or departs from [repairs-briefs.md](repairs-briefs.md).

- **Not yet:** "A pull section is pinned at the change's head" and "A brief section's read that fails": a pull past the pin fails the brief, transient, whoever pushed it, the task's own earlier attempt included, until the task is held; a lost reread superseded by a later attempt of the same task records no head; the `brief read refused` line names no reason or heads, `error=None` for a refusal the forge answered (`temper-engine-domain-forge/src/brief.rs`, `brief_answer`).

- **Design departs:** from the request `brief-failure` of 2026-11-05, each
  decided by its plan's cut, for the user to see:
  - The request's true capacity is a place, an account or a brief slot. `repairs.md` also
    pauses, with no bound, for a forge client `Busy` at the brief's deadline, a
    forge not connected and a connector with no room, so a repair waits out a
    disconnected forge unheld, though saying why, rather than making a person
    release each.
  - The request has a CI repair's stories in the forge world. They run in the
    engine's domain world (`tests/engine/domain/tests/suite/forge_routes/brief_failure.rs`),
    where the core and the connector meet; the forge world
    (`tests/engine/forge/tests/suite/brief.rs`) tells a lone section's failure.

- **Not yet:**
  - the whole document: the walk the request `brief-failure` of 2026-11-05 ends with has not
    passed: on a real Forgejo, a change whose CI goes red gets a repair that
    starts, and a repair whose brief can't be read ends held with the reason in
    `temper ctl`. Its QA rung is not drafted yet, so both stand on the fake
    forge's stories alone
    (`tests/engine/domain/tests/suite/forge_routes/brief_failure.rs`).

- **Not yet:**
  - the whole document: `[the reviews could not be read: REASON]`, a gate's repair standing on
    its pull, is unobserved: it is built (`temper-engine-domain-forge/src/brief.rs`,
    `brief_answer`), but those stories run a CI repair's pair alone and no test
    holds its words.
