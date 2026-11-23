# The store's crates

Provisional, 2026-11-16. From `temper/docs/design/draft/gate-compile.md`, 5, milestone C5 of
`temper/docs/design/draft/gate-compile.execution.md`: `jig-protocol-store` split by
`temper/docs/design/draft/crate-graph.md`, 5.2's rules, and landed as its 5.3 says. It says what
each piece holds and why, the preparation that lets every file go whole to one piece, where the
tests go, the change of moves, and the readers. What the store does is `README.md`'s and its
documents': nothing here changes a behaviour, a public path, a stored byte or a test's check.

## 1. In one page

- **Two crates and a kit.** The records' translation, `jig-protocol-store-records` in
  `jig/crates/`: jig's records to the store's generated families and back (`records.md`, 1), today's
  `codec` module. And `jig-protocol-store` above it, the component: its state, its boundary, keys,
  commits, reads, opening, jobs, transcripts and secrets. The tests' shared domain fixtures go to a
  kit, `jig/testing/jig-protocol-store-kit` (section 4).
- **Why there** (2.3). Of the 116 landings that changed the crate's library from 2026-10-28 to
  `0eaa914bb1`, 51 left the translation alone, and 36 of those changed nothing below it either: a
  build reuses the translation in about a third of the history's builds, 16 of the 42 in its last
  week. Everything else takes the component's state, `Store`, which a split may not cut
  (`crate-graph.md`, 5.2).
- **Preparation first,** two changes side by side, behaviour unchanged: what the translation names
  of the component moves up to it (section 3), and the tests are staged by the piece they go to
  (4.3).
- **Then one change of moves** (section 5): files moved and not edited but where the build forces it,
  so that git's rename detection carries branches in flight across it.
- **No reader moves** (section 6): the two packages that name the translation alone reach the
  component through other dependencies.

## 2. The crates

```
jig-protocol-store            the component: Store, its boundary, keys, commits, reads, opening, jobs, transcripts, secrets; src/generated/openings.rs; the re-exported `codec` path
└── jig-protocol-store-records    jig's records to the store's generated families and back: src/generated/ translation modules and the hand-written `codec` module that calls them; limits, names, encode and decode, the component's own rows
    └── jig-protocol-store-{core,tasks,people,settings,families}   the generated record code (`../README.md`, 5)
jig-protocol-store-kit        testing: the domain values and limits both pieces' tests build
```

### 2.1 What each holds

Under each crate's `src/`. A directory goes with the file of its name.

| Piece | Files |
|---|---|
| `jig-protocol-store-records` | hand-written `codec.rs`, `codec/`, calling the translation modules in `generated/`, one per mapped family |
| `jig-protocol-store` | `lib.rs` and the component's modules, including `generated/openings.rs` |

Their sizes, counted as raw lines of today's files, are each under the bound of `crate-graph.md`,
7.3: about 24,000 for the records' translation, 17,700 of code and 6,300 of tests; about 43,000 for
`jig-protocol-store`; about 1,700 for the kit.

### 2.2 Its edges

- **The records' translation names no piece;** `jig-protocol-store` names it.
- **It depends on what its files name,** from today's manifest: skein-lib, `skein-codec`, the five
  generated crates (`jig-protocol-store-core`, `-families`, `-people`, `-settings`, `-tasks`) and the
  domain's vocabulary it translates (`jig-domain-core-vocabulary`, `jig-domain-accounts`,
  `-authority`, `-fleet`, `-link`, `-notes`, `-people`, `-tasks`), and on no other. It links no smith
  crate and neither skein-kv nor skein-io. Its dev-dependencies are those its tests name: the kit and
  `skein-rows`.
- **`jig-protocol-store` keeps today's dependencies,** less any its remaining files no longer name,
  and adds the records' translation, and the kit as a dev-dependency.
- **A step crate,** under `jig/crates/`, on the step profile (`[lints] workspace = true`, jig's
  `clippy.toml`), `no_std` but under test, in the strict subset (`AGENTS.md`, 2; `jig/AGENTS.md`):
  no trait and no generic item crosses the cut.
- **The kit depends on neither piece** (4.2), so that both pieces' tests can use it.

### 2.3 Why there

From the crate's history, `crate-graph.md`, 5.2's first rule, and its module graph, its fourth:

- **The history.** From 2026-10-28, the repository's first landing, to `0eaa914bb1`, 116 landings
  changed the crate's library sources, its `src/tests/` not counted. The translation changed in 65,
  the rest in 112, both in 61. The rest changed without the translation in 51.
- **Below the translation:** of those 51, 36 changed no crate the translation builds on (the domain
  crates it names, the five generated crates, skein-lib and `skein-codec`, and theirs). In those the
  translation would have been reused, about a third of the history's builds. From 2026-11-09, 16 of
  42. A split saves only in such builds (`gate-compile.md`, 5); the figure after landing says
  whether it did (`gate-compile.md`, 8).
- **The translation below, though it changes often with the rest:** it names the domain's records
  and the generated families, and, once the preparation of section 3 has landed, nothing of the
  component; the component names it at every commit, load and open.
- **The rest is one state machine.** Every other module of the crate but nine takes the component's
  state, `Store`, or the methods its modules add to it, which `crate-graph.md`, 5.2's fourth rule
  keeps whole. The nine that don't (`boundary`, `keys`, `intervals`, `format`, `ended`, `owners`,
  `patch_set`, `generations` and the layouts' glue, about 4,400 lines) name the owner sets, the jobs'
  types and the restore's failures, which sit beside `Store`'s methods. A boundary piece below the
  component is Later (section 7).
- **The tests go with the code they test** (`crate-graph.md`, 5.2): the translation's tests with it,
  the domain values that both pieces' tests build in a kit, compiled once (section 4).

### 2.4 Their roots

- **The records' root,** `jig-protocol-store-records`' `src/lib.rs`, declares `pub mod codec;`, so
  that every moved file compiles unedited at `crate::codec::`. It brings into scope, privately and
  under today's names, the generated families its files name at `crate::` (`settings_v3`, `core_v6`,
  `tasks_v13`, `people_v17`, `notes_v1`, `accounts_v2`, `component_v3`), each from its own crate.
  It carries today's crate attributes that its files meet (`no_std` but under test,
  `forbid(unsafe_code)`, the `clippy::manual_map` expectation if its files still meet it) and
  `extern crate alloc;`, and declares `#[cfg(test)] mod tests;`. Its public `generated` module
  declares the translations in `src/generated/`, one per mapped family: accounts, component,
  core, notes, people, settings and tasks, called by the hand-written `codec` module.
- **`jig-protocol-store` keeps its name and every public path** (`crate-graph.md`, 5.3): its root
  re-exports `pub use jig_protocol_store_records::codec;` in place of `pub mod codec;`, so that
  `jig_protocol_store::codec::` and the component's own `crate::codec::` paths stay. Every other
  line of today's root stays. The component's `generated` module holds the openings translation,
  `src/generated/openings.rs`, called by its hand-written openings code.
- **No item widens.** Once the preparation has landed, the component names only public items of the
  translation; its `pub(crate)` items are the translation's own.

## 3. The preparation

One change in `jig-protocol-store` before the move, behaviour unchanged, every test checking what it
checked and keeping its name. A moved item takes its doc comments, attributes and private helpers
with it. After it, no file under `codec/` nor `codec.rs` names an item of the crate outside them,
by `crate::`, a grouped `use` or a `super::` that climbs out, but the generated families' root
re-exports (2.4) and, in its tests, `crate::tests::fixtures::CORE_LIMITS` (4.3); and no item of the
translation is extended outside it.

### 3.1 What names the component

Each function of the translation that names the component's items moves up to the component, its
name and signature unchanged, beside its callers:

| From | To | Names |
|---|---|---|
| `codec.rs`' `catalogue`, the model changes read at open | `catalogue.rs`, a module of the component's root | `Store`, `restore`, `keys`, `pages::reader`, `KeptMap` |
| `codec/component.rs`' `wires`, `floor` and `kept_wires`, the start's wires, the floor row and the last start's wires | `rows.rs`, a module of the component's root: "the component's own rows that carry its values" | `Wire`, `FLOOR`, `KeptWire` |
| `codec/openings.rs`, whole: a worker's opening's `encode` and `decode` | `rows/openings.rs` | `WorkerOpening`, `OpenedWire`, `OpenedTerm` |

- **The callers follow:** `open.rs`, `commits.rs`, `rehearsal.rs`, `openings.rs` and
  `upgrade/phase/sealing.rs`, and a test that names one (`src/tests/phase/registry/edges.rs` names
  `wires`). A moved function names the translation's items through `crate::codec::`.
- **`codec.rs`' `pub(crate) mod openings;` goes** with its file.

### 3.2 The levels by owner

`upgrade/admission.rs`' inherent `impl Levels`, on the translation's `component::Levels`, becomes two
free functions of `upgrade::admission`, their bodies unchanged but `self` read as `levels`:

```rust
pub(crate) fn level(levels: &Levels, owner: Owner) -> Option<u32>
pub(crate) fn set_level(levels: &mut Levels, owner: Owner, level: Option<u32>)
```

Their callers, in `format.rs`, `upgrade/admission.rs` and the tests, call them. `Owner` is the
component's, so the functions stay with it.

## 4. Its tests

### 4.1 Where each test goes

- **To the lowest piece that holds everything it uses,** through that piece's dependencies and
  dev-dependencies, as `temper/docs/design/client/view-crates.md`, 4.1 has it. A test file goes
  whole to one piece, but where its tests go to two.
- **The records' translation:** `src/tests/codec.rs` and every file under `src/tests/codec/` but
  `lifecycle.rs`, and the tests of `src/tests/settings.rs`' own body: each uses only the
  translation, the generated families and the domain's values. The inline test modules under
  `codec/` stay with their files.
- **Split by test:** `src/tests/codec.rs`' `a_deleted_mark_round_trips_with_its_number_only_key`
  and `a_deletion_row_round_trips_at_each_step` check the people's key encoding as well, by
  `keys::encode` and `keys::decode`: each stays whole with the component's tests. `src/tests/
  settings.rs`' children, `applications.rs`, `replaced.rs` and `models.rs`, open a store or name its
  keys: they stay with the component under a parent module of their own.
- **`src/tests/codec/lifecycle.rs` stays** with the component: it reads the tasks family's text from
  `JIG_STORE_SET`.
- **`jig-protocol-store`:** every other test, `src/tests.rs`' own among them.
- **No piece's tests use a crate that depends on that piece,** which would give them two copies of
  its types.
- **What each test checks stays,** and its name with it. A helper that tests of both pieces call
  is copied into each, trimmed to what that piece's tests use; one that reads the component's
  values in a test going down (`format::NONE`, the empty levels) takes its value there.

### 4.2 The kit

`jig/testing/jig-protocol-store-kit`, the library `jig_protocol_store_kit`: the domain values and
limits both pieces' tests build, compiled once for both (`tools/crates/design.md`, 3.2).

- **What it holds:** every fixture of `src/tests/fixtures.rs` and `src/tests/fixtures/` that names,
  with everything it calls, only crates outside the store and its translation: the domain's limits
  (`CORE_LIMITS`, `TASKS_LIMITS`, `NOTES_LIMITS` and their kin), its records' builders (`task_record`,
  `task_phase`, `people_member` and their kin, `record.rs`, `core_rows.rs`, `calls.rs`, `holds.rs`,
  `people_outcomes.rs`, and `reads.rs`' and `store.rs`' domain values), and the named constants.
- **What it doesn't:** a fixture that names the component's items (`Store`, its keys, `sizes`, the
  disk) stays with the component's tests; one that names the translation's items and no component's
  (`codec_limits`) is copied into each piece's fixtures, trimmed. A kit fixture names no fixture
  that stays.
- **Its form:** the `harness` profile with a `clippy.toml` of its own, `[lib] test = false`, no
  `#[test]` and no `cfg(test)` code; a fixture that was `pub(crate)` or `pub(super)` becomes `pub`.
  It depends on what its fixtures name today: the domain crates, `jig-domain-brief` and
  `jig-domain-views` among them, and skein-lib.
- **Each piece's tests keep a `fixtures` module** that re-exports the kit's items under today's
  paths (`fixtures::task_record`, `fixtures::core_rows::run`), beside its own fixtures, so that its
  test files read unchanged.

### 4.3 Staged before the move

So that the move only moves, the tests are staged in `jig-protocol-store` by the piece they go to,
each stage a `#[cfg(test)]` module of the crate's root, in one change beside section 3's:

| Stage | Becomes |
|---|---|
| `src/store_kit.rs` and `src/store_kit/` | the kit's `src/lib.rs` and `src/` |
| `src/records_tests.rs` and `src/records_tests/` | the records' translation's `src/tests.rs` and `src/tests/` |
| `src/tests.rs` and `src/tests/` | stay, `jig-protocol-store`'s |

- **A staged file names its fixtures and its siblings through `super::`,** never by its stage's
  path from the crate's root, so that it compiles unedited where it goes. It names the translation
  at `crate::codec::` and the generated families at `crate::`, as 2.4's roots keep them.
- **The kit's stage names only crates outside the store.** No stage names another stage's tests,
  and neither does `src/tests/`: each stage's `fixtures` module is the one place that names the
  kit's stage, by `crate::store_kit::`, which the move rewrites to `jig_protocol_store_kit::`.
- **`src/records_tests/fixtures.rs` re-exports `CORE_LIMITS`,** so that the inline test module of
  `codec/people.rs`, which names `crate::tests::fixtures::CORE_LIMITS`, reads the component's tests'
  fixtures until the move and the translation's after it.

### 4.4 Data and paths

- **The records' owner stays** `jig-protocol-store`: its `schema/`, the patch set that embeds the
  schemas, `jig/tests/codec-families.rs`' entries and the store codecs world's reads of them stay
  with the owner (`tools/crates/design.md`, 3.2). Codec tests build fixed-rule values from embedded
  schema texts or build generated records by their public fields; the store and test connector
  keep no family golden directories
  (skein's `data/codec/golden.md`, 1).
- **The component's test fixtures by path,** `src/tests/fixtures/*.schema` and
  `src/tests/golden/history-record.bin`, stay with the tests that read them, the component's.
  The latter pins hand-written history-log format-1 framing, not a generated family's codec.
- **No file is read by path** in the translation or its tests: no `include_str!`,
  `include_bytes!`, `#[path]` or manifest directory.
- **`.config/fixture-readers.toml`** names the kit's fixtures and each piece's, with their readers:
  the kit's are read by both pieces' tests.

## 5. Landing the split

One change of moves, after both preparations (`crate-graph.md`, 5.3):

- **Each file moved whole** to its piece, by 2.1 and 4.3, in a commit of moves alone, and not edited
  but where the build forces it, in a commit after the moves: a visibility (4.2), what a newly
  public item's lints ask, the stages' `crate::store_kit::` paths, and a crate-level `expect` a
  piece no longer meets.
- **New files:** the translation's manifest and root (2.4); the kit's manifest and `clippy.toml`;
  `jig-protocol-store`'s root re-exporting (2.4) and its manifest naming both; the workspace's
  members and dependencies. The translation takes the store's lint profile, `step`, and the kit
  `harness`, each by `tools/lints/lints.py --set`.
- **The checks follow:** the size check's exception for `jig-protocol-store` goes
  (`tools/crates/design.md`, 3.1), each piece under the bound; jig's crate-graph test
  (`jig/tests/boundary/src/tests/graph.rs`) and its dependency lists (`jig/tests/boundary/src/`)
  name the translation wherever a crate that names it must be allowed it, and class it as the
  store is classed (an engine protocol crate, which no client links), with no smith crate; the kit
  is classed with jig's other testing kits.
- **Citations of the moved files follow:** a document or comment naming a file under
  `jig-protocol-store/src/codec/` or its `codec.rs` names it under
  `jig-protocol-store-records/src/`.
- **Out of the way of plans in flight:** it lands between other landings in the crate, as
  `crate-graph.md`, 5.3 says, and goes before none that was marked first. Main is read afresh
  before it: a landing there in the crate that the goal's branch lacks is caught up first, so that
  the moves carry it.

## 6. The readers

After the move, as `temper/docs/design/client/domain-crates.md`, 6 has it: a package whose own code
needs only the translation, and none of whose other dependencies of the same kind reach
`jig-protocol-store`, depends on the translation instead. From the readers at `0eaa914bb1`, none
does:

- **`jig-one-process-world`** names only `codec::{limits, names, View}`, but its channel kit, its
  test service and jig's configuration each depend on the component.
- **`temper-engine-service-stock`'s tests** name only `codec::limits`, but its normal dependency on
  jig's configuration reaches the component, which its test build compiles anyway.
- **The others** name the component's boundary, keys, sizes, transcripts' frames or `Store` as well,
  `jig-protocol-config` and the store codecs world among them: they keep `jig-protocol-store`.

## 7. Later and open questions

- **Later: the component's boundary as a piece of its own,** below `Store`: the boundary's types,
  keys, intervals, the owner sets and the jobs' vocabulary, which most readers above name alone
  (the accounts' and clients' components, the engine service's vocabulary). The restore's failures
  and the owner sets come down from the modules that take `Store` first, so that the piece names
  nothing above it.
- **The figure after landing** is `gate-compile.md`, 8's: each piece's units against
  `jig-protocol-store`'s row and whether the builds after it reused the translation.
