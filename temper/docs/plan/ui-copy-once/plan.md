# ui-copy-once: the welcome's Copy reads once

The request: `inputs/request.md`. The design already says this (`skein/docs/design/ui-web.md`, 6: a button shows its name as its text;
`temper/docs/design/client/README.md`, 3.5: *The command's Copy*), so this plan writes none. The code comes to it.

## Parts and tasks

| Part | Task | Needs | What it makes run |
|---|---|---|---|
| `copy-once` | `tasks/copy-once.md` | | The fix in `block()`, a test of the three labels, a walk of every gallery state, one throwaway render |

Critical path: `copy-once`, alone.

## What this plan decided

- **The walk is the request's, with one exception.** The request asks that no `Button`, `Tab` or `Option` has both a name and a text child
  in a browser. A recommended choice has both by design: its name, then a `.rec` tag saying "recommended" (`client/README.md`, 3.5, *Needs
  you*; `pages/ask.rs`, `pages/goal.rs`, `pages/needs.rs`; `tests/needs.rs` holds it). Those are different words, not the name drawn twice,
  and the gallery has samples that draw it, so the literal assertion would be red on a correct page. The walk excepts "recommended" alone. The
  task checks the exception bites.
- **The gallery walks every state simply,** so the walk is in: `samples::samples()` and `view()`, as `no_state_reads_of_zero` has them, in
  the browser shell. Nothing new is built.
- **No other site is the same bug, by this cut's reading.** The request counted 14 sites opening a role held in a variable; this cut counts 22, and
  4 `match` or `if` forms. None draws a name and a text child in a `Button`, `Tab` or `Option`. The list is in the task. It was read, not run: a
  cut compiles nothing. The walk is what checks it, and the task has the option ready if it finds one. Home's Skip is as the request says.
- **The label still changes with no text child.** A name patch rewrites a button's own text (`skein-ui-web/src/page/web.rs`, `refresh` and
  `content`), as Away's toggle already relies on. This too was read, not run: the press isn't run in a real browser in this plan, and
  `client/README.status.md` already records that no browser test presses the Copy. The status stays as it is.

## Overlaps

- `temper/tools/gallery/src/tests.rs`, with `long-words/composer-view` (`temper/tools/gallery/**`), not yet merged. The walk is one function
  added mid-file, after `no_state_reads_of_zero`, to keep the hunks apart. No other draft beside this one names the two `welcome_copy.rs` files.
- **A later draft can trip the walk:** a page that adds a named control with a text child goes red once it meets this test. That is the
  walk's purpose, and that draft's to repair.
