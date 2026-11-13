# gate-compile: execution

2026-11-13. How [gate-compile.md](gate-compile.md) gets carried out. It designs nothing: each milestone writes its own design from the draft as it runs, and lands whole. Plans use the focused and fuzzy suites alone. The figure of the draft's section 8 is read after each lands, and is no plan's.

## 1. The milestones

| Milestone | Draft parts | Owner | Needs | Its figure after landing | State |
|---|---|---|---|---|---|
| C1, generated code compiles once: an inventory of each codec crate's library tests; the generator writing a file's tests as their own output by the records' public path; the drift test writing, reading and checking both outputs; each codec crate's `tests/generated/main.rs`; hand-written tests that use public items moved there; `[lib] test = false` where nothing else is tested; the inventories following; and the check, with an exception for each crate that keeps library tests | 3, 7 | temper/dev | none; `jig-client-view`'s records after crate-graph's G3 moves them to a codec crate, or with a public path | the store codec crates' own tests against their 15 to 21 s in `gate-compile.md`, 2; the build's compile | landed: each codec crate's generated tests are its `tests/generated` target's, written by its owner's drift test (skein's `data/codec/README.md`, section 6), its library building no test target where it holds no other tests; the generated-tests check, with the crates that keep library tests as its exceptions (`tools/crates/design.md`, 3.4); its figure is read on the builders by whoever runs them |
| C2, one version's generated code: `temper-protocol-store-codec`'s `v16` and `v17`, `jig-protocol-store-people`'s `people_v16` and `smith-transcript`'s `v6` out; the owners' `schema/` holding the current text alone; temper's refusals and `temper-fake`'s check reading the current version; jig's `p0001.rs` and temper's store tests reading older records as generic values of their kept texts; jig's family inventory following; and the check, with skein's codec fixtures as its exception | 4, 7 | temper/dev | none | the temper store's codec crate against its 15.5 s library | not started |
| C3, `temper-client-view` split, its preparatory changes first | 5 | temper/dev | none | its row against `gate-compile.md`, 5; the gate's build step | not started |
| C4, `temper-client-domain` split, its pages reaching the root `Domain` through an interface first | 5 | temper/dev | C3's figure | its row | not started |
| C5, `jig-protocol-store` split, its preparatory changes first | 5 | temper/dev | C3's figure | its row | not started |
| C6, `temper-client-tools` split | 5 | temper/dev | C3's figure | its row | not started |
| C7, `jig-client-view` split | 5 | temper/dev | C3's figure | its row | not started |

## 2. Order

- **No priority mark:** builds aren't one of v0.1.0's critical pieces. Its plans start when places are free and marked work isn't about to arrive.
- **C1 and C2 first,** side by side. They touch the same generated files, so the second to land merges the first's layout; neither needs the other.
- **C3 next, then its figure:** if its crate's pieces were reused in the builds after it, C4 to C7 follow, one at a time; if not, the user hears before the next starts. Each lands as a change of moves between other landings in its crate (`crate-graph.md`, 5.3). The client crates carry v0.1.0's UI with looks: each splits when its crate is quiet.
- **Each milestone removes its exceptions** from the checks it touches, as `crate-graph.execution.md`, 2, says.
