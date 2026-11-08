+++
id = "view-shared"
part = "view-shared"
kind = "interface"
needs = []
files = ["temper/crates/temper-client-view/src/shared.rs", "temper/crates/temper-client-view/src/lib.rs", "temper/crates/temper-client-view/src/pages/chat_line.rs", "temper/crates/temper-client-view/src/pages/terminal.rs", "temper/crates/temper-client-view/src/pages/terminal/words.rs", "temper/crates/temper-client-view/src/pages/terminal/commands.rs", "temper/crates/temper-client-view/src/pages/home.rs", "temper/crates/temper-client-view/src/pages/chats.rs", "temper/crates/temper-client-view/src/pages/chat.rs", "temper/crates/temper-client-view/src/choice/picker.rs", "temper/crates/temper-client-view/src/choice.rs", "temper/crates/temper-client-view/src/tests.rs", "temper/crates/temper-client-view/src/tests/shared.rs", "temper/crates/temper-client-view/src/tests/chat_line.rs", "temper/docs/design/client/README.status.md"]
effort = "high"
minutes = 200
+++

# tui-launchpad: the view's shared words, and `see`'s phrases in a run's line

## Goal

`temper-client-view` gains a public module, `shared`, that says as words and data what
the browser's chat terminal says and the TUI says too: Home's headline, a chat's state, a
need's line, a run's line, the running line, the picker's closed line, a chat's short
heading, the commands' words and when each is offered, the notes' words and the
terminal's own phrases. Each is the function the browser's builder then builds from, so
the browser's trees stay byte for byte as they are. A run's line also names Ask temper's
`see` calls by their address, for the browser and the TUI alike.

Acceptance:

- `temper_client_view::shared` holds every item of
  `temper/docs/plan/tui-launchpad/notes/contracts.md`, section 2 marked *view-shared*,
  spelled as given there, each documented with its design section.
- The browser's builders build from them: `chat_line::Run::flush` from `Run::line`;
  `terminal::chat_running` from `shared::running`; `terminal::under`'s quiet line from
  `shared::say` and `shared::closed_line`; `commands::under` and `commands::notes` from
  `shared::description`, `shared::offered` and `shared::note`. Home's intro uses the
  function `shared::home_headline` wraps; Home's chat rows the one `shared::chat_state`
  wraps; Home's lines under its first card the one `shared::need_line` wraps. Every
  existing test of `temper-client-view` passes unchanged: no golden, gallery state or
  table row moves.
- `see`'s phrases, `temper/docs/design/client/README.md`, 3.5, *its calls*, the table
  under *its name*: a phrase for each first word of the call's address, in the order
  first called; counted, alone and under way as the table has them; `more` adding no
  phrase beside another `see`; another first word or an unreadable address `see`'s
  name counted. The address is `calls::touched`'s `Touched::Other { argument }` (the
  input's first text member); `temper-client-domain` is not changed.
- Rows, each a failing test first:
  - `src/tests/chat_line.rs`: a new table test `see_says_its_addresses_first_word`, a
    row a case: `see changes` then `see change 1`, `2`, `3` is "Listed changes, read 3
    changes"; `see change 3` alone "Read change 3"; `see change 3` twice "Read 1 change",
    one distinct number, in the singular as `read_docs`'s "read 1 doc"; `see task 9` and
    `see task 9 tree` "read 1 task" with another phrase before it; `see goals` twice
    "listed goals 2×" after another phrase; `see help hold` alone "Read the page hold";
    `see help search landing` alone "Searched the docs for landing"; `see more x1` after
    `see changes` "Listed changes"; `see more x1` alone "Read on"; `see frobnicate` "see";
    a call under way of `see change 2` "reading change 2…" in the terminal's words.
  - `src/tests/shared.rs`, new, `mod shared;` in `src/tests.rs`: one table test,
    `shared_words_are_the_browsers`, whose rows build the browser's tree with the view's
    existing fixtures (`src/tests/fixtures.rs`, `terminal.rs`'s and `home.rs`'s roots
    where they serve) and check that each shared function's bytes are the words the tree
    holds: `say(Commands)` "/ for commands"; `closed_line` the quiet line's button name;
    `running` the running status's name, and none where the browser draws none;
    `description` and `offered` for each command of `Command::ALL` against the commands
    menu's items on a chat and on `NewChat`; `note` for `Unknown` and `Closed` against
    the notes drawn; `home_headline` against Home's level-1 heading for a brief-less Home
    and a minute's tick; `chat_state` against a Home chat row; `need_line` against a
    Home line; `heading` against a chat's bar's heading.
- `temper/docs/design/client/README.status.md`: delete the **Not yet** entry "3.5, *its
  calls*, and 6.14, *Its words*: `see`'s phrases in a run's line, and the view's shared
  surface's words, ..." (its bullet's lines alone, not the blank lines around it), a
  stated departure from the implementer's text: the design is written, and its status
  must say what is built when the branch lands.

## Inputs

- `temper/docs/design/client/README.md`: 3.5, *A chat's conversation* (*its calls*, the
  run's line and `see`'s table), *Home on any project* (*the headline*), *Home*'s
  *Needs you* and *Chats*, *A chat's terminal* (*The running line*, *The box*, *the
  quiet line*, *Commands*, *The terminal's notes*); 6.14, *Its words*.
- `temper/docs/design/draft/ask-temper.md`, 5.1 (`see`'s addresses).
- `temper/docs/plan/tui-launchpad/notes/common.md`;
  `temper/docs/plan/tui-launchpad/notes/contracts.md`, sections 0 and 2.

## Owns and may change

- Owns the files in `files`, in `temper-client-view` alone. Keep off `src/frame.rs`,
  `src/look.rs`, `src/words.rs` and `src/tests/rail.rs`: another draft is changing them.
  `TerminalWord`'s phrases are registered by `words.rs` already; making the enum `pub`
  changes no line there. If a function you share needs a line in one of the four
  files, stop and say so in the report rather than edit it.
- Readers to keep compiling and passing (`rg` found them): `chat_line::Run` in
  `pages/chat.rs` (the conversation's walk), `src/tests/chat_line.rs` and
  `src/tests/terminal.rs`; `chats::status_words` and `status_name` in `pages/chats.rs`,
  `pages/chat.rs`, `pages/terminal.rs`, `pages/home.rs`; `choice::picker::closed_line` in
  `choice.rs`, `pages/terminal.rs` and `src/tests/choice.rs`; `commands::description` and
  `offered` within `commands.rs`; `pages::chat::short_title`. Their signatures may change
  where the share needs it, each reader edited in the same commit.
- Other crates read nothing of this yet (`temper-client-tui` reads `words` alone); the
  task `tui-surface` reads `shared` next.
- `Times` (`words.rs`) is crate-private: `shared::home_headline` makes its own from `now`
  and returns the tick it computed. If Home's intro needs a `Times` field it doesn't
  have, keep the headline's composition in `pages/home.rs` as one `pub(crate)` function
  both call.
- `Run::line` empties the run as `flush` does; `flush` keeps its signature and builds its
  disclosure from the `RunLine`. `RunLine::failed` is the count `flush` reads for its
  styles.
- **`see`'s phrases and `words.rs`:** the run's other phrases are `Word::Work*` of
  `words.rs`, which is closed here. Register `see`'s phrases through an extension point
  outside it, as `pages/terminal/words.rs`'s `extend` registers the terminal's (that
  function, or one beside it that it calls), with their number holes and a singular for
  one ("read 1 change", "read 1 task"). If that needs a line of `words.rs` (a capacity
  or a count fixed there), build them instead as bytes in `chat_line.rs`, literal words
  with numbers by `skein_lib::Decimal`, as `commands::description`'s are literal, and say
  so in the report.

## Steps

1. The rows; `TEMPER_BUNDLE=none heavy cargo nextest run -p temper-client-view --lib -E
   'test(/chat_line::/) | test(/shared::/)'`; see them fail for their reasons.
2. `see`'s phrases in `chat_line.rs`'s table (counted, alone, under way), then `Run::line`
   and `flush` over it.
3. `src/shared.rs` and `pub mod shared;`; each browser builder over its shared function.
4. The status entry.
5. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p
   temper-client-view --all-targets -- -D warnings`; `TEMPER_BUNDLE=none heavy cargo
   nextest run -p temper-client-view -p temper-client-tui -p temper-client-shell-terminal`;
   `TEMPER_BUNDLE=none heavy cargo check -p temper-client-shell-web -p
   temper-client-gallery -p temper-client-world --tests`.
6. Commit with the trailer `Look-request: tui-launchpad`, push.

## Report

As the implementer's text says; each row's failure seen; each shared function and the
builder that now builds from it; any line of the four closed files you needed.
