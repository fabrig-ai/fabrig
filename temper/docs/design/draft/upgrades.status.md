# upgrades: status

2026-11-12. Where the design lags or departs from [upgrades.md](upgrades.md).

The design carries sections 3 to 13 across jig's `protocol/store/README.md` (levels, patching at open, generations, effects in flight), skein's `data/codec/README.md` (5, 9 and 12: patches, fingerprints, the command and its check) and `skein/docs/design/data/kv.md`, smith's `protocol/transcript.md` (`fresh-windows`), `../domain/root.md` (the settled stop) and `../protocol/binaries.md` and `../protocol/README.md`, 7 (`temper versions`, `temper upgrade --check` and `--back`, the baseline).

- **Design lags:** 4.1 and 4.2, since 2026-11-12: skein's `data/codec/README.md`, section 5, jig's `protocol/README.md`, section 7, and `../protocol/README.md`, section 7 still keep every version unmoved until the first release, where the baseline is now the store `main` wrote at `bc23255eb7` and a version moves at the landing that changes its layout. Section 9 is `redeploys.md`'s now, and its lags are that draft's status. Otherwise none found in a check of sections 3, 4, 5, 7.4, 8 and 12. Where the design cites this draft for a rule instead of stating it (jig's `protocol/store/README.md`, 21 places; skein's `data/codec/README.md`, 8; temper's `../protocol/README.md`, 6), the rule is this draft's until the design states it.
- **Design departs:** 10: `../protocol/README.md`, 7 versions only the forge's setup `state`; the worker's secret and digest, the demo model's key, the forge's sign-in client secret, the box's started markers and the admin token's handover are fixed forms, never versioned, a change taking a new file name. And the owned forge's preparation writes `state` before the engine starts, so going back past a preparation that migrated it needs the forge's dump, where this draft writes no small file before the new generation's first commit. The reason the design gives: `state` follows the forge's own upgrade. Who decided is not recorded.
- **Design departs:** 6.4: a link the start cannot make refuses the
  start, naming the record and what it links to, where this draft has a
  panic (jig's `domain/engine/restart.md`; `../domain/root.md`, 8.1).
  The user decided it by the request `store-crash-safe` of 2026-11-09:
  a store failing a link check at start stops with a reason, not a
  panic.
