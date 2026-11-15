# The client domain's crates

Provisional, 2026-11-15. From `temper/docs/design/draft/gate-compile.md`, 5, milestone C4 of
`temper/docs/design/draft/gate-compile.execution.md`: `temper-client-domain` split by
`temper/docs/design/draft/crate-graph.md`, 5.2's rules, and landed as its 5.3 says. It says what
each piece holds and why, the interface through which the pages reach the root, the preparation
that lets every file go whole to one piece, where the tests go, the change of moves, and the
readers. What the client root does is `README.md`'s, sections 3 and 4: nothing here changes a
behaviour or a test's check, and the public paths stay but the three that section 2.4 names.

## 1. In one page

- **Three crates for one client root,** in `temper/crates/`, each a step crate: the forge's side
  with the vocabulary every piece names (`temper-client-domain-forge`), the pages
  (`temper-client-domain-pages`) above it, and `temper-client-domain` above them, the root itself
  (section 2).
- **Why there** (2.3). The pages read the forge's side in 26 of their modules, so the pieces are a
  chain, not side by side. Of the 174 landings that changed the crate's sources from 2026-10-28 to
  `a0ed35aa3c`, 98 also changed a crate below it, which rebuilds every piece; of the other 76, the
  forge's side would have been reused in 41. And the readers that need only the forge's side, the
  agents' tools and the engine's service crates' tests, stop rebuilding with the pages and the root
  (section 6).
- **The pages reach the root through an interface first** (3.2): `pages::Root`, what a page reads
  of the root, and `pages::Routing`, what a page's act asks of the root's routes, at the point it
  asks today. `Domain` implements the first; the root's routes pass an adapter that implements both.
- **The forge's side's edges up are cut first** (3.1): the few items of the forge's side that name
  the pages or the root move to where they belong.
- **Then one change of moves** (section 5): files moved and not edited but where the build forces
  it, so that git's rename detection carries branches in flight across it.
- **Each test goes to the lowest piece that holds everything it uses** (section 4). The tests that
  build a `Domain`, nearly all of them, stay with the root.

## 2. The crates

```
temper-client-domain              the root: Domain, its routes and its boundary, the approval, authorization, the native client's ask, copy and the sign-in tab; every piece's items under today's paths
└── temper-client-domain-pages        the pages: Pages, each page's state, facts and acts, the acts' confirmations, stock ways; Root and Routing
    └── temper-client-domain-forge        the forge's side, forge::Side and its children, and the vocabulary: addresses, limits, owners, keys, words, questions, drafts, views, sign-in lines, the public reader, a delegation's facts
```

### 2.1 What each holds

By today's files, under `temper/crates/temper-client-domain/src/`, once the preparation of section 3
has landed. A directory goes with the file of its name unless a row says otherwise.

| Piece | Files |
|---|---|
| forge | `forge.rs` and `forge/` (`stock_ways.rs` leaves, 3.1), `address.rs`, `amend.rs`, `delegation.rs` (3.1), `key.rs`, `limits.rs`, `owner.rs`, `public.rs` and `public/`, `question.rs`, `sign_in.rs`, `views.rs`, `words.rs` |
| pages | `pages.rs` and `pages/` (with `pages/stock_ways.rs`, 3.1, and `pages/root.rs`, 3.2), `acts.rs` |
| `temper-client-domain` | `lib.rs`, `domain.rs`, `route.rs`, `boundary.rs`, `approve.rs`, `asked.rs`, `authorize.rs`, `copy.rs`, `sign_in_tab.rs` |

Their sizes, read by `tools/crates/design.md`, 2's rules, are each under the bound of
`crate-graph.md`, 7.3, estimated from today's files, code and own tests: about 14,000 and 3,500
lines for the forge's side; 26,000 and 2,700 for the pages; and 2,000 and 43,000 for
`temper-client-domain`, nearly all of it the tests that build a `Domain` (4.1).

### 2.2 Its edges

- **The forge's side names no piece;** the pages name the forge's side; `temper-client-domain`
  names both.
- **Below them as today:** each piece depends on the external crates and workspace crates its own
  files name, from today's `temper-client-domain` manifest (skein-lib, `skein-json`, `skein-ui`,
  `jig-client`, `jig-client-protocol-wire`, `jig-client-protocol-link`,
  `temper-client-protocol-wire`, `temper-protocol-records`, `temper-protocol-reaches`), and on no
  other.
- **Step crates,** each under `temper/crates/`, on `temper-client-domain`'s lint profile, `no_std`
  but under test, in the strict subset (`AGENTS.md`, 2).

### 2.3 Why there

From the crate's history, `crate-graph.md`, 5.2's first rule, and its module graph, its fourth:

- **The history.** From 2026-10-28 to `a0ed35aa3c`, 174 landings changed the crate's library
  sources, a module's own tests not counted. By the pieces of 2.1, the pages changed in 122, the
  forge's side in 100 and the root's own modules in 60. The files that changed most: `pages.rs` in
  76, `route.rs` in 51, a chat's page in 48, a task's in 46, the acts in 41, home in 34, `forge.rs`
  in 33. The pages alone changed in 54, the forge's side alone in 31, the root alone in 3.
- **Below every piece:** 98 of the 174 also changed a crate below the crate (`jig-client`,
  `skein-ui`, the wires, `temper-protocol-records`, skein-lib), which rebuilds every piece. Modelled
  on the other 76, the forge's side would have been reused in 41 builds and the pages in 6. And
  156 of the 174 changed `temper-client-view` too, so the view's pieces rebuild in most builds
  anyway. What the split buys is the forge's side reused under the pages' changes, smaller units,
  and the readers of section 6 off the pages and the root. The figure after landing says whether
  the pieces were reused (`gate-compile.md`, 8).
- **A chain, not side by side.** The pages read the forge's side's state in 26 of their modules,
  72 places, so the pages can't compile beside it. The forge's side names the pages in 7 places and
  the root in 4, which section 3.1 moves.
- **One graph today.** The pages take the root's `Domain` in 38 places and call its routes in 19:
  the readers' facts (`pages::home::facts` and its kin) take `&Domain`, and the acts that answer at
  once (`pages::stop_asking`, `pages::team::hand`, the reconciliations) take `&mut Domain` and route
  through the root. So the cut needs an interface (3.2).
- **The vocabulary is not a crate of its own.** Addresses, limits, owners, keys, words, questions,
  drafts, views, sign-in lines and the public reader are about 2,400 lines; a crate of their own
  would save under two seconds to a landing that changes the forge's side alone, less than a
  crate costs (5.2's fifth rule). They go below with the forge's side, which names them.
- **The pages are not cut further.** `Pages` holds every page's state and `pages::step` routes
  each of them: it is the pages' state machine, with the methods every page module adds to it
  (`crate-graph.md`, 5.2's fourth rule).

### 2.4 Their roots

- **A piece's root declares its own modules and re-exports, under the same names, the lower
  piece's modules and root items its files name at `crate::`,** so that a moved file compiles
  unedited: the forge's side's root declares its modules and re-exports
  `address::{Address, ChatsFilter, Choice, GoalsFilter, ModelWay, Place, Step, WayKind, WorkFilter}`
  and `limits::{Limits, valid}`; the pages' root declares `pages` and `acts` and re-exports the
  forge's side's modules and those root items.
- **`temper-client-domain` keeps its name and every public path** (`crate-graph.md`, 5.3), but the
  three below: its root re-exports `address`, `amend`, `forge`, `key`, `limits`, `owner`, `public`,
  `question`, `sign_in`, `views` and `words` from the forge's side and `pages` and `acts` from the
  pages, keeps `domain`, `approve`, `asked`, `authorize`, `boundary` and `copy`, and today's root
  items (`Address` and its kin, `Action`, `Event`, `Fact`, `Request`, `Tab`, `Domain`, `fire`, `new`,
  `next_deadline`, `shown`, `step`, `take`, `unlinked`, `Limits`, `valid`).
- **Three paths change,** by 3.1 and 3.2: `forge::stock_ways` is `pages::stock_ways`;
  `forge::listed::{setup, Setup}` are `pages::welcome::{setup, Setup}`; `forge::listed::playground`
  takes the forge's side, not the root. Their readers follow in the change that moves them.
- **An item another piece names is `pub`** in its piece; nothing else widens.

## 3. The preparation

Each preparation lands in `temper-client-domain` before the move, behaviour unchanged, every test
checking what it checked. A moved item takes its doc comments, attributes and private helpers with
it.

### 3.1 The forge's side below the pages

- **A delegation's facts** go from `pages/delegation.rs` to `delegation.rs`, a module of the crate's
  root: they read a task and its calls alone, and `forge::panel` reads them. `pages` re-exports it,
  `pub use crate::delegation;`, so `pages::delegation` keeps its path; `forge::panel` names it at
  `crate::delegation`.
- **Stock ways** go from `forge/stock_ways.rs` to `pages/stock_ways.rs`, the module
  `pages::stock_ways`: `Pages` holds it, and it answers in the pages' requests and toasts. Its
  readers, in the crate and outside it, name it there; the old path keeps no re-export, which the
  forge's side couldn't give from below the pages.
- **`forge::ways::proposals`** takes each of the inbox's cards' read values, in the cards' order
  (`Option<&c::ReadValue>`, all it reads of a card), not the pages' `inbox::Card`: the root's
  caller, `route::ways_progress`, passes them from `root.pages.cards()`. Its body is otherwise
  unchanged.
- **`authorize::QUERY_BYTES`** goes to `address.rs`, whose parser bounds a query by it; `authorize`
  re-exports it, a public path.
- **`boundary::Fact`,** what the root tells of an asker it doesn't own, goes to `owner.rs`;
  `boundary` re-exports it, so the root's `Fact` stays.
- Then the files of the forge's side (2.1) name no module of the pages or of the root, but for
  `forge::listed` and `forge::trials`, which take the root and move with 3.2.

### 3.2 The pages reach the root through `Root`

In `pages/root.rs`, a module of `pages` that re-exports its items as `pages::{Root, Routing,
Parts}`:

```rust
/// What a page reads of the client root (temper/docs/design/client/domain-crates.md, 3.2).
pub trait Root {
    fn pages(&self) -> &Pages;
    fn client(&self) -> &jig_client::Client;
    fn forge(&self) -> &forge::Side;
    fn forges(&self) -> &forge::forges::State;
    fn zone_minutes(&self) -> i32;
    fn away_on(&self, project: u32, wall: skein_lib::Wall) -> bool;
    fn away_shown(&self, project: u32, wall: skein_lib::Wall) -> bool;
}
/// What a page's act asks of the root's routes, at the point it asks
/// (temper/docs/design/client/domain-crates.md, 3.2).
pub trait Routing: Root {
    fn pages_mut(&mut self) -> &mut Pages;
    fn parts(&mut self) -> Parts<'_>;
    /// The root routes one page request now, as it routes the requests of `pages::step`.
    fn request(&mut self, env: &Env<Limits>, request: Request);
    /// The root routes a page event now, as `pages::Event`s from the shell.
    fn event(&mut self, env: &Env<Limits>, event: Event);
    /// The root routes the forge's side's requests now.
    fn forge_requests(&mut self, env: &Env<Limits>, requests: &mut Queue<forge::Request>);
}
/// The root's state a page's act changes, borrowed together.
pub struct Parts<'a> {
    pub pages: &'a mut Pages,
    pub client: &'a jig_client::Client,
    pub forge: &'a mut forge::Side,
    pub terminal: &'a mut accounts::terminal_login::Terminal,
}
```

`Request` and `Event` are the pages' own (`pages::Request`, `pages::Event`), `Limits` the client's
(`limits::Limits`), `Env` and `Queue` skein-lib's.

- **`Domain` implements `Root`** in `domain.rs`, each method calling `Domain`'s inherent method of
  the same name, which stay as they are: no logic moves.
- **The root's adapter,** `route::Routed { root: &mut Domain, out: &mut Queue<crate::Request> }`,
  implements `Root` as `Domain` does and `Routing`: `pages_mut` and `parts` lend the domain's
  fields; `request` is what `route::route_pages` does with that one request today, which
  `route_pages` then calls for each (so a `pages::Request::Client` is routed through `route::client`
  at once, and `Show` and `Replace` push the root's `Request::Show` with `replace` false and true);
  `event` is `route::pages`; `forge_requests` is `route::route_forge`.
- **A page function that takes `&Domain` takes `&dyn Root`.** It reads through the trait's methods
  (`root.pages()`, not `root.pages`). Its callers, in the crate and outside it, pass a `&Domain` as
  today, which coerces; a call the coercion can't take (a `&&Domain`) drops its extra borrow.
- **A page function that takes `&mut Domain` and the root's `out` takes `&mut dyn Routing` and no
  `out`.** Each route it calls becomes the trait's, at the same point of the function: a
  `route::client(root, env, event, out)` is `root.request(env, Request::Client(event))`, a
  `route::pages` is `root.event`, a `route::route_forge` is `root.forge_requests`, and a push of the
  root's `Request::Show { address, replace }` is `root.request` of the pages' `Request::Show` or
  `Request::Replace`. So the root does and pushes what it does today, in today's order. Where it
  borrows the pages and the client or the forge's side at once, it takes `root.parts()`; where it
  hands the root to a function taking `&dyn Root`, it passes `&*root`. A function that takes
  `&mut Domain` with the pages' own queue of requests (`pages::chat::sheet::step`,
  `pages::chats::refresh`) takes `&mut dyn Routing` and keeps its queue, which its caller drains
  as today. The root's calls pass `&mut Routed { root, out }`.
- **Why a trait object.** A generic `&impl Root` would compile each page function again in the root
  and in every view crate that calls it, the compile the split saves. `Domain` stays one type, its
  fields where the root's routes borrow them apart. And an act reaches the root's routes where it
  does today: a request queued and routed after the act returns would reorder what `route::client`
  tells at once against what the act does next (`pages::team::hand::told` reads the hand's sheet
  after its unwatch is routed).
- **Turned down:** the fields the pages read moved into a struct of the pages' that `Domain` derefs
  to, which the routes can't borrow apart and which leaves the acts still needing the routes; the
  functions that take the root moved up to it, which are most of `stop_asking` and `team::hand` and
  the facts the view calls at `pages::` paths; and requests queued for the root, which reorder.
- **What the forge's side has of the root moves with it:** `forge::listed::playground` takes
  `&forge::Side` and its callers pass `root.forge()`; `forge::listed::setup` and `Setup` go to
  `pages::welcome`, taking `&dyn Root`, since they read welcome's decision; `forge::trials::look`
  goes to the root, `route::trials`, called where `route::told` calls it today, and what it names of
  `forge::trials` widens to the crate.
- **`route::program_settled`,** a function of the accounts document alone, goes to
  `pages::accounts`, where connecting a model's login and the root's routes call it.
- Then the pages' files (`pages.rs`, `pages/`, `acts.rs`) name no module of the root: neither
  `Domain`, `route`, `boundary`, `approve`, `authorize`, `asked`, `copy` nor `sign_in_tab`.

## 4. Its tests

### 4.1 Where each test goes

- **To the lowest piece that holds everything it uses,** through that piece's dependencies and
  dev-dependencies, as `../protocol/services.md`, 4.3 has it. From the tests at `a0ed35aa3c`:
  - **The forge's side:** `src/tests/` `address.rs`, `amend.rs`, `change.rs`, `drafts.rs`,
    `forge_words.rs`, `listed.rs`, `making.rs`, `public.rs`, `question.rs`, `sheet.rs`,
    `status.rs`, `views.rs` and `pages/delegation.rs`, with `fixtures/panel.rs`.
  - **The pages:** `src/tests/pages/` `calls.rs`, `chat_document.rs`, `choice.rs`, `goal.rs`,
    `parts.rs`, `report.rs` and `told.rs`, with `fixtures/chat.rs` and `fixtures/chats.rs`.
  - **`temper-client-domain`:** every other, each of which builds or steps a `Domain`, about 43,000
    lines, and `tests/suite/`, the integration target.
- **A test file goes whole to one piece:** one with a test that builds a `Domain` stays with the
  root. A test that builds a `Domain` only to borrow its client may take a client of its own, built
  as the root builds it (`jig_client::Client::unlinked` from the limits' client part), so that it
  goes with what it tests; `pages/told.rs`'s do, since it is an inline module of a page (4.3).
- **But one test leaves its file:** `pages/told.rs`'s
  `devices_are_own_scope_watched_only_while_the_page_is_open` signs in, steps a `Domain` through
  its addresses and reads its requests, so it checks the root's routing to the page. It goes to the
  root's tests, `src/tests/told.rs`, with its `event` and `up` helpers and a copy of the
  `ENDPOINT` and `row()` it reads, its checks and name unchanged; the rest of the file goes with
  the page. An inline module's file can't stay with the root, and a harness of `Pages` and a client
  would check less.
- **No piece's tests use a crate that depends on that piece,** which would give them two copies of
  its types.
- **What each test checks stays,** and its name with it.

### 4.2 Fixtures

- **The root's tests keep** `src/tests/fixtures.rs` and `src/tests/fixtures/`.
- **Each lower piece's tests keep a private copy of the fixtures they use,** trimmed to them (the
  limits', the environment's and the frame's builders, the documents' re-export of
  `temper-fixtures`, the chats' and the panel's builders), as the view's words do
  (`view-crates.md`, 4.2). No kit: the copies are a few hundred lines, and a kit is a crate.
- **Their dev-dependencies** are what their files name, from today's (`temper-fixtures`,
  `jig-client-test-records`, `skein-world`).

### 4.3 Staged before the move

So that the move only moves, the tests are staged in `temper-client-domain` by the piece they go to,
each stage a `#[cfg(test)]` module of the crate's root:

| Stage | Becomes |
|---|---|
| `src/forge_tests.rs` and `src/forge_tests/` | the forge's side's `src/tests.rs` and `src/tests/` |
| `src/pages_tests.rs` and `src/pages_tests/` | the pages' |
| `src/tests.rs` and `src/tests/` | stay, `temper-client-domain`'s |

- **A staged file names its fixtures and its siblings through `super::`,** and the crate's items as
  its piece's files do, never by its stage's path from the crate's root, so that it compiles
  unedited under `tests`.
- **No stage names another stage's tests or fixtures, and neither does `src/tests/`:** they become
  another crate's `cfg(test)` code at the move, out of reach.
- **Each stage has its `fixtures` module,** its trimmed copy (4.2).
- **The inline test modules follow their files' pieces:** `forge/listed.rs`, `forge/making.rs` and
  `forge/sheet.rs` name their tests in `src/forge_tests/`, `pages/told.rs` in `src/pages_tests/`, by
  `#[path]`. Their `super` is the library's module, so each names its stage's fixtures by one `use`
  of `crate::forge_tests::fixtures` or `crate::pages_tests::fixtures`, the one path from the
  crate's root a staged file holds; the move makes it `crate::tests::fixtures` (5). A second copy
  of the fixtures inside each inline module was turned down: `#[path]` resolves a nested `mod` as a
  `mod.rs` file's, which breaks the fixtures' own submodules, and each copy's unused builders are
  dead code.

### 4.4 Data and paths

- **No test reads a file by path** at `a0ed35aa3c`, by `include_str!`, `std::fs` or the manifest's
  directory.
- **`.config/fixture-readers.toml`** names each piece's fixtures with their package, as it names
  `temper-client-domain`'s.

## 5. Landing the split

One change of moves, after the preparation (`crate-graph.md`, 5.3):

- **Each file moved whole** to its piece, by 2.1 and 4.3, and not edited but where the build forces
  it, in a commit after the moves: a `mod` line naming a file now in another piece becoming its
  re-export, a visibility (2.4), what a newly public item's lints ask (`#[must_use]`, `Debug`), a
  `#[path]` naming a staged file, an inline test module's `use` of its stage's fixtures becoming
  `crate::tests::fixtures` (4.3), and a crate-level `expect` its piece no longer meets.
- **New files:** each piece's manifest and root; `temper-client-domain`'s root and manifest,
  re-exporting (2.4); the workspace's members and dependencies. Each piece takes
  `temper-client-domain`'s lint profile, by `tools/lints/lints.py --set`.
- **The checks follow:** the size check's exception for `temper-client-domain` goes
  (`tools/crates/design.md`, 3.1), each piece under the bound; the `temper` binary's packages in
  `tests/boundary/tests/binaries.rs` name the two new pieces; `.config/gate/wasm32.txt` names them
  beside `temper-client-domain`; `.config/fixture-readers.toml` follows (4.4).
- **Out of the way of plans in flight:** it lands between other landings in the crate, as
  `crate-graph.md`, 5.3 says, and goes before none that was marked first. Main is read afresh
  before it: a landing there in the crate that the goal's branch lacks is caught up first, so that
  the moves carry it.

## 6. The readers

After the move, as `../protocol/services.md`, 4.4 and `view-crates.md`, 6 have it: a package whose
own code needs only the forge's side, or the forge's side and the pages, and none of whose other
dependencies of the same kind reach `temper-client-domain`, depends on them instead. From the
readers at `a0ed35aa3c`:

- **`temper-client-tools`** reads the forge's side, `amend`, `key`, `owner` with `Fact` (3.1),
  `words`, `Limits` and `pages::delegation::merge_commit` (3.1): it depends on the forge's side, so
  that a change to the pages or the root rebuilds none of it.
- **`temper-engine-service-component` and `temper-engine-service-vocabulary`,** in their tests,
  read the forge's side's `forges`, `changes::panel` and `Limits`: the forge's side.
- **`temper-service`'s** dev-dependency names nothing of the crate: it goes.
- **`temper-client-browser-world`,** in its tests, reads `address::print`: the forge's side.
- **`temper-client-shell-cli`** reads the forge's side's `forges` and `sign_in`: the forge's side,
  once the tools no longer reach the root, where none of its other normal dependencies do.
- **The others** build or read a `Domain` (the view and its pieces, the view's kit, the TUI, the
  shells, the gallery, the client's world, `temper-engine-domain`'s fuzzy tests) or reach the root
  through another dependency, and keep `temper-client-domain`.

## 7. Later and open questions

- **Later: the tests that build a `Domain` in the pieces.** About 43,000 of the root's lines are
  tests that build a `Domain`, which `temper-client-domain` compiles again with any piece. A page's
  tests could drive its page through a stand-in `Routing` in the pages' piece. That changes how the
  tests set up, not what they check.
- **Later: the view's pieces on the domain's pieces.** The view's words read accounts and Me
  through `&Domain` in one function (`words::accounts::mark` in `temper-client-view-words`); through
  `&dyn pages::Root` they would need only the pages' piece, and a change to the root alone would
  rebuild none of the view's words.
- **The figure after landing** is `gate-compile.md`, 8's: each piece's units against
  `temper-client-domain`'s in its section 2, and whether the builds after it reused a piece.
