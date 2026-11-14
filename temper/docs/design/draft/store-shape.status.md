# store-shape: status

2026-11-13. Where the design lags or departs from [store-shape.md](store-shape.md).

Section 10.1's S2 is designed and built: identity walks rewrite the head alone (skein's `data/codec/patches.md`, 3.1; jig's `protocol/store/opening.md`, 2, step 4). Section 5's generations by hard links are designed (milestone S3: jig's `protocol/store/opening.md`, sections 2, 4 and 5, and skein's `runtime/io/files.md` and `runtime/kernel.md`, 6.4). The rest of this draft is not designed or built yet: its execution file's milestones write the design as they run.

- **Design lags:**
  - 2: skein's `data/kv.md`, section 6 plans payload logs for turns alone, and none is built; jig's `protocol/store/README.md`, sections 1, 2 and 10.1 to 10.4, and jig's `protocol/store/loads.md`, sections 2 to 4, keep every task's history in the map; jig's `protocol/store/transcripts.md`, sections 1 to 6, keeps turns in files of their own, uncompressed, outside the generations.
  - 3: skein's `data/codec/README.md`, sections 3 to 5 and 9, has every field present with no default, the family's version at a value's head and in its fingerprint; skein's `data/codec/diff.md`, sections 2 to 5, makes every derived change a patch; skein's `data/codec/patches.md`, sections 1 to 7, converts at open through generic values; skein's `data/codec/command.md`, section 2 freezes a text once a landed patch names it; jig's `protocol/store/records.md`, sections 1, 3 and 4, keeps broad roots per family and a layouts row of families' fingerprints, not a registry.
  - 4: jig's `protocol/store/opening.md`, sections 1 to 3, patches every value of a changed family at open, over a copy, and moves no key.
  - 5: jig's `protocol/store/opening.md`, section 2 links no task's log, since none exists before S7 (section 2's logs); and it makes a new generation only when a patch is pending, where section 4, step 3 makes one whenever the binary differs, S4's to design.
  - 6: jig's `protocol/store/README.md`, section 6.1 keeps transcripts past a longer horizon than the full records, where this draft drops turns 14 days after the end and full records after 30, and designs no ended task's summary beyond its size; jig's `domain/tasks/README.md`, section 3 and jig's `domain/tasks/lifecycle.md`, sections 1, 2 and 6, keep a milestone per phase change, and jig's `client/domain/objects.md`, section 11 and jig's client wire carry them; `../protocol/store.md`, sections 1, 2, 4, 5 and 7, keeps change records whole and with no horizon.
  - 7: jig's `domain/tasks/changes.md` amends a task's definition inside its one record; `../protocol/store.md`, section 5 keeps a change whole; no owner splits a hot record.
  - 8: no census is designed; skein's `data/kv.md`, section 10.1 sizes the map at 15 entries a task.
- **Design departs:** none yet.
- **Open:** none.
