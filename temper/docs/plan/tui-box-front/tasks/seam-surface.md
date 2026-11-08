+++
id = "seam-surface"
part = "tui-front"
kind = "interface"
needs = ["tui-surface"]
files = [
  "temper/crates/temper-client-shell-tui/src/host.rs",
  "temper/crates/temper-client-shell-tui/src/run.rs",
  "temper/crates/temper-client-shell-tui/src/tests.rs",
  "temper/crates/temper-client-shell-tui/src/journey/record.rs",
  "temper/crates/temper-client-shell-tui/tests/suite/run.rs",
  "temper/crates/temper/src/tui.rs",
  "temper/tests/client/src/tui.rs",
]
effort = "medium"
minutes = 40
+++

# The TUI's seam gains the front's names, with no behaviour yet

## Goal

The interface step of the TUI's seam in front of a box (`notes/contracts.md`, section 8; `temper/docs/design/client/README.md`, 6.17, *The seam*): `temper-client-shell-tui` gains exactly the names the binary's `front-command` and the TUI's `tui-seam` build on, so that the binary can be written while the seam's bodies are. Nothing a TUI does changes: the run reads nothing of the front yet, and the host only hands a box's line to the TUI's state.

Acceptance, by `notes/contracts.md`, section 8:

1. **The names**, exactly the contracts', each with its contract comment:
   - `host.rs`: `Start` gains `pub front: bool` after `note`; `Report` gains `Front(temper_client_tui::BoxLine)` after `Local`.
   - `run.rs`: `Settings` gains `pub front: Option<Front>` after `record`; `pub struct Front { pub link: skein_lib::Secret, pub lines: skein_io::kernel::Fd }`, re-exported nowhere else (`run::Front`).
2. **The host's arm:** `Report::Front(line)` tells the TUI's state `Told::BoxLine { line }` and nothing else (the contract's meaning, `tui-surface`'s state; the views draw it once they land). `Start.front` is read by nothing yet: `tui-seam` gives it its meaning (`Told::Front`, the handed link's sign-in).
3. **The run ignores `front`:** it gives `Start { front: false, .. }` and `local` as today, never reports `Front`, and neither reads nor closes `front.lines` nor uses `front.link` (`tui-seam` adopts and reads them). Say in a comment at the field that the run reads it from `tui-seam` on.
4. **The recorder** (`journey/record.rs`): `Facts::start`'s `Start { .. }` gains `front: false`; `Recorder::report`'s match on `Report` gains `Report::Front(_)`, which gives the report to the host and keeps no line for it (the record `box-line` is `tui-seam`'s). The replay matches on lines, not reports, and doesn't change; `Facts` gains no field here.
5. **Literals the compiler forces**, the field alone: `temper/crates/temper/src/tui.rs`' `Settings { .. }` gains `front: None` (`front-command` sets it); `temper/tests/client/src/tui.rs`' `Start { .. }` gains `front: false`; the crate's own `Start { .. }` literals in `src/tests.rs` and `Settings { .. }` literals in `run.rs`' tests and `tests/suite/run.rs`.
6. **One test,** a failing test first: `src/tests.rs`, `a_box_line_report_is_told_to_the_tui`: a started host given `Report::Front(BoxLine::Restarting { step })` and then `Report::Front(BoxLine::Log { line })` has its state's `box_state()` restarting that step and `box_log()` holding the line (the host's `state` is reachable from the crate's tests). No status bullet is deleted: `tui-seam`'s stays.
7. **Runs,** through `heavy`, counts in the report: `cargo nextest run -p temper-client-shell-tui`; `cargo check -p temper --tests`; `cargo check -p temper-client-world --tests`; `cargo check -p temper-client-journeys --tests`.

## Inputs

- `temper/docs/plan/tui-box-front/notes/common.md`; `notes/contracts.md`, sections 7 and 8.
- `temper/docs/design/client/README.md`, 6.17, *The seam*.
- The code: `host.rs` (`Start` 36, `Report` 46, `report`'s match 148-182); `run.rs` (`Settings` 24, `started` 354-388, its test `Settings` about 1008); `journey/record.rs` (`Facts::start` 26-41, `Recorder::report` 88-103); `src/tests.rs` (`start` 24 and the other `Start { .. }` literals about 263, 661, 1207); `tests/suite/run.rs:108`; `temper/crates/temper/src/tui.rs:221`; `temper/tests/client/src/tui.rs:77`.

## Owns and may change

Owns the names and the arms above. `tui-seam` builds the bodies on them right after, in the same files; `front-command` uses `run::Settings.front` and `run::Front` in the temper crate: change nothing there but the one field. Not `temper-client-tui`.

Options, each decided:

- **`Report::Front` before `Start`** panics as any report before it does (the existing assertion); no new check.
- **A `Front` report after closing** does nothing, as every report but `Heard` and `Kept` (the existing guard).

## Steps

1. The failing test of acceptance 6; then the names, the host's arm, the recorder's arm, the literals; the narrowest test, `heavy cargo nextest run -p temper-client-shell-tui -E 'test(/a_box_line_report_is_told_to_the_tui/)'`.
2. The runs of acceptance 7; `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-client-shell-tui -p temper -p temper-client-world --all-targets -- -D warnings`; `python3 -I -B tools/citations/citations.py --base origin/main`.
3. Commit (why: the seam's front names, so the binary and the seam's bodies are written apart), with `Look-request: tui-box-front` directly above the other trailers; push.

## Report

As the implementer's text says, with the test and the rule broken by hand for it (the arm telling nothing), and each literal filled.
