+++
id = "release-workflow"
part = "release-macos"
kind = "feature"
needs = []
files = [
  ".github/workflows/release-macos.yml",
  "temper/docs/design/release.status.md",
]
effort = "high"
minutes = 25
+++

# Apple Silicon's release file, from a dispatched GitHub workflow

## Goal

The site serves temper for Apple Silicon at `/temper/Darwin-arm64`, and building it needs a Mac. A new workflow, `.github/workflows/release-macos.yml`, started by a dispatch alone on GitHub's `macos-26`, builds `temper` for release with its browser bundle embedded, checks that it is an arm64 Mach-O that prints its usage, and uploads it with the commit it was built from as the artifact `temper-Darwin-arm64`. Nothing in this plan runs it: its first dispatch comes after landing.

Acceptance, each checked by reading the file (`notes/common.md`), item by item, and reported with the file's line numbers:

1. **The trigger:** `on:` holds `workflow_dispatch:` alone, with no inputs; no `schedule`, `push`, `pull_request` or other event. `permissions:` at the top level is `contents: read` and nothing else; no job widens it. `concurrency:` is the group `release-macos`, with no `${{ ... }}` in it, so one run at a time whatever the ref, and `cancel-in-progress: false`.
2. **The runner and Rust:** one job on `runs-on: macos-26`, `timeout-minutes: 120` (the plan's question `release-macos-timeout`, its default), `shell: bash`. Rust is installed as `macos.yml`'s step does (the check that `rustup` is on the image, `rustup toolchain install "$RUST_VERSION" --profile minimal`, `rustup default "$RUST_VERSION"`), with `--target wasm32-unknown-unknown` on the install; `RUST_VERSION` is `"1.95"`, the workspace's `rust-version` in the root `Cargo.toml`. No `cargo-nextest`.
3. **The build:** one step runs, literally, `TEMPER_BUNDLE=embed cargo build --release --locked -p temper --bin temper`. Nothing in the workflow sets a profile, flag or feature: no `CARGO_PROFILE_*`, `RUSTFLAGS`, `CARGO_INCREMENTAL`, `CARGO_BUILD_*`, `--features`, `--target`, `--config`, no cargo configuration written. The workflow's `env:` holds `RUST_VERSION` and `CARGO_TERM_COLOR: never` alone. Nothing else is built or run through cargo.
4. **The checks, before the upload:** `file -b target/release/temper` is printed and must contain `Mach-O 64-bit`, `executable` and `arm64` (libmagic's versions order these words differently, so each is a substring); `./target/release/temper --help` must exit 0 and its first line start with `usage: temper ` (`temper/crates/temper/src/commands.rs`, `help`). Each check that fails prints an `::error title=...::` annotation saying what it found and exits 1, so the upload is skipped.
5. **The upload:** one artifact, `name: temper-Darwin-arm64`, `retention-days: 7`, `if-no-files-found: error`, whose root holds exactly two files: `temper`, the built binary, and `commit`, `git rev-parse HEAD`'s one line, the full SHA of the commit built. They are staged in `$RUNNER_TEMP/artifact/` and that directory is the upload's `path`, so neither file sits under `target/release/` in the artifact.
6. **The pins:** every `uses:` is pinned by full commit SHA with its version in a comment, as `macos.yml`'s are: `actions/checkout@d23441a48e516b6c34aea4fa41551a30e30af803 # v6.1.0` (with `persist-credentials: false`), `actions/cache/restore@caa296126883cff596d87d8935842f9db880ef25 # v5.1.0`, `actions/upload-artifact@cf430e030ddbb5b0abf93d22962f4752f3646cd9 # v7.0.2`. The registry is restored with `macos.yml`'s `path:` block, `key:` and `restore-keys:` byte for byte (actions/cache's version hashes its paths, so a path that differs misses `macos.yml`'s entries); no `actions/cache/save`, and no cache of `target`.
7. **The file's header comment** cites `temper/docs/design/release.md, 5.1`, and no line of the file names a machine, tool or session of the lab (`notes/common.md`); `heavy cargo nextest run -p fabrig-boundary --test outward` passes after the commit, its count in the report.
8. **Status:** in `temper/docs/design/release.status.md`, the **Not yet** entry `5.1: .github/workflows/release-macos.yml.` is deleted, the paragraph rewrapped so that it reads `... 5: \`scripts/release.sh\`. 6: the walk on a fresh machine.`; no other line of any design file changes.

## Inputs

- `temper/docs/plan/temper-release-macos/notes/common.md`.
- `temper/docs/plan/temper-release-macos/inputs/request.md`, whole: items 1 to 6 are acceptance 1 to 6.
- `temper/docs/design/release.md`, 5.1 (*Apple Silicon's file*, written by this cut); read only.
- `.github/workflows/macos.yml`: the model for the header comment, `concurrency:`, the Rust step and the registry's restore. Read only.
- `temper/docs/design/client/browser.md`, 2.6, for what `TEMPER_BUNDLE=embed` builds (a nested cargo for `wasm32-unknown-unknown`, no other tool); read only.

## Owns and may change

Owns the new `.github/workflows/release-macos.yml`, and in `temper/docs/design/release.status.md` the one entry of acceptance 8.

Nothing else: not `macos.yml`, not `Cargo.toml`, not code, not `temper/docs/design/release.md` (what the work taught goes in the report).

Options, each decided:

- **Step names and comments** are yours, in the sketch's style (what is true, in the design's words); the sketch's commands, keys and values are not.
- **`actionlint`:** if it is on `PATH`, run it on the new file and quote its output in the report; a finding it raises against a line of acceptance 1 to 6 is fixed only where the fix keeps that item, else reported. If it is not on `PATH`, say so; install nothing, fetch nothing (`notes/common.md`).
- **A pin you doubt** (a SHA that looks wrong beside its comment): keep the spec's, and say why in the report; the cut checked each.

## Steps

1. Write `.github/workflows/release-macos.yml` from this sketch:

   ```yaml
   # Apple Silicon's file for the site (temper/docs/design/release.md, 5.1):
   # temper built for release at the dispatched commit, checked, and left as
   # the run's artifact. It writes nothing to the repository and uses no secret.
   name: release-macos

   on:
     workflow_dispatch:

   permissions:
     contents: read

   # One run at a time, whatever the ref; a new run waits, and a later run
   # takes the waiting one's place.
   concurrency:
     group: release-macos
     cancel-in-progress: false

   env:
     RUST_VERSION: "1.95"
     CARGO_TERM_COLOR: never

   jobs:
     release-macos:
       runs-on: macos-26
       timeout-minutes: 120
       defaults:
         run:
           shell: bash
       steps:
         - uses: actions/checkout@d23441a48e516b6c34aea4fa41551a30e30af803 # v6.1.0
           with:
             persist-credentials: false

         - name: Rust at the workspace's rust-version, with wasm32 for the browser's bundle
           run: |
             # rustup is on the image; no installer script runs here.
             if ! command -v rustup >/dev/null; then
                 echo "::error title=rustup::rustup is not on the macos-26 image"
                 exit 1
             fi
             rustup toolchain install "$RUST_VERSION" --profile minimal --target wasm32-unknown-unknown
             rustup default "$RUST_VERSION"
             rustc --version
             uname -a

         # The registry by macos.yml's keys, restored and never saved here;
         # the release's target is never cached.
         - name: Restore the registry
           uses: actions/cache/restore@caa296126883cff596d87d8935842f9db880ef25 # v5.1.0
           with:
             path: |
               ~/.cargo/registry/index
               ~/.cargo/registry/cache
               ~/.cargo/git/db
             key: cargo-home-macos-26-${{ hashFiles('Cargo.lock') }}
             restore-keys: cargo-home-macos-26-

         - name: The build, under the release profile as the workspace has it
           run: TEMPER_BUNDLE=embed cargo build --release --locked -p temper --bin temper

         - name: An arm64 Mach-O executable
           run: |
             kind=$(file -b target/release/temper)
             echo "$kind"
             if [[ $kind != *"Mach-O 64-bit"* || $kind != *executable* || $kind != *arm64* ]]; then
                 echo "::error title=file::target/release/temper is not an arm64 Mach-O executable: $kind"
                 exit 1
             fi

         - name: It prints its usage
           run: |
             status=0
             ./target/release/temper --help >"$RUNNER_TEMP/help.txt" || status=$?
             cat "$RUNNER_TEMP/help.txt"
             first=$(head -n 1 "$RUNNER_TEMP/help.txt")
             if [ "$status" -ne 0 ] || [[ $first != "usage: temper "* ]]; then
                 echo "::error title=temper --help::exit $status, first line: $first"
                 exit 1
             fi

         - name: The artifact's two files
           run: |
             mkdir "$RUNNER_TEMP/artifact"
             cp target/release/temper "$RUNNER_TEMP/artifact/temper"
             git rev-parse HEAD >"$RUNNER_TEMP/artifact/commit"
             cat "$RUNNER_TEMP/artifact/commit"

         - name: Upload temper-Darwin-arm64
           uses: actions/upload-artifact@cf430e030ddbb5b0abf93d22962f4752f3646cd9 # v7.0.2
           with:
             name: temper-Darwin-arm64
             path: ${{ runner.temp }}/artifact/
             if-no-files-found: error
             retention-days: 7
   ```

2. Read it through against acceptance 1 to 7, one item at a time, and compare the restore step with `macos.yml`'s by diffing the two `path:`, `key:` and `restore-keys:` blocks (a throwaway command, nothing written to the slot). Run `actionlint` if it is on `PATH` (Options).
3. Delete the status entry (acceptance 8).
4. `python3 -I -B temper/scripts/old_forms.py`; `python3 -I -B tools/citations/citations.py --base origin/main`. No `cargo fmt` or clippy: no Rust changes.
5. Commit (why: the site's Apple Silicon download needs a Mac, and GitHub's macOS runners are free on this public repository; the workflow builds it at a dispatched commit and hands it over as an artifact, writing nothing to the repository), then `heavy cargo nextest run -p fabrig-boundary --test outward` (it reads git's tracked files, so it runs after the commit), and push with `git push origin HEAD:refs/heads/<your branch>`.

## Report

fab's implementer report, with: acceptance 1 to 8 each with the file's line numbers that hold it; the restore blocks' comparison with `macos.yml`'s; `actionlint`'s output, or that it was not on `PATH`; the outward test's count; and anything in the sketch you changed, with why. No run of the workflow: its first dispatch is after landing.
