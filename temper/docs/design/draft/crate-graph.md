# crate-graph: a change rebuilds little, and no build waits on a long chain

Draft, 2026-11-12. How fabrig's crates are cut so that a landing recompiles little of the workspace, the builds of one change share one cache whatever packages they select, and no build waits on one chain of large crates. It comes from a session with the user on 2026-11-12 on speeding up fabrig's builds "from a crates organization perspective", from a read of main at `a9e804e340`, from the 794 landings on main in the seven days to `ff0f59e915`, and from each crate's compile time in a from-scratch build of main `9d576e2719`'s libraries on one builder that day. It covers every tree, as `cli.md`'s conventions do (`cli.md`, 3). It finishes what `cli.md`, 3.2, began, and moves generated code toward the codec crates skein's design describes (`skein/docs/design/data/codec/README.md`, 2).

## 1. In one page

- **A crate is the unit of every rebuild** (section 2). Cargo compiles a crate whole, and a shared compiler cache keys each crate by its sources, its settings and the keys of every crate below it. A change to any file of a crate recompiles it and every crate above it.
- **One feature set for every build of a context** (section 3). Once the round in flight lands, no workspace crate has features (`cli.md`, 3.2). External crates still do: a build of one package and a build of the workspace compile `rustls`, `serde` or `bitflags` with different features, and with them every crate above, under keys the other never asks for. Our own rule chooses each external crate's features once, at the root, as the union, and members inherit them. Cargo's unification, on the nightly the workspace moves to, covers the crates below ours. A context is a platform and whether the build has the dev-dependencies.
- **Nightly, with LLVM** (section 2). Cargo's unification needs nightly, and nightly's LLVM takes a sixth less compile than stable's. Cranelift would take a further 30% less, but doesn't yet catch panics, so the workspace waits for it (section 11).
- **Generated code in codec crates** (section 4). Two store crates mix 166,000 generated lines with hand-written translation that depends on domain crates. So the generated lines wait for the domains, and recompile with every hand-written change. The generated record code moves to codec crates that depend on lib and `skein-codec` alone, each large family to a crate of its own. Schemas, kept texts and patches stay with their owner.
- **The crates that change most split along the lines they change** (section 5): `temper-service`, `temper-engine-domain-forge` and `jig-domain-core` first, into pieces that compile side by side where they can, by rules for where to cut.
- **Test targets** (section 6). Measured under Cranelift, a test build compiles five times what its libraries do, mostly crates' own tests and test binaries. A crate's own tests shrink with sections 4 and 5. A module that several test binaries of a package compile becomes a testing crate: `temper-box`'s `tests/support`, 26,500 lines, is compiled five times.
- **Checks hold the shape** (section 7): features chosen once, generated record code only in codec crates, a bound on a crate's size in a test build, and no test source compiled twice. Each is in the boundary suite, with exceptions that only shrink. The figures are the user's.
- **A figure after landing says whether it works** (section 8): the longest chain of a build from scratch, the chain and the compile a landing costs, and the costliest crates.
- **Expected** (section 9), from an optimistic model of the splits: the libraries' longest chain from scratch from 120 s to about 90 s on stable, and from 71 s to 54 s under Cranelift; a recompiling landing's test-build chain under Cranelift from 64 s to 59 s on average; about 6 hours a week less test compile from 6.3, an estimate; and the builds of one change sharing one cache.

## 2. What a build costs today

From the reads named above, on stable Rust 1.95 with LLVM, and then on nightly with Cranelift. The workspace moves to `nightly-2026-10-21` with LLVM (G9 of the execution file). In one trial on one builder, its LLVM took 17% less processor time than stable's for the libraries, and 19% less for two crates' test builds; Cranelift took about 43% less for both, but 151 of the focused suite's 34,696 tests fail under it: it doesn't catch panics, and lacks an SSE2 intrinsic a dependency uses. So stable's figures below are an upper bound for nightly's LLVM, and Cranelift's show the build's shape. A crate's time is rustc's for it, front end and code generation. A chain of full times is an upper bound of what a build waits on its dependencies, since Cargo starts a crate once its dependencies' metadata is written, before their code generation ends.

- **The workspace:** 380 members, 360 with a library. A from-scratch build of the libraries alone compiles for 741 s in all.
- **A build from scratch has one long chain.** It took 148 s from start to end. Its longest chain of full times is 120 s: `skein-lib`, `jig-domain-tasks` (7 s), `jig-domain-core` (22 s), `jig-protocol-store` (40 s), `jig-protocol-engine-clients` (9 s), `jig-protocol-engine-clients-http` (1 s), `jig-protocol-config` (4 s), `jig-client-kit` (15 s), `jig-client-protocol-kit` (7 s), `jig-channel-kit` (4 s) and `temper-one-process-world` (10 s). Pipelining shortens what a build waits on it. More cores don't.
- **A landing:** 412 of the 794 changed no file under a library's directory, its own tests aside: documents, tests, schemas' text. Over all 794, a library build recompiled 88 s a landing on average, and 301 s or more for the tenth slowest. The landings that recompiled anything had chains of full times of 61 s on average, 50 s at the median and 116 s at the tenth slowest. A crate counts as changed when a file under its directory changed. The model leaves out files a crate builds in from elsewhere: 4 of the 412 rebuild through temper's help pages alone (section 11). It also leaves out dependency updates. Each landing is scored against the graph at `ff0f59e915`.
- **Where the compile goes.** Over the week, by crate: its time; the landings that changed its code; the landings that rebuilt it, through its own change or one below it; its share of the week's 19.4 hours of library compile; and the crates above it:

| Crate | Time | Changed | Rebuilt | Share | Above it |
|---|---|---|---|---|---|
| `temper-service` | 17 s | 149 | 264 | 6% | 17 |
| `temper-engine-domain-forge` | 28 s | 83 | 156 | 6% | 27 |
| `jig-protocol-store` | 40 s | 34 | 91 | 5% | 56 |
| `temper-one-process-world` | 10 s | 40 | 341 | 5% | 0 |
| `temper-real-loop-world` | 8 s | 55 | 351 | 4% | 0 |
| `temper-protocol-store` | 17 s | 38 | 164 | 4% | 21 |
| `temper-client-domain` | 16 s | 101 | 141 | 3% | 24 |
| `jig-client-kit` | 15 s | 29 | 144 | 3% | 11 |
| `temper-engine-domain` | 12 s | 107 | 172 | 3% | 19 |
| `jig-domain-core` | 22 s | 66 | 80 | 3% | 70 |

- **On nightly with Cranelift** (`nightly-2026-10-21`, Cargo's `--timings`, main `9d576e2719`, no cache, one loaded builder):
  - **The libraries:** 584 s of compile and 74 s from start to end, so the build is bound by the builder's cores now, not by its chain. The chain of full times is 71 s, or 49 s with metadata pipelining, through the same crates. `jig-protocol-store` takes 22 s, 17 of them its front end; `jig-domain-core` 11 s; `temper-service` 11 s.
  - **The workspace with its tests** (`--all-targets`): 3,042 s of compile and 383 s from start to end, five times the libraries. Each unit's time includes its link. Its largest units are crates' own tests, compiled with the whole crate again: `jig-protocol-store`'s 83 s, `smith-bench`'s 54 s, `jig-client-protocol-wire`'s 51 s and `temper-service`'s 48 s. Next come the worlds' suites, `temper-engine-domain-world`'s 48 s and `jig-core-world`'s 46 s.
  - **A landing's test build of the workspace,** modelled from those units: 472 s of compile on average. For the landings that recompile anything, chains are 64 s on average and 94 s at the tenth slowest (6.1).
- **Generated code:** 440,000 lines in 32 files under `src/generated/`, which use `skein_lib` and `skein_codec` alone. 166,000 of them are in two crates that also hold hand-written code depending on other workspace crates. In `jig-protocol-store`, 116,000 of its 161,000 lines outside its tests are generated, and it depends on 12 other workspace crates, `jig-domain-core` among them. In `temper-protocol-store`, 49,000 of 63,000 are, with 15 others. `jig-client-protocol-wire` is a codec crate as the design wants, wholly generated: 132,000 lines from one schema, 30 s, with 86 crates above it.
- **The same crate under different keys.** On main, `cargo tree` resolves an external crate's features from the packages a build selects:

| A build of | Compiles, unlike a test build of the workspace |
|---|---|
| `-p jig-protocol-store` | `rustls`, `rustls-pki-types`, `rustls-webpki`, `once_cell` and `bitflags` without `std` |
| `-p temper-service` | `serde` and `serde_core` without `alloc`, `serde_json` without `float_roundtrip`, `sha2`, `digest`, `crypto-common` and `bitflags` without `std` |
| `-p skein-tls` | `libc` and the `rustls` crates without `std` |
| the workspace's libraries | `bitflags` without `std`, which the workspace's build with tests has |

  206 of the 380 members, about 450 s of the 741 s, sit above one of these. The delivery check builds with `-p` for the packages a change touches, clippy and the focused tests alike (`tools/pre-pr/design.md`, 4), so a change's builds rarely share their keys with main's.

## 3. One feature set for every build of a context

### 3.1 What still differs

- **Workspace crates:** none has features once the round that turns smith's features and skein's `seam` into fixtures crates lands (`cli.md`, 3.2 and 6). This draft adds nothing there.
- **External crates chosen by our manifests:** `cli.md`, 3.2, chooses each one's features once, in `[workspace.dependencies]`. But a member may still name an external crate by a bare version, `serde_json = "1.0"`, which takes the crate's defaults rather than the root's choice. The root names `rustls` twice, as `rustls` without `std` and as `rustls-std`, which are one package to Cargo. A native build that reaches only the first compiles it without `std`. And the root's entries ask for less than the workspace ends up resolving: `serde` without `alloc`, `sha2` without `std`.
- **External crates below them:** `bitflags`, `once_cell`, `libc` and others get their features from the external crates that use them, which no manifest of ours names.
- **So** a crate's features follow what a build selects: `-p` of one package, `--workspace`, a build without the dev-dependencies (section 2's table). Raising the root's entries to the union alone clears `serde_json` and `sha2`, with `digest` and `crypto-common` below it, from `-p temper-service`'s differences. It leaves `serde` while a member names it by version, the `rustls` crates while the root names two, and `bitflags` and `once_cell` (checked on main, 2026-11-12).

### 3.2 The rule

**Every build of one context compiles each external crate with one feature set, whatever packages it selects:** `cargo build`, `cargo test`, `cargo clippy --all-targets` and `cargo nextest run`, with `-p` of any packages or `--workspace`.

- **A context** is a platform and a mode. The platforms are the native target, `wasm32-unknown-unknown` and the host, for build scripts and procedural macros. The modes are with the dev-dependencies (tests, `clippy --all-targets`, nextest) and without (libraries and binaries alone).
- **The two modes differ only below a dev-dependency** that adds features to a crate further down, as `bitflags`'s `std`. Landings build with the dev-dependencies, so that mode is the one their cache keys share.
- **The browser's nested wasm32 build** is a context of its own (3.4).

### 3.3 Our own: no features, and one choice for each external crate

The user's plan: no features anywhere, and an external crate that needs features gets the same ones everywhere.

- **No workspace crate has features** (`cli.md`, 3.2).
- **Every external dependency is inherited:** a member names an external crate only as `NAME.workspace = true`, choosing nothing, in every dependency table. No bare version, no `features`, `default-features` or `optional`.
- **The root carries the union:** each root entry asks for every feature that a build with the dev-dependencies resolves for its crate on its platform. So the entry says what every build compiles, and the two modes agree on every crate a member names.
- **A second entry only for another platform:** where a platform needs a crate otherwise, as `wasm32-unknown-unknown` needs `rustls` without `std`, the root names it a second time. That second entry is named only in that platform's table of each member that builds for it, and every native use of the crate is gated to native targets. Today's `rustls` and `rustls-std` are such a pair once their unconditional users are gated, `temper-forge-setup-shell` among them.

### 3.4 Cargo's, for the crates below ours

- **Two settings** in the workspace's `.cargo/config.toml`, on the nightly the workspace moves to: `[unstable] feature-unification = true` and `[resolver] feature-unification = "workspace"`. Every build then resolves each external crate as if it selected every member. That covers the crates below ours, which 3.3 can't reach. With them, on `nightly-2026-10-21`, `cargo tree -p jig-protocol-store` resolves `rustls` and `bitflags` with `std`, as the workspace's build does (checked 2026-11-12).
- **It unifies within a context, not across them:** Cargo keeps the host's resolution apart from the target's, and leaves the dev-dependencies' features out of builds without them. So the modes still differ below a dev-dependency (3.2).
- **The browser's nested wasm32 build keeps its own selection,** with `CARGO_RESOLVER_FEATURE_UNIFICATION=selected` in its environment, beside the code generator it keeps. A union over every member reaches it: on that nightly it gives `once_cell` `std` and `web-sys` six more features there. A crate whose `std` breaks on `wasm32-unknown-unknown` would break with it, as `rustls`'s would. The nested build always selects the same package, so it is a context of its own and stable.
- **Both layers stay.** 3.3 holds without the settings, for every crate our manifests name, and keeps the root the one place features are chosen. The settings add the crates below.

### 3.5 The check

In the boundary suite (section 7), building nothing:

- every member's external dependency is inherited, choosing nothing, and every use of a platform's second entry is gated to that platform;
- for a fixed set of contexts, each read by one `cargo tree --offline --locked` of the workspace, each root entry asks for every feature its crate resolves to there;
- the two settings are in `.cargo/config.toml`, and the bundle's nested build sets `selected`.

## 4. Generated code in codec crates

### 4.1 The rule

**A codec crate holds a family's generated record code, and depends on lib and `skein-codec` alone.** Hand-written code that translates the records to and from a domain lives in the crate above, its owner, which depends on the codec crates and the domains. The generated code then compiles beside the domain crates, not after them, and recompiles only when a schema changes.

**The rest stays with the owner:** the schemas, the kept texts, the patches and their context, the generated glue, the golden bytes and the drift tests. A store's patches may name its domains' types (`jig/crates/jig-protocol-store/patches/context.rs`), and one patch may move several families under one owner (`jig/crates/jig-protocol-store/patches/0001.patch`). So they can't go to codec crates that know no domain. This keeps less in a codec crate than skein's design does (`skein/docs/design/data/codec/README.md`, 2).

### 4.2 What departs

Every crate that holds `src/generated/` and depends on more than lib and `skein-codec`, by generated lines (outside tests) and other workspace dependencies:

| Crate | Generated | Of its lines | Other workspace dependencies |
|---|---|---|---|
| `jig-protocol-store` | 116,457 | 160,809 | 12, `jig-domain-core` among them |
| `temper-protocol-store` | 49,429 | 63,333 | 15 |
| `jig-client-test-protocol-wire` | 12,629 | 14,327 | 1 |
| `jig-test-store` | 9,454 | 11,142 | 8 |
| `jig-protocol-engine-clients` | 4,342 | 21,035 | 13 |
| `smith-local-protocol` | 2,336 | 7,758 | 8 |
| `jig-client-view` | 1,397 | 38,466 | 2 |
| `skein-ui-example-view` | 724 | 1,767 | 2 |
| `skein-oauth` | 498 | 6,223 | 2 |
| `jig-protocol-accounts` | 346 | 4,711 | 8 |

`jig-protocol-worker-wire` and `smith-channel` depend on `skein-channel` too, which the rule allows, with the reason, or moves.

### 4.3 One large family, one crate

- **A family whose generated code passes a size gets a codec crate of its own.** Then a change to one family recompiles that family alone, and the families compile side by side. `jig-protocol-store`'s `people` (40,000 lines) and `tasks` (39,000) are each the size of a large crate, `core` (15,000) and `settings` (13,000) a middling one, and its four small families stay together. The size is a figure (7.3).
- **Names** follow the tree's: the store's families as `jig-protocol-store-<family>`, or one `jig-protocol-store-families` for the small ones. The milestone settles them.

### 4.4 What moves

- **To the codec crate:** each family's `src/generated/<family>_v<N>.rs`. The codec command learns to write a family's generated code there, while the owner's files and paths, its baseline and its patches stay where they are (`skein/docs/design/data/codec/command.md`, 1).
- **Stays with the owner:** everything 4.1 names, the generated glue among it (`src/generated/layouts.rs`, `patches/mod.rs`); the translation between records and domain; and the families' bounds derived from the domain's limits.
- **The inventories follow:** the drift and golden tests that list each family's generated file, inside and outside the owner (`jig/tests/codec-families.rs`, `jig/tests/store-codecs/`, `temper/tests/codecs/`), read the codec crates' paths in the same change.
- **Callers** name the codec crate's paths, or the owner re-exports them for a time, as section 5.3 says.

### 4.5 The client wire

`jig-client-protocol-wire` already holds to the rule. Once sections 4 and 5 shorten the chains through the store and the domains, it heads the longest chain of a build from scratch (section 9): 30 s for one schema of 1,827 types, about 72 lines of Rust a type. A schema is one crate, so the lever is the generator's output, or a wire in parts (section 11).

## 5. The crates that change most

### 5.1 Which

Ranked by compile a week (section 2's table) and by their place on the longest chains:

1. **`temper-service`:** 35,000 hand-written lines and 49,000 more in its own tests; 17 s; changed by 149 landings in the week, a fifth of them; on the chain of every temper engine change. Its own tests end most test-build chains (6.1). Its worker's half becomes crates of its own in the round in flight (`cli.md`, 6).
2. **`temper-engine-domain-forge`:** 39,000 lines and 35,000 in tests; 28 s; changed by 83; on the longest chain once section 4 lands.
3. **`jig-domain-core`:** 59,000 lines and 40,000 in tests; 22 s; changed by 66, with 70 crates above it; on the longest chain today and after section 4.

Next, by the same measure, once these land: `temper-client-domain` (38,000 lines, changed by 101), `temper-engine-domain` (33,000, by 107), `temper-client-view` (45,000, by 126) and, at the top of most chains, the test harnesses `jig-client-kit` (15 s) and `temper-one-process-world` (10 s).

### 5.2 Where to cut

The milestone that splits a crate draws its lines by these rules, from the crate's module graph and its history:

- **What changes together stays together.** The pairs of files that the same landings changed, over the weeks before the split, group the modules. A script reads them from git.
- **Stable below, changing above.** What many modules use and rarely changes, the crate's vocabulary and its types, goes down to a crate of its own. What changes often goes up, with few crates above it.
- **Side by side where possible.** Two pieces neither of which uses the other compile together, and that shortens the chain. A cut that leaves a chain still saves each change the pieces below it, but not the chain.
- **A cut must build.** A type's inherent methods stay in the crate that defines it, a piece reaches another only through its public interface, and the pieces depend on each other without a cycle. A domain's state machine, its type and the methods every module adds to it, may not be cut at all. Then the split takes what lies around it.
- **A piece is no smaller than it needs to be.** Each crate costs a manifest, a lint profile and a link. A piece is worth its crate when it saves the landings that don't touch it a recompile of several seconds.
- **The layering stands:** a step crate stays in its tree's `crates/`, in the strict subset (`AGENTS.md`, 2). Its own tests go with the code they test, as step tests (`skein/docs/foundation/testing-strategy.md`, 2.1). The trees' directions hold (`tests/boundary/tests/direction.rs`).

### 5.3 Landing a split

- **Preparation first, where the cut needs it:** a type's methods gathered, a private item made the piece's interface. Each lands as its own change, behaviour unchanged, before the move.
- **Then one change per crate split, mechanical:** files moved and not edited in the same change, so that git's rename detection carries branches in flight across it, and no behaviour changed.
- **The old crate re-exports for a time.** It keeps its name and depends on the pieces, so callers build unchanged through its public paths. Callers then move to the pieces they use, and the old crate goes when none names it. Until they move, a caller still rebuilds with any piece.
- **Out of the way of plans in flight in the crate,** as `cli.execution.md`'s moves do (`cli.execution.md`, 2): a split lands between other landings in the crate, and marked work goes first.

### 5.4 What the history shows

The modules whose files the week's landings changed most, as the milestones' starting point, not their decision:

- **`jig-domain-core`:** `routing` (18,900 lines, changed by 50 landings), then `documents`, `reads`, `settings` and `runs` (3,700 to 7,800 lines, 20 to 23 each), and `vocabulary` (1,500 lines, 23 landings), which most of the crate likely uses. `routing` adds methods to `Core`, the crate's state, so it can't leave as it stands (5.2).
- **`temper-service`:** `stock` (1,100 lines, 57 landings), `limits` (2,200, 45), `startup` (6,600, 40), `family` (3,000, 32), `route` (1,100, 28) and `clients` (4,800, 27).
- **`temper-engine-domain-forge`:** `domain` (10,500 lines, 58 landings), `boundary` (2,600, 41), `brief`, `landing`, `views` and `settings` (900 to 2,600, 13 to 19 each).

## 6. Test targets

### 6.1 Where a test build's time goes

Under Cranelift (section 2), a test build of the workspace compiles crates' own tests for 1,377 s, integration tests for 878 s, and libraries for 710 s. Modelled over the week's 794 landings, test builds compile for 104 hours, 83% of it in test targets. The 417 landings that recompile anything have test-build chains of 64 s on average and 94 s at the tenth slowest. 200 of those chains end in `temper-service`'s own tests (48 s), 65 in `jig-protocol-store`'s (83 s), 57 in `smith-bench`'s (54 s) and 42 in `temper-box`'s test binaries. The model compiles a crate's own tests beside its library and its other test binaries after it, and a package's binaries side by side.

### 6.2 A crate's own tests compile the crate again

- **A test build compiles a library twice:** once as the library its dependents use, and again with `cfg(test)` and its own tests, as one test binary. A change to the crate or to its tests recompiles the second whole.
- **So in a test build, a crate's size is its hand-written code and its own tests together,** which 7.3 bounds. Sections 4 and 5 cut it where it costs most. `jig-protocol-store`'s 116,000 generated lines leave its own tests for the codec crates' tests, which only a change of schema recompiles. Each split of section 5 halves a crate's own tests with it.
- **Step tests stay step tests,** in the crate they test (`skein/docs/foundation/testing-strategy.md`, 2.1).

### 6.3 Shared test code is a crate

- **A module that several test binaries of a package compile goes to a testing crate** in its tree's `testing/` (`AGENTS.md`, 1), which the binaries depend on. It is compiled once and cached, and each binary compiles only its own lines.
- **`temper-box` today:** five of its six test binaries compile its `tests/support`, 26,500 lines, so it compiles 149,000 lines of tests for 43,000 distinct. Three of the binaries hold 187 lines of tests between them and take 53 s. Its test binaries were rebuilt about 361 times in the week, 12.7 hours, 12% of the week's test compile.
- **An estimate, before measuring:** from those units' times, a rebuild would cost about 60 s less, about 6 hours a week. But the support crate then compiles before the binaries that use it, which may lengthen their chain. The milestone measures both.
- **What the move takes:** the crate gets a public interface for what the binaries reach today through their parent module (`write_private` among them). Its paths through `crate::support` and the manifest directory it assumes are made its own. Its helpers under `cfg(test)`, which a dependency is never built with, become plain or stay with the binaries. The binary's path (`CARGO_BIN_EXE_temper-box`), which only the test package is given, is passed in.
- **Smaller cases:** `smith`'s tests compile 6,000 lines again, `skein-browser-world`'s 1,600, `jig-test`'s 900 and `temper-real-loop-world`'s 11.

### 6.4 Binaries and the tiers

The tiers select tests by binary and, within a binary, by test path (`.config/nextest.toml`): `live` and `process` by binary, `proof_live`'s `rung_b0s_` tests and `live_github_managed`'s `managed_ready_client` by path. Nothing here requires a binary per tier, and nothing merges them. Merging binaries is for the tiers' owner to choose, and 6.3 removes most of what makes extra binaries costly.

### 6.5 Test crates

Section 5.2's rules hold for test crates too. The worlds' suites, `temper-engine-domain-world`'s (48 s) and `jig-core-world`'s (46 s), are leaves that end few chains. They cost every landing that rebuilds them, 5.5 hours in the week for the first. They split only where their history says they change in parts.

## 7. The checks

Each runs in the focused suite, in `tests/boundary/`, building nothing. Each reads the manifests, `cargo metadata`, the `cargo tree --offline --locked` runs of 3.5 or the sources, the last by fixed rules: `mod`, `#[path]` and `include!`. Each carries exceptions with their reasons, which only shrink, as `cli.md`, 3.8, does.

### 7.1 Features

Inherited external dependencies, a platform's second entries gated, root entries carrying the union in every context, and Cargo's settings (3.5).

### 7.2 Codec crates

A package holding generated record code, `src/generated/<family>_v<N>.rs`, has no normal dependency on a workspace crate but `skein-lib`, `skein-codec` and those the check names with a reason. Its dev-dependencies are free, `skein-codegen` for its tests among them. Generated glue (4.4) may sit with its owner. Section 4.2's crates are its exceptions until their milestones.

### 7.3 Sizes

- **A library's hand-written code with its own tests,** the Rust a test build of it compiles outside generated record code: at most 60,000 lines (6.2). Eight crates are over today, each an exception until its split: `temper-client-view` (110,000), `jig-domain-core` (100,000), `temper-engine-service` (66,000) and `temper-client-domain` (83,000), `temper-engine-domain-forge` (73,000), `jig-protocol-store` (69,000), `temper-client-tools` (67,000) and `jig-client-view` (64,000).
- **A family's generated code** in a codec crate shared with other families: at most 20,000 lines. Past that, it gets a crate of its own (4.3). A codec crate with one family has no bound, as one schema is one crate (4.5).
- **The figures are the user's:** 60,000 and 20,000 are proposals. Lines stand in for compile time, which a check can't read cheaply. Section 8's figure tells whether they stand in well.

### 7.4 Shared test code

No source file is compiled by two test binaries of one package, read from each test target's module tree (6.3).

## 8. The figure after landing

- **A tool, `tools/crates/`,** reads a timed build of main from scratch and the history, and prints:
  - the longest chain of the build, by each crate's time to its metadata and to its end, from Cargo's `--timings`;
  - over a fixed window of landings, the last seven days' first-parent commits to a named main, each one's recompile and its chain, the mean, the median and the tenth slowest;
  - the ten crates costing the most compile in the week, as section 2's table.
- **It builds the shapes landings build:** the libraries, and the workspace with its tests and dev-dependencies (`--all-targets`).
- **It runs after landing,** on a builder and by whoever runs the builders, once a day and after each milestone of this draft. No plan runs it (`AGENTS.md`, 5). Its first figure, before any milestone lands, is the baseline the others are read against.
- **Targets are the user's** (section 11).

## 9. What to expect

Modelled from section 2's figures, as an optimistic scenario until the cuts are drawn. Each split crate becomes two halves side by side, each keeping all the old crate's dependencies, and every landing changes one half. A test build recompiles each library above a change with its tests and binaries, and the tests of each package that uses one of them in its tests.

| | Today | Sections 4 and 5 |
|---|---|---|
| Stable, LLVM: the libraries' longest chain of full times from scratch | 120 s | 89 s |
| Stable: a recompiling landing's library chain, mean and tenth slowest | 61 s, 116 s | 47 s, 83 s |
| Stable: a landing's library compile, mean of all | 88 s | 83 s |
| Cranelift: the libraries' longest chain from scratch, full and pipelined | 71 s, 49 s | 54 s, 35 s |
| Cranelift: a landing's test build, compile, mean of all | 472 s | 460 s, and about 30 s less with 6.3 (an estimate) |
| Cranelift: a recompiling landing's test-build chain, mean and tenth slowest | 64 s, 94 s | 59 s, 93 s |

- **The chains fall more than the compile.** The splits spare a landing the halves it doesn't touch, a few seconds each. What they cut is the waiting: the store's families beside the domains, two halves of a crate side by side.
- **The test build gains less.** Its chains end in the large crates' own tests, which sections 4 and 5 shrink. `jig-protocol-store`'s 83 s unit loses about two thirds of its lines for each hand-written change. Its slowest landings gain little. 6.3 cuts `temper-box`'s compile, and its effect on the chain is unmeasured.
- **Section 3's gain isn't modelled.** It makes a change's builds of one context share their keys with main's and with each other's, whatever they select: a cache hit for each of the 206 members above an unpinned crate that the change didn't touch.
- **After them,** the libraries' longest chain from scratch runs through `jig-client-protocol-wire`, `jig-client-view`, `jig-client-kit` and `temper-one-process-world` (4.5 and 5.1's next).

## 10. Who carries what

| Part | Who |
|---|---|
| The workspace on nightly with LLVM, the macOS workflows following its pin (2, 3.4), telling whoever runs the builders | temper/migration |
| Inherited external dependencies, the root's unions, gated platform entries, Cargo's settings and the check (3, 7.1) | temper/migration |
| The store crates' generated record code in codec crates, the codec command writing into them, the inventories, and the check (4, 7.2) | temper/migration |
| The other codec crates of 4.2, each removing its exception | temper/migration, as each crate's area plans it |
| The splits of 5.1, one milestone each | temper/migration |
| Shared test code in testing crates and its check (6.3, 7.4), telling the tiers' owner | temper/migration |
| The size check (7.3) and the figure's tool (8) | temper/migration |
| Running the figure after landing (8) | whoever runs the builders |

## 11. Later and open questions

- **Open, the user's:** the sizes of 7.3, 60,000 lines of a library's hand-written code and its own tests, and 20,000 generated lines a shared family.
- **Open, the user's:** the targets of section 8, proposed as the longest chain from scratch under 90 s and a landing's mean chain under 50 s.
- **Later: Cranelift,** once it catches panics and has the intrinsic (section 2). The trial's figures under it, in sections 2, 6 and 9, hold only with it.
- **Later: back on stable,** if the workspace returns to it, the crates below ours drift again (3.4). A features crate, as cargo-hakari writes, would pin them: one package every member above them depends on, naming each with the workspace's union.
- **Later: the client wire** (4.5): the generator's output per type, measured against its compile time, or the schema in parts, each its own crate.
- **Later: embedded files.** `temper-docs` builds in 68 of temper's help pages and an example from outside its directory (`temper/crates/temper-docs/src/lib.rs`). 132 landings in the week changed one. 114 of those recompiled more for it, about 17 s each, and 4 rebuilt through those pages alone.
- **Later: binaries per package** (6.4): merging `temper-box`'s live and process binaries, with the tiers' filters moved to test paths.
- **Later: links.** The test units' times in section 2 include their links, but how much of each is the linker isn't measured.
