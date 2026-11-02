+++
id = "directory-words"
part = "client-words"
kind = "feature"
needs = []
files = [
  "temper/crates/temper-client-shell-cli/src/directory.rs",
  "temper/crates/temper-client-shell-cli/src/host/input.rs",
  "temper/crates/temper-client-shell-cli/src/tests.rs",
  "temper/crates/temper-client-shell-cli/src/tests/fixtures.rs",
  "temper/crates/temper-client-shell-cli/src/tests/moves.rs",
  "temper/crates/temper-client-shell-cli/src/tests/keys.rs",
  "temper/docs/design/client/tools.status.md",
]
effort = "high"
minutes = 60
+++

# A DIRECTORY or FILE that fails says which step and which error

## Goal

fab's after-landing ChatGPT story failed with `temper ctl: record fileDIRECTORY could not be
read`: `key_file_fail` glues `record file` to a reason that is DIRECTORY's, and
`directory.rs` maps a dozen stages to one line naming neither the step nor the error (an
absent `address` among them, which is a DIRECTORY never signed in). Make every DIRECTORY and
FILE failure say what failed, in the words of `temper/docs/design/client/tools.md, 8.1`.

Acceptance:

- A DIRECTORY with no `address` prints `temper ctl: DIRECTORY is not signed in`, as one with
  no `token` does; exit 1, nothing sent.
- Any other failed or unexpected answer at a DIRECTORY step prints `temper ctl: DIRECTORY
  could not be read: STEP: ERROR` (STEP `open`, `lock`, `address`, `token`), or `DIRECTORY
  could not be written: address: ERROR` at `sign-in`'s store; ERROR the file layer's
  `Error` by its `Debug` name (`NotFound`, `Permission`), `too large` for `TooLarge`,
  `unexpected answer` for any other event. No event's bytes are ever printed.
- A FILE's failed read prints `FILE could not be read: ERROR` (`sign-in`'s link, `--call`'s
  call), `text file could not be read: ERROR` (a text file), `key file could not be read:
  ERROR` (a key file); `account import` prints `record file` for `key file`, and
  `forge connect` appends `: PATH` to its key file's reasons, as today.
- `account import`, `forge connect` and the other key-file calls print a DIRECTORY failure
  whole, never with `record file` before it or `: PATH` after it.
- The tests below pass (red before the change, the import's with exactly `temper ctl: record
  fileDIRECTORY could not be read`); every test of `temper-client-shell-cli` passes; clippy is
  clean.
- `temper/docs/design/client/tools.status.md` no longer holds this plan's 8.1 entry.

## Inputs

- `temper/docs/plan/live-temper-stories/notes/common.md`.
- `temper/docs/design/client/tools.md, 8.1`: the bullets "A DIRECTORY the call cannot use" and
  "A FILE the call reads", the words this task prints; 8.2, step 2 (what opening DIRECTORY
  does).
- `temper/docs/plan/live-temper-stories/inputs/request.md`, item 1.

## Owns and may change

Owns, all in `temper/crates/temper-client-shell-cli/src/`:

- `directory.rs`: `answer_file`'s catch-all arm (~270-280), `answer_key`'s two `could not be
  read` arms (~312-337), `load_link`'s no-parent line (~555); new private helpers beside them.
- `host/input.rs`: `key_file_fail` (~151-166) alone.
- `tests.rs`: two new tests beside `a_zero_byte_token_is_not_signed_in` (~121).
- `tests/fixtures.rs`: two new `Frame` fields, their initial values in `Frame::act`, and their
  use in `open_file`'s `address` and `token` arms (~420-424).
- `tests/moves.rs`, `tests/keys.rs`: the new tests below.
- `temper/docs/design/client/tools.status.md`: remove the **Not yet** entry that begins
  `8.1: a DIRECTORY that fails a step` (six lines, after `FORGE, which
  temper-client-tools' own step tests hold alone.`), nothing else. This is the task's one
  departure from the implementer's text ("Not yours: any `docs/design/`").

Read only: `host.rs` (~723-733 calls `key_file_fail` for any directory error of a key-file
call: unchanged, since `key_file_fail` now sorts the reasons), `arguments.rs`.

Pre-approved options:

- If an existing test asserts the old catch-all words (none was found: `rg 'could not be read'
  temper/crates/temper-client-shell-cli`), change its expected line to the new words and list
  it.
- If `Ready` or `Closing` can be shown never to reach the catch-all, they may still print
  `open`; do not restructure the stage machine.
- If the fixture's `Load` cannot answer `TooLarge` for an `address` or `token` past its bound,
  drop that row of the test and say so.

## Steps

1. `directory.rs`. Sketch (bodies yours):

   ```rust
   /// What a failed or unexpected answer says (client/tools.md, 8.1): the file layer's error
   /// by name, `too large`, or `unexpected answer`; never an event's bytes.
   fn answer_words(event: &file::Event) -> String;
   ```

   In `answer_file`, before the catch-all, `(Stage::Address, file::Event::Failed { error:
   Error::NotFound, .. })` fails `DIRECTORY is not signed in`. The catch-all then fails:
   - `Link`: `FILE could not be read: WORDS`, or `text file could not be read: WORDS` when
     `self.text_root` is set;
   - `Store`: `DIRECTORY could not be written: address: WORDS`;
   - `Lock`: `DIRECTORY could not be read: lock: WORDS`; `Address`: `...: address: WORDS`;
     `Token`: `...: token: WORDS`;
   - `Start`, `Exists`, `Checked`, `Private`, `Ready`, `Closing` (and the record stages,
     which `answer` routes elsewhere): `DIRECTORY could not be read: open: WORDS`.

   `answer_key`: the failed arm's last case is `key file could not be read: {error:?}`; the
   unexpected events' arm is `key file could not be read: unexpected answer`.
   `load_link`'s no-parent line becomes `FILE could not be read: file has no parent`.

2. `host/input.rs`, `key_file_fail`: a reason that does not begin `key file` or `record file`
   is DIRECTORY's and is failed whole; the import arm puts `record file` before the rest of a
   `key file`/`record file` reason; the `forge connect` arm appends `: PATH` to the key file's
   reasons alone; the others print the reason.

3. `tests/fixtures.rs`: `pub address_error: Option<Error>` and `pub token_error:
   Option<Error>`, `None` in `Frame::act`; `open_file` returns `Err(error)` for `address` or
   `token` when set (the `token` arm still counts `token_reads`).

4. Tests (the `temper-client-shell-cli` crate's lib tests; `tests.rs`'s `fixtures::Frame`):

   ```rust
   // tests.rs
   /// client/tools.md, 8.1: `address` is written by sign-in, so a DIRECTORY without it is not signed in.
   #[test]
   fn a_directory_without_an_address_is_not_signed_in() {
       // Frame::act(CHAT), address_error = Some(Error::NotFound), run:
       // stderr == "temper ctl: DIRECTORY is not signed in\n", exit 1, requests() == 0
   }
   /// client/tools.md, 8.1: a step's failure names the step and the error.
   #[test]
   fn a_directory_step_that_fails_names_the_step_and_the_error() {
       // CHAT, one Frame a row:
       //   address_error Permission -> "temper ctl: DIRECTORY could not be read: address: Permission\n"
       //   token_error Permission   -> "...: token: Permission\n"
       //   address 65 bytes (limits::ADDRESS + 1) -> "...: address: too large\n"
       //   token 41 bytes                          -> "...: token: too large\n"
       // each exit 1, requests() == 0, stderr never holds TOKEN_BYTES
   }

   // moves.rs
   /// The live story's import named a DIRECTORY never signed in: DIRECTORY's line, alone.
   #[test]
   fn an_import_into_a_directory_that_fails_prints_the_directorys_line_alone() {
       // Frame::act(IMPORT), link = RECORD:
       //   address_error NotFound  -> "temper ctl: DIRECTORY is not signed in\n"
       //   token_error Permission  -> "temper ctl: DIRECTORY could not be read: token: Permission\n"
       // each: done(&f, 1, false), requests() == 0
   }
   /// The record file's own failure keeps `record file` and names the error.
   #[test]
   fn an_import_whose_record_cannot_be_read_names_the_record_file_and_the_error() {
       // IMPORT, linked_leaf = Some(Error::Permission):
       // "temper ctl: record file could not be read: Permission\n", done(&f, 1, false)
   }

   // keys.rs
   #[test]
   fn a_key_calls_directory_failure_prints_the_directorys_line_alone() {
       // ADD and APP, link = b"private-value\n", address_error = Some(Error::NotFound):
       // stderr == "temper ctl: DIRECTORY is not signed in\n" (no ": /linked-parent/link-file")
       // ADD, linked_leaf = Some(Error::Permission): "temper ctl: key file could not be read: Permission\n"
   }
   ```

   Run each alone first (`heavy cargo nextest run -p temper-client-shell-cli --lib -E
   'test(=tests::NAME)'`, the module path as nextest lists it), then the crate's whole lib.

5. The status entry (Owns). Then `cargo fmt`; `heavy cargo clippy -p temper-client-shell-cli
   --all-targets -- -D warnings`; `heavy cargo nextest run -p temper-client-shell-cli`; its
   direct dependents' builds: `heavy cargo check -p temper --tests`; `python3 -I -B
   tools/citations/citations.py --base origin/main` after committing.

6. Commits: the words and their tests; the status entry. Push your branch.

## Report

As the implementer's text says, and also: the words each stage now prints (a table), the
tests' red lines before the change as they printed them, and any existing test whose expected
line changed.
