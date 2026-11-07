+++
id = "tui-record"
part = "tui-journeys"
kind = "feature"
needs = ["tui-replay"]
files = ["temper/crates/temper-client-shell-tui/src/run.rs", "temper/crates/temper-client-shell-tui/tests/suite/run.rs", "temper/crates/temper/Cargo.toml", "temper/crates/temper/src/tui.rs"]
effort = "high"
minutes = 150
+++

## Goal

The recording build (`temper/docs/design/client/tui/journeys.md`, section 2 and 3.1): `temper` built with its own `record` feature records every TUI it runs, from its start, through `journey::Recorder` in the run; Alt+Shift+M marks the frame and Alt+Shift+S saves, both read before the host is given them; a save stores `recording` whole in the TUI's directory, `0o600`; closing saves once more. A `temper` built without its `record` records nothing, whatever the workspace's features.

Acceptance:

- `temper`'s `Cargo.toml` has `[features] record = ["temper-client-shell-tui/record"]`; `heavy cargo check -p temper --features record` and `heavy cargo check -p temper` both build.
- Contracts, section 6's last bullet, holds: `run::Settings.record: Option<skein_io::kernel::Fd>`; the binary opens that root on the TUI's directory's parent only `#[cfg(feature = "record")]` and gives `None` otherwise; every `run::Settings` literal gains the field.
- With a root, the run makes the recorder at `Start` from the facts it gives the host (`journey::Facts`: the host's seed and its making's clocks, the size, the scheme, the address, the loaded records and credential); gives each report through `Recorder::report`; opens each iteration that reports anything as a turn and ends it with the host's deadline; takes `Key::Alt('M')` and `Key::Alt('S')` (Alt+Shift+M and Alt+Shift+S as skein's decoder gives them: check its table) out of the keys before the host sees them; a mark calls `Recorder::mark`; a save stores `Recorder::save`'s bytes at `tui/recording` through a skein-io file layer of the run's own on that root, replacing the file whole, `0o600`; closing saves before the terminal's `LEAVE`, and the run's emptiness waits for that store. A failed store draws nothing and the next save tries again.
- With `None`, the run is as today: no recorder, the two keys given to the host as any key.
- Rows in the run's world (`tests/suite/run.rs`, skein's simulated machine), red first: with a root, a walk's mark and save keys write `tui/recording` holding the facts, a `mark` and an `end`, mode `0o600`, and the host never got those keys (the recording holds no `key alt` line); a second save replaces the file with a longer recording; closing writes it once more; with `None`, nothing is written and Alt+M reaches the host. The recording read back by `journey::lines` and replayed by `journey::Replay` to its end.
- `heavy cargo nextest run -p temper-client-shell-tui` and the `temper` package's TUI tests (`-E 'test(/tui/)'`) pass; clippy clean for both, `temper` with and without `--features record`.

## Inputs

- `temper/docs/design/client/tui/journeys.md`, section 2 and 3.1.
- `temper/docs/design/client/README.md`, 6.12 (the directory) and 6.16 (the loop, closing).
- skein's `docs/design/io.md` on the file layer (`file::Request::Store`) and `ui-screen.md`, 6.1 (the decoder).
- `temper/docs/plan/tui-recorded/notes/contracts.md`, section 6 (binding).
- `temper/docs/plan/tui-recorded/notes/common.md`.
- The code: `src/run.rs` (`reports`, `report`, `close`, `is_empty`, the io it owns for signals), `src/journey/record.rs` (merged), `temper/crates/temper/src/tui.rs` (`start`, `open_root`), `tests/suite/run.rs` (the world on skein's simulated machine).

## Owns and may change

- Owned: the files above.
- Not yours: the journey module and the host. A recorder signature that blocks you is a question.
- Options:
  - The file layer on the run's existing io beside its signals (default) or an io of its own: whichever skein-io allows with its token namespaces kept apart; say which.
  - Closing's last save: before `LEAVE` (default); if the terminal's close order forbids it, after, and say so.

## Steps

1. Read the inputs. The feature and the field; the recorder's wiring; the keys; the store; closing.
2. The rows above, red first. Break each rule once by hand (the save key given to the host; no save at closing), see a row fail, restore; name them.
3. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-client-shell-tui -p temper --all-targets -- -D warnings`, and `-p temper --features record`; the focused tests above.
4. Commit and push.

## Report

The implementer's report, with: how the decoder gives the two keys; the file layer's place in the run; the rows and their failures.
