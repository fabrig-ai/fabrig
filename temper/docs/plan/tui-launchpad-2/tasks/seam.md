+++
id = "seam"
part = "seam"
kind = "interface"
needs = []
files = ["temper/crates/temper-client-shell-tui/src/host.rs", "temper/crates/temper-client-shell-tui/src/run.rs", "temper/crates/temper-client-shell-tui/src/limits.rs", "temper/crates/temper-client-shell-tui/src/journey/**", "temper/crates/temper-client-shell-tui/src/tests.rs", "temper/crates/temper-client-shell-tui/tests/suite/run.rs", "temper/crates/temper/src/tui.rs", "temper/crates/temper/src/tests/tui.rs"]
effort = "high"
minutes = 70
+++

## Goal

The TUI's start carries two new facts, through the binary, the run, the
host's seam and the recording, so that later tasks can draw by them:

- **The terminal's background** (request item 3): the run asks it at
  every start, whatever `COLORFGBG` says, and `Start.background` carries
  the answer, or `None`; `COLORFGBG`, where it decides, still decides
  the scheme.
- **On a box** (the look request `tui-chat-title-below`): the binary
  reads the engine's link file at start whatever the address, and
  `run::Settings.boxed` and `Start.boxed` say whether its link names the
  TUI's address.
- **Recorded and replayed** as the start's facts `background R G B` and
  `box` (`temper/docs/design/client/tui/journeys.md`, 3.2).

The host only keeps both: task `arrows` hands them to the theme and the
TUI's state.

Acceptance:

- `host::Start`, `run::Settings` and `journey::Facts` are as
  `temper/docs/plan/tui-launchpad-2/notes/contracts.md`, section 2 says;
  `Host` keeps `background` and `boxed` from `Start` in fields of its
  own (a scoped `#[expect(dead_code, reason = "...")]` until task
  `arrows` reads them is allowed).
- `limits::terminal` asks the background (`Some(BACKGROUND)`) whatever
  the scheme; the run's `Start` carries `Some(rgb)` for
  `Background::Answered(rgb)` and `None` otherwise, and its scheme is the
  settings' where `COLORFGBG` gave one, else as today.
- The binary: one pure function, `on_box(link: Option<&[u8]>, engine:
  SocketAddr) -> bool` in `temper/crates/temper/src/tui.rs`, true when
  `open::target` reads the bytes as a link whose engine is `engine`;
  rows in `temper/crates/temper/src/tests/tui.rs`: a link for the
  address (true), for another port (false), no file (false), bytes that
  are no link (false). `start` reads the link file whatever the
  argument, gives `selected_address` the link only when no address was
  given (as today), and sets `Settings.boxed` by `on_box`. The secret is
  dropped there; nothing else keeps it.
- The recording: `background` after `screen` when the start has one,
  `box` after `local` when `boxed`, read back to the same facts; a
  recording without them reads as `None` and `false` and replays as it
  stands (both committed TUI journeys replay unchanged). Unreadable: a
  `background` with a number over 255, or a field missing or over, names
  the line.
- Rows added to existing tables: the line round trip (both records,
  present and absent); the replay's `Start` holding both;
  `background_selects_the_recorded_start_scheme` (in
  `tests/suite/run.rs`) extended so that an answered background is
  recorded and given in `Start` both with a `COLORFGBG` scheme and
  without one, and an unanswered one is `None`.
- `heavy cargo nextest run -p temper-client-journeys` passes with no
  reprint (no frame changes).
- Commits carry the trailer `Look-request: tui-chat-title-below`
  (`notes/common.md`).

## Inputs

- `temper/docs/design/client/README.md`, 6.12 (*The engine's link, kept
  at rest*, *Read*, *On a box*), 6.13 (the `Start` sketch, *The
  reports*, *The seam's shape*), 6.15 (*The scheme*) and 6.16 (*Its
  settings*, *Its run's settings*, *Each iteration*).
- `temper/docs/design/client/tui/journeys.md`, 3.2 (the table's
  `background` and `box` rows, *The start's facts*) and 4.1.
- `temper/docs/plan/tui-launchpad-2/notes/common.md` and
  `notes/contracts.md`, sections 2 and 3.

## Owns and may change

- Owns the files in `files`. In `host.rs`, only `Start`, the host's two
  new fields and `Host`'s start handling: task `look` edits the same
  file's `tui::theme` calls and `Host::event` at the same time.
- Compiler-forced edits, the new fields' literals alone (`background:
  None`, `boxed: false`, or the facts' values), wherever a `Start`,
  `run::Settings` or `journey::Facts` literal stands:
  `temper/tests/client/src/tui.rs` (the joint world),
  `temper/tools/journeys/src/**`, and any other; name each.
- The journey's redaction and scan (`journey/redact.rs`,
  `journey/scan.rs`): arms for the two new lines, which hold no bytes and
  no original.
- Pre-approved: if `open::target` is not reachable from `tui.rs` as
  `selected_address` uses it, call it the same way `selected_address`
  does. If a skein-ui-screen-terminal test assumed the background is not
  asked under a `COLORFGBG` scheme, it is not yours: report it under
  `## Left`.

## Steps

1. `host.rs`: the two `Start` fields and the host's two fields.
2. `limits.rs` and `run.rs`: ask always; `started` builds `Start` (and
   the recorder's `Facts`) with `background` and `settings.boxed`.
3. `journey/`: the two lines (write, read, place in the start's facts),
   `Facts`, the recorder writing them, the replay reading them into
   `Start`.
4. `temper/src/tui.rs`: `on_box`, the read whatever the argument,
   `Settings.boxed`.
5. Tests as the acceptance lists; break each once (drop the `box` line's
   write, read `background` only without `COLORFGBG`, invert `on_box`),
   see its row fail, restore.
6. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo
   clippy -p temper-client-shell-tui -p temper -p temper-client-world -p
   temper-client-journeys --all-targets -- -D warnings`; `heavy cargo
   nextest run -p temper-client-shell-tui -p temper-client-journeys`;
   `heavy cargo nextest run -p temper --test suite -E 'test(/tui/)'` and
   `heavy cargo nextest run -p temper --lib -E 'test(/tui/)'` (read the
   count; if either selects nothing, run the lib's `tests::tui` module
   by its path); `heavy cargo check -p temper-client-world --tests`.
7. Commit (with the trailer) and push.

## Report

As the implementer's text says; list each compiler-forced edit by file.
