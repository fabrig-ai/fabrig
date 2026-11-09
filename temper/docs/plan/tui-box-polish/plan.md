# tui-box-polish: the plan

The user's five requests of 2026-11-09 (`inputs/request.md`), after a
night of `temper box` with the TUI in front. The design this cut wrote:
`temper/docs/design/client/README.md`, 6.13 to 6.17 (*Typed before the
sign-in lands*, *The notice*, *The notice's time*, *`/quit`*, *`/theme`*,
*The theme chosen*, and the box's ink in 6.15), skein's
`skein/docs/design/ui-screen.md`, sections 3, 5.4 and 9 (a token's
focused value's paint), and `temper/docs/design/client/tui/journeys.md`,
3.2 (`screen`'s `none`). The departures from the drafts are in
`temper/docs/design/draft/tui.status.md` and `surfaces.status.md`, as
the user's requests of 2026-11-09. The shapes that cross tasks are
`notes/contracts.md`; the standing rules `notes/common.md`.

## Parts and tasks

| Part | Task | Needs | What |
|---|---|---|---|
| box-ink | `focused-value` | | skein: `StyleLook::focused_value`, a focused field's value and caret in its token's paint, the mark alone in the focus's |
| notice | `notice` | | `Notice`, `Told::Notice`, the quiet line's notice, the host's 6-second due; the unknown command's note becomes the notice |
| quit | `quit` | | `/quit` names `Command::Exit` in the TUI, `/exit` whole still closes; help page |
| theme | `theme-start` | | `ThemeChoice`, the `theme` record in the small store, `Start.scheme` the terminal's word or none, `keep_theme` and the store's answers kept from the root |
| typing | `typing` | `notice` | keys go to the box whenever it is drawn, the sign-in never clears it, the words carried to Home, Enter before it lands notices "Still signing in" |
| theme | `theme-list` | `theme-start`, `notice`, `quit` | `/theme` the TUI's own, its list (Auto with what it reads, Light, Dark), the choice taken, kept and noticed |
| box-ink | `box-ink` | `focused-value`, `quit`, `notice`, `typing`, `theme-list` | `tui-field` names a focused value of none; every golden's box line and the journeys' frames follow |

## The critical path

`theme-start` (90 min, at once, beside `notice` and `quit`) →
`theme-list` (90) → `box-ink` (50): about 230 implementer-minutes.
`typing` (70) runs after `notice` beside `theme-start` and
`theme-list`. `focused-value` runs at once and waits for no one.

`box-ink` waits for every task that adds or changes a golden or a
journey's frame, though it builds on `focused-value`'s code alone: the
TUI's goldens are kept by hand and the journeys' frames are printed, so
a golden made by a task merged after `box-ink` would keep the old paint
and turn the trunk red. Running last, it repaints them all once.

## Decided

- **The box's ink is skein's mechanism first:** skein's renderer laid
  the focus's paint over a focused field's mark, value and caret alike,
  so the TUI could not paint the mark alone. `StyleLook::focused_value`
  is generic (skein's `ui-screen.md`, section 3), as `active_name` was;
  the TUI names `Some(Paint::NONE)` on `tui-field`. The sign-in's link
  field keeps the focus's paint.
- **The notice replaces its right half:** while it stands, the box's
  state, "playground", "paused", the count and "n to walk" give way to
  it, then stand again ("a notice shows over the state for its time").
  It is painted `tui-signal`, as the box's state it generalizes.
- **"In a narrow window the notice goes before the quiet line's own
  words"** reads two ways (dropped first, or drawn first). The design
  draws it first, at the row's start, the line's own words after it, cut
  from their end: a notice that is the person's only answer to Enter is
  never dropped. Recorded in `draft/tui.status.md`.
- **Not notices:** `/open`'s "Open ADDRESS in your browser." stays a
  note (it holds an address to copy, which 6 seconds would take away),
  as do `/brief`'s, `/help`'s and `/log`'s and the give up's.
- **Before the sign-in lands** is "the link signed out, or `me` not yet
  read" (`landing`): Enter there keeps the words and notices "Still
  signing in"; no command list opens and no command runs, so nothing is
  sent or run before the person's next Enter. The words typed on the
  no-project box before `me` are carried into Home's composer when the
  host goes to the launchpad.
- **The theme is kept in the small store,** as the request asks and as
  the browser keeps its look: the record `theme` (`auto`, `light`,
  `dark`), taken out of `Start.kept` before the root, whose store reads
  only jig's records. The native shell answers each `Save` and `Erase`
  once and in order, so the host notes which writes were its own and
  gives the root only the answers to the root's. The record is a stored
  format, new before v0.1.0: no migration.
- **`Start.scheme` is the terminal's word or none,** where it was always
  a scheme: Auto must say "your terminal doesn't say" (emacs vterm never
  answers OSC 11). The recording's `screen` gains `none`; the committed
  journeys keep `dark`.
- **`/theme` becomes the TUI's own command** (`Own::Theme`), in the
  place the root's held; the browser's terminal's `/theme` is the
  root's and flips as before. Between `theme-start` and `theme-list`,
  the root's flip is kept through `keep_theme`, so the trunk is whole at
  each merge.
- **`/quit` is the TUI's name** for the root's `Command::Exit`
  (`command_name`); `temper-client-view`'s `shared` words, which the
  browser's terminal says, are unchanged: `/exit` stays the browser's.
- **Public interfaces changed before v0.1.0,** no compatibility:
  `host::Start.scheme`, `Told::Unknown` (now `Told::Notice`),
  `Tui::unknown` (now `Tui::notice`), skein's `StyleLook` (a field).

## Overlaps

- Every task but `focused-value` edits `temper-client-tui` or
  `temper-client-shell-tui`, so the TUI's crates have several writers in
  one wave. Each spec names where it adds (fields after a named field,
  host tests after a named test, status entries in separate places,
  lib.rs lines apart), so that merges stay apart. Unavoidable: `notice`
  and `quit` both edit `tests/foot.rs`'s `note_states` (different
  lines), and `notice`, `quit`, `theme-start` and `typing` all edit
  `host.rs` (different functions).
- Siblings: none of `merge-job-runs-2`, `ui-cli-c1` or `ui-ask-m2` touches
  the TUI's crates; `ui-cli-c1` edits `temper-client-view`'s tests,
  which this plan leaves alone.
- The request names temper/migration's F1 goal changing `host.rs`'s
  local sign-in on "not ready", and finding-469's repair: `typing` edits
  `typed`, `send` and `launchpad`, not the local sign-in's `heard`,
  `found` or `need_local`; a catch-up may still meet them.

## Found

Room for what the tasks find.
