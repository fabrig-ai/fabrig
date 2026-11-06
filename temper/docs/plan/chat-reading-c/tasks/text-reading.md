+++
id = "text-reading"
part = "text-shell"
kind = "feature"
needs = ["log-surface"]
files = [
  "skein/crates/skein-ui-text/src/command.rs",
  "skein/crates/skein-ui-text/src/render.rs",
  "skein/crates/skein-ui-text/src/tests/**",
  "skein/docs/design/ui-text.status.md",
]
effort = "medium"
minutes = 30
+++

# In the terminal, a log's edges are lines a person types

## Goal

The terminal has no keys and no view (`skein/docs/design/ui-text.md`, 6 and 7), so a bound log's `Near` comes from a line: `up`, `down`, and what PgUp and PgDn make, with Enter. temper's terminal reads earlier and later lines with them.

Acceptance:

- `up` and the line `ESC [ 5 ~` give `Command::Act { node, first: Act::Near { edge: LogEdge::Head }, submit: false }` on the first bound log in preorder; `down` and `ESC [ 6 ~`, `LogEdge::Foot`. Spaces are trimmed as for any line.
- With no bound log on the page, those lines are refused as anything else is: `NAME: commands: n, n words, p, q, up, down`, and that refusal's words are the same everywhere.
- A bound log prints `(up, down: more lines)` first under its name, at its children's indent. An unbound log prints as today. A log is never numbered.
- The line in `skein/docs/design/ui-text.status.md` is gone, and it says `No gap.` again.

## Inputs

- `skein/docs/design/ui-text.md`, 5 and 6, *A log's edges are lines too*.
- `skein/docs/design/ui.md`, section 6, *A log's edge near*.
- `temper/docs/plan/chat-reading-c/notes/contracts.md`, *The text shell*, binding.
- `temper/docs/plan/chat-reading-c/notes/common.md`.

## Owns and may change

Owns the files in the header.

Readers and neighbours: a test in any tree that pins the refusal's words `commands: n, n words, p, q`, for its expected value alone. `rg` finds them.

Options, each decided:

- `command`'s signature takes the tree already: the first bound log is found there.
- The printed line is the shell's words, as `(completed)` is. No phrasebook.

Not this task: any temper page. No log in temper is bound until `chat-page`, so temper's prints don't move here. If one does, that's a red.

## Steps

1. Red first, in `skein-ui-text/src/tests`: `up_and_down_near_the_first_bound_log`, `the_lines_pgup_and_pgdn_make_are_up_and_down`, `no_bound_log_refuses_them`, `a_bound_log_says_how_to_read_on`, `an_unbound_log_prints_as_it_did`.
2. `command.rs`, `render.rs`.
3. `cargo fmt`; `python3 -I -B skein/scripts/old_forms.py`; `heavy cargo clippy -p skein-ui-text --all-targets -- -D warnings`; `heavy cargo nextest run -p skein-ui-text`; `heavy cargo nextest run -p temper-client-shell-terminal`. Commit and push.

## Report

As the implementer's text asks.
