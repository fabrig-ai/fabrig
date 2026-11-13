# store-shape: status

2026-11-13. Where the design lags or departs from [store-shape.md](store-shape.md).

Its section 8, the census, is designed (S1): skein's `data/kv.md`, 2.2, jig's `protocol/store/README.md`, 6.3, jig's `client/domain/contract/documents.md`, 1, `../client/pages/settings.md`, 1 and `../client/ctl/tools.md`, 2. The rest is not designed yet: its execution file's milestones write the design as they run.

- **Design lags:**
  - 2: skein's `data/kv.md`, section 6 plans payload logs for turns alone, and none is built; jig's `protocol/store/README.md`, sections 1, 2 and 10.1 to 10.4, and jig's `protocol/store/loads.md`, sections 2 to 4, keep every task's history in the map; jig's `protocol/store/transcripts.md`, sections 1 to 6, keeps turns in files of their own, uncompressed, outside the generations.
  - 3: skein's `data/codec/README.md`, sections 3 to 5 and 9, has every field present with no default, the family's version at a value's head and in its fingerprint; skein's `data/codec/diff.md`, sections 2 to 5, makes every derived change a patch; skein's `data/codec/patches.md`, sections 1 to 7, converts at open through generic values; skein's `data/codec/command.md`, section 2 freezes a text once a landed patch names it; jig's `protocol/store/records.md`, sections 1, 3 and 4, keeps broad roots per family and a layouts row of families' fingerprints, not a registry.
  - 4: jig's `protocol/store/opening.md`, sections 1 to 3, patches every value of a changed family at open, over a copy, and moves no key.
  - 5: jig's `protocol/store/opening.md`, section 2 copies both maps file by file; skein's `runtime/io/files.md` has no link.
  - 6: jig's `protocol/store/README.md`, section 6.1 keeps transcripts past a longer horizon than the full records, where this draft drops turns 14 days after the end and full records after 30, and designs no ended task's summary beyond its size; jig's `domain/tasks/README.md`, section 3 and jig's `domain/tasks/lifecycle.md`, sections 1, 2 and 6, keep a milestone per phase change, and jig's `client/domain/objects.md`, section 11 and jig's client wire carry them; `../protocol/store.md`, sections 1, 2, 4, 5 and 7, keeps change records whole and with no horizon.
  - 7: jig's `domain/tasks/changes.md` amends a task's definition inside its one record; `../protocol/store.md`, section 5 keeps a change whole; no owner splits a hot record.
  - 8: skein's `data/kv.md`, section 10.1 sizes the map at 15 entries a task.
- **Design departs:**
  - 8: skein-kv counts, not jig's component (skein's `data/kv.md`, 2.2): only the store sees each operation it applies with the value it replaces, its snapshots' starts and the frames its recovery replays, and a generic mechanism goes into skein before its consumer. The component gives the rule, names the keyspaces and reports them (jig's `protocol/store/README.md`, 6.3). The designer of S1, 11-13.
  - 8: `temper store census` is `temper ctl DIR deployment census` (`../client/ctl/tools.md`, 2): the `temper` binary's commands are its terminal client, `box` and `help`, a running engine is read by a party through ctl, and ctl's names are a noun of the map, then a part's name (`../client/ctl/tools.md`, 1). The designer of S1, 11-13.
  - 8: the census reaches the deployment's document at most once a minute, not with every commit as the fill does, so that its watchers aren't sent a document per commit (jig's `protocol/store/README.md`, 6.3). The designer of S1, 11-13.
- **Open:** none.
