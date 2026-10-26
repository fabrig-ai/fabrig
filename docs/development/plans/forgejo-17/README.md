# forgejo-17: temper on Forgejo v17.0

2026-10-16. Start commit: temper `118f7fb7` ("Retarget the forge's
design to Forgejo v17.0 and forgejo-runner v13.2"); each session starts
from temper's `origin/main`. Only temper changes.

temper moves from Forgejo v16.0.5 and forgejo-runner v12.13.2 to Forgejo
17.0.0 and forgejo-runner 13.2.0, the release `forge.md` now requires,
and takes what v17 adds that the design adopts: a failed job is read at
its failed step, whose log's end goes into the repair's brief and the
agent's `job` tool, and comparisons ask only for what they keep. The
throwaway Forgejo, `tools/forgejo-conformance` and `tools/git-transcripts`
record their transcripts and facts again on v17, with the facts
`forge.md`, section 13 lists to check; the fake forge's jobs get steps.

## Inputs

- **temper's design,** at `118f7fb7`: `docs/design/STATUS.md`, the rows
  naming "the next plan"; `forge.md`, sections 1, 3.2 (the rows
  `Read::Compare`, `Read::Job`, `Write::OpenPull`, and the notes "Job
  attempts", "A failed job's log is its failed step's end",
  "Comparisons ask for what they keep"), 3.4, 9, 10 and 13;
  `domain/forge.md`, sections 2 ("Tools", "Brief sections"), 8.4 and
  20.1; `git.md`, sections 9 and 10; `domain/testing.md`, 2.6.
- **The workflow:** temper's `docs/development/workflow.md`, sections 1
  and 2.
- **`decisions.md`:** the plan's decisions, and what the user may veto.
- **`findings/`:** the code audit at the base, one file per area
  (`tools.md`: the pin and the two recording tools; `component.md`: the
  component's job read and comparisons; `answer.md`: the job answer's
  users, the store, the bounds; `fake-forge-and-brief.md`), and
  `measurements.md` (the suites at the base, and the v17 facts probed
  while planning).
- **`later.md`:** what the design names that this plan does not build.

## The sessions

A session waits for what "Needs at start" names; an increment that needs
more says so in its row and its file. Dependencies are by capability:
"temper 02.1" means that increment has landed on `origin/main`.

| File | Session | Needs at start | Increments that need more |
|---|---|---|---|
| `temper/01-pin.md` | temper 01: Forgejo 17.0.0 and runner 13.2.0 pinned; the conformance's and git's transcripts and facts recorded on them | nothing | 01.2: 01.1 |
| `temper/02-documents.md` | temper 02: v17's documents: a job with its steps, a step's log by a suffix range, the comparisons' flags, and their facts | temper 01.1 (the pin) | 02.2: 02.1 (its captures follow 02.1's) |
| `temper/03-fake-jobs.md` | temper 03: the fake forge's jobs with steps, their logs by step, suffix ranges | temper 02.1 (the job document, `Request::Job`, `JobLog`'s step) | 03.2's Proof: 03.1 |
| `temper/04-job-read.md` | temper 04: `Read::Job` answers the failed step and its log's end | temper 02.1, temper 03.1 (the fake serves a job's steps) | 04.2: temper 03.2 (the fake's suffix ranges) |
| `temper/05-brief.md` | temper 05: the repair's brief names the failed step and its log's end | temper 04.1 (the answer names the step) | |
| `skein/00-s1-print-room.md` | skein-00.s1 (ruling 104): skein's print takes its room from its own tree | nothing | |
| `temper/01-s1-print-room.md` | temper-01.s1 (ruling 104): temper's skein moved onto it; the full process tier | skein-00.s1 | |

5 sessions, 9 increments; the increments per session are in each file.

**How many at once:** at most three, by the dependencies: temper 01.2
(git's transcripts) beside temper 02; then temper 03 and temper 04 once
temper 03.1 has landed. The order is the table's. temper 02.1 (the job
document) comes first in its session, since temper 03 and 04 wait on
it, and 02.2 (the comparisons' flags) lands after it.

## Rules of this plan

- **The binaries.** The throwaway Forgejo is
  `TEMPER_FORGEJO=$HOME/src/rust/tmp/temper/forgejo/v17.0.0/forgejo-17.0.0-linux-amd64`,
  the runner
  `TEMPER_FORGEJO_RUNNER=$HOME/src/rust/tmp/temper/forgejo/runner-v13.2.0/forgejo-runner-13.2.0-linux-amd64`
  (decisions.md, 2). No other Forgejo or runner is used, and none is
  downloaded.
- **The recording tools run only where an increment's Shape or Proof
  says,** once, and once more after each fix
  aimed at a cause their output names; the third failure with the same
  symptom is a question (development.md, 5.5). A recording's
  `facts.txt` is read after the run: a `fails` line still exits 0.
  Transcripts and `facts.txt` are written by the tools only, never by
  hand; a new capture is appended after the existing ones, since masking
  numbers ids and times in order of appearance (`findings/tools.md`, 2).

- **The store's version does not move** (decisions.md, 4):
  `STORE_VERSION` stays as `origin/main` has it, and a record changes in
  place in the schema `origin/main` holds (`ls
  crates/temper-protocol-store/schema/` names it). Checked: "Done when",
  item 8.
- **client-api runs beside this plan,** on carbon-forge, on the same
  `main` (decisions.md, 3). Its temper sessions change
  `crates/temper-protocol-store/` (the schema moved to `store-v2.schema`),
  `crates/temper-engine-domain/`, `tests/engine/domain/` and the
  pins in `Cargo.toml` and `Cargo.lock`. A rebase conflict is resolved
  before merging; a conflict in
  `crates/temper-protocol-store/schema/` takes `origin/main`'s file and
  applies this increment's change to it, then regenerates (the store's
  drift test, `findings/answer.md`, 2). This plan changes no
  dependency but temper-01.s1's skein move (ruling 104): a conflict in
  `Cargo.lock` takes `origin/main`'s, and temper-01.s1 moves again.
- **The process tier runs where a Proof line or "Done when" says,**
  once; never in the gate.
- **The suites' aims for the whole run** (development.md, 5.6): the
  default suite at the base takes 5.25 s, the fuzzy one 26.6 s (medians
  of three on x1, `findings/measurements.md`); the run adds at most
  0.5 s to the first and 1.5 s to the second. No session has a share:
  each is smaller than one gate's swing on x1 (about 0.15 s and 1.4 s).

## Prerequisites the run checks at its start

The coordinator runs each once, before the first session, and writes
`blocked.md` (kind: process) on a failure:

1. `cd ~/src/rust/tmp/temper/forgejo/v17.0.0 && sha256sum -c
   forgejo-17.0.0-linux-amd64.sha256` prints `OK`.
2. `cd ~/src/rust/tmp/temper/forgejo/runner-v13.2.0 && sha256sum -c
   forgejo-runner-13.2.0-linux-amd64.sha256` prints `OK`.
3. `git --version` prints `git version 2.53.0` (git's transcripts and
   their tests name it, `findings/tools.md`, 4).
4. `cargo nextest --version` prints a version.

## One test of a suite that needs a built artifact

None: no suite of temper's needs an artifact the gate builds first.

## Live services

None. The throwaway Forgejo and runner are local programs on loopback,
started and removed by the tools; no agent signs in anywhere, and no LLM
is called.

## Repin

None: temper is the last repository of the workspace's order. The plan
changes skein in one step, skein-00.s1, and temper takes it in
temper-01.s1's `Cargo.lock` alone (ruling 104); it changes no other
(decisions.md, 12).

## Pulse

```
words: actual|genuine|legacy|compat|workaround
types: ^(Legacy|Old|Compat)[A-Z]|V2$
file-lines: 800
budgets: 15 60
```

## Done when

Each command runs once, after `done.md`, reading `origin/main`, in
`~/src/rust/temper` unless it says otherwise.

1. Every step is landed or dropped: `plan status` lists none waiting,
   ready, running or blocked.
2. The pin: `git show
   origin/main:testing/temper-throwaway-forgejo/src/binary.rs | grep -c
   -E 'VERSION: &str = "(17\.0\.0|13\.2\.0)"'` prints 2, and `git grep
   -n -e '16\.0\.5' -e '12\.13\.2' origin/main -- crates testing tools
   tests` prints nothing.
3. The conformance's facts are v17's: `git show
   origin/main:tools/forgejo-conformance/facts.txt | head -2` names
   `17.0.0` and `v13.2.0`; `git show
   origin/main:tools/forgejo-conformance/facts.txt | grep -E '^fails '`
   prints the four lines it printed at the base
   (`pull-for-names-the-newest-of-several`, `open-pull-no-changes-is-422`,
   `delete-missing-branch-is-404`, `effective-protection-name-for-a-writer`),
   and nothing else.
4. The facts `forge.md`, section 13 lists to check hold: `git show
   origin/main:tools/forgejo-conformance/facts.txt | grep -c -E
   '^holds (job-steps-number-from-0-set-up-first-complete-last|failed-workflow-step-and-completion-step-are-failure|step-log-holds-that-steps-lines-alone|step-log-suffix-range-answers-206-with-its-last-bytes|step-log-suffix-content-range-names-its-first-byte|empty-log-suffix-range-answers-206-empty|step-past-the-last-is-404|compare-files-false-lists-no-commits-files|compare-files-false-keeps-its-own-files-empty|compare-verification-false-changes-nothing-kept|compare-files-false-keeps-total-commits-and-shas)$'`
   prints 11 (temper 02's "Done when", item 2).
5. The component's facts on v17 hold: `git show
   origin/main:tools/forgejo-conformance/facts.txt | grep -E
   '^(fails|unchecked) component-'` prints nothing, and `grep -c
   '^holds component-'` prints at least 40 (39 at the base, and temper
   04.2's `component-read-job-answers-the-failed-steps-end`).
6. git's facts are v17's: `git show
   origin/main:tools/git-transcripts/facts.txt` names `17.0.0` in its
   header, and prints `holds git-follows-a-renamed-repository-redirect`
   and `holds protected-push-is-remote-rejected-declined`, and no
   `fails` line.
7. The answer and the brief name the failed step: `git grep -n -F
   'step: Option<Step>' origin/main --
   crates/temper-engine-domain-forge-client/src/api.rs` prints a line;
   `git grep -n -F 'Job log tail' origin/main -- crates` prints nothing.
8. The store's version did not move in this plan: `git log origin/main
   --grep='^Plan: forgejo-17$' -p --
   crates/temper-protocol-store/src/lib.rs | grep -E
   '^[-+].*STORE_VERSION'` prints nothing.
9. The process tier passes on v17, in a worktree of temper at
   `origin/main`:
   `TEMPER_FORGEJO=$HOME/src/rust/tmp/temper/forgejo/v17.0.0/forgejo-17.0.0-linux-amd64
   scripts/proof-one-process.sh` exits 0.
10. The plan's commits: `git log origin/main --grep='^Plan:
    forgejo-17$' --format='%(trailers:key=Step,valueonly,separator=)' |
    sort -u` lists the 9 steps the session files number: `temper-01.1`,
    `temper-01.2`, `temper-02.1`, `temper-02.2`, `temper-03.1`,
    `temper-03.2`, `temper-04.1`, `temper-04.2`, `temper-05.1`. A supplementary step (`.s<k>`) adds a line; a
    step dropped by a ruling is named in decisions.md's rulings instead.
11. The test-time aims, measured once in a worktree of temper at
    `origin/main`: `sh -c 'export
    NEXTEST_DOUBLE_SPAWN=0; for i in 1 2 3; do cargo nextest run
    --workspace 2>&1 | grep Summary; cargo nextest run --workspace
    --profile fuzzy 2>&1 | grep Summary; done'`; the medians of the
    three `Summary` times per suite, against the aims (Rules) on top of
    what client-api's commits added since the base: the same command at
    the parent of this plan's first commit (`git log origin/main
    --grep='^Plan: forgejo-17$' --format=%H | tail -1`) gives that
    reference, measured the same way, once. An aim missed is recorded
    with the three slowest tests by name in decisions.md's rulings, for
    the user; the run is done either way.
12. No budget was raised by this plan: `git log origin/main
    --grep='^Plan: forgejo-17$' -p -- .config/nextest.toml` prints
    nothing.
13. The aims above are these items; none other is checked.
