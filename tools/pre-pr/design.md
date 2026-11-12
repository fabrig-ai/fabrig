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
  `temper/docs/design/domain/worker/checkouts.md`). The check's base is
  the merge base of that reference and `HEAD`, and `MERGE_HEAD` in a merge
  (`design.md`, section 2). Without the reference it runs today's checks alone and says so.
- **What changed, and who reads it.** The packages owning a path the
  delivery changed against its base, and their direct dependents in the
  workspace (section 3).
- **Code, docs, or both:** code inputs select the format check, counted
  baselines, clippy and focused tests; Markdown anywhere and every file
  under a `docs/` directory select citations and design size limits.
  Declared documentation inputs to tests select both (sections 3 and 4).
- **A binary is a process.** The focused tests run through
  `tools/tiers/one-process default`: each test binary's selected tests in
  one process, one at a time, binaries in parallel, nextest only selecting
  and building (section 4). A starved run is a warning.
- **One hold for the check.** With `TIERS_HOLD` set, the check holds its
  build directory from before its first build to its verdict, so the
  executables one-process built are still there when it runs them
  (section 4; `tools/tiers/design.md`, 3).
- **Known reds, and inherited counts.** Nothing is built or tested at the
  base. A lint or test failure is inherited only when the checked tree's
  `.temper/known-reds.md` lists it; a build failure is always the
  delivery's. An exceeded baseline count is compared with the base's, with
  no build; citations compare their counted findings and design sizes
  there too. Inherited failures are named and pass (section 5); every
  other failure refuses.
- **A summary last,** within the host's kept tail: each failing check and
  test with its log's tail, then the inherited ones (section 6).
- **Fast on a warm checkout:** no `cargo clean`, no whole-workspace run,
  no build at the base, and the passed-tree cache keyed by tree and base
  (sections 7 and 8).

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
- **A repair's pin is the head it repairs.** A repair, a resolution and a
  merge job name no landing branch, and the worker pins the head their
  delivery replaces instead (`temper/docs/design/domain/worker/checkouts.md`
  ): the base is that head, so a repair of a goal's landing into main
  checks what its own edits changed, not the goal's packages. A resolution's
  base is its branch's head before the merge, so its check selects what the
  merge and its edits change on the branch.
- **An unchecked save is stepped past.** Where the base is temper's
  unchecked save, a commit whose subject is `Unchecked: work in progress,
  saved before a question` (`temper/docs/design/domain/agent/README.md`,
  section 5), the base is its first parent instead, again until it is not
  one, or it has none: a repair resumed after it saved checks the saved
  edits with its new ones. The summary's base line names the commit
  stepped to.
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
4. **The focused tests:** `tools/tiers/one-process --out <dir>/tests
   default -p … --no-tests=warn` over the tests' set, from the tree's root,
   `<dir>` a private temporary directory removed at the end. Each test
   binary's selected tests run in one process, one at a time, and binaries
   in parallel; nextest is the front end, selecting with
   `.config/nextest.toml`'s `default` profile and building
   (`tools/tiers/design.md`, 11;
   `skein/docs/foundation/testing-strategy.md`, 8, *A binary is a
   process*). The check passes no budgets and no configuration of its own:
   the repository's `.config/nextest.toml` is read as it stands, and the
   profile's `global-timeout` is the run's only bound. The run's exit
   decides:
   - **0:** passed, a starved run included (below);
   - **100:** test failures, each a `testcase` of the run directory's
     `junit.xml` with a `failure` or an `error`, named by its suite (the
     binary id) and its name, its log the case's text, but a test the
     directory's `results.json` gives the `outcome` `cancelled`: stopped at
     the global timeout, it is starved, not failed
     (`tools/tiers/design.md`, 11.6). A failure whose `results.json` test
     has `alone` `passed` ends its log with one-process's label as a line,
     `may depend on an earlier test: A, B`, the tests its `before` names,
     joined by `, `, so that a test which cannot yet share its binary's
     process is told from one that fails alone. A 100 that leaves no failure is a failure of the check
     itself;
   - **2, its output ending with the line `one-process: nextest listing
     exited with 101`:** a build failure. The listing builds the tests as
     nextest's run would (`tools/tiers/design.md`, 11.2), and nextest's
     101 is a failed build;
   - **any other:** a failure of the check itself, a 2 that names no
     failed build included (a refusal or an invalid run:
     `tools/tiers/design.md`, 11.1 and 11.7).

   **Other packages' binaries** that the tests run are built first: for each
   package of the tests' set that `tools/tiers/packages.toml`'s
   `[test-binaries]` lists, the check runs `cargo build -p PACKAGE --bin BINARY`
   for each binary listed, in order, since nextest builds only the selected
   packages' own. A failed build is a build failure, and no test runs.
   `smith`'s end-to-end tests run `smith-fake-claude`'s `fake-claude`
   (`smith/docs/design/testing.md`, 2.3).

   **A starved run is a warning** (`skein/docs/foundation/testing-strategy.md`,
   8.1). Where the directory's `run.json` has `starved` true, or
   `results.json` gives any test the `outcome` `starved` or `cancelled`,
   the check prints, after the run, `pre-pr: tests starved: <N> starved or
   cancelled, <M> unobserved, a warning
   (skein/docs/foundation/testing-strategy.md, 8.1)`, N those tests and M
   the entries of `results.json`'s `unobserved`, and the summary repeats
   the line (section 6). It refuses nothing; a failure beside it still
   refuses. A `junit.xml`, `results.json` or `run.json` the check needs and
   cannot read is a failure of the check itself.

5. **The docs:** `python3 -I -B tools/citations/citations.py --base <base>
   --head <tree>` against section 3's private snapshot, including staged,
   unstaged and untracked documentation. The checker also enforces the
   design size limit for `tools/citations/sized.txt`'s trees (`DESIGN.md`,
   5). A docs-only delivery runs no formatter, counted baseline, clippy
   or test. A code-only delivery runs no full citation checker; smith's
   counted `comment:dead_citation` baseline still checks Markdown path
   existence in Rust comments as part of check 2, as before
   (`tools/old_forms/README.md`).

**The environment** is the check's, with `TEMPER_INTERNAL_BUNDLE=none` set where it
is unset (`AGENTS.md`, 5). Builds and tests do not inherit the snapshot's
private index; their scratch repositories keep their own indexes. No
`cargo clean`, no `--workspace`, no network
fetch beyond what cargo's own configuration does.

**The hold.** With `TIERS_HOLD` set, the check takes the build directory's
hold as `tools/tiers/design.md`, section 3 gives it, once: after the
passed-tree cache has not answered (section 7), before its first cargo
command (the first `cargo metadata`), and kept to its verdict, the cache's
record written, whatever the checks ended in. Every check runs under it
with the setting removed, so one-process, check 4, takes none of its own,
and the check's builds and the executables they built are one hold, not one
claim per cargo command: an environment that lends a build directory only
while a cargo command runs could otherwise reclaim it after one-process's
listing has built the tests and before their processes start. A cache hit
takes none. A hold that fails prints its warning, `pre-pr: hold: <why>;
going on unheld`, and refuses nothing; the summary does not repeat it, since
it is the deployment's, not the delivery's. Killed, the check releases the
hold by its death (`tools/tiers/design.md`, 3).

## 5. Inherited failures

Only selected checks contribute failures. The format check runs within
the code set: its failures are never inherited. Nothing is built or
tested at the base: a lint or test failure goes back to the delivery
unless the known reds list it, and a build failure always does, so a
failing delivery costs no base build (section 8). The comparisons that
need no build stand as they are: a red base's exceeded counts, broken
citations and oversized documents refuse nothing the delivery did not
cause.

- **An exceeded baseline count** is keyed by tree and form, with its
  current count and recorded limit. The base runs check 2 only for the
  trees with exceeded counts, in the kept base worktree below. A count
  can be inherited only with a base. A count is inherited when the base exceeds that form too,
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
  worktree (`git worktree add --detach --force`), made at the first
  exceeded count and kept; each later need moves it with `git checkout
  --detach --force <base>` then `git clean -ffdxq`. It holds check 2's
  base runs alone, without the private index (`GIT_INDEX_FILE`): no cargo
  runs in it, so the head's target directory and the timestamps of either
  checkout's files stay as they are. It is not made for a lint, test or
  build failure.
- **The known reds** are `.temper/known-reds.md`, read from section 3's
  private snapshot (`git cat-file blob <tree>:.temper/known-reds.md`), so
  the delivery's own edits to it count, staged, unstaged or new. A tree
  without it has no known reds. It is read once a run, after the cache's
  check and before check 1, whatever the selection, with or without a
  base; when present, one line says so: `pre-pr: known reds: <N> entries
  (.temper/known-reds.md)`. It is Markdown, so that a designer delivers
  it. Each line, its trailing whitespace stripped, is an entry when it is
  one of:
  - `- test <binary id> <name>`: a failing test, keyed as below: its
    binary id, which holds no whitespace, then a space and its name, the
    rest of the line;
  - `- lint <package> <code> <file>`: a lint error, keyed as below without
    its message: its package and lint code, neither holding whitespace,
    then a space and its file, the rest of the line. An error with no
    lint code has no entry.

  Every other line is prose and is ignored, an indented one included.
  Entries match exactly, case included.
- **The cap:** at most 64 entries and 8,192 bytes. A file over either is
  the failure `known-reds`, never inherited, its log naming the file, what
  it holds and the cap (`.temper/known-reds.md: 65 entries, 2,310 bytes;
  the cap is 64 entries and 8,192 bytes`); none of its entries counts.
- **A lint error** is keyed by its package's name, its lint code (empty
  for a compiler error without one), its primary span's file, relative to
  its worktree's root, and its message; never by line or column. It is
  inherited when a `- lint` entry gives its package, code and file,
  whatever its message; otherwise it is the delivery's.
- **A failing test** is keyed by its binary id and its name. It is
  inherited when a `- test` entry gives both; otherwise, failing at the
  base or not, it is the delivery's.
- **A build failure** (section 4's) is always the delivery's, whatever
  the known reds hold; so is a failure of a check itself (`check clippy`,
  `check one-process`, `check selection`).
- **A starved test** is neither failed nor inherited: section 4's warning
  names it, and it refuses nothing, listed or not.
- **A base run that cannot run** (the worktree cannot be made, or a
  tree's `scripts/old_forms.py` fails of itself at the base): the
  exceeded counts stay the delivery's, and the summary says why the base
  could not judge them. The citation check's independent comparison and
  the known reds still stand.
- **Inherited never refuses:** each is named in the summary, with the
  base for a count or a citation and with the file for a known red, and
  the delivery passes on them.

## 6. Its output

- **As it goes:** section 3's selection lines, each selected check's
  announcement, then what its tools print,
  so a stopped check's tail shows where it was (smith's run.md, 8.1).
- **The summary, last,** at most 8,192 bytes, which smith's kept tail
  holds (smith's `protocol/limits.md`, `shell_tail`):
  - `pre-pr: passed` or `pre-pr: refused`, with the base (`base <id>,
    merge base of refs/temper/landing and HEAD`) or that there was none;
  - the sets: `lints: <packages>`, `tests: <packages>`;
  - section 4's starved line, where the tests starved;
  - for each failure this delivery made: its check (`format`, `old-forms
    <tree>`, `clippy <package>`, `test <binary id> <name>`, `build`,
    `known-reds`, `check <name>` for a check that failed of itself), then the last lines
    of its log: a test's JUnit output, with its alone label last where it
    has one, an error's rendered text, a tool's
    output; each at most 1,536 bytes, cut at a line's start and marked
    `[… N bytes cut]`;
  - then each inherited failure, one line each: a known red as
    `inherited: test <binary id> <name> (known red, .temper/known-reds.md)`
    or `inherited: clippy <package> <code> <file> (known red,
    .temper/known-reds.md)`, exceeded counts as `inherited: old-forms
    <tree> <form>`, and the inherited citation total;
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
- **No build or test at the base.** A lint, test or build failure goes
  back to the delivery at once, judged against the known reds alone
  (section 5): a failing delivery takes no longer than a passing one. The
  base's worktree is made only for an exceeded count, holds no build, and
  is kept for the next.
- What a check took is the run's: smith tells its host when checks start,
  with their deadline, and when they end (run.md, 8.1), and the check
  prints each check's seconds. The checks' deadline is the deployment's
  `check_timeout`, which the owner sets separately.

## 9. Testing

The boundary world `tests/boundary/tests/pre_pr.rs` (package
`fabrig-boundary`, in the focused suite) runs `.temper/pre-pr` in a scratch
repository against fake `cargo`, `rustfmt` and `tools/tiers/one-process`
programs that record what they were asked and answer from a ledger:
metadata with members, their directories and dependencies; clippy's JSON
messages; the one-process run's exit and its `junit.xml`, `results.json`
and `run.json` in the `--out` directory it was given. It reads no real
build and runs no real `tools/tiers/one-process`, whose own suite proves it
(`tools/tiers/design.md`, 11.10). Its tables: the base (no pin, a pin, a
merge in progress, no merge base, an unchecked save stepped to its parent);
the selection (a package, its dependents, a path reader, a path in no
package, a deletion), with the tests' set given to one-process as section
4's command and no nextest run of the check's own; known reds and new
failures of each kind, a build failure among them, with no cargo run and
no worktree made at the base; a starved run that passes with its warning;
the cache by tree and base; the summary's bound; and the hold: a fake hold
program, named by `TIERS_HOLD`, records its start and its stdin's end in the
ledger, so that one hold starts before the first fake `cargo` call and ends
after the fake one-process has returned, the fake one-process sees no
`TIERS_HOLD`, a cache hit and an unset setting start none, and a hold that
exits before its line leaves the warning and the verdict as it would be.

`tools/pre-pr/test_pre_pr.py` first exercises code-only, docs-only, both
and neither in scratch repositories. It runs the citation checker against
private snapshots: new and inherited broken citations, a deleted cited
document, and a design size violation. It checks that docs-only changes
skip every code check even on a red baseline; unchanged red counts pass
while a rising count or tighter limit refuses. Non-Markdown fixtures,
declared docs inputs and check configuration select code. A pin at one or
two unchecked saves steps to the checked head: the summary names that head,
and package selection includes both the saved edits and the delivery's new
edits. Any other subject keeps the base, and the pin never moves.

It reads the known reds from the snapshot: an unstaged entry for a failing
test or lint makes it `inherited: … (known red, .temper/known-reds.md)` and
passes, prose lines around it ignored; an entry with another key, or no
file at all, leaves the failure the delivery's; a build failure refuses whatever the
entries; and a file over 64 entries or 8,192 bytes fails as `known-reds`.
No cargo runs at the base. It reads no real build.

It reads a one-process run's directory as section 4 does, from a fake run
that writes the files: failures from `junit.xml`, keyed by binary id and
name; a cancelled test left out and the starved line printed, with the
check passing; the alone label ending a failure's log; exit 2 as a build
failure when its output ends with the listing's 101 line and as `check
one-process` otherwise; and the command: `default`, a `--out` that does not
yet exist, one `-p` per package, `--no-tests=warn`, and no `--workspace`,
`--profile` or `--tool-config-file`.

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
- **A test that cannot yet share its binary's process** fails every
  delivery whose tests' set holds its package, with the alone label
  (section 4). Its repair is its own tree's; until then the stopgap is the
  `process-of-its-own` test group (`tools/tiers/design.md`, 11.3), an
  override in `.config/nextest.toml` saying why beside it.
- **A resolution's own edits.** A resolution's base is the branch's head
  before the merge (section 2), so its selection holds what the merge
  brings in as well as its hand edits; a merge of a far-moved landing
  branch selects much. Selecting only the paths its edits changed against
  both parents is not designed.
