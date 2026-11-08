# tui-link-colours: the plan

**Interfaces that change.** skein: `skein-ui-screen-terminal` gains `Background`,
`ASK_BACKGROUND`, `Settings.background`, `Terminal::background`, `Decoder::answers` and
`Decoder::background` (`skein/docs/design/ui-screen.md`, 6.1 and 8); no skein crate of
`fail-stop`, `heap`, `io`, `json`, `lib`, `scratch`, `shell`, `sim` or `world` changes.
jig: `jig_client::LinkView` gains `expires` (`jig/docs/design/client/domain/client.md`,
8.2). temper: the TUI's host seam (`Start`, `Report`, `Duties`), its run's settings, its
journey's records (optional, version `2` kept), its directory's new `sign-in` note, the
engine's new link file, and the box's block's last line. No compatibility is kept: none
is owed before v0.1.0.

Two changes the user asked for on 2026-11-07 (`inputs/request.md`): a TUI on the same
machine as a box signs in with nothing to paste, from the engine's live link kept at rest;
and the TUI draws on the terminal's own colours. The design is written by this cut, in
place: `temper/docs/design/client/README.md`, 6.11 to 6.16 and 7.1;
`temper/docs/design/client/tui/journeys.md`, 3.2, 4.1, 5, 6.4 and 8.2;
`skein/docs/design/ui-screen.md`, 3, 4, 6.1, 7, 8 and 9; `jig/docs/design/client/domain/client.md`,
8.2; `temper/docs/design/protocol/setup.md`, 7.2. The draft's departures are in
`temper/docs/design/draft/tui.status.md` (4.1; 4.4 and 5). Contracts:
`notes/contracts.md`; standing rules: `notes/common.md`.

## Decisions

- **jig's printed link is left as it is.** After a sign-in spends the live link, jig draws
  the next when a spent or expired one is tried and none is live. So the browser's way in
  after the TUI's link is today's: the spent link opened says "This link was already
  used. A fresh one is in your terminal.", and the engine prints and keeps the next. The
  TUI, answered `link_used` or `link_expired` by the file's link, reads the file again a
  second later, at most twice, since its own try drew the next: that is the common way
  (the browser signed in first). Drawing a link at every spend and expiry instead would
  print one every two hours on the box's terminal and keep an owner link live always.
- **Expired is told from signed out by the TUI's note,** `sign-in` in its directory: the
  expiry the sign-in's answer gave (jig's new `LinkView.expires`), compared with the
  wall clock of the engine's own machine, a minute's slack; signed out before it is
  noted `signed-out`, and only a paste or an approval signs in there again. The engine
  answers both alike, so no wire changes.
- **The engine's file** is serve's, written at each `Link` and at a boxed start with the
  box's first secret, removed at exit; Linux `$XDG_RUNTIME_DIR/temper/printed-link-KEY`
  (none without `XDG_RUNTIME_DIR`), macOS `DATA/printed-link`; KEY from the data
  directory's SHA-256. Bare `temper` takes that file's address first.
- **Colours:** the base is the terminal's defaults (a paint with no colour, which skein
  already draws as SGR 39/49 through its reset); `sel` leaves the palette; the focus is
  the accent beside its mark. Dark or light: `COLORFGBG`, else OSC 11 waited 200 ms, else
  dark. The query is skein's (its `ui-screen.md`, 10 had it as Later); luma's threshold
  is temper's (`Scheme::of_background`).
- **The journey stays replayable** throughout: the new records are optional and the
  version stays `2`, so `sign-in-approved` replays until the last task records it again.
  `again` gives the TUI a runtime directory of its own, so the journey still walks the
  approval.

## Overlaps

- `temper box`'s printed block is the box owner's: one line, `In a terminal: temper
  http://HOST:PORT`, at the first block's end (`box-side`; `setup.md`, 7.2;
  `temper/docs/help/box.md`'s expected block).
- `temper-client-journeys` is shared with the web client's owner: this plan changes only
  its TUI parts (`tui_again.rs`, the TUI print's reprint) and none of the browser's
  journeys, redaction or scan. The file's secret needs no new original kind: the TUI's
  *link* original gains the `found` record as a source, in the TUI's own redaction.
- `temper/tests/client` (`joint`) adds to `src/tui.rs`, `src/component.rs` and
  `tests/suite/tui.rs`; `ui-account-again`'s story changes `account_tools.rs` and
  `front/account_acts.rs` there, which this plan leaves alone.
- Test spawns of `temper` that don't clear their environment inherit the runner's
  `XDG_RUNTIME_DIR`: their serve writes its link file there and removes it at exit; one
  killed outright leaves it. `box-side` points the box fixtures at their scratch.

## Parts and tasks

| Task | Needs | Crates | Wave |
|---|---|---|---|
| `jig-expiry` | | jig-client-protocol-link, jig-client | 1 |
| `skein-background` | | skein-ui-screen-terminal, skein/tests/ui-screen | 1 |
| `tui-local` | | temper-client-tui (note, state, view) | 1 |
| `box-side` | | temper (link_file, serve), temper-box | 1 |

Later batches (designed above, specs to follow): `host-local` (the host seam and its
journey records; needs `tui-local`, `jig-expiry`), `tui-colours` (the theme on the
terminal's colours and the reprint; needs `tui-local`), `run` (the run's file read, note
write and background wait; needs `host-local`, `skein-background`), `joint` (the joint
world's stories; needs `host-local`), `binary-tui` (bare `temper`'s address, the note and
the scheme at start; needs `run`, `box-side`), `again` (`sign-in-approved` recorded again;
needs `binary-tui`, `tui-colours`).

## Critical path

`tui-local` → `host-local` → `run` → `binary-tui` → `again`, about 2 + 3 + 2.5 + 2 + 1.5
implementer-hours.
