+++
id = "ctl-stopped"
part = "stopped-box"
kind = "feature"
needs = []
files = [
  "temper/crates/temper-client-shell-cli/src/transport.rs",
  "temper/crates/temper-client-shell-cli/src/host.rs",
  "temper/crates/temper-client-shell-cli/src/tests.rs",
  "temper/crates/temper-client-shell-cli/src/tests/fixtures.rs",
  "temper/docs/help/ctl.md",
  "temper/docs/design/client/tools.status.md",
  "temper/docs/plan/test-speed/figures/ctl-stopped.md",
]
effort = "high"
minutes = 75
+++

# `temper ctl` says at once that nothing listens

## Goal

`a_story_panicking_after_its_box_stopped_leaves_the_held_record_at_its_path` takes 31 to 35 s of wall for 1.3 s of CPU in all 23 runs read. Its give-back hook calls `temper ctl DIR account list` on a stopped box, probably waiting out `limits::DEADLINE` (`host.rs`, near line 591). A person meets the same wait. First confirm the cause; then a call whose connection is refused says so at once.

Acceptance:

- **The cause, confirmed:** `figures/ctl-stopped.md` says how long `temper ctl DIR me` takes against a signed-in directory whose box is stopped, before the change, measured, and what the test's 30 s was. If the wait isn't this one, stop at that: fix what it is in the test, say so in the report, and leave section 6.3's entry in `tools.status.md` standing as it is.
- **Red first, at the lowest tier,** in `temper-client-shell-cli/src/tests.rs` beside `an_unheard_read_exits_1_at_the_link_deadline`: a read whose connection the kernel refuses exits 1 with `temper ctl: nothing listens at http://127.0.0.1:4000; is temper running?\n`, with no time advanced. Seen failing, then the fix.
- **What it leaves alone:**
  - an act refused so keeps what it sent in the small store (`kept_records`, as `an act` at the deadline does near line 650), and its lock is released;
  - a call that has heard the engine and then has its connection refused is the link down: the deadline holds, a test says so (a wait outlives a restart);
  - a connection accepted and left unanswered keeps `did not answer within 30 s`: the existing tests stay as they are;
  - `sign-in`'s call says the same at once where it shares the path.
- **The story:** the test above takes under 5 s of wall on the process tier, its assertions unchanged.
- **The help:** `temper/docs/help/ctl.md`'s exit status 1 names it (a departure from the implementer's text, which closes `docs/help/`), within the page's byte bound; shorten rather than raise it.
- **Figures** before and after (`notes/common.md`), with the tier measured at both ends.
- `tools.status.md`'s entry for 6.3 goes.

## Inputs

- `temper/docs/design/client/tools.md`, 6.3 (*Nothing listening says so at once*) and 8.2.
- `skein/crates/skein-io/src/kernel.rs`, its module comment: `Refused` means nothing listened there when the connect arrived.
- `temper/docs/plan/test-speed/notes/common.md`.
- `temper/crates/temper/tests/support/held_login.rs`, `write_hook`, the hook's lines; read, not changed.

## Owns and may change

Owns the files in the header. This stays in temper's tree: `Memory::event` in `transport.rs` sees every `io::Event` of the transport's sockets before `jig_client_shell_native::Transport` reads it (which turns a failure into `Fault::Reset`), so it can note an `io::Event::Failed` that comes to a socket with no `io::Event::Connected` yet and whose cause is the kernel's refusal.

Sketch, the outer layer:

```rust
// transport.rs
impl Memory {
    /// A connection the kernel refused, before any was made (client/tools.md, 6.3).
    pub(crate) fn refused(&self) -> bool;
}
// host.rs, in the arm that fails at `self.until`, and before it:
// a refusal with nothing heard from the engine in this call fails at once.
```

Whether "nothing heard" reads `self.link`, a `me` delivered or a field of `Memory` is yours: it must be false once the engine has answered in this call.

Readers and neighbours: `temper-client-shell-cli/src/tests/fixtures.rs` gains the refused connection (it has a `refused` field and `refuse_link` of other meanings; name yours so as not to confuse them).

Options, each decided:

- If `io::Event::Failed` doesn't carry the cause in a form this crate can read, take the smallest public accessor in `skein-io` as a neighbour edit, and say so in the report; a change of meaning there is a question.
- If a process-tier story starts a `ctl` while its box restarts and counted on the wait, make that story wait for its box's printed line (`temper/crates/temper/tests/support/**`, those lines alone), never the product, and name it in the report.
- `temper open` and the terminal client aren't this task: if they meet the same wait, list them under `## Left`.
- A help page's other lines stay as they are.

This changes what a shared path means: run the tests of `temper-client-shell-cli`, `temper-client-tools` and `temper` (the `suite` and `process` binaries).

## Steps

1. Build, stop a box, time `ctl`. Note the figure.
2. Measure the tier at the base.
3. The red test, then the fix, then the other cases. For each new test, break its rule once by hand and see it fail for that reason.
4. The story, the help, the status line.
5. Measure the tier at your head; write the figures.
6. `cargo fmt`, `old_forms.py`, clippy on the packages changed, their tests. Commit and push.

## Report

The confirmed cause; before and after, the test's and the tier's; the rule broken for each new test; any story changed; whether a person's other commands meet the wait.
