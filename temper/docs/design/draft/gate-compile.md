# gate-compile: a gate compiles each change once, and little else

Draft, 2026-11-13. What to change next so that temper's CI gate compiles less for each landing, now that `crate-graph.md`'s milestones have landed but G3. It comes from a measure the user asked for that day: the gate's own step times for every full gate since the CI began on 2026-11-11, and, for the gates from 06:35Z to 10:46Z on 2026-11-13, the shared compiler cache's record of each unit the gate compiled. It leaves out what the cache does wrong in operation, which goes to whoever runs the builders (section 11).

## 1. In one page

- **The gate got faster, and the splits did what `crate-graph.md` said** (section 2). Its build step, every test target of the workspace, went from a median of 196 s to 147 s, with Cranelift and a quieter builder mixed in. Measured per unit against units no milestone touched, `jig-protocol-store`'s own tests compile 70% faster, `temper-service`'s about half, and `temper-box`'s test binaries 55 to 73%.
- **The chain no longer limits the gate. The builder's cores do** (section 2). From its 10th second to its 130th, a build keeps 7 of the builder's 8 cores compiling on average. What's left to gain is how much a gate compiles: 700 to 750 s of compile each time, half of it crates' own tests.
- **Generated code compiles once** (section 3). Each generated file carries its own golden and bound tests, so a codec crate's own tests compile all its generated code a second time: 70 to 115 s a gate. The generator writes those tests as an integration test of the crate instead, against the public items they already use.
- **One version's generated code** (section 4). Codec crates hold older versions' generated code that the codec design says only the current version has. `temper-protocol-store-codec` holds three of 3.5 MB each, and recompiles all three at each of the store's version moves: five in 20 hours.
- **The crates over the size bound split** (section 5). Five crates over `crate-graph.md`, 7.3's bound have no planned split. The four client crates among them recompile 55 to 60 s a gate between them.
- **The next cuts come from the measure** (section 6). No crate dominates now: the twenty that cause the most rebuilds cause 36% of them. The figure's tool ranks them each week.
- **Expected** (section 9): 90 to 150 s less compile a gate, 12 to 20%, so a build step of about 120 to 130 s in place of 147 s.

## 2. What a gate compiles today

The gate builds every test target of the workspace (`cargo nextest run --no-run`), then runs the focused and fuzzy suites, on one CI builder with 8 cores, sharing the compiler cache with the builders that run goals. Each full gate prints its steps' times.

| Window | Full gates | Build step, median | Mean |
|---|---|---|---|
| 2026-11-12 08:04Z to 15:19Z: stable Rust with LLVM, no milestone landed | 19 | 196 s | 198 s |
| 2026-11-13 03:19Z to 10:46Z: nightly with Cranelift, all milestones but G3, other builds paused during a gate | 14 | 147 s | 131 s |

- **What moved it is mixed.** Cranelift compiled 43% less in its trial (`crate-graph.md`, 2), and pausing other builds removes contention. The milestones' own share can't be read from these two rows.
- **Per unit, it can.** Units no milestone touched compile on the CI builder in about 0.45 of their time in `crate-graph.md`, 2's trial under Cranelift: `jig-client-protocol-wire`'s own tests 51.4 s then, 22.4 s now; `temper-client-view`'s 36.4 and 18.0; `temper-client-domain`'s 28.6 and 13.7; `temper-engine-domain-world`'s suite 47.6 and 19.0. Against that ratio:

| Unit | Trial | At 0.45 | Now |
|---|---|---|---|
| `jig-protocol-store`'s own tests | 82.6 s | 37 s | 10.9 s |
| `jig-protocol-store`'s library | 18.3 s | 8.2 s | 4.0 s |
| `temper-service`'s own tests | 48.2 s | 21.7 s | 10.8 s in `temper-engine-service-component`, the largest piece, and 5.9 s in `temper-service` |
| `temper-box`'s `suite`, `process`, `live`, `proof_live` | 32.5, 30.9, 19.0, 18.3 s | 14.6, 13.9, 8.6, 8.2 s | 6.2, 6.8, 2.8, 2.2 s, and the kit they share 10.4 s, once |
| `temper-engine-domain-forge`'s own tests | 34.5 s | 15.5 s | 12.4 s |
| `jig-domain-core`'s own tests | 26.1 s | 11.7 s | 12.4 s |
| `temper-protocol-store`'s own tests | 20.1 s | 9.0 s | 4.6 s, and its codec crate's 21 to 37 s |

- **The chain is gone as a limit.** In each of the three gates the cache recorded whole, the build step kept 7 of 8 cores busy on average from its 10th second to its 130th, and its last unit ended 141 to 145 s in.
- **A gate compiles 700 to 750 s.** Of it: crates' own tests 333 to 449 s, integration test binaries about 180 s, libraries 97 to 237 s.
- **The longest unit is the temper store's codec crate:** its library 15.5 to 28.6 s and its own tests 21 to 37 s, rebuilt in every gate. It holds three versions of the store's family (section 4).
- **Rebuilds of unchanged units:** 51 to 86 units a gate missed the cache with a key an earlier gate had already used, 160 to 233 s of compile. That is the cache's operation, outside this draft (section 11).

## 3. Generated code compiles once

### 3.1 What it costs

- **Each generated file ends with its tests:** a `golden_tests` module under `cfg(test)`, its golden values and its bounds in tables of up to 1,024 cases (skein's `data/codec/README.md`, section 6). They use the file's public items alone: its records and their parts, `Limits` and `CEILINGS`.
- **So a codec crate's own tests compile its generated code again,** as `crate-graph.md`, 6.2, describes for every crate. `temper-protocol-store-codec`'s own tests take 21 to 37 s beside its library's 15.5 to 28.6 s; `jig-client-protocol-wire`'s 19 to 22 s beside 11.6 s; `jig-protocol-store-people`'s 10.5 s; `temper-client-protocol-wire`'s 9.1 s. In a gate, the own tests of crates holding generated code take 70 to 115 s.

### 3.2 The rule

- **The generator writes a file's tests as an integration test of its crate,** `tests/generated/<file>.rs`, one per generated file, naming its items through the crate's path. The tables and their cases are as today.
- **A crate whose library is generated code alone builds no test target of its library** (`[lib] test = false`). Its generated code compiles once, as the library, and the test binary compiles only the tables, which change only with their file.
- **An owner that still holds generated code,** `crate-graph.md`, 4.2's crates until G3, takes the same layout for its generated files. Its hand-written code's own tests stay as they are.
- **The drift test** reads and writes the tests where they now sit, and the codec crates' inventories (`crate-graph.md`, 4.4) follow.

## 4. One version's generated code

### 4.1 What the code holds

- **The design keeps one version's code.** A family's generated code reads its current version only. An older version's record is read as a generic value and patched to the current version before the generated code reads it (skein's `data/codec/README.md`, section 5). A kept version is its text, in `schema/kept/` (`data/codec/command.md`, section 1).
- **The code holds more:** `temper-protocol-store-codec` holds `v16`, `v17` and `v18`, about 3.5 MB each, and `temper-protocol-store/schema/` holds texts from `v14` to `v18`. `jig-protocol-store-people` holds `people_v16` and `people_v17`.
- **Their readers are few.** `temper-protocol-store`'s own tests decode older rows by `v16`'s and `v17`'s code (`src/tests/catch_up.rs`, `src/tests/batches.rs`). `temper-fake`'s check reads one limit of `v17` (`src/check.rs`). `temper-protocol-store`'s `records/application.rs` encodes and decodes a refusal by `people_v16`'s code.
- **And the store's version moves often:** from `v13` at 13:02Z on 2026-11-12 to `v18` at 08:41Z the next day. Each move recompiles every version the crate holds, twice (section 3), and every crate above.

### 4.2 The rule

- **A family's generated code is its current version's alone,** in its codec crate or its owner, and `schema/` holds the current text alone. The older versions stay as kept texts.
- **A test that needs an older record** builds it from the kept text, as a generic value (skein's `data/codec/README.md`, section 8), or takes it from a patch's goldens (`data/codec/command.md`, section 1).
- **A reader in the product reads the current version,** or reads an older one as a generic value and patches it. `records/application.rs`'s refusal is the one to move.
- **A wire bridged both ways,** where a bridge reads its previous version by generated code (`redeploys.md`, section 4), keeps that version's code, named in the check's exceptions with its bridge.
- **So a version's move** compiles one version once: about a third of today's 36 to 65 s for the temper store's codec crate, at each move and at each gate that rebuilds it.

## 5. The crates over the bound split

- **Five crates are over `crate-graph.md`, 7.3's bound with no planned split** (`tools/crates/design.md`, 3.1, its exceptions): `temper-client-view` (about 110,000 lines), `temper-client-domain` (84,000), `jig-protocol-store` (69,000), `temper-client-tools` (67,000) and `jig-client-view` (64,000).
- **They change often:** in the 45 landings on main from 20:00Z on 2026-11-12 to `e40ee6b2d9`, leaving out the milestones, `temper-client-view`'s and `temper-client-domain`'s library sources changed in 7 each, and `jig-protocol-store`'s in 6.
- **They cost every gate:** the four client crates' libraries and own tests compile 55 to 60 s a gate between them, mostly their own tests (`temper-client-view`'s 17 to 18 s, `temper-client-domain`'s 13 to 14 s, `temper-client-tools`' 10 to 12 s). `jig-protocol-store`'s take 10 to 15 s.
- **Each splits by `crate-graph.md`, 5.2's rules,** its preparatory changes first and the moves as one change (5.3), along the lines its history says change apart. The client crates carry v0.1.0's UI with looks, so their splits land between that work's landings, each when its crate is quiet.
- **`jig-domain-core` and `temper-engine-domain-forge` wait** for their state grouped into owned parts, as `tools/crates/design.md`, 3.1, says, though their own tests still take 9 to 12 s a gate each.

## 6. The next cuts, by measure

- **A crate's cost to the gates** is how often its library's sources change times the packages a test build recompiles when they do, itself and every package above it.
- **No crate dominates now.** Over the 45 landings of section 5, `jig-protocol-config` leads with 7 changes and 85 packages, 4% of all rebuilds, then `jig-protocol-store` (6 and 88), `skein-world` (2 and 187) and `jig-domain-core` (4 and 86). The twenty first cause 36%.
- **The figure's tool ranks them** (`tools/crates/design.md`, 4) over each week's landings, a test-only change counting for its own package alone. A crate over 5% of a week's rebuilds is the next split's candidate, by section 5's rules.

## 7. The checks

In the boundary suite, as `crate-graph.md`, section 7's are, each with exceptions that only shrink:

- **Generated tests outside the library:** no generated file holds a `cfg(test)` module, and a crate whose library is generated code alone builds no test target of it (3.2).
- **One version's code:** a family's generated code and its `schema/` text are of one version, its greatest, but for the bridges named (4.2).

## 8. The figure after landing

- **The gate's build step,** its median over a day's full gates, against section 2's 147 s, and its compile from the shared cache's records, against 700 to 750 s, by whoever runs the CI builder.
- **Section 2's units,** each against its row, and the temper store's codec crate against its 36 to 65 s.
- **Section 6's ranking,** from the figure's tool.

## 9. What to expect

| Part | Compile saved | When |
|---|---|---|
| Generated code compiles once (3) | 40 to 70 s | each gate, about the generated code's second compile |
| One version's code (4) | 25 to 45 s | each move of the temper store's version, and each gate that rebuilds its codec crate |
| The splits (5) | 25 to 35 s | each gate, halving what the client crates and `jig-protocol-store` recompile |

- **About 90 to 150 s of a gate's 700 to 750 s,** 12 to 20%. The gate being bound by its cores, its build step falls about as much: to about 120 to 130 s from 147 s.
- **The cache's rebuilds of unchanged units,** 160 to 233 s a gate, are more than all of these. They aren't this draft's (section 11).

## 10. Who carries what

| Part | Who |
|---|---|
| Generated tests as integration tests, the generator and the drift test writing them, and their check (3, 7) | temper/dev |
| One version's generated code: the temper store's and jig's people family's older versions out, their readers moved, and the check (4, 7) | temper/dev |
| The splits of section 5, one milestone each | temper/dev |
| The ranking in the figure's tool (6) | temper/dev |
| Reading the figure after landing (8) | whoever runs the CI builder |

## 11. Later and open questions

- **Open, the user's:** the 5% at which a crate becomes the next split's candidate (6).
- **Not this draft's: the cache's operation.** Units rebuilt with an unchanged key, 160 to 233 s a gate, and slow remote lookups go to whoever runs the builders.
- **Later: one test binary per package.** The ten packages with two or more test binaries the tiers don't name hold 37 of them. Merging them would save about 20 s a gate, too little for now. `crate-graph.md`, 6.4's choice stays the tiers'.
- **Later: crates' own tests,** half of a gate's compile. A crate's own tests compile it again (`crate-graph.md`, 6.2). Tests that use only public items could become integration tests, against the testing strategy's step tests (`skein/docs/foundation/testing-strategy.md`, 2.1).
- **Later: the client wires' public families** in crates of their own (`tools/crates/design.md`, 5).
