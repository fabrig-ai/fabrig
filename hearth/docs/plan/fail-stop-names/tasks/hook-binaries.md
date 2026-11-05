+++
id = "hook-binaries"
part = "fail-stop"
kind = "feature"
needs = []
files = [
  "temper/tools/stage-one/Cargo.toml",
  "temper/tools/stage-one/src/main.rs",
  "temper/tests/client-browser/Cargo.toml",
  "temper/tests/client-browser/src/bin/playground-serve.rs",
  "Cargo.lock",
]
effort = "medium"
minutes = 15
+++

# The two binaries that lack the fail-stop hook

## Goal

`temper-stage-one` and `playground-serve` are the only 2 of the workspace's 30 binary targets whose
`main` doesn't install `skein_fail_stop`'s hook first. Each takes the dependency and the call, so
that `fail-stop-test`'s check over the workspace passes.

Acceptance:

- Each `main`'s first statement is `skein_fail_stop::install();`, nothing before it but `use` and
  `const` items and comments.
- Each package has `skein-fail-stop.workspace = true` under `[dependencies]`, and `Cargo.lock`
  agrees.
- Neither binary does anything else differently.

## Inputs

- `skein/docs/foundation/programming-model.md`, 2: "Panics abort".
- `skein/crates/skein-fail-stop/src/lib.rs`: the hook.
- `hearth/docs/plan/fail-stop-names/notes/common.md`.
- The pattern: `temper/tools/gallery/src/main.rs` and `temper/tools/gallery/Cargo.toml`.

## Owns and may change

- Owns `temper/tools/stage-one/Cargo.toml`, `temper/tools/stage-one/src/main.rs`,
  `temper/tests/client-browser/Cargo.toml`,
  `temper/tests/client-browser/src/bin/playground-serve.rs`.
- May change `Cargo.lock`: the two packages' dependency lists alone, by whatever a build writes.
- The cut read both for a reason they can't take the hook and found none: `playground-serve` serves
  on one thread and catches nothing, and `temper/tools/stage-one` has no thread and no
  `catch_unwind`, so a panic already ended either. A panic now ends on `SIGABRT` rather than exit
  101, and nothing in the repository reads that status. **If you find a reason, it is a question,
  not an exemption:** stop and ask.

## Steps

1. `temper/tools/stage-one/Cargo.toml`: add the dependency. `src/main.rs`:

   ```rust
   fn main() -> ExitCode {
       skein_fail_stop::install();
       let args: Vec<String> = std::env::args().skip(1).collect();
   ```

2. `temper/tests/client-browser/Cargo.toml`: add it under `[dependencies]`, a normal dependency
   because a `src/bin` target uses it. `src/bin/playground-serve.rs`:

   ```rust
   fn main() -> io::Result<()> {
       skein_fail_stop::install();
       let mut args = std::env::args_os().skip(1);
   ```

3. Tests: `heavy cargo nextest run -p temper-stage-one`, and
   `heavy cargo nextest run -p temper-client-browser-world --bin playground-serve`. The browser tier
   that starts `playground-serve` is a slow tier: don't run it, and say so in the report.
4. `cargo fmt`, temper's `old_forms.py`, `heavy cargo clippy -p temper-stage-one -p
   temper-client-browser-world --all-targets -- -D warnings`; the common note's checks. Commit and
   push.

No new test: `fail-stop-test` adds the check that pins this, so the implementer's "a new test must
bite" doesn't arise here.

## Report

The commit, the tests and their counts, and whether either binary had any reason against the hook.
