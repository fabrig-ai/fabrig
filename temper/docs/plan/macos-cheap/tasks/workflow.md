+++
id = "workflow"
part = "workflow"
kind = "feature"
needs = []
files = [".github/workflows/macos.yml"]
effort = "medium"
minutes = 30
+++

# The macOS workflow: every target compiled

## Goal

`.github/workflows/macos.yml` exists again, cut down to what this plan runs: on GitHub's `macos-26`, on the newest `main` a few times a day by a schedule and by hand, it installs Rust and cargo-nextest, restores its caches, and compiles every target of skein, smith, jig and temper. It blocks nothing and names nothing of whoever runs this development.

Acceptance:

- the file parses as YAML, each `run:` block passes `bash -n`, and `git show 44c5be34fa:.github/workflows/macos.yml` is its starting point, with what the sketch below removes gone;
- `cargo nextest run -p fabrig-boundary` passes, `outward` and the rest;
- nothing but this file changes.

## Inputs

- `temper/docs/design/domain/testing.md`, 2.8.
- `temper/docs/plan/macos-cheap/notes/common.md`; `temper/docs/plan/macos-cheap/notes/contracts.md`, "The workflow's seam".
- The former workflow, in this repository's history: `git show 44c5be34fa:.github/workflows/macos.yml`.

## Owns and may change

Owns `.github/workflows/macos.yml`.

Pre-approved:

- If `hearth*` does not match every hearth member of `cargo metadata --no-deps`, name them as the excludes need, and say so in the report.
- If a pinned action or nextest URL in the former file no longer matches its sketch, keep the former pin: this task checks no download.

Nothing else: a compile failure on macOS is a later task's, seen on the runner.

## Steps

Take the former file and keep its shape, so that the former's comments that stay true stay. Sketch of what differs:

```yaml
# What macOS runs (temper/docs/design/domain/testing.md, 2.8): every target
# compiled. A red run blocks nothing: it is a finding.
name: macos

on:
  schedule:
    - cron: "17 3,9,15,21 * * *"
  workflow_dispatch:

permissions:
  contents: read

concurrency:
  group: macos-${{ github.ref }}
  cancel-in-progress: false

env:
  RUST_VERSION: "1.95"
  NEXTEST_VERSION: 0.9.148
  NEXTEST_SHA256: 6c23b7fb4ca82c571fc8dfedbab45bfa773d45918f933170c85d21cd7f4b852b
  CARGO_TERM_COLOR: never
  CARGO_INCREMENTAL: "0"
  CARGO_PROFILE_DEV_DEBUG: line-tables-only
  CARGO_PROFILE_TEST_DEBUG: line-tables-only
  NEXTEST_DOUBLE_SPAWN: "0"
  TEMPER_BUNDLE: none

jobs:
  macos:
    runs-on: macos-26
    timeout-minutes: 120
```

Steps of the job, in order:

1. `actions/checkout`, as the former.
2. `The runner of each check`, as the former: `annotate.py` and `$RUN`.
3. `Rust at the workspace's rust-version, and cargo-nextest`, `id: rust`, as the former, without the `wasm32-unknown-unknown` target and without `rustfmt` and `clippy`.
4. `Restore the registry`, `Restore the target`, as the former.
5. `Compile every target`: `$RUN compile cargo check --workspace --exclude 'hearth*' --all-targets --keep-going`, under the former's `if`.
6. `Save the registry`, `Save the target`, as the former, moved after step 5. Their cache paths and keys stay.
7. `The disk`: `if: always()`, `df -h "$GITHUB_WORKSPACE" && du -sh target ~/.cargo/registry || true`.

Gone from the former: the Chrome and DejaVu steps and their `env`, formatting, clippy, the wasm checks, the TLS machine, `Build the tests`, the suites, the bundle, the browser tier, its `push` trigger, and the comment that names who a red run goes to.

Run, once, the narrowest checks: a YAML load of the file, `bash -n` over each `run:` block, `heavy cargo nextest run -p fabrig-boundary`. Commit and push.

## Report

As the implementer's text has it, with the workflow's YAML check and how each `run:` block was checked.
