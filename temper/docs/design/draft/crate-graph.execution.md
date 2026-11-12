# crate-graph: execution

2026-11-12. How [crate-graph.md](crate-graph.md) gets carried out. It designs nothing: each milestone writes its own design from the draft as it runs, and lands whole. Plans use the focused and fuzzy suites alone. The figure of the draft's section 8 is read after each lands, and is no plan's.

## 1. The milestones

| Milestone | Draft parts | Owner | Needs | Its figure after landing | State |
|---|---|---|---|---|---|
| G1, one feature set for every build of a context: every external dependency inherited from the root, root entries carrying the union, a platform's second entries gated (`rustls-std`'s users first), Cargo's unification in `.cargo/config.toml` with the bundle's nested build kept to its selection, and the check over its fixed contexts | 3, 7.1 | temper/migration | the round that turns smith's features and skein's `seam` into fixtures crates (goal 1409), so that no workspace crate has features; G9 | `cargo tree` of a `-p` build and of the workspace's build with tests resolve each external crate alike; the cache's hits for a change's builds | not started |
| G2, the store crates' generated record code in codec crates: `jig-protocol-store`'s families, a large one a crate (7.3's figure), and `temper-protocol-store`'s; the codec command writing a family's generated code into its codec crate while schemas, kept texts, patches and baselines stay with the owner; the drift and golden inventories reading the new paths; the check, with the other crates of `crate-graph.md`, 4.2, as its exceptions | 4.1, 4.3, 4.4, 7.2 | temper/migration | G7's baseline | the longest chain from scratch; the two crates' rows | landed: `jig-protocol-store`'s families in five codec crates (jig's `protocol/README.md`, section 5) and `temper-protocol-store`'s current version in `temper-protocol-store-codec` (temper's `protocol/store.md`, section 4), the drift tests writing there, and the codec check (`tools/crates/design.md`, 3.2); its figure is read on the builders by whoever runs them |
| G3, the other crates of `crate-graph.md`, 4.2, each removing its exception, `jig-test-store` and `jig-protocol-engine-clients` first | 4.2 | temper/migration | G2 | none of its own | not started |
| G4, `temper-engine-service` split by `crate-graph.md`, 5.2's rules, its preparatory changes first | 5 | temper/migration | its worker's crates from the round in flight (`cli.md`, 6; goal 1409); G7's tool | its row; landings' chains | not started |
| G5, `temper-engine-domain-forge` split | 5 | temper/migration | G7's tool | its row; the longest chain from scratch | not started |
| G6, `jig-domain-core` split | 5 | temper/migration | G7's tool | its row; the longest chain from scratch | not started |
| G7, the sizes' check and the figure's tool | 7.3, 8 | temper/migration | the user's figures (`crate-graph.md`, 11), or the proposals until they come; G9 landed, before its first figure | the first figure, today's numbers for every later one | landed, on the proposals, which the user kept on 2026-11-12 (`tools/crates/design.md`, 1); its first figure, the baseline, waits for whoever runs the builders (`tools/crates/design.md`, 4.7) |
| G8, shared test code in testing crates: `temper-box`'s `tests/support` first, with its public interface, its own paths, its `cfg(test)` helpers made plain or kept with the binaries, and the binary's path passed in; then `smith`'s, `skein-browser-world`'s, `jig-test`'s and `temper-real-loop-world`'s; the check | 6.3, 7.4 | temper/migration, telling temper/tiers | G7's baseline | `temper-box`'s test units' times and its chain, against 6.3's estimate | landed, designed in `tools/crates/design.md`, 2.4 and 3.2, with two exceptions its survey found; its figure waits for whoever runs the builders (`tools/crates/design.md`, 4.7) |
| G9, the workspace on `nightly-2026-10-21` with LLVM: first, nightly clippy's three `chunks_exact_to_as_chunks` findings in `skein-lib`'s `crypto_rng.rs` fixed; then a `rust-toolchain.toml` naming it, with clippy, rustfmt and the `wasm32-unknown-unknown` target and no other backend; the macOS workflows installing the pinned toolchain in place of their `RUST_VERSION` | 2, 3.4 | temper/migration, telling whoever runs the builders, whose default follows the pin | none: the builders have the toolchain | the libraries' compile from scratch, against section 2's stable figure; the shared cache starts cold once | landed, goal 1914; its figure is read after landing |

## 2. Order

- **No priority mark:** builds aren't one of v0.1.0's critical pieces. Its plans start when places are free and marked work isn't about to arrive.
- **G9 first,** as soon as a place is free: it is small, and G1's unification needs it.
- **G7 next:** its tool's first figure, read after G9 lands, is the baseline every later milestone is read against, on one toolchain.
- **Then G1, G2 and G8,** side by side. G1 also waits for the round in flight to land.
- **G4, G5 and G6 after G7,** one at a time. Each lands as one change of moves, between other landings in its crate (`crate-graph.md`, 5.3). G6 first if its crate is quiet, since `jig-domain-core` is on the longest chain today and after G2.
- **G3 last,** each crate as its area next works there.
- **Each milestone removes its exceptions** from the checks it touches. One left matching nothing fails its check, as `cli.md`, 3.8's do.
