# tui-whole-t1: the plan

T1 of `temper/docs/design/draft/whole-story.execution.md`: the launchpad as
drawn (`temper/docs/design/draft/surfaces.md`, 11.1 and 11.4, less the team
lines, `/talk` and `/tell`) and views printed (11.5), after the mockup
`temper/docs/design/client/tui/whole-story/launchpad.html`
(`inputs/request.md`). Released in batches; this file grows with each.

## Batch 1: views printed

Design written by this cut: `temper/docs/design/client/README.md`, 6.14,
*A chat*, *a view*; 6.15's token rows; 5.16's pointer to them; the
**Not yet** entry in `README.status.md`.

| Part | Task | Crates (files) | Needs | Minutes |
|---|---|---|---|---|
| `views` | `tui-views` | temper-client-tui (`view/printed.rs`, `view/chat*`, tests) | | 120 |

- **Printed by a module of its own,** `view/printed.rs`, with no new word,
  token or theme entry: the marks are glyph constants painted by existing
  tokens, so the launchpad's tasks own `words.rs`, `look.rs` and
  `theme.rs` alone, and this task runs beside them with no merge in common
  but a `mod` line.
- **A chat's document** is not drawn in the TUI until T2's side pane; the
  same `printed::draw` prints its views then.

## Batch 2: the launchpad as drawn

Being cut: surveys of the domain's reads (needs, chats, goals, the brief),
the commands and the opener, and the journeys run now.

## Critical path

`tui-views` alone so far.

## Overlaps

- **`temper-client-domain`** is temper/ui's too (P1 `ui-p1-rail`, P2
  `ui-p2-home`); batch 2 names the smallest change the TUI needs there.
- **`temper-client-tui`** is this plan's alone; `tui-views` and the
  launchpad's tasks share it, in disjoint files but for `view.rs`' `mod`
  line.
