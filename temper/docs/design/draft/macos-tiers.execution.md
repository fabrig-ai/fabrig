# macos-tiers: execution

2026-11-18. How [macos-tiers.md](macos-tiers.md) gets built. It designs nothing: each milestone writes its own design from the draft as it runs, and lands whole. Plans use the focused and fuzzy suites alone: no plan runs `macos.yml`, which runs after landing. Every goal here takes the lowest priority (`macos-tiers.md`, section 8).

## 1. The milestones

| Milestone | Draft parts | Owner | Needs | Its walk | State |
|---|---|---|---|---|---|
| T1, the fetch repaired: `macos.yml`'s fetch step through the box's CLI as main has it, the one story reached again | 2's fourth point; 7, step 1 | a goal of the draft's | none | after landing: the next run of `macos.yml` passes its fetch step | landed, 803ec4abac; its walk waits for the next run |
| T2, the workflow as a tier: `macos.yml` dispatch-only with its four inputs and its check that the commit is on main; its parts, one job each, an id's part read from its binary, a part skipped with none of `tests`' ids; the one report `junit` and each part's own file; `build.<part>` as a red, a runner's failure a fault; per-part caches, disk and wall time; Chrome for Testing and the wasm target for `browser`; the doubled slow timeouts for `cfg(target_os = "macos")`; the `live` job, its environment and the key's file; no schedule and no concurrency group; `tier-macos-e2e.yml` and `tier-macos-suites.yml` on the label `relay`; `../testing/macos.md` rewritten to match | 3; 4; 5.1; 6's second to fourth points | a goal of the draft's | T1 | after landing: a run by hand of the set `all` at main's tip reports every part, the one story among them, in one `junit` | not started |
| T3, the relay: `$CI_TIER` for a macOS tier as 5.2's contract has it, the label `relay`'s runner, and the token in its private file | 5.2; 6's first point | the relay's keeper, outside this repository | T2's inputs, which it may build against before T2 lands | after landing: a dispatch through the relay at main's tip ends with the run's report, and one past its bound ends as a fault | its code landed outside this repository, 2026-11-19; its runner and token wait |
| T4, bring-up: each failure of T2's first run fixed, or built for Linux alone with its reason in its tree's list, one goal per tree (temper, skein, smith, jig) | 7, steps 2 and 3 | a goal of the draft's per tree | T2 | after landing: the tree's parts green in a run by hand | not started |
| T5, the tiers on: `macos-e2e`, span 3 hours, then `macos-suites`, span 6 hours, its parts' trees once green, in the reference deployment's configuration | 3's first and second points; 7, step 4 | the deployment's operator, at a restart the user approves | T3; T4 for each tree | none: the deployment's own runs | not started |
| T6, live on macOS: the environment `macos-live` and its key, then the relay's daily dispatch of `live`, read beside the live tier's stories | 6 | the user for the key; the relay's keeper for the dispatch | T2, T3 | after landing: the day's live run reports both macOS stories | not started |
| T7, `suites` daily: `macos-suites`' span 24 hours | 3's fourth point | the deployment's operator, at a restart the user approves | T5, a week green | none | not started |

## 2. Order

- **T1 first,** landed: until the one story runs again, nothing on macOS is checked but its build.
- **T2 next,** and **T3 beside it** against T2's inputs; **T4** from T2's first run by hand, the trees in parallel.
- **T5 tier by tier:** `macos-e2e` once `process` and `browser` are green; `macos-suites` with the parts of the trees that are.
- **T6 once the key is in,** after T2 and T3.
- **Section 11's Later items** are no milestone's.
