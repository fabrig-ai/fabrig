+++
id = "shell-keeper"
part = "shell-keeper"
kind = "interface"
needs = []
files = ["skein/crates/skein-shell/src/lib.rs", "skein/crates/skein-shell/src/keeper.rs", "skein/crates/skein-shell/src/process.rs", "skein/docs/design/shell.status.md", "skein/docs/development/debt/D045-shell-keeper-syscalls.md"]
effort = "medium"
minutes = 60
+++

# What a harness alone needs is fenced in `skein_shell::keeper`

## Goal

Pay skein's D045. A service author sees raw process and descriptor calls beside the `Host` loop, with nothing saying they aren't for services. They move to one public module, `skein_shell::keeper`, and every caller follows in this branch: the request allows this public change, since it's internal to fabrig.

The item names four calls. **Paid when** says "the calls only the test harness's keeper needs", and the cut's search found more with no production reader, so those go too. It also found that one of the four isn't the harness's alone: `close_keeper_fd` closes a descriptor on a startup failure in `hearth`, `temper` and `temper-forge-setup-shell`. Those callers get a name that says so.

Acceptance:

- `skein_shell::keeper` exists, and `lib.rs`'s root no longer exports any name in the table below.
- This search finds nothing: `rg -n 'skein_shell::(hosted_pipes|HostedPipes|start_binary|poll_child|signal_kept_child|abandon_binary|write_service_signal|open_signal_pipe|signal_current_thread|start_binary_in_cgroup|make_subreaper|subreaper|set_subreaper|open_pidfd|open_child_pidfd|open_cgroup|pidfd_exited|wait_cgroup_change|wait_readable|close_keeper_fd|child_id|signal_group|group_live|process_live|group_has_other_members)\b' --type rust skein smith jig temper hearth tests tools`.
- No `use skein_shell::{...}` group names them either: the cut saw two, in `temper-box/tests/real_loop.rs` and `tests/conformance/real/tests/suite/listener_readiness.rs`.
- `keeper`'s names appear in production code nowhere: `rg -n 'skein_shell::keeper' hearth/crates temper/crates/temper-forge-setup-shell temper/crates/temper/src/lib.rs` finds nothing. `temper/crates/temper/src/fake/` is a test peer and may use it.
- No function moves body, signature or behaviour: this is a change of path, and one new name for one call.
- `lib.rs`'s module docs say the root holds a service's calls, and `keeper.rs`'s says it's for harnesses alone, once.
- `skein/docs/development/debt/D045-shell-keeper-syscalls.md` is deleted, and the **Not yet** naming it leaves `skein/docs/design/shell.status.md`.

**This task can't end with fewer lines:** it moves names and adds a module, with no body to delete. Its cap: `keeper.rs` at most 45 lines, and the task's net at most +60, longer paths that rustfmt wraps included. Your report gives the net.

## Inputs

- `skein/docs/development/debt/D045-shell-keeper-syscalls.md`, whole.
- `skein/docs/design/shell.md`, 7.5, which the cut rewrote for this: the layout's `keeper` row and "What a harness alone needs is fenced".
- `temper/docs/plan/debt-28/notes/common.md`.

## Owns and may change

Owns the files in the header.

Callers, in every tree: each call's path only, and `use` lines. A line rustfmt then wraps is fine. The cut saw about 200 sites in 31 files: skein 153, temper 37, hearth 5, smith 4, jig 2. Nothing else in those files changes. This is the plan's one branch across trees, as the request allows.

A design file naming a moved call is the cut's already. `skein/docs/development/debt/reviews/` is history: leave it.

Other drafts have touched some callers (`temper-box/tests/real_loop.rs`, `temper/tests/fake_backends/box_link.rs`): find each site by its text after fab merges the trunk in. A new caller a merge brings takes the new path too.

If a name in the table turns out to have a production reader the cut didn't see, it stays at the root and your report says so. No option here needs a question.

## Steps

```rust
// skein-shell/src/keeper.rs
//! What a test harness alone needs of the processes it keeps (shell.md, 7.5).
//! A service has no use for this module: its calls are at the crate's root.

pub use crate::group::{child_id, group_has_other_members, group_live, process_live, signal_group};
pub use crate::process::{
    HostedPipes, abandon_binary, close_keeper_fd, hosted_pipes, open_child_pidfd, open_signal_pipe, poll_child,
    signal_current_thread, signal_kept_child, start_binary, write_service_signal,
};
pub use crate::system::wait_readable;

#[cfg(target_os = "linux")]
pub use crate::process::{
    make_subreaper, open_cgroup, open_pidfd, pidfd_exited, set_subreaper, start_binary_in_cgroup, subreaper,
    wait_cgroup_change,
};
```

`lib.rs` gains `#[cfg(unix)] pub mod keeper;`, and its `pub use` lines keep what's left: `exec_dying_with_parent` and `open_termination_signals` from `process`, and nothing from `group` or `system`. `group_has_other_members` isn't in `shell.md`'s list by name, but it has no production reader: move it, and say so if the search shows otherwise.

`process.rs` gains the one new name, beside `close_keeper_fd`, whose doc already says it's a test keeper's:

```rust
/// Closes a descriptor opened at startup and never given to io (shell.md, 7.5).
pub fn close_startup_fd(descriptor: Fd) {
    system::process::close_keeper_fd(descriptor);
}
```

| Caller | Becomes |
|---|---|
| `hearth/crates/hearth/src/lib.rs`, `temper/crates/temper/src/lib.rs`, `temper/crates/temper-forge-setup-shell/src/{command,parent}.rs` | `skein_shell::close_startup_fd` |
| every other `skein_shell::close_keeper_fd` | `skein_shell::keeper::close_keeper_fd` |
| every other name above | `skein_shell::keeper::<name>` |

A `sed` over `skein_shell::<name>\b` does nearly all of it, then the two `use` groups by hand. The crate's own `crate::process::` and `system::process::` don't change; a test in the crate that says `crate::<name>`, as `readiness.rs`'s `crate::signal_current_thread` does, says `crate::keeper::<name>`.

macOS: `lib.rs` and `keeper.rs` are shared, and `darwin.rs` isn't touched, since `system::wait_readable` names the same thing on both. If `aarch64-apple-darwin` is installed on your builder, `heavy cargo check -p skein-shell --target aarch64-apple-darwin`; if it isn't, say so.

No new test: a path has no rule of its own to break, and the search above holds the fence.

Run, one at a time: `heavy cargo check -p skein-shell --tests`, then `heavy cargo check --tests` naming every package whose file you changed; `cargo metadata` tells you the owners. `heavy cargo nextest run -p skein-shell`, and the same for `hearth` and `temper-forge-setup-shell`, the production callers. `cargo fmt`; each tree's old forms; `heavy cargo clippy` naming the packages you changed, `--all-targets -- -D warnings`; `python3 -I -B tools/citations/citations.py --all`; `python3 -I -B tools/debt/debt.py`. Commit, push.

## Report

As your text says, with the search's reading and the net lines.
