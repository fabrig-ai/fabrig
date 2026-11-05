+++
id = "fail-stop-test"
part = "fail-stop"
kind = "feature"
needs = ["hook-binaries"]
files = [
  "tests/boundary/tests/fail_stop.rs",
  "tests/boundary/Cargo.toml",
  ".config/gate/README.md",
  "hearth/scripts/fail_stop.py",
  "Cargo.lock",
]
effort = "high"
minutes = 45
+++

# The fail-stop check, a boundary test over the whole workspace

## Goal

`hearth/scripts/fail_stop.py` checks hearth's binaries and no gate runs it. Replace it with a test
in `fabrig-boundary`, over every binary target of the workspace, keeping the script's rule: in each
`fn main`, only `use` and `const` items and comments may come before
`skein_fail_stop::install();`. The script goes.

Acceptance:

- `tests/boundary/tests/fail_stop.rs` exists, in the manner of `direction.rs` and `lints.rs`, and
  passes over the workspace's 30 binary targets.
- A failure names the package, the binary and the path. A source it can't read, and metadata it
  doesn't read, is a failure, never a skip. A test shows the check catches a breach, and the
  workspace test fails if the check read no binary of a tree.
- `.config/gate/README.md` lists the check.
- `hearth/scripts/fail_stop.py` is gone. The cut already removed the one other thing naming it, the
  Later entry in `hearth/docs/design/domain/testing.md`; nothing in the repository names the script
  now but the plan's own files.

## Inputs

- `skein/docs/foundation/programming-model.md`, 2: "Panics abort".
- `hearth/scripts/fail_stop.py`: the rule, its two expressions and the sweep.
- `tests/boundary/tests/direction.rs`: the manner: `read` over a closure, `breaches`, a fixture, a
  test that a line it doesn't read fails it. `tests/boundary/tests/lints.rs`: running a tool from
  the workspace root.
- `.config/gate/README.md`, `AGENTS.md`, 2.
- `hearth/docs/plan/fail-stop-names/notes/common.md`.

## Owns and may change

- Owns `tests/boundary/tests/fail_stop.rs` (new).
- May change: `tests/boundary/Cargo.toml`: one line, `serde_json.workspace = true` under
  `[dev-dependencies]`. `Cargo.lock`: `fabrig-boundary`'s list alone. `.config/gate/README.md`: the
  new item alone. `hearth/scripts/fail_stop.py`: `git rm`, this request's one change under `hearth/`.
- This task stays out of a tree on purpose: `tests/boundary` is the repository's, and the script's
  removal is what replaces it.
- **Binaries come from `cargo metadata`, not from the manifests.** The workspace has `[[bin]]` paths
  outside `src/` and shared by packages, and `src/bin` targets cargo finds by itself; reading
  manifests would have to model that. `--no-deps` resolves nothing, needs no network and took 0.3 s.
- If `cargo` can't be started by the test under `heavy`: stop and ask; don't fall back on reading
  manifests. If a binary fails the check once `hook-binaries` is in: that's a finding, say which;
  it isn't to be exempted.
- `skein/docs/foundation/programming-model.md`, 10.1's Fail-stop row ("build profile; review") is
  left as it is: the request doesn't name it. The cut's report carries that.

## Steps

1. The test, with no `regex` dependency: a small hand scanner for the script's two expressions.

   ```rust
   //! Every binary's `main` installs `skein_fail_stop`'s hook before anything else
   //! (skein/docs/foundation/programming-model.md, 2): the dev profile unwinds, so a panic aborts
   //! a process only once the hook is in. The check asks `cargo metadata` for the workspace's
   //! bin targets, wherever cargo finds them, and reads each one's source: in each `fn main()`,
   //! only white space, `//` comments and `use` and `const` items may come before
   //! `skein_fail_stop::install();`. A source with no `fn main()` breaches it. A source it cannot
   //! read and metadata it does not read are failures, never skips.

   /// A bin target: its package, its name and its source's path from the workspace's root.
   #[derive(Debug, PartialEq, Eq)]
   struct Binary {
       package: String,
       name: String,
       path: String,
   }

   /// The bin targets in `cargo metadata`'s output, in its order: a `path` from its
   /// `workspace_root`; a package or target of another shape is an error.
   fn binaries(metadata: &str) -> Result<Vec<Binary>, String>;

   /// Whether `source` has a `fn main()` and each one installs the hook first.
   fn installs_first(source: &str) -> bool;

   /// Each binary that breaches the rule or cannot be read: `PACKAGE BINARY: PATH: why`.
   fn breaches<F>(binaries: &[Binary], read: F) -> Vec<String>
   where
       F: Fn(&Binary) -> Result<String, String>;
   ```

   The scanner keeps the script's expressions exactly: a `main` is `\bfn main\(\)[^{]*\{`, and what
   may follow it is `(?:\s+|//[^\n]*|(?:use|const)\s[^;]*;)*`, then the call. So a `/* */` comment
   isn't admitted, and a `fn main()` inside a comment counts as a main; say so in the report if
   that bites.

   A target is a binary when its `kind` holds `bin`, as the script has it.

2. The tests, sketched:

   - `every_binary_installs_the_hook_first`: runs `cargo metadata --no-deps --format-version 1
     --locked` from `skein_scratch::paths::workspace()`, asserts its status with its stderr, and
     asserts that each of `skein/`, `smith/`, `jig/`, `temper/` and `hearth/` gave at least one
     binary ("the check reads {tree}'s binaries"), then that `breaches` is empty, joining them on
     new lines under one line stating the rule and citing the section.
   - `a_main_that_does_anything_first_is_a_breach`, over literal sources: no `main`; an empty
     `main`; a `let` before the call; the call only in another function; two `main`s of which the
     second lacks it; a `/* */` comment first.
   - `only_use_const_and_comments_may_come_first`: `-> ExitCode`; a `use` over several lines; a
     `const`; `//` comments; two `main`s both with it, as
     `skein/tests/conformance/real/src/bin/signal_sender_fixture.rs` has.
   - `a_breach_names_the_package_the_binary_and_the_path`: exact strings from `breaches`, over a
     closure.
   - `a_source_the_check_cannot_read_fails_it`: a `read` returning `Err` gives a breach.
   - `metadata_the_check_does_not_read_fails_it`: not JSON; no `packages`; a package with no
     `targets`; a target with no `kind`; a `src_path` outside `workspace_root`.

3. `.config/gate/README.md`, after item 8:

   ```markdown
   9. **The fail-stop check:** `cargo nextest run -p fabrig-boundary`, included in
      `default`. Rule: `skein/docs/foundation/programming-model.md`, 2, and
      `tests/boundary/tests/fail_stop.rs`'s module doc: every binary's `main` installs
      `skein_fail_stop`'s hook first.
   ```

4. `git rm hearth/scripts/fail_stop.py`.
5. Bite, per the implementer's text: remove the call from one binary's `main` (hearth's will do),
   run `heavy cargo nextest run -p fabrig-boundary --test fail_stop -E
   'test(=every_binary_installs_the_hook_first)'`, see it name that package, binary and path;
   restore.
6. `cargo fmt`; `python3 -I -B scripts/old_forms.py` in `hearth/`; `heavy cargo clippy -p
   fabrig-boundary --all-targets -- -D warnings`; `heavy cargo nextest run -p fabrig-boundary`; the
   common note's checks. Commit and push.

## Report

The test's count, the bite and what it printed, the time the check takes, and any scanner choice
that matters.
