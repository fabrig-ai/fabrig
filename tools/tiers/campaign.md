# Campaigns

Provisional, 2026-11-27. `tools/tiers/campaign` runs a tree's fuzzy sweeps after landing with fresh seeds, in rounds of the tree's campaign pair, replays each failing case alone, marks each finding the tree's last campaign also had, and writes what it found for the user to route. Written from `temper/docs/design/draft/fuzzy-campaigns.md`, 5 and 7. The cases and the lines it reads are skein's seam (`skein/docs/design/testing/sweeps.md`, 4.2 to 4.5); the pair is `tools/tiers/layers.md`, 9.3; a round's CPU rows are `tools/tiers/budgets.md`, 4.2; each run under it is `tools/tiers/run --cpu`'s (`tools/tiers/design.md`, 7 and 10.3).

## 1. In one page

- **A campaign is rounds** (section 2). A round runs the tree's pair `campaign-TREE` once, with one campaign seed, so that every case of every adopted sweep is fresh. Its seed and its commit give the same cases again.
- **A round is read from its report** (section 3): the seam's lines in each test's output give its failing cases, its cases' count and its sweeps' reach; a failed test with no case line is a failure outside a case.
- **Each failing case is replayed alone** (section 4.1), through the same pair at the same commit, with one seed: failing again, it is a finding's case; passing alone, its world doesn't replay.
- **Against the last campaign** (section 4.2): with `--last`, each finding is marked as also in the last campaign, at the commit it ran at, where that campaign's `campaign.json` holds a finding of its signature, and as not in the last campaign otherwise. Nothing is checked out, built or run at that commit.
- **Findings by signature** (section 5): the cases that failed again, one finding for each binary, test and first line with its digits read as `N`, with its title and body for filing by hand.
- **Its files** (section 6): `campaign.json` and `summary.txt` in the campaign's directory, beside each round's and each replay's run directory, rewritten after each step so that a campaign cut short keeps its finished rounds.
- **Its cost is reported, never judged** (section 7): each round's layers against the tree's fuzzy ceilings, as warnings.
- **After landing alone** (section 8): once a day on a builder apart from the gate, by hand until a runner runs it. Never in the gate or the delivery check.

## 2. The command and its rounds

`tools/tiers/campaign TREE [--rounds N] [--seed SEED] [--out DIR] [--last DIR]`, from the checkout root.

- **TREE** is `skein`, `smith`, `jig`, `temper` or `hearth`, and its pair `[profile.campaign-TREE]` is in `.config/nextest.toml`.
- **`--rounds N`:** a positive integer, 1 by default.
- **`--seed SEED`:** sixteen hex digits, either case, with no prefix, the seam's form (sweeps.md, 4.3). Round r's seed, r counting from 1, is `mix(mix(SEED) ^ r)`, with `mix` as sweeps.md, 2 fixes it. Without it, each round's seed is 64 bits of the system's entropy.
- **`--out DIR`:** the campaign's directory, absent or empty. By default a fresh `tiers-campaign/TREE/<UTC time>-<pid>` beneath Cargo metadata's target directory. The directory is printed first.
- **`--last DIR`:** the directory of the tree's last campaign, as section 6 leaves it, however that campaign ended. Its `campaign.json` is the one file read from it, once, before anything runs: its `tree`, its `commit`, LAST, and its findings' signatures (section 4.2). Which campaign was the last is the caller's to say: the runner keeps no record of campaigns. LAST is reported, never checked out, so the repository needn't hold it.
- **Refused, exit 2, before anything runs:** an invalid invocation (another tree, a count that isn't a positive integer, a seed of another form); a tree with no campaign pair, `no campaign pair for TREE (tools/tiers/layers.md, 9.3)`; a `--out` that exists and isn't empty; a checkout whose tracked files differ from its commit, since its cases wouldn't replay at that commit; a `--last` that names no campaign of the tree:
  - `--last DIR holds no campaign: WHY`, where its `campaign.json` is missing or unreadable, isn't a JSON object, has no `tree` string, no `commit` of 40 or 64 lower-case hex digits or no `findings` list, or holds a finding that isn't an object with `binary`, `test` and `signature` strings, WHY saying which;
  - `--last DIR is a campaign of OTHER, not TREE`.
- **The selection:** before the first round, the campaign lists the pair's tests, as `tools/tiers/run --verify` lists a pair over its map (`tools/tiers/design.md`, 6), its disk wait and wrapper included. A round's report compares what finished with it. A listing that fails ends the campaign (section 6).
- **A round,** in order:
  1. it prints `campaign: round R of N seed SEED`, SEED in lower case;
  2. it runs `tools/tiers/run --cpu DIR/round-R TREE campaign`, with `SKEIN_TEST_FUZZY_CAMPAIGN` set to its seed, `SKEIN_TEST_FUZZY_CASE` removed and the caller's environment otherwise, its output forwarded;
  3. it reads the round (section 3);
  4. it replays each failing case (section 4.1);
  5. it rewrites its files (section 6).
- **Its commit** is the checkout's `HEAD`, read once before the first round.
- **An interruption** (SIGINT, SIGTERM or SIGHUP) reaches the running `run`, which stops as `run --cpu` does (`tools/tiers/design.md`, 7.3), and ends the campaign once it has returned, its step `interrupted` (section 6).
- **Its exit status:** 0 where every round asked for ran and none had a failing case or a failure outside a case; 1 where one did; 2 where it was refused or ended early (section 6). Starvation, CPU warnings and section 4.2's marks leave it as it is.

## 3. Reading a round

A round is read from its run's directory (`tools/tiers/design.md`, 10.3): `junit.xml`, and `summary(DIR, budgets('tools/tiers/budgets.toml'), pair='campaign-TREE')` (the same, 10.2) for its starvation, its unobserved attempts, its CPU rows and its warnings.

- **It ran** where its `junit.xml` exists or the summary reads it starved. Otherwise, its run's exit status 2 or a build that failed, it ends the campaign (section 6). A summary that refuses the round's records (the same, 7.3) leaves its CPU rows empty and says why among its warnings; its tests are read all the same.
- **Each test** is a JUnit test case: its binary (the case's `classname`, nextest's binary id), its name, its time, and its outcome. A test the wall cut short, one the summary reads starved or unobserved, is unfinished, not failed; any other with a `failure` or `error` element failed; the rest passed.
- **Its output** is the text of its `system-out` and `system-err`: the pair keeps them for passing tests too (`tools/tiers/layers.md`, 9.3). The lines read are the seam's, each from the start of a line (sweeps.md, 4.4 and 4.5):
  - `fuzzy case: TEST seed SEED`, then `: FINDING` where the check returned one;
  - `fuzzy cases: TEST N`;
  - `fuzzy reach: SWEEP FAMILY/KIND N`;
  - `fuzzy input: TEST seed SEED DIGEST`, written under a replay alone (sweeps.md, 4.4).

  Any other line is the test's own.
- **A case's input** is DIGEST, 64 lower-case hex digits, of the first input line with its TEST and SEED in its test's output; a case with none has no input.
- **A failing case** is each distinct TEST and SEED of a failed test's case lines, in the order they first appear: a test that panics with its case's line names it twice, and a reader names a case by its TEST and SEED (sweeps.md, 4.4). A case line in a passed test is no failing case. Its first line is FINDING's first line where a line of that case has one; otherwise the first line of the panic's message, the line after the first `thread '…' panicked at …` line following the case line; otherwise `no message`.
- **A failure outside a case** is a failed test with no case line: a setup error (`fuzzy setup:`, sweeps.md, 4.3), an assertion after the loop, a crash before any case. It is a verdict of its own, by test, with its first line: its panic message's first line, else its `failure` element's `message`, first line, else `no message`. It isn't replayed, since it names no case.
- **The cases a test ran:** the sum of its `fuzzy cases:` lines' N, and one more for each failing case whose sweep wrote no count, since a case that panics leaves the hook's line in its count's place; that sweep's earlier cases go uncounted. A test with neither kind of line took no case, and the report names it, with its time (the draft, 5.2).
- **Reach:** each sweep's `fuzzy reach:` counts, summed by sweep and kind over the round's tests, a kind at nought kept.
- **Unfinished:** the listed tests (section 2) the round has no passed or failed test for, those the wall cut short among them. A round that is starved has its finished tests read as any.

## 4. Replays

### 4.1 At the round's commit

- **Each failing case of a round, in its order,** K counting from 1, runs `tools/tiers/run --cpu DIR/round-R-replay-K TREE campaign -- -E 'binary_id(=BINARY) & test(=NAME)'` at the round's commit, with `SKEIN_TEST_FUZZY_CASE` set to the case's seed and `SKEIN_TEST_FUZZY_CAMPAIGN` removed, and is read as a round is (section 3).
- **Its command for a person** is `SKEIN_TEST_FUZZY_CASE=SEED tools/tiers/run TREE campaign -- -E 'binary_id(=BINARY) & test(=NAME)'` (the draft, 5.4).
- **Its verdict:**
  - `failed`: it selected that one test, which failed with a failing case of the same TEST and SEED. The case replays, and is a finding's case (section 5). The replay's first line is kept beside the round's.
  - `passed`: it selected that one test, which passed having run one case. The case doesn't replay alone, at the same commit, seed and runner, so its world breaks the strategy's section 1 (`skein/docs/foundation/testing-strategy.md`): a finding of its own, against the world, reported apart with both run directories (the draft, 7.2).
  - `unreplayed`, with why: it selected no test (nextest's exit status 4, with or without a report), or more than one; its test ran no case, or more than one, as a test of several sweeps does under a replay (sweeps.md, 7); it failed outside a case or with another case; the wall cut it short.
- **Its input:** the case's input in the replay's output (section 3), or none, kept in the case's replay (section 6), so that a person who replays the case at another commit can tell whether it drew the same (section 4.2). A round runs in a campaign's mode, where no input line is written, so a case's input is its replay's.
- **A replay whose run didn't run** ends the campaign as a round's does.

### 4.2 Against the last campaign

The draft's 7.3, as the user decided it on 2026-11-27: nothing runs at the commit the tree's last campaign ran at, LAST, so that a campaign keeps to one checkout and one build (`temper/docs/design/draft/fuzzy-campaigns.status.md`). With `--last` (section 2), the campaign reads that campaign's findings from its `campaign.json`, and nothing else of it.

- **Its signatures:** each of the last campaign's findings' `binary`, `test` and `signature` (section 6), read with its `tree` and `commit` before anything runs. A last campaign ended early holds the findings it reached, and those are its signatures.
- **Each finding is marked** as it is made (section 5): also in the last campaign, at LAST, where one of those signatures has its binary, its test and its signature; not in the last campaign otherwise. The mark needs nothing the campaign runs, so it is whole from the finding's first case on.
- **What it says** is the signature's presence alone, not a cause: a signature the last campaign held may fail for another fault now, and one it didn't hold may be an older fault its seeds didn't draw (section 5).
- **Bisecting is a person's,** by hand, when they need it: the finding's replay command (section 4.1), run in a checkout of LAST or of a commit between. A case's draws are its test's code at each commit, so only a world that prints what it drew says whether the two replays drew the same input (the draft, 7.3; sweeps.md, 4.4), which the campaign keeps as its case's input (section 4.1).
- **No git, no worktree, no build:** `--last` reads one file. The campaign neither asks whether the repository holds LAST nor checks it out, so it needs no git beyond what its checkout holds (the user, 2026-11-27).

## 5. Findings

- **A finding** is the cases that failed again on replay (section 4.1) and share a signature: their binary, their test, and the round's first line with each run of ASCII digits read as one `N` (the draft, 7.1). Findings keep the order of their first cases, by round and then by case.
- **What it holds:** its binary, its test, its first case's TEST, its signature, its first case's first line, round and replay command, its cases' count, and its cases' seeds, at most 8, the first first, each with its round; with `--last`, whether the last campaign held its signature (section 4.2). It is unattributed: an old fault or a recent landing's, whatever its mark, since that says only whether the last campaign's cases failed the same way.
- **Its title** is `TEST fails on a fresh seed: FIRST LINE`, TEST its first case's, within a finding's title's bounds (`temper/docs/design/domain/findings.md`, 2): a control character becomes a space, and a title past 200 bytes is cut at a character's boundary.
- **Its body,** within a finding's body's 4,096 bytes (the same), in this order: its commit; the campaign's `--seed` or `drawn`, and its first case's round and that round's seed; its cases' seeds; its replay command; with `--last`, its mark (section 4.2), `Also in the last campaign, at LAST.` or `Not in the last campaign.`; then its first case's output from its case line on, cut at a line to fit.
- **Apart from findings:** the cases that don't replay (section 4.1's `passed`), the cases unreplayed with their why, the failures outside a case by round, and each configured kind whose summed reach over the campaign's rounds is nought, named as a question for its world, since a finite sample proves no kind unreachable (the draft, 5.4).
- **Where it goes:** the campaign's report, for the user to route; filed by hand as a temper finding, it follows findings' rules (the draft, 7.4).

## 6. Its files

The campaign's directory holds each round's run directory, `round-R`, each replay's, `round-R-replay-K`, and two files of its own, each written whole and renamed into place after the listing, after each round's reading and after each replay, so that a campaign stopped any way keeps its finished rounds whole (the draft's notes, 4).

- **`campaign.json`,** one object:
  - `tree`, `pair`, `commit`, `seed` (the `--seed`'s, in lower case, or null) and `rounds_asked`;
  - `last`: null without `--last`; else its `directory`, absolute, and its `commit`, LAST;
  - `selection`: the listed tests, each `binary` and `test`;
  - `rounds`: one for each round run, with `round`, `seed`, `directory`, `returncode`, `starved`; `tests`, each `binary`, `test`, `outcome` (`passed`, `failed` or `unfinished`), `time_s` and `cases` (null for a test that took no case); `cases`, each failing case's `binary`, `test`, `identity` (its TEST), `seed`, `first_line`, `command` and `replay` (`directory`, `verdict`, `why`, `first_line` and `input`, the case's input or null, null where they don't apply); `failures`, each `binary`, `test` and `first_line`; `reach`, each `sweep`, `kind` (`FAMILY/KIND`) and `cases`; `unfinished`, each `binary` and `test`; `layers` and `warnings`, the summary's (section 7);
  - `findings` (section 5), each with `binary`, `test`, `identity`, `signature`, `first_line`, `round`, `command`, `count`, `seeds` (each `seed` and `round`), `title`, `body` and `last`: null without `--last`; else `true` where the last campaign held its signature and `false` where it didn't (section 4.2);
  - `nonreplaying` and `unreplayed`: their cases, as a round's `cases` hold them, with their `round`;
  - `reach`: the rounds' reach summed, and `never_fell`: each `sweep` and `kind` at nought;
  - `ended`: null, or where the campaign ended early, `step` (`listing`, `round`, `replay` or `interrupted`), `round` and `replay` (null where none), `returncode`, and `output`, the last 20 lines of the step's output.

  Seeds are sixteen lower-case hex digits; a directory is named relative to the campaign's.
- **`summary.txt`:** the same for a person to read: the campaign's commit and seed, the last campaign's commit and directory or none, and the rounds run of those asked; for each round its seed, its tests and cases, its failing cases with their replays' verdicts and commands, its failures outside a case, its tests that took no case with their times, its reach, its unfinished tests, and its CPU rows and warnings; then the findings, each title and body, the cases that don't replay, those unreplayed, the reach summed, and the kinds that fell in none. Its layout is the code's.
- **Ended early:** a listing, a round's run or a replay's run at the round's commit that didn't run, or an interruption, ends the campaign with `ended` set and exit 2, its files holding what finished before.

## 7. Its cost

- **CPU:** each round's `layers` and `warnings` are its summary's: each layer of the round against the tree's fuzzy ceiling for it, as a warning, never red (`tools/tiers/budgets.md`, 4.2). The gate's figures are a comparison, not a forecast: a round runs a process per test, and its cases draw other work than the gate's seeds (the draft, 5.3).
- **Wall:** the pair's own (`tools/tiers/layers.md`, 9.3). A round it cuts short is starved, a warning, and names its unfinished tests.
- **One checkout, one build:** a campaign builds and runs at its own commit alone, in its checkout; `--last` adds a file's read and nothing that runs (section 4.2).
- **Its envelope,** in rounds a day for each tree, and its builder are the user's figures (the draft, 5.3 and 5.5).

## 8. Where it runs

- **After landing,** once a day, on a builder apart from the gate and the landings (the user, 11-26). Which builder, its envelope and its wall are the user's. Until a runner runs campaigns there, one runs by hand at a landed commit: `tools/tiers/campaign temper --rounds 1`.
- **Never at the gate:** `tools/tiers/one-process` removes both variables from what it runs (`tools/tiers/design.md`, 11.5), so the gate, the delivery check and anyone's run of the suites take the gate's mode. Only the campaign sets them, for its pair's runs.

## 9. Testing

`python3 -I -B tools/tiers/test_campaign.py` proves it offline, on recorded reports: run directories as `tools/tiers/run --cpu` leaves them, under `tools/tiers/fixtures/campaign/`, which `.config/fixture-readers.toml` names as that suite's. A fake `tools/tiers/run` copies a recorded directory into the one it is given, exits with its recorded status and records its arguments, its working directory and its environment; a fake `cargo` answers the listing. A recorded `--last` campaign is a `campaign.json` alone. No build, no test binary and no campaign runs.

- **The command:** each refusal, each of `--last`'s among them; the round seeds for a given `--seed`, pinned; drawn seeds printed and differing; each round's and replay's command, directory and environment, `SKEIN_TEST_FUZZY_CAMPAIGN` and `SKEIN_TEST_FUZZY_CASE` each set and removed as section 2 and 4 give them; the exit statuses.
- **Reading:** a returned finding; a panicked case, its first line taken from the panic; a case named twice read once; cases counted across a test's sweeps and after a panic; a test that took no case; a failure outside a case and a setup error; reach summed, nought kept; a starved round's unfinished tests, cut and not started.
- **Replays:** each verdict, the test of several sweeps, the selection of nothing; a case's input read from its replay.
- **Against the last campaign:** a finding whose binary, test and signature the recorded last campaign holds marked `true`, with its body's line; one it doesn't hold, the same test failing on another first line among them, marked `false`, with its line; the mark in place from the finding's first save on; no run beyond the rounds' and their replays', no worktree and no `git worktree` or `git cat-file` call, with findings and without; a last campaign at a commit the repository doesn't have read as any; each of `--last`'s refusals.
- **Findings:** the signature's digits; seeds at most 8, the first first; their order; the title's cut at a character's boundary and the body's bound.
- **Its files:** rewritten after each step, so a campaign stopped after its first of two rounds holds that round whole; a failed build ending the campaign with exit 2.

## 10. Later

- **Filing** a campaign's findings into temper by its runner, with a source, a matching rule and a closing rule of their own (the draft, 7.4 and 11).
- **More cases** for a sweep that asks a campaign for them (the draft, 6; sweeps.md, 7).
- **Bisecting** a finding over the landings since the last campaign, a person's by hand until then (section 4.2), and **shrinking** a case (the draft, 11).
