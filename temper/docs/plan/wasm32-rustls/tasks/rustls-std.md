+++
id = "rustls-std"
part = "rustls-std"
kind = "feature"
needs = []
files = [
  "Cargo.toml",
  "temper/crates/temper/Cargo.toml",
  "temper/crates/temper-forge-setup-shell/Cargo.toml",
]
effort = "high"
minutes = 45
+++

# rustls's std native only, so temper-playground builds for wasm32

## Goal

`cargo clippy --target wasm32-unknown-unknown -p temper-playground -- -D warnings` fails in
rustls 0.23.41 with E0599, no `UnixTime::now` (`src/ticketer.rs:56`, 186, 200): rustls's `std`
path, which needs a clock wasm32-unknown-unknown lacks. The design already says that path is
native only: rustls "builds without std" (`skein/docs/design/tls.md`, 1), and "rustls's `std`,
whose clock that target lacks, is native only" (`skein/docs/design/tls.md`, 4). The code departs
in one line. When the four trees became one workspace, the root `Cargo.toml`'s
`[workspace.dependencies]` line for rustls took temper's `std`. skein's own line had none. So
every crate that inherits it gets `std` on every target, `skein-tls` among them. `skein-tls` is
`no_std`, refuses TLS on that target and reads no clock.

Measured at the plan's base with `cargo tree` (it builds nothing): in `temper-playground`'s wasm32
normal graph, `skein-tls` is the only crate that names rustls. With the root line's `std`
removed, `cargo tree --target wasm32-unknown-unknown -e normal,features -p temper-playground -i
rustls` shows no `std`. Natively it stays: `skein-fake-peers` asks for it on `cfg(unix)`, and so
do the skein test worlds that use rustls's std API.

Acceptance:

- The root `Cargo.toml`'s rustls line reads exactly
  `rustls = { version = "=0.23.41", default-features = false, features = ["ring", "tls12"] }`,
  under the comment sketched in step 1. Nothing else in the file changes.
- `temper/crates/temper/Cargo.toml` and `temper/crates/temper-forge-setup-shell/Cargo.toml`
  each name rustls in their `[dev-dependencies]` as
  `rustls = { workspace = true, features = ["std"] }`, on the line where `rustls.workspace = true`
  is now. Their tests use rustls's `std` API (`StreamOwned`, `ServerConnection`,
  `ClientConnection`), and now ask for it, as skein's test worlds do.
- `cargo tree --target wasm32-unknown-unknown -e normal,features -p temper-playground -i rustls`
  lists no `rustls feature "std"`.
- `heavy cargo clippy --target wasm32-unknown-unknown -p skein-tls -- -D warnings` passes (the
  lowest tier that shows the fault).
- `heavy cargo clippy --target wasm32-unknown-unknown -p temper-playground -- -D warnings`
  passes. This is the request's bar, and the gate's form of the step (`cargo clippy --target
  wasm32-unknown-unknown -p <package> -- -D warnings`, with no `--all-targets`). The
  pre-approved cases below say what to do when it does not.
- Natively, these stay green: `heavy cargo nextest run -p temper-playground` (the request's) and
  `heavy cargo nextest run -p skein-tls` (built alone, its rustls now has no `std`). Two narrow
  runs cover the std users: `heavy cargo nextest run -p temper --test suite -E
  'test(=fake::llm_run)'` and `heavy cargo nextest run -p temper-forge-setup-shell --test suite -E
  'test(=fetch::local_tls_fetch)'`. Each selects at least one test; check the counts.
- `heavy cargo clippy -p skein-tls -p temper-playground --all-targets -- -D warnings` passes.
- The report's first line, when the wasm32 clippy passes, is
  `Done: rustls's std is native only; temper-playground's wasm32 clippy passes`, so that
  agents/loop can add `temper-playground` to `.config/gate/wasm32.txt`.

## Inputs

- `temper/docs/plan/wasm32-rustls/inputs/request.md`: the request and its checks.
- `skein/docs/design/tls.md`, 1 (its item "It wraps rustls's unbuffered connection") and 4 ("The
  exception", its items "Refused on the portable target" and "No clock"). Read only.
- `skein/docs/design/testing.md`, 10 ("The portable target"): TLS builds for
  wasm32-unknown-unknown, and a world runs in plaintext there. Read only.
- `skein/crates/skein-fake-peers/Cargo.toml`: the form of a crate that asks for rustls's `std`
  natively only (`[target.'cfg(unix)'.dependencies]`).
- `skein/tests/tls/Cargo.toml`: the form of a crate that asks for it on every target it builds
  for.

## Owns and may change

- `Cargo.toml` (the root): the rustls line in `[workspace.dependencies]` and the comment above
  it, as in step 1; nothing else in the file. `[profile.dev.package.rustls]` stays.
- `temper/crates/temper/Cargo.toml` and `temper/crates/temper-forge-setup-shell/Cargo.toml`:
  their rustls dev-dependency line alone, as above.
- `Cargo.lock`: no change is expected, since features are not recorded in it. If cargo changes it
  by itself, commit what it wrote and give the diff's `--stat` in the report.
- Leave as they are: `skein/crates/skein-tls/Cargo.toml` (its `rustls.workspace = true` now
  inherits the line without `std`, as the design means), `skein/crates/skein-fake-peers/Cargo.toml`,
  `skein/tests/{tls,llm,fake-peers}/Cargo.toml` (they already ask for `std`), all Rust code,
  `.config/gate/wasm32.txt` (agents/loop adds the line once this lands), every `docs/` file (no
  design changes: `tls.md` already says this, and `tls.status.md` records no gap for it), and
  plan playground-browser's files, `temper/crates/temper-client-shell-web/**` and
  `temper/tests/playground/**`.

Pre-approved:

- If a native build fails because a crate uses rustls's std API without asking for it (a missing
  `StreamOwned`, `ServerConnection`, `ClientConnection`, `ClientConfig::builder` or a
  `std::error::Error` impl), add `features = ["std"]` to that crate's rustls line. Use the
  section the line already sits in, `[dev-dependencies]` or `[dependencies]`. Name the file
  under `## Changed` in the report. The survey found no such crate beyond the two above.
- If step 2 still shows `rustls feature "std"` on wasm32, the tree names the crate that asks
  for it. Make that ask native only, as `skein-fake-peers` does: move its rustls line to
  `[target.'cfg(not(all(target_arch = "wasm32", target_os = "unknown")))'.dependencies]`, or
  under `cfg(unix)` when the crate builds only there. Say so in the report.
- If a wasm32 clippy (step 3 or 4) fails inside rustls or a crate beneath it (rustls-webpki,
  rustls-pki-types, ring, once_cell) for a reason other than `std`, stop. rustls without `std`
  does not build on that target, and the design assumes it does. Commit and push your work. The
  report's first line is `Question: rustls 0.23.41 without std does not build for
  wasm32-unknown-unknown`, followed by the first error (crate, file:line, code, message). The
  options are:
  (a) keep `std` and give pki-types the `web` feature on wasm32, a clock that departs from
  `tls.md`, 4;
  (b) pin another rustls.
  Recommend (a) only if the error leaves no fix within rustls's features.
- If step 4 fails in a workspace crate for another reason (a lint, a native item reached from
  the portable face, another dependency's std path), keep your commits. The rustls edge is cut,
  which is this task, and the next blocker is not yours. Run step 4b. Then report `Done: rustls's
  std is native only; temper-playground's wasm32 clippy now stops at <crate>`. Under `## Left`,
  put first the first error of step 4 (the lint or error code, the message, the crate, the file
  and line, and the item), then every error and warning of step 4b, one line each, grouped by
  crate. Fix none of them, even in `skein-tls`: the next cut takes them.
- If step 3 fails in `skein-tls` itself on a lint, treat it as step 4's other-reason case: run
  step 4 anyway, then 4b, and report both under `## Left`.
- If heavy reports the wasm32 target missing on the builder (`can't find crate for \`core\``,
  "the target may not be installed"), report `Failed: wasm32 target not installed`, with the host
  from heavy's output. Never `rustup target add`: a builder's toolchain is the user's.

## Steps

1. Edit the three manifests. The root's line and comment, in the words of the design:

   ```toml
   # rustls, pinned, without std and with ring for its cryptography (skein/docs/design/tls.md, 4):
   # its std, whose clock wasm32-unknown-unknown lacks, is native only, asked for by each crate
   # that uses it.
   rustls = { version = "=0.23.41", default-features = false, features = ["ring", "tls12"] }
   ```

   The comment's words are yours within that sense. Keep it to what is true now, with no history.
   In `temper/crates/temper/Cargo.toml` and `temper/crates/temper-forge-setup-shell/Cargo.toml`,
   change `rustls.workspace = true` to `rustls = { workspace = true, features = ["std"] }`.
2. `cargo tree --target wasm32-unknown-unknown -e normal,features -p temper-playground -i rustls`
   runs directly (it builds nothing). It must list `ring` and `tls12` and no `std`. Run
   `git status --short`: only the three manifests have changed. Commit, saying why: rustls's std
   is native only (`tls.md`, 4), and the workspace's line had given it to every target.
3. `heavy cargo clippy --target wasm32-unknown-unknown -p skein-tls -- -D warnings`, once. On
   failure, read the log around the first error (`grep -n -m5 -A12 '^error'`), never whole, and
   follow `Pre-approved`.
4. `heavy cargo clippy --target wasm32-unknown-unknown -p temper-playground -- -D warnings`,
   once. On failure, read the log as in step 3 and follow `Pre-approved`.

   4b. Only for step 4's other-reason case, run once more without denying warnings, so clippy
   goes past the first failing crate:
   `heavy cargo clippy --keep-going --target wasm32-unknown-unknown -p temper-playground`. Read
   its log by `grep -n -E '^(warning|error)|^ *--> '`, never whole.
5. Natively, one command at a time: `heavy cargo nextest run -p skein-tls`; `heavy cargo nextest
   run -p temper-playground`; the two narrow runs of the acceptance; then `heavy cargo clippy -p
   skein-tls -p temper-playground --all-targets -- -D warnings`. A failure: read its log before
   anything reruns (`AGENTS.md`, 6). One your change cannot reach runs once alone and is
   reported as the implementer's text says.
6. `cargo fmt --check` (nothing changes, since no Rust changed); `python3 -I -B
   temper/scripts/old_forms.py`, where no count may rise; `python3 -I -B
   tools/citations/citations.py --base origin/main` from the repository root, which reads TOML
   comments and must print no finding your branch adds.
7. Push your branch.

Departures from the implementer's text, stated: clippy runs natively for `skein-tls` and
`temper-playground` only, not for `temper` and `temper-forge-setup-shell`. Those two change only
a dev-dependency's feature list, which their builds already unified (through `skein-tls-world`),
so their code builds the same; the narrow runs above build the test binaries that use rustls.
`docs/design/` stays untouched, as the implementer's text says.

## Report

As the implementer's text says, and also:

- `cargo tree`'s rustls lines on wasm32 (step 2), and whether `Cargo.lock` changed.
- Each heavy command's outcome, wall time and host.
- The test counts of step 5.
- The citations check's output.
