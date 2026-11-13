# gate-compile: a gate compiles each change once, and little else

Draft, 2026-11-13. What to change next so that temper's CI gate compiles less for each landing, now that `crate-graph.md`'s milestones have landed but G3, which is in flight. It comes from a measure the user asked for that day: the gate's own step times for every full gate since the CI began on 2026-11-11, and the shared compiler cache's record of each unit the gate compiled in its three complete builds from 06:35Z to 10:46Z on 2026-11-13 (gates 570, 574 and 577). It leaves out what the cache does wrong in operation, which goes to whoever runs the builders (section 11).

## 1. In one page

- **The gate got faster, and the splits cut their largest units** (section 2). Its build step, every test target of the workspace, went from a median of 196 s to 147 s, with Cranelift and a quieter builder mixed in. Against units no milestone touched, the split crates' largest units are about half or less of what they were. Their totals fell where a build reused some pieces, and rose a little where it rebuilt them all.
- **A gate compiles 720 to 750 s,** recorded by the cache in a build step of 142 to 156 s: 324 to 444 s of it crates' own tests. Whether the builder's cores or a chain of crates limits the build isn't measured. This draft cuts what it compiles.
- **Generated code compiles once** (section 3). Each generated file carries its own golden and bound tests, so a codec crate's own tests compile all its generated code a second time. The generator writes those tests as a target of their own, and a codec crate whose library then holds no other tests builds no test target of it.
- **One version's generated code** (section 4). The codec design already keeps a family's current version's code alone. The code holds older versions: the temper store's codec crate holds three, about 3.5 MB each, and recompiles all three at each of the store's version moves, five in about 20 hours.
- **The crates over the size bound split** (section 5), one at a time, the first measured before the next: a split saves only in builds that reuse some of its pieces.
- **No ranking of the next cuts for now** (section 6), by the user's word.
- **Expected** (section 9): sections 3 and 4 together save about 30 to 45 s of compile in a build that rebuilds the store codec crates, as each build of the sample did. The splits' gain is read after the first. The build step's figure is read after landing, not forecast.

## 2. What a gate compiles today

The gate builds every test target of the workspace (`cargo nextest run --no-run`), then runs the focused and fuzzy suites, on one CI builder with 8 cores, sharing the compiler cache with the builders that run goals. Each full gate prints its steps' times.

| Window | Full gates | Build step, median | Mean |
|---|---|---|---|
| 2026-11-12 08:04Z to 15:19Z: stable Rust with LLVM, no milestone landed | 19 | 196 s | 198 s |
| 2026-11-13 03:19Z to 10:46Z: nightly with Cranelift, the milestones but G3 (G5 landing at 04:39Z, G4 at 07:11Z), other builds paused during a gate | 14 | 147 s | 131 s |

- **What moved it is mixed.** Cranelift compiled 43% less in its trial (`crate-graph.md`, 2), and pausing other builds removes contention. The milestones' own share can't be read from these two rows.
- **Per unit, roughly.** Units no milestone touched compile on the CI builder in 0.40 to 0.50 of their time in `crate-graph.md`, 2's trial under Cranelift, about 0.45: `jig-client-protocol-wire`'s own tests 51.4 s then, 22.4 s now; `temper-client-view`'s 36.4 and 18.0; `temper-client-domain`'s 28.6 and 13.7; `temper-engine-domain-world`'s suite 47.6 and 19.0. Their sources changed between the two reads too, so the ratio is a rough yardstick. Against it, each split crate's old units and what replaced them, in the three complete builds:

| Old units | Trial | At 0.45 | What replaced them, per build |
|---|---|---|---|
| `jig-protocol-store`'s own tests | 82.6 s | 37 s | its own tests and its five codec crates': 31.4, 9.6 and 12.1 s; its largest unit 10.9 s |
| `temper-service`'s own tests | 48.2 s | 22 s | the six service crates' own tests: 26.7, 25.7 and 24.9 s; the largest, `temper-engine-service-component`'s, 10.7 s |
| `temper-box`'s six test binaries | 122.2 s | 55 s | its six binaries and their shared kit: 30.5, 29.3 and 32.6 s; the largest binary 6.8 s |
| `temper-engine-domain-forge`'s own tests | 34.5 s | 15.5 s | with its vocabulary's: 12.3, 14.3 and 13.8 s |
| `jig-domain-core`'s own tests | 26.1 s | 11.7 s | with its vocabulary's: 14.0, 9.4 and 11.3 s |

- **So a split shrinks its largest unit, and saves in builds that reuse some of its pieces.** `jig-protocol-store`'s pieces cost a quarter to a third of the old unit when its codec crates were reused, and most of it when they weren't. The service's pieces cost a little more than the old unit, rebuilt together. Sharing `temper-box`'s test code (`crate-graph.md`, 6.3) saved about 45%.
- **What the three complete builds compiled,** classified from Cargo's targets:

| Gate | All | Libraries | Own tests | Integration tests | Other |
|---|---|---|---|---|---|
| 570 | 730.5 s | 96.8 s | 444.1 s | 180.7 s | 8.8 s |
| 574 | 748.5 s | 237.9 s | 324.4 s | 177.6 s | 8.7 s |
| 577 | 723.5 s | 158.2 s | 371.2 s | 183.8 s | 10.4 s |

- **Their builds' last compiles ended 141.5, 156.3 and 154.1 s after each gate began,** with 6.6 to 7.1 units in flight on average, the cache's own work among them, on 8 cores. That's busy, but not a measure of the cores' use or of a chain.
- **The temper store's codec crate** recompiled its library in 15.6 and 15.5 s in gates 574 and 577, and its own tests in 15.2, 21.3 and 21.0 s in all three. Gate 570's head held two versions, the others three (section 4).
- **Rebuilds of unchanged units:** in gates 574 and 577, 71 and 57 units missed the cache with a key an earlier captured build had used, 201 and 211 s of compile. Gate 570's predecessors weren't captured. That is the cache's operation, outside this draft (section 11).

## 3. Generated code compiles once

### 3.1 What it costs

- **Each generated file ends with its tests:** a `golden_tests` module under `cfg(test)`, its golden values and its bounds in tables of up to 1,024 cases (skein's `data/codec/README.md`, section 6). Across the workspace's 39 such modules they use the file's public items alone, with helpers of their own.
- **So a codec crate's own tests compile its generated code again,** as `crate-graph.md`, 6.2, describes for every crate. The own tests of crates holding generated code took 90.0, 70.3 and 78.1 s in the three builds. Of the temper store's codec crate's 15 to 21 s, its library's compile is about 15.5 s.

### 3.2 The rule

- **The generator writes a file's tests as an output of their own,** apart from its record code, naming the records by the public path it is given. The drift test (skein's `data/codec/README.md`, section 6) writes, reads and checks both outputs, and the inventories list both (`crate-graph.md`, 4.4).
- **A codec crate has one integration test target for them,** `tests/generated/main.rs`, a module for each generated file, written with them.
- **A codec crate builds no test target of its library** (`[lib] test = false`) once nothing else is tested there. Its hand-written tests move to the integration target where they use public items. A crate whose remaining tests need private items keeps its library tests, and gains only from smaller tables. Today the two client wires, the two worker wires, `smith-charter`, `smith-channel`, `smith-transcript` and `temper-protocol-records`' patch module hold such tests. The temper store's codec crate and jig's five store codec crates hold none.
- **A generated module behind a private path,** such as `jig-client-view`'s records, takes a public one first, or follows G3, which moves it to a codec crate.
- **Generated tests are integration tests, by exception.** A crate's step tests stay in it (`skein/docs/foundation/testing-strategy.md`, 2.1; `crate-graph.md`, 6.2). The generator's tests check its output through the public items the crate's users see, so they test the crate from outside.

## 4. One version's generated code

### 4.1 What the code holds

- **The design keeps one version's code.** A family's generated code reads its current version only. An older version's record is read as a generic value and patched to the current version before the generated code reads it (skein's `data/codec/README.md`, section 5; `temper/docs/design/protocol/store.md`, section 4). A kept version is its text, in `schema/kept/` (`data/codec/command.md`, section 1).
- **The code holds more:** `temper-protocol-store-codec` holds `v16`, `v17` and `v18`, about 3.5 MB each, and `temper-protocol-store/schema/` holds texts from `v14` to `v18`. `jig-protocol-store-people` holds `people_v16` beside `people_v17`, and `smith-transcript` holds `v6` beside `v7`.
- **Their readers:**
  - `temper-protocol-store`'s own tests: `catch_up.rs`'s `v16` closer can read the current version, and its `v16` landing rows and `batches.rs`' `v17` rows can be built as generic values of their kept texts.
  - `temper-fake`'s check reads one limit of `v17`, which the current version has too.
  - temper's refusals (`records/application.rs` and its neighbours) read `people_v16`. The patch to `people_v17` appends refusal tags and leaves the existing layouts alone, so they can read the current version.
  - jig's patch `p0001.rs` decodes archived executor pieces by `people_v16` while it upgrades a store, and `jig/tests/codec-families.rs` keeps the file for it. It reads them as generic values of the kept text instead.
  - `smith-transcript`'s `v6` has no reader of its generated code: archived turns are rewritten and decoded by `v7`.
- **And the store's version moves often:** from `v13` at 13:02Z on 2026-11-12 to `v18` at 08:41Z the next day. Each move recompiles every version the crate holds, and every crate above.

### 4.2 The rule

- **A product family's generated code is its current version's alone,** in its codec crate or its owner, and `schema/` holds the current text alone. The older versions stay as kept texts.
- **A reader of an older record,** a patch or a test, reads it as a generic value of its kept text (skein's `data/codec/README.md`, section 8), or takes it from a patch's goldens (`data/codec/command.md`, section 1).
- **skein's codec fixtures keep two versions' code on purpose:** their tests compare the old and new generated decoders (`skein/tests/codec/`), and fixtures follow their stories (`data/codec/command.md`, section 1).
- **No wire keeps an older version's code for a bridge:** a bridge goes up over generic values and down by reversing patches (`redeploys.md`, section 4).

## 5. The crates over the bound split

- **Five crates are over `crate-graph.md`, 7.3's bound with no planned split** (`tools/crates/design.md`, 3.1, its exceptions). Read by that check's rules on `e40ee6b2d9`: `temper-client-view` about 115,000 lines, `temper-client-domain` 91,000, `jig-protocol-store` 80,000, `temper-client-tools` 70,000 and `jig-client-view` 64,000.
- **They change often.** Over the 55 landings on main from 20:00Z on 2026-11-12 to `e40ee6b2d9`, less the ten that carried `crate-graph.md`'s milestones, `temper-client-view`'s and `temper-client-domain`'s library sources changed in 7 each, `jig-protocol-store`'s in 6 and `jig-client-view`'s in 3.
- **They cost every build:** the four client crates' libraries and own tests 53.7, 56.0 and 58.2 s in the three builds; `jig-protocol-store`'s 12.7, 9.6 and 14.6 s.
- **A split saves only in builds that reuse some of its pieces** (section 2). When a crate below changes, every piece rebuilds, and the pieces cost a little more than the crate did. So the first split is measured before the next starts.
- **Each splits by `crate-graph.md`, 5.2's rules,** along the lines its history says change apart, its preparatory changes first and the moves as one change (5.3). A cut through code that reaches back to the crate's root takes an interface first: `temper-client-domain`'s pages take the root `Domain` and call its routes (`temper/crates/temper-client-domain/src/pages.rs`). The client crates carry v0.1.0's UI with looks, so each splits when its crate is quiet.
- **`jig-domain-core` and `temper-engine-domain-forge` wait** for their state grouped into owned parts, as `tools/crates/design.md`, 3.1, says.

## 6. The next cuts

- **No ranking for now** (the user, 2026-11-13): the next cut after section 5's is chosen as those were, from the size bound and the measure of section 2. A ranking over each week's landings waits (section 11).
- **No crate dominates by itself.** A model score, each crate's library-source changes over section 5's landings times the packages above it, puts `jig-protocol-config` first at 3.6% of the summed score (7 changes, 85 packages above), and the twenty first at 37%. A landing that changes several crates counts a dependent once for each, so the scores are a model, not shares of real rebuilds.

## 7. The checks

In the boundary suite, as `crate-graph.md`, section 7's are, each with exceptions that only shrink:

- **Generated tests outside the library:** no generated record file holds a `cfg(test)` module, each codec crate's generated tests are its `tests/generated` target's, and a codec crate whose library holds no other tests builds no test target of it (3.2).
- **One version's code:** a product family's generated code and its `schema/` text are of its current version alone; skein's codec fixtures are the exception (4.2).

## 8. The figure after landing

- **The gate's build step,** its median over a day's full gates, against section 2's 147 s, and its compile from the shared cache's records, against 723 to 748 s, by whoever runs the CI builder.
- **Section 2's units:** the temper store's codec crate against its 15.5 s library and 15 to 21 s own tests, and each split crate's replacement set against its row.

## 9. What to expect

Each in a build that rebuilds the crates named, as each of the three builds did:

| Part | Compile saved |
|---|---|
| Generated code compiles once (3), in the store codec crates, the temper store's and jig's five | about their libraries' second compile: 15.6 s in build 574, 16.4 s in 577, and 27.9 s in 570, where jig's five rebuilt too |
| Then one version's code (4), in the temper store's codec crate | about two thirds of its library's 15.5 s and of its tables, about 14 s; and half of `jig-protocol-store-people`'s 6.5 s library when it rebuilds |
| The client wires, worker wires and smith's wires (3) | only as far as their own tests can move out |
| The splits (5) | only in builds that reuse a split crate's pieces: read after the first |

- **About 30 to 45 s in such a build,** 4 to 6% of its compile, from sections 3 and 4. The build step's figure is read after landing.
- **The cache's rebuilds of unchanged units,** 201 and 211 s in two of the builds, are larger than all of these. They aren't this draft's (section 11).

## 10. Who carries what

| Part | Who |
|---|---|
| Generated tests as their own output and target, the drift test writing them, the library tests moved, and the check (3, 7) | temper/dev |
| One version's generated code: the older versions out, their readers moved, and the check (4, 7) | temper/dev |
| The splits of section 5, one milestone each | temper/dev |
| Reading the figure after landing (8) | whoever runs the CI builder |

## 11. Later and open questions

- **Not this draft's: the cache's operation.** Units rebuilt with an unchanged key, 201 and 211 s in two builds, and slow remote lookups go to whoever runs the builders.
- **Later: a ranking of the next cuts** (6), in the figure's tool over each week's landings, counting each landing's rebuilt packages once, with a share that makes a crate the next split's candidate. The user's, when they want it.
- **Later: one test binary per package.** The ten packages with two or more test binaries the tiers don't name hold 37 of them. Merging them would save about 20 s a build, too little for now. `crate-graph.md`, 6.4's choice stays the tiers'.
- **Later: crates' own tests,** about half of a build's compile. A crate's own tests compile it again (`crate-graph.md`, 6.2). Tests that use only public items could become integration tests, against the testing strategy's step tests (`skein/docs/foundation/testing-strategy.md`, 2.1).
- **Later: the client wires' public families** in crates of their own (`tools/crates/design.md`, 5).
