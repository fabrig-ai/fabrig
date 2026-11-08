+++
id = "tui-seam"
part = "tui-front"
kind = "feature"
needs = ["seam-surface"]
files = [
  "temper/crates/temper-client-shell-tui/src/host.rs",
  "temper/crates/temper-client-shell-tui/src/run.rs",
  "temper/crates/temper-client-shell-tui/src/limits.rs",
  "temper/crates/temper-client-shell-tui/src/tests.rs",
  "temper/crates/temper-client-shell-tui/src/journey/line.rs",
  "temper/crates/temper-client-shell-tui/src/journey/record.rs",
  "temper/crates/temper-client-shell-tui/src/journey/replay.rs",
  "temper/crates/temper-client-shell-tui/src/journey/redact.rs",
  "temper/crates/temper-client-shell-tui/src/journey/scan.rs",
  "temper/crates/temper-client-shell-tui/src/journey/walk.rs",
  "temper/crates/temper-client-shell-tui/src/journey/tests.rs",
  "temper/crates/temper-client-shell-tui/src/journey/line/tests.rs",
  "temper/crates/temper-client-shell-tui/src/journey/redact/tests.rs",
  "temper/crates/temper-client-shell-tui/src/journey/walk/tests.rs",
  "temper/crates/temper-client-shell-tui/tests/suite/run.rs",
  "temper/tools/journeys/src/tests.rs",
  "temper/docs/design/client/README.status.md",
  "temper/docs/design/client/tui/journeys.status.md",
]
effort = "high"
minutes = 130
+++

# The TUI's seam in front of a box: the handed link, the box's lines, quitting, `/log`, and their recording

## Goal

The host and the run of the TUI in front of a box (`temper/docs/design/client/README.md`, 6.17, and the pointers in 6.13 and 6.16), on `tui-surface`'s surface and `seam-surface`'s names: the run takes the front's settings (`run::Front`, which `front-command` fills), answers the first duty `local` with the handed link, reads the box's lines and reports them, and closes at their end; the host signs in by the handed link with `held`, tells the TUI's state each line, asks before quitting while work runs, and takes `/log` as a command of its own; the recording keeps the start fact `front` and the record `box-line`. The views (`tui-welcome`, `tui-front-views`) draw what this tells; the redaction of a `box-line` link is `tui-redact`'s.

Acceptance, by `notes/contracts.md`, section 8 (and 7 for what it calls), and 6.17's *Signed in by the handed link*, *The seam*, *The box's lines*, *Quitting*, *`/log`*, *The playground*'s *Read again*, 6.13's *Its pages*:

1. **The seam's names are `seam-surface`'s,** on your base: `Start.front`, `Report::Front`, `run::Settings.front`, `run::Front`, the host's arm telling `Told::BoxLine { line }` for each `Report::Front`, the recorder's arm keeping no line, and the run ignoring `front`. This task gives them their bodies, below; their names and shapes don't change.
2. **The handed link** (host): at `Start` with `front`, the host tells `Told::Front`, keeps the start's credential as today, starts the root holding none (`Event::Start { with: None, .. }`) and begins the need for the link whatever the credential and the note (the note's `SignedOut` doesn't stop it); every sign-in of that need is `SignIn { with: Link { secret, held: self.credential.clone() } }`, so a sign-in the engine still holds is kept and none made. Out of front nothing changes: `begin_local` still gates on no credential and no signed-out note, and sends `held: None`. The rest of the need (the digest, `link_used`/`link_expired` read again, any other end falling back) is 6.14's, unchanged.
3. **The run:** `Start.front` is `settings.front.is_some()`, and `Start.local` is true with it whatever `link_file` is. The first duty `local` is answered at once with `Report::Local { link: Some(handed) }`, the handed link taken; each later one as today (the link file, or none). The run adopts `lines` on its own io as a read pipe (`io.adopt_read_pipe`, as `temper-service`'s `adopt_box` adopts the box's), its io limits grown for it (`sockets` 3; `intake` and `receive` at least `BoxLine::MOST`), and reads it a line at a time (`stream::Read::Scan { until: Delimiter::LF, max: BoxLine::MOST }`, as `temper-service`'s `box_io` reads its pipe): a line with its line feed is given to `BoxLine::read` without it and, read, queued; a read of `MOST` bytes with no line feed is dropped with the rest of its line, to the next line feed; a last piece with no line feed at the end is dropped. Each queued line is reported as `Report::Front(line)` once `Start` was given, in order, after `Kept` and before the transport's events. The lines' end (`Up::End` or `Up::Failed`, or the pipe's `Failed` or `Closed` event) closes the run as SIGTERM does (`close()`). The `unreachable!` arm of `fn event` takes the pipe's events; a `Stream` of another owner stays unreachable.
4. **The lines told** (host): as `seam-surface` left it, `Report::Front(line)` tells `Told::BoxLine { line }`; nothing more.
5. **Quitting** (host), in front: Ctrl+C (`Command::Close`) and `/exit` (`Event::Command { command: Exit }`, after its `Told::Cleared`) close when the question stands, when the box gave up (`box_state()` is `GaveUp`), or when `tui::work_runs(&root, state)` is false; else they tell `Told::Quitting`. `Event::Stop` closes; `Event::Stay` tells `Told::Stay`. Out of front both close at once, as today.
6. **`/log`** (host): `Event::Log`, in front, is `terminal::Act::Dismiss` to the root, then `Told::Log` and `Told::Cleared`; out of front it does nothing. `send()`: words that begin a command (`"/"`, no white space) and begin none of the root's commands offered there, in front and with `tui::LOG_COMMAND.starts_with(words)`, are `Event::Log`; else today's unknown command.
7. **Back to the start's address** (host, `fn launchpad`, in front or not, 6.13 *Its pages*): after a report in which the link is live, `me` read, and `root.pages().project()` names a project `me`'s projects don't hold, `went(Address::Inbox { filter: Filter::ALL })`; the existing move to the first project then applies from there at the next project `me` names.
8. **The recording** (`journey/*`, `tui/journeys.md`, 3.2 and 4.1): `Line::Front`, word `front`, no field, among the start's facts after `local` (`Facts.front: bool`, written by `Recorder::new` and given by `Facts::start` as `Start.front` in place of `seam-surface`'s `false`, the run giving it from its settings); `Line::BoxLine(Box<[u8]>)`, word `box-line`, its bytes in quotes, a turn's report line for `Report::Front(line)`, written `line.print()` (`Recorder::report`, in place of `seam-surface`'s arm keeping none) and replayed as `Report::Front(BoxLine::read(bytes))`, a `box-line` that doesn't read unreadable, naming its line; both refused out of their places, as `local` and `found` are. In the redaction and the scan, `box-line`'s bytes are a text field (`journey/redact.rs`' field table), and its kind's word is `box-line`; its link is not yet an original (`tui-redact`'s). `journey/walk.rs` changes only where the compiler forces it.
9. **Readers, literal fields alone:** `temper/tools/journeys/src/tests.rs`' `Facts { .. }` literals gain `front: false`, and so do the crate's own `Facts { .. }` literals (`journey/tests.rs`, `journey/walk/tests.rs`, `journey/redact/tests.rs`, `journey/replay.rs`). `seam-surface` filled the `Start` and `Settings` literals in the temper crate and the joint world; this task touches neither file.
10. **Tests,** a failing test first for each (`notes/common.md`, *Tests stay cheap*):
    - `src/tests.rs`' `the_engines_link_signs_in_and_falls_back`: `LocalRow` gains `front: bool` (false in every existing row) and `LocalWant::SendHeld` (the sign-in's exchange carries the held credential); rows `front fresh` (no credential: `Read`, then the handed link found, `Send`, signed in, `Keep`), `front held` (a credential: `Read`, not the approval; `SendHeld`; answered signed in with no credential: no `keep` duty), `front signed-out note` (note `SignedOut`: still `Read`).
    - `src/tests.rs`' `each_event_becomes_its_acts`: rows for `Stop` (close), `Stay` (the question lowered), `Log` (a `Dismiss` act, `/log`'s note standing in front; nothing out of front).
    - `src/tests.rs`, one new table test, `in_front_quitting_asks_while_work_runs`, rows: out of front, Ctrl+C closes with work running; in front, nothing running, Ctrl+C closes; work running, Ctrl+C asks (no close duty, `quitting()`), then Ctrl+C again closes; `/exit` asks alike; asked, then `Stay`, then Ctrl+C asks again; asked, then `Stop` closes; the box given up (a `Report::Front` of `gave-up`), Ctrl+C closes with work running; `/log` sent whole and begun (`/lo`) in front stands its note, and out of front is the unknown command's note; each `BoxLine` reported changes the state as `tui-surface`'s table says (one row each). Work running is a Home chat of the project delivered working (`ChipPhase::Active(RunStage::Due)`, `WaitsOn::Nothing`), as `the_host_goes_to_the_launchpad_and_follows_the_root` delivers Home's reads.
    - `src/tests.rs`' `the_host_goes_to_the_launchpad_and_follows_the_root`: a row where `me` loses the root's project (the playground's reset) and the host goes back to `Inbox`, then to the next project `me` names.
    - `tests/suite/run.rs`: `every_way_out_leaves_the_terminal_as_it_was_found` gains the row "the box's lines end" (a run in front whose lines' writer closes: `LEAVE` written last, exit 0); one new test, `a_front_answers_its_first_read_with_the_handed_link_and_reports_the_boxs_lines`: the first `Local` report holds the handed link, a later duty reads the link file; lines written to the pipe, one split across two writes, one past `MOST` and one with an unknown word, reach the host as the readable ones alone, in order (read through the run's recording, `Line::BoxLine`, or the host's frame state; your choice, said in the report).
    - `journey/line/tests.rs`' `records_have_their_exact_text_and_round_trip`: rows for `front` and a `box-line`; `unreadable_rows_name_their_line`: `front` with a field, `box-line` with none; `journey/tests.rs`' `local_records_are_refused_outside_their_places`: `front` in a turn and `box-line` among the facts; and one new test, `front_reports_replay_through_box_line_read`: a walk in front (the handed link found, a `restarting` and a `log` line reported, Ctrl+C asking) recorded and replayed to the same duties and frames, as `local_reports_replay_and_compare_the_new_duties` does.
    The handed link and any link line are named in words and compared by equality, never printed (`notes/common.md`, *Never shown*).
11. **Status:** delete the **Not yet** bullet naming `tui-seam` in `temper/docs/design/client/README.status.md` and the one in `temper/docs/design/client/tui/journeys.status.md`. No other design line.
12. **Runs,** through `heavy`, counts in the report: `cargo nextest run -p temper-client-shell-tui`, and again with `--features record` where its suite's recording stories are compiled only with the feature; `cargo nextest run -p temper --lib -E 'test(/tui/)'`; `cargo check -p temper --tests`, `cargo check -p temper-client-world --tests`, `cargo check -p temper-client-journeys --tests`; then, as readers of a change of meaning (`launchpad` now goes back to the start's address), `cargo nextest run -p temper-client-world --test suite -E 'test(/^tui::/)'` and `cargo nextest run -p temper-client-journeys`: a red there is yours where this change made it.

## Inputs

- `temper/docs/plan/tui-box-front/notes/common.md`; `notes/contracts.md`, sections 5, 7 and 8.
- `temper/docs/design/client/README.md`, 6.17 (*Signed in by the handed link*, *The seam*, *The box's lines*, *Quitting*, *`/log`*, *Read again*), 6.13 (*Its pages*, *back to the start's address*; *The TUI's events*), 6.14 (*Signed in from the engine's link*), 6.16; `temper/docs/design/client/tui/journeys.md`, 3.2 and 4.1; `temper/docs/design/protocol/setup.md`, 7.6 (the box's lines, the front's descriptors).
- The code: `host.rs` (`Start` 36, `Report` 46, `Host` 72-100, `report` 140-237, `start` 239-272, `begin_local` 274, `found` 298-326, `event` 350-386, `send` 400-440, `launchpad` 448, `close` 625); `run.rs` (`Settings` 24, `Run` 39-75, `new` 75-160, `complete` 190, `event` 218-262, `close` 264, `report` 287-340 with `duties.local` 329-335, `started` 354-388, `reports` 390-461); `temper-service/src/system.rs:526-552` (`adopt_box`) and `:1120-1140` (`box_io`, a line read with `Read::Scan`); `journey/line.rs` (`Line` 9-47, writer 140-230, parser 536-590), `record.rs` (`Facts` 14-41, `Recorder::new` 56-80, `report` 88-103), `replay.rs` (facts 270-300, reports 168-210, places 100-135 and 360-410), `redact.rs` (field table 60-90, `found` 138-150), `scan.rs` (`kind` words); tests as named above, with `LocalRow` at `src/tests.rs:455-485` and `local_row` 659.
- `tui-surface`'s items: `BoxLine`, `Told::{Front, BoxLine, Quitting, Stay, Log}`, `Tui::{front, quitting, box_state}`, `BoxState`, `Event::{Stop, Stay, Log}`, `LOG_COMMAND`, `work_runs`.

## Owns and may change

Owns `temper-client-shell-tui` (the files listed), the literal fields of acceptance 9, the two status bullets. `temper-client-tui` is not this task's (`tui-welcome` and `tui-front-views` change it in the same wave): if a `temper-client-tui` item is missing or wrong, report it under `## Left`, don't change it. Not the temper crate: `front-command` changes `temper/crates/temper/src/tui.rs` and adds `temper/crates/temper/src/tui_front.rs` in the same wave, on `seam-surface`'s names, and every `Start { .. }` and `Settings { .. }` literal those names forced is `seam-surface`'s, landed. Nor `temper/tests/client/**` (`world-front`'s).

Options, each decided:

- **The line buffer:** a `Vec<u8>` of at most `MOST` bytes and a `skipping` flag in the run (ordinary Rust, as `run.rs` is); if the io layer's `Scan` already gives a line whole or `MOST` bytes, keep only the flag; say which.
- **Lines before `Start`** wait in the queue (none is reported before it, 6.16); a lines' end before `Start` closes the run as one after it.
- **The handed link and `with_local_link`:** the run answers the first duty from `settings.front` before it looks at the link file; a run in front with no link file answers later duties `None`, as today.
- **`held` beyond the start:** the need begun at a front's start sends `held` on each of its sign-ins (its second read too); a need begun later (an expiry) is 6.14's, `held: None`.
- **A test fixture for work running** that `src/tests.rs` lacks goes at the bottom of `src/tests.rs`.

## Steps

1. The failing tests of acceptance 10, seen failing; then `host.rs` (seam, handed link, lines told, quitting, `/log`, back to start), `run.rs` (front settings, first duty, the pipe, its end), the journey's records; the narrowest test after each, e.g. `heavy cargo nextest run -p temper-client-shell-tui -E 'test(/in_front_quitting_asks_while_work_runs/)'` and `heavy cargo nextest run -p temper-client-shell-tui --test suite -E 'test(/front/)'`.
2. The literal fields; the status edits; the runs of acceptance 12; `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-client-shell-tui -p temper-client-journeys --all-targets -- -D warnings`; `python3 -I -B tools/citations/citations.py --base origin/main`.
3. Commit (why: the TUI runs in front of a box: signed in by the handed link, reading the box's lines, asking before it stops the box), with `Look-request: tui-box-front` directly above the other trailers; push.

## Report

As the implementer's text says, with each test added or extended and the rule broken by hand for it (for instance Ctrl+C closing with work running, or the lines' end left open), the line buffer chosen, how the run test reads the reports, and any reader's red under `## Left`.
