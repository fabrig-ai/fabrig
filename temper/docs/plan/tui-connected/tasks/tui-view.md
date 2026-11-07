+++
id = "tui-view"
part = "tui"
kind = "feature"
needs = ["screen-surface"]
files = [
  "temper/crates/temper-client-tui/**",
  "Cargo.toml",
  "Cargo.lock",
]
effort = "high"
minutes = 180
+++

# temper-client-tui: the TUI's state, words, view and theme

## Goal

The step crate of temper's TUI: its own state beside the client root, the
events its view makes of a person's acts, its phrases beside the view's,
its two screens as skein-ui's tree, its look's names and its two themes,
and its tree's limits. The host (`tui-host`) and the goldens
(`tui-goldens`) build on it; nothing calls it yet.

Acceptance:

1. `temper/crates/temper-client-tui` exists: a member of the root
   `Cargo.toml` and an entry of its `[workspace.dependencies]` beside
   `temper-client-view`; `no_std` with `alloc` (`#![cfg_attr(not(test),
   no_std)]` as `temper-client-view`'s `lib.rs`), `#![forbid(unsafe_code)]`;
   its lint block from `python3 -I -B tools/lints/lints.py --set step
   temper/crates/temper-client-tui` (`[lints] workspace = true`), under the
   tree's `temper/clippy.toml`; depending on `skein-ui`, `skein-ui-screen`,
   `skein-lib`, `jig-client`, `temper-client-domain` and
   `temper-client-view` alone (`notes/contracts.md`, section 1).
2. Its surface is `temper/docs/plan/tui-connected/notes/tui.md`, section
   1, with those names, fields and signatures: `look`, `words`, `theme`,
   `limits`, `Tui`, `Told`, `Signing`, `Refusal`, `Event`, `View`, `build`,
   `event`, `Scheme`, `theme`, `limits`.
3. `Tui::step` does what `notes/tui.md`, 1.2's table says, row by row.
4. `build` lays the two trees of `notes/tui.md`, section 2, with section
   3's phrases and the view's, and the sign-in line and the state's word
   by their tables; it builds only when the root's `shown` or the TUI's
   revision moved.
5. `event` gives `Typed` for `Input` on the link's field, `SignIn` for its
   `Submit`, `Quit` for `Press` on "q to quit", and `None` for every other
   act and node, reading the view alone.
6. `theme(Dark)` and `theme(Light)` are section 4's, and `valid()`; every
   region and token of `look::NAMES` has its look; `skein_ui::look::check`
   finds no fault in `[look::NAMES, temper_client_view::look::NAMES]`.
7. `limits(&domain, 512)` is `Some` and `skein_ui::valid` for the test
   fixture's domain limits (copied from `temper-client-view`'s
   `fixtures::limits()`); a first screen with `client.domain.projects`
   projects, each name `client.domain.text` bytes, and a sign-in screen
   with a 512-byte field and its longest line, each build within it. The
   shipped limits' check is the host's (`tui-host`).
8. Step tests in `src/tests.rs` (and `src/tests/*.rs` modules), each named
   in the report, all rows of tables: section "Steps" below.

## Inputs

- `temper/docs/plan/tui-connected/notes/common.md`.
- `temper/docs/plan/tui-connected/notes/contracts.md`, sections 1 and 2
  (`skein-ui-screen`'s theme types): they bind.
- `temper/docs/plan/tui-connected/notes/tui.md`, sections 1 to 4: they
  bind.
- `temper/docs/design/client/README.md`, 6.14 (the pages, the state, the
  words) and 6.15 (the theme); section 2's bullets on the crate.
- `skein/docs/design/ui-screen.md`, section 3 (the theme), 5.2 and 5.3
  (how regions and a spread row are laid), 5.4 (a field, a secret, the
  focus request).
- `skein/docs/design/ui.md`, section 9 (phrases), 6.1 (shortcuts), section
  8 (tokens and region names), 3.2 (the focus request).
- The models to copy from: `temper/crates/temper-client-view/src/words.rs`
  (a `Word` enum, its English table, `Phrasebook::load`, `say`),
  `temper/crates/temper-client-view/src/look.rs` (`NAMES`),
  `temper/crates/temper-client-view/src/frame.rs` (the link's state as
  words, `unavailable_main`'s refusals: `pub(crate)`, so the TUI copies
  their matches, not their code), `temper/crates/temper-client-view/src/root.rs`
  (`View`, `build`, `event`).
- The view's test fixtures, to copy into this crate's:
  `temper/crates/temper-client-view/src/tests/fixtures.rs`'s
  `root_at_with_values` (an unlinked root started with `With::Transport`,
  `Up::Served`, `me` delivered on its watch) and `fixture_me`, and
  `temper/crates/temper-client-view/src/tests/frame.rs`'s tests that find
  "Offline" and "Out of date" (how each state is made).

## Owns and may change

**Owned:** `temper/crates/temper-client-tui/**`; the root `Cargo.toml`'s
member line and `[workspace.dependencies]` entry; `Cargo.lock`'s entry.

**Readers and neighbours:** none: no crate names this one yet. No change to
`temper-client-domain` or `temper-client-view` (`notes/common.md`).

**Options, each with its default:**

- Dev-dependencies for the tests: `temper-client-protocol-wire` (for
  `FORGE`, the connector the view's `Words::english` takes) and whatever
  `temper-client-view`'s fixtures use to make `me` (its `[dev-dependencies]`):
  copy those lines. If a fixture needs a `temper-client-domain` item that is
  `pub(crate)`, make the root the same way through its public events, and
  say so.
- The test fixtures in one file, `src/tests/fixtures.rs`. If you make a
  `src/tests/fixtures/` directory instead, add its entry to
  `.config/fixture-readers.toml` as `temper-client-domain`'s is (`kind(lib)`).
- `Words::english`'s phrasebook limits: the TUI's phrases' holes bounded by
  the domain's limits (`client.domain.text` for a name, the projects' line
  `client.domain.projects` names joined, an address at 64 bytes). If the
  joined projects' line passes a `u32` or the phrasebook's hole bound, cut
  the line's names to those that fit and say so.
- The field's `written` count: the state's own writes (the emptying at
  `Asked` and at `Link`), kept in `Tui`. If skein-ui's diff needs it
  otherwise, follow skein-ui and say so.
- `shown`: the root's `temper_client_domain::shown` plus the TUI's
  revision, saturating. If a change of the link's state or of
  `not_signed_in` moves neither (find it in a test), build on every call
  and say so.

## Steps

1. The crate: manifest, `lib.rs` citing `temper/docs/design/client/README.md`,
   6.11 to 6.16, at module level, the modules of `notes/tui.md`, section
   1.
2. `look` and `theme`: the owner, regions, tokens and bindings' tokens;
   `NAMES`; the two themes from section 4's table.
3. `words`: section 3's phrases under `look::OWNER`, the view's `Words`
   beside them.
4. `state` and `event`: section 1.2 and 1.3.
5. `view`: section 2's trees: one `Page`, `Main` with `TOP`, `ContentInfo`
   with `BOTTOM`; the sign-in screen while the link is `SignedOut`, the
   first screen otherwise; the field secret, submitting on Enter, focused
   by `tui.openings()`; the button bound to `QUIT` with the shortcut `q`
   (`Modifier::Plain`).
6. `limits`: the bound of acceptance 7, from the trees' shape.
7. The tests, as tables (`src/tests.rs` and modules beside it):

   ```rust
   #[test]
   fn the_states_steps_are_its_table() {
       // one row per line of notes/tui.md, 1.2's table: a Tui in a starting
       // state, a Told, and what signing(), field(), signed_out(), unkept(),
       // openings() and revision() read after it
   }
   #[test]
   fn the_sign_in_screen_says_each_line() {
       // rows: (Signing, not_signed_in made on the root or none, the line's text, its token);
       // each builds the sign-in screen and finds that one paragraph's text and style;
       // Idle has none; signed_out() adds "Signed out." above the address line
   }
   #[test]
   fn the_first_screen_says_the_links_state_and_me() {
       // rows: Live with me ("● Live", LIVE, "Connected to 127.0.0.1:8367 as Owner",
       // "Projects: Hello · Demo"), Live without me ("● Connecting", "Connecting to …"),
       // Offline with me (me still drawn, "● Offline", DOWN), Out of date (OutOfDateMain,
       // "● Out of date"), me with no project ("No project yet"), unkept (the alert)
   }
   #[test]
   fn each_act_is_its_event() {
       // Input on the field: Typed; Submit on it: SignIn; Press on "q to quit": Quit;
       // Press on the field, Input on the button, an unbound node: None
   }
   #[test]
   fn the_themes_are_valid_and_name_every_look() { /* acceptance 6 */ }
   #[test]
   fn the_trees_fit_their_limits() { /* acceptance 7 */ }
   ```

   Bite: make `Told::Link` leave the field once, and drop `Busy`'s row of
   the sign-in line's match once; see their rows fail, and restore.
8. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo
   clippy -p temper-client-tui --all-targets -- -D warnings`; `heavy cargo
   nextest run -p temper-client-tui --lib`. Commit ("temper-client-tui: the
   TUI's state, words, two screens and themes"), push.

## Report

As the implementer's text says, and: how the projects' line is bounded,
how `shown` is made, the tests' time, and anything in the trees you laid
otherwise than `notes/tui.md`, section 2, with why.
