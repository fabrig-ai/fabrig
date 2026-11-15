# The tools' crates

Provisional, 2026-11-15. From `temper/docs/design/draft/gate-compile.md`, 5, milestone C6 of
`temper/docs/design/draft/gate-compile.execution.md`: `temper-client-tools` split by
`temper/docs/design/draft/crate-graph.md`, 5.2's rules, and landed as its 5.3 says. It says what
each piece holds and why, what crosses the cut, where the tests go, the change of moves, and the
readers. What the tools do is `README.md`'s and `tools.md`'s: nothing here changes a behaviour, a
public path or a test's check.

## 1. In one page

- **Two crates for the agents' tools,** in `temper/crates/`, each a step crate: jig's part, the
  module `jig` (`temper-client-tools-jig`), and `temper-client-tools` above it, temper's part and
  the tools' root (section 2).
- **The cut is the one `README.md`, section 4 already draws.** The module `jig` names no item of
  the crate outside it, and a boundary test holds that; every one of temper's modules names it. So
  the pieces are a chain, and no preparation is needed (section 3).
- **Why** (2.3). Of the 165 landings that changed the crate's library sources from 2026-10-28 to
  `fb1d45a513`, 48 changed temper's part alone, and in 34 of those jig's part would have been
  reused. A test build compiles the crate's library twice; split, a change to temper's part
  compiles about 60,000 lines instead of 117,000.
- **One change of moves** (section 5): files moved and not edited but where the build forces it,
  so that git's rename detection carries branches in flight across it.
- **The tests stay with temper's part** but four inline modules and the boundary test (section 4):
  the tests of `src/tests/jig/` drive the runner through the tools' root and table.
- **No reader moves** (section 6): each names temper's part too.

## 2. The crates

```
temper-client-tools              temper's part: its table and groups, the forge's tools, the work's tools, the limits, the help pages, push and recovery, the tools' root; jig's part under today's path, jig
└── temper-client-tools-jig          jig's part, the module jig: the generic table, a call, the argv parser and help, the runner, digests and conditions, waits, the rendering (text, JSON), the settings file
```

### 2.1 What each holds

By today's files, under `temper/crates/temper-client-tools/src/`. A directory goes with the file of
its name.

| Piece | Files |
|---|---|
| jig's part | `jig.rs` and `jig/`, with the inline test modules of `jig/render/document/clock.rs`, `hold_states.rs`, `offers.rs` and `read_owner_words.rs` |
| `temper-client-tools` | `lib.rs`, `adopt.rs`, `amend.rs`, `change.rs`, `demo.rs`, `drafts.rs`, `finding.rs`, `follow.rs`, `forge.rs`, `forges.rs`, `help.rs`, `key.rs`, `landing.rs`, `limits.rs`, `making.rs`, `push.rs`, `recovery.rs`, `root.rs`, `settings.rs`, `table.rs`, `tier.rs`, with their directories |

Their sizes, counted as raw lines of today's files, are each under the bound of `crate-graph.md`,
7.3, which `tools/crates/design.md`, 2's rules count a little lower: about 28,300 for jig's part,
its tests a few hundred lines of it; and about 45,500 for `temper-client-tools`, 14,600 of code and
30,900 of tests (4.1).

### 2.2 Its edges

- **jig's part names no piece;** `temper-client-tools` names it.
- **jig's part depends on what its files name,** from today's manifest: skein-lib, `skein-json`,
  `skein-ui`, `jig-client`, `jig-client-protocol-wire` and `temper-settings-form` (as
  `settings-form`), and on no other. It has no dev-dependency (4.1).
- **`temper-client-tools` keeps today's dependencies and dev-dependencies,** each named by its own
  files, and adds jig's part.
- **Step crates,** each under `temper/crates/`, on `temper-client-tools`' lint profile, `no_std` but
  under test, in the strict subset (`AGENTS.md`, 2).

### 2.3 Why there

From the crate's history, `crate-graph.md`, 5.2's first rule, and its module graph, its fourth:

- **The history.** From 2026-10-28 to `fb1d45a513`, 165 landings changed the crate's library
  sources, its `src/tests/` not counted. By the pieces of 2.1, jig's part changed in 117 and
  temper's in 89; jig's alone in 76, temper's alone in 48.
- **Below every piece:** 85 of the 165 also changed a crate below jig's part (`jig-client`, the
  wire, skein-lib, `skein-json`, `skein-ui`, `temper-settings-form`), which rebuilds every piece. Of
  the 48 that changed temper's part alone, 34 changed none of those, and would have reused jig's
  part.
- **What a test build compiles.** Today the crate's library, about 42,900 lines, and its test
  target, about 73,800, which compiles the library again (`crate-graph.md`, 6.2). Split, a change to
  temper's part compiles its library and its test target, about 60,000 lines; a change to jig's part
  compiles about as much as today, in units that build side by side once jig's library is built.
  The figure after landing says whether the pieces were reused (`gate-compile.md`, 8).
- **A chain, not side by side.** Each of temper's 20 modules names `crate::jig`; nothing under
  `src/jig/` names the crate outside it (`README.md`, section 4).
- **jig's part is not cut further.** Its rendering (about 17,500 lines) and its runner name each
  other, and so do the call, the rows and the argv parser through them: one graph, and a piece
  well under the bound. A cut there would need an interface for no landing the history shows.
- **temper's part is not cut further.** At about 45,500 lines with its tests it is under the bound,
  and its modules meet in the tools' table and root.

### 2.4 Their roots

- **jig's part's root declares `pub mod jig;`,** so that every moved file, which names
  `crate::jig::`, compiles unedited. It carries today's crate attributes that its files meet
  (`no_std` but under test, `forbid(unsafe_code)`, the `clippy::manual_map` expectation) and
  `extern crate alloc;`.
- **`temper-client-tools` keeps its name and every public path** (`crate-graph.md`, 5.3): its root
  re-exports `pub use temper_client_tools_jig::jig;` in place of `pub mod jig;`, so its own files and
  its readers name `jig` as today.
- **An item temper's part or its tests name is `pub` in jig's part,** with the modules on its path;
  nothing else widens. At `960d207e53`, all `pub(crate)` today:
  - modules: `argv::value`, `render::{holds, lists, outcome, task_source}`,
    `render::document::{clock, subjects}`, `runner::{acts, reads}` and `runner::acts::choice`;
  - re-exports: `jig::form_limits`, of `settings_adapter::limits` (already `pub`, its module staying
    private), and `render::document::part_bytes`, with the function it re-exports;
  - items: `argv::value::{decode, finding}`; `call::{value, number, argument}`;
    `help::{argument_type, argument_bound}`; `json::{document_depth, measured_length, encode_tool}`;
    `key::{local, reserve}`; `table::name_words`; `render::{read_task, answers_full}`;
    `render::document::{Document, print, phrasebook, hex, refusal_json}`;
    `render::document::subjects::encode`; `render::document::clock::{deadline, minute}`;
    `render::holds::{summary, way_out}`; `render::lists::argument`;
    `render::task_source::capacity`; `render::outcome::{Operation, print, kept, newline}` with
    `Operation::{for_call, of}`; `Runner::{calls, forget_kept, kept_json, kept, start}`;
    `runner::acts::{me, Failed, Prepared, words, Request, prepared}`; `runner::acts::choice::llm`;
    `runner::reads::{project, task_query}`; `wait::{fire, next_deadline}`.
- **A newly public item takes what its lints ask** (`Debug`, `#[must_use]`), and nothing else of it
  changes.

## 3. What crosses the cut

No preparation lands before the move: each of `crate-graph.md`, 5.2's conditions for a cut that
builds already holds.

- **No upward edge.** No file under `src/jig/` names an item of the crate outside `crate::jig`, by
  `crate::`, a grouped `use` or a `super::` that climbs out (`src/tests/boundary.rs`).
- **No method across.** No inherent `impl` outside `src/jig/` is for a type of jig's part.
- **One trait across, legally:** temper's `root::Parts` implements jig's `render::Parts`
  (`src/root.rs`), a local type implementing a foreign trait.
- **Nothing test-only across:** no `#[cfg(test)]` item of jig's part is used outside it, which a
  dependency, built without `cfg(test)`, couldn't give. No macro crosses.
- **No file read by path** in jig's part's library: no `include_str!`, `include_bytes!`,
  `#[path]` or manifest directory.

## 4. Its tests

### 4.1 Where each test goes

- **To the lowest piece that holds everything it uses,** through that piece's dependencies and
  dev-dependencies, as `../domain-crates.md`, 4.1 has it.
- **jig's part:** the four inline test modules of 2.1, with their files, and
  `src/tests/boundary.rs`, as its `src/tests.rs`'s module. The boundary test keeps its name and its
  checks: it reads `src/jig.rs` and `src/jig/` from its package's directory, which hold the same
  files at the same paths in jig's part.
- **`temper-client-tools`:** every other test, `src/tests/jig/` among them. Each reaches temper's
  part, directly or through the shared fixtures: `fixtures::Harness` holds the tools' root
  (`root::Root`), and the fixtures parse calls by the tools' table (`table::parse`) and bound them
  by its limits (`limits`). A test file goes whole to one piece.
- **No piece's tests use a crate that depends on that piece,** which would give them two copies of
  its types.
- **What each test checks stays,** and its name with it.

### 4.2 Fixtures

- **jig's part's tests keep a private copy of what the boundary test uses,** in its
  `src/tests/fixtures.rs`: `fixtures::sources` and its bound, `SOURCE_FILES`, trimmed to them.
- **`temper-client-tools`' tests keep** `src/tests/fixtures.rs` and `src/tests/fixtures/`, less
  `sources` and `SOURCE_FILES`, which nothing of theirs then calls.

### 4.3 Data and paths

- **temper's tests read files by path from their package's directory,** which doesn't move:
  `src/tests/golden/`, `src/tests/fixtures/overlays/`, the help reference under `temper/docs/help/`
  and the reference settings under `temper/docs/examples/`. Each stays as it is.
- **`.config/fixture-readers.toml`** keeps `temper-client-tools`' entry. jig's part has no
  `src/tests/fixtures/` directory, so no entry of its own.

## 5. Landing the split

One change of moves, with no preparation before it (`crate-graph.md`, 5.3):

- **Each file moved whole** to its piece, by 2.1 and 4.1, in a commit of moves alone, and not edited
  but where the build forces it, in a commit after the moves: a visibility (2.4), what a newly public
  item's lints ask, and a crate-level `expect` a piece no longer meets.
- **New files:** jig's part's manifest, root, `src/tests.rs` and `src/tests/fixtures.rs` (4.2);
  `temper-client-tools`' root re-exporting `jig` (2.4) and its manifest naming jig's part; the
  workspace's members and dependencies. jig's part takes `temper-client-tools`' lint profile, by
  `tools/lints/lints.py --set`.
- **The checks follow:** the size check's exception for `temper-client-tools` goes
  (`tools/crates/design.md`, 3.1), each piece under the bound; the `temper` binary's packages in
  `tests/boundary/tests/binaries.rs` name jig's part beside `temper-client-tools`.
- **Citations of the moved files follow:** a comment naming a file under
  `temper-client-tools/src/jig/` names it under `temper-client-tools-jig/src/jig/`.
- **Out of the way of plans in flight:** it lands between other landings in the crate, as
  `crate-graph.md`, 5.3 says, and goes before none that was marked first. Main is read afresh
  before it: a landing there in the crate that the goal's branch lacks is caught up first, so that
  the moves carry it.

## 6. The readers

After the move, as `../domain-crates.md`, 6 has it: a package whose own code needs only jig's part,
and none of whose other dependencies of the same kind reach `temper-client-tools`, depends on it
instead. From the readers at `960d207e53`:

- **None needs jig's part alone.** Each names temper's part too: the command line's host
  (`temper-client-shell-cli`) its root, table, limits, help, push and recovery; the box's crate
  (`temper-box-shell`) and the box's tests its root, table, limits or help; the engine service's
  component, startup and stock crates' tests and `temper-engine-smith`'s its limits; the client's
  world, the engine domain's world and the real loop's world its root, table, limits or `key`. Each
  keeps `temper-client-tools`.
- **`temper-service`'s** dev-dependency names nothing of the crate: it goes.

## 7. Later and open questions

- **Later: the tools' tests in jig's part.** `src/tests/jig/`, about 17,400 lines, drives the
  runner through the tools' root and table, so `temper-client-tools` compiles it again with any
  change to either piece. A harness of the runner and jig's table rows alone, without temper's
  root, would let those tests move down. That changes how the tests set up, not what they check.
- **Later: extraction into jig** (`README.md`, section 11) is jig's part moved to jig's tree, once
  it names no temper crate: it names `temper-settings-form` today.
- **The figure after landing** is `gate-compile.md`, 8's: each piece's units against
  `temper-client-tools`' and whether the builds after it reused jig's part.
