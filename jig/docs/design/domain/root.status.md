# root: status

2026-10-29. Where jig's code lags or departs from [root.md](root.md).

- **Built:** 4, 10: jig-owned `JournalSizes { commits, writes, door, release }`; commits and door set the watermark, writes supplies the store's commit edge, release bounds each release call. Core, hub and conformance composition; 3: finishing derivation and idle fence. 3, 4, 10: the reference root admitted by a watermark (`try_step`), its journal uncapped (`decide`); orders tested under load. 3, 4, 10: ops admitted by a watermark (`try_step`, the refused event handed back), its journal uncapped. 10: the core names no journal room; `Limits.routes` bounds its drains (small 20,783, standard 151,655), with each predicate asserted empty. 10: the core's route scratch grows as it fills; the watermark defaults to the journal's commits plus its door.
- **Not yet:** 2, 5, 6: inline agents. 3: mixed events split. 11, 12: ops root still decodes routed bytes. 5, 8: settings hand-offs unknown.
