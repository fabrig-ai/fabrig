+++
id = "run-words"
part = "words"
kind = "feature"
needs = []
files = [
  "temper/crates/temper-client-view/src/pages/chat_line.rs",
  "temper/crates/temper-client-view/src/tests/chat_line.rs",
  "temper/crates/temper-client-view/src/words.rs",
  "temper/crates/temper-client-domain/src/pages/calls.rs",
  "temper/crates/temper-client-domain/src/tests/pages/calls.rs",
  "temper/docs/design/client/README.status.md",
]
effort = "high"
minutes = 80
+++

# The line's table words the tools a chat calls most

## Goal

A run of file and command calls reads as `read_docs` and `settings` read today: "Read 3 files, ran 2 commands", counted by what they touched, and "reading src/main.rs" while under way. Today `read` and `shell` print as their names, "read ×4, shell". The table is one function, so that the user's reading of it on a real chat moves one place.

Acceptance, each in `temper-client-view/src/tests/chat_line.rs` or `temper-client-domain/src/tests/pages/calls.rs`:

- **The table,** as `temper/docs/design/client/README.md`, 3.5, *its name*, has it:

  | Tools | Counts | Phrase, one and many | Alone | Under way |
  |---|---|---|---|---|
  | `read`, `Read` | distinct arguments | read 1 file, read N files | Read ARG | reading ARG |
  | `list`, `Glob` | distinct arguments | listed 1 folder, listed N folders | Listed ARG | listing ARG |
  | `write`, `Write` | distinct arguments | wrote 1 file, wrote N files | Wrote ARG | writing ARG |
  | `edit`, `Edit` | distinct arguments | edited 1 file, edited N files | Edited ARG | editing ARG |
  | `remove` | distinct arguments | removed 1 file, removed N files | Removed ARG | removing ARG |
  | `search`, `Grep` | calls | searched the files, searched the files N× | Searched for ARG | searching for ARG |
  | `shell`, `Bash` | calls | ran 1 command, ran N commands | Ran ARG | running ARG |

  `read_docs` and `settings` keep their words. Everything else, `read_file` among it, is its name, counted: the gallery's `chat-work-unknown` still reads "read_file ×2".
- **Distinct:** a call whose input can't be read counts one, each. The same path through three calls is one file.
- **Capitalised:** the first phrase of a run, where its tool is in the table; a tool's own name never.
- **Alone with nothing to say:** a table tool whose input can't be read says its counted phrase, "Read 1 file".
- **Under way with nothing to say:** a call under way whose input can't be read is the tool's name, never "reading " with nothing after it. Today `read_docs` with an unreadable input does that.
- **A cut input:** a `write` under way, its input past the live bound, `{"path":"src/a.rs","content":"aaaa`, still says "writing src/a.rs" and its row "write src/a.rs". `calls::touched` reads a cut input as far as its first member closes, where that member is a text or a number. A first member that doesn't close is `Touched::Other { argument: None }`.
- **A text argument is its first line:** `shell` with a multi-line command says and rows its first line. It applies to `Touched::Other` alone; a topic or a search stays as it is.
- **The terminal writes the "…":** where `words.shell` is `Shell::Terminal`, the description of a running line ends in "…"; the browser's stylesheet still adds its own, so a browser's description doesn't hold it. A test prints it through `skein_ui_text::render`: "Read 2 files: reading src/main.rs…".
- **Never a result's body:** a table tool's row, with a multi-line outcome, holds its first line alone, and the line's name holds none of it.
- **The status:** the **Not yet** line "3.5, *its calls*, its name, *while it runs* and *opened*" is gone from `temper/docs/design/client/README.status.md`.

## Inputs

- `temper/docs/design/client/README.md`, 3.5, *its calls*.
- `temper/docs/design/draft/chat-reading.md`, sections 2 and 7.
- `temper/docs/plan/chat-reading-b/notes/common.md`.

## Owns and may change

Owns the files in the header. In `words.rs`, only the new words' lines: the enum, `ALL`, the ids, the phrases, and `hole_bytes`' list of work arguments, which every new ARG word joins.

Readers, each only for the words this changes; each now has a table tool where it had a name:

- `temper/crates/temper-client-view/src/tests/chat.rs`, near lines 1378, 1393, 1408, 2185 and 2244: `read` with `{}` is now "Read 2 files", or "Read 1 file" alone.
- `temper/crates/temper-client-view/src/tests/task.rs`, near 1330 and 1345; `tests/document.rs`, near 550.
- `temper/tools/gallery/src/tests.rs`, near 1957 ("read ×2") and 2484 ("read ×4, shell"): the names they now print.
- The terminal's goldens, regenerated, where a line names a table tool: read each.
- Any other test naming the old phrase takes the new one. A test in `tests/chat_line.rs` that means "a tool with no words" takes a tool outside the table, such as `read_file`.

Options, each decided:

- **The table's shape is yours,** one function in `chat_line.rs` that `phrases`, `alone` and `under_way` all read, so no tool's name is compared outside it. The two lines today comparing `read_docs` and `settings` to capitalise go into it.
- **`read_docs`' and `settings`' special cases** (a search, a setting's kind) may stay in their arms of that function.
- **If a phrase's name passes 256 bytes,** it is cut as a name is today (`cut`).

**A change of meaning runs its readers' tests:** run `temper-client-view`, `temper-client-domain`, `temper-client-shell-terminal` and `temper-client-gallery`.

Not this task: a run's time, `Run::call`'s signature, `pages/chat.rs`, `agent.rs`, `temper.css`, design text.

## Steps

1. Red first: `file_tools_count_what_they_touched`, `a_tool_outside_the_table_is_its_name`, `one_call_alone_says_its_argument`, `a_call_under_way_says_its_present_form`, `an_unreadable_call_under_way_is_its_name`, `a_cut_input_says_its_first_member`, `a_command_is_its_first_line`, `the_terminal_writes_the_ellipsis`, `a_table_tools_row_holds_no_results_body`.
2. `calls.rs`: a text's first line; the cut input. Its test rows in `tests/pages/calls.rs`.
3. `words.rs`, then the table in `chat_line.rs`.
4. The readers above; regenerate the goldens and read them.
5. Each new test bites: break its rule once, see it fail for that reason, restore.
6. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-client-view -p temper-client-domain -p temper-client-shell-terminal -p temper-client-gallery --all-targets -- -D warnings`; the focused tests of those. Commit and push.

## Report

As the implementer's text asks, with what was read of the goldens, and any tool the worlds call that now reads oddly.
