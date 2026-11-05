# debt-20: the twentieth debt round

Seven independent items (`inputs/request.md`): skein D002, D018; jig D015, D025; smith D003, D007; hearth D007. All seven are open at this plan's base. No temper client view, no `temper.css`, no `jig-client-view`.

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
| skein D018 | `io-states` | `io-signals` | | high | 20 | `Signals` on a state |
| skein D018 | `io-states` | `io-pipe` | | high | 75 | four new pins; `Pipe` on a state |
| skein D018 | `io-states` | `io-file-states` | | high | 75 | `Work`, `Stop`, `Cancelling`; `Event::failed`; the **Measure** reads 1 |
| skein D018 | `io-states` | `io-file-limits` | the three above | high | 50 | `Limits`, `step`, thin forms; the item goes; D049 opens |
| skein D002 | `skein-widening` | `widen-io` | `io-file-limits` | medium | 15 | 27 sites |
| skein D002 | `skein-widening` | `widen-close` | all `widen-*` | medium | 15 | an old form at the floor; the item goes |
| smith D003 | `host-lanes` | `shell-on-stage` | | medium | 15 | the shell off `native_io_valid`; `runner.rs` on the stage |
| smith D003 | `host-lanes` | `agent-on-stage` | | high | 30 | the agent on the stage; `io_valid` on `io::valid` |
| smith D003 | `host-lanes` | `host-io-lanes` | `shell-on-stage` | high | 50 | no route table; `io::valid`; the stage |
| smith D003 | `host-lanes` | `worlds-on-stage` | `agent-on-stage`, `host-io-lanes` | medium | 35 | the worlds on the stage; the floor of 2; the item goes; D020 opens |
| jig D015 | `classifiers` | `classify-people` | | high | 120 | `Outcome::family`, `Ask::family`, `Ask::roster`, a fold |
| jig D015 | `classifiers` | `classify-tasks` | | high | 90 | `MessageKind::family`, `result`, `Hold::placed` |
| jig D015 | `classifiers` | `classify-core` | | high | 80 | `Subject::shape`, `touched_by`, `Act::decision` |
| jig D015 | `classifiers` | `classify-ops` | | medium | 25 | `part_fits` and a view's list, once |
| jig D015 | `classifiers` | `classify-close` | the four above | medium | 20 | the core's lists of `people::Outcome`; the item goes |

## Critical path

`io-pipe` or `io-file-states`, `io-file-limits`, `widen-io`, `widen-close`, 155 minutes. `tools-world-names`, `tools-one-name`, 150. `classify-people`, `classify-close`, 140. `store-copy`, `store-remaps`, 120. `shell-on-stage`, `host-io-lanes`, `worlds-on-stage`, 100. Only D002 waits on another part: `widen-io` on D018, since `skein-io`'s files are being rewritten.

## Production code, item by item

The request asks every item but skein D018 to end with no more production code than it started, and says D018 may add a little.

| Item | Net production (estimate) |
|---|---|
| skein D002 | falls: each site shrinks or stays one line |
| **skein D018** | **about +235: more than "a little".** Below |
| jig D015 | about -825 |
| jig D025 | about -25 |
| smith D003 | about -60 |
| smith D007 | about -50 |
| hearth D007 | about -60, all test code |

## Decisions: D018

Re-measured: the **Measure** reads 53, as the item says, but 32 are production and 21 tests, across `file_layer/` and `tests/`, not all in `file_layer.rs`. `Pipe` has 7 bools and 8 `Option`s, `Signals` 2 and 4.

- **The size is what 5.4 asks for.** About +35 for `Signals`, +100 for `Pipe`, +40 for the file states and +60 for the limits: most of it is exhaustive arms, one handler per cell. Leaving `Pending`'s tuple match would save about 60 lines and leave a known departure, so the caps hold the estimate with some margin and the tasks compact first. fabrig/debt, the plan's owner, can say otherwise.
- **The readers stay on thin forms, so the item's **Paid when** holds for the machine and not yet for its readers.** `hearth-service-builder/src/lib.rs` calls `FileIo`'s old names and the request keeps this plan out of that file, so the names must stay, which means nothing outside `skein-io` is compiler-forced. A move is 216 sites in 44 files, 149 outside `skein-io`'s tests, and some readers (`temper-client-shell-cli`, `jig-client-shell-native`, `jig-protocol-store`) keep no clock in scope. It isn't this round's, so `io-file-limits` deletes the item and opens `D049-file-layer-thin-forms`, with a measure that can only fall, and `io.status.md` records the departure.
- **`store::Store` and `private::Opening` stay out,** in D049 too: the item names `FileIo`, and 20 phases and 9 `Option`s can't be "a little".
- **The **Measure** ends at 1, not 0:** the constructor's own line. Reaching 0 would mean reordering its fields to game the search.
- **Four pins come first for `Pipe`:** no test reaches a Spill's write error, a pipe's read error, the abort of a read or Finish then Demand. They're written against today's code, then held through the change.
- **The new forms are `file_layer::step`,** since Rust has no overloading. `io.md`, 5 says so.
- **`Vec` stays:** the **Paid when** doesn't ask, and `List` has no `insert` or `pop`.
- **`widen-io` waits for all of it,** so 27 lines don't conflict with a rewrite.

## Decisions: D003

Re-measured: the **Measure** reads 15. The item is stale on four points: skein has the lane (`file_layer::owns`) and `valid`, nothing collides, the doubling it asks for sits inside `valid`, and it misses `smith-local-shell`'s `runner.rs`. So **no skein task:** this part is all smith.

- **The floor is 2, not 0.** `service/tests.rs` asserts what one `resume` does, and the machine world is ordered fired, ready, down, completions by its own `#[expect]`, with no multiplexing. Neither is what the **Paid when** names.
- **A verdict changes, and `limits.status.md` already says it should:** from 2^24 sockets to `u32::MAX / 8`, limits that passed `native_io_valid` and then panicked in `Io::new` are refused. `host_io`'s `invalid_limits_are_refused` pins `u32::MAX / 8` as valid, so `host-io-lanes` names that one assertion as its exception to the request. No normal configuration moves: host and account limits are small constants, and inline sockets derive from `conversations`.
- **`stage::up` moves corners no test should reach:** more than `sockets` entries ready or timers due. It's what `skein/docs/design/io.md`, 2 says. A task whose test sees a change stops, except where it's pre-approved to keep one site by hand (`transport.rs`, `runner.rs`, the hosts world), which raises the floor by one and says so.
- **`stage::down` stays out:** pacing tests pin the hand-driven `down` loops, and the **Measure** counts only `resume`.
- **The agent's table stays,** since the **Paid when** names `HostIo` and its `next_operation` seeds a file's randomness. It goes in `D020-hand-set-token-tags` with the hand-set bits, and `skein/docs/design/lib.md`'s citation points there.
- **D018 doesn't hold this up:** its change to `file_layer` is additive, so the two run side by side.

## Decisions: D015

Re-measured: people 1,014 (986 at the item's base), the core 3,233, `ops` 579, tasks 861, with `MessageKind::` at 464.

- **The **Measure** can't reach 0, and the item names no target,** since routing stays. The tasks pin people at 620 and under (~548 reachable), tasks at 660 and `MessageKind::` at 250, four of the core's files at 250 (464 today, since other plans move the crate's total), and `ops` at 565.
- **Answer enums, not the item's sub-enums.** A sub-enum of `Ask` or `Outcome` is possible with no format change, but it reaches 542 production and 1,596 test mentions, and `Event` is priced by `size_of`. `Ask::family()` asks the same question and touches nothing else.
- **A table over every variant comes first,** because no list has an exhaustive pin. A classifier can't drop a variant, but it can misfile one. Written against today's code, held through the change.
- **`reads/*` stays, and the item's **Paid when** still holds.** It's off limits (`ui-chat-draft/notes-list`), and its `tasks::Stored` lists are each a different set of variants: one question each, not one question asked again. `routing/task/save.rs`'s five are likewise.
- **`ops` has little to pay.** Its 38 `core::Held` variants are routing or pass-through, and a method would move them into `vocabulary.rs` and remove nothing. `part_fits` and a view's list go, as `ops`' own functions, since `ApplicationPart` is `jig-client-domain`'s.
- **Two writers in `jig-domain-tasks` and in `jig-domain-core`, by design,** departing from one writer per crate: `store-remaps` and `store-copy` stay in their own files, and neither family builds on the other, so neither waits.

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
- **`jig-domain-core`:** `store-copy`, `store-remaps`, `classify-core` and `classify-close` stay away from `routing/mod.rs`, `routing/effect/`, `routing/run/`, `stored.rs`, `vocabulary.rs`, `effects.rs`, `runs/`, `admission/` and `reads/`, which `ui-chat-draft` is changing. A new enum's export may add a line to `lib.rs`, which `ui-settings/project-templates` also touched: a one-line merge at worst.
- **`jig-domain-people`:** `classify-people` stays off `batch.rs` and `limits.rs` (`ui-chat-draft/person-words`).
- **`jig-client-domain/src/contract/common.rs`** is `ui-chat-draft/notes-list`'s, so `classify-ops` stays off it.
- **`hearth-service-builder/src/lib.rs`** isn't touched, and `io-file-limits` compiles it unchanged.
- **No temper client view, no `temper.css`, no `jig-client-view`** in any task.
