+++
id = "version-crate"
part = "engine-version"
kind = "feature"
needs = []
files = ["temper/crates/temper-version/**", "temper/docs/design/release.status.md"]
effort = "high"
minutes = 60
+++

## Goal

A new crate, `temper-version`, holds what a binary's `--version` prints:
the workspace's version and the commit the build was made from, read
from git by its build script (`temper/docs/design/release.md`, section
3). Only binaries' packages will link it, so a commit relinks them and
nothing they link; `temper-engine` is the first (task `engine-binary`).

Acceptance:

- `temper/crates/temper-version`: its library `#![no_std]` with `alloc`,
  `#![forbid(unsafe_code)]`, on the `binary` lint profile with the
  `ordinary` `clippy.toml` copy (as `temper-client-bundle` has them), no
  dependency; `build = "build/main.rs"`, ordinary `std` Rust, no build
  dependency. Its surface is `notes/contracts.md`, 2's: `VERSION`
  (`env!("CARGO_PKG_VERSION")`), `COMMIT` (set by the script through
  `cargo:rustc-env=TEMPER_COMMIT=…`) and `line(name)`, each with a doc
  comment citing `release.md`, 3.
- The script, its logic in `build/commit.rs` as pure functions over what
  git answered, `build/main.rs` running git and printing cargo's lines:
  - COMMIT is `git rev-parse --short=12 HEAD`'s twelve hex digits, then
    `-dirty` when `git status --porcelain --untracked-files=no` prints
    anything, and `unknown` when git is missing, fails, or answers
    anything but twelve lowercase hex digits; git runs in the package's
    directory (`CARGO_MANIFEST_DIR`), stdin null, stderr discarded.
  - It prints `cargo:rerun-if-changed=` for `HEAD` and `index` under
    `git rev-parse --git-dir`, and for `packed-refs` and the ref `HEAD`
    names (`git symbolic-ref -q HEAD`, a detached `HEAD` naming none)
    under `git rev-parse --git-common-dir`, each made absolute against
    the package's directory where git answers a relative path, and
    `cargo:rerun-if-changed=build` for its own sources; outside git, the
    script's own sources alone. A worktree's `.git` is a file, so these
    are read from git, never from `.git/` by name.
  - A failure of git is `unknown`, never a failed build.
- `tests/commit.rs`, one test binary, includes `build/commit.rs` by
  `#[path]` as `temper-client-bundle/tests/steps.rs` does, and holds one
  table test of the commit's word: `(git's head answer, status answer) →
  COMMIT` rows: a clean head, a dirty head, no answer (`unknown`), an
  answer of the wrong length or with an uppercase or non-hex byte
  (`unknown`), a trailing newline trimmed; and one table test of the
  watched paths: relative and absolute git dirs, a worktree's distinct
  git dir and common dir, a detached `HEAD`. A third test reads the
  library: `line("temper-engine")` is `temper-engine {VERSION} ({COMMIT})`
  and `COMMIT` is twelve hex digits, `-dirty`-suffixed or not, or
  `unknown`. Break the `-dirty` rule once, see its row fail, restore.
- The status entry closes: in `temper/docs/design/release.status.md`, the
  **Not yet** 3 entry "`temper-version` doesn't exist: …", its lines
  alone. This departs from the implementer's text, which closes
  `docs/design/` to you: the entry is this task's.

## Inputs

- `temper/docs/design/release.md`, section 3.
- `temper/docs/design/protocol/README.md`, section 5, the row
  `crates/temper-version`; 4.1, `--version`.
- `temper/crates/temper-client-bundle` (`Cargo.toml`, `clippy.toml`,
  `build/`, `tests/steps.rs`): the pattern of a build script and its
  tested logic.
- `tools/lints/design.md`, sections 3 and 4.
- `temper/docs/plan/ui-engine-split/notes/contracts.md`, 2.
- `temper/docs/plan/ui-engine-split/notes/common.md`.

## Owns and may change

- Owns the files in `files`.
- The root `Cargo.toml`: the member line `"temper/crates/temper-version",`
  right after `"temper/crates/temper-client-bundle",`, and the dependency
  line `temper-version = { path = "temper/crates/temper-version" }` right
  after `temper-client-domain = { … }`; nothing else there. `Cargo.lock`
  as cargo writes it.
- Pre-approved: `std::env::var` is a disallowed method under the
  `ordinary` variant: read with `var_os`, or a scoped
  `#[expect(clippy::disallowed_methods, reason = "…")]` as the bundle's
  script does. If the gate's fail-stop check
  (`tests/boundary/tests/fail_stop.rs`) reads build scripts' `main`s as
  binaries' and asks for `skein_fail_stop::install()`, check how
  `temper-client-bundle`'s script passes it and do the same, naming it.
- Not yours: any package depending on it (none yet); `release.md` itself.

## Steps

1. The manifest, `python3 -I -B tools/lints/lints.py --set binary
   temper/crates/temper-version`, and the `ordinary` `clippy.toml` copy.
2. `build/commit.rs` and its tests first; see them fail, then the
   functions; `heavy cargo nextest run -p temper-version`.
3. `build/main.rs` and the library; the library's test.
4. The status entry.
5. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `python3 -I
   -B tools/lints/lints.py`; `heavy cargo clippy -p temper-version
   --all-targets -- -D warnings`; `heavy cargo nextest run -p
   temper-version -p fabrig-boundary`.
6. `python3 -I -B tools/citations/citations.py --base origin/main`.
   Commit and push.

## Report

As the implementer's text says, with the COMMIT your own build printed
and the failure the broken `-dirty` rule showed.
