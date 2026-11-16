# The view's crates

Provisional, 2026-11-16. From `temper/docs/design/draft/gate-compile.md`, 5, milestone C7 of
`temper/docs/design/draft/gate-compile.execution.md`: `jig-client-view` split by
`temper/docs/design/draft/crate-graph.md`, 5.2's rules, and landed as its 5.3 says. It says what
each piece holds and why, the preparation that lets every file go whole to one piece, where the
tests go, the change of moves, and the readers. What the reference UI's view does is `view.md`'s and
`view-state.md`'s: nothing here changes a behaviour, a public path or a test's check.

## 1. In one page

- **Two crates for the reference UI's view,** in `jig/crates/`, each a step crate: its words
  (`jig-client-view-words`), the phrasebook, each part's vocabulary, jig's owner, region names and
  style tokens; and `jig-client-view` above them, the state, the tree, the bindings and the sizes
  (section 2).
- **Why there** (2.3). Of the 53 landings that changed the crate's library sources from 2026-10-28 to
  `5e6b0d671e`, 50 also changed the library of a crate below every piece (`jig-client` and its
  family in 45), so a build rarely reuses a piece, as
  with temper's view (`temper/docs/design/client/view-crates.md`, 2.3). What the split buys is the
  size bound kept, two library units, and the words built once for every crate above them.
- **Preparation first,** one change, behaviour unchanged, so that the words name nothing of the
  crate outside them: what a `Words` method draws by the tree's builders moves down to the words
  (3.1); the one saying of the state's moves up to the tree (3.2); the text bound a path needs moves
  down to the words (3.3); a test-only phrasebook moves to its test, and the words' tests take their
  own fixture (3.4).
- **Then one change of moves** (section 5): files moved and not edited but where the build forces it,
  so that git's rename detection carries branches in flight across it.
- **The tests stay with the view** but the owner's, the regions' and the phrasebook's (section 4):
  the rest drive the state through the shared `Fixture` or build trees.
- **Two readers move** to the words alone (section 6): jig's browser parts and the client's browser
  world.

## 2. The crates

```
jig-client-view               the reference UI's view: its state, the tree, the bindings, the sizes, the boundary, its records; the words' items under today's paths
└── jig-client-view-words         its words: the phrasebook and English, each part's vocabulary, the fragments a Words method draws, jig's owner, region names and style tokens
```

### 2.1 What each holds

By today's files, under `jig/crates/jig-client-view/src/`, once the preparation of section 3 has
landed. A directory goes with the file of its name.

| Piece | Files |
|---|---|
| words | `words/` whole, the module `words`, with `words/fragments.rs` and `words/fragments/` (3.1); `owner.rs`, `regions.rs`, `styles.rs` |
| `jig-client-view` | `lib.rs`, `binding.rs`, `boundary.rs`, `state/`, `tree/`, `sizes/` |

`words` is an inline module of today's `lib.rs` (`pub mod words { ... }`), and so are `state`,
`tree`, `sizes` and `generated`: the words' root takes the `words` module's lines, and the view's
root keeps the others.

Their sizes, counted as raw lines of today's files, are each under the bound of `crate-graph.md`,
7.3, which `tools/crates/design.md`, 2's rules count a little lower: about 13,400 for the words, its
tests about 250 of it; and about 51,200 for `jig-client-view`, 24,000 of code and 27,000 of tests
(4.1).

### 2.2 Its edges

- **The words name no piece;** `jig-client-view` names them.
- **The words depend on what their files name,** from today's manifest: skein-lib, `skein-ui` and
  `jig-client`, and on no other. They have no dev-dependency (4.2).
- **`jig-client-view` keeps today's dependencies and dev-dependencies,** each named by its own
  files, and adds the words.
- **Step crates,** each under `jig/crates/`, on the step profile (`[lints] workspace = true`, jig's
  `clippy.toml`), `no_std` but under test, in the strict subset (`AGENTS.md`, 2; `jig/AGENTS.md`):
  no trait and no generic item crosses the cut.

### 2.3 Why there

From the crate's history, `crate-graph.md`, 5.2's first rule, and its module graph, its fourth:

- **The history.** From 2026-10-28, the repository's first landing, to `5e6b0d671e`, 53 landings
  changed the crate's library sources, its `src/tests/` not counted. The tree, `binding.rs` and the
  sizes changed in 41, the words with the owner, regions and styles in 27, the state with the
  boundary in 18. The tree changed without the words in 22, the words without the tree in 8.
- **Below every piece:** 50 of the 53 also changed the library of a crate every piece builds on: 45
  of `jig-client`, `jig-client-domain` or `jig-client-protocol-link`, the others of `skein-ui`,
  skein-lib, `skein-codec` or `jig-protocol-transcript`. So a piece is reused in about three of the
  history's builds, whatever the cut. A split saves only in such builds
  (`gate-compile.md`, 5); the figure after landing says whether it did (`gate-compile.md`, 8).
- **One graph today.** The words, the state, the tree, the sizes and the bindings lie on one cycle:
  `Words` methods draw chips and outcomes by the tree's builders, `Words::progress` names the state's
  `Progress`, and the words read the sizes' text bound; the state names the tree's kept notices and
  frame summary, and the sizes name the state and the tree. The owner, the regions and the styles
  name only the owner.
- **The words below, though they change often:** every builder of the tree names its words, and
  the words name nothing above them once the preparation of section 3 has landed, as temper's
  vocabulary is below its view (`temper/docs/design/client/view-crates.md`, 2.3).
- **The state is not a piece of its own.** It would sit between the words and the tree, after its
  kept notices, its frame summary and its configuration came down from the tree and the sizes. But
  its tests drive `Ui` through the shared `Fixture` (`src/tests/fixtures/client.rs`), which the
  tree's tests use too, and which reads the state's crate-private items: the state's tests would
  need the fixtures twice, once in their own crate and once above it, or would stay with the view
  above it. Either way a state piece saves no build the history shows and costs a crate (5.2's
  fifth rule). It is Later (section 7).
- **The rest is not cut further.** At about 51,200 lines, its tests among them, `jig-client-view` is
  under the bound.

### 2.4 Their roots

- **The words' root declares `pub mod owner;`, `pub mod regions;`, `pub mod styles;` and the `words`
  module,** with today's `mod` lines of `words`, so that every moved file compiles unedited at
  `crate::words::`, `crate::owner::`, `crate::regions::` and `crate::styles::`. It re-exports, under
  today's names, the items its files name at `crate::` (`crate::Owner` among them), and carries
  today's crate attributes that its files meet (`no_std` but under test, `forbid(unsafe_code)`, the
  `clippy::manual_map` expectation if its files still meet it) and `extern crate alloc;`.
- **`jig-client-view` keeps its name and every public path** (`crate-graph.md`, 5.3): its root
  re-exports `pub use jig_client_view_words::{regions, styles, words};` in place of those modules,
  and brings `owner` into scope privately so that `pub use owner::Owner;` and its files'
  `crate::owner::` paths stay; every other line of today's root stays, `pub use
  styles::JigStyle;`, `pub use words::{document_words, literal_segment, phrases};` among them.
- **An item the view's files or tests name is `pub` in the words,** with the modules on its path;
  nothing else widens. Each module of `words` that the view names today as `pub(crate)` becomes
  `pub`, and so do the items of it the view names, and so `jig_client_view::words::` shows them too.
- **A newly public item takes what its lints ask** (`Debug`, `#[must_use]`), and nothing else of it
  changes.
- **A type a public item's signature or public field exposes is `pub` where it stands,** its module
  staying private where nothing above names the module, so that `private_interfaces` holds.

## 3. The preparation

One change in `jig-client-view` before the move, behaviour unchanged, every test checking what it
checked and keeping its name. A moved item takes its doc comments, attributes and private helpers
with it. After it, no file under `words/`, nor `owner.rs`, `regions.rs` or `styles.rs`, names an
item of the crate outside them, by `crate::`, a grouped `use`, a root re-export (`crate::Ui`,
`crate::Config`) or a `super::` that climbs out; and no `#[cfg(test)]` item of theirs is used outside
them.

### 3.1 What a `Words` method draws

The tree's functions that a `Words` method calls move down to a new module of the words,
`words::fragments` (`words/fragments.rs`), "the fragments a `Words` method draws beyond its own
text", the same subject as the tree's fragments seen from the words (`DESIGN.md`, 5):

| From | To |
|---|---|
| `tree/fragments/outcome.rs`, whole: `render`, `MAX_NODES` and its helpers | `words/fragments/outcome.rs` |
| `tree/fragments/chip.rs`'s `ending_summary`, `hold_failure` and `retry_reason`, with its helpers `text`, `number`, `wall` and `document`; `tree/cards/chip.rs`'s `failure` | `words/fragments/chip.rs` |
| `tree/fragments/deployment/hello.rs`'s `refusal`, with `agent` and what they need of `deployment.rs` (`text`, `field_number`, `duration`) | `words/fragments/hello.rs` |

- **Names and signatures stay;** only the module changes. A helper the tree still uses too, such as
  chip's `text`, is named by the tree at its new home.
- **The tree's modules re-export what moved** for the tree's own callers
  (`tree::fragments::outcome` as `pub(crate) use crate::words::fragments::outcome;`,
  `tree::fragments::deployment::hello_refusal`), or the callers name the words' path: the producer
  chooses per caller. The words' callers name the words' path.
- **Moving `hello.rs` whole** with what `hello` itself needs (`region`) is the producer's choice, if
  it leaves fewer helpers split across the two modules.

### 3.2 The saying of a request's progress

`Words::progress` (`words/words.rs`), the one `Words` method naming the state, `state::intents::
Progress`, becomes a free function of the tree, `pub fn progress(b: &mut Builder, words: &Words,
progress: Progress)` in `tree::cards::common`, its body unchanged but `self` read as `words`. Its
four callers, in `tree/cards.rs` and `tree/sections/`, call it. Nothing outside the crate calls it.

### 3.3 The text bound

`sizes::limits::node_text`, "room for a path's preceding words, omitted-segment mark and one escaped
segment", which the words read and which names only `literal_segment::max_bytes`, moves to
`words/literal_segment.rs`. `sizes::limits` re-exports it, `pub use crate::words::literal_segment::
node_text;`, so that `jig_client_view::limits::node_text`, which jig's test view and `ops`' view read,
stays.

### 3.4 Tests

- **`words::wake_words::english`,** a `#[cfg(test)]` phrasebook that only the tree's wake tests
  (`src/tests/tree/fragments/wake.rs`) call, moves to that test file as its helper, with its
  `#[cfg(test)]` imports, built from `wake_words`' `shapes` and `language` as today. After the split
  it is the view's test code, and the words' `cfg(test)` items are out of the view's reach.
- **`src/tests/words/phrases.rs`** names `tests::tree::cards::CLIENT` for its client's text bound
  alone: it takes its own copy of what it uses, so that it names nothing of the crate's tests
  outside `src/tests/owner.rs`, `src/tests/regions.rs`, `src/tests/words.rs` and `src/tests/words/`.

## 4. Its tests

### 4.1 Where each test goes

- **To the lowest piece that holds everything it uses,** through that piece's dependencies and
  dev-dependencies, as `temper/docs/design/client/domain-crates.md`, 4.1 has it. A test file goes
  whole to one piece.
- **The words:** `src/tests/owner.rs`, `src/tests/regions.rs`, `src/tests/words.rs` and
  `src/tests/words/phrases.rs`, as their `src/tests.rs`'s modules. Each uses only the words, the
  owner, the regions or the styles, and crates outside the view.
- **`jig-client-view`:** every other test, `src/tests.rs`'s root helpers, the fixtures and the kit
  among them. The state's tests drive `Ui` through `fixtures::Fixture`, and the tree's build trees;
  a few tests in the tree's files read only the words' phrasebook (in `src/tests/tree/fragments/`),
  and stay whole with their files.
- **No piece's tests use a crate that depends on that piece,** which would give them two copies of
  its types.
- **What each test checks stays,** and its name with it.

### 4.2 Fixtures

- **The words' tests keep their own,** in what they moved with (3.4): they use no shared fixture,
  and so need no dev-dependency.
- **`jig-client-view`'s tests keep** `src/tests/fixtures.rs`, `src/tests/fixtures/` and
  `src/tests/kit.rs`, and their dev-dependencies, `skein-scratch` and `skein-ui-text`.

### 4.3 Data and paths

- **The rendered goldens** under `golden/rendered/` are read by `kit::Goldens` from its package's
  directory (`skein_scratch::paths::package()`), which doesn't move.
- **The records' owner stays** `jig-client-view`: its `schema/records-v1.schema` and `golden/*.bin`,
  and `jig/tests/codec-families.rs`' entry for them, are unchanged (`tools/crates/design.md`, 3.2).
- **`.config/fixture-readers.toml`** keeps `jig-client-view`'s entry. The words have no
  `src/tests/fixtures/` directory, so no entry of their own.
- **No file is read by path** in the words' library: no `include_str!`, `include_bytes!`, `#[path]`
  or manifest directory.

## 5. Landing the split

One change of moves, after the preparation (`crate-graph.md`, 5.3):

- **Each file moved whole** to its piece, by 2.1 and 4.1, in a commit of moves alone, and not edited
  but where the build forces it, in a commit after the moves: a visibility (2.4), what a newly public
  item's lints ask, and a crate-level `expect` a piece no longer meets.
- **New files:** the words' manifest, root and `src/tests.rs`; `jig-client-view`'s root re-exporting
  (2.4) and its manifest naming the words; the workspace's members and dependencies. The words take
  `jig-client-view`'s lint profile, `step`, by `tools/lints/lints.py --set`.
- **The checks follow:** the size check's exception for `jig-client-view` goes
  (`tools/crates/design.md`, 3.1), each piece under the bound; the `temper` binary's packages in
  `tests/boundary/tests/binaries.rs` name the words beside `jig-client-view`;
  `.config/gate/wasm32.txt` names them beside it; jig's crate-graph test
  (`jig/tests/boundary/src/tests/graph.rs`) names them wherever it names `jig-client-view` as a
  dependency a crate may have, and classes them as the view is classed (a client step crate, linking
  no smith crate).
- **Citations of the moved files follow:** a document or comment naming a file under
  `jig-client-view/src/words/`, or its `owner.rs`, `regions.rs` or `styles.rs`, names it under
  `jig-client-view-words/src/`.
- **Out of the way of plans in flight:** it lands between other landings in the crate, as
  `crate-graph.md`, 5.3 says, and goes before none that was marked first. Main is read afresh
  before it: a landing there in the crate that the goal's branch lacks is caught up first, so that
  the moves carry it.

## 6. The readers

After the move, as `temper/docs/design/client/domain-crates.md`, 6 has it: a package whose own code
needs only the words, and none of whose other dependencies of the same kind reach
`jig-client-view`, depends on the words instead. From the readers at `5e6b0d671e`:

- **`jig-client-shell-web`,** jig's browser parts, reads `Owner` and `regions::JigRegion` in its
  library and `JigStyle` in its tests: it depends on the words, so that a change to the state or the
  tree rebuilds none of it.
- **`jig-client-browser-world`** reads the phrasebook alone (`phrases::english`, `JigPhrase`): it
  depends on the words where none of its other normal dependencies reach `jig-client-view`, and
  keeps the view otherwise.
- **The others keep `jig-client-view`:** the client's roots (`jig-ops-client-domain`,
  `jig-client-test-domain`), the view roots (`jig-ops-client-view`, `jig-client-test-view`), the kits
  (`jig-client-kit`, `jig-client-protocol-kit`, `jig-client-test-shell-web`) and the worlds each name
  the state, the boundary, the tree or the sizes.

## 7. Later and open questions

- **Later: the state as a piece of its own,** between the words and the tree, once its tests have a
  harness of their own that the tree's tests needn't share, or the view nears the bound again. Its
  kept notices (`tree/notices/kept.rs`), its frame summary (`tree/frame/me.rs`) and its
  configuration (`sizes/limits/ui.rs`) come down to it first, and the boundary goes with it.
- **The figure after landing** is `gate-compile.md`, 8's: each piece's units against
  `jig-client-view`'s and whether the builds after it reused the words.
