# The crate graph's checks and figure

Provisional, 2026-11-12. This document comes from `temper/docs/design/draft/crate-graph.md`, 7.3 and 8, milestone G7 of
`temper/docs/design/draft/crate-graph.execution.md`. It covers how the repository measures its crates. A
check in the boundary suite bounds a library's size and a generated family's size. A figure, run
after landing, reads a timed build from scratch and the week's landings. Each later milestone adds
its check of the draft's section 7 (features, codec crates, shared test code) to section 3.

## 1. In one page

- **A crate is the unit of every rebuild.** Cargo compiles a crate whole, so a crate's size is what
  a change to any of its files costs again (`temper/docs/design/draft/crate-graph.md`, 2).
- **A library's size** is the Rust that a test build of it compiles, outside generated record code.
  That is the files of its module tree, its own tests among them, counted in lines (section 2).
- **A family's size** is the lines of its generated record code, `src/generated/<family>_v<N>.rs`, over
  every version it keeps (section 2.2).
- **The size check** bounds both: a library at 60,000 lines, and a family that shares its crate with
  other families at 20,000. It is in the focused suite, in `tests/boundary/`, and builds nothing.
  Today's crates over a bound are its exceptions, each with the milestone or the work that removes
  it. Exceptions only shrink (section 3.1).
- **The codec check** finds a package holding generated record code that depends on a workspace
  crate other than `skein-lib` and `skein-codec`, so that generated code compiles beside the domains
  (section 3.2).
- **The figure** is a tool, `tools/crates/crates.py`. It times the two shapes landings build, the
  libraries and the workspace with its tests, each from scratch. Then it scores the last seven days'
  landings against that build. It prints the longest chains, each landing's recompile and chain, and
  the ten costliest crates (section 4).
- **Whoever runs the builders runs the figure,** once a day and after each milestone of the draft.
  No plan runs it. Its first run, after G9 lands and before a later milestone lands, is the baseline
  every later figure is read against (section 4.7).
- **The figures are the user's:** 60,000 and 20,000 lines, and the figure's targets, 90 s and 50 s,
  are the draft's proposals, which the user kept on 2026-11-12. A change to them is the user's.

## 2. What a crate's size is

### 2.1 A library's module tree

A workspace member's library is its target of a library kind in `cargo metadata` (`lib`, `rlib`,
`proc-macro` and the rest). Its module tree starts at that target's `src_path` and is read by fixed
rules, as Rust resolves them, from the source text alone:

- **`mod NAME;`** names a file, whatever its visibility, attributes or `cfg`, `cfg(test)` among
  them. Without `#[path]`, the file is `NAME.rs` or `NAME/mod.rs` in the module's directory. With
  `#[path = "P"]`, P is resolved as the Rust Reference's rules for the `path` attribute say,
  including inside an inline module. An inline module, `mod NAME { ... }`, adds its name to the
  directory of the declarations inside it.
- **`include!("P")`** names P, relative to the including file's directory.
  `include!(concat!(env!("CARGO_MANIFEST_DIR"), "/P"))` names P under the package's directory. An
  `include!` that names `OUT_DIR` is build output and is not read. Any other form of `include!` is a
  finding.
- **Comments and literals** are skipped, so a `mod` inside a comment or a string names nothing.
  `include_str!` and `include_bytes!` are not Rust and are not counted.
- **A file reached twice** counts once. A named file that is missing is a finding.
- **By a scan, not a parse:** the rules need only tokens, so the check reads each file once by a
  lexical scan. A full parse of the workspace's libraries would slow the focused suite down.

### 2.2 Generated record code and families

- **Generated record code** is a file under a package's `src/generated/` named `v<N>.rs` or
  `<family>_v<N>.rs`, with N a decimal number and the family lowercase letters, digits and
  underscores. The codec command writes these files (`skein/docs/design/data/codec/command.md`,
  1). Every other file there is generated glue, such as `layouts.rs`. Glue counts as hand-written,
  as the patches do (`temper/docs/design/draft/crate-graph.md`, 4.1).
- **A family** is a family name's files: `people_v15.rs` is the family `people`. `v<N>.rs` is the
  package's own family, which has no name in the file. **A family's lines** are the sum over every
  version file the package keeps, since they compile together.
- **A package holds generated record code** when its `src/generated/` has at least one such file,
  whether it is a codec crate or an owner that still holds its families
  (`temper/docs/design/draft/crate-graph.md`, 4.2).

### 2.3 Lines

A file's lines are its lines as Rust's `str::lines` counts them: blank lines and comments included,
and a last line without a newline counted. **A library's lines** are the sum over its module tree's
files, outside generated record code. So a crate's own tests count, as do its patches and glue.
Integration tests under its `tests/`, its benches and its examples do not count. This is the draft's
measure: `jig-protocol-store`'s 69,000 are its `src/` and `patches/`, outside its eight families.

## 3. The checks

Each check is in `tests/boundary/`, in the focused suite. It reads `cargo metadata` and the sources,
builds nothing, and runs on every change (`.config/gate/always.txt`). It carries exceptions, each
with what removes it. An exception that matches nothing fails its check, so the change that brings a
crate into line removes its exception, as `skein/docs/foundation/cli.md`, 9 has it.

### 3.1 Sizes

`tests/boundary/tests/sizes.rs`. Its findings:

- **A library over 60,000 lines** (section 2.3). The finding names the package, its lines and the
  bound.
- **A family over 20,000 lines in a package with more than one family** (section 2.2). The finding
  names the package, the family, its lines and the bound. A package with one family has no bound on
  it, since one schema is one crate (`temper/docs/design/draft/crate-graph.md`, 4.5).
- **A source the check can't read:** metadata, a named file that is missing, or an `include!` of
  another form. Each fails the check and is never skipped.

Its exceptions when it lands, with today's lines as read by section 2's rules:

| Exception | Lines | Until |
|---|---|---|
| `temper-client-view` | about 110,000 | its split, not yet planned (`temper/docs/design/draft/crate-graph.md`, 5.1) |
| `jig-domain-core` | about 100,000 | G6 |
| `temper-service` | about 84,000 | G4, or sooner if the worker's crates of `temper/docs/design/draft/cli.md`, 6 take it under |
| `temper-client-domain` | about 84,000 | its split, not yet planned |
| `temper-engine-domain-forge` | about 74,000 | G5 |
| `jig-protocol-store` | about 69,000 | its split, not yet planned: G2 moves only its generated record code, which this bound leaves out |
| `temper-client-tools` | about 67,000 | its split, not yet planned |
| `jig-client-view` | about 64,000 | its split, not yet planned |
| `jig-client-protocol-wire`'s own family | about 130,000 | a crate of its own for its `public` family (section 5) |
| `temper-client-protocol-wire`'s own family | about 38,000 | a crate of its own for its `public` family (section 5) |

The two client wires each hold a small second family, the public wire (`public_v1.rs`), beside
their own. That makes their own family a shared crate's family. The draft read
`jig-client-protocol-wire` as one schema (`temper/docs/design/draft/crate-graph.md`, 4.5). The check
keeps the rule as written and excepts both wires.

### 3.2 Codec crates

`tests/boundary/tests/codecs.rs`, from `temper/docs/design/draft/crate-graph.md`, 7.2. A package that
holds generated record code (section 2.2) is a codec crate, and its generated code compiles beside the
domains only when nothing but skein's lowest crates sits below it (skein's
`data/codec/README.md`, section 2). Its findings:

- **A normal dependency on a workspace member** other than `skein-lib` and `skein-codec`, of a
  package that holds generated record code. A normal dependency is one that `cargo metadata` gives no
  `kind`, under any target. A workspace member is a package of the workspace, read from the same
  metadata, by the dependency's package name. Dev-dependencies are free, `skein-codegen` for the
  crate's tests among them. The finding names the package and the dependency.
- **Glue makes no codec crate:** a package whose `src/generated/` holds glue alone (section 2.2),
  as `jig-protocol-store`'s `layouts.rs`, is an owner, and its dependencies are free.
- **A source the check can't read,** the metadata or a `src/generated/` directory, fails the check
  and is never skipped.

A dependency the rule allows, with its reason, is no finding. An allowance that matches no
dependency fails, as an exception does:

| Package | Dependency | Reason |
|---|---|---|
| `jig-protocol-worker-wire` | `skein-channel` | it declares its channel's kinds and schema in skein-channel's vocabulary beside its records, and skein-channel depends on `skein-lib` alone |
| `smith-channel` | `skein-channel` | the same, for smith's channel |

Its exceptions when it lands, each a package, which excuses every finding of that package: the
other crates of `temper/docs/design/draft/crate-graph.md`, 4.2, each until G3 of
`temper/docs/design/draft/crate-graph.execution.md`, the codec crate its area plans for it:

| Exception | Until |
|---|---|
| `jig-client-test-protocol-wire` | G3 |
| `jig-test-store` | G3 |
| `jig-protocol-engine-clients` | G3 |
| `smith-local-protocol` | G3 |
| `jig-client-view` | G3 |
| `skein-ui-example-view` | G3 |
| `skein-oauth` | G3 |
| `jig-protocol-accounts` | G3 |

The two store crates, `jig-protocol-store` and `temper-protocol-store`, are no exceptions: G2 moves
their generated record code to their codec crates (jig's `protocol/README.md`, section 5;
temper's `protocol/store.md`, section 4). The check reads section 2.2's classification of files as
the size check does, so the two agree on what generated record code is.

## 4. The figure

### 4.1 Its commands

`tools/crates/crates.py`: Python's standard library and git, run from the repository's root as
`python3 -I -B tools/crates/crates.py COMMAND`, as the other tools are.

- **`build --out DIR`** times the two builds of section 4.2 from scratch at the checkout's `HEAD` and
  writes them to `DIR`. It refuses a working tree with changes or untracked files, and a `DIR` that
  exists and isn't empty.
- **`figure DIR`** reads what `build` wrote, and the repository's history up to the build's commit,
  then prints the figure (section 4.6) on standard output as Markdown.
- **Exit 0** when it is done. **Exit 1** on a failure: a refusal, a cargo or git failure, or an input
  that is missing or malformed, with the reason on standard error. **Exit 2** on a usage error.

### 4.2 The timed builds

- **The two shapes landings build** (`temper/docs/design/draft/crate-graph.md`, 8):
  - **the libraries:** `cargo build --workspace --lib --locked --timings`;
  - **the tests:** `cargo build --workspace --all-targets --locked --timings`, the workspace with
    its tests and dev-dependencies.
- **From scratch:** each build has a fresh target directory of its own under `DIR`, and no compiler
  wrapper. The tool removes `RUSTC_WRAPPER`, `CARGO_BUILD_RUSTC_WRAPPER` and
  `RUSTC_WORKSPACE_WRAPPER` from the builds' environment. It sets `CARGO_INCREMENTAL=0`, as a shared
  compiler cache's builds do, and `TEMPER_INTERNAL_BUNDLE=none`, as development builds do
  (`AGENTS.md`, 5). The rest of the environment is the caller's, and so is the toolchain.
- **What `DIR` keeps:**
  - `libraries.html` and `tests.html`, Cargo's timing report of each build
    (`cargo-timings/cargo-timing.html` in its target directory);
  - `metadata.json`, `cargo metadata --format-version 1 --locked` with the resolved graph;
  - `build.json`, which holds the commit, `HEAD`'s branch name if it has one, the first line of
    `rustc -vV` and its `release:` line, the builder's CPU count, each build's start time and
    command, and the environment the tool set.
- **The target directories go** once their reports are copied, since they are large.

### 4.3 Units and what each waits for

- **A unit** is one row of a report's `UNIT_DATA`: its package's name and version, its target
  description and mode, its start, its duration and its metadata time. A unit with no metadata time
  writes its metadata at its end. A unit belongs to the package with that name and version in
  `metadata.json`. Units of external packages count from scratch, but no landing changes them.
- **What a unit waits for** comes from the resolved graph, by Cargo's rules:
  - a package's library waits for the libraries of its normal dependencies and for its build
    script's run;
  - the run waits for the build script's compile, which waits for the libraries of its build
    dependencies;
  - its other units wait for the libraries of its normal dependencies and its build script's run. Its
    integration tests, binaries, benches and examples also wait for its own library. The units built
    for tests (its library's own tests, its integration tests, benches and examples) also wait for
    the libraries of its dev-dependencies.
  - A package whose library Cargo built more than once, for the host and the target, has its
    dependents wait for the last of those builds.

### 4.4 Chains

- **The chain of full times** of a set of units: each unit starts when every unit of the set that it
  waits for has ended. The chain is the latest end, and the units along the path that gives it.
- **The pipelined chain:** a library that waits only on libraries starts once each of them has
  written its metadata. A unit that links (tests, binaries, build scripts) and anything that waits on
  a procedural macro or a build script's run waits for the end.
- **From scratch,** the set is every unit of the build, and the figure prints both chains. **For a
  landing,** the set is the units it rebuilds, and the figure prints the chain of full times, as the
  draft's figures do (`temper/docs/design/draft/crate-graph.md`, 2).

### 4.5 The landings

- **The window:** the build commit's first-parent commits whose committer time is within seven days
  before the build commit's, the commit itself included. Each one is a landing. A commit with no
  parent is left out.
- **Its changed paths:** `git diff-tree -r --no-renames --name-only` between its first parent and
  itself. A path belongs to the member whose directory is its longest leading run of whole path
  components, as the delivery check reads it (`tools/pre-pr/design.md`, 3). A path under no member
  changes nothing.
- **What a changed path rebuilds,** for its member P, by the path within P's directory:
  - under `tests/`: P's integration tests in the tests' shape, and nothing in the libraries';
  - under `benches/` or `examples/`: those units of P in the tests' shape, and nothing in the
    libraries';
  - anything else: every unit of P in each shape.
- **Then upward:** a unit is rebuilt when a unit it waits for is rebuilt (section 4.3).
- **A landing's recompile** is the sum of its rebuilt units' durations, and **its chain** is section
  4.4's over them. Every landing is scored against the build commit's graph and times, as the
  draft's week was against one graph (`temper/docs/design/draft/crate-graph.md`, 2). The model
  leaves out files that a crate builds in from elsewhere, and dependency updates.
- **The tenth slowest** is the value that the slowest tenth of a list reaches or passes: the
  ⌈n/10⌉-th from the top. A recompile's mean and tenth slowest are over every landing. A chain's
  mean, median and tenth slowest are over the landings that rebuilt anything.

### 4.6 What it prints

Markdown, in this order:

1. **The build:** the commit and its branch, the toolchain, the CPU count, and the window's first
   and last landing, with how many there are.
2. **For each shape,** libraries then tests:
   - from scratch: the units, their total compile, and the build's wall time (the latest end less
     the earliest start); the chain of full times and the pipelined chain, each as its total and its
     units in order with each one's time;
   - the landings: how many rebuilt anything; the recompile's mean and tenth slowest; the chain's
     mean, median and tenth slowest;
   - the ten costliest members, as the draft's table in its section 2: each one's time (its units'
     durations), the landings that changed it, the landings that rebuilt it, its share of the week's
     compile (its rebuilt units' durations over every landing, divided by every rebuilt unit's), and
     the members above it (those with a unit that waits on its library, directly or not). They are
     ranked by its share.
3. **Each landing,** oldest first: its short commit, date and subject, then its recompile and chain
   in each shape.

Times are in whole seconds and shares in whole percent.

### 4.7 Who runs it

- **On a builder, by whoever runs the builders,** once a day and after each milestone of
  `temper/docs/design/draft/crate-graph.execution.md` lands. It builds the workspace twice from
  scratch, so no plan runs it (`AGENTS.md`, 5), and the delivery check never does.
- **The first figure,** of main once G7 and G9 have landed and before a later milestone has, is the
  baseline that every later figure is read against. Section 2's numbers in the draft came from a
  model close to this one, but not from this tool.
- **A figure that is kept** goes to `temper/docs/development/` as `crate-graph-<date>-<commit>.md`,
  dated in its name, since it describes the tree at that date.

### 4.8 Its tests

`tools/crates/test_crates.py`, run with `python3 -I -B tools/crates/test_crates.py`. It uses small
fixtures: a `metadata.json`, timing reports holding a `UNIT_DATA` of a few units, and a scratch git
repository whose commits carry fixed committer dates. `build` runs against a fake `cargo` and a fake
`rustc`, as the delivery check's tests fake theirs (`tools/pre-pr/design.md`, 9). No test builds the workspace.

## 5. Open questions and Later

- **Later: the draft's other checks** join section 3 with their milestones: features (G1) and
  shared test code (G8). The last reads test targets' module trees by section 2.1's rules.
- **Later: the codec check's exceptions** (section 3.2), each removed by its crate's codec crate in
  G3.
- **Later: the client wires' public families,** each to a crate of its own, which removes the two
  wires' exceptions (section 3.1).
- **Later: the libraries over the bound that no milestone splits:** `temper-client-view`,
  `temper-client-domain`, `temper-client-tools`, `jig-client-view` and `jig-protocol-store`'s
  hand-written code, by the draft's rules for where to cut
  (`temper/docs/design/draft/crate-graph.md`, 5.2).
- **Later: whether lines stand in well for compile time.** The figure's per-crate times against
  each crate's lines say so (`temper/docs/design/draft/crate-graph.md`, 7.3).
