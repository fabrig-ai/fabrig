+++
id = "box-lifetime"
part = "lifetime"
kind = "feature"
needs = ["sign-in-settings"]
files = [
  "skein/crates/skein-io/src/kernel.rs",
  "skein/crates/skein-shell/src/terminal.rs",
  "skein/crates/skein-shell/src/linux/process.rs",
  "skein/crates/skein-shell/src/linux/sync.rs",
  "skein/crates/skein-shell/src/darwin/signals.rs",
  "skein/crates/skein-shell/src/darwin/process.rs",
  "skein/crates/skein-shell/src/readiness.rs",
  "temper/crates/temper-box/src/host.rs",
  "temper/crates/temper-box/src/tests/**",
  "temper/crates/temper/src/box_command.rs",
  "temper/crates/temper/tests/suite/box_listeners.rs",
  "temper/crates/temper/tests/suite/box_listeners/**",
  "temper/crates/temper/tests/suite/box_kinds.rs",
  "temper/docs/help/box.md",
]
effort = "high"
minutes = 130
+++

# A box ends whole, however it ends, and a walk of a default box

## Goal

Closing the terminal leaves Forgejo and serve running, and the next `temper box` refuses on their ports. The box takes `SIGHUP` as a stop, in its usual order. On Linux, each step starts through `internal forge-child`, so it ends with the box however the box ends. macOS keeps its direct spawn. Then walk a default box end to end, by a throwaway driver, with `sign-in-settings` merged.

This task crosses skein and temper, departing from a branch staying in its tree: the request holds the plan to two tasks, and the skein mechanism has no other reader.

Acceptance:

1. **`SIGHUP` as a stop.** With `temper box` sent `SIGHUP`: its log says what a stop says, the long steps stop in reverse order, each by a signal and its grace, nothing is left running, and its last log line ends `Stopped`.
2. **Never a second signal.** A `SIGHUP` while the box already stops, for whatever reason, does nothing. A second `SIGINT` or `SIGTERM` after a hangup still closes everything.
3. **`nohup` holds.** A box started with `SIGHUP` ignored runs on through a `SIGHUP`, on both systems. A service that doesn't ask for it changes in no way: a hangup still ends `temper serve` at once.
4. **However it ends.** On Linux a box sent `SIGKILL` takes every step with it, each by its `SIGTERM`. Each step keeps its descriptors, its environment, its process group and its pid, so the listeners, serve's pipes and first-link stdin, and the box's own supervising are as they were.
5. **macOS stays as it is:** no `forge-child`, no parent-death signal; `SIGHUP` as a stop does hold there, and a child's `SIGHUP` goes back to its default at spawn. The Darwin arms at least compile, by the check the gate uses, or the report says they could not be compiled here.
6. **The walk,** by a throwaway driver at 1440 (`## Steps`): the report says what each of its steps showed.
7. The **Not yet** entries this plan wrote for it are gone: `setup.status.md`'s that begins `6.4, 7.1: the box spawns`, `shell.status.md`'s that begins `6.3, 7.5: `Watch::TerminationAndHangup``, `io.status.md`'s that begins `7: `ServiceSignal::Hangup``, and `kernel.status.md`'s that begins `6.3: `ReadSignal` reading`. Every other entry stays.

## Inputs

- `temper/docs/design/protocol/setup.md`, 6.4 (*The hidden `internal forge-child`*), 7.1 (*A step's start*), 7.2 (*Stops*, *A closed terminal's hangup*, *Its children*), 13.1 (*The box's lifetime*).
- `temper/docs/design/domain/testing.md`, *Linux's own tests, by name*.
- `skein/docs/design/shell.md`, 3 (*A child-side exec*), 6.3 (*A terminal's closing*), 7.5; `skein/docs/design/io.md`, 7; `skein/docs/design/kernel.md`, 6.3.
- `temper/docs/design/protocol/sign-in.md`, 9.9; `temper/docs/design/client/forge.md`, 5.4, for the walk.
- `temper/docs/development/ui-repo-link-review.md`, *The real boundary*: how the last walk drove a box and a browser.
- `temper/docs/plan/ui-repo-link-2/notes/common.md`.

## Owns and may change

- Owns: this spec's `files`.
- Compiler-forced arms for the new variant, each the signal's own or the arm beside it: `temper-forge-setup-shell/src/parent.rs` (`Hangup` is `Stop::Terminate`) and `temper-client-shell-cli/src/host.rs` (`Hangup` is `limits::TERMINATED`). Neither opens `SIGHUP`, so neither sees it.
- Reader edits: `Settings` literals gain `parent: None`: `temper-box/src/tests/fixtures.rs`, `tests/host.rs`, whose `Debug` string also gains the field, `temper/crates/temper/tests/suite/box_kinds.rs`, and whatever else the compiler names.
- **A change of meaning runs its readers' tests:** every step of every box now starts through `forge-child`. Run `skein-io`, `skein-shell`, `temper-box`, `temper-forge-setup-shell`, `temper-client-shell-cli`, `temper-fake` and `temper`'s focused tests. `heavy cargo check -p temper --tests` compiles the process tier's stories: none of them runs.
- Help: `temper/docs/help/box.md`, after "Stop real backends with Ctrl-C or SIGTERM;" in its sentence's words, may say a closed terminal stops the box too. Within the page's bound.
- **A fix the walk finds,** when small: in the crate it belongs in, a failing test first where the tier allows, named in the report (this departs from the implementer's text). A fix past about thirty lines, or one that changes `sign-in.md`, 9.9's decisions, is reported with a recommendation, not made.
- Options, each pre-approved:
  - **`debt-29/shell-posix`** moves code in `skein-shell`'s `linux` and `darwin` modules. If your base has it, edit where it put `open_signals` and `signal_number`, in `posix/` or beside them. Nothing changes in the shapes below.
  - **A second `SIGHUP` already coalesced** with the first by the kernel is one signal: no test need force two reads where a fake event does it.
  - **The Forgejo and runner binaries for the walk:** the box's own fetch. If the builder can't reach the pins, place the throwaway's binaries in the cache at the paths and digests the pins name (`setup.md`, 6.6; the box's fetch finds a file there and hashes it), and say so.
  - **A browser build:** `TEMPER_BUNDLE=none` serves no browser, so the walk's binary takes a bundle as `temper/docs/design/client/browser.md`, 2.6 says. If the builder can't build it, say so, and walk by an HTTP client with a cookie jar, as the review's URL replays did, saying what a browser didn't show.
  - **A click on the new port's page:** a cookie doesn't care for ports, so the first sign-in may hold; if it doesn't, a fresh link from `temper open`, and say which.

## Steps

Failing tests first. Sketches:

```rust
// skein-io/src/kernel.rs
pub enum ServiceSignal {
    Interrupt,
    Terminate,
    /// A terminal's closing (`SIGHUP`), read only by a service that asks (shell.md, 6.3).
    Hangup,
}

// skein-shell/src/terminal.rs
pub enum Watch {
    Termination,
    TerminationAndWindow,
    /// Read SIGINT, SIGTERM and SIGHUP, but a SIGHUP already ignored at startup, which stays ignored.
    TerminationAndHangup,
}

// temper-box/src/host.rs
pub struct Settings {
    // ...
    /// The box's own id where each step starts through `internal forge-child` (protocol/setup.md, 7.1).
    pub parent: Option<u32>,
}
```

- **skein-shell, Linux:** `open_signals` adds `SIGHUP` after reading its disposition with `sigaction`, leaving it out when `SIG_IGN`: a blocked signal is never discarded, so a signalfd would read it whatever the disposition. `linux/sync.rs`, `read_signal`: `SIGHUP` is `Signalled::Termination { signal: ServiceSignal::Hangup, sender }`. The two matches on `ServiceSignal` in `linux/process.rs` (about 400 and 420), that send a service its own signal, map `Hangup` to `SIGHUP`; `darwin/process.rs` (about 443 and 453) the same.
- **skein-shell, Darwin:** the same read in `open_signals`, before it sets `SIG_IGN`; its decoding; `darwin/process.rs`' spawn adds `SIGHUP` to the signals set back to default, or a step would inherit the ignoring.
- **`temper-box/src/host.rs`,** at `Order::Spawn` (about 1118), its one `Request::Spawn`: with `settings.parent` `Some(BOX)`, `args` are `internal forge-child --parent BOX --signal terminate -- PROGRAM ARGS...`, `PROGRAM` `settings.program`; with `None`, `description.arguments()` as today. `program` stays `settings.program`.
- **`temper-box/src/host.rs`,** at `Event::Shutdown` (about 434):
  ```rust
  Event::Shutdown { signal, .. } => match signal {
      ServiceSignal::Hangup if self.machine.outcome().is_some() => {}
      ServiceSignal::Interrupt | ServiceSignal::Terminate | ServiceSignal::Hangup => self.input(Input::Terminate),
  },
  ```
  `outcome().is_some()` is what `Machine::stop` escalates on, so it's the one condition. The machine doesn't change.
- **`temper/crates/temper/src/box_command.rs`,** `run_configuration`: `skein_shell::open_signals(skein_shell::Watch::TerminationAndHangup)` in place of `open_termination_signals`, and `parent: cfg!(target_os = "linux").then(super::forge_exec::id)`, `forge_exec` being `temper_forge_setup_shell::process`, where `std::process` is allowed.

Test sketches, each biting:

- `skein-shell/src/readiness.rs`, beside `a_signal_source_waits_then_reports_its_signal`: `a_hangup_source_reports_hangup`; `a_hangup_ignored_at_startup_is_not_watched`. Each process is its own under nextest, so setting `SIG_IGN` there touches no other test.
- `temper-box/src/tests/host.rs`: `each_step_starts_through_forge_child_with_the_boxs_id`, its argument list whole; `with_no_parent_a_step_starts_directly`; `a_hangup_stops_in_the_usual_order_and_a_second_changes_nothing`; `a_hangup_during_a_termination_changes_nothing`; `a_termination_after_a_hangup_closes_the_steps`.
- `temper/crates/temper/tests/suite/box_listeners.rs`, on `box_link::Boxed` and `support::parent::Guard::observing_test`:
  - `a_hangup_stops_a_box_and_leaves_nothing_running`: `SIGHUP` to the box; exit 0; `all_exited`; the log's last line ends `Stopped`; a second box on the same directories starts, where a port still held would refuse it.
  - `a_box_ignoring_hangups_runs_on`: started by `sh -c 'trap "" HUP; exec "$0" "$@"'`; `SIGHUP`; still answering; then stopped by `SIGTERM`.
  - `#[cfg(target_os = "linux")] a_killed_box_takes_its_steps_with_it`: `SIGKILL` to the box; `all_exited` within the deadline.

**The walk,** after the focused tests are green. An untracked script, gone before the push, on a binary with its browser, with `temper`'s default runner (`temper init --box` unchanged) and separate config, data and cache directories, at 1440 by CDP (`notes/common.md`). The box runs on a pty the driver holds, as its controlling terminal, the box its session's leader:

1. `temper init --box`, then `temper box`, on the pty. Set the owned Forgejo up as `temper/docs/help/box.md`, step 4 has it, with a first repository.
2. One click on the repository's link lands on its page, signed in, with no password and no Link account page.
3. Stop the box, start it again. A click still lands.
4. Stop the box. Edit temper's clients port in `CONFIG/temper.json`. Start it. A click lands, its redirects passing `/oauth/authorize` on the new port.
5. Close the pty's master: a real hangup. Every process of the box is gone within the box's stop's grace, none left by its recorded pids, and the box's ports are free: a `temper box` on the same directories starts, and stops.

The report has a line per step: what it did, what it showed, and each redirect chain's hosts and ports. Nothing committed names a lab path.

Then the implementer's text's closing steps. Commit and push.

## Report

The implementer's text's report, and: the walk's five lines; whether the Darwin arms compiled; the box's exit code at step 5 (1 is allowed, `setup.md`, 7.2) and each fix the walk made, or recommends.
