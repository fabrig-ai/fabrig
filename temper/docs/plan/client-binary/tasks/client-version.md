+++
id = "client-version"
part = "version"
kind = "feature"
needs = []
files = [
  "temper/crates/temper-client/**",
  "Cargo.toml",
  "Cargo.lock",
]
effort = "high"
minutes = 75
+++

# The `temper-client` package, and `temper-client --version`

## Goal

Create the deployment's client's package, `temper-client`, with its
`temper-client` binary target, and give it `--version`: one line, `temper-client
VERSION (COMMIT)`, COMMIT set by the package's own build script
(`temper/docs/design/protocol/README.md`, section 4, *The deployment's client*;
`temper/docs/design/release.md`, section 3). Its table, help and commands are
task `client-table`'s; this task lays the package they go in.

Acceptance:

1. **The package** as `notes/contracts.md`, section 9: `Cargo.toml` (lib
   `temper_client`, bin `temper-client` with `test = false`, `build =
   "build/main.rs"`, test `suite`), profile `binary`, `clippy.toml`, the root
   `Cargo.toml`'s member line (`notes/common.md`, *A new crate*). Description:
   "The deployment's client: the TUI and temper ctl (protocol/README.md, section
   4)". Dependencies now: `skein-shell`, `skein-fail-stop`; no build
   dependencies.
2. **The build script** `build/main.rs`, with `build/commit.rs`' `pub fn
   word(head: Option<&str>, dirty: bool) -> String`: `head` the full hex commit
   git names, `None` when git or the repository is missing or fails; the word is
   its first twelve characters, then `-dirty` when `dirty`, or `unknown` for
   `None` and for a head that isn't at least twelve lowercase hex digits. The
   script runs git in `CARGO_MANIFEST_DIR` (`std::process::Command`, as
   `temper/crates/temper-client-bundle/build/main.rs` runs programs), never
   writing the repository: `git rev-parse HEAD`; `git --no-optional-locks status
   --porcelain --untracked-files=no`, dirty when it prints anything; and prints
   `cargo:rustc-env=TEMPER_CLIENT_COMMIT=WORD`. It reruns when `HEAD` or the
   index moves: `cargo:rerun-if-changed=` for the git directory's `HEAD` and
   `index`, the ref `HEAD` names under the common git directory, and its
   `packed-refs` (`git rev-parse --path-format=absolute --git-dir
   --git-common-dir`, `git symbolic-ref -q HEAD`), each only where the file
   exists, and for `build/`.
3. **`src/lib.rs`:** `pub const NAME: &str = "temper-client";` and `pub fn
   version() -> String`, `"{NAME} {CARGO_PKG_VERSION} ({TEMPER_CLIENT_COMMIT})\n"`
   (contracts, section 9), its doc citing `release.md`, section 3. 
4. **`src/main.rs`:** `skein_fail_stop::install()`; the arguments
   (`skein_shell::arguments()`) exactly `--version` print `version()` on stdout,
   exit 0. Anything else, until `client-table` gives the table, prints on stderr
   `temper-client: no command WORDS; temper-client --help lists them` (WORDS the
   arguments joined by a space; with none, `temper-client: no command;
   temper-client --help lists them`), exit 1.
5. **Tests,** each seen failing first (`notes/common.md`):
   - `src/tests.rs`: `the_commit_word_is_twelve_hex_digits_dirty_or_unknown`, a
     table over `word` (a 40-digit head clean and dirty; `None` clean and dirty;
     an 11-digit head; a head with a non-hex digit), the script's file included
     by `#[path = "../build/commit.rs"] mod commit;`;
     `the_version_line_names_the_binary_its_version_and_commit`: `version()` is
     `temper-client`, the package's version, and a word `word` can make, in
     brackets, then a line feed.
   - `tests/suite/main.rs` with `mod version;`: `the_binary_prints_its_version_and_refuses_the_rest`,
     a table running `env!("CARGO_BIN_EXE_temper-client")` (`std::process::Command`,
     stdin null): `--version` exits 0 printing `temper_client::version()` alone,
     stderr empty; `--version extra`, `nope` and no argument exit 1, stdout empty,
     stderr as acceptance 4 says.

## Inputs

- `temper/docs/plan/client-binary/notes/common.md`; `notes/contracts.md`,
  section 9 (its first two bullets, `NAME` and `version`).
- Design: `temper/docs/design/protocol/README.md`, section 4 (*The deployment's
  client*, *`temper-client --version`*) and section 5 (the row
  `crates/temper-client`); `temper/docs/design/release.md`, section 3.
- The code: `temper/crates/temper/src/main.rs` (how the box's binary starts:
  `skein_fail_stop::install`, `skein_shell::arguments`, `ExitCode`);
  `temper/crates/temper-client-bundle/Cargo.toml` and `build/main.rs` (a build
  script's lints, `env::var_os`, running a program); `temper/crates/temper/tests/suite/startup.rs`
  (a suite test running a built binary).

## Owns and may change

Owns `temper/crates/temper-client/**` (new) and the root `Cargo.toml`'s member
line; `Cargo.lock` follows. Nothing in another package.

Options, each decided:

- **Git's answers:** a failed `git` (absent, not a repository, non-zero exit)
  reads as `None` and prints no `rerun-if-changed` for git's files; the build
  never fails on git.
- **`--path-format=absolute`** needs git 2.31 or later; if the builder's git
  refuses it, resolve the relative paths against `CARGO_MANIFEST_DIR` instead and
  say so.
- **Lints in the build script:** `env::var_os` for cargo's variables; a
  `disallowed_*` finding there takes a scoped `#[expect]` with its reason, as the
  bundle's script does.

## Steps

1. The package, the script and `commit.rs`, the library and `main`, the tests
   failing first. `heavy cargo nextest run -p temper-client`.
2. Check the rerun by hand once: a commit or an index change rebuilds the script
   (`heavy cargo build -p temper-client -v` shows it run), an unchanged tree
   doesn't; say what you saw.
3. Once: `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`;
   `python3 -I -B tools/lints/lints.py`; `heavy cargo clippy -p temper-client
   --all-targets -- -D warnings`; `heavy cargo nextest run -p temper-client`.
4. Commit ("temper-client: the deployment's client's package, and --version") and
   push.

## Report

`Done:`, `Failed:` or `Question:` first, then the package's files, the version
line your build printed, what the rerun check showed, the runs and their counts,
the rule each new test bit on, anything left.
