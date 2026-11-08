+++
id = "tui-surface"
part = "tui-surface"
kind = "interface"
needs = ["screen-entries", "view-shared"]
files = ["temper/crates/temper-client-tui/src/**", "temper/crates/temper-client-shell-tui/src/host.rs", "temper/crates/temper-client-shell-tui/src/limits.rs", "temper/tools/journeys/src/tui/print.rs", "temper/tools/journeys/src/tui/print/tests.rs", "temper/docs/design/client/tui/journeys/theme.css", "temper/docs/design/client/README.status.md"]
effort = "high"
minutes = 210
+++

# tui-launchpad: the TUI's events, state, names, theme, words and view parts

## Goal

`temper-client-tui` gains everything the launchpad, the foot and a chat are built from,
with nothing drawn differently yet: its events and state, its look's names, its two
themes whole (`sel`, the regions and tokens with their marks, bands, alignments,
descriptions' paints, rows between children and the box's label), every word of its own,
the commands it offers, its tree's bound for a chat, and its view split into one module
a part, each a stub that draws today's first screen and bottom row, so that the tasks
`tui-foot`, `tui-launchpad` and `tui-chat` each fill their own files.

Acceptance:

- Every item of `temper/docs/plan/tui-launchpad/notes/contracts.md`, section 3, spelled as
  given there: `Event` (with `Quit` gone, the `QUIT` binding decoding to `Command {
  command: Exit }`), `Told`'s five new variants, `Tui`'s five accessors, `escape`,
  `View::words`, `offered`, `limits`'s new signature, the names' table in `look.rs`, the
  theme, the words.
- **`Told`:** `Size` sets `columns`; `Went` moves `focus` and `box_written`; `Cleared`
  moves `box_written`; `Aside` sets `aside`; `Unknown` sets `unknown`; each moves
  `revision` only where it changed what `Tui` holds. `focus` also moves wherever `openings`
  does today (the sign-in screen shown again), and once on the first `Told::Link` that is
  not `SignedOut`, so that the box asks the focus as the launchpad first opens.
- **`offered(root, words)`,** README.md, 6.14, *The commands*, *Offered*, a table test
  `the_tui_offers_its_commands` in a new `src/tests/commands.rs`, a row a case, each a
  failing test first: the launchpad with a usable picker (`/model /new /chats /theme
  /help /exit`) and with none (no `/model`); no project (`/theme /help /exit`); a chat
  with the model's efforts, Pause and Close offered and no delegate (`/model /effort
  /pause /close /new /chats /theme /help /exit`); the same chat with a delegate (no
  `/close`); never `/draft`, `/goal` or `/font` (a chat whose draft the browser would
  offer them on). Each built from `temper_client_view::shared::offered` and the screen.
- **The theme,** README.md, 6.15's tables, as contracts.md, section 3 sets each look:
  rows in `src/tests/theme.rs`, a row a region and a row a token for both schemes, each a
  failing test first: the palette's seven names in order with `sel`'s values; each
  region's place, arrangement, border (`tui-box`'s `sides: false`), padding; each token's
  paint, `align`, `band`, `mark`, `described`, `spaced` and `label`; `Theme::active` the
  `sel` background alone; `Theme::valid` true.
- **The view's parts:** `src/view.rs` keeps `View`, `build`, `event`, `escape`, signing in
  and `Binds`; `src/view/launchpad.rs`, `src/view/foot.rs` and `src/view/chat.rs` each hold
  `pub(crate) struct` of its data (`Launchpad`, `Foot`, `Chat`), `pub(crate) fn read(root:
  &Domain, tui: &Tui, words: &Words, wall: Wall) -> PART` and `pub(crate) fn draw(b: &mut
  Builder, part: &PART, words: &Words, binds: &mut Binds)` (`foot::draw` also returns the
  `Option<Event>` Esc makes). `view.rs` holds `pub(crate) enum Body { Launchpad(..),
  Chat(..) }`, `pub(crate) struct Signed { body: Body, foot: Foot }` and `pub(crate) fn
  signed(view: &mut View, signed: &Signed, out: &mut Queue<Patch>)`, which lays the whole
  signed-in tree from data alone (the page, the parts in order: the body's top, the
  body, the foot), so that a golden needs no root; `build` reads, then calls it. The
  stubs: `launchpad` draws today's first screen's top lines (`first`, moved there whole),
  `foot` today's bottom row ("q to quit" and the state), `chat` what `launchpad` draws, the
  root on a chat being unreachable until `host-launchpad`.
- **Nothing drawn moves:** every golden of `src/tests/golden/` is byte for byte as it was,
  and `screens.rs`'s and `event.rs`'s rows pass, but `event.rs`'s `q` row, which now gives
  `Command { command: Exit }`.
- **The tests' files for the parts,** each `mod` line in `src/tests.rs`: `src/tests/foot.rs`,
  `src/tests/launchpad.rs`, `src/tests/chat.rs`, each a module doc naming its task and no
  test yet; `src/tests/goldens.rs`'s comparison factored into `pub(crate) fn check(rows:
  &[Golden<'_>])` with `pub(crate) struct Golden<'a> { name: &'a str, tree: &'a
  skein_ui::Tree, scheme: Scheme, size: Size, focus: Focus, want: &'a [u8] }` (or the
  fields the comparison needs), its existing rows through it.
- **Its readers, compiler-forced:** `temper-client-shell-tui`'s `host.rs` (`Event::Quit`'s
  arm becomes `Event::Command { command: Command::Exit }`'s, closing; the other new
  events' arms do nothing yet; `host-launchpad` fills them) and `limits.rs` (passes the
  view's tree bound, `temper_client_shell_terminal::limits::view(&native)`'s
  `.0.tree`); `temper/tools/journeys`' print over a seven-colour palette, its committed
  `theme.css` reprinted by `heavy cargo run -p temper-client-journeys` with `sel` among
  its colours and nothing else of `temper/docs/design/client/tui/journeys/` changed.
- `temper/docs/design/client/README.status.md`: delete the **Not yet** entry "6.14, the
  TUI's state and events, and 6.15, `sel`, the regions, ..." (its bullet's lines alone),
  a stated departure from the implementer's text.

## Inputs

- `temper/docs/design/client/README.md`: 6.13 (*A key*, *The TUI's events*), 6.14 (its
  intro, *The TUI's state*, *Its words*, *The foot*, *The commands*), 6.15 (all of it),
  6.16 (*Its settings*).
- `temper/docs/design/client/tui/journeys.md`, 3.3 (the events' prints) and 6.4
  (`theme.css`).
- `skein/docs/design/ui-screen.md`, 3 (the theme's looks), 5.2 and 5.3.
- `temper/docs/plan/tui-launchpad/notes/common.md`;
  `temper/docs/plan/tui-launchpad/notes/contracts.md`, sections 0, 1, 2 (`offered`) and 3.

## Owns and may change

- Owns `temper-client-tui` whole for its wave, and the reader lines named above.
  `temper-client-shell-tui`'s `host.rs` and `limits.rs`: the compiler-forced lines alone;
  `host-launchpad` owns them next.
- **What later tasks own,** so that `tui-launchpad` and `tui-chat` run side by side after
  `tui-foot`: `src/view/foot.rs`, `src/tests/foot.rs` and goldens named `foot-*` and
  `first-*` are `tui-foot`'s; `src/view/launchpad.rs`, `src/tests/launchpad.rs`,
  `launchpad-*` goldens `tui-launchpad`'s; `src/view/chat.rs`, `src/tests/chat.rs`,
  `chat-*` and `question-*` goldens `tui-chat`'s. Every word, token, event and `Told` they
  use is written here: `words.rs`, `look.rs`, `theme.rs`, `event.rs`, `state.rs`, `view.rs`
  and `lib.rs` are closed to them but for the lines their specs name.
- `Words::english`'s capacities and `limits`' bound grow with the new words; the plural
  `Needing` takes the phrasebook's `PluralRule::English` forms (`One`, `Other`), as the
  view's `WaitingForDelegates` does.
- The marks, `described`, `spaced` and `label` are skein's `screen-entries` and
  `screen-label` fields, landed: if one is spelled otherwise there, take skein's and say
  so.
- `Binds`: a `List<Event>` the build fills, the binding's token its index plus 4;
  `event` returns a clone of the bound event for a press, and for the box (`BOX`)
  `Typed` on input and `Send` on submit.
- If a golden moves, something drew differently: find it; the goldens are not yours to
  change in this task.

## Steps

1. The rows (`commands.rs`, `theme.rs`, `state.rs`'s new `Told`s, `event.rs`'s `q` row);
   `TEMPER_BUNDLE=none heavy cargo nextest run -p temper-client-tui --lib`; see the new
   ones fail.
2. `event.rs`, `state.rs`, `look.rs`, `theme.rs`, `words.rs`, `limits.rs`; the view's split
   and stubs; `offered`; the readers' lines; `theme.css` reprinted.
3. The status entry.
4. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p
   temper-client-tui -p temper-client-shell-tui -p temper-client-journeys --all-targets --
   -D warnings`; `TEMPER_BUNDLE=none heavy cargo nextest run -p temper-client-tui -p
   temper-client-shell-tui -p temper-client-journeys`; `TEMPER_BUNDLE=none heavy cargo
   check -p temper -p temper-client-world --tests`.
5. Commit with the trailer `Look-request: tui-launchpad`, push.

## Report

As the implementer's text says; each row's failure seen; the goldens unchanged; the
`theme.css` lines that moved.
