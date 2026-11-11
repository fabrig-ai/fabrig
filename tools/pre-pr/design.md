# The delivery check

Provisional, 2026-11-11. What fabrig's `.temper/pre-pr` runs when a run
delivers: the project's checks (smith's `smith/docs/design/domain/run.md`,
8.1; temper's `temper/docs/design/protocol/configuration/reference.md`), written
from `temper/docs/design/draft/delivery-checks.md`. Coding agents run no build
or test; this check runs what the delivery changed, on the agent's warm
checkout, before anything is pushed. A pull request into a goal's branch runs
no CI, so this is a change's only check before it lands there; the landing's
CI into main, the project's full gate (`.config/gate/README.md`), is the
guard. An agent can change what this check runs: it is not a security
boundary (smith's run.md, 8.1).

## 1. In one page

- **A base the worker pins.** The worker writes `refs/temper/landing`, the
  landing branch's tip as fetched at prepare, in each change producer's
  checkout and never moves it within the hold
  (`temper/docs/design/protocol/git.md`, 4.3;
  `temper/docs/design/domain/worker.md`, section 5). The check's base is
  the merge base of that reference and `HEAD`, and `MERGE_HEAD` in a merge
  (section 2). Without the reference it runs today's checks alone and says so.
- **What changed, and who reads it.** The packages owning a path the
  delivery changed against its base, and their direct dependents in the
  workspace (section 3).
- **Code, docs, or both:** code inputs select the format check, counted
  baselines, clippy and focused tests; Markdown anywhere and every file
  under a `docs/` directory select citations and design size limits.
  Declared documentation inputs to tests select both (sections 3 and 4).
- **Inherited reds don't refuse.** An exceeded baseline count, lint or
  test failure is compared with the base; citations compare their counted
  findings and design sizes there too. Inherited failures are named and
  pass (section 5). Only what this delivery broke refuses it.
- **A summary last,** within the host's kept tail: each failing check and
  test with its log's tail, then the inherited ones (section 6).
- **Fast on a warm checkout:** no `cargo clean`, no whole-workspace run,
  the base's builds only on a failure, and the passed-tree cache keyed by
  tree and base (sections 7 and 8).

## 2. The base

- **With the pin.** `git rev-parse --verify --quiet refs/temper/landing^{commit}`
  names a commit: the base is `git merge-base refs/temper/landing HEAD`,
  or, where `git rev-parse --verify --quiet MERGE_HEAD` names one (a merge
  in progress: smith's run.md, 8.3), `git merge-base refs/temper/landing
  HEAD MERGE_HEAD`, git's merge base of the reference with a hypothetical
  merge of `HEAD` and `MERGE_HEAD`. Where git gives several, the first it
  prints.
- **So the base is where the change starts on its landing branch:** the
  commit its branch started from, or the landing branch's tip when that
  is newer and already merged into the branch, or being merged.
- **Without it.** No reference, or no merge base: no base. The check runs
  section 4's checks 1 and 2 alone and prints one line saying why:
  `pre-pr: no base (no refs/temper/landing): the format check and the counted baselines alone`,
  or `(no merge base of refs/temper/landing and HEAD)`. A checkout outside
  temper has none until its owner makes one, as
  `git update-ref refs/temper/landing origin/main`.
- **Read, never written.** The check never moves `refs/temper/landing`.
  The worker writes it at prepare alone, and the next hold writes it
  afresh.

## 3. What it selects

- **The tree** is the working tree as git sees it, untracked files
  included and ignored ones not: written through a private copy of the
  index (`git add -A`, `git write-tree`), the real index untouched, as
  before.
- **Changed paths:** `git diff-tree -r --no-renames --name-only -z <base> <tree>`,
  deletions included.
- **Docs inputs:** every Markdown path (`.md`), anywhere, and every file
  with a `docs` directory component. Additions, edits and deletions all
  select check 5. No docs input means no full citation check.
- **Code inputs:** Rust (`.rs`); Cargo manifests and locks; rustfmt,
  clippy and Rust toolchain configuration, including under `docs/`;
  non-docs paths under a `.cargo/` directory, `.config/` and `tools/`;
  `.temper/pre-pr`; `skein/scripts/*.sh` (the boundary profile check's
  inputs); each tree's
  `scripts/old_forms.py` and `scripts/old-forms.txt`; and non-docs paths
  owned by workspace packages or matching a prefixed path reader below.
  Package ownership includes fixtures, generated goldens and build-script
  inputs of any format, without guessing from their extension. The
  tooling directories include the checks' programs, configuration and
  fixtures.
- **Declared readers:** globs in `.config/gate/read-by-tests.txt` and
  reader-bearing fixture entries' `paths` and generator `regenerate`
  paths in `.config/fixture-readers.toml` are code inputs too, even when
  they are Markdown or under `docs/`. These lists distinguish prose
  from data read by code. A package README or a drawing under `docs/`
  selects docs alone unless declared as an input. The fixture list
  classifies inputs here; it does not add package mappings to the
  selection below.
- **Select the sets before running checks:** code inputs select checks
  1–4, docs inputs select check 5, and both select both sets. With neither,
  no check runs. Plain docs require no cargo metadata. Each selected set
  gets one line naming its checks and a changed input that selected it:
  `pre-pr: docs only: citations and design size limits (docs input …)`,
  `pre-pr: code only: format, counted baselines, clippy, focused tests (code input …)`,
  or a line each starting `pre-pr: code and docs:`. With neither, print
  `pre-pr: no code or docs inputs changed`.
- **The workspace's packages:** `cargo metadata --offline --no-deps
  --format-version 1` at the tree. A member's directory is its
  manifest's parent; a changed path belongs to the member whose directory
  is its longest leading run of whole path components. A path under no
  member's directory belongs to none: the root's `Cargo.toml` and
  `Cargo.lock`, a tree's `clippy.toml`, `.config/`, `tools/` and the
  docs above a crate.
- **Changed packages (C):** the members owning a changed code input.
- **Direct dependents (D):** the members outside C one of whose
  `dependencies` entries, of any kind and any target, has a `path` and
  names a member of C by its package name.
- **Path readers (R):** each `.config/gate/always.txt` line of the form
  `PACKAGE PATH-PREFIX` whose prefix, as written, starts a changed code input
  adds its package, where it is a member. A line without a prefix runs
  for every change in the gate, not here. A line naming no member is
  skipped and printed.
- **The lints' set** is C; **the tests' set** is C ∪ D ∪ R. Each is
  ordered by package name and passed as one `-p` per package.
- **No package selected for code:** the lints' and tests' sets empty,
  code still runs checks 1 and 2, printing `pre-pr: no package changed`.
  Docs still run check 5 when selected. Without a base, section 2's
  fallback remains checks 1 and 2 alone: selection needs a base.

## 4. What it runs

In this order, each announced by a line `pre-pr: <check> …` before it
starts and ended by `pre-pr: <check>: passed|failed|inherited, <seconds> s`.
Every selected check runs, whatever an earlier one ended in, so that one
delivery hears every failure; the verdict is section 6's.

1. **The format check:** `tools/pre-pr/pre_pr.py`'s batched `rustfmt
   --check` over every workspace target, as before
   (`temper/docs/design/protocol/configuration/reference.md`). No build.
2. **The counted baselines:** `(cd TREE && python3 -I -B scripts/old_forms.py)`
   for each of skein, smith, jig, temper and hearth, against the private
   index, as before.
3. **The lints:** `cargo clippy --all-targets --message-format=json -p … -- -D warnings`
   over the lints' set. The check prints each compiler message's
   `rendered` text as it arrives, and keeps each error's key (section 5).
4. **The focused tests:** `cargo nextest run --profile default
   --no-fail-fast --no-tests=warn -p …` over the tests' set, with
   `--tool-config-file temper-pre-pr:<dir>/nextest.toml`, an overlay
   holding only `[profile.default.junit] path = "<dir>/junit.xml"`, `<dir>`
   a private temporary directory removed at the end. The repository's
   `.config/nextest.toml` is not changed. nextest's exit decides: 0
   passed; 100, test failures, each a JUnit `testcase` with a `failure` or
   an `error`, named by its suite (the binary id) and its name; 101, a
   build failure; any other, a failure of the check itself.

5. **The docs:** `python3 -I -B tools/citations/citations.py --base <base>
   --head <tree>` against section 3's private snapshot, including staged,
   unstaged and untracked documentation. The checker also enforces the
   design size limit for `tools/citations/sized.txt`'s trees (`DESIGN.md`,
   5). A docs-only delivery runs no formatter, counted baseline, clippy
   or test. A code-only delivery runs no full citation checker; smith's
   counted `comment:dead_citation` baseline still checks Markdown path
   existence in Rust comments as part of check 2, as before
   (`tools/old_forms/README.md`).

**The environment** is the check's, with `TEMPER_BUNDLE=none` set where it
is unset (`AGENTS.md`, 5). Builds and tests do not inherit the snapshot's
private index; their scratch repositories keep their own indexes. No
`cargo clean`, no `--workspace`, no network
fetch beyond what cargo's own configuration does.

## 5. Inherited failures

Only selected checks contribute failures. A failure can be inherited
only with a base. The format check runs within the code set: its failures are never
inherited. A red base refuses nothing the delivery did not cause.

- **An exceeded baseline count** is keyed by tree and form, with its
  current count and recorded limit. The base runs check 2 only for the
  trees with exceeded counts, in the same kept worktree used for lints
  and tests. A count is inherited when the base exceeds that form too,
  the head's count is no greater, and its excess over the limit is no
  greater. A rising count, a newly exceeded form or a tighter limit that
  increases the excess refuses. Each count is judged separately and
  named as `old-forms <tree> <form>`; a malformed baseline or a tool error
  cannot be inherited from an exit code alone.
- **A broken citation** uses the citation checker's counted keys per
  source file and target, with renames mapped; line movement does not
  make an old finding new. Check 5's `--base` comparison already refuses
  only new occurrences. Its inherited total is reported as
  `inherited: citations (N broken at base)`, even when new findings also
  refuse. Design documents already over the size limit pass when they
  do not grow; crossing the limit or growing while over it refuses.
  Neither comparison needs a cargo build or the kept base worktree.

- **The base's worktree** is `<git dir>/temper-pre-pr/base`, a detached
  worktree (`git worktree add --detach --force`), made at the first need
  and kept; each later need moves it with `git checkout --detach --force
  <base>` then `git clean -ffdxq`. Its builds use the head's target
  directory (`cargo metadata`'s `target_directory`, through
  `CARGO_TARGET_DIR`): what both sides depend on is built once, and cargo
  can reuse the dependencies. Workspace members have artifact keys
  relative to their workspace root, so the base can overwrite the head's
  artifacts, and the base can likewise reuse the head's artifacts.
  After the base's checkout and clean, before any base build, refresh its
  surviving changed files within that checkout to a timestamp taken
  after the head build. After every base judgment, including a failed
  judgment, a `finally` block refreshes the head's copies to a timestamp
  taken after the base build. The paths are the NUL-delimited diff from
  the base to section 3's private snapshot, so committed, staged,
  unstaged and new files are covered. Deleted files stay absent on each
  side; cargo sees missing inputs as stale. Build
  scripts see refreshed tracked inputs and regenerate their outputs.
  The caller's index and file contents stay as they are. The next head
  build must rebuild each package whose sources differ; the base judge
  must likewise build the base's code. Cargo rebuilds their consumers,
  while unchanged
  dependencies stay warm. No target scan or clean is needed.
- **A lint error** is keyed by its package's name, its lint code (empty
  for a compiler error without one), its primary span's file, relative to
  its worktree's root, and its message; never by line or column. The base
  runs check 3 over the packages of C that have an error and are members
  at the base. A head error whose key the base's run also gives is
  inherited.
- **A failing test** is keyed by its binary id and its name. The base runs
  check 4 over the packages owning them that are members at the base,
  with `-E` the disjunction of `(binary_id(=<id>) & test(=<name>))` for
  each, and its own JUnit file. A head failure the base's run also fails
  is inherited; one the base passes, or doesn't have, is the delivery's.
- **A build failure** (nextest's 101): the base runs check 4's build alone
  (`--no-run`) over the same packages that are members at the base. A
  base that fails to build too makes the head's build failure inherited,
  and no test of that set ran: the summary says so. Otherwise it is the
  delivery's.
- **A base run that cannot run** (the worktree cannot be made, cargo
  exits for a reason of its own): failures needing that worktree stay the
  delivery's, and the summary says why the base could not judge them.
  The citation check's independent comparison still stands.
- **Inherited never refuses:** each is named in the summary, with the
  base, and the delivery passes on them.

## 6. Its output

- **As it goes:** section 3's selection lines, each selected check's
  announcement, then what its tools print,
  so a stopped check's tail shows where it was (smith's run.md, 8.1).
- **The summary, last,** at most 8,192 bytes, which smith's kept tail
  holds (smith's `protocol/limits.md`, `shell_tail`):
  - `pre-pr: passed` or `pre-pr: refused`, with the base (`base <id>,
    merge base of refs/temper/landing and HEAD`) or that there was none;
  - the sets: `lints: <packages>`, `tests: <packages>`;
  - for each failure this delivery made: its check (`format`, `old-forms
    <tree>`, `clippy <package>`, `test <binary id> <name>`, `build`,
    `check <name>` for a check that failed of itself), then the last lines
    of its log: a test's JUnit output, an error's rendered text, a tool's
    output; each at most 1,536 bytes, cut at a line's start and marked
    `[… N bytes cut]`;
  - then each inherited failure, one line each: `inherited: test <binary
    id> <name>` or `inherited: clippy <package> <code> <file>`, and the
    base's build failure as `inherited: build`, exceeded counts as
    `inherited: old-forms <tree> <form>`, and the inherited citation total;
  - past 8,192 bytes, the failures it has not room for are counted in
    one line, `[… N more failures]`, the first kept whole.
- **Its exit:** 0 when nothing this delivery made failed, inherited
  failures included; 1 otherwise.

## 7. The passed-tree cache

`<git dir>/pre-pr-passed` holds `<tree> <base>` (`<tree> -` without a base)
once every check passed or was inherited. A check of the same tree on the
same base prints `pre-pr: passed before (tree <tree>, base <base>)` and
exits 0 at once: within a hold the base does not move, so a `finish`
repeats its `deliver`'s pass. A failure, an interruption or a timeout never
writes it. An earlier record of another shape matches nothing.

## 8. Time

- No `cargo clean` and no whole-workspace run: the sets are what the
  delivery changed and their direct dependents, on the checkout's warm
  target directory.
- The base's worktree and builds are made only after a failure, and kept
  for the next.
- What a check took is the run's: smith tells its host when checks start,
  with their deadline, and when they end (run.md, 8.1), and the check
  prints each check's seconds. The checks' deadline is the deployment's
  `check_timeout`, which the owner sets separately.

## 9. Testing

The boundary world `tests/boundary/tests/pre_pr.rs` (package
`fabrig-boundary`, in the focused suite) runs `.temper/pre-pr` in a scratch
repository against fake `cargo` and `rustfmt` programs that record what they
were asked and answer from a ledger: metadata with members, their
directories and dependencies; clippy's JSON messages; nextest's exit and a
JUnit file at the overlay's path, by worktree (head or base). It reads no
real build. Its tables: the base (no pin, a pin, a merge in progress, no
merge base); the selection (a package, its dependents, a path reader, a
path in no package, a deletion); inherited and new failures of each kind;
the cache by tree and base; the summary's bound.

`tools/pre-pr/test_pre_pr.py` first exercises code-only, docs-only, both
and neither in scratch repositories. It runs the citation checker against
private snapshots: new and inherited broken citations, a deleted cited
document, and a design size violation. It checks that docs-only changes
skip every code check even on a red baseline; unchanged red counts pass
while a rising count or tighter limit refuses. Non-Markdown fixtures,
declared docs inputs and check configuration select code.

It also builds a dependency and its consumer with real offline cargo in a
scratch workspace. A kept base checkout predates
the head's shared artifacts: the base judge must execute the base's code,
then the next head build must execute the head's code. This holds for
source changes and build-script inputs generating Rust in `OUT_DIR`,
including an input deleted at head. The boundary world also models the
shared artifact's mtime in both directions for lint, test and build
judgments and a base tool error, with new paths containing
whitespace and a deleted source.

## 10. Later

- **Inputs read by tests** (`.config/gate/read-by-tests.txt`) outside
  workspace packages and prefixed path readers still map to no package:
  they select the code set, but no lint or test package. The landing's CI
  runs their readers.
- **What no package owns** (the root manifest and lock, a tree's
  `clippy.toml`, `.config/nextest.toml`) selects the code set but no lint
  or test package; the landing's CI checks its consumers.
- **The gate's other checks** (wasm32 clippy, the fuzzy suite, the lint
  profiles) stay the landing's.
- **Repairs, resolutions and merge jobs** are given no landing branch
  (`temper/docs/design/domain/worker.md`, 4.1), so their checks run
  without a base.
