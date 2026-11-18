# gate-units: a gate builds what it runs

Draft, 2026-11-18. What the CI gate compiles has grown since 2026-11-14, and the shared compiler cache on its builder no longer keeps one gate's units for the next. A survey of the cache's records against the forge's gate logs, 2026-11-13 to 2026-11-18, found three causes in this repository and one in the gate's runner, outside it. This draft takes the three. Every test stays, and so does what it checks. **Run, unless marked read.**

## 1. In one page

- **What a gate builds** (section 2). On 2026-11-14 a gate's units came to about 28 GiB. Now the candidate's own build is about 32 GiB, and the base checks add 3 to 25 GiB of the base's units. Two gates in a row need 50 to 79 GiB, more than the cache keeps, so a gate rebuilds units the one before it had made: 148 s a gate in the 6 hours to 2026-11-18 09:04Z, 320 s in the 2 hours to 00:43Z.
- **The base checks build only their tests' targets** (section 3). `tools/tiers/one-process --alone` lists each flagged binary's whole package, so every target in it is built at the base.
- **The fixtures kit splits along what its users need** (section 4). `temper-fixtures` gained the worker's, the engine's and the forge's crates as normal dependencies on 2026-11-17, for two of its modules, and every client test that reads its documents now links them.
- **The `parent` test gets a harness of its own** (section 5): one process guard, linked through all of `temper-box-kit`, makes a 412 MiB executable.
- **The figure after landing** (section 6): a gate's units, and two gates' together, from the cache's records.

## 2. What a gate builds

From the cache's records of each gate's distinct units, cut by its log from its `full gate:` line to its end:

| Gate | Date | Its build step | The whole gate |
|---|---|---|---|
| 725 | 2026-11-14 | 27.96 GiB | 28.27 GiB |
| 1296 | 2026-11-17 | 32.68 GiB | 57.48 GiB |
| 1325 | 2026-11-18 | 31.98 GiB | 40.18 GiB |
| 1327 | 2026-11-18 | 19.68 GiB (the temper tree alone) | 22.70 GiB |

- **The difference between the two columns is the base checks:** units the base's phases used and the candidate's didn't, 2.9 to 24.6 GiB a gate (gate 1296's 24.6). Same source and settings, other dependencies: gate 1327's `temper-box` `suite` was built twice, 473 MiB each, its keys differing only in their externs.
- **The candidate's own build grew about 4 GiB:** existing test executables 2.5 GiB larger, the seven client and view crates' own tests among them, each more than doubled (`temper-client-view`'s 80 to 166 MiB); new test targets 0.9 GiB net.
- **Not a profile:** `Cargo.toml`'s profiles, `.cargo/config.toml` and `rust-toolchain.toml` are as they were on 2026-11-14, and the two builds of one executable share their compiler and link fingerprints.
- **Two gates in a row** need 25 to 68 GiB in the 6 hours to 2026-11-18 09:04Z, and up to 78.7 GiB measured before (gates 1293 and 1296).

## 3. The base checks build only their tests' targets

`tools/tiers/references.md`, 5: the base's run "lists ... with `--profile PROFILE`, a `-p` for each flagged binary's package", and that listing builds every target of each package. Gate 1327's default base visit built `temper-box`'s `process`, `live`, `live_github` and `proof_live`, 1.65 GiB, which the default profile doesn't select and so can't have flagged.

- **The listing names each flagged binary's own target:** `--lib` for a crate's own tests, `--test NAME` for an integration target, the binary's target for a binary's, grouped by package, so the base builds those targets and their dependencies alone.
- **What stays:** the flagged tests and their names, a test missing at the base reported `missing`, ignored tests' rules, the empty process's figure, and the verdict (`tools/tiers/references.md`, 5 and 6), which this section's text follows.

## 4. The fixtures kit splits along what its users need

`temper/testing/temper-fixtures/Cargo.toml` had four normal dependencies on 2026-11-14 (`temper-test-env`, `jig-client`, `skein-lib`, `skein-io`). It now has eighteen, among them the worker's shell, the engine's startup, stock and vocabulary, the forge's change, landing and tiers, and the clients' wire and test records (`3e97d6b18b`, `45a5e01e8b`, `5332ef1eea`, `c9c334454e`, 2026-11-17). Two modules need most of them, `src/agent.rs` (its `given` runs the startup's fixed-values reader, the figures composer and the worker's configuration writer) and `src/landing.rs`. The client and view tests read `client_documents` and `memory_literal`, and link the rest.

- **`agent` and `landing` move to kits of their own,** each depending on what it uses, their tests and production writers moved whole: a configuration written by the real writer stays so written.
- **`temper-fixtures` keeps** `browse`, `client_documents`, `first_run`, `memory_literal`, `milestones` and `ports`, and the dependencies they need.
- **Each user depends on the kit it reads:** the 45 test executables that link `temper-fixtures` grew 1.7 GiB together since 2026-11-14, the seven client and view crates' own tests 0.5 GiB of it; what this split wins back is read after it.

## 5. The `parent` test gets a harness of its own

`temper/testing/temper-box-kit/tests/parent.rs` (`f042ff8fa9`, 2026-11-14) checks the process guard of `src/parent.rs` and `src/parent/executable_read.rs`: a child's identity kept after it exits, a process that can't be read. Its executable is 412 MiB because it links all of `temper-box-kit`.

- **The guard and its observation move to a small crate,** on the process primitives, deadlines and shell keeper operations it uses, which `temper-box-kit` depends on; `parent`'s tests move with it, real `/bin/cat`, inode and argv, pidfd and unreadable-process cases all kept.
- **Its place in the suites stays:** `tools/tiers/layers.toml` and any selector naming its binary follow the move.

## 6. The figure after landing

- **A gate's units and two gates' together,** from the cache's records on the CI builder, against section 2's table, by whoever runs the builder, after each milestone.
- **Not forecast.** Section 2's figures are measured; what each section wins back isn't, and section 3's depends on what each gate flags.

## 7. Not in this plan

- **The gate visiting the base once,** for both suites, and running the candidate on the build it already has: the gate's runner, outside this repository.
- **The cache's size,** an operational setting.
- **Folding small new test targets into existing harnesses** (`temper-engine-service-vocabulary`'s `figures`, `jig-test-service`'s `stock_cap`, `jig-protocol-config`'s `base` and `overrides`, 0.21 GiB together): later, if section 6 still needs it.
- **`temper-version`'s build identity,** which changes its key at every checkout: it stays true.

**Tests stay cheap:** the suites are the check, and a milestone runs a test it moves by name alone, at most. **A plan runs the focused and fuzzy suites alone and is complete when it lands (`AGENTS.md`, 5).** A stored format or protocol change is a question first. `walk = false`.
