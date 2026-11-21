# reconciling-start: status

2026-11-20. Where the design lags or departs from [reconciling-start.md](reconciling-start.md).

- **Design lags:**
  - 3: `jig/docs/design/domain/model.md`, 4, has a restart read every connector afresh before deciding anything new; `jig/docs/design/domain/engine/restart.md` opens after every connector's settle; `jig/docs/design/domain/connectors.md`, 5.1, has no reconciliation after opening; `jig/docs/design/protocol/service.md`, 3, admits clients only once decisions open.
  - 5: a forge's reconciliation after the opening, its census row and the forge pages' "reconciling" wait for section 3: until then reading afresh and settling the outbox are steps of the start, whose waiters `../protocol/README.md`, 6.3 names in the log alone.
  - 6: `../protocol/binaries.md`, 2, ends `upgrade --check` at the switch; `../redeploy.md`, 4.1, passes the check there.
- **Design departs:**
  - 4: the bounds are printed by `temper-engine versions`, not `--version`: `--version` stays the one line the redeploy tool and the binary's test read whole, and `versions` is the engine's version details, reading no configuration, store or settings, so it prints the bounds at the stock limits (`../protocol/binaries.md`, 2). The goal's designer, 2026-11-20.
  - 5: the report bound is one fixed figure, 30 s, for every step of a start and every full table, not one derived from each step's work on the deployment's store: `versions` reads no store, and dev.fabrig.ai's whole start took ten seconds once its cycle was cut (`../protocol/configuration/derived.md`, 5.6). The goal's designer, 2026-11-20.
  - 5: "the health census" and "the deployment census" are the deployment's forges, the forges' document on the client's Deployment page and `temper ctl forge list`: no census of waits exists, and the outbox and read admission are the forge connector's, one for every forge (`../protocol/README.md`, 6.3; `../client/forge/deployment.md`, *A full table*). The goal's designer, 2026-11-20.
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
