# temper-release-macos: temper's Apple Silicon release binary, from a GitHub Actions job

## Why

fabrig.ai's install line downloads `https://fabrig.ai/temper/$(uname -s)-$(uname -m)` and runs
`./temper box`, for "x86-64 Linux · Apple Silicon". The Linux binary is built off GitHub. The
Apple Silicon one, `Darwin-arm64`, needs a macOS machine, and GitHub's hosted macOS runners are
free on this public repository. The binary reaches fabrig.ai from the job's artifact, fetched by
the host that serves the site; that fetch isn't part of this plan.

## What

One new workflow, `.github/workflows/release-macos.yml`, beside `macos.yml`, with its shape:

1. **Started by hand only:** `on: workflow_dispatch` and nothing else. No schedule, no push or
   pull-request trigger. `permissions: contents: read`. One run at a time.
2. **On the runner `macos.yml` uses** (`macos-26`, Apple Silicon), with Rust at the workspace's
   `rust-version` as `macos.yml` installs it, plus the `wasm32-unknown-unknown` target, which the
   browser's bundle needs.
3. **The build:** `TEMPER_BUNDLE=embed cargo build --release --locked -p temper --bin temper`,
   at the dispatched commit, with the release profile exactly as main has it. No profile, flag
   or feature changes; nothing else is built.
4. **Checks before the upload:** the binary is a Mach-O arm64 executable (`file`), and
   `./target/release/temper --help` exits 0 and prints its usage line.
5. **The upload:** one artifact named `temper-Darwin-arm64`, holding the file `temper` and a
   file `commit` with the built commit's full sha, kept 7 days.
6. **Pinned actions**, by full commit sha with the version in a comment, as `macos.yml` pins
   them. The registry may be restored from `macos.yml`'s cache keys; a release target is not
   cached.

## Not in this plan

- Signing or notarizing (curl doesn't quarantine a download; the linker's ad-hoc signature
  suffices to run on Apple Silicon).
- A GitHub release, a tag, or any write to the repository. No secret is used.
- The fetch to fabrig.ai and the nightly start; they're ops' (on hel8, outside the repository).
- Any change to `macos.yml`, the release profile, or the code.

## Done when

The workflow is on main and passes `actionlint`-style review by reading. Its first dispatch
runs after landing, by ops.
