+++
id = "words-and-link"
part = "words-and-link"
kind = "feature"
needs = []
files = ["temper/crates/temper/src/directories.rs", "temper/crates/temper/src/tests/program_environment.rs", "temper/crates/temper/src/box_command.rs", "temper/crates/temper/src/tests/box_link_form.rs", "temper/crates/temper-box/src/terminal.rs", "temper/crates/temper-box/src/host.rs", "temper/crates/temper-box/src/tests/terminal.rs", "temper/crates/temper-box/src/tests/host.rs", "temper/crates/temper-box/src/tests/fixtures.rs", "temper/crates/temper-box/src/tests/sink.rs", "temper/crates/temper-box/tests/real_loop.rs", "temper/crates/temper/tests/suite/box_kinds.rs"]
effort = "high"
minutes = 50
+++

## Goal

Two things `box-small` missed, one task as the request has it. Each `TEMPER_` variable in `program_environment`'s words prints only when its own directory is off its own default, so a box with only its data moved says `TEMPER_DATA=~/x temper`. And the box's sign-in link, the first and every fresh one, is wrapped in OSC 8 when its stdout is a terminal and `TERM` isn't `dumb`, so that the terminal can make it clickable and it survives a wrapped line whole.

Acceptance:

- `Directories::program_environment` answers each of the eight combinations of the three directories moved or not, as *Steps* tabulates. A directory named by a flag or a variable at its default's own path doesn't print. An environment that gives no defaults (no `HOME`) still prints all three: `no_home_names_all_three` stays as it is. So do `the_words_survive_a_shell`, `the_widest_directories_fit_the_wire` and `a_binary_word_past_the_answer_bound_gives_way_to_temper`.
- With `LinkForm::Osc8`, a block's stdout item is exactly `ESC ] 8 ; ; URL ESC \ URL ESC ] 8 ; ; ESC \ LF`, the first block's and a fresh one's alike. Its stderr items don't change.
- With `LinkForm::Bare`, every byte stays as it is now: every existing test of the block, the stream and the real box's pipe passes with no change to what it asserts.
- A URL of exactly `LINE_BYTES` (4096) is written whole on a terminal, 8,207 bytes with its line feed. One byte more is refused in the words it is today, with none of it printed. That bound on the URL does not move.
- `LinkForm::of` is `Osc8` only with a terminal and a `TERM` other than exactly `dumb`; an unset or empty `TERM` isn't `dumb`. `temper box` takes it from its real stdout and its `TERM`.
- Remove the two **Not yet** entries this closes: `temper/docs/design/protocol/setup.status.md`, the one naming 7.2, *On a terminal*; `temper/docs/design/client/README.status.md`, the one naming 4.6, *The program's environment*.

## Inputs

- `temper/docs/design/client/README.md`, 4.6, *The program's environment* (the sentence on `TEMPER_CONFIG=…` was rewritten for this task) and *The words of a command for this machine*.
- `temper/docs/design/protocol/setup.md`, 7.2, *Its stdout*, *The sign-in block* and *On a terminal* (written for this task); 13.1, *The box's terminal and logs*.
- `skein/docs/foundation/testing-strategy.md`, section 1 and 8.1.
- `temper/docs/plan/box-small-2/inputs/request.md`.

## Owns and may change

- Owns the files in the header, and creates `temper/crates/temper/src/tests/box_link_form.rs`.
- **Departure from the implementer's text:** the two status entries named in *Goal* are yours to remove, and nothing else under `docs/design/`, because this cut wrote them for this task alone. Then run `python3 -I -B tools/citations/citations.py --base origin/main` before you push.
- `temper/crates/temper/src/tests.rs`: the one line `mod box_link_form;`.
- Readers of what changes, call sites only:
  - `host::Settings` gains a field. Its literals are in `temper/crates/temper/src/box_command.rs`, `temper/crates/temper-box/src/tests/fixtures.rs`, `temper/crates/temper-box/src/tests/host.rs` (the one at its `FirstLink` Debug test), `temper/crates/temper-box/tests/real_loop.rs` (three) and `temper/crates/temper/tests/suite/box_kinds.rs`. All but `box_command.rs` take `LinkForm::Bare`.
  - `Formatter::new` takes an argument. Its callers are `host.rs` and, in tests, `temper/crates/temper-box/src/tests/terminal.rs` and `tests/sink.rs`, which take `LinkForm::Bare`.
  - `fixtures::Opening` gains the field `link_form`. Its literals are in `fixtures.rs` and `tests/host.rs` (three), which take `LinkForm::Bare` but where your new tests say otherwise.
- Leave as they are, since they carry the answer as opaque bytes and three moved directories remain a legal answer: `temper-service/src/tests/family.rs`, `temper-client-protocol-wire/src/tests.rs`, and the `TEMPER_CONFIG=…` fixtures of `temper-client-view/src/tests/{connect_model,welcome}.rs` and `temper/tests/client/tests/suite/connect_model.rs`. Other drafts are working in `temper-client-view`: don't touch it.
- Leave `program_login::line` and its three flags: the request names `program_environment` alone. Leave `Output::Legacy`, which is a raw piece and no link line; `temper serve` run alone, which isn't the box; and `docs/help/`, whose "The link alone is on stdout" stays true.
- If the literal `std::io::stdout()` is refused by a lint in `box_command.rs`, take the descriptor from `settings.stdout`'s value through a `BorrowedFd` with a scoped `#[expect]` and say so. If `/dev/ptmx` can't be opened on your builder, leave `box_link_form.rs`'s terminal rows out, keep the rest and say so in the report; don't make the test skip by itself.

## Steps

Failing tests first; see each fail for its rule before the code goes in.

### 1. Each variable off its own default

In `temper/crates/temper/src/directories.rs`, `program_environment` compares each directory with its own default. `resolve` refused means no default is known, and all three print. Inside that the shape is yours; `same` goes, and the word is added when any words precede it or it isn't `temper`.

In `temper/crates/temper/src/tests/program_environment.rs`, replace `one_moved_directory_names_all_three` with `each_directory_is_named_only_off_its_own_default`, the moved ones given as `TEMPER_` variables with the value `/moved`, the word `temper`, on `home()`:

| Moved | Answer |
|---|---|
| none | (empty) |
| config | `TEMPER_CONFIG=/moved temper` |
| data | `TEMPER_DATA=/moved temper` |
| cache | `TEMPER_CACHE=/moved temper` |
| config, data | `TEMPER_CONFIG=/moved TEMPER_DATA=/moved temper` |
| config, cache | `TEMPER_CONFIG=/moved TEMPER_CACHE=/moved temper` |
| data, cache | `TEMPER_DATA=/moved TEMPER_CACHE=/moved temper` |
| all three | `TEMPER_CONFIG=/moved TEMPER_DATA=/moved TEMPER_CACHE=/moved temper` |

And `a_directory_off_its_default_alone_is_named`, whose rows are:

- `TEMPER_DATA=/home/person/x`, word `temper`: `TEMPER_DATA=~/x temper`
- the same, word `~/bin/temper`: `TEMPER_DATA=~/x ~/bin/temper`
- the flags `--data /moved`, no variable: `TEMPER_DATA=/moved temper`
- `TEMPER_DATA=/home/person/.local/share/temper`, its default's own path, with `TEMPER_CACHE=/moved`: `TEMPER_CACHE=/moved temper`
- `XDG_DATA_HOME=/xdg`, no `TEMPER_DATA`: empty, `/xdg/temper` being its default

`moved_directories_and_the_binary_share_short_words` stays: all three are off their defaults there.

### 2. The link's form

In `temper/crates/temper-box/src/terminal.rs`, beside the block's words:

```rust
/// How a sign-in link reaches the box's stdout (protocol/setup.md, 7.2).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LinkForm {
    /// The URL alone on its line: a pipe, a file or `TERM=dumb`.
    Bare,
    /// The URL in OSC 8, then its line feed: a terminal.
    Osc8,
}
impl LinkForm {
    /// `terminal`: the box's stdout is one. `term`: `TERM`'s value, if it has one.
    #[must_use]
    pub fn of(terminal: bool, term: Option<&[u8]>) -> Self;
    /// One link's line, ending in its line feed as `Formatter::block` is given it.
    /// A line with no URL stays as it is.
    #[must_use]
    pub fn line(self, line: Box<[u8]>) -> Box<[u8]>;
}
/// A link's widest line on a terminal: the URL twice, 14 marker bytes and the line feed.
pub const LINK_LINE_BYTES: u32 = /* 2 * LINE_BYTES + 15, checked as `limits::RECORD_BYTES` is */;
```

`Formatter::new(form: LinkForm)` keeps it, and `block` writes its stdout item through `self.form.line(url)`. Both of the host's calls of `block` then have it, the first link's and serve's.

In `temper/crates/temper-box/src/host.rs`, `Settings` gains `pub link_form: LinkForm`, documented as `first_link` is, and `Service::new` hands it to the formatter. `.max(LINE_BYTES.checked_add(1).expect("URL LF"))` becomes `.max(terminal::LINK_LINE_BYTES)`.

`terminal::failure_bytes` already holds `bounds.output` far above 8,207 bytes, so removing that `.max` fails no test, and the cap isn't preallocated. Keep the `.max`, since it says what the line needs. Bite `LINK_LINE_BYTES` and the widest story by changing the formula, and don't hunt for a way to bite the `.max`: the report says so.

Tests in `temper/crates/temper-box/src/tests/terminal.rs`:

- `a_link_takes_osc_8_on_a_terminal_unless_term_is_dumb`, `LinkForm::of`'s table. On a terminal: `None`, `xterm-256color`, `eterm-color`, empty and `dumber` give `Osc8`; `dumb` gives `Bare`. Off a terminal: `None`, `xterm-256color` and `dumb` give `Bare`.
- `the_first_block_on_a_terminal_wraps_its_link` and `a_fresh_block_on_a_terminal_wraps_its_link`, the exact items, the stdout one `b"\x1b]8;;http://link\x1b\\http://link\x1b]8;;\x1b\\\n"`.
- `the_widest_link_fills_its_line`: a URL of `LINE_BYTES` bytes gives a line of exactly `LINK_LINE_BYTES`, ending in its line feed. A `\n` alone stays `\n`, in both forms.

Tests in `temper/crates/temper-box/src/tests/host.rs`, on the harness, `Opening` taking the form:

- `each_link_reads_on_stdout_as_its_form_has_it`, for each of `Bare` and `Osc8`: the first link and then a fresh one from serve, written to its pipe in two pieces, are on `harness.stdout()` as those exact bytes and nothing else, and `terminal_bytes()` holds the block's words in their order around them.
- `the_widest_link_is_whole_on_a_terminal_and_one_byte_more_is_refused`: `line_cap()` and a line feed from serve, under `Osc8`, reaches `harness.stdout()` as 8,207 bytes. `line_cap()`, then `x\n`, prints none of its bytes and says `box: serve: sign-in link too long`, and the next line is read normally, as the existing test asserting `box: serve: sign-in link too long\nOpen this link` has it.

### 3. The binary's choice

In `temper/crates/temper/src/box_command.rs`:

```rust
/// How the box's stdout takes a sign-in link (temper/docs/design/protocol/setup.md, 7.2).
pub(crate) fn link_form(stdout: &impl std::io::IsTerminal, environment: &[Box<[u8]>]) -> temper_box::terminal::LinkForm;
```

It finds `TERM=` among the `NAME=VALUE` entries `run_configuration` already collected, and `run_configuration` passes `&std::io::stdout()`. `std::env::var` is disallowed here, so don't read `TERM` by that.

`temper/crates/temper/src/tests/box_link_form.rs`, `the_box_reads_its_stdout_and_term`, on real descriptors in the unit tier. No process, no network:

| `stdout` | Environment | Form |
|---|---|---|
| `/dev/ptmx`, opened read-write | `TERM=xterm-256color` | `Osc8` |
| the same | `HOME=/h`, `TERMCAP=dumb`, no `TERM` | `Osc8` |
| the same | `TERM=dumb` | `Bare` |
| a scratch file | `TERM=xterm-256color` | `Bare` |
| `/dev/null` | `TERM=xterm-256color` | `Bare` |

No test runs a real `temper box` on a real terminal: there is no pty harness, and the user tries it. So `&std::io::stdout()` is the one line no test holds. The bare form on a real pipe stays held by `the_first_link_opened_at_once_waits_and_signs_in` (`temper/crates/temper/tests/suite/box_listeners.rs`), which reads the real box's link through `box_link.rs`.

### 4. Narrow tests, then the checks

A value `program_environment` returns changes meaning. Its readers carry it opaque and draw it after no check of their own, so no reader's tests change; run them as the list below has it.

```
heavy cargo nextest run -p temper-box --lib
heavy cargo nextest run -p temper-box --test real_loop
heavy cargo nextest run -p temper --lib -E 'test(program_environment) | test(box_link_form) | test(box_kinds)'
heavy cargo nextest run -p temper --test suite -E 'test(box_listeners) | test(box_kinds)'
heavy cargo nextest run -p temper-client-view --lib -E 'test(connect_model)'
```

Then `cargo fmt`, the old-forms count and clippy as the implementer's text has them, naming `temper` and `temper-box`. Commit small, saying why, and push.

## Report

As the implementer's text asks. Name each rule broken and what failed, the `.max` above being the one line that can't bite, and whether `/dev/ptmx` opened on your builder.
