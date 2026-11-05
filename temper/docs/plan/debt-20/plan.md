# debt-20: the twentieth debt round

Seven independent items (`inputs/request.md`): skein D002, D018; jig D015, D025; smith D003, D007; hearth D007. All seven are open at this plan's base. No temper client view, no `temper.css`, no `jig-client-view`.

This file grows with each release. Released so far: hearth D007 and all of skein D002 but `skein-io`.

## Parts and tasks

| Source | Part | Task | Needs | Effort | Min | What it does |
|---|---|---|---|---|---:|---|
| hearth D007 | `hearth-fakes` | `mmdebstrap-once` | | medium | 20 | `apt::pass` once; the item goes |
| skein D002 | `skein-widening` | `widen-forge` | | medium | 45 | 115 sites |
| skein D002 | `skein-widening` | `widen-oauth` | | medium | 30 | 71 sites |
| skein D002 | `skein-widening` | `widen-llm` | | medium | 30 | 68 sites |
| skein D002 | `skein-widening` | `widen-fakes` | | medium | 40 | 103 sites |
| skein D002 | `skein-widening` | `widen-kit` | | medium | 55 | 137 sites |
| skein D002 | `skein-widening` | `widen-ui` | `widen-kit` | medium | 20 | 43 sites |

## Critical path, as it stands

`widen-kit`, `widen-ui`, 75 minutes.

## Decisions: D002

Re-measured: the item's **Measure** reads 591, not 603. 27 of those are generated (`skein-oauth/src/generated/v3.rs`, `examples/ui/view/src/generated/v1.rs`), so 564 are hand-written. A further 10 `usize::try_from(..)` wrap over a line and 12 `u64::try_from(x.len())` are nested or wrapped: the **Paid when** covers them, so the tasks' second search does.

- **The generated sites stay.** The schema compiler prints them, and changing it would regenerate every tree's codecs for no reader's gain. They're in the floor, and the old forms' engine skips `generated/` anyway.
- **Tasks group crates by family,** one writer per crate, in place of one task per crate: 36 crates, most with under 20 sites, and the compiler does the sorting.
- **Dependents don't run:** no signature and no result changes.
