# forge-setup: temper's forge, set up and boxed

2026-10-16. Start commits: temper `e40485aa7` ("Defer the forge path
story until segment bounds are available"; the findings were read at
`0ff45022`, and every command of this plan ran once at `e40485aa7`,
`findings/base-runs.md`), skein `04c1562` ("Seed Hmac's two states;
keep SHA-256's sweeps in the fuzzy suite"; its crates equal
`ec686b8a`). Each session starts from its
repository's `origin/main`. skein and temper change.

temper gets the forge's setup of `setup.md`: the owned Forgejo prepared,
run and reconciled by `temper forge prepare|run|reconcile`, `temper box`
wiring them into one throwaway environment, and webhooks over loopback
that hint the engine, with polling as the fallback. skein's lib gets
SHA-256, `sha256::Hmac` and `sha256::equal` (`lib.md`, 6.2). The process
tier moves onto an owned Forgejo with CI and webhooks, and the first
milestone gets its setup guide. The box comes first, on polling; webhooks
join it (decisions.md, 1).

## Inputs

- **temper's design,** at the start commit: `setup.md` (the whole, in
  its section 14's order); `forge.md`, sections 5, 7, 9 and 13;
  `configuration.md`, sections 3, 6, 7 and 10; `domain/testing.md`, 2.6;
  `domain/forge.md`, 5.2 and 8.2; `credentials.md`, 2 and 5;
  `protocol.md`, 5; `draft/mvp.md`, 5.1.
- **skein's design:** `lib.md`, 6.2 and 10.
- **The workflows:** temper's `docs/development/workflow.md`, sections 1
  and 2; skein's `scripts/check.sh`.
- **`decisions.md`:** the plan's decisions, and what the user may veto.
- **`findings/`:** the code audit at the base, one file per area:
  `skein-sha256.md`, `skein-io.md` (what skein gives the hosts),
  `skein-main-in-temper.md` (temper on skein's `main`: it builds and
  passes its gate), `forge-and-tiers.md` (the Forgejo crate, the
  throwaway, the conformance, the process tier), `webhooks.md`,
  `configuration-and-binary.md`, `client-api.md` (the run beside this
  one, its hotspots), `measurements.md` (the gate and the suites at the
  base), and `base-runs.md` (each command of this plan, run once at the
  base).
- **`code-organization.md`:** crates, names, owners, fixtures, the
  "Don't" list, sizes and the count.
- **`later.md`:** what the design names that this plan does not build.

## The sessions

A session waits for what "Needs at start" names; an increment that needs
more says so in its row and its file. Dependencies are by capability:
"temper 07.1" means that increment has landed on `origin/main`.

| File | Session | Needs at start | Increments that need more |
|---|---|---|---|
| `skein/01-sha256.md` | skein 01: SHA-256, `Hmac` and `equal` in skein's lib; skein's three private copies moved onto it | nothing | |
| `temper/01-users-tokens.md` | temper 01: the codec's gaps; admin users, and the bot's tokens on the admin's routes, both sides, with their facts | nothing | |
| `temper/02-orgs-hooks.md` | temper 02: orgs, teams, members and org hooks, both sides, with their facts | temper 01.2 (the codec's gaps, the conformance's setup recorder) | |
| `temper/03-command-line.md` | temper 03: the pin in temper-forge-forgejo; the facts of Forgejo's command line and the runner | nothing | |
| `temper/04-deliveries.md` | temper 04: the deliveries' facts, and their documents both sides | nothing | |
| `temper/05-webhooks.md` | temper 05: temper on skein's `sha256`; signed deliveries from the fake; the component's listener; `application.webhooks` | skein 01.2 (`sha256::Hmac`, `equal`), temper 04.2 (the deliveries' documents) | 05.3: client-api's temper 05.1 and 05.2 (temper-service's listeners and `application.clients`) |
| `temper/06-settle.md` | temper 06: the fake's hooks as Forgejo's; `settle` | temper 04.1 (the deliveries' timing facts) | |
| `temper/07-prepare.md` | temper 07: temper-forge-setup; Forgejo's files; `forges[].user` as a file and `application.setup`; the preparation's machine | nothing | 07.4: temper 03.3 (the `cli-…` and `runner-…` facts) |
| `temper/08-run.md` | temper 08: `temper forge prepare` and `run`; the process tier's Forgejo started by them | temper 07.3 (the configuration), 07.4 (the preparation's machine) | |
| `temper/09-reconcile-machine.md` | temper 09: reconcile's machine, and its world on the fake forge | temper 01.2 (admin users), temper 07.2 (the crate, `Setup`, `place::File`) | 09.2: temper 01.3 (tokens); 09.3: temper 02.2 (org hooks), 04.2 (the listener's path) |
| `temper/10-reconcile.md` | temper 10: `temper forge reconcile`; the process tier on owned, with CI | temper 08.3 (`temper forge run`, the tier's Forgejo by it), temper 09.3 (the whole machine) | |
| `temper/11-hinted.md` | temper 11: webhooks on the process tier; the hinted story | temper 05.3, 06.2, 10.2 | |
| `temper/12-box-machine.md` | temper 12: temper-box's machine and its host | temper 07.1 (`Step`, `steps`) | |
| `temper/13-box.md` | temper 13: `temper box`, and its test | temper 12.2, 10.1 | 13.1's Proof: temper 10.2 |
| `temper/14-example.md` | temper 14: the owned example, and the setup guide | temper 05.3, 13.1 | |
| `temper/15-upgrade.md` | temper 15: the upgrade | temper 08.2 | |

16 sessions, 40 increments; the increments per session are in each
file.

**How many at once:** as many as the dependencies and the machine allow.
At the start five need nothing: skein 01, temper 01, 03, 04 and 07.
When slots are short, start them in this order, first what others wait
on: temper 07 (08, 09, 12 wait on it), temper 01 (02, 09), temper 03
(07.4), skein 01 and temper 04 (05, 06); then, as they become ready, 09,
08, 12, 02, 10, 13, 05, 06, 11, 14, 15.

**The critical path,** the box on polling: temper 07.1 → 07.2 → 07.3 →
07.4 (with temper 03.3) → 08.1 → 08.2 → 08.3, beside temper 01.1 → 01.2 →
09.1 → 09.2 → 09.3 (with 02.2 and 04.2); then 10.1 → 13.1 (with 12.2).
The webhooks' path: skein 01.1 → 01.2 and temper 04.2 → temper 05.1 →
05.2 → client-api's temper 05.1 and 05.2 → temper 05.3 → 11.1 → 11.2.

## Rules of this plan

- **The binaries.** The Forgejo is
  `TEMPER_FORGEJO=$HOME/src/rust/tmp/temper/forgejo/v17.0.0/forgejo-17.0.0-linux-amd64`,
  the runner
  `TEMPER_FORGEJO_RUNNER=$HOME/src/rust/tmp/temper/forgejo/runner-v13.2.0/forgejo-runner-13.2.0-linux-amd64`.
  No other is used, and none is downloaded.
- **client-api runs beside this plan,** on carbon-forge, on the same
  temper `main` (`findings/client-api.md`). Its hotspots, each free once
  its last step there has landed:
  - temper-service's `startup/application.rs`, `startup/checks.rs`,
    `startup.rs`, `settings.rs`: client-api's temper 05.2 (about 10-18
    14Z). This plan's 05.3 and 07.3 touch them;
  - the listeners' plumbing, temper-service's `system.rs`, `engine.rs`,
    `lib.rs`, `limits.rs`: client-api's temper 05.1 (about 10-18 13Z);
    this plan's 05.3 hosts its listener on what that step lands;
  - `crates/temper/src/lib.rs`: client-api's temper 05.2. This plan's
    07.3, 08.1, 08.2, 10.1 and 13.1 touch it;
  - `crates/temper/tests/process.rs` and `tests/support/*`: until about
    10-18 20Z. This plan's 05.3, 07.3, 08.1, 08.2, 08.3, 10.1, 10.2, 11.1,
    11.2, 13.1, 13.2, 14.1 and 15.1 touch them.
  A rebase conflict there is resolved before merging. client-api leaves alone temper-forge-forgejo, the fake
  forge, the throwaway, the conformance tool,
  temper-engine-protocol-forge, `tests/engine/forge` and new crates.
- **`Cargo.lock` is never merged by hand:** a conflict takes
  `origin/main`'s, and the step's `cargo update` runs again. temper's
  skein moves once, in temper 05.1, by `cargo update -p skein-lib
  --precise <skein's origin/main>` in a commit of its own
  (`findings/skein-main-in-temper.md`); the three new crates add their
  own entries in the commits that create them.
- **skein's gate on x1** is `scripts/check.sh` with
  `SKEIN_TEST_CHROMIUM` exported (`~/.profile` sets it; a background
  session sources it first), and the browser test
  `a_page_of_another_origin_is_opened_by_its_step` filtered: it fails on
  x1 alone, and is client-api's skein 01. From the step's worktree, after
  `. ~/.profile` (run once at the base: `findings/base-runs.md`):

  ```sh
  bash -c 'mkdir -p target && sed "/--profile browser\$/s/\$/ -E \"not test(=a_page_of_another_origin_is_opened_by_its_step)\"/" scripts/check.sh > target/check-x1.sh && sh target/check-x1.sh'
  ```
- **The recording tools** (the conformance tool) run where an
  increment's Shape or Proof says, once, and once
  more after each fix aimed at a cause their output names; the third
  failure with the same symptom is a question (development.md, 5.5).
  Transcripts and `facts.txt` are written by the tools only; a new
  capture is appended after the existing ones.
- **Byte-stable recordings (rulings 121 and 124).** A recording of a
  group that changes a capture of an earlier recording is fixed in the
  tool, not accepted, without asking: a creation whose answer carries a
  timestamp is made in a second of its own, so that the mask's
  first-appearance rule aliases no earlier timestamp; a set-valued array
  that Forgejo orders at random (the deprecated `units`) is sorted by
  the mask; an id the mask misses (a pkt-line's) is masked. A fix to
  the shared mask (`tools/forgejo-conformance/src/mask.rs`) lands in the
  increment that needs it, named in its commit body; a session whose
  captures a landed fix changes refreshes them in its next landing. The
  check, once per increment that adds captures: after a second recording
  in a row, `git status --short tools/forgejo-conformance/transcripts
  tools/forgejo-conformance/facts.txt` prints nothing. A difference
  these fixes do not cover is a question. This takes the place of the
  session files' "Byte-stability" question, until the run ends.
- **Sizes (rulings 102 and 125).** An increment of this plan lands
  within 3.5 times its Size without asking, a Size marked "(codec)"
  within 4.5 times; a ruling that gives an increment its own number
  keeps it where it is larger. Its report gives the count by
  code-organization.md 6's command, by file. Past that, it stops on a
  question, as the implementation rules says. This takes the place of 8.6's
  twice and 2.5 times, and of ruling 102's 2.6 and 3 times, for every
  session of this plan, until the run ends. Unchanged: never trim a doc comment,
  a test or the formatting to meet a Size; a split increment's parts
  count together.
- **The component fixture's HTTP 500 (ruling 114).** The conformance
  tool's `Fixture::new` met `provisioning /api/v1/repos/fixture/specimen/pulls:
  HTTP 500` in 04.1's and 01.2's recordings, in the `--component` phase,
  after passing before. temper 01.2 makes `Fixture::api`
  (`tools/forgejo-conformance/src/fixture.rs:84`) keep the answer's body,
  at most 200 bytes, in its error, and `Fixture::pull` (:98) try an HTTP
  500 again, at most 3 times, 1 s apart, each try's status and body on
  stderr. Until that lands, a recording that fails so is run once more,
  unchanged, without asking; a second such failure is a question, with
  the body.
- **The process tier runs where a Proof line or "Done when" says,**
  once; never in the gate.
- **The suites' aims for the whole run** (decisions.md, 13): temper's
  default suite at the base takes 4.43 s and its fuzzy suite 24.22 s
  (medians of three on x1, `findings/measurements.md`); the run adds at
  most 1.0 s to the first and 3.0 s to the second. skein's default suite
  takes at most 0.5 s more. No session has a share.

- **One behaviour per test, and a literal expectation (ruling 140; seen at 23:03Z, 02:28Z and 11:19Z).** A test
  asserts one behaviour, and its expected value is a literal or a named size, never one the code under test returns.
  The hourly check reads each landing's new tests for it.
- **Named sizes, and both sides of a bound (ruling 140; seen at 00:30Z, 02:28Z and 11:19Z).** A size, a count, a
  limit or an id in a test is a named constant with its derivation beside it (the code's constant, the fixture's
  content, or the design's section). A limit is tested at the bound, which holds, and one past it, which does not, each
  in a test or row of its own. Landed tests that no increment touches stay.
- **A setup written once (ruling 148; seen at 11:19Z and 12:37Z).** When two tests, or two helpers, would repeat
  three lines or more (a parse and its assertions, a lay and a write of `config.json`), those lines are one named
  function that takes what differs, and each test keeps its own name and its own further assertions. The hourly check
  reads each landing's new tests for it. Landed tests that no increment touches stay.
- **A shared support function changed, its users run (ruling 148; seen at 12:37Z).** An increment that changes a
  `support` function that other stories call (`forgejo::configuration`, `owned::application`) lists each override or
  member it drops or moves in its report, one line each, and runs the process tier once before landing, with `--no-fail-fast`; a story red on
  `main` for a known cause (ruling 139's landing story) stays red, its cause named in the commit body, and the
  increment lands without waiting for the fix (ruling 149).

## Prerequisites the run checks at its start

The coordinator runs each once, before the first session, and writes
`blocked.md` (kind: process) on a failure:

1. `cd ~/src/rust/tmp/temper/forgejo/v17.0.0 && sha256sum -c
   forgejo-17.0.0-linux-amd64.sha256` prints `OK`.
2. `cd ~/src/rust/tmp/temper/forgejo/runner-v13.2.0 && sha256sum -c
   forgejo-runner-13.2.0-linux-amd64.sha256` prints `OK`.
3. `cargo nextest --version` prints a version.
4. `test -n "$SKEIN_TEST_CHROMIUM" && "$SKEIN_TEST_CHROMIUM" --version`
   prints `Chromium 147.0.7727.101`, after `. ~/.profile`.

## One test of a suite that needs a built artifact

None. skein 01.3's bundler is checked by its golden tests and by
the gate, which builds the bundle and runs the browser suite whole;
01.3 runs no browser test of its own.

## Live services

None. Forgejo and its runner are the pinned local binaries on loopback;
the process tier's LLM is skein's fake peer; no agent signs in
(decisions.md, 16). The guide's `temper sign-in` is the person's, and
temper 14.2's check stops before it.

## Repin

temper's alone, in temper 05.1 (decisions.md, 2 and 3): skein 01 adds
`skein_lib::sha256` and moves three private copies onto it, keeping every
public item, so nothing downstream breaks. smith (smith/wall-clock runs
on hetzner), jig (its pins are client-api's to move) and hearth (its M1
plan, paused, takes skein at its next repin) take it at their next
repins: `later.md`, and each repository's `STATUS.md`, under "Not
planned", in the designer's next landing (development.md, 5.7).

## Pulse

```
words: actual|genuine|legacy|compat|workaround
types: ^(Legacy|Old|Compat)[A-Z]|V2$
file-lines: 800
budgets: 15 60
```

## Done when

Each command runs once, after `done.md`, in a worktree of temper (or
skein, for skein's) at `origin/main`, the plan's worktrees removed. A
session file's item that runs the process tier cites item 4's run
instead of running it again; a count of tests in a suite that later
sessions grow is read as "at least".

1. Every step is landed or dropped: `plan status` lists none waiting,
   ready, running or blocked.
2. The plan's commits: `git log origin/main --grep='^Plan: forge-setup$'
   --format='%(trailers:key=Step,valueonly,separator=)' | sort -u` in
   temper lists the 37 temper steps the session files number
   (`temper-01.1` to `temper-15.1`), and the same in skein lists
   `skein-01.1`, `skein-01.2`, `skein-01.3`. A supplementary step
   (`.s<k>`) adds a line; a step dropped by a ruling is named in
   decisions.md's rulings instead.
3. Each session file's "Done when" holds: every item of each of the 16
   files, run as written, in the sessions table's order.
4. The box, on polling and with webhooks, end to end: in a worktree of
   temper at `origin/main`,

   ```sh
   export TEMPER_FORGEJO=$HOME/src/rust/tmp/temper/forgejo/v17.0.0/forgejo-17.0.0-linux-amd64
   export TEMPER_FORGEJO_RUNNER=$HOME/src/rust/tmp/temper/forgejo/runner-v13.2.0/forgejo-runner-13.2.0-linux-amd64
   scripts/proof-one-process.sh
   ```

   exits 0, within 180 s
   (domain/testing.md, 2.6; decisions.md, 12), the hinted story (temper 11.2),
   the box's tests (temper 13) and the upgrade's (temper 15) among its
   tests. The session files' process-tier items read this run.
5. The conformance's facts: `git show
   origin/main:tools/forgejo-conformance/facts.txt | grep -E
   '^(fails|unchecked) '` prints the four `fails` lines it printed at the
   base (`pull-for-names-the-newest-of-several`,
   `open-pull-no-changes-is-422`, `delete-missing-branch-is-404`,
   `effective-protection-name-for-a-writer`), one `fails delivery-…`
   line for each of temper 04's two pairs (the loser), at most
   `fails delivery-branch-deletion-sends-no-push` besides (temper 04,
   04.1), and nothing else: the base's `unchecked webhook-deliveries` is
   gone (temper 04.1); and each `holds` count by prefix that the "Done
   when" of temper 01 to 04 gives still holds.
6. temper's skein holds `sha256::Hmac`: `git -C ~/src/rust/skein
   merge-base --is-ancestor $(git -C ~/src/rust/skein log origin/main
   --all-match --grep='^Plan: forge-setup$' --grep='^Step: skein-01.2$'
   --format=%H -1) $(grep -o 'forgetic/skein.git#[0-9a-f]*' Cargo.lock
   | sort -u | cut -d'#' -f2)` exits 0, and that `grep` prints one
   revision.
7. The test-time aims, measured once in a worktree of temper at
   `origin/main`: `sh -c 'export
   NEXTEST_DOUBLE_SPAWN=0; for i in 1 2 3; do cargo nextest run
   --workspace 2>&1 | grep Summary; cargo nextest run --workspace
   --profile fuzzy 2>&1 | grep Summary; done'`; the medians of the
   three `Summary` times per suite, against the aims (Rules), on top of
   what client-api's commits added since the base: the same command at
   the parent of this plan's first temper commit (`git log origin/main
   --grep='^Plan: forge-setup$' --format=%H | tail -1`) gives that
   reference, measured the same way, once. skein's default suite the
   same way, in skein, against skein's parent of `skein-01.1`. An aim
   missed is recorded with the three slowest tests by name in
   decisions.md's rulings, for the user; the run is done either way.
8. No budget was raised by this plan: `git log origin/main
   --grep='^Plan: forge-setup$' -p -- .config/nextest.toml` prints
   nothing, in temper and in skein.
9. The aims above are these items; none other is checked.
