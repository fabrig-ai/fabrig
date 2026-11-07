+++
id = "tui-replay"
part = "tui-journeys"
kind = "feature"
needs = ["tui-lines", "approval-host"]
files = ["temper/crates/temper-client-shell-tui/src/journey.rs", "temper/crates/temper-client-shell-tui/src/journey/record.rs", "temper/crates/temper-client-shell-tui/src/journey/replay.rs", "temper/crates/temper-client-shell-tui/src/journey/tests.rs", "temper/crates/temper-client-shell-tui/src/journey/tests/**"]
effort = "high"
minutes = 180
+++

## Goal

The TUI's recorder and replay at its host's seam (`temper/docs/design/client/tui/journeys.md`, 3.1, 3.3, 3.4 and section 4): `journey::Recorder` keeps what crosses `host::Host`'s seam as the file's lines, the start's facts first; `journey::Replay` gives a recording back to a fresh shipped host natively, checks each duty, deadline and key's event, and yields each mark's frame; `journey::prove` is the first replay's proof against the live prints. All of it in the host's world: reports given by the test, no terminal, no io.

Acceptance:

- Contracts, section 6, holds as written (`Facts`, `Facts::start`, `event`, `Recorder`, `Mark`, `Stale`, `Refusal`, `Replay`, `prove`), in `src/journey/record.rs` and `src/journey/replay.rs`, re-exported from `src/journey.rs`; `run::Settings.record` is `tui-record`'s, not yours.
- The recorder: the facts' lines in 3.2's order (`journey 2`, `seed`, `clocks`, `screen`, `address`, `stored`…, `credential`); a turn's `turn` written before its first kept line and its `deadline` at its end only when it kept one; each report's record, then its duties' records in 3.2's order; every key kept with its event or `none` (3.3); a mark before each key whose event is not `Typed`, in that key's turn; `mark` and `save` taking `Frame::print` of the host's frame; `save` closing a copy with `end` and the recorder going on.
- The replay: the host from the facts as the run makes it, each report given, each key's event checked before it is given (`event`), each duty and deadline compared as their lines, a mark yielding the host's frame (`Mark.frame`), its live print (`Mark.live`) and the key after it (`Mark.key`); stale at the first difference, naming the turn, the line and the two lines.
- `prove`: each mark's live print equals the replay's `Frame::print`, or `Refusal::Differs` naming the mark and the first differing line of the print; the recording returned with each `mark` and `end` field dropped (`mark`, `end` with no print).
- Tests in `src/journey/tests.rs` (the host's world; fake wire bytes built in the test, as `src/tests.rs` builds replies), within their cost:
  - a pasted link's walk and an approval's walk, each recorded and replayed to the same duties and frames, and its lines read back to the records written;
  - stale at a changed ask (a `send`'s bytes), a changed deadline, a key whose event changed, a `browse` not asked and one asked that was not recorded;
  - a key with no event (`tab`) kept and replayed;
  - `prove` refusing a mark whose print differs, and passing a whole walk, its prints dropped;
  - two saves of one walk, the first a whole recording that the second extends (the second's lines begin with the first's, but its `end`).
- `heavy cargo nextest run -p temper-client-shell-tui` passes; clippy clean with and without `--features record`.

## Inputs

- `temper/docs/design/client/tui/journeys.md`, 3.1 to 3.4 and section 4.
- `temper/docs/design/client/README.md`, 6.13 (the seam) and 6.16 (the run's order of reports).
- `temper/docs/design/client/journeys.md`, 3.1 and section 4 (the browser's recorder and replay, the model).
- `temper/docs/plan/tui-recorded/notes/contracts.md`, sections 5 and 6 (binding).
- `temper/docs/plan/tui-recorded/notes/common.md`.
- The code: `src/journey/line.rs` (`tui-lines`, merged), `src/host.rs` and `src/run.rs` (`approval-host`, merged: `Run::new` makes the host at `Time::ZERO` and `Wall::EPOCH`, then `reports()` gives `Start` and the rest), `src/tests.rs` (the host's rows, their replies' bytes); the browser's `temper/crates/temper-client-shell-web/src/journey/record.rs` and `replay.rs`, for the shape.

## Owns and may change

- Owned: the files above. `line.rs` is `tui-lines`'s: a bug there that blocks you is yours to fix in its own commit, named in the report.
- Not yours: `host.rs`, `run.rs`, `lib.rs`. If the host needs a reading the replay cannot do without (a frame's print at a point the seam does not give), stop and ask.
- Options:
  - The host's world's helpers shared with `src/tests.rs` by moving them into a `#[cfg(test)]` module both use (default), or copied: either; never change what `src/tests.rs` checks.
  - `Stale`'s `recorded` and `asked` as the lines' bytes written by `Line::write` without the newline (default).

## Steps

1. Read the inputs. Write `Facts` and `event`, then the recorder, then the replay, then `prove`.
2. The tests above, red first where a rule is new. Break each rule once by hand (no mark before an Enter; a key's event not checked; a deadline not compared), see a test fail, restore; name them.
3. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-client-shell-tui --all-targets -- -D warnings`, and with `--features record`; `heavy cargo nextest run -p temper-client-shell-tui`.
4. Commit and push.

## Report

The implementer's report, with: the tests and their times; each broken rule's failure; anything of the seam the replay found unfaithful.
