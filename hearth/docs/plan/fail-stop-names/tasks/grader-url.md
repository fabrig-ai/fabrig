+++
id = "grader-url"
part = "grader"
kind = "feature"
needs = []
files = [
  "smith/benchmarks/tasks/repository/shell-cwd/task.toml",
  "smith/benchmarks/tasks/repository/shell-cwd/grader/Cargo.toml",
]
effort = "high"
minutes = 45
+++

# The shell-cwd grader names no private host, and builds as it did

## Goal

`smith/benchmarks/tasks/repository/shell-cwd/grader/Cargo.toml` has two git dependencies on the
private host. The grader takes those two lines from the frozen workspace's own root `Cargo.toml` at
grade time, so the host leaves every tracked file and the build keeps its sources.

**Why a rename won't do.** No run fetches that URL: grades are `--offline`, against a frozen bundle
held outside the repository, whose digest `seed/bundle.toml` binds. But the URL is the source's
identity: the frozen `Cargo.lock` names it, and the bundle's `.cargo/config.toml` replaces exactly
that source with `vendor/`. Another URL has no replacement and fails offline. A `path` dependency
into `vendor/` makes a second `skein-lib` whose types don't unify with `smith-domain`'s, and the
grader passes its `Token` into `tools::Repo`. A `[patch]` is keyed by the URL. A new bundle would
change the frozen seed.

Acceptance:

- `git grep -n ekanayaka -- smith` finds nothing.
- `task.toml` is at `version = 3` and its first `[[grade]]` has the command below; nothing else in
  it changes. `smith/docs/design/benchmarks.md`, 5.2: `version` changes with the grader or checks.
- `grader/Cargo.toml` is as below.
- `seed/bundle.toml`, `seed_sha256`, `grader/src/**` and `reference/**` are unchanged.
- The rehearsal of step 3 passes, with both its controls failing as they should.
- `smith-bench`'s tests and `check` pass.

## Inputs

- `smith/docs/design/benchmarks.md`, 5.2: `version`, `seed/` and `grader/`.
- `smith/benchmarks/README.md`, from "The repository suite uses `shell-cwd`": how the bundle is made.
- `smith/benchmarks/src/run.rs`, around lines 797 to 800, and `smith/benchmarks/src/grade.rs`, around
  line 220: a grade runs in a writable copy of the workspace, with a fresh copy of the grader beside
  it as `../grader`, so `>>` writes that copy. `smith/benchmarks/src/validate.rs`, 127: the same there.
- `hearth/docs/plan/fail-stop-names/notes/common.md`.

## Owns and may change

- Owns the two files above.
- **The workspace's own files stay unwritten.** Only `../grader` is written: `validate` reads the
  protected files after the grades and refuses a change to `Cargo.toml` or `Cargo.lock`.
- If a test pins a literal of this task's version or command: change that one literal and say so.
  The cut found none. If `check`, a baseline, a summary or a file in
  `smith/benchmarks/tests/recorded/**` refuses version 3: stop and ask. Baselines, summaries and
  recorded files stay as they are: they hold their own `task_version`, and a stale baseline is a
  notice, which the cut's report carries.
- `smith/benchmarks/README.md` and the design don't change: the two manifests' comments say this.
- **`grader/src/lib.rs` stays as it is,** whatever you see. Line 40 calls `tools::valid`, which the
  frozen crate very likely lacks. That is the plan's question `oracle-valid`, not this task's: if a
  run shows it, list it under `## Left`.
- **A departure from the implementer's text:** step 3's rehearsal runs plain `cargo` in `$TMPDIR`
  rather than through heavy, with its own `CARGO_HOME` there: it builds two stub crates in seconds,
  no input of the workspace. It is throwaway, gone before the push.

## Steps

1. `task.toml`: `version = 3`, and the first grade's command:

   ```toml
   command = ["sh", "-c", "grep -E '^skein-(lib|llm-world) = ' Cargo.toml >> ../grader/Cargo.toml && cp Cargo.lock ../grader/Cargo.lock && cargo test --offline --manifest-path ../grader/Cargo.toml"]
   ```

2. `grader/Cargo.toml`: `[package]` stays, and from `[workspace]` on it reads:

   ```toml
   [workspace]

   [profile.dev]
   debug = 0
   overflow-checks = true

   # Last on purpose: the first grade appends the frozen workspace's own
   # skein-lib and skein-llm-world lines here, whose git source is the one
   # its Cargo.lock and vendored sources name.
   [dependencies]
   smith-domain = { path = "../workspace/crates/smith-domain" }
   smith-protocol-llm = { path = "../workspace/crates/smith-protocol-llm" }
   smith-protocol-machine = { path = "../workspace/crates/smith-protocol-machine" }
   serde_json = "=1.0.151"
   ```

3. A rehearsal, because the bundle isn't on a builder. The frozen commit isn't in this repository
   either; the nearest tree, `80b89dd8d1`, has the form: `git show 80b89dd8d1:Cargo.toml | grep -cE
   '^skein-(lib|llm-world) = '` prints 2, each `{ git = "<URL>" }`. In `$TMPDIR`, with
   `CARGO_HOME` under it:

   - a local git repository of a workspace with two stub crates, `skein-lib` (a public `Token`) and
     `skein-llm-world`;
   - a `bench/workspace` with `skein-lib = { git = "file://..." }` and the same for
     `skein-llm-world` under `[workspace.dependencies]`, one member `crates/smith-domain` taking
     `skein-lib.workspace = true` with a function that takes a `skein_lib::Token`; its `Cargo.lock`,
     then `cargo vendor --locked vendor` with the printed replacements in `.cargo/config.toml`,
     `directory = "vendor"`;
   - a `bench/grader` of the new manifest's shape, whose test passes a `skein_lib::Token` to that
     function;
   - from `bench/workspace`, with `CARGO_NET_OFFLINE=true`, the exact `sh -c` string of step 1: it
     passes, and `Cargo.toml` and `Cargo.lock` there are unchanged.

   Two controls, each on a fresh grader copy: a renamed URL in the grader fails offline, and a
   `path` dependency into `vendor/skein-lib` fails with a type mismatch.

4. `heavy cargo nextest run -p smith-bench`; check the count. The tests that read this task are
   `recorded_catalogues_and_results` and `the_parity_suite_runs_58_attempts_interleaved`.
5. `heavy cargo run -p smith-bench -- check`. Then `heavy cargo run -p smith-bench -- validate
   repository/shell-cwd` once: it refuses where the bundle is absent, which is the expected outcome
   on a builder. Say which you saw. Don't look for the bundle yourself. What stays unobserved without
   it: the real frozen lines and their match count, the grade's time against its 80 s deadline, and
   the seed failing and the reference passing.
6. `cargo fmt`; smith's `old_forms.py`; the common note's checks. Commit and push.

Bite: the rehearsal's two controls are the test that fails for the rule's reason. No repository test
can reach a grade, since graders never run in the focused suite.

## Report

The commit, the rehearsal and its controls as they ended, each count, what `validate` said, and what
stays unobserved.
