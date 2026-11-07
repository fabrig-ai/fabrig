+++
id = "approval-host"
part = "tui-approval-flow"
kind = "feature"
needs = ["tui-surface"]
files = ["temper/crates/temper-client-shell-tui/src/host.rs", "temper/crates/temper-client-shell-tui/src/opener.rs", "temper/crates/temper-client-shell-tui/src/run.rs", "temper/crates/temper-client-shell-tui/src/lib.rs", "temper/crates/temper-client-shell-tui/src/tests.rs", "temper/crates/temper-client-shell-tui/tests/suite/run.rs", "temper/crates/temper/src/tui.rs", "temper/crates/temper/src/tests/tui.rs"]
effort = "high"
minutes = 180
+++

## Goal

The TUI's way through the browser's approval, in its host, its run and its binary (`temper/docs/design/client/README.md`, 6.13 and 7.1, *The TUI's flow*): signed out, the sign-in screen asks for an approval through the root's native ask; the engine's answer becomes a duty to open the approval page's address, made from the TUI's own parts alone; the run gives that address to the opener as its one argument, or reports that no browser opens; the opener's end comes back as `Report::Opened`; Enter in the empty field asks again once the approval ended. The pasted link still signs in beside it.

Acceptance:

- Contracts, sections 2 and 3, hold as written: `Report::Opened`, the ask once per opening, the duty `open` once per ask answered, `Told::Browse` and `Told::Opened`, Enter's ask again, `opener::{Opener, spawn}`, `run::Settings.opener`, the run's carrying of `open`, the host's `deadline()` with the view's tick, and `temper`'s `tui::opener`.
- Host rows in `src/tests.rs`, red first where they are new rules:
  - signed out at start: `start` asks once, an `AskApproval` exchange; a held credential at start asks nothing until the engine refuses it, then once;
  - **the address made from the TUI's own parts alone**, a table of engine answers to the ask, given as `Report::Heard(Reply { .. })` with the wire's bytes: a well-formed answer opens exactly `http://127.0.0.1:8367/terminal?code=WDJB-MJHT`; at another address (`127.0.0.1:9000`) it opens that address's; an answer whose code is off the alphabet (`wdjb-mjht`, `WDJB-MJH0`, `WDJB-MJHT&next=http://x`), and one with bytes after its record carrying `http://evil.example/`, open nothing; no row opens any address but `approval_address`'s;
  - `open` once per ask: a second report with nothing new opens nothing; asking again opens the new code's address;
  - `Opened { opened: false }` and `true` reach the TUI's state (`Tui::browser()`);
  - Enter in the empty field: asks again after refused, expired, used, unknown, full and not asked; is the paste's "not a link" refusal while the take waits; a pasted link sent sets `pasted`, and then Enter in the emptied field asks again;
  - `deadline()` is the view's tick when it comes first.
  The existing rows that assert `start` asks nothing (`empty(&start(..))`) are this change's to repair: they now expect the ask.
- **The opener gets exactly one argument:** a table test of `opener::spawn`: `program` as given, `args` exactly `[address]`, the TUI's environment, and no descriptor of the terminal's among the child's 0, 1 and 2; and a row of the run's world (`tests/suite/run.rs`) in which the host's `open` with no opener reports `Opened { opened: false }` in the same iteration, and with an opener submits one spawn whose arguments are the address alone (if skein's simulated kernel answers spawns; else the submission read and the report given by the test, and say so).
- `temper`'s `tui::opener` table (`src/tests/tui.rs`): SSH set, no display on Linux, a display on Linux, macOS with no display, `PATH` with a relative directory first, a directory without the program, the program found; each the expected `Option`.
- The pasted link's stories in `temper/tests/client/tests/suite/tui.rs` still pass with the ask beside them (`heavy cargo nextest run -p temper-client-world --test suite -E 'test(/tui/)'`, or the package's name as its manifest has it); a helper of `temper/tests/client/src/tui.rs` the ask trips is yours to repair (carry `duties.open` by reporting `Opened { opened: true }`, ignore it otherwise), keeping what each story checks.
- `heavy cargo nextest run -p temper-client-shell-tui -p temper` (the `temper` package's TUI tests: `-E 'test(/tui/)'`) passes; clippy clean for `temper-client-shell-tui`, `temper` and the client world.

## Inputs

- `temper/docs/design/client/README.md`, 6.13 (the seam), 6.16 (the loop, closing), 6.11 (the binary) and 7.1 (*The root's half*, *The TUI's flow*).
- `temper/docs/plan/tui-recorded/notes/contracts.md`, sections 1 to 3 (binding).
- `temper/docs/plan/tui-recorded/notes/common.md`.
- skein's `docs/design/kernel.md` and `io.md` on spawning a process and its descriptor table (`skein_io::kernel::Spawn`, `io::Request::Spawn`, `io::Event::Spawned`, `Exited`), and `temper/crates/temper-service/src/worker/jobs.rs` for a spawn on skein's io.
- The code: `src/host.rs` (`report`, `event`, `route`), `src/run.rs` (`report`, the `debug_assert!` on `open`), `temper/crates/temper/src/tui.rs` (`start`, `lookup`, `scheme`), `temper/crates/temper-client-domain/src/asked.rs`, `jig/crates/jig-client/src/client.rs` (`Event::SignIn`'s `send_again`, which a pasted link replaces), and the wire's approval records (`rg -n 'Approval' jig/crates/jig-client-protocol-wire/src/lib.rs`).

## Owns and may change

- Owned: the files above; `opener.rs` is new, with `pub mod opener;` in `lib.rs`.
- Readers, call sites only: every `run::Settings` literal gains `opener` (`temper/crates/temper/src/tui.rs`, `tests/suite/run.rs`); every match on `host::Report` gains `Opened`; `temper/tests/client/src/tui.rs` as above. `temper-client-tui` is `approval-lines`'s: read it, change nothing there.
- Options:
  - The run's opener on its own io (default), its child's stdin, stdout and stderr opened on `/dev/null` through the spawn's descriptor table; if skein's `Spawn` cannot give a child `/dev/null`, give it pipes the run reads and drops, and say so.
  - Closing with an opener still running: abort its process alone (default) or leave it, whichever skein's io allows without waiting; never wait for it; say which.
  - The binary's `executable` check: a regular file with any execute bit, through `std::fs::metadata` (default; the binary is ordinary Rust).
  - The host's ask when the root's link is signed out with an ask held from an earlier opening: the root's `asked` is dropped only once the link leaves signed out, so after a sign-out elsewhere the root may still hold the old ask; if so, step `AskApproval` anyway at the new opening (the root replaces it) and say so.

## Steps

1. Read the inputs. The host first: the ask, the tab, `Opened`, Enter; then `opener.rs`; then the run's carrying; then the binary's choice.
2. Tests as above, red first for the new rules. Break each rule once by hand (the host opening the answer's bytes; `spawn` with two arguments; the ask at every report), see the row fail, restore; name them.
3. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-client-shell-tui -p temper -p <the client world> --all-targets -- -D warnings`; the focused tests above.
4. Commit and push.

## Report

The implementer's report, with: each row and its broken-rule failure; how the opener's descriptors and closing were done; what the client world's helper needed; the sim's answer to a spawn.
