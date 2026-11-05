# debt-20: the twentieth debt round

Seven independent items (`inputs/request.md`): skein D002, D018; jig D015, D025; smith D003, D007; hearth D007. All seven are open at this plan's base. No temper client view, no `temper.css`, no `jig-client-view`.

This file grows with each release. Released so far: hearth D007, all of skein D002 but `skein-io`, jig D025 and smith D007.

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
| jig D025 | `store-rules` | `store-copy` | | high | 40 | `Write::Copy` |
| jig D025 | `store-rules` | `store-remaps` | `store-copy` | high | 80 | the remaps move to their domains; the item goes |
| smith D007 | `tools-namespace` | `tools-world-names` | | high | 100 | every test on one-name mounts |
| smith D007 | `tools-namespace` | `tools-one-name` | `tools-world-names` | high | 50 | the types change; the code goes; the item goes |

## Critical path, as it stands

`tools-world-names`, `tools-one-name`, 150 minutes. `store-copy`, `store-remaps`, 120. `widen-kit`, `widen-ui`, 75.

## Decisions: D002

Re-measured: the item's **Measure** reads 591, not 603. 27 of those are generated (`skein-oauth/src/generated/v3.rs`, `examples/ui/view/src/generated/v1.rs`), so 564 are hand-written. A further 10 `usize::try_from(..)` wrap over a line and 12 `u64::try_from(x.len())` are nested or wrapped: the **Paid when** covers them, so the tasks' second search does.

- **The generated sites stay.** The schema compiler prints them, and changing it would regenerate every tree's codecs for no reader's gain. They're in the floor, and the old forms' engine skips `generated/` anyway.
- **Tasks group crates by family,** one writer per crate, in place of one task per crate: 36 crates, most with under 20 sites, and the compiler does the sorting.
- **Dependents don't run:** no signature and no result changes.

## Decisions: D025

Re-measured: the sites are where the item says. `codec/tasks/models.rs` also holds a remap, and `codec/tasks.rs` two copied literals (the item's "domain message literal"). `store.md`, 3 and 6 had no word on the copy, so this plan wrote it, with **Not yet** entries the tasks close.

- **Only the store's `Write` changes.** `core::Write::CopySettingsRevision`, and the same word in temper's engine, `ops` and `jig-test-domain`, are a domain's own vocabulary, which the item doesn't name. Renaming them would touch about 40 test lines in both trees, with no gain to the store. Routes translate.
- **`u8::MAX` stays the index, and is now the domain's name** (`ModelEffort::GONE`), so no stored value changes. Its readers already refuse an index no model has.
- **`remap_models` moves too, with no sentinel in it:** it's the same rule, and the item's "Where" reaches it through the review's "three models". `referenced` and `encode` are translating, so they stay.
- **`model_names.rs` stays:** it refuses on a missing name, which is other work.
- **Three masks, three `remap`s:** `jig-domain-authority`, `jig-domain-tasks` and `jig-domain-people` each own a `Models`, and step code in jig has no generics. Sharing them isn't this item.

**On "breaking public interfaces".** The request asks to stop and ask on those. The items here name their own changes (`Write::Copy`, `Directory.mount`), the types are the workspace's own, with path dependencies, and every reader is migrated in the same branch, so nothing outside the repository is broken. The plan takes those as asked for, and the tasks stop on anything further.

## Decisions: smith D007

Re-measured: the code is as the item says, and no wire or stored form reads `Directory` or `Authority` (they're copied into a session, not encoded). `smith/docs/design/domain/tools.md`, 2 already says one name per directory, so no design text changes.

- **The tests move first, while the kit still takes both shapes.** Then `tools-one-name` only changes types and removes code, and its "no assertion changed" is checkable. A test that had to change with the types couldn't tell a spelling from a regression.
- **This changes assertions, which the request forbids in general, and the item asks for it:** "the fixtures rewritten to one-name mounts". So `tools-world-names` states the exception: a path's spelling, or a story of a shape that goes. An outcome that changes kind stops the task.
- **`smith/benchmarks/tasks/repository/shell-cwd/grader`** still names `tools::Repo` and `names("/work/src/vendor")`. It isn't built in the workspace, and it's a fixture a benchmark run grades against, so it stays.

## Overlaps with the drafts beside this one

- **`ui-chat-draft/skein-keys`** added 3 lines to `skein-ui-web/src/page.rs`, which also holds 2 widening sites. `widen-ui` goes later and changes only the sites.
- **`temper-service/src/store/route.rs`** has one arm changed by `store-copy`; no draft lists the file.
- **`jig-domain-core`:** `store-copy` and `store-remaps` stay in `settings/`, away from `routing/`, `stored.rs`, `vocabulary.rs`, `effects.rs`, `runs/` and `reads/`, which `ui-chat-draft` is changing.
