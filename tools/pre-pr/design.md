# The delivery check

Provisional, 2026-11-15. What fabrig's `.temper/pre-pr` runs when a run
delivers: the project's checks (smith's `smith/docs/design/domain/run.md`,
8.1; temper's `temper/docs/design/protocol/configuration/reference.md`), written
from `temper/docs/design/draft/delivery-checks.md`, for the goal's reds,
its flakes and its results, `temper/docs/design/draft/branch-reds.md`,
for where its tests sit, `temper/docs/design/draft/focused-pyramid.md`,
for the failures file, `temper/docs/design/draft/pre-pr-failure-list.md`, and,
for the boundary packages and the fuzzy tests, the user's word of 2026-11-15,
after a landing's gate caught three reds in tests no delivery had run.
Coding agents run no build
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
  baselines, clippy, and the focused and fuzzy tests; Markdown anywhere and
  every file under a `docs/` directory select citations and design size
  limits. Declared documentation inputs to tests select both (sections 3
  and 4).
- **The boundary packages, whatever was selected.** Every delivery that
  selects either set also tests the boundary packages: `fabrig-boundary`
  and each package on a `.config/gate/always.txt` line without a prefix,
  whose tests read files across the tree. A docs-only delivery runs their
  focused and fuzzy tests beside citations, so a name the outward rule
  forbids (`AGENTS.md`, 1) never reaches a pushed branch unchecked
  (section 3).
- **A binary is a process.** The focused tests, then the fuzzy tests of
  the same set, run through `tools/tiers/one-process default` and
  `tools/tiers/one-process fuzzy`: each test binary's selected tests in one
  process, one at a time, binaries in parallel, nextest only selecting and
  building (section 4). A starved run is a warning.
- **A goal's design delivery too.** A goal run's `deliver` of Markdown
  straight to its goal branch runs the project's checks before the worker
  pushes, and a failure refuses the push with the check's output
  (`temper/docs/design/domain/forge/branches.md`, 3; smith's run.md, 8.1).
  The worker pins its checkout's `refs/temper/landing` at g's tip as for a
  change, so a design delivery runs this check's docs set and the boundary
  packages' tests, with no check of its own.
- **One hold for the check.** With `TIERS_HOLD` set, the check holds its
  build directory from before its first build to its verdict, so the
  executables one-process built are still there when it runs them
  (section 4; `tools/tiers/design.md`, 3).
- **The goal's reds, and inherited counts.** Nothing is built or tested
  at the base. A lint or test failure is inherited only when the goal's
  reds, which temper keeps and writes to `temper/reds` in the git
  directory, name it, the check waiting for that file where temper says
  it comes; the tree keeps no list of its own. A build failure is always
  the delivery's. An exceeded baseline count is compared with the base's, with
  no build; citations compare their counted findings and design sizes
  there too. Inherited failures are named and pass (section 5); every
  other failure refuses.
- **Its process again.** A failing test no entry names runs once more
  after the run, its process's tests up to it in one process with
  nothing else of the check running, at most temper's `temper/rerun`
  bound; passing then, it failed under load: a flake, named, refusing
  nothing (sections 4 and 5).
- **A summary last,** within a cap of its own: its verdict, then a line
  that counts the failures by class and names the failures file, then
  each failing check and test with its log's tail, then the flakes and
  the inherited ones (section 6).
- **Every failure's key kept.** Before the summary prints, it writes every
  failure it found, new, flake and inherited alike, with its key, to
  `temper/pre-pr-failures.json` in the git directory, never the tree;
  the file stays until the next completed check replaces it (section 6).
- **Its results for temper.** Under temper's marker it writes
  `temper/check-results` at its end, passed or refused: its tree, base
  and verdict, the goal's reds it passed or failed, and its flakes with
  their logs, which the worker reports to the engine (section 6).
- **Fast on a warm checkout:** no `cargo clean`, no whole-workspace run,
  no build at the base, and the passed-tree cache keyed by tree, base and
  the goal's reds (sections 7 and 8).

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
  1–4, docs inputs select check 5 and check 4 over the boundary packages
  alone (below), and both select both sets. With neither, no check runs,
  the boundary packages' tests included. Docs alone read cargo metadata
  for the boundary packages and nothing else of cargo before check 4. Each
  selected set gets one line naming its checks and a changed input that
  selected it:
  `pre-pr: docs only: citations and design size limits (docs input …)`,
  `pre-pr: code only: format, counted baselines, clippy, focused and fuzzy tests (code input …)`,
  or a line each starting `pre-pr: code and docs:`; then, once, the
  boundary line, `pre-pr: boundary tests: <packages>`, the boundary
  packages joined by a space, or `none`. With neither, print
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
  adds its package, where it is a member. A line without a prefix names
  a boundary package instead (A, below). A line naming no member is
  skipped and printed.
- **Boundary packages (A):** `fabrig-boundary`, then the package of each
  `.config/gate/always.txt` line without a prefix, each where it is a
  member; one that is not is skipped and printed, `pre-pr: boundary
  package <name>: not a workspace member, skipped`. Their tests read
  files of the whole tree by path (the outward rule, a kit's vocabulary),
  so any change, prose included, can break them: every delivery that
  selects either set adds them, whatever its inputs. A package on both a
  prefixed and an unprefixed line is in A.
- **The lints' set** is C; **the tests' set** is C ∪ D ∪ R ∪ A. Each is
  ordered by package name and passed as one `-p` per package. A
  docs-only delivery's C, D and R are empty: its tests' set is A.
- **No package selected for code:** the lints' set and C ∪ D ∪ R empty,
  code still runs checks 1 and 2, printing `pre-pr: no package changed`,
  and check 4 over A. Docs still run check 5 and check 4 over A when
  selected. Without a base, section 2's fallback remains checks 1 and 2
  alone, with no boundary packages: selection needs a base.

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
4. **The focused tests, then the fuzzy tests:** first `tools/tiers/one-process --out <dir>/tests
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

   **A failing test's process again** (`temper/docs/design/draft/branch-reds.md`,
   7). After the run and before the judgement, each test failure
   `temper/reds` names no entry for (section 5), a test
   `results.json` gives the `outcome` `failed` or `crashed`, runs once
   more: `python3 -I -B tools/tiers/one-process --again <dir>/tests --out
   <dir>/again-<n> <binary id> <names>`, the names the test's `before` in
   `results.json`, then the test, so its process's tests up to it in the
   same order, in one process, with no build and nothing else of the
   check running (`tools/tiers/design.md`, 11.9). `<n>` counts from 1;
   they run one at a time, in the order of the failures' keys. A failure
   an entry names refuses nothing already and is not run again, so that
   a goal's expected failures never spend the bound; nor is a build
   failure, a failure of the check itself or a cancelled test.
   - **The bound is temper's:** `temper/rerun`, at `git rev-parse
     --git-path temper/rerun`, one line, `temper-rerun 1 N`, N from 0 to
     16, read once with the reds, before the cache's check
     (`temper/docs/design/domain/forge/reds.md`, 5.2). No file is 0, as
     a checkout outside temper has none; a file of another form is 0, and
     one line says so, `pre-pr: rerun: ignored temper/rerun (malformed)`.
     Where more failures would run again than N, none does, since that
     many is no flake, and one line says so: `pre-pr: tests again: none
     (<F> failures, at most <N> run again)`.
   - **Its lines:** each announced `pre-pr: tests again: <binary id>
     <name> (<K> tests) …`, K the names, and ended `pre-pr: tests again:
     <binary id> <name>: passed|failed, <seconds> s`.
   - **What it decides:** the test's `outcome` in that run's
     `results.json` `passed` makes it a flake (section 5); any other
     outcome, an exit other than 0 or 100, or a directory the check can't
     read leaves it failed, judged as section 5 judges it, its log the
     first run's, its alone label included.

   **Then the fuzzy tests** of the same tests' set: `tools/tiers/one-process
   --out <dir>/fuzzy fuzzy -p … --no-tests=warn`, the same `-p` arguments in
   the same order, `.config/nextest.toml`'s `fuzzy` profile its selection
   and its `global-timeout` its only bound. It runs where the focused run
   ran, whatever that ended in (a failure, a build failure, a failure of
   the check itself), once the focused run and its tests again have ended;
   where the focused run did not run (an empty tests' set, a failed build of
   the other packages' binaries, no base), neither does it. It reuses the
   other packages' binaries built for the focused run.
   - **Its lines:** announced `pre-pr: fuzzy tests …` and ended `pre-pr:
     fuzzy tests: passed|failed|inherited, <seconds> s`. Each run's end
     line judges its own failures alone.
   - **Read as the focused run is:** its exit decides as above, its test
     failures from `<dir>/fuzzy`'s `junit.xml`, keyed by binary id and name
     (a fuzzy test's name starts `fuzzy::`), a cancelled test starved, and
     its starved line `pre-pr: fuzzy tests starved: <N> starved or
     cancelled, <M> unobserved, a warning
     (skein/docs/foundation/testing-strategy.md, 8.1)`. Its failures are
     judged against the goal's reds (section 5) and run again as above,
     `--again <dir>/fuzzy`, each with a fresh `--out` of its own, under
     the same bound applied to its own failures alone.
   - **Its own names** where they would be the focused run's: a build
     failure is `build fuzzy`, a failure of the check itself `check
     one-process fuzzy`.

5. **The docs:** `python3 -I -B tools/citations/citations.py --base <base>
   --head <tree>` against section 3's private snapshot, including staged,
   unstaged and untracked documentation. The checker also enforces the
   design size limit for `tools/citations/sized.txt`'s trees (`DESIGN.md`,
   5). A docs-only delivery runs no formatter, counted baseline or clippy,
   and check 4 over the boundary packages alone (section 3), before this
   check. A code-only delivery runs no full citation checker; smith's
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
unless the goal's reds name it, and a build failure always does, so a
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
- **The goal's reds,** which temper keeps and its worker writes at each
  check's start (`temper/docs/design/domain/forge/reds.md`, 5), are
  `temper/reds` under the git directory, at `git rev-parse --git-path
  temper/reds`; temper's marker beside it is `temper/reds-wait`. They are
  the only entries the check counts:
  - **Waited for:** where `temper/reds-wait` is one line,
    `temper-reds-wait 1 MS`, MS from 1 to 600,000, the check waits, first
    of all, before the cache's check, until `temper/reds` exists, looking
    every 50 ms, at most MS milliseconds. With no marker it waits for
    nothing; a marker of another form is no marker, and one line says so,
    `pre-pr: reds: ignored temper/reds-wait (malformed)`.
  - **Read once,** right after the wait, before the cache's check, whole
    or not at all: the bytes read are the ones the cache keys and the
    judgement counts (section 7). A file of more than 2,097,152 bytes is
    oversized and not read.
  - **Its form:** the first line exactly `temper-reds 1`, then each line an
    entry of one of two forms, matched exactly, case included, with no
    prose and no trailing whitespace; a final line feed is optional and an
    empty file is malformed. A file of the header alone is no reds. The
    forms:
    - `- test <binary id> <name>`: a failing test, keyed as below: its
      binary id, which holds no whitespace, then a space and its name, the
      rest of the line;
    - `- lint <package> <code> <file>`: a lint error, keyed as below
      without its message: its package and lint code, neither holding
      whitespace, then a space and its file, the rest of the line. An
      error with no lint code has no entry.
  - **No reds, and why:** a missing, malformed or oversized file counts
    no entry, and one line says which: `pre-pr: reds: none (temper/reds
    not written within MS ms)` where the marker said to wait, `pre-pr:
    reds: none (temper/reds malformed at line L)`, `pre-pr: reds: none
    (temper/reds oversized: B bytes)`. With no marker and no file it
    prints nothing, as a checkout outside temper has none. A file read
    prints `pre-pr: reds: <N> entries (temper/reds)`.
  - **No cap of its own:** temper bounds the file (`forge.branch_reds`).
- **No list in the tree:** the tree keeps no known reds of its own
  (`temper/docs/design/draft/branch-reds.md`, 8). A file a branch still
  carries from before, its lines of the entries' forms included, is not
  read, counts nothing and prints nothing: a failure only it names is the
  delivery's.
- **A lint error** is keyed by its package's name, its lint code (empty
  for a compiler error without one), its primary span's file, relative to
  its worktree's root, and its message; never by line or column. It is
  inherited when a `- lint` entry gives its package, code and file,
  whatever its message; otherwise it is the delivery's.
- **A failing test** is keyed by its binary id and its name. It is
  inherited when a `- test` entry gives both; otherwise, failing at the
  base or not, it is the delivery's.
- **A build failure** (section 4's, `build` or `build fuzzy`) is always
  the delivery's, whatever the goal's reds hold; so is a failure of a
  check itself (`check clippy`, `check one-process`, `check one-process
  fuzzy`, `check selection`).
- **A starved test** is neither failed nor inherited: section 4's warning
  names it, and it refuses nothing, named by an entry or not.
- **A flake,** a failing test that passed when its process ran again
  (section 4), failed under load: it refuses nothing and is neither the
  delivery's nor inherited. The summary names it and the results file
  carries it (section 6). One that failed again is judged as above: a
  test that depends on an earlier one stays the delivery's.
- **A base run that cannot run** (the worktree cannot be made, or a
  tree's `scripts/old_forms.py` fails of itself at the base): the
  exceeded counts stay the delivery's, and the summary says why the base
  could not judge them. The citation check's independent comparison and
  the goal's reds still stand.
- **Inherited never refuses:** each is named in the summary, with the
  base for a count or a citation and with the file for a goal's red, and
  the delivery passes on them.

## 6. Its output

- **As it goes:** section 3's selection lines, each selected check's
  announcement, then what its tools print,
  so a stopped check's tail shows where it was (smith's run.md, 8.1).
- **The summary, last,** at most 8,192 bytes. The cap is pre-pr's own,
  apart from what the host keeps: smith keeps a longer tail of a check's
  output (smith's `protocol/limits.md`, `shell_tail`, 48 KiB by default),
  but a session may later fit those words to a shorter prefix, so the
  summary's start is what survives, and what must never be lost comes
  first:
  - `pre-pr: passed` or `pre-pr: refused`, with the base (`base <id>,
    merge base of refs/temper/landing and HEAD`) or that there was none;
  - the failures line, naming the failures file (below) and counting
    its entries by class, the same counts as the file's:
    `failures: <N> new, <F> flakes, <I> inherited; every key in <path>`,
    `<path>` the file's absolute path. Where the file could not be
    written: `failures: <N> new, <F> flakes, <I> inherited; not written
    to <path>: <error>`, `<path>` as far as it was found, else
    `temper/pre-pr-failures.json`, and `<error>` the error's first line,
    at most 512 bytes, cut at a character's end;
  - the sets: `lints: <packages>`, `tests: <packages>`;
  - section 4's starved lines, the focused run's then the fuzzy run's,
    each where its tests starved;
  - why the base could not judge an exceeded count, where it could not
    (section 5).

  The first two lines are kept whole; what follows them before the first
  failure is cut to keep these lines within 4,096 bytes, at a
  character's end, so that a large set leaves room for a failure and the
  count. Then:
  - for each failure this delivery made: its check (`format`, `old-forms
    <tree>`, `clippy <package>`, `test <binary id> <name>`, `build`,
    `build fuzzy`, `check <name>` for a check that failed of itself), then the last lines
    of its log: a test's JUnit output, with its alone label last where it
    has one, an error's rendered text, a tool's
    output; each at most 1,536 bytes, cut at a line's start and marked
    `[… N bytes cut]`;
  - then each flake (section 5): `flake: test <binary id> <name> (failed
    under load; passed when its process ran again: <K> tests, <seconds>
    s)`, then its first run's log as a failure's, within the same bound;
  - then each inherited failure, one line each: a goal's red as
    `inherited: test <binary id> <name> (red, temper/reds)` or
    `inherited: clippy <package> <code> <file> (red, temper/reds)`,
    exceeded counts as `inherited: old-forms <tree> <form>`, and the
    inherited citation total;
  - past 8,192 bytes, the failures it has not room for are counted in
    one line, `[… N more failures]`, the first kept whole; the
    failures file holds each of them.
- **The failures file** holds every failure the run found, new, flake
  and inherited alike, so that a key the summary had no room for is
  still there to read once the run's directories are gone:
  - **Where:** `git rev-parse --git-path temper/pre-pr-failures.json`,
    read from the checkout's root as the results file's path is: the git
    directory, never the tree, and in a linked worktree that worktree's
    own git directory (`temper/` is none of git's shared paths). It is
    written through `temper/pre-pr-failures.json.tmp` renamed over it,
    `temper/` made where it is missing, so a reader sees one check's
    file whole.
  - **When:** after the checks and before the summary prints, by every
    run that prints a summary, passed or refused, with no failure as with
    hundreds, with or without temper's marker; and by a cache hit, marked
    as one (section 7). A run stopped, interrupted or failed before its
    summary writes none, so the last completed check's file stays until
    the next completed check replaces it.
  - **Its form:** one JSON object in UTF-8, text as itself, non-ASCII
    included (a string no UTF-8 can hold, a lone surrogate, keeps its
    `\uXXXX` escape). Its members before `failures` and the list's
    opening on the first line, each failure on a line of its own, and
    `]}` on the last, so that it reads and searches by line:

    ```
    {"form": "pre-pr-failures 1", "tree": "<tree>", "base": "<base>", "cached": false, "verdict": "refused", "counts": {"new": 2, "flake": 0, "inherited": 1}, "failures": [
    {"class": "new", "name": "test A::rules rule_00", "key": {"kind": "test", "binary": "A::rules", "test": "rule_00"}},
    {"class": "new", "name": "format", "key": null},
    {"class": "inherited", "name": "clippy api clippy::rule api/src/lib.rs", "key": {"kind": "lint", "package": "api", "code": "clippy::rule", "file": "api/src/lib.rs", "message": "<message>"}}
    ]}
    ```

    - `tree`, the tree the check ran on (section 3; the cache's
      `<tree>`), and `base`, section 2's base, or `null` without one;
    - `cached`, `true` on a cache hit alone; `verdict`, `passed` or
      `refused`, as the summary's first line;
    - `counts`, the entries of each class;
    - `failures`, one entry for each failure the summary shows or would
      show with room, in its order: the new ones, then the flakes, then
      the inherited ones. `class` is `new` (this delivery's, refusing),
      `flake` (section 5) or `inherited`; `name` is the summary's name
      for it; `key` its structured key: a test's `{"kind": "test",
      "binary", "test"}`, a lint's `{"kind": "lint", "package", "code",
      "file", "message"}` (`code` and `file` empty where the compiler
      gave none), an exceeded count's `{"kind": "old-forms", "tree",
      "form", "count", "baseline"}`, its numbers as numbers, and `null`
      for any other failure (`format`, `build`, `check <name>`, an
      old-forms run's failure, citations), whose name says it whole.

    It holds no log: the summary keeps the logs that fit.
  - **A write that fails,** the path's lookup included, is named in the
    failures line and changes nothing else: the exit, the summary's
    other lines and the results file are as they would have been.
- **Its exit:** 0 when nothing this delivery made failed, inherited
  failures and flakes included; 1 otherwise. Each tests run's end line,
  the focused and the fuzzy, says `inherited` where each of its own
  failures was inherited or a flake.
- **The results file,** for temper (`temper/docs/design/domain/forge/reds.md`,
  5.5, its form): where the check found a marker `temper/reds-wait` of
  its form (section 5), after the summary, passed or refused, it writes
  `temper/check-results` at `git rev-parse --git-path
  temper/check-results`, through `temper/check-results.tmp` renamed:
  - `temper-check 1`, then `check <tree> <base> passed|refused`, `<base>`
    `-` without one;
  - for each entry of `temper/reds` that the run selected: `passed <key>` or `failed <key>`, in key order. A `- test`
    entry is selected where its test is in the focused or the fuzzy run's `results.json`:
    `passed` where its `outcome` is `passed`, `failed` where its
    failure is section 5's, inherited or not, and neither where it
    starved, was cancelled or went unobserved; an ignored test the run
    never selected is neither. A `- lint` entry is selected where its
    package is in the lints' set and clippy ended with no failure of the
    check itself: `failed` where an error has its key, `passed` where none
    does and no error without a lint code is in its package, which would
    hide its lints;
  - then each flake, `flake <key>`, its first run's log as the summary
    cuts it, each line `| <text>`, an empty one `|`.

  A cache hit writes none, nor does a check that wrote no summary.

## 7. The passed-tree cache

`<git dir>/pre-pr-passed` holds `<tree> <base> <reds>` once every check
passed or was inherited: `<base>` is `-` without a base, and `<reds>` the
SHA-256, in lowercase hexadecimal, of the bytes of `temper/reds` the run
read (section 5), or `-` where it read none. A check of the same tree, on
the same base, with the same reds prints `pre-pr: passed before (tree
<tree>, base <base>, reds <reds>)` and exits 0 at once: within a hold the
base does not move, so a `finish` repeats its `deliver`'s pass, while a
red ended or recorded since changes the file and so the key, and a pass
under a red that has ended is never reused. A failure, an interruption or
a timeout never writes it. An earlier record of another shape, the
two-field one included, matches nothing. A cache hit observes nothing:
it writes no `temper/check-results` (section 6), so temper keeps what the
check's last pass reported. It does write the failures file (section 6),
saying so: `cached` `true`, `verdict` `passed`, the tree and base it
reused, zero counts and no failures, since it observed none; after its
line it prints `failures: a cached pass, none observed; in <path>`, or
`…; not written to <path>: <error>` as the summary's failures line does.
The record holds the last pass alone, so a hit is always of the last
tree that passed here.

## 8. Time

- No `cargo clean` and no whole-workspace run: the sets are what the
  delivery changed, their direct dependents and the boundary packages, on
  the checkout's warm target directory.
- **What the boundary packages and the fuzzy run cost** every delivery
  that selects a set: a docs-only one, which ran citations alone, builds
  and runs the boundary packages' tests twice, focused then fuzzy, and a
  code delivery adds the fuzzy run of its tests' set. That is the price
  of catching before the push what the landing's gate would catch after
  it (the user, 2026-11-15). Their seconds are on each run's end line; the
  change that brought them measured them on its own delivery and said so
  in its description. They count against the checks' deadline as every
  check does (below).
- **No build or test at the base.** A lint, test or build failure goes
  back to the delivery at once, judged against the goal's reds alone
  (section 5): a failing delivery takes no longer than a passing one. The
  base's worktree is made only for an exceeded count, holds no build, and
  is kept for the next.
- What a check took is the run's: smith tells its host when checks start,
  with their deadline, and when they end (run.md, 8.1), and the check
  prints each check's seconds. The checks' deadline is the deployment's
  `check_timeout`, which the owner sets separately.

## 9. Testing

The check's tests sit in two places, by what they check
(`temper/docs/design/draft/focused-pyramid.md`, 5, its fifth item). The driver's unit
tests, `tools/pre-pr/test_pre_pr.py`, hold the tables: every row that
classifies what a program or a file gave back (a verdict, a printed line,
a set, a cache key, a results line). They call the driver's `main`
in-process in a scratch repository, with git real and the check's other
programs mocked or faked. The boundary world `tests/boundary/tests/pre_pr.rs`
(package `fabrig-boundary`, in the focused suite) keeps one story per
external path: each program and file the check reaches outside its own
code, run for real from `.temper/pre-pr`. A row is the unit tests' unless
it is its path's one story, and no check is dropped in placing it.

### 9.1 The boundary world

It runs `.temper/pre-pr` in a scratch repository against fake `cargo`,
`rustfmt`, `git`, old-forms and hold programs and a fake
`tools/tiers/one-process`, which record what they were asked and answer
from a ledger: metadata with members, their directories and dependencies;
clippy's JSON messages; the one-process run's exit and its `junit.xml`,
`results.json` and `run.json` in the `--out` directory it was given, and
its `--again` runs. It reads no real build and runs no real
`tools/tiers/one-process`, whose own suite proves it
(`tools/tiers/design.md`, 11.10). Each story runs the fewest deliveries its
path needs; a repeat delivery that would only classify again is the unit
tests'. Its stories, a test each:

1. **No base:** the hook; `cargo metadata`; `rustfmt` on every target,
   with its edition and its tree's configuration; each tree's old-forms
   count; the pass record written and the real index unchanged; the format
   check and the counts within the fixture's 60,000 simulated ms.
2. **A program that fails or dies:** the formatter failing, its error in
   the log; a formatter killed by a signal; one tree's count failing. None
   passes or writes the pass record, and the same tree then passes.
3. **The passed-tree cache:** a pass; the same tree's hit, which prints
   its line and starts no program; a changed source checked again.
4. **The base from git:** a merge in progress, whose base `git merge-base
   refs/temper/landing HEAD MERGE_HEAD` gives and the log names; a
   docs-only delivery running the citation checker and the boundary packages'
   focused and fuzzy tests, with no formatter, counted baseline or clippy.
5. **The private index:** new sources the counts read through it, the
   real index unchanged.
6. **The sets given to the tools:** a changed package and its direct
   dependent, from `cargo metadata`, given to clippy and to one-process as
   section 4's commands, with no nextest run of the check's own.
7. **Clippy's messages:** a lint no entry names, read from clippy's JSON,
   refusing by its package, code and file; the tests still run; no cargo
   runs and no worktree is made at the base; the summary comes after the
   tools' streamed output.
8. **temper's files and a process again:** `temper/reds-wait`,
   `temper/rerun` and `temper/reds` read from the git directory; a failing
   test no entry names in each of the focused and fuzzy runs run again
   through one-process's `--again` with its `before`, each from its own
   run directory under its own bound, a flake refusing nothing, and an
   entry's failure not run again; `temper/check-results` written whole
   with both runs' observed reds and flake logs, its `.tmp` gone; then a
   cache hit writing none.
9. **The hold:** the hold program `TIERS_HOLD` names starts before the
   first `cargo` call and ends after one-process has returned, the pass
   record written by then; one-process sees no `TIERS_HOLD`; a cache hit
   starts none.
10. **A hold that exits early:** its warning once, and the check going on
    unheld to the verdict it would have given.

### 9.2 The driver's unit tests

`tools/pre-pr/test_pre_pr.py` first exercises code-only, docs-only, both
and neither in scratch repositories. It runs the citation checker against
private snapshots: new and inherited broken citations, a deleted cited
document, and a design size violation. It checks that docs-only changes
skip formatting, counted baselines and clippy even on a red baseline;
unchanged red counts pass
while a rising count or tighter limit refuses. Non-Markdown fixtures,
declared docs inputs and check configuration select code. A pin at one or
two unchecked saves steps to the checked head: the summary names that head,
and package selection includes both the saved edits and the delivery's new
edits. Any other subject keeps the base, and the pin never moves.

Its table of the base: no pin, and a pin with no merge base, each print
the no-base line with its reason and run the format check and the counts
alone; a pin behind `HEAD` prints its merge base. Its table of the
selection, by the lints' and the tests' sets: a dependent through a dev
dependency; a dependent's own dependents left out; an untracked and a
deleted source; the root manifest, and a directory beside a package's
whose name begins with it, selecting no package and saying so; a docs
path; a reader in `.config/gate/always.txt` by its prefix; a reader
alone, its input in `.config/gate/read-by-tests.txt`; a reader line
without a prefix, selecting its boundary tests; and a reader naming no package,
printed. Each tree's count failing refuses, naming its tree.

It reads the goal's reds from the git directory: an entry of
`temper/reds` for a failing test or lint makes it `inherited: … (red,
temper/reds)` and passes, the check's end line saying `inherited`; an
entry with another key (another test, binary, lint code or file) leaves
the failure the delivery's, a test's JUnit output in the summary; a build
failure refuses whatever the entries; a header alone, a
missing file, a malformed or an oversized one count nothing, each with its
line. A list in the tree counts nothing: a tree whose old list file names
a failing test or lint, with no `temper/reds` entry for it, refuses it as
the delivery's and prints no line of the list. No cargo runs at the base,
and it reads no real build. A marker with a small
MS and no file waits about MS and judges with no reds; a marker and a file
written while the check waits is read; no marker and no file print
nothing. Its cache: a pass with a reds file is reused with the same bytes
and not after they change, nor after the file goes, a two-field record
matches nothing, and a moved pin is checked again.

It runs a failing test's process again from a fake one-process whose
`--again` runs answer from the ledger: a failure no entry names that
passes again is a flake, refusing nothing, named in the summary and
written to the results file with its log; one that fails again refuses,
its alone label kept; a failure an entry names is not run again; more
failures than `temper/rerun`'s N, an N of 0, no file and a malformed one
run none again, each with its line; the command: `--again` the run's
directory, a fresh `--out`, the binary id, then the test's `before` and
the test.

It writes the results file only under a marker: its header and check
line, passed and refused; a `temper/reds` entry passed, failed (inherited)
and starved (neither), a lint entry passed and failed; a flake's `|` lines; nothing on a cache hit,
and nothing without a marker.

It writes the failures file (section 6) at every summary and on a cache
hit, marker or none, and its tests are this file's alone: with 26, 194,
349 and 450 failures of mixed classes and kinds, the file holds every key
in the summary's order with its class, its counts match, and the summary
stays within 8,192 bytes with its first two lines the verdict and the
failures line, which names the file's path and the counts; a test name
of several kilobytes and keys with non-ASCII text are kept whole and read
back equal; from a fake one-process run, the file still holds every
failure once the run's temporary directory is gone; in a linked
worktree, the file is in that worktree's git directory and not in the
main one's; a write that fails, the file's place taken by a directory, is
named in the failures line and leaves the exit and the rest of the
summary as they would be; a later passing check replaces a refused one's
file, and a cache hit writes one marked `cached`.

It reads a one-process run's directory as section 4 does, from a fake run
that writes the files: failures from `junit.xml`, keyed by binary id and
name; a cancelled test left out and the starved line printed after the
run and again in the summary, with the check passing; the alone label
ending a failure's log; exit 2 as a build
failure when its output ends with the listing's 101 line and as `check
one-process` otherwise; and the command: `default`, a `--out` that does not
yet exist, one `-p` per package, `--no-tests=warn`, and no `--workspace`,
`--profile` or `--tool-config-file`.

It selects the boundary packages (section 3): a Markdown-only change runs
one-process over them alone, focused then fuzzy, beside citations, and no
format check, counted baseline or clippy; a package on an unprefixed
`.config/gate/always.txt` line is in the tests' set of a code delivery
that reaches nothing of it, and `fabrig-boundary`, a member, is in every
set; a line naming no member is printed and skipped; no inputs, or no
base, run neither. It runs the fuzzy tests after the focused ones, as
section 4 gives them: `fuzzy`, a `--out` of its own, the same `-p`
arguments in the same order; after a focused run that failed, or ended in
a build failure, as after one that passed; and not where no focused run
was selected (an empty tests' set, a failed build of the other packages'
binaries). A fuzzy run's failure is read as a focused one's: keyed by
binary id and name, inherited from a `- test` entry, its starved line
printed and repeated in the summary, a failure no entry names run again
from `<dir>/fuzzy`; its build failure is `build fuzzy`; and each run's end
line judges its own failures alone.

The summary keeps section 6's bound: forty failures of 4,096 bytes each
refuse within 8,192 bytes, the first kept, each log's tail starting at a
whole line under its `[… N bytes cut]` marker and keeping its last line,
the rest counted in one `[… N more failures]` line. The hold: an unset or
empty `TIERS_HOLD` starts none, and a hold that exits early under a
refusal leaves the refusal.

## 10. Later

- **Inputs read by tests** (`.config/gate/read-by-tests.txt`) outside
  workspace packages and prefixed path readers still map to no package:
  they select the code set, but no lint or test package. The landing's CI
  runs their readers.
- **What no package owns** (the root manifest and lock, a tree's
  `clippy.toml`, `.config/nextest.toml`) selects the code set but no lint
  or test package; the landing's CI checks its consumers.
- **The gate's other checks** (wasm32 clippy, the lint profiles) stay the
  landing's.
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
