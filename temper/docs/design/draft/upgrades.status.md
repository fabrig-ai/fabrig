# upgrades: status

2026-11-02. Where the design lags or departs from [upgrades.md](upgrades.md).

The design carries sections 3 to 13 across jig's `protocol/store.md` (levels, patching at open, generations, effects in flight), skein's `codec.md` (5, 9 and 12: patches, fingerprints, the command and its check) and `kv.md`, smith's `protocol/transcript.md` (`fresh-windows`), `../domain/root.md` (the settled stop) and `../protocol/README.md`, 4 and 9 (`temper versions`, `temper upgrade --check` and `--back`, the baseline).

- **Design lags:** none found in a check of sections 3, 4, 5, 7.4, 8, 9 and 12. Where the design cites this draft for a rule instead of stating it (jig's `protocol/store.md`, 21 places; skein's `codec.md`, 8; temper's `../protocol/README.md`, 8), the rule is this draft's until the design states it.
