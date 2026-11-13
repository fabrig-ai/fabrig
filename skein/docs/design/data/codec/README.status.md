# codec: status

2026-11-13. Where skein's code lags or departs from [README.md](README.md).

- **Departs:** 4: `Output` still has `rust`, the records' module with its tests inside. Only the drift tests of the older versions' files that milestone C2 of `temper/docs/design/draft/gate-compile.execution.md` deletes read it: `temper-protocol-store-codec`'s v16 to v19, `jig-protocol-store-people`'s `people_v16` and `smith-transcript`'s v6. Those files keep their test modules until C2 (`gate-compile.status.md`, Design departs), and C2 removes `rust` along with them. C1's coordinator's choice, 2026-11-13.
