# The view's crates

Provisional, 2026-11-13. From `temper/docs/design/draft/gate-compile.md`, 5, milestone C3 of
`temper/docs/design/draft/gate-compile.execution.md`: `temper-client-view` split by
`temper/docs/design/draft/crate-graph.md`, 5.2's rules, and landed as its 5.3 says. It says what
each piece holds and why, the preparation that lets every file go whole to one piece, where the
tests go, the change of moves, and the readers. What the view does is `view.md`'s: nothing here
changes a behaviour, a public path or a test's check.

## 1. In one page

- **Five crates for one view,** in `temper/crates/`, each a step crate: the vocabulary
  (`temper-client-view-words`), what pages draw with (`temper-client-view-parts`), the pages of a
  project's work (`temper-client-view-work`) and the pages that set temper up
  (`temper-client-view-setup`) side by side, and `temper-client-view` above them, the view itself
  (section 2).
- **Why there** (2.3). Of the 215 landings that changed the crate in the three weeks to
  `71ae32ef30`, 152 also changed `temper-client-domain`'s sources, below every piece, and 140 changed
  `words.rs`, which every page names. So a build of a landing rarely reuses a piece; what the split
  buys is smaller units side by side, and a piece's own tests built alone for work inside it. The
  figure after landing says whether the pieces were reused (`gate-compile.md`, 8).
- **Preparation first,** each change behaviour unchanged, so that every file goes whole to one
  piece: the vocabulary and the frame's bindings gathered below the pages (3.1, 3.2); the page
  items that another piece names moved to the parts' `common` module (3.3); every file naming an
  item at its home (3.4); the tests staged by the piece they go to (4.3).
- **Then one change of moves** (section 5): files moved and not edited but where the build forces
  it, so that git's rename detection carries branches in flight across it.
- **Each test goes with what it exercises,** to the lowest piece that holds everything it uses. The
  tests that build the whole view stay with `temper-client-view`. The tests' shared fixtures become
  a kit, `temper/testing/temper-client-view-kit` (section 4).
- **The readers move** where they need only the lower pieces: `temper-client-tui` to the words and
  the parts (section 6).

## 2. The crates

```
temper-client-view          the view: its state, build and act, the frame, the pages outside a project, the approval; every piece's items under today's paths
├── temper-client-view-work     the pages of a project's work: home, goals, a goal, tasks, changes, needs, the inbox, asks, chats, a chat and the terminal's look
├── temper-client-view-setup    the pages that set temper up: accounts, models and their checks, welcome, settings, how work runs, team, roles, ways, builders, the deployment, remembered and told
├── temper-client-view-parts    what pages draw with: bindings, the column, cards, choices, confirmations, flows, the forge's and the accounts' parts, the shared surface, views, the public pages and sign-in, and common, the page items another piece names
└── temper-client-view-words    the vocabulary: the phrasebook and its phrase families, the look's tokens and owner, the paths' drawing, the view's limits
```

### 2.1 What each holds

By today's files, under `temper/crates/temper-client-view/src/`, once the preparation of section 3
has landed. A directory goes with the file of its name unless a row says otherwise.

| Piece | Files |
|---|---|
| words | `words.rs` and `words/` (the phrase families: `words/accounts.rs`, `words/cards.rs`, `words/terminal.rs`, `words/forge.rs`, 3.1), `look.rs`, `paths.rs` and `paths/`, `limits.rs` |
| parts | `bindings.rs` (3.2), `column.rs` (3.2), `common.rs` and `common/` (3.3), `accounts.rs`, `cards.rs` and `cards/`, `choice.rs` and `choice/`, `confirm.rs`, `flow.rs`, `forge.rs` and `forge/`, `forge_sign_in.rs`, `shared.rs` and `shared/`, `views.rs`, `public.rs` and `public/`, `sign_in.rs` |
| work | `pages/` `agent`, `all_home`, `approval`, `ask`, `ask_page`, `change`, `changes`, `chat`, `chat_sheet`, `chats`, `decided`, `document`, `findings`, `goal`, `goals`, `home`, `home_cards`, `inbox`, `needs`, `new_chat`, `only_you`, `session`, `start_goal`, `task`, `tell`, `terminal` (with `terminal/commands.rs` and `terminal/question.rs`), `work` |
| setup | `pages/` `accounts` (with `accounts/rows.rs`), `account_programs`, `builders`, `claude_login`, `connect_model`, `deployment`, `how`, `model_check`, `models`, `remembered`, `roles`, `settings`, `sign_in_wait`, `team` (with `team/`), `told`, `welcome`, `welcome_copy`, `welcome_model` |
| `temper-client-view` | `lib.rs`, `root.rs`, `frame.rs`, `outside.rs`, `approve.rs`, `pages.rs` |

`pages/chat_line.rs` and `pages/ways.rs` go whole to the parts as `common/chat_line.rs` and
`common/ways.rs` (3.3). A page module of work or setup keeps its path in the view
(`temper_client_view::pages::home`), through `temper-client-view`'s `pages` module (2.4).

Their sizes, read by `tools/crates/design.md`, 2's rules, are each under the bound of
`crate-graph.md`, 7.3, estimated from today's files, code and own tests: about 13,000 and 1,500
lines for the words; 13,000 and 5,000 for the parts; 12,000 and 12,000 for work; 7,500 and 3,000
for setup; and 2,000 and 45,000 for `temper-client-view`, nearly all of it the tests that build the
whole view (4.1).

### 2.2 Its edges

- **The words name no piece;** the parts name the words; work and setup name the parts and the
  words, never each other; `temper-client-view` names all four. So work and setup compile side by
  side, and neither waits on the other.
- **Below them as today:** each piece depends on the external crates and workspace crates its own
  files name, from today's `temper-client-view` manifest (skein-lib, `skein-ui`, `jig-client`,
  `jig-client-protocol-wire`, `temper-client-domain`, `temper-client-protocol-wire`,
  `temper-protocol-reaches`), and on no other. No piece links io, HTTP, smith or an engine crate
  (`README.md`, section 2).
- **Step crates,** each under `temper/crates/`, on the step profile (`[lints] workspace = true`,
  temper's `clippy.toml`), `no_std` but under test, in the strict subset (`AGENTS.md`, 2).

### 2.3 Why there

From the crate's history, `crate-graph.md`, 5.2's first rule, and its module graph, its fourth:

- **The history.** Over the three weeks to `71ae32ef30`, 215 landings changed the crate, a module's
  own tests counted with it. `words.rs` changed in 140, the frame in 81, the look in 69, a chat's
  page in 57, the cards in 50, the choices in 49, home in 49, the forge's parts in 48, the root in
  47, welcome in 44, connecting a model in 43. The vocabulary changes with everything: the frame
  with the words in 71, the look with the words in 64. The pages cluster: welcome with connecting a
  model in 27 and with accounts in 25, accounts with connecting a model in 21; a chat with the chats
  in 26 and with home in 22.
- **Below every piece:** 152 of the 215 also changed `temper-client-domain`'s sources, which every
  piece names, and most of the other 63 changed `jig-client` or `skein-ui`. Modelled on those 63, a
  piece would have been reused in about 7 builds for work, 11 for setup, 15 for the parts and 31 for
  the words. A split saves only in such builds (`gate-compile.md`, 5): this one is measured before
  the next starts (`gate-compile.execution.md`, 2).
- **One graph today.** Every module but `pages.rs` lies on a cycle: the words name the frame's
  `owner`, the frame names the words; cards, choices and the shared surface name pages, and the
  pages name them. So a cut needs its preparation (section 3).
- **Work is not cut further.** A project's work pages and its conversations name each other in
  about twenty places (a task's page and the chats' rows, a chat's page and the inbox's cards and a
  task's header, an agent's page and a chat's) and change together. Setup's pages name work's in two
  places and work's name setup's in fourteen, which section 3.3 moves to the parts.
- **The vocabulary below, though it changes often:** every page names its words and the look's
  tokens. No cut avoids that, as for the engine's limits (`../protocol/services.md`, 4.1). A
  phrasebook split by piece is Later (section 7).

### 2.4 Their roots

- **A piece's root declares its own modules and re-exports, under the same names, the lower
  pieces' modules its files name at `crate::`,** so that a moved file compiles unedited: the parts'
  root `pub use temper_client_view_words::{look, paths, words, limits};`, and work's and setup's the
  words' and the parts' modules they name. Work's and setup's `pages` module declares its own pages
  and re-exports `common`'s `project_name` and `serves_pages` (3.2).
- **`temper-client-view` keeps its name and every public path** (`crate-graph.md`, 5.3): its root
  re-exports `words`, `look`, `paths`, `limits`, `Limits`, `valid`, `cards`, `forge`,
  `forge_sign_in`, `public`, `shared` and `sign_in` from their pieces, and keeps `frame`, with
  `owner` and `link_region`, and `View`, `build`, `act`, `act_event`, `event`, `href`,
  `opening_tab`, `shell_event` and `ShellEvent`. Its `pages` module re-exports work's and setup's
  page modules, `chats`, `goals` and `home` public as today.
- **An item another piece names is `pub`** in its piece; nothing else widens.
- **The card methods on `Words` are a trait of the parts.** `cards.rs`'s `impl Words` (`say_card`,
  `card_text`, `status`, `refusal`, `outcome`, `ending`) can't stay an inherent impl once `Words` is
  the words' type (E0116), and its drawing belongs to the parts, not the words. It becomes
  `pub trait CardWords` in `cards`, implemented for `Words` with the same names, signatures and
  bodies; each file that calls one adds `use crate::cards::CardWords as _;` (a forced edit), and
  no call site changes. No reader outside the view's crates calls them.

## 3. The preparation

Each preparation lands in `temper-client-view` before the move, behaviour unchanged, every test
checking what it checked. A moved item takes its doc comments, attributes and private helpers with
it.

### 3.1 The vocabulary gathered

- **The phrase families** move under `words/`: `accounts/words.rs` to `words/accounts.rs`,
  `cards/words.rs` to `words/cards.rs`, `pages/terminal/words.rs` to `words/terminal.rs` and
  `forge/words.rs` to `words/forge.rs`, each a module of `words`. The module that held one keeps its
  path by a re-export (`accounts::words` is `crate::words::accounts`).
- **`frame::owner`** moves to `look.rs`; `frame` re-exports it, a public path.
- **`choice::MODEL_SHOWN` and `choice::shown`** move to the words, and **`accounts::owner`** to
  `words/accounts.rs`; their modules re-export them.
- Then the words' files, `look.rs`, `paths.rs` and `paths/`, and `limits.rs` name no other module of
  the crate.

### 3.2 The bindings and the column

- **`bindings.rs`,** a module of its own: the frame's `Bindings` with its methods and its fields,
  its `Act` with `decode_approve`, `root::ShellEvent`, the frame's helpers the pages use (`insert`,
  `reading`, `block_submissions`, `remembered_field`, `remembered_return`, `model_times`,
  `ask_address`, `forge_mark`, `account_mark`), and the root's `link`.
- **`column.rs`,** a module of its own: `outside`'s `column`, `words` and `box_line`, and the frame's
  `unavailable`, which the choices name.
- **`common.rs`:** `pages.rs`'s `project_name` and `serves_pages`.
- **`choice`** names `temper_client_domain::pages::chats::ended` directly, not through the chats'
  page.
- `frame`, `outside`, `pages` and the root keep what they still use by `use` lines, and the root
  re-exports `ShellEvent`, a public path.

### 3.3 Common: the page items another piece names

Each goes from `pages/<page>.rs` to `common/<page>.rs`, a module of `common` named for the page it
came from, with what it needs of its page that no caller in the page's own piece keeps alone. The
page re-exports what it moved for its own piece's callers, and `pub` where the page's path is public
(`pages::goals::ways`, which `temper-client-tui` reads). From the module graph at `71ae32ef30`:

| From | Items another piece names |
|---|---|
| `change` | `say` |
| `terminal`, `terminal/commands.rs` | `shown`, `kicker`, `details`, `running_words`; `commands::{description, offered, note, confirms}`, with `efforts` and `help` |
| `goals` | `ways`, `Way`, `way_name` |
| `inbox` | `card_key`, `waiting_since`, and `source_task`, `task_title`, `card_words` for the asks' reading |
| `chats` | `working`, `status_name`, with `active`, `status_words`, `words_not_begun` |
| `chat_line` | the whole page, `Run` and its helpers |
| `home` | `intro_words`, `need_words`, `need_sentence`, `number_word`, with `Intro`, `kicker`, `needs_sentence` |
| `all_home` | `headline`, with `landed`, `headline_words` |
| `chat` | `short_title`, with `SHORT_TITLE`, `title_space`, and `asking` |
| `ask` | `read`, `Kind`, with `Asked`, `read_parts`, `bounded_words`, `person_in_words`, `title_for`, `need_kind`, `first_sentence`, `need_title` |
| `approval` | `address`, `review_words`, with `forge`, `question` |
| `connect_model` | `label`, `command`, with `names`, `named`, `say`, `description`, `command_prompt`, `printed_command` |
| `accounts`, `accounts/rows.rs` | `unloaded`, `usable_account`, `account_words`, `home_accounts`, `first_usable_kind`, `named`, with `run`, `first_usable`, `provider_name`, `account_heading`, `heading`, `rows::playground` |
| `settings` | `path`, `way_path`, `FlowTab`, `FlowWay`, `open`, `open_alone`, `Tab`, `button_to`, `quiet_band`, `section_head`, with `flow_path`, `text`, `deployment`, `project_name` |
| `how` | `Who`, `chip`, with `head`, `ask`, `project_name`, `text` |
| `ways` | the whole page |
| `model_check` | `stopped`, with `row_trial`, `Trial`, `First`, `State`, `Answer`, `account_line`, `first_source`, `sign_in`, `identity_detail`, `failure`, `named` |
| `team/areas.rs` | `named_holder`, with `role`, `role_name` |

A row's "with" items are those the named ones need; one a page's own code still uses stays in reach
through the page's re-export. An item that turns out to need something of a higher piece takes it
along, or the page item and its caller stay together in work or setup, whichever keeps every edge
of 2.2; the change that moves it says which.

### 3.4 Every file names its items at their home

- **A file of the words, the parts, work or setup** names each item of the crate at a module of its
  own piece or of a lower one (2.2), never through a module of a higher piece or of its sibling:
  `crate::bindings::Bindings`, not `crate::frame::Bindings`; `crate::common::goals::ways` from the
  parts or setup.
- **An old path keeps a re-export** only for callers in its own module's piece and for today's
  public paths (2.4). A re-export no caller uses goes.
- **Tests** name items as their files do, or through a page's re-export within the piece the test
  goes to (4.1).

## 4. Its tests

### 4.1 Where each test goes

- **To the lowest piece that holds everything it uses,** through that piece's dependencies and
  dev-dependencies, as `../protocol/services.md`, 4.3 has it: a test of the words' phrases or the
  look's tokens to the words, of a card or a choice drawn into a builder to the parts, of a page
  drawn into a builder to work or setup.
- **A test that builds the whole view,** through `View`, `build`, `act` or the fixtures that hold a
  `View`, stays with `temper-client-view`: today about 45,000 of the crate's 67,000 lines of tests.
- **No piece's tests use a crate that depends on that piece,** which would give them two copies of
  its types.
- **A test file whose tests go to two pieces splits,** each test kept whole and each assertion kept.
  Inline test modules stay with their files. `tests/suite/` stays with `temper-client-view`.
- **What each test checks stays,** and its name with it.

### 4.2 The kit

`temper/testing/temper-client-view-kit`, the library `temper_client_view_kit`: the tests' shared
fixtures, compiled once for every piece's tests (`tools/crates/design.md`, 3.2).

- **What it holds:** today's `src/tests/fixtures.rs`, `fixtures/task.rs` and `fixtures/panel.rs`
  but for what needs a piece above the words: the words' and limits' builders, the tree's readers
  (`text`, `main_text`, `find`, `person_find`, `styles`, `frame_statuses` and their kin), the
  domain's builders (`root_at` and its kin, the inbox's, chats', tasks' and panels'), the named
  constants, and the domain-only helpers that test modules share today.
- **What it doesn't:** the builders of a `View` (`view`, `built`, `build`, `page_view` and their
  kin) stay with `temper-client-view`'s tests; a fixture that draws with a piece's code (a part's
  text drawn by the forge's parts, a chat's tree, a closing card) goes with the tests of that piece.
- **Its form:** the `harness` profile with a `clippy.toml` of its own, `[lib] test = false`, no
  `#[test]` and no `cfg(test)` code; a helper that was `pub(super)` becomes `pub`. It depends on the
  words and on what its fixtures name today (`temper-client-domain`, skein-ui, `skein-ui-text`,
  `skein-fake-person`, `temper-fixtures`, `jig-client-test-records`, `jig-client-protocol-link`
  among them).
- **The words' tests** can't use the kit, which depends on the words: they keep a private copy of
  the few fixtures they use, trimmed to them.

### 4.3 Staged before the move

So that the move only moves, the tests are staged in `temper-client-view` by the piece they go to,
each stage a `#[cfg(test)]` module of the crate's root:

| Stage | Becomes |
|---|---|
| `src/view_kit.rs` and `src/view_kit/` | the kit's `src/lib.rs` and `src/` |
| `src/words_tests.rs` and `src/words_tests/` | the words' `src/tests.rs` and `src/tests/` |
| `src/parts_tests.rs` and `src/parts_tests/` | the parts' |
| `src/work_tests.rs` and `src/work_tests/` | work's |
| `src/setup_tests.rs` and `src/setup_tests/` | setup's |
| `src/tests.rs` and `src/tests/` | stay, `temper-client-view`'s |

- **A staged file names its fixtures and its siblings through `super::`,** never by its stage's
  path from the crate's root, so that it compiles unedited under `tests`.
- **No stage names another stage's tests or fixtures, and neither does `src/tests/`:** they become
  another crate's `cfg(test)` code at the move, out of reach. A helper that draws with a piece's
  code and that two stages' tests share is copied into each, trimmed to what that stage's tests use
  (as the words' stage keeps its copy, 4.2). Within one stage, a file names a sibling's helper
  through `super::<sibling>::`, or keeps its own copy when the sibling is staged by a change running
  beside it.
- **Each stage has its `fixtures` module:** the words' stage its trimmed copy; the others the kit's
  items, re-exported, and the fixtures that draw with their piece.
- **The kit's stage names only the words' modules** (`crate::words`, `crate::look`,
  `crate::paths`) and crates outside the view.

### 4.4 Data and paths

- **`src/tests/release-card.txt`** goes with the test that reads it, and that test reads it from its
  own package.
- **The words' declarations test,** which reads `src/words.rs` from its package, goes to the words.
- **`include_str!` of `temper-client-shell-web`'s `temper.css`** stays relative to the including
  file, at the same depth in each piece.
- **`.config/fixture-readers.toml`** names the kit and each piece's fixtures, with their readers.

## 5. Landing the split

One change of moves, after the preparation (`crate-graph.md`, 5.3):

- **Each file moved whole** to its piece, by 2.1 and 4.3, and not edited but where the build forces
  it, in a commit after the moves: a `mod` line naming a file now in another piece becoming its
  re-export, a visibility (2.4), what a newly public item's lints ask (`#[must_use]`, `Debug`), a
  path that `include_str!` or a test reads, and a crate-level `expect` its piece no longer meets.
- **New files:** each piece's manifest and root; work's and setup's `pages.rs`; the kit's manifest
  and `clippy.toml`; `temper-client-view`'s root, `pages.rs` and manifest, re-exporting (2.4); the
  workspace's members and dependencies.
- **The checks follow:** the size check's exception for `temper-client-view` goes
  (`tools/crates/design.md`, 3.1), each piece under the bound; the `temper` binary's packages in
  `tests/boundary/tests/binaries.rs` name the four new pieces; `.config/gate/wasm32.txt` names them
  beside `temper-client-view`; `.config/fixture-readers.toml` follows (4.4).
- **Out of the way of plans in flight:** it lands between other landings in the crate, as
  `crate-graph.md`, 5.3 says, and goes before none that was marked first.

## 6. The readers

After the move, as `../protocol/services.md`, 4.4 has it: a library whose own code needs only the
words and the parts, and none of whose other normal dependencies reach `temper-client-view`,
depends on them instead, and its tests too where they need no more. From the readers at
`71ae32ef30`:

- **`temper-client-tui`** reads `Limits`, the words, the look, `paths::WORDS`, the shared surface
  and `pages::goals::ways`, which 3.3 moves to the parts: it depends on the words and the parts, so
  that a change to a page of work or setup, or to the view's root, rebuilds none of it.
- **`temper-playground-page` and `temper-playground-world`** read `sign_in::Ways` alone: the parts,
  where none of their other dependencies reach `temper-client-view`.
- **The shells, the gallery, the client's world and `temper-engine-domain`'s tests** build a `View`
  and keep `temper-client-view`.

## 7. Later and open questions

- **Later: the whole-view tests in their pieces.** About 45,000 of the view's lines are tests that
  build the whole view, which `temper-client-view` compiles again with any piece. A page's tests
  could build its page alone, through a harness that stands in for the view's frame and acts, in
  the page's piece. That changes how about 1,300 tests set up, not what they check.
- **Later: a phrasebook per piece.** Each piece's words as a family of its own, as the accounts'
  and the cards' are, so that a new phrase rebuilds its piece and the view alone.
- **The figure after landing** is `gate-compile.md`, 8's: each piece's units against the view's row
  in its section 2, and whether the builds after it reused a piece.
