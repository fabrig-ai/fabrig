# The delivery check

Provisional, 2026-11-15. What fabrig's `.temper/pre-pr` runs when a run
delivers: the project's checks (smith's `smith/docs/design/domain/run.md`,
8.1; temper's `temper/docs/design/protocol/configuration/reference.md`), written
from `temper/docs/design/draft/delivery-checks.md`, for the goal's reds,
its flakes and its results, `temper/docs/design/draft/branch-reds.md`,
for where its tests sit, `temper/docs/design/draft/focused-pyramid.md`,
for the failures file, `temper/docs/design/draft/pre-pr-failure-list.md`,
for the boundary packages and the fuzzy tests, the user's word of 2026-11-15,
after a landing's gate caught three reds in tests no delivery had run, and,
for the failures file's place in the tree and a prose-only delivery's check
with no build, `temper/docs/design/draft/small-fixes-1116.md`, 1 and 3,
and, for the budgets it holds where its builder keeps main's ledgers,
`temper/docs/design/draft/test-budgets.md`, 3, and for the layers it
runs, the same draft's 4.

## 1. In one page

Every worker `deliver` and `finish` calls `.temper/pre-pr` in the implementer's
checkout. The hook checks the worktree as delivered, including staged,
unstaged and new files. Its private Git index records the checked tree and
leaves staging intact. The local prepare pin identifies committed work since
the branch started. The hook neither fetches a branch nor checks out a base.
It uses the workspace's existing Cargo target and works offline.

Delivery formats changed handwritten Rust files with rustfmt and skips codec
outputs listed in `tools/merge/codecs.txt`. It reads offline Cargo metadata
for their editions. The worker hook inspects the files in place with rustfmt.
When a file needs formatting it renders the original snapshot blob, updates
the private index and applies the resulting patch once. A clean selection
needs one formatter pass per edition and configuration. The remaining checks
and publication use the formatted tree. Explicit child and goal-close
checks are read-only.

The worker can establish the scope at its comparison base, from the last
check of the same base and exact published tree in this checkout, or from
a main fetch made since the worker installed the checkout and integrated
at HEAD or in the pending merge. Without that evidence delivery refuses
before Cargo metadata or rustfmt. Fetching main and merging it reestablishes
the scope. The hook does no fetch and writes no source-tree receipt.

Counted baselines, citations, outward names, journey and
budget rules apply to the changed paths. Checker changes also run its
subprocess-free Python suite. Delivery and finish perform no Rust build or test.
The worker receives its check results and the receipt at
`target/temper/pre-pr-light-failures.json`. A new tree requires a new check.

After status settlement a checked close passes the pinned main commit to
`pre_pr.py --close DESTINATION`. It applies the lightweight checks above
to the entire difference from the merge base to the staged goal tree,
including earlier children and a close that changed no files. The selector
runs focused Rust tests for changed packages and their immediate consumers.
An edit confined to an integration test target selects that target alone.
Close builds any required test helpers, checks packages without test targets
and runs changed Python modules’ sibling suites. Per-delivery
journey mixing and budget edits were checked on delivery. The final goal
tree still receives the overlay check. The receipt records the pre-settlement
head, staged tree, destination and merge base. A changed snapshot refuses.

The main Gate runs the checker suite for a candidate confined to the checker.
A mixed product candidate also receives its regular Gate. The accumulated
goal proof is available by explicit diagnostic dispatch. It checks the goal
head against its supplied destination and uses the full selections described
below. Ordinary delivery and PR events require no separate CI checkout.

The following sections record the full diagnostic and earlier delivery
policies. For the current worker command, selection, formatting, tests and
receipts, this section takes precedence.

- **A base the worker pins.** The worker writes `refs/temper/landing`, the
  landing branch's tip as fetched at prepare, in each change producer's
  checkout and never moves it within the hold
  (`temper/docs/design/protocol/git.md`, 4.3;
  `temper/docs/design/domain/worker/checkouts.md`). The check's base is
  the merge base of that reference and `HEAD`, and `MERGE_HEAD` in a merge
  (`design.md`, section 2). Without the reference it runs today's checks alone and says so.
- **What changed, and who reads it.** The packages owning a path the
  delivery changed against its base, and their direct dependents in the
  workspace (section 3). A path a merge brought in, as the merged tip
  holds it, is not the delivery's: a merge repair selects what it changes
  against both parents, never what the landing branch's own gate passed.
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
  (section 3), unless it is prose only (below).
- **Prose only, no build.** A delivery whose every changed path is
  Markdown under a `docs/` directory that no registered reader declares
  as its input (`.config/gate/read-by-tests.txt`,
  `.config/fixture-readers.toml`) runs the outward scan as a text check
  (`tools/outward/design.md`) and citations, and no cargo command at all:
  the boundary packages' tests read no such file but through the outward
  scan, which the text check gives the same verdict as the Rust test
  (section 3).
- **A binary is a process.** The focused tests, then the fuzzy tests of
  the same set, run through `tools/tiers/one-process default` and
  `tools/tiers/one-process fuzzy`: each test binary's selected tests in one
  process, one at a time, binaries in parallel, nextest only selecting and
  building (section 4).
- **The layers that replay, as the gate runs them.** One-process runs by
  default the binaries of the layers that replay, step tests and domain,
  system, machine, protocol, io and simulated worlds, with the
  repository's checks, and leaves out the real loop and end to end, which
  run after landing (`tools/tiers/layers.md`, 9;
  `skein/docs/foundation/testing-strategy.md`, 8). The check passes no
  `--layers`, so it runs what the gate runs, judges only those layers, and
  builds no real-loop or end-to-end binary. Before linking, the delivery
  omits an integration target when neither profile can select any of its
  cases, or its layer is one the run leaves out. A filter on a case
  name keeps the target whenever the name could match. Cargo must resolve
  the same build units and dependency features for every retained target;
  otherwise the delivery keeps the ordinary package build. A starved run
  is a warning.
- **The budgets, where the builder keeps main's ledgers.** With
  `TIERS_LEDGER` naming a directory that holds main's ledger for a suite,
  which `ledger()` reads, that suite's run takes it and the gate's
  verdict: a layer the delivery takes over its budget, or a test past its
  reference, refuses it, named as any failure is. Without one, the suite
  runs as before, its comparison a warning, and a line says why (section
  4; `tools/tiers/budgets.md`, 5).
- **Every figure the user's.** A delivery that changes
  `tools/tiers/budgets.toml` and any other path, and raises a figure there
  or adds a reason's line, is refused, as the gate refuses it; one that
  changes that file alone passes (section 4, check 7;
  `tools/tiers/budgets.md`, 10.1).
- **Journey files, the writer's alone.** A delivery whose own changes
  hold a journey's recording, page or file printed beside them and any
  other path is refused, and so is a nonempty `overlay.css`; a delivery
  of those files alone passes (section 4, check 8;
  `temper/docs/design/client/journey-writer.md`, 2 and 5).
- **A goal's design delivery too.** A goal run's `deliver` of Markdown
  straight to its goal branch runs the project's checks before the worker
  pushes, and a failure refuses the push with the check's output
  (`temper/docs/design/domain/forge/branches.md`, 3; smith's run.md, 8.1).
  The worker pins its checkout's `refs/temper/landing` at g's tip as for a
  change, so a design delivery runs this check's docs set and the boundary
  packages' tests, with no check of its own; one of status files and other
  prose under `docs/` alone is prose only, and builds nothing.
- **One hold for the check.** With `TIERS_HOLD` set, the check holds its
  build directory, and the build places its environment admits builds to,
  from before its first build to its verdict, so the executables
  one-process built are still there when it runs them and no cargo command
  of its waits for a place again (section 4; `tools/tiers/design.md`, 3).
  A hold that says it waits for a place has the check tell smith, so that
  the wait does not count against the checks' bound (smith's
  `smith/docs/design/domain/run.md`, 8.1). A prose-only delivery builds
  nothing and takes none.
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
  `target/temper/pre-pr-failures.json` under the checkout's root, ignored
  by git and outside the git directory, so that an agent's `read` reaches
  it; the file stays until the next completed check replaces it (section 6).
- **Its results for temper.** Under temper's marker it writes
  `temper/check-results` at its end, passed or refused: its tree, base
  and verdict, the goal's reds it passed or failed, and its flakes with
  their logs, which the worker reports to the engine (section 6).
- **Fast on a warm checkout:** no `cargo clean`, no whole-workspace run,
  no build at the base, and the passed-tree cache keyed by tree, base, the
  goal's reds and main's ledgers (sections 7 and 8).

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
  ): the base is that head, so a CI repair of a goal's landing into main
  checks what its own edits changed, not the goal's packages. A resolution's
  base is its branch's head before the merge, so its check selects what the
  merge and its edits change on the branch against both parents: a path
  as the merged tip holds it was checked there (section 3, *Changed
  paths*).
- **A landing's repair is pinned at that landing's base.** A repair asked
  by a landing's gate names a pin, the base that landing's round used
  (`temper/docs/design/domain/worker/checkouts.md`, *A named pin*), and is
  judged as the landing judged its change: the base is that commit once
  the repair has merged it, whichever parent of the merge it is. The
  whole diff against that base holds what the change and the repair bring
  onto it, never what the base gained since the change's source. Selection
  then sets aside merged tips' entries (section 3, *Changed paths*): an
  unchanged path from the change's head is excluded when that head is a
  merged tip. Check 7 starts from the whole diff and sets aside figures
  itself (section 4). A repair that leaves the pin unmerged is judged from
  where its branch meets it, with the same selection rule. Every other
  repair keeps the head it repairs.
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

### Delivery and CI entry points

The default light check requires no goal ID, destination argument or
worker upgrade. A checkout with a landing pin uses it for the per-change
comparison. The explicit `.temper/goal-child-pre-pr DESTINATION` command
also checks light rules and verifies that the supplied commit matches the
worker's pin. Its scoped report is `target/temper/goal-child-pre-pr.json`.

`.temper/goal-pre-pr HEAD DESTINATION` checks a completed goal in an
isolated checkout at HEAD. The caller supplies both full commit IDs and
the history needed to find their common ancestor. The command requires a
clean index and worktree, the specified head, and a single merge base.
It selects the entire diff from that base to HEAD, including changes
introduced by merged children. An independent advance on the destination
leaves the comparison at the fork. The selected packages run through the
ordinary build, lint and test commands with the same builder hold.

Journey file mixing and budget figure changes are judged on each delivery.
The whole-tree empty-overlay rule also applies at goal completion.
The completed-goal command runs afresh and records the goal head,
destination and comparison base in its verdict. The CI caller uses the
exit status and validates the receipt against the exact pair it requested.
A later head or destination requires a fresh invocation.

## 3. What it selects

- **The tree** is the working tree as git sees it, untracked files
  included and ignored ones not: written through a private copy of the
  index (`git add -A`, `git write-tree`), the real index untouched, as
  before.
- **Changed paths:** `git diff-tree -r --no-renames --name-only -z <base> <tree>`,
  deletions included, less each path whose entry in the tree is the one a
  merged tip holds at that path, its own or absent alike: a merge brought
  it as it stood there, where it was checked (the landing branch's own
  gate, or the delivery that put it on that branch). The merged tips are
  section 4, check 7's. So a merge's changed paths are those it changes
  against both parents: its resolution's and its later edits', and a
  path both sides changed; a merge repair of a goal's branch that took
  main in selects none of main's packages for what main brought as main
  holds it. Every reader of the changed paths below takes them so, and
  check 8 too; check 7 alone is started by the whole diff, merged tips'
  entries included, since it sets them aside itself (section 4, check
  7). Where there are merged tips, one line before the
  selection's names them, `pre-pr: merged tips: <commit> …: what they
  hold is not the delivery's`, the commits in check 7's order joined by
  a space. What the two sides make together where no path differs from
  both is the landing's gate's (section 10).
- **Docs inputs:** every Markdown path (`.md`), anywhere, and every file
  with a `docs` directory component. Additions, edits and deletions all
  select check 5. No docs input means no full citation check.
- **Code inputs:** Rust (`.rs`); Cargo manifests and locks; rustfmt,
  clippy and Rust toolchain configuration, including under `docs/`;
  non-docs paths under a `.cargo/` directory, `.config/` and `tools/`;
  `.temper/pre-pr`, `.temper/goal-child-pre-pr` and `.temper/goal-pre-pr`;
  `skein/scripts/*.sh` (the boundary profile check's
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
  `pre-pr: no code or docs inputs changed`. A prose-only delivery
  (below) prints its own two lines instead of the docs-only line and the
  boundary line.
- **Prose only:** a delivery with docs inputs and no code input, whose
  every changed path, deletions included, ends in `.md` and has a `docs`
  directory component, is prose only. It runs check 5 and check 6 (the
  outward text check, section 4) and nothing else, and reads nothing of
  cargo: no `cargo metadata`, no boundary packages, no build, no test and
  no hold (section 4, *The hold*). Its lines, in place of the docs-only
  line and the boundary line:
  `pre-pr: prose only: outward names, citations and design size limits (docs input …)`,
  then `pre-pr: boundary tests: none (prose only)`. Its lints' and tests'
  sets are empty.
  - **Why it needs no test:** a test that reads a Markdown file by path
    or compiles it in declares it (*Declared readers*, above), which makes
    it a code input; of the boundary packages' tests, only
    `fabrig-boundary`'s outward scan (`tests/boundary/tests/outward.rs`)
    reads undeclared Markdown under `docs/`, and check 6 gives its verdict
    with no build (`tools/outward/design.md`, 1).
  - **Why under `docs/`:** `fabrig-boundary`'s `cli_names` test scans
    every tracked file outside a `docs/` directory for an environment
    variable's name, Markdown included (a package's README, `AGENTS.md`,
    `DESIGN.md`), and a glob of `read-by-tests.txt` cannot name that set.
    A delivery that changes Markdown outside `docs/` stays docs only, with
    the boundary packages' tests.
  - **The declarations it rests on:** each boundary package's test that
    reads Markdown under `docs/` is declared in
    `.config/gate/read-by-tests.txt`: among them every tree's
    `docs/design/cli/*-env.md` and `*-args.md`, which the command-line
    tools' reference tests compare whole, and
    `temper/docs/design/testing/qa/setups.md`, whose recipe the QA tool's
    tests compare. A new test that reads Markdown declares it there
    (`AGENTS.md`, 2).
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
  selects either set adds them, whatever its inputs, but a prose-only one
  (above), whose prose they read only through the outward scan. A package
  on both a prefixed and an unprefixed line is in A.
- **The lints' set** is C; **the tests' set** is C ∪ D ∪ R ∪ A. Each is
  ordered by package name and passed as one `-p` per package. A
  docs-only delivery's C, D and R are empty: its tests' set is A, and a
  prose-only one's is empty.
- **No package selected for code:** the lints' set and C ∪ D ∪ R empty,
  code still runs checks 1 and 2, printing `pre-pr: no package changed`,
  and check 4 over A. Docs still run check 5 and check 4 over A when
  selected. Without a base, section 2's fallback remains checks 1 and 2
  alone, with no boundary packages: selection needs a base.

## 4. What it runs

In this order, each announced by a line `pre-pr: <check> …` before it
starts and ended by `pre-pr: <check>: passed|failed|inherited, <seconds> s`.
Independent checks continue after a failure so the delivery reports their
results together. The test phases below share build prerequisites. Section 6
defines the verdict.

An optional single-retry trial moves the focused profile before formatting,
counted baselines and Clippy. A checkout-local `temper/repair-trial` in
the Git directory names one exact new focused failure in the preceding
uncached refusal in the last hour. Its base and HEAD write must match and the candidate
tree must have changed. Consume the invitation once. A known red, disabled
replay, selection error or required helper build keeps the ordinary order.
An absent case in the current listing resumes the ordinary order before
executing any tests. The runner prioritizes the existing process containing
the case, preserving its predecessors, the full profile and its ledger.
On a serial replay that reproduces that exact new failure, report the
observed results, defer the remaining checks and leave the pass cache empty.
Otherwise complete each selected profile once and use the ordinary verdict.
Record the case, tree, base, focused duration and elapsed time to a confirmed
failure, when observed, in the Git directory’s `temper/repair-trial-result.json`.

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
   process*). One-process runs the binaries of the layers that replay and
   the repository's checks, as at the gate, and leaves out the real-loop and
   end-to-end ones (`tools/tiers/layers.md`, 9.1). The check passes no budgets and no configuration of its own:
   the repository's `.config/nextest.toml` is read as it stands, and the
   profile's `global-timeout` is the run's only bound. one-process compares
   each layer the run measured with `tools/tiers/budgets.toml`. Where the
   check took main's ledger for the run's profile (*Main's ledgers*,
   below), it passes `--ledger LEDGER` before the profile. The run includes
   the ledger's unselected binaries in its comparison and measures new and
   flagged tests alone (`tools/tiers/budgets.md`, 5 and 8;
   `tools/tiers/references.md`, 3). Both profiles print layer budget and
   per-test CPU reference overruns as warnings. The run's exit decides:
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
     101 is a failed build. The delivery reports the compiler output and
     skips the fuzzy listing of the same test targets;
   - **any other:** a failure of the check itself, a 2 that names no
     failed build included (a refusal or an invalid run:
     `tools/tiers/design.md`, 11.1 and 11.7).

   **Other packages' binaries** that the tests run are built first: for each
   package of the tests' set that `tools/tiers/packages.toml`'s
   `[test-binaries]` lists, the check runs `cargo build -p PACKAGE --bin BINARY`
   for each binary listed, in order, since nextest builds only the selected
   packages' own. A failed build is a build failure, and no test runs.
   `smith`'s end-to-end tests run `smith-fake-claude`'s `fake-claude`
   (`smith/docs/design/testing.md`, 2.3); since they run after landing,
   in `real-smith` (`tools/tiers/layers.md`, 9.2), the build serves none
   of the check's runs, and is kept while `[test-binaries]` names it.

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
   and its `global-timeout` its only bound. It runs after the focused tests
   and their replays, including after test failures or errors of the check
   itself. A focused listing that reports a build failure prints
   `pre-pr: fuzzy tests: skipped (focused test build failed)` and leaves
   fuzzy results unobserved. An empty tests' set, a failed build of the
   other packages' binaries, or no base skips both profiles. Fuzzy tests
   reuse the other packages' binaries built for the focused run.
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

   **Main's ledgers** (`temper/docs/design/draft/test-budgets.md`, 3): a
   builder that runs goals may keep each suite's
   (`tools/tiers/budgets.md`, 7) as `default.json` and `fuzzy.json` in the
   directory `TIERS_LEDGER` names. The check reads them once, after
   `temper/rerun` and before the cache's check, so the bytes the cache
   keys are those the runs are judged by (section 7):
   - **Unset or empty:** no ledger, and one line, `pre-pr: budgets:
     warnings only (TIERS_LEDGER unset)`.
   - **Otherwise,** for `default` then `fuzzy`, the file
     `<TIERS_LEDGER>/<profile>.json`, a relative directory taken from the
     tree's root, is read whole, its bytes written to
     `<profile>-ledger.json` in the check's private temporary directory,
     and that copy read by `tools/tiers/cpu.py`'s `ledger()`. It is taken
     where `ledger()` returns and its `profile` is the profile: `pre-pr:
     budgets: <profile> warnings only with main's ledger
     (TIERS_LEDGER/<profile>.json)`. Otherwise the profile has none, and
     one line says why, `pre-pr: budgets: <profile> warnings only
     (<why>)`, `<why>` one of `no <profile>.json in TIERS_LEDGER`,
     `<profile>.json unreadable: <error>`, `<profile>.json refused:
     <error>` and `<profile>.json holds the <other> ledger`, `<error>` the
     error's first line, at most 512 bytes, cut at a character's end.
   - **The copy is what a run takes,** as `--ledger`;
     `--again` omits it, as one-process requires.
   - **Never a failure:** a missing or refused ledger refuses nothing;
     its line is the deployment's, which the summary does not repeat, as
     the hold's. A layer over at main stays a warning, as at the gate
     (`tools/tiers/budgets.md`, 8), and so does a test that swings alone
     at main, named with its figures in one-process's output
     (`tools/tiers/references.md`, 6), which the summary does not repeat:
     both are main's, not the delivery's.

5. **The docs:** `python3 -I -B tools/citations/citations.py --all
   --head <tree>` against section 3's private snapshot, including staged,
   unstaged and untracked documentation. The checker also enforces the
   design size limit for `tools/citations/sized.txt`'s trees (`DESIGN.md`,
   5). A docs-only delivery runs no formatter, counted baseline or clippy,
   and check 4 over the boundary packages alone (section 3), before this
   check. A code-only delivery runs no full citation checker; smith's
   counted `comment:dead_citation` baseline still checks Markdown path
   existence in Rust comments as part of check 2, as before
   (`tools/old_forms/README.md`).
6. **The outward names,** for a prose-only delivery alone (section 3):
   `python3 -I -B tools/outward/outward.py` from the tree's root, with
   section 3's private index as `GIT_INDEX_FILE`, so that it scans the
   delivery's tree as the snapshot holds it, untracked files included and
   ignored ones not (`tools/outward/design.md`, 2 and 3). It runs after
   check 5, announced `pre-pr: outward …` and ended `pre-pr: outward:
   passed|failed|inherited, <seconds> s`. Its exit decides:
   - **0:** passed;
   - **1:** the scan found a name: one failure, its log the scan's
     output, the verdict's lines (`tools/outward/design.md`, 3). It stands
     for the Rust test the delivery does not run, and is named and keyed
     as that test's failure would be: `test fabrig-boundary::outward
     no_file_names_the_lab`, binary id `fabrig-boundary::outward`, name
     `no_file_names_the_lab`, so that a goal's red naming that test
     inherits it (section 5) and the failures file keys it as a test
     (section 6);
   - **any other:** a failure of the check itself, `check outward`, its
     log the scan's output.
7. **The figures,** where the paths the tree changes against its base,
   `git diff-tree` as section 3 runs it before it sets merged tips'
   entries aside, include `tools/tiers/budgets.toml`, whatever else they
   hold, a delivery that selected neither set included: `python3 -I -B
   tools/tiers/figures.py --base <base> --head <tree> [--merged <rev>]…` from the tree's
   root, against section 3's private snapshot, as check 5 runs citations
   (`tools/tiers/budgets.md`, 10.1). The merged tips are the commit named by
   `git rev-parse --verify --quiet MERGE_HEAD`, where it names one, then
   every parent after the first of each commit in
   `git rev-list --first-parent --merges --parents <base>..HEAD`, deduplicated
   in that order. Each is passed as `--merged <rev>` after `--head <tree>`;
   without a merge the argv has no addition. It runs last, after every other
   selected check, announced `pre-pr: figures …` and ended `pre-pr:
   figures: passed|failed, <seconds> s`; it reads git alone, so it takes no
   hold and builds nothing, and it selects nothing else. Its exit decides:
   - **0:** passed, a delivery that changes that file alone included;
   - **1:** a figure raised or a reason's line added: one failure,
     `figures`, its log the check's output, which names each and ends
     with the repair line. It keeps only figures raised or added against
     its base and every merged tip, so it is the delivery's own, never
     inherited (section 5);
   - **any other:** a failure of the check itself, `check figures`, its
     log the check's output.
8. **The journey files,** for every delivery, with a base or none: the
   driver's own reading of git, after every other check, announced
   `pre-pr: journeys …` and ended `pre-pr: journeys: passed|failed,
   <seconds> s`. It starts no program of the check's, takes no hold and
   selects nothing. A written file is
   `temper/docs/design/client/journey-writer.md`, 2's: under
   `temper/docs/design/client/web/journeys/` or
   `temper/docs/design/client/tui/journeys/`, a journey directory's
   `recording` or a page of it, `NN.html` or `NN.txt` with two digits or
   more, or, at either root, `skein-ui.css`, `journey.js`, `theme.css`
   or `tui-journey.js`.
   - **The delivery's own paths:** with a base, section 3's changed
     paths, which already leave out each whose entry in the tree is the
     one a merged tip of check 7's holds at that path; without a base,
     the paths the tree changes against `HEAD`, staged, unstaged and
     untracked.
   - **Refused,** one failure, `journeys`: those paths holding a
     written file and any path that is not one; or a tree whose
     `overlay.css` under either root holds a byte. Its log names the
     written files, the first twenty and then a count, one other path,
     and each nonempty overlay, and ends with the repair: `a journey's
     recording, pages and the files beside them are written on main by
     the journeys' writer alone (AGENTS.md, 4): restore them with git
     checkout BASE -- PATH..., and compare pages by cargo run -p
     temper-client-walk -- print --out DIR in your own tree`, BASE the base or
     `HEAD`. It is the delivery's own, never inherited (section 5).
   - **Passed:** neither; a delivery whose own paths are written files
     alone among them.
9. **The merge job's tests,** where section 3's changed paths include
   `.temper/merge` or a path under `tools/merge/` other than its
   `README.md` and `codecs.txt`: `python3 -I -B tools/merge/test_merge.py`
   from the tree's root, after check 7 and before check 8, announced
   `pre-pr: merge …` and ended `pre-pr: merge: passed|failed, <seconds>
   s`. Its test with the real generator builds `skein-codegen` under the
   check's hold (`tools/merge/README.md`). Its exit decides:
   - **0:** passed;
   - **any other:** one failure, `merge`, its log the tests' output. It
     is the delivery's own, never inherited (section 5): it runs only
     for a delivery that changes the job.

**The environment** is the check's, with `TEMPER_INTERNAL_BUNDLE=none` set where it
is unset (`AGENTS.md`, 5), and `TIERS_LEDGER` removed, as `TIERS_HOLD` is
(below), so that no test running a check of its own reads the builder's
ledgers. Builds and tests do not inherit the snapshot's
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
listing has built the tests and before their processes start. So too the
build places an environment admits its builds to, a limit on builds at
once: one admission for the check, not one wait per cargo command, where
the hold takes them (`tools/tiers/design.md`, 3). A hold that says it
waits for them before it holds them has the check tell smith so on its
stderr, `smith-check: waiting` at the first such line and `smith-check:
admitted` once held or gone, so that the checks' bound counts only the
time the check holds its places (smith's
`smith/docs/design/domain/run.md`, 8.1, *A wait for what a check
needs*); the 1,800-second figure a deployment sets stays as it is. A cache hit
takes none, nor does a prose-only delivery, whose selection reads git
alone and which runs no cargo command (section 3). A hold that fails prints its warning, `pre-pr: hold: <why>;
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
- **Citations and design sizes** are checked on the delivery snapshot alone.
  A broken citation or an oversized design document in that snapshot refuses
  delivery. Check 5 can run when historical file contents are unavailable.

- **The base's worktree** is `<git dir>/temper-pre-pr/base`, a detached
  worktree (`git worktree add --detach --force`), made at the first
  exceeded count and kept; each later need moves it with `git checkout
  --detach --force <base>` then `git clean -ffdxq`. It holds check 2's
  base runs alone, without the private index (`GIT_INDEX_FILE`): no cargo
  runs in it, so the head's target directory and the timestamps of either
  checkout's files stay as they are. It is not made for a lint, test or
  build failure.
- **The base's target is its own.** Every program the check starts in
  the base's worktree has `CARGO_TARGET_DIR` set to `<git
  dir>/temper-pre-pr/target`, whatever the check's environment named or
  left unset: beside the worktree, so its `git clean` keeps it, made when
  first needed and kept, and never the head's. Nothing of the check
  builds there, but a build at the base, by whatever runs one, must never
  share the head's target: cargo judges an output fresh by file times,
  against dep-info paths relative to each package, so two trees of one
  workspace in one target can each take the other's outputs, and a check
  then passes or fails on the wrong build. A deployment whose cargo gives
  a workspace's paths one target keeps the setting the check makes, or
  keys this worktree to a target of its own
  (`temper/docs/design/domain/worker/README.md`, 5).
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
  base or not, it is the delivery's. The outward text check's failure is
  keyed as its Rust test's (section 4, check 6) and judged so; it is never
  run again, since no process of tests ran.
- **A build failure** (section 4's, `build` or `build fuzzy`) is always
  the delivery's, whatever the goal's reds hold; so is a failure of a
  check itself (`check clippy`, `check one-process`, `check one-process
  fuzzy`, `check outward`, `check selection`).
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
  could not judge them. The candidate's citation check and
  the goal's reds still stand.
- **Inherited never refuses:** each is named in the summary, with the
  base for a count and with the file for a goal's red, and
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
    `failures: <N> new, <F> flakes, <I> inherited; every key in
    target/temper/pre-pr-failures.json`, the file's path from the
    checkout's root, as an agent's `read` takes it in its workspace
    (smith's `smith-protocol-machine` refuses any path with a `.git`
    part, and the check names no such path). Where the file could not be
    written: `failures: <N> new, <F> flakes, <I> inherited; not written
    to target/temper/pre-pr-failures.json: <error>`, `<error>` the
    error's first line, at most 512 bytes, cut at a character's end;
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
    or an exceeded count as `inherited: old-forms <tree> <form>`;
  - past 8,192 bytes, the failures it has not room for are counted in
    one line, `[… N more failures]`, the first kept whole; the
    failures file holds each of them.
- **The failures file** holds every failure the run found, new, flake
  and inherited alike, so that a key the summary had no room for is
  still there to read once the run's directories are gone:
  - **Where:** `target/temper/pre-pr-failures.json` under the
    checkout's root, the directory `.temper/pre-pr` runs from (`git
    rev-parse --show-toplevel`): in a linked worktree, that worktree's
    root, so each worktree has its own. It is that path whatever cargo's
    target directory is (`CARGO_TARGET_DIR` does not move it), and the
    repository's `.gitignore` ignores `/target/`, so the file is never in
    the delivery's tree, the snapshot or a commit. It is outside the git
    directory because an agent's `read` refuses any path with a `.git`
    part (`smith/crates/smith-protocol-machine/src/files.rs`, `check`),
    and smith keeps that guard: an exception naming this file would put
    the application's knowledge in the kit
    (`temper/docs/design/draft/small-fixes-1116.md`, 1). It is written
    through `target/temper/pre-pr-failures.json.tmp` renamed over it,
    `target/temper/` made where it is missing, so a reader sees one
    check's file whole. A file an earlier check left in the git
    directory, at `temper/pre-pr-failures.json`, is neither read nor
    removed. temper's own files, `temper/reds`, `temper/rerun` and
    `temper/check-results`, stay in the git directory (sections 5 and 6).
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
      for any other failure (`format`, `build`,
      `check <name>`, an old-forms run's failure, citations), whose
      name says it whole.

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

  CPU warnings appear in the streamed run output. A cache hit writes no record, nor does a
  check that wrote no summary.

## 7. The passed-tree cache

`<git dir>/pre-pr-passed` holds `<tree> <base> <reds> <ledgers>` once every check
passed or was inherited: `<base>` is `-` without a base, `<reds>` the
SHA-256, in lowercase hexadecimal, of the bytes of `temper/reds` the run
read (section 5), or `-` where it read none, and `<ledgers>` `-` where
the check took no ledger (section 4, *Main's ledgers*), and otherwise the
SHA-256 of the text `default <D>\nfuzzy <F>\n`, `<D>` and `<F>` the
SHA-256 of the bytes of the ledger it took for that profile, or `-`. A
check of the same tree, on the same base, with the same reds and ledgers
prints `pre-pr: passed before (tree <tree>, base <base>, reds <reds>,
ledgers <ledgers>)` and exits 0 at once: within a hold the
base does not move, so a `finish` repeats its `deliver`'s pass, while a
red ended or recorded since changes the file and so the key, and a pass
under a red that has ended is never reused; a ledger replaced or newly
kept changes it too, and the delivery is judged again. A failure, an interruption or
a timeout never writes it. An earlier record of another shape, the
two- and three-field ones included, matches nothing. A cache hit observes nothing:
it writes no `temper/check-results` (section 6), so temper keeps what the
check's last pass reported. It does write the failures file (section 6),
saying so: `cached` `true`, `verdict` `passed`, the tree and base it
reused, zero counts and no failures, since it observed none; after its
line it prints `failures: a cached pass, none observed; in
target/temper/pre-pr-failures.json`, or `…; not written to
target/temper/pre-pr-failures.json: <error>` as the summary's failures
line does.
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
- **A prose-only delivery builds nothing** (section 3): in a checkout
  with no `target/`, the boundary packages' tests were a cold compile,
  and a goal delivering only status files ran past the checks' deadline
  three times (`temper/docs/design/draft/small-fixes-1116.md`, 3). It
  costs citations and the outward text check, which reads each tracked
  file once, a few seconds on this repository and no cargo.
- **What main's ledgers cost** a delivery whose builder keeps them: each
  run's new and flagged tests run alone after its suite, each in a
  process of its own beside its binary's empty one, and twice more each
  where the first passes its limit (`tools/tiers/references.md`, 3 and
  4), within the profile's
  `global-timeout`; and a pass
  reused only under the ledgers it was judged by, which the runner
  replaces after each landing on main (section 7).
- **No build or test at the base.** A lint, test or build failure goes
  back to the delivery at once, judged against the goal's reds alone
  (section 5): a failing delivery takes no longer than a passing one. The
  base's worktree is made only for an exceeded count, holds no build, and
  is kept for the next.
- What a check took is the run's: smith tells its host when checks start,
  with their deadline, and when they end (run.md, 8.1), and the check
  prints each check's seconds. The checks' deadline is the deployment's
  `check_timeout`, which the owner sets separately. It counts the time the
  check holds its build places alone: a wait its hold says it makes for
  them is told to smith and left out (section 4, *The hold*), so a check
  queued behind other workspaces' builds is not refused for their time.

## 9. Testing

### 9.1 The worker entry point

Offline acceptance runs invoke `.temper/pre-pr` in a prepared workspace with
its real local pin. A fixable formatting error is corrected before publication. A clean import
from a merged branch stays untouched across publication and finish. A Rust
parse error refuses delivery without applying a partial formatting patch.
A formatted Rust
change with a failing assertion must pass the light checks without a build,
then fail focused testing at goal close even if settlement changes no files.
The target is two minutes on a typical warm delivery.

### 9.2 The driver's unit tests

Run `python3 -I -B tools/pre-pr/test_pre_pr.py`. The suite uses recording
fakes for external commands. It checks target selection, checkout and receipt
identity, refusals and the Gate decision. The target for the entire suite is
ten seconds. The tests use the same policy functions as the worker.

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
- **What a merge's two sides make together.** A merge selects what it
  changes against both parents (section 3). Where the base is the
  merge's first parent (a repair, a resolution, a merge job: section 2),
  a package of either side that the other side's change breaks, with no
  path changed against both, is selected by neither and left to the
  landing's gate, which runs every package. A change that merged its
  landing branch's newer tip still selects its own packages, which the
  tests then run in the merged tree.
