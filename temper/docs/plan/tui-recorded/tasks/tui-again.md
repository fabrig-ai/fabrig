+++
id = "tui-again"
part = "tui-journeys"
kind = "feature"
needs = ["tui-print", "tui-record", "tui-walk"]
files = ["temper/tools/journeys/Cargo.toml", "temper/tools/journeys/src/main.rs", "temper/tools/journeys/src/lib.rs", "temper/tools/journeys/src/tui_again.rs", "temper/tools/journeys/src/tui_again/**", "temper/tools/journeys/tests/suite/arguments.rs"]
effort = "high"
minutes = 240
+++

## Goal

`cargo run -p temper-client-journeys -- again NAME` for a TUI journey (`temper/docs/design/client/tui/journeys.md`, section 8): it follows the journey's `walk` on a fresh fake box, the recording build of the TUI in a pseudo-terminal and the approval given in headless Chromium, then checks the new recording against the walk and adds it. It needs no earlier recording, so it makes a TUI journey's first one. Ordinary Rust over skein's harness, as the browser's `again` is.

Acceptance:

- `again NAME` takes the TUI's path when `tui/journeys/NAME/walk` exists, the browser's otherwise; the walk is read (`journey::walk`) before anything is built or started, an unreadable one refused naming its line.
- 8.2 as written: `temper` built with `--features record` and `fake-claude`, and the browser's bundle (`scripts/bundle.sh --loopback`), copied into a fresh scratch directory under `/tmp`; `temper box --fake-backends` from that copy serving that bundle, every directory in the scratch directory; headless Chromium from `SKEIN_TEST_CHROMIUM` through skein's fake person's browser face and harness, as the browser's `again` drives them, its printed link opened; the TUI started on a pseudo-terminal of 80 by 24 (skein's harness or `skein_shell::start_binary` with a terminal), `temper http://HOST:PORT` at the box's address, its environment as 8.2 says (`XDG_STATE_HOME` and `HOME` in the scratch directory, `TERM`, no `SSH_` variables, `DISPLAY`, `PATH` led by a scratch directory whose `xdg-open` links to `true`), its output read and dropped.
- Each step as 8.1's table: `see` by saves replayed natively (`journey::shows`), `key` written to the terminal as a person types it, `mark` and `end` by the two keys, `approve` by the save's `browse` (`journey::browsed`) opened in the browser and its "Approve" pressed by role and name; a save told by `recording` replaced; the bounds (30 s) stopping `again` naming the step; then `journey::follows` and `add NAME FILE`, its README and walk left as they were.
- Its end: the TUI, Chromium and the box stopped; the scratch directory removed, or kept and named with what failed.
- Tests, with no terminal, browser or box: `again` of a TUI journey whose walk is unreadable or missing its `end` refuses before building (in `tests/suite/arguments.rs` or `src/tests.rs`, as the browser's `starting` is tested); the key bytes a `key` step writes to the terminal for each key word (a table in the module's unit tests: `enter` `\r`, `tab`, `char`, `control c` `\x03`, `alt M` `\x1bM`, `paste` in bracketed paste as skein's decoder reads it); the TUI's environment built from a given one (`SSH_` dropped, `PATH` led by the scratch directory).
- No run of `again` here: the first journey's task runs it. `heavy cargo nextest run -p temper-client-journeys` passes; clippy clean.

## Inputs

- `temper/docs/design/client/tui/journeys.md`, section 8 (and 3.1, the saves).
- `temper/docs/design/client/journeys.md`, 8.1 and 8.2 (the browser's `again`, its harness, its bounds).
- `temper/docs/plan/tui-recorded/notes/contracts.md`, sections 6, 8 and 9 (binding).
- `temper/docs/plan/tui-recorded/notes/common.md`.
- The code: `tools/journeys/src/again.rs` (the harness's `Setup`, `Walker`, `progress`, `HORIZON`), `src/main.rs` (`record_again`, `build`, `scratch`); skein's `testing/skein-world/src/end_to_end.rs` (`Mode::Terminal`, `take_streams`) and `skein_shell::start_binary`; skein's `ui-screen.md`, 6.1 (the bytes the decoder reads); `temper-client-shell-tui`'s `journey` (`walk`, `follows`, `shows`, `browsed`, merged).

## Owns and may change

- Owned: the files above; `src/tui_again.rs` is new, the browser's `again.rs` left as it is.
- Options:
  - The TUI on its terminal through skein-world's end-to-end harness (default) or `skein_shell::start_binary` with the walker reading the terminal's master itself: whichever fits the browser's harness loop; say which.
  - The pseudo-terminal's size set by the harness, or by `TIOCSWINSZ` on the master before the TUI starts: the TUI's `screen` fact must read 24 by 80.
  - `again`'s build of `temper` with `record` beside `fake-claude` in one `cargo build` (default).

## Steps

1. Read the inputs. The dispatch; the build; the box, Chromium and the TUI; the walker; the check and `add`.
2. The tests above. Break a rule once by hand (a key word written wrong; a walk without `end` accepted), see a test fail, restore; name them.
3. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-client-journeys --all-targets -- -D warnings`; `heavy cargo nextest run -p temper-client-journeys`.
4. Commit and push.

## Report

The implementer's report, with: how the terminal was started and read; how a save is told; what is left for its first run to prove.
