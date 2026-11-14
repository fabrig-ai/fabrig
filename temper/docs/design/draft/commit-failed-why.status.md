# commit-failed-why: status

2026-11-14. Where the design lags or departs from [commit-failed-why.md](commit-failed-why.md).

- **Design lags:** none.
- **Design departs:**
  - 2, beyond "which of the store's failures it was": the engine's store routing also stops the engine as at a failed commit, with the last commit it routed, when a load's cursor or page goes wrong or the store answers a refused open or its secrets not ready while the engine runs. So `Stop::CommitFailed`'s cause is temper's `CommitStop`, the store's `CommitFailure` or the routing's own `RouteStop`, each with its words (`../protocol/store.md`, 4), because the line must name the cause of every stop it is printed for. Decided by this goal's design, 2026-11-14.
  - 2, "its refusal's kind": skein-kv's `Refusal` itself carries the kind and the full map's figures (skein's `data/kv.md`, 2), since only skein-kv's admission knows the bytes a commit would leave; jig's `CommitFailure` carries that refusal whole (jig's `protocol/store/README.md`, 5). Decided by this goal's design, 2026-11-14.
- **Open:** none.
