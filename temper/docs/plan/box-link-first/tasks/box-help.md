+++
id = "box-help"
part = "box-help"
kind = "docs"
needs = []
files = ["temper/docs/help/box.md", "temper/docs/help/tier.md", "temper/docs/help/party.md", "temper/docs/help/reference/commands.md", "temper/crates/temper/src/commands.rs", "temper/crates/temper/src/tests/help.rs", "temper/crates/temper-docs/src/tests/rules.rs"]
effort = "medium"
minutes = 30
+++

# `temper help box` says what the box now prints

## Goal

The help a person reads, and the tests that read it, say the new terminal. **Departure from your text:** `docs/help/` is yours here, and so is the `box` row's text in `commands.rs`. Help changes before the code on this trunk: it goes with the design, and no test of the box reads it.

Acceptance:

- `box.md`, about lines 115-118: `Expect:` becomes the block, its words exact as `temper/docs/design/protocol/setup.md`, 7.2 has them, the link alone on stdout. Name `DATA/box/logs/box.log` where it names `Logs:`, and the fake's `box:` lines as following the block.
- `box.md`: add, once, near the start's `If not`: a link opened once the box has given up gets no answer, and the terminal has said why; mend what it said and run `temper box` again, for a new link. A port taken refuses before any link is printed.
- `box.md`, about line 147, and `party.md`, about lines 54-56: a fresh link is printed alone on the terminal: a blank line and the link, with no `A fresh sign-in link:`. `party.md`'s "`temper serve` prints ... and `temper box` passes it on" says the box prints the first link, and passes on each fresh one.
- `box.md`, about lines 167-171: `temper box`'s terminal (stderr) becomes `DATA/box/logs/box.log`, the same two lines. `temper-docs/src/tests/rules.rs`, about line 430, reads the new words.
- `box.md`, about line 34, and `tier.md`, about line 139: `wait for it to exit`, for `wait for `Stopped``. `temper/crates/temper/src/tests/help.rs`, about line 646, reads the new words.
- `box.md`, about line 296: the last line of `DATA/box/logs/box.log` ends `Stopped`.
- `commands.rs`, about lines 112 and 124, the `box` row. `--echo`: `none prints errors and a failed step's log; steps prints every step's lines too, prefixed by its name.` Its notes: `Its stdout is its sign-in links, each line whole; its stderr, the block's other lines and errors, and what --echo chooses; the box and each step log under the data directory.`
- `box_command.rs`, the one line `ECHO_SETTINGS`' `about`: `Show errors and a failed step's log, or every step's lines too.`
- `docs/help/reference/commands.md` is regenerated: run the test `the_page_commands_is_what_the_table_renders` with `SKEIN_CODEGEN_WRITE=1` set. Never edit it by hand.
- `heavy cargo nextest run -p temper --lib -E 'test(help) or test(commands)'` and `heavy cargo nextest run -p temper-docs` pass.

## Inputs

- `temper/docs/design/protocol/setup.md`, 7.2: *Its terminal*, *Its own log*, *The sign-in block*.
- `temper/docs/design/draft/box-link-first.md`, sections 2 and 5.
- `temper/docs/plan/box-link-first/notes/common.md`.

## Owns and may change

Owns the files in the header. The one line in `temper/crates/temper/src/box_command.rs`, `ECHO_SETTINGS`' `about`, and nothing else of it: `box-draws-link` owns the rest of that file.

Leave as they are: `github.md`, about line 18, and `forgejo.md`, about line 29 (the `box:` lines stand); `ctl.md`, about line 30; `temper/README.md`, about line 29.

Options, each pre-approved:

- A page past its byte bound may be condensed, losing no fact.
- If `rg` finds a further sentence in `docs/help/` naming `Logs:`, `Starting`, `Stopped`, `Ctrl-C stops temper.` or `A fresh sign-in link:` as the box's terminal, change it the same way and name it in the report. `temper init`'s `Expect: init: wrote ...` is not the box's terminal, and stays.

## Steps

Edit, regenerate, run the tests above. Then fmt, clippy on `temper` and `temper-docs`. Commit and push.

## Report

As your text says, with each further sentence you changed.
