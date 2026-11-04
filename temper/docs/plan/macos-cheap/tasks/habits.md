+++
id = "habits"
part = "story"
kind = "feature"
needs = ["pin-move"]
files = ["temper/crates/temper/src/lib.rs", "temper/crates/temper/src/tests.rs", "temper/crates/temper/tests/support/parent.rs", "temper/crates/temper/tests/support/csv_repair.rs", "temper/docs/design/domain/testing.md"]
effort = "high"
minutes = 75
+++

# What the story needs on macOS, and that every target compiles

## Goal

The Linux habits the acceptance story reaches stop stopping it, and nothing it does not reach changes: `serve --box`'s pipe check becomes Linux's alone, the login answers on macOS, `parent::Guard` exists on macOS, and the story's CI stops using GNU's `base64 -w0`. Then every target of skein, smith, jig and temper compiles for `aarch64-apple-darwin`, as far as a Linux builder can tell.

Acceptance:

- the pipe check and the login behave on Linux exactly as they do today, by their existing tests;
- a focused test shows the macOS login path answering the same name as the `/etc/passwd` path, on Linux;
- `temper` compiles for Linux with no warning and, by the check of step 5, for `aarch64-apple-darwin`, every target;
- the story passes on Linux, at the pin's binaries.

## Inputs

- `temper/docs/design/domain/testing.md`, 2.8.
- `temper/docs/design/protocol/setup.md`, 7.2 (the pipes), 8.2 (the person's login).
- `temper/docs/plan/macos-cheap/notes/common.md`.

The survey's findings, read but not compiled: the two uses of `parent::Guard` outside a Linux gate are `tests/support/story_command.rs:38` and `tests/support/forge_land/rung.rs:546`; `parent.rs`' imports of `Deadline, Expired, Starved` and of `Duration` go unused on macOS; `box_command.rs:265` and `boxed.rs:1053` already gate on Linux; the story calls `delegate`, `wait`, `read`, `edit`, `write` and `finish`, so it never reaches `/usr/bin/rg`, and `/usr/bin/git` exists on macOS.

## Owns and may change

Owns the files above.

Reader and neighbour files, for compile fixes alone: any test crate of skein, smith, jig or temper, where a Darwin compile fails: a `#[cfg(target_os = "linux")]` on the test, module or binary, with a comment of its reason, and nothing else. No step crate, and no production code outside `temper/crates/temper/src/lib.rs`, without asking.

A departure from the implementer's text: `temper/docs/design/domain/testing.md`'s table in 2.8, "Linux's own tests, by name", is yours to add a row to, for a temper test you gate; a smith or jig test you gate goes in your report, which the plan takes up.

Pre-approved:

- **It fetches,** only as `conformance` does: `https://codeberg.org/forgejo/forgejo/releases/download/v17.0.1/forgejo-17.0.1-linux-amd64` and `https://code.forgejo.org/forgejo/runner/releases/download/v13.2.1/forgejo-runner-13.2.1-linux-amd64`, by `curl`, into `target/pins/forgejo-17.0.1/forgejo` and `target/pins/forgejo-runner-13.2.1/forgejo-runner`, each checked against the pin's digest, `chmod 0755`, never committed, gone before the push. The builders' environment still names the old binaries.
- **A throwaway stand-in for Apple's C compiler,** a departure from `AGENTS.md`, 4: untracked under `target/darwin-check/`, gone before the push. See step 5. If `ring`'s build script still refuses after fifteen minutes' trying, fall back to the check of `notes/common.md` over what it reaches, and say so in the report: the workflow then has the rest.
- If a fix for the Darwin compile would touch production code outside `lib.rs`, or more than a gate: leave it, list it under `## Left`, and go on. The first run on the runner takes it.

## Steps

1. `lib.rs`, `serve`: the pipe check runs under `cfg!(target_os = "linux")` alone, its words unchanged.
2. `lib.rs`, `login`: on `cfg!(target_os = "macos")` the answer is `/usr/bin/id -un`'s, stdin null, environment cleared, its one line without its line feed, an error naming the program; otherwise the `/etc/passwd` reading, unchanged. Both compile on both systems, as two functions, so the first is tested on Linux. Its doc comment says why. A scoped `#[expect(clippy::disallowed_methods, reason = "...")]` if the tree's lints ask it.
3. `parent.rs`: for every system but Linux, `pub struct Guard { pub child: Option<Child> }`, `Guard::new` taking the child with no parent-death signal, `Drop` ending it with `end_child` if it is still held; gate the imports only Linux uses. What `Guard` offers beyond `new` and `child` stays Linux's.
4. `csv_repair.rs:39`: `| base64 | tr -d '\n'`, both systems' tools. The other copies are not reached and stay: `forgejo.rs:549`, `landing_by_turn.rs:17`, `docs/development/acceptance.md`, `tests/real-loop/tests/fixtures/b1/forge.json`.
5. The Darwin compile. `cargo check` links nothing, so `ring`'s C need only seem to compile:

   ```sh
   mkdir -p target/darwin-check
   # target/darwin-check/cc: writes an empty file at its -o argument, exit 0.
   # target/darwin-check/ar: writes "!<arch>\n" at its archive argument, exit 0.
   heavy env CC_aarch64_apple_darwin=$PWD/target/darwin-check/cc AR_aarch64_apple_darwin=$PWD/target/darwin-check/ar \
       SDKROOT=$PWD/target/darwin-check \
       cargo check --target aarch64-apple-darwin --workspace --exclude 'hearth*' --all-targets --keep-going
   ```

   Fix what fails, within this task's scope. The workflow's own compile is this same command, on the runner.
6. Focused tests of what you changed; `heavy cargo nextest run -p temper`. Then the story, on Linux:

   ```sh
   heavy env TEMPER_BUNDLE=none TEMPER_FORGEJO=<absolute>/forgejo TEMPER_FORGEJO_RUNNER=<absolute>/forgejo-runner cargo nextest run --profile process-temper -p temper --test process -E 'test(=a_csv_change_passes_ci_and_a_blocking_agent_review_on_the_owned_forge_then_lands_and_survives_restart)'
   ```

   Check that it selected one test.
7. `cargo fmt`; the old-forms counts; clippy of the changed packages. Delete `target/pins` and `target/darwin-check`. Commit and push.

## Report

As the implementer's text has it, with: whether the stand-in let `ring` through, and the compile's last line; each gate added and its reason; each failure left.
