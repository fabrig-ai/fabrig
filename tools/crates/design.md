# The crate graph's checks and figure

Provisional, 2026-11-12. This document comes from `temper/docs/design/draft/crate-graph.md`, 7.3 and 8, milestone G7 of
`temper/docs/design/draft/crate-graph.execution.md`, and from its 6.3 and 7.4, milestone G8. It covers how the repository measures its crates. A
check in the boundary suite bounds a library's size and a generated family's size, and another
finds test code that two test binaries of one package compile, which moves to a library. A figure, run
after landing, reads a timed build from scratch and the week's landings. Each later milestone adds
its check of the draft's section 7 (features, codec crates) to section 3.

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
- **Shared test code is compiled once.** A source file that two test binaries of one package compile
  is a finding of the shared-test-code check, beside the size check. Such code is a library the
  binaries depend on: a kit under the tree's `testing/`, or a world's own library. A kit holds no
  tests of its own, so a test build compiles it once (section 3.2).
- **The codec check** finds a package holding generated record code that depends on a workspace
  crate other than `skein-lib` and `skein-codec`, so that generated code compiles beside the domains
  (section 3.2).
- **The features check** reads three fixed contexts with `cargo metadata` and `cargo tree`: each
  root entry a member names there asks for every feature its crate resolves to, a platform's own
  entry is named on its platform alone, and Cargo's unification is set (section 3.3).
- **The generated-tests check** finds generated record code that holds its own tests, a codec crate
  without its `tests/generated` target, and a codec crate's library that builds a test target, so
  that a test build compiles generated code once (section 3.4).
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

### 2.4 A package's test binaries

- **A package's test binaries** are its targets whose `test` is true in `cargo metadata`, each
  compiled as one test binary. A library's own tests are one, as are each binary's and each
  integration test's, and a bench's or an example's that sets `test`. A target whose `test` is
  false is no test binary, whatever its kind.
- **A test binary's module tree** starts at its target's `src_path` and is read by section 2.1's
  rules, `cfg(test)` modules among them, since a test binary compiles them. A file reached twice
  within one tree is one file of that tree.
- **A file a test binary compiles** is a file of its module tree. A `#[path]` or `include!` that
  reaches into another package names a file of this test binary all the same.

## 3. The checks

Each check is in `tests/boundary/`, in the focused suite. It reads `cargo metadata` and the sources,
builds nothing, and runs on every change (`.config/gate/always.txt`). It carries exceptions, each
with what removes it. An exception that matches nothing fails its check, so the change that brings a
crate into line removes its exception, as `skein/docs/foundation/cli.md`, 9 has it.

The checks share one reader of the metadata and of module trees by section 2's rules:
`fabrig-boundary`'s library, `tests/boundary/src/lib.rs`, which each check's test binary depends
on. It holds no tests of its own (section 3.2); the reader's tests stay in `sizes.rs`.

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
| `jig-domain-core` | about 100,000, and about 88,000 after G6 | `Core`'s state grouped into owned parts (`jig/docs/development/debt/D003-core-one-object.md`), then a split along them: G6 moves its vocabulary out, but `Core` can't be cut (`jig/docs/design/domain/engine/README.md`, 3) |
| `temper-client-domain` | about 84,000 | its split, not yet planned |
| `temper-engine-domain-forge` | about 79,000, and about 70,000 after G5 | `Domain`'s state grouped into owned parts (`temper/docs/development/debt/D013-forge-one-object.md`), then a split along them: G5 moves its vocabulary out, but `Domain` can't be cut (`temper/docs/design/domain/forge/README.md`, 3) |
| `jig-protocol-store` | about 69,000 | its split, not yet planned: G2 moves only its generated record code, which this bound leaves out |
| `temper-client-tools` | about 67,000 | its split, not yet planned |
| `jig-client-view` | about 64,000 | its split, not yet planned |
| `jig-client-protocol-wire`'s own family | about 130,000 | a crate of its own for its `public` family (section 5) |
| `temper-client-protocol-wire`'s own family | about 38,000 | a crate of its own for its `public` family (section 5) |

The engine service's five crates, the worker service and the composition have no size
exception: G4 cut the engine's half into pieces each under the library bound
(`temper/docs/design/protocol/services.md`, 2 and 4.1).

The two client wires each hold a small second family, the public wire (`public_v1.rs`), beside
their own. That makes their own family a shared crate's family. The draft read
`jig-client-protocol-wire` as one schema (`temper/docs/design/draft/crate-graph.md`, 4.5). The check
keeps the rule as written and excepts both wires.

### 3.2 Shared test code and codec crates

#### Shared test code

`tests/boundary/tests/shared_test_code.rs`. A module that several test binaries of a package
compile is compiled once per binary, and again with each change to it
(`temper/docs/design/draft/crate-graph.md`, 6.3). Its findings:

- **A file that two or more test binaries of one package compile** (section 2.4). The finding names
  the package, the file by its path from the repository's root, and each test binary that compiles
  it, by its target's kind and name. A file that another package's test binary reaches by `#[path]`
  is no finding of either package: each compiles it once.
- **A source the check can't read,** as in section 3.1.

An exception names a package and a path from the repository's root, a file or a directory. It
covers that package's findings for files at or below the path, and says what removes it. One that
covers no finding fails the check.

**Where shared code goes:** to a library that the test binaries depend on, compiled once and cached.

- **A kit under the tree's `testing/`,** named for what it serves (`temper-box-kit`) or by its kind,
  on the `harness` lint profile with a `clippy.toml` of its own (`tools/lints/design.md`, 3). Its
  library sets `test = false` and holds no `#[test]` and no `cfg(test)` code, since a test build would
  compile a library's own tests as the whole crate again (`temper/docs/design/draft/crate-graph.md`,
  6.2). A test that sat in the moved code moves to one of the binaries: a `#[test]` there calls a
  public function of the kit that holds its body. A helper that sat under `cfg(test)`, which a
  dependency is never built with, becomes plain public code.
- **A world's own library,** when the code drives that world alone and adds no dependency to the
  library, as skein's layout has it for a world's binaries (`skein/docs/design/testing/README.md`,
  6).
- **The binaries keep their paths.** A binary that declared `mod support;` names the kit with
  `use <kit> as support;`, so its own code reads unchanged. What the binaries reached through their
  parent module becomes the kit's public interface.
- **A kit's paths are its own,** from the running test, through `skein-scratch`'s `paths`. A
  package's binary comes from `paths::binary`, read from the environment the test's runner gives
  the running test binary: so the binary's path is passed in when the test runs, and no kit compiles
  `env!("CARGO_BIN_EXE_...")`. Fixtures come from `paths::tree()` or `paths::workspace()`, never from
  `paths::package()`, which is the calling test's package. An `include_bytes!` or `include_str!`
  names its file from the kit's own place.
- **What one binary alone compiles stays with that binary,** beside its root, even where it sat in
  the shared directory.

**Today's findings,** read on main at `c3729d66f0`, and where each goes. The check lands with an
exception for each, and each move removes its own:

| Package | What its test binaries share | Goes to |
|---|---|---|
| `temper-box` | `tests/support/`, about 26,300 lines, which `process`, `live`, `live_github`, `suite` and `proof_live` compile whole and `live_github_managed` in part; `tests/fake_backends/box_link.rs`, which `process` and `suite` compile | `temper/testing/temper-box-kit` |
| `smith` | `tests/end-to-end/support/`, about 3,000 lines, which `end_to_end`, `live` and `live_configuration` compile | `smith/testing/smith-end-to-end-kit` |
| `skein-browser-world` | `tests/support/real.rs` and `real/`, about 1,500 lines, which `browser_facts` and `browser_real` compile; `src/pages.rs`, which they and `suite` compile | `skein/testing/skein-browser-rig` |
| `jig-test` | `tests/support/mod.rs`, `ports.rs` and `trace.rs`, about 600 lines, which `process` and `live` compile; `jig/testing/jig-protocol-accounts-kit/src/visitor.rs`, which both include by `#[path]` | `jig/testing/jig-test-kit`; the visitor from its own kit, as a dependency |
| `temper-real-loop-world` | `tests/support/mod.rs`, 11 lines, which `live_text` and `suite` compile | the world's library |
| `temper-client-shell-terminal` | `tests/support/world.rs`, about 300 lines, which its library's own tests and `suite` compile | an exception |
| `temper-engine-domain-ways` | `src/tests/invariants.rs`, 35 lines, which its library's own tests and `suite` compile | an exception |

The last two are shared between a library's own tests and its integration suite. A kit would
depend on the library, and the library's own tests, a second build of it, would then meet two
copies of its types. Each stays an exception until its area moves the library's tests that use it
into its suite, or keeps one copy alone.

**`temper-box-kit`** is `temper/testing/temper-box-kit`, the library `temper_box_kit`: what
`temper-box`'s test binaries share, the stories' processes, forges, people and fixtures.

- **What moves:** every file under `tests/support/`, its root `mod.rs` becoming the kit's
  `src/lib.rs`, and `tests/fake_backends/box_link.rs`. Three stay with their binary, as each is
  its alone: `listeners.rs` with `suite`, and `managed_client.rs`, `managed_sitting.rs` and
  `relay_managed.rs` with `live_github_managed`.
- **The GitHub fixtures,** `tests/support/github/`, are data. They go to
  `bin/temper-box/tests/fixtures/github/`, beside the cluster's, and the kit names both from
  `paths::tree()`, as it names the cluster's today.
- **Its tests:** the five `#[test]`s in `forge_land.rs` and `forge_land/rung.rs` move to `suite`, so
  they run once, where `suite` and `proof_live` each ran them. The eight in the managed files stay
  with them. The helpers under `cfg(test)` that `suite` calls become plain.
- **The managed binary** keeps its own `write_private`, which creates a file new with mode 0600, for
  its own modules. The relay's `count.txt`, the one write of a moved file that reached it, is
  created new with mode 0600 in the kit, as the relay's journal beside it is.
- **From outside:** the tests of `temper-box-shell`, `temper-engine-shell` and `temper-worker-shell`
  compile `configuration.rs` by `#[path]`, and follow it to its new place.

**The others:**

- **`smith-end-to-end-kit`,** the library `smith_end_to_end_kit`: the shipped-binary stories' person,
  browser, peers and scratch over skein's real loop, today's `tests/end-to-end/support/`. It depends
  on what the module uses as the binaries name it, smith's fixtures crates among them
  (`temper/docs/design/draft/cli.md`, 3.2).
- **`skein-browser-rig`,** the library `skein_browser_rig`: real Chromium in one service loop with a
  `skein-http` page server, today's `tests/support/real.rs` and `real/`, and the pages the world's
  tests serve, today's `src/pages.rs`. `skein-browser-world` takes it as a dev-dependency. It
  depends on no package under `skein/tests/` (`skein/docs/design/testing/README.md`, 6).
- **`jig-test-kit`,** the library `jig_test_kit`: the reference binary's stories' rig, the binary, its
  native person and the provider on skein's real ring, today's `tests/support/mod.rs`, `ports.rs`
  and `trace.rs`. The sign-in visitor is `jig-protocol-accounts-kit`'s `visitor` module, which the
  kit and the binaries take as a dependency, with `Visitor::hand` and `handed` public. The other
  files under `tests/support/` are each one binary's and stay.
- **`temper-real-loop-world`'s rail steps,** `start_change`, go to its library's `terminal` module,
  which they are built from.

#### Codec crates

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
| `jig-protocol-worker-wire` | `smith-channel` | jig's `protocol/channel/wires.md`, 2: `spoken()` states smith's channel identity |
| `jig-protocol-worker-wire` | `smith-charter` | jig's `protocol/channel/wires.md`, 2: `spoken()` states smith's charter identity |
| `jig-protocol-worker-wire` | `smith-transcript` | jig's `protocol/channel/wires.md`, 2: `spoken()` states smith's transcript identity |
| `temper-protocol-worker-wire` | `jig-protocol-worker-wire` | temper's `protocol/wires.md`, 6: `application()` builds jig's `Application`, `Spoken` and roots for temper's payloads |

It has no exceptions. An exception, were one needed, is a package, excuses every finding of that
package, names what removes it, and fails the check once it matches no finding, as an allowance
does. Every package of `temper/docs/design/draft/crate-graph.md`, 4.2 keeps its generated record
code in codec crates below it, named and placed as skein's `data/codec/README.md`, section 2 says.
The two stores' are G2's (jig's `protocol/README.md`, section 5; temper's `protocol/store.md`,
section 4). The other eight are G3's, one codec crate each, its owner's `-codec`:

| Owner | Its codec crate | Directory | Module |
|---|---|---|---|
| `jig-test-store` | `jig-test-store-codec` | `jig/testing/jig-test-store-codec` | `v2` |
| `jig-protocol-engine-clients` | `jig-protocol-engine-clients-codec` | `jig/crates/jig-protocol-engine-clients-codec` | `cursor_v1` |
| `jig-client-test-protocol-wire` | `jig-client-test-protocol-wire-codec` | `jig/testing/jig-client-test-protocol-wire-codec` | `v2` |
| `jig-client-view` | `jig-client-view-codec` | `jig/crates/jig-client-view-codec` | `records_v1` |
| `jig-protocol-accounts` | `jig-protocol-accounts-codec` | `jig/crates/jig-protocol-accounts-codec` | `v1` |
| `smith-local-protocol` | `smith-local-protocol-codec` | `smith/crates/smith-local-protocol-codec` | `local_store_v1` |
| `skein-oauth` | `skein-oauth-codec` | `skein/crates/skein-oauth-codec` | `v3` |
| `skein-ui-example-view` | `skein-ui-example-view-codec` | `skein/examples/ui/view-codec` | `records` |

- **A codec crate** holds its family's generated file at the path it had in the owner,
  `src/generated/<file>.rs`, as the module of the table at its root, and depends on `skein-lib`
  and `skein-codec` alone. It is step code on the `step` lint profile, as G2's are, and builds
  wherever its owner builds, `wasm32-unknown-unknown` among them for a client's.
- **The owner keeps the rest** (skein's `data/codec/README.md`, section 2): its schemas, kept
  texts and patches, its glue (`smith-local-protocol`'s `src/generated/layouts.rs`), its golden
  bytes, its drift test and its translation. It depends on its codec crate and names the module
  where it named it before: a public one is re-exported at the same path (`skein_oauth::v3`), so
  callers keep their paths.
- **The drift test** reads the schema and golden bytes in the owner and the code in the codec
  crate, and writes each where it reads it (skein's `data/codec/README.md`, section 6).

The check reads section 2.2's classification of files as the size check does, so the two agree on
what generated record code is.

### 3.3 Features

`tests/boundary/tests/features.rs`, from `temper/docs/design/draft/crate-graph.md`, 3.5 and 7.1. It
holds the external crates' rule of `skein/docs/foundation/cli.md`, 3: one feature set for every
build of a context. That document's check 3 (its section 9), in `tests/boundary/tests/cli.rs`, finds
a member's dependency that neither names a path nor inherits; this check takes the rest. It has no
exceptions: milestone G1 lands the rule whole.

**The contexts,** fixed, each with the dev-dependencies, the mode landings build
(`skein/docs/foundation/cli.md`, 3):

| Context | Why |
|---|---|
| `x86_64-unknown-linux-gnu` | the builders' native target |
| `aarch64-apple-darwin` | the macOS workflows' builds |
| `wasm32-unknown-unknown` | the browser's and the playground's modules |

Each context's reading lists the host's packages, for build scripts and procedural macros, beside
the target's, so the host is read in each.

**What a context reads,** building nothing, each command once per run of the test binary and shared
by its tests:

- `cargo metadata --format-version 1 --offline --locked --filter-platform TARGET`:
  - the entries each member names on that target: its resolve node's dependencies on packages
    outside the workspace, each by its name in code (`rustls_std` for the root's `rustls-std`) and
    the package it resolves to;
  - what each entry asks, from the members' dependency records of it (`features` and
    `uses_default_features`), which inherit the root's entry;
  - each external package's feature table.
- `cargo tree --workspace --offline --locked --target TARGET`, with every kind of edge (normal, build
  and dev), printing each package with its enabled features: what each package resolves to there, at
  each of its occurrences, the host's among them.

**What an entry asks:** its features, with `default` unless it sets `default-features = false`,
closed under its package's feature table: a feature adds the features it lists; `NAME/FEATURE` adds
`NAME` when the package has a feature of that name; `dep:NAME` and `NAME?/FEATURE` add nothing. For
example, serde 1.0.229's entry `{ version = "1", features = ["derive"] }` asks `default`, `std`,
`derive` and `serde_derive`, and not `alloc`, which the workspace resolves.

**The platform entries,** a fixed table, as `skein/docs/foundation/cli.md`, 3, names the one pair:

| Entry | Its platforms |
|---|---|
| `rustls` | `wasm32-unknown-unknown` |
| `rustls-std` | `x86_64-unknown-linux-gnu`, `aarch64-apple-darwin` |

Every other entry may be named in any context.

**Its findings:**

- **A feature the entry doesn't ask:** in a context, an entry that a member names there, whose
  package has an occurrence there with a feature the entry doesn't ask. The finding names the
  context, the entry, the package and the features. Its repair is the root's entry, raised to the
  union. An entry may ask for more than its crate resolves to: that is no finding.
- **An entry its members ask differently:** two members' records of one entry that ask different
  features or defaults. Only a member choosing features can make it, which check 3 also finds.
- **A platform's entry outside its platforms:** a member that names a platform entry in a context the
  table doesn't give it. The finding names the member, the entry and the context. The check reads
  where Cargo resolves the entry, not the condition's text, so a narrower condition holds, as
  `skein-fake-peers`'s `cfg(unix)` for `rustls-std`.
- **A setting missing:** `.cargo/config.toml` at the repository's root without `feature-unification =
  true` in its `[unstable]` table, or without `feature-unification = "workspace"` in its `[resolver]`
  table. The file is read by lines: a table's header, then its keys until the next header, with
  comments and blank lines skipped.
- **A source the check can't read:** a command that fails, output it can't parse, a package its
  metadata doesn't hold, or a missing `.cargo/config.toml`. Each fails the check and is never
  skipped.

**The bundle's nested build** sets `CARGO_RESOLVER_FEATURE_UNIFICATION=selected`
(`temper/docs/design/client/web/README.md`, 2.6). Its own crate's step tests pin it on the nested
commands they already read (`temper-client-bundle`'s `tests/steps.rs`), since the boundary suite
can't call a build script's code.

### 3.4 Generated tests

`tests/boundary/tests/generated_tests.rs`, from `temper/docs/design/draft/gate-compile.md`, 3.2 and
7, milestone C1 of its execution file. A codec crate's generated tests are a test target of their
own, `tests/generated/`, so that a test build compiles its generated code once, and its library
builds no test target when it holds no other tests (skein's `data/codec/README.md`, section 6). The
check reads `cargo metadata` and the sources, as the other checks do, and builds nothing. For each
package holding generated record code (section 2.2), its findings:

- **A test module in generated record code:** a generated record file whose text holds `cfg(test)`
  or `#[test]` outside comments and literals, read by section 2.1's scan. The finding names the
  package and the file.
- **Generated tests missing:** the package has no test target named `generated` whose `src_path` is
  its `tests/generated/main.rs`, or a generated record file `src/generated/<file>.rs` has no
  `tests/generated/<file>.rs` beside that root. The finding names the package and the missing target
  or file.
- **A library test target:** the package's library target has `test` true in `cargo metadata`
  (section 2.4), as Cargo gives it unless the manifest's `[lib]` sets `test = false`. The finding
  names the package. Its fix is `test = false`, once the library holds no other tests: a test that
  uses public items alone moves to the `generated` target.
- **A source the check can't read,** the metadata, a `src/generated/` or `tests/generated/`
  directory or a generated file, fails the check and is never skipped.

An exception names a package and either a file, which excuses that file's findings, or the library
test target, which excuses that finding. Each says what removes it, and one that covers no finding
fails the check. Its exceptions when it lands, read on the code as milestone C1 leaves it:

- **The library test target of each codec crate whose library keeps tests that need private
  items,** until its area moves them or they use public items alone: the wires
  (`jig-client-protocol-wire`, `jig-protocol-worker-wire`, `temper-client-protocol-wire`,
  `temper-protocol-worker-wire`), smith's `smith-charter`, `smith-channel` and `smith-transcript`,
  and the owners whose patch module's golden tests are in their library (`temper-protocol-records`,
  `skein-codec-tests`). The change that adds the check reads the list against the code as C1's
  other changes leave it: a crate left with library tests is an exception with its reason, and one
  whose library holds none is none.
- **Each older version's generated record file,** which keeps its test module and has no tests'
  module until milestone C2 of the same execution file removes it: of `temper-protocol-store-codec`,
  `jig-protocol-store-people` and `smith-transcript`, and the wires' older versions
  (`smith-channel`'s `v7`, `jig-protocol-worker-wire`'s `v12`), whichever the code still holds when
  the check lands. A wire keeps no older version's code (`gate-compile.md`, 4), so C2's
  one-version check removes the wires' files too. C2 removes these exceptions with the files, or
  none is written if C2 has landed.

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

- **Later: the two shared files left as exceptions** (section 3.2), in
  `temper-client-shell-terminal` and `temper-engine-domain-ways`, each when its area next works on
  its library's tests.
- **Later: libraries with no tests of their own.** A world's or a kit's library that holds no test
  is still compiled a second time, as an empty test binary, unless it sets `test = false`. Section
  3.2's kits set it; the others aren't surveyed.
- **Later: more contexts** for the features check (section 3.3), each a row of its table, if the
  repository builds for another platform.
- **Later: the client wires' public families,** each to a crate of its own, which removes the two
  wires' exceptions (section 3.1).
- **Later: the libraries over the bound that no milestone splits:** `temper-client-view`,
  `temper-client-domain`, `temper-client-tools`, `jig-client-view` and `jig-protocol-store`'s
  hand-written code, by the draft's rules for where to cut
  (`temper/docs/design/draft/crate-graph.md`, 5.2).
- **Later: whether lines stand in well for compile time.** The figure's per-crate times against
  each crate's lines say so (`temper/docs/design/draft/crate-graph.md`, 7.3).
