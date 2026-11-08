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

## Batch 2: the launchpad's names

Design written by this cut: `temper/docs/design/client/README.md`, 6.13
(*The TUI's events*: `Own`, `Open`, `Walk`, `EndWalk`), 6.14 (*The
launchpad* as drawn and where it departs from its mockup, *Where you left
off*, *Needs you*, *Walking what needs you*, *Start*, `/brief`'s note, the
box's placeholder, *The quiet line*, *The TUI's own commands*, `/help`'s
note), 6.15's token rows, 6.17's box state on the quiet line's right, 11's
later parts; `tui/journeys.md`, 9 (`launchpad-drawn`); their status
files; the drafts' status (`surfaces.status.md`, 11 and 15;
`tui.status.md`).

| Part | Task | Crates (files) | Needs | Minutes |
|---|---|---|---|---|
| `needs` | `view-needs` | temper-client-view (`shared/needs.rs`, `shared/question.rs`) | | 75 |
| `surface` | `tui-surface` | temper-client-tui (events, state, words, rows' shape, keys, focus), shell-tui's empty arms | | 130 |

## Batch 3: the launchpad drawn, walked and opened

| Part | Task | Crates (files) | Needs | Minutes |
|---|---|---|---|---|
| `launchpad` | `tui-launchpad` | temper-client-tui (`view/launchpad.rs`, the quiet line in `view/foot.rs`, every golden it moves, the TUI's journey prints) | `tui-surface`, `view-needs`, `tui-views` | 180 |
| `walk` | `tui-walk` | temper-client-tui (`view/foot/lists.rs`, `view/foot/notes.rs`, the walk's goldens) | `tui-launchpad`, `tui-surface`, `view-needs` | 140 |
| `host` | `shell-launchpad` | temper-client-shell-tui (`host.rs`, `run.rs`) | `tui-surface` | 150 |
| `world` | `world-launchpad` | temper-client-world (`tests/suite/tui.rs`, `src/tui.rs`) | `shell-launchpad`, `tui-walk` | 90 |
| `journeys` | `journeys` | the TUI's journeys, temper-fake's cue | `tui-launchpad`, `tui-walk`, `shell-launchpad`, `tui-views` | 120 |

- **The goldens in sequence:** the quiet line and the placeholder reach
  every TUI golden, and the launchpad's body reaches the foot's, so
  `tui-launchpad` regenerates them all after `tui-views` has added its
  own, and `tui-walk` follows it. `shell-launchpad` changes no golden and
  runs beside both.
- **`tui-surface` and `tui-views` share temper-client-tui** in one wave, a
  stated departure from one writer per crate: their files are disjoint
  but for `view.rs`, where `tui-views` adds one `mod` line.
- **Journeys:** each look change reprints the TUI's journey pages
  (`temper-client-journeys` with no argument), which the focused check
  reads; `journeys` records `launchpad-chat` again on a fresh fake box and
  adds `launchpad-drawn`.

## Decisions

- **The smallest change outside the TUI** is the view's
  `shared::need_prompts` (`view-needs`), in `temper-client-view`, not
  `temper-client-domain`: the root already holds every read the launchpad
  needs (Home's facts, chats, goals, needs and approvals), and a `/change`
  chat starts through the root's existing `NewChat` path. `temper-client-domain`
  is not changed by this plan.
- **Across projects** waits for Home across projects (P2, temper/ui): the
  brief, the rows and the needs are the project's, each row naming its
  project so that nothing moves when they span several
  (`client/README.md`, 6.14, *Where it departs from its mockup*; 11).
- **The header holds the brief's sentence, `/brief` the whole:** the
  engine keeps a sentence of at most 200 bytes and a lede of at most 600
  (`domain/forge.md`, 5.7), so "the brief, framed" is the sentence and
  `/brief` its whole, with no row counting in the TUI.
- **`/chats N`,** the open question's default (`surfaces.md`, 15), numbers
  a chat's row; a goal's row has none, and Enter or `/open` opens it on
  the web: the TUI has no goal's page.
- **`n` walks on the launchpad's empty box alone;** a message that begins
  with n still types (the walk opened by `n` gives the `n` back).
- **The walk's "Next"** passes a need without touching it; the browser's
  Later is not used, so walking in the terminal moves nothing on the web.
- **Formats and interfaces changed before v0.1.0, no migration:** the
  TUI's `Event` (its recordings keep events as their `Debug` print), the
  view's `shared` surface.

## Critical path

`tui-surface` (130) → `tui-launchpad` (180) → `tui-walk` (140) →
`journeys` (120): about 9.5 implementer-hours. `tui-views` (120) and
`view-needs` (75) run beside `tui-surface`; `shell-launchpad` (150) after
it, beside `tui-launchpad`; `world-launchpad` (90) after `tui-walk` and
`shell-launchpad`, beside `journeys`.

## The launchpad's walks' layout faults

The repository's status files record none for the launchpad's layout:
the one walk fault they name, `launchpad-chat/04` lacking the person's
opening question (finding-465, `tui/journeys.status.md`), is a chat's,
repaired by `post-landing-probes`, and `journeys` drops its bullet only
where the new recording shows the question. Walk findings filed outside
the repository were not read by this cut; this plan redraws the whole
launchpad, header, sections and quiet line, so any open finding against
the old launchpad's layout is this plan's and is closed against its
landing.

## Overlaps

- **`temper-client-domain`** is temper/ui's too (P1 `ui-p1-rail`, P2
  `ui-p2-home`): this plan doesn't change it.
- **`temper-client-view`'s `shared`** is temper/ui's too: `ui-reaches`
  changes `shared/reach.rs` and names `shared/mod.rs`; `view-needs` adds
  `shared/needs.rs`, one re-export line in `shared.rs` and a per-card
  function inside `shared/question.rs`. If `ui-reaches` moves `shared.rs`
  to `shared/mod.rs` first, the re-export line moves with it.
- **The launchpad's journeys** (`temper/docs/design/client/tui/journeys/**`)
  are reprinted by each task whose look change reaches them; `ui-p1-rail`
  and `ui-reaches` regenerate the browser's journeys and
  `temper-client-shell-terminal`'s goldens, which this plan doesn't
  touch.
- **`temper-client-tui`** is this plan's alone; `tui-views` and the
  launchpad's tasks share it, in disjoint files but for `view.rs`' `mod`
  line.
