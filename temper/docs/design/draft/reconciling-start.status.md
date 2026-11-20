# reconciling-start: status

2026-11-20. Where the design lags or departs from [reconciling-start.md](reconciling-start.md).

- **Design lags:**
  - 2: `../domain/forge/branches.md`, 1, has a restart remake a missing goal branch in read-afresh; `skein/docs/design/forge/client.md`, 8, has the restart's reads before the settle with no rule against a read waiting on a write.
  - 3: `jig/docs/design/domain/model.md`, 4, has a restart read every connector afresh before deciding anything new; `jig/docs/design/domain/engine/restart.md` opens after every connector's settle; `jig/docs/design/domain/connectors.md`, 5.1, has no reconciliation after opening; `jig/docs/design/protocol/service.md`, 3, admits clients only once decisions open.
  - 5: a forge's reconciliation after the opening, its census row and the forge pages' "reconciling" wait for section 3: until then reading afresh and settling the outbox are steps of the start, whose waiters `../protocol/README.md`, 6.3 names in the log alone.
  - 6: `../protocol/binaries.md`, 2, ends `upgrade --check` at the switch; `../redeploy.md`, 4.1, passes the check there.
- **Design departs:**
  - 4: the bounds are printed by `temper-engine versions`, not `--version`: `--version` stays the one line the redeploy tool and the binary's test read whole, and `versions` is the engine's version details, reading no configuration, store or settings, so it prints the bounds at the stock limits (`../protocol/binaries.md`, 2). The goal's designer, 2026-11-20.
  - 5: the report bound is one fixed figure, 30 s, for every step of a start and every full table, not one derived from each step's work on the deployment's store: `versions` reads no store, and dev.fabrig.ai's whole start took ten seconds once its cycle was cut (`../protocol/configuration/derived.md`, 5.6). The goal's designer, 2026-11-20.
  - 5: "the health census" and "the deployment census" are the deployment's forges, the forges' document on the client's Deployment page and `temper ctl forge list`: no census of waits exists, and the outbox and read admission are the forge connector's, one for every forge (`../protocol/README.md`, 6.3; `../client/forge/deployment.md`, *A full table*). The goal's designer, 2026-11-20.
- **Open:** none.
