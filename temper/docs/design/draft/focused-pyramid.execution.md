# focused-pyramid: execution

2026-11-15. How [focused-pyramid.md](focused-pyramid.md) gets carried out: in small steps, each landing on its own before the next in its stream starts from main, so no branch grows large or old. It designs nothing: each milestone writes its own design from the draft as it runs, and lands whole. Plans use the focused and fuzzy suites alone. The figure of the draft's section 6 is read after each lands, and is no plan's. The first plan's K1 to K5 were withdrawn on 2026-11-15 before any landed; these start at K6.

## 1. The milestones

Each is a goal, one change of one concern. A milestone that grows past a day's change splits, and this file says how.

| Milestone | Draft parts | Stream | Needs | Its figure after landing | State |
|---|---|---|---|---|---|
| K6, a wrong format refused before the registry: `format::check_versions` before `registry::open` in `jig/crates/jig-protocol-store/src/open.rs`, the refusals' order and every level and family of the baseline test kept | 5.2 | store open | none | `jig-store-world`'s `upgrades::baseline::real_jig_set_versions_match_each_level` against its 52.5 s | done, by goal 3653 (jig's `protocol/store/records.md`, 3, and `protocol/store/opening.md`, 1.7); its figure is read after landing |
| K7, a parsed schema kept: `Binary::new` keeps the schema it validated, and the registry's texts take it rather than parse it again | 5.1 | store open | K6 | jig's and temper's focused CPU | not started |
| K8, one bounds check per open: the checked set and its metadata carried through the registry's admission, patch planning and readiness | 5.1 | store open | K7 | the same | not started |
| K9, an open's parsed texts and layouts reused across the registry's rows and checks, dropped when the open settles, no cache outliving it | 5.1 | store open | K8 | the same | not started |
| K10, the reconnect fixtures' statements prepared once per fixture and version, every connection, loss and assertion still run (`jig/tests/engine-workers/src/fixtures.rs`, `jig/tests/worker-link/src/fixtures.rs`) | 5.4 | wire | none | `jig-engine-workers-world`'s and `jig-worker-link-world`'s suites against their 14.8 and 12.2 s | not started |
| K11, temper's application bridge made only when the agreed version needs a conversion, root and admission still checked (`temper/crates/temper-protocol-worker-wire/src/application.rs`) | 5.3 | wire | none | the engine component against its 20 CPU s; temper's focused CPU | not started |
| K12, the wire's fixed tables (its spoken range, fingerprints and bounds) built once per service and shared by its parts (`jig/crates/jig-protocol-worker-wire/src/`) | 5.3 | wire | K11 | the same | not started |
| K13, `cli_names` searches every declared name in one pass (`tests/boundary/tests/cli_names.rs`) | 5.5 | boundary | none | the boundary checks' CPU against their 99 s | not started |
| K14, one walk of the tracked files per boundary binary, shared by the checks that read them (`skein_worlds.rs`, `shared_test_code.rs`) | 5.5 | boundary | K13 | the same | not started |
| K15, `pre_pr`'s rows that classify results as the driver's Python unit tests, its end-to-end tests one per external path (`tests/boundary/tests/pre_pr.rs`, `tools/pre-pr/test_pre_pr.py`) | 5.5 | boundary | none | `pre_pr` against its 72 s | not started |
| K16, the one-process world's upgrade stories placed (its `upgrade` module, 17 tests, 13 past 2 s, 209 s): power loss back to its design's drawn moment, the cuts swept in fuzzy, the store's own cuts in `jig-store-world`'s suite; the settled story's image a constant fixture if each case still runs its own upgrade, cut and recovery | 4, 5.4 | one-process | K8, K11 | the worlds below the domain against 8 s; its stories against 2 s | not started |
| K17, the one-process world's `one_process` and `public` modules placed (27 tests, 124 s), each story past 2 s by what it checks | 4 | one-process | K16 | the same | not started |
| K18, the one-process world's other modules placed (20 tests past 2 s, the modules 148 s: `m15`, `across_versions`, `golden`, `core_after_project`, `copy`, `github`, `story_a`, `paging`, `push`, `ready_copies`) | 4 | one-process | K17 | the same | not started |
| K19, the real loop's stories past 2 s placed, its first half (`cluster`, `terminal`, `github_repair`, `github`, `github_land`, `notes`: 15 tests, all past 2 s, 98 s), each keeping one end-to-end path | 4 | real loop | K8, K11 | the real loop against 15 s; its stories against 2 s | not started |
| K20, the real loop's other stories past 2 s placed (`forge`, `sign_in`, `model_check`, `lifecycle`, `case_5_3`, `goal_delivery`, `plan_chat`, `restart`, `lock`, `lost`, `tools`: 13 tests past 2 s, the modules 58 s) | 4 | real loop | K19 | the same | not started |

Times are the CI builder's focused run of 2026-11-15 10:04Z (the draft's section 2).

## 2. Order

- **Each milestone is a goal** that temper/queue starts once it's approved.
- **No priority mark:** builds aren't one of v0.1.0's critical pieces. Its plans start when places are free and marked work isn't about to arrive.
- **One at a time within a stream,** each from main once the one before it has landed: its files are the stream's, so no two branches change them at once.
- **The streams side by side.** The first to start: K6, K10, K11, K13 and K15. The one-process and real-loop streams wait for K8 and K11, which lower every start's and open's cost, so their stories are placed against what they cost then; they run side by side with each other.
- **A story still past its guide** once its stream is done goes to the user with its measure (the draft's section 4, last item).
