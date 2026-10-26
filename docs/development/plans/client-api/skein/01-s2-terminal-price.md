# Supplement skein-01.s2: the terminal's worst case, operations and line bytes

Added by ruling 143, from the design answer of 2026-10-16T15:20Z (skein
`8907ea3a`, `docs/design/ui-text.md`, 1, 3, 4 and 8). temper 03.1 needs
its landing; jig 19.4 moves jig's skein onto it. Read first: skein's
`AGENTS.md`, `docs/development/workflow.md`, `ui-text.md`, 3 and 4; this
plan's README, "Rules of this plan".

Scope: `ui-text.md`, 3: "*its line bytes*: the most bytes of each of the
application's own lines, without the name, `: ` and the newline the
terminal adds (section 4); 0 for an application that prints none", and
"`Terminal::worst_case`, the most heap the terminal holds, and
`Terminal::operations`, the most operations it has in flight at once ...
Each is none for settings or limits the terminal refuses, and none past
its integer"; 4: "The host says a line only within that room, and of at
most the settings' line bytes. A line said beyond the room, or longer, is
the host's defect".
Needs: nothing (skein's `main`, which holds skein-01.s1's `a11ed29` and
forgejo-17's skein step's `267730eb`).
Files: `crates/skein-ui-text/src/terminal.rs`: `Settings.line_bytes:
u32` (`Settings` at `:16`); one private builder of `new`'s checks and io
limits (`:72-91`, `new` and the limits it builds), used by `new` and by
the associated functions (not free ones: `skein_ui_text::worst_case` is
render's, `lib.rs:8`) `pub fn worst_case(settings: &Settings, view: &skein_ui::Limits) ->
Option<u64>` and `pub fn operations(settings: &Settings, view:
&skein_ui::Limits) -> Option<u32>` (`skein_io::operations` of those
limits, `crates/skein-io/src/limits.rs:91`). `worst_case` sums, checked:
`skein_io::worst_case` of them (`limits.rs:115`); `Queue::worst_case` of
completions 32, submissions 256, raw 256, events 32 (`:103-106`),
commands `BATCH` (`:107`), lines `lines + BATCH + 1`; `lines × (name + 3
+ line_bytes)`; `(BATCH + 1) × (name + 40)` (the longest refusal, `: no
[`, 20 digits, `] on the page\n`); `BATCH × longest` (commands' words);
`BATCH` (a read); `List::<u8>::worst_case(longest + 1)`; a line's
unwritten rest, the longest line said; `List::<NodeId>::worst_case(view.nodes)`;
`render::worst_case(view)?.1` (`render.rs:25`). `say` asserts
`line.len() <= line_bytes`, with the message "application line within
its bytes". `tests/ui-text/tests/terminal.rs`:
`settings()` (`:17-18`) gains `line_bytes: 64`; no other
`skein_ui_text::Settings` literal is in skein (`git grep -n
'Terminal::new' -- '*.rs'` prints `:30` and `:183`, both through
`settings()`, at skein `8907ea3`). Downstream, jig's one literal
(`tests/client-protocol/src/native/world.rs:232`) is jig 19.4's repair,
and temper has none until 03.1. jig's `tests/client-protocol/src/native.rs:443`
writes the terminal's operations as `10`, a copy jig 19.4 may take from
`Terminal::operations`.
Test shape, in `tests/ui-text/tests/terminal.rs`:
- `the_worst_case_counts_each_application_line_at_its_bytes`: at
  `fixtures::LIMITS`, `lines: 2`, `line_bytes` 65 gives 3 more than 64, two lines and a line's rest;
  `operations` is `skein_io::operations` of 2 sockets, 10; both `None`
  for a 129-byte name.
- `a_line_past_its_bytes_stops_the_terminal`: `#[should_panic(expected =
  "application line within its bytes")]` (a bare `#[should_panic]` fails
  clippy's `should_panic_without_expect`), a 65-byte line at 64.
Proof: skein's gate.
Don't: change `BATCH`, a refusal, `say`'s prefix or `Terminal::new`'s
signature; give `line_bytes` a default; name a downstream repository
(`git -C <worktree> diff origin/main...HEAD | grep -i -w -E
'^\+.*(jig|temper|smith)'` prints nothing).
Size: at most 80 lines.
Done when: `git -C ~/src/rust/skein log origin/main --all-match
--grep='^Plan: client-api$' --grep='^Step: skein-01.s2$' --format=%H -1`
prints its landing.
