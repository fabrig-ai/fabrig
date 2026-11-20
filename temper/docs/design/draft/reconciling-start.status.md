# reconciling-start: status

2026-11-20. Where the design lags or departs from [reconciling-start.md](reconciling-start.md).

- **Design lags:**
  - 3: `jig/docs/design/domain/model.md`, 4, has a restart read every connector afresh before deciding anything new; `jig/docs/design/domain/engine/restart.md` opens after every connector's settle; `jig/docs/design/domain/connectors.md`, 5.1, has no reconciliation after opening; `jig/docs/design/protocol/service.md`, 3, admits clients only once decisions open.
  - 4: `../domain/root.md`, 8.2, hears a stop only once the root is ready; `../../help/server.md` sets no unit timeout from the engine's bounds.
  - 5: no design names a start's or a reconciliation's waiters.
  - 6: `../protocol/binaries.md`, 2, ends `upgrade --check` at the switch; `../redeploy.md`, 4.1, passes the check there.
- **Design departs:** none.
- **Open:** none.

Section 2's check of every term that ends reading afresh (S1, 2026-11-20;
`temper/crates/temper-engine-domain-forge/src/domain.rs`, `read_afresh_complete`):

- **Reads alone:** the forge client's fresh pass, the landings' startup
  gathers, the pending CI reads, the goal files' reads and the lost
  writers' reads. None waits for an outbox place, an entry number, the
  outbox's readiness, a write's outcome or a task's release. While the
  outbox is paused no entry is sent or holds a call slot, so a read
  answered `Busy` waits on other reads alone (skein's
  `forge/client.md`, 8).
- **The one that waited:** the goal branches' makings. A held branch
  found missing with no entry asked the root for an entry number during
  the read, then waited in the outbox's line, which asks no one before
  the settle. A making now counts only while it reads, and its write
  waits in the line for after the settle (`../domain/forge/branches.md`,
  1).
- **No term of their own:** jig's start script and temper's root add
  nothing to the phase but the forge's report that it read afresh.
- **Left to section 3:** each of these reads is asked again, with no
  bound, while its forge stays away. That is a wait on a read, not on a
  write, and section 3 takes it out of the start.
