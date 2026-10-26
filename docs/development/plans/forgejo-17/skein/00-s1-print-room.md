# skein-00.s1: a print's room from its own tree

A supplement step of forgejo-17 (ruling 104), from the designer's answer
of 2026-10-16T14:46:00Z (skein `054c06e1`, temper `551b1d2d`). In skein.
Needs nothing; temper-01.s1 waits on it.

Read first: skein's `docs/design/ui-text.md`, section 3 ("A print takes
its room from its own tree"), 4 ("One print at a time": the output in
pieces) and 8 ("Its bound").

Files: `crates/skein-ui-text/src/render.rs` (`Print::new` `:15-22`,
`render` `:94`); `crates/skein-ui-text/src/terminal.rs` (`Terminal::new`
`:71`, its output room `:77`, `write` `:309`); their tests, and
`tests/ui-text/tests/terminal.rs`; the users
`examples/ui/view/src/tests/text.rs` and `tests/ui-text/`, kept passing.

Shape:
- `Print::new` reserves no text: it holds the print's numbers at the
  limits' nodes, as now, and refuses no limits for their worst case.
  `render` reserves the print's bytes as it makes it, by `worst_case`'s
  formula over that tree alone: its nodes, its deepest nesting, its text
  with the shell's name. A tree whose own room passes a u32 prints, in
  place of the page, the line `NAME: page too long to print`, and
  numbers no node.
- `Terminal::new`'s output room is 64 KiB, or the longest line with a
  node's text and 64 bytes when that is more, not `worst_case`'s bytes. A
  print goes to the output in pieces of at most that room; its rest waits
  as the print and an offset.
- The API stays: `Print.text: List<u8>`, `worst_case`'s `(u32, u64)`,
  both constructors' signatures.

Test shape (ui-text.md, 8, "Its bound"): a print and a terminal made
from limits whose `worst_case` passes a u32, printing a small tree; a
tree whose own room passes a u32, printed as `NAME: page too long to
print`, with no number acting. The existing bound tests stay.

Proof: skein's gate, `scripts/check.sh`, which runs at the merge, once. On x1
its browser tier needs `export
SKEIN_TEST_CHROMIUM=$HOME/.local/opt/chromium-147.0.7727.101/bin/chromium`
in the shell that merges; a login shell gets it from `~/.profile`.
A failure outside `skein-ui-text` and its users: block, with the log.

Size: at most 80 production lines, by code-organization.md, section 6's
command in skein's worktree.

Commit trailers: `Plan: forgejo-17`, `Step: skein-00.s1`. client-api's
skein floor greps `^Step: skein-` under `^Plan: forgejo-17$` (its README,
"The print's room").

Don't: name a downstream repository; change skein-ui's limits or
`worst_case`'s formula; change a public signature of `Print` or
`Terminal`.
