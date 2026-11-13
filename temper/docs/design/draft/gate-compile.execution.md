# gate-compile: execution

2026-11-13. How [gate-compile.md](gate-compile.md) gets carried out. It designs nothing: each milestone writes its own design from the draft as it runs, and lands whole. Plans use the focused and fuzzy suites alone. The figure of the draft's section 8 is read after each lands, and is no plan's.

## 1. The milestones

| Milestone | Draft parts | Owner | Needs | Its figure after landing | State |
|---|---|---|---|---|---|
| C1, generated code compiles once: the generator writing each generated file's tests as an integration test of its crate, the drift test reading and writing them there, codec crates building no test target of their library, the inventories following, and the check | 3, 7 | temper/dev | none | the own tests of crates holding generated code, against section 2's 70 to 115 s a gate | not started |
| C2, one version's generated code: `temper-protocol-store-codec`'s `v16` and `v17` and `jig-protocol-store-people`'s `people_v16` out, the owners' `schema/` holding the current text alone, their readers moved to the current version or to generic values, and the check with the bridges it names | 4, 7 | temper/dev | none | the temper store's codec crate, against its 36 to 65 s | not started |
| C3, `temper-client-view` split | 5 | temper/dev | none | its row; the gate's build step | not started |
| C4, `temper-client-domain` split | 5 | temper/dev | none | its row | not started |
| C5, `jig-protocol-store` split | 5 | temper/dev | crate-graph's G3 | its row | not started |
| C6, `temper-client-tools` split | 5 | temper/dev | none | its row | not started |
| C7, `jig-client-view` split | 5 | temper/dev | none | its row | not started |

## 2. Order

- **No priority mark:** builds aren't one of v0.1.0's critical pieces. Its plans start when places are free and marked work isn't about to arrive.
- **C1 and C2 first,** side by side. C1 and C2 touch the same generated files, so the second to land merges the first's layout.
- **C3 to C7 one at a time,** each a change of moves between other landings in its crate (`crate-graph.md`, 5.3). The client crates carry v0.1.0's UI with looks: each splits when its crate is quiet. C5 after crate-graph's G3, which moves `jig-protocol-store`'s generated code first.
- **Each milestone removes its exceptions** from the checks it touches, as `crate-graph.execution.md`, 2, says.
