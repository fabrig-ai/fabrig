+++
id = "shell-posix"
part = "shell-posix"
kind = "feature"
needs = []
files = ["skein/crates/skein-shell/src/posix.rs", "skein/crates/skein-shell/src/posix/process.rs", "skein/crates/skein-shell/src/posix/files.rs", "skein/crates/skein-shell/src/linux.rs", "skein/crates/skein-shell/src/linux/sync.rs", "skein/crates/skein-shell/src/linux/process.rs", "skein/crates/skein-shell/src/darwin.rs", "skein/crates/skein-shell/src/darwin/files.rs", "skein/crates/skein-shell/src/darwin/process.rs", "skein/crates/skein-shell/src/darwin/signals.rs", "skein/crates/skein-shell/src/process.rs"]
effort = "medium"
minutes = 40
+++

# What Linux and Darwin write alike lives once, in `posix`

## Goal

Finding 5 of `temper/docs/plan/debt-29/inputs/debt27-review.md`. `skein/docs/design/shell.md`, 7.5 says `linux` and `darwin` "hold only what differs", and six functions stay identical. The text stays; the code comes to it.

Acceptance:

- `status_flags`, `nonblocking`, `close` and `effective_user` are each written once, in `posix` or `posix/files.rs`, and each system takes them with its `pub(crate) use` or `pub use` line.
- `usage` is written once, in `posix/process.rs`, over each system's own `resources`, which stay apart: Linux's `ru_maxrss` is in KiB and Darwin's in bytes.
- `open_termination_signals` is written once, in `skein-shell/src/process.rs`, which calls `system::process::open_signals(crate::Watch::Termination)`. The three wrappers under it go. `open_signals` stays each system's.
- Every path a caller outside these files uses still resolves: `skein_shell::effective_user`, `system::status_flags`, `system::process::usage`, `crate::open_termination_signals`.
- `heavy cargo clippy -p skein-shell --target aarch64-apple-darwin --all-targets -- -D warnings` passes, as well as the Linux one.
- The net is fewer lines.

## Inputs

- `temper/docs/plan/debt-29/inputs/debt27-review.md`, finding 5.
- `skein/docs/design/shell.md`, 7.5 and the **Darwin check** in its verification.
- `temper/docs/plan/debt-29/notes/common.md`.

## Owns and may change

Owns the files in the header. `skein/docs/design/shell.md` stays as it is: this makes it true.

- **Each module's `# The unsafe` header** says what its calls are: a call moved in gets its words, and a module that no longer holds one loses them. The `#[expect(unsafe_code, reason = ..)]` follows.
- **`close`'s comment:** Linux's says a retry could close a reused descriptor. It's true of a close on both systems, so it moves with the function, and neither retries.
- **`linux/sync.rs`'s own `close(fd: i32)`,** which returns nothing and is re-exported as `close_unchecked`, isn't one of the six: it stays.
- **If the Darwin target isn't installed on your builder,** don't work round it: list that under `## Left`, because the trunk's end check builds it.
- **If one of the six proves not to read the same against both systems,** leave it apart, and write its reason in your report; `shell.md`, 7.5's text allows "one that differs in a type or a step".

## Steps

Move each function, keeping its body. No caller changes but those the wrappers' removal forces. `darwin/process.rs`'s and `linux/process.rs`'s `open_termination_signals` go, as does `darwin/signals.rs`'s, and `darwin/process.rs` keeps its `open_signals`.

`resources` becomes `pub(crate)` or `pub(super)` where `posix/process.rs` needs to reach it through `crate::system::process`; neither system's module calls the other's.

No new test: nothing changes what it does, and `skein-shell`'s own tests and worlds hold it.

`heavy cargo nextest run -p skein-shell`, then `heavy cargo check --tests` naming skein-shell's direct dependents in skein (`cargo metadata` lists them). `cargo fmt`; `(cd skein && python3 -I -B scripts/old_forms.py)`; `heavy cargo clippy -p skein-shell --all-targets -- -D warnings`, and the Darwin one above. Commit, push.

## Report

As your text says, with where each of the six went, the Darwin clippy's result, and the net lines.
