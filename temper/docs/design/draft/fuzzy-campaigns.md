# fuzzy-campaigns: fresh seeds off the gate, one case replayed, and worlds widened by what escapes

Draft, 2026-11-26, with the user; its review's corrections, and the user's word on where campaigns run and on the gate's samples, the same day. The fuzzy suite runs fixed seeds at the gate, the same work at every run (`test-budgets.md`, 4), so it explores only where the code beneath it changes. The user asked (11-24) whether it should run "separately with a different seed each time", and (11-26) how fuzzy testing could catch production's problems early. The prompt was an agent crash that smith's session world, the world drawing random batches of tool calls, couldn't draw; a0f6f9d91b fixed it. This draft runs the fuzzy suite's sweeps after landing, with fresh seeds, in campaigns of their own:

- one seam through which a sweep takes its cases;
- one case replayed alone;
- each finding read and kept;
- the kinds a campaign reached, counted;
- a repair that widens the world its fault escaped.

The gate keeps its seeds. What a seed can draw (agents generated from a run's tools, a table of wakes against parks, conservation checks) is the misbehaving agents draft's, this one's companion, not yet landed. This draft widens how many seeds run, and keeps what they find.

## 1. In one page

- **The gate keeps its seeds** (section 3). The fuzzy suite stays at the gate in full, with fixed seeds, so a red there is the head's alone. It catches what a change breaks beneath its sweeps, and holds every finding pinned. Two tests registered twice go.
- **A case, through one seam** (section 4). `skein_world::sweep::cases` takes a test's identity, its seeds and a check for each case:
  - at the gate, it yields the seeds as given;
  - in a campaign, it yields fresh ones, each from the campaign's seed, the test's identity and the case's number;
  - under a replay, it yields one seed, in the campaign's configuration.

  A failing case names its test and seed. `each_seed` stays as it is, for the tests that keep fixed seeds.
- **Every seed a valid case** (section 4.4). A world adopts the seam only once any seed draws a case it can run. A case's own checks run in every mode. A check over the gate's set of seeds (every kind fell, an exact count left out) runs only on the gate's seeds, and in a campaign a sweep counts the cases each kind fell in instead.
- **A campaign** (section 5) is rounds of a tree's campaign pair, `campaign-TREE`, run by `tools/tiers/campaign`. Each round has its own campaign seed, drawn and printed. It runs over the binaries whose sweeps take their cases from the seam, in the layers that replay. A round's CPU is reported against the tree's fuzzy ceilings, as a warning. Campaigns run once a day, on a builder apart from the gate (the user, 11-26). Which builder, their envelope and their wall are the user's figures.
- **Wider in a campaign** (section 6): a sweep may configure more kinds in a campaign than at the gate, kind by kind as its world learns to inject and judge them, and take more cases there.
- **A finding** (section 7) is a failing case, replayed alone at its commit. It reads as its test, its failure's first line, its seed and the configuration it failed under. One more replay, at the last campaign's commit, says what that case did there. It is kept in the campaign's report, for the user to route, and pinned at the gate, in the lowest tier that shows it, once fixed.
- **An escape widens its world** (section 8): a fault found outside the worlds, which the nearest fuzzy world's draws couldn't reach, is repaired with its case pinned, and with that world's draws widened to reach its kind. The class is then drawn, not only the instance.
- **Adoption** (section 9) goes world by world, each keeping its gate counts.

## 2. Where it stands

- **The suite:** the `fuzzy` profile's selection in `.config/nextest.toml`, mostly each package's `suite` binary's `fuzzy` module, run as `tools/tiers/one-process fuzzy`. The gate runs it in full in the layers that replay (`tools/tiers/layers.md`, 9.1), within its fuzzy budgets by tree and layer (`tools/tiers/budgets.toml`). A failing seed is fixed and kept, as a scenario or among the sweep's pinned seeds (`skein/docs/foundation/testing-strategy.md`, 8).
- **Seeds are chosen for reach:** a sweep runs the fewest seeds at which every kind it configures falls (`temper/docs/design/testing/workloads-sweeps.md`, 1.4). temper's composed sweep runs one, seed 6, "the minimum" that draws all twelve families (`temper/tests/system/tests/suite/fuzzy/workload.rs`). jig's core people sweep hand-picks three seeds, each named for what it reaches (`jig/tests/core/tests/suite/fuzzy/workload.rs`). At the gate a sweep proves that its kinds fall, on the seeds that make them fall, and explores past them only when the code moves.
- **The seam exists in part.** `each_seed` (`skein/testing/skein-world/src/sweep.rs`) judges each seed of a range. Seven calls in six files use it: temper's engine-domain walking, opening and follows sweeps, its composed workload, jig's sign-ins, and hearth's render properties, whose helper serves focused tests too. Most sweeps loop over seeds of their own. `Streams` gives a world independent streams from one seed (`skein/docs/design/testing/sweeps.md`, 2).
- **A failure names its seed where its test prints one.** Nothing in the seam does. A replay runs the whole test function, every seed of it.
- **What escaped** (11-26): the agent shell panicked on dev.fabrig.ai. The shape was a batch whose earlier call the tools answered invalid at their entrance, before a write, with a read past the batch. Starting that read with no room reserved for its result underflowed the session's reserved room (a0f6f9d91b). smith's session world draws random batches over 1000 seeds (`smith/tests/session/tests/suite/fuzzy/simulation.rs`). But its menu of argument bodies holds no `job` and no `yield_seconds` (`smith/tests/session/src/provider.rs`), so none of its seeds could draw the shape, and more seeds alone wouldn't have found it. The misbehaving agents draft widens that world's draws, and section 8 makes a repair do so.
- **The evidence:** a survey of main on 11-24, in `fuzzy-campaigns.notes.md`.

## 3. The gate keeps its seeds

- **Fixed, as `test-budgets.md`, 4 has it:** the same work at every run, so a red at the gate is the head's alone, and its figures move only with the code. A fresh seed's finding is unattributed: an old fault, or a recent landing's, never the head's to answer for at the gate. Rotating seeds at the gate is no option of this draft's (notes, 1).
- **The fuzzy suite stays at the gate in full** (the user asked on 11-25 whether it would leave the gate). Its fixed seeds still reach new states wherever a change moves the code beneath them, and they caught later changes' faults (notes, 2). Every campaign finding is pinned in the gate's suites too (7.5), so the gate gains the regressions campaigns find. Each sweep keeps its whole sample at the gate, campaigns or not (the user, 11-26).
- **Nothing changes at the gate for the failure worlds:** their sweeps run the seeds their budget allows there (`failure-worlds.md`, 5 and 8), and a campaign adds seeds after landing alone.
- **Apart from the replay after landing:** the daily run of the gate's suites at a landed commit, which checks that they replay (`test-budgets.md`, 4), keeps the gate's fixed seeds. A campaign is another run.
- **Two tests registered twice go.** skein's channel suite registers `generic_protocol.rs` as a module of its own (`skein/tests/channel/tests/suite/main.rs`), and again under `fuzzy/channel.rs`. So `both_directions_progress_over_socket_and_pipes` and `a_last_word_reaches_the_peer_after_the_writer_pipe_fails` run in both suites. The fuzzy module's second copy goes, and its 64-seed caller stays.

## 4. A case

### 4.1 The seam

A sweep that takes part in campaigns takes its cases through `cases`:

```rust
/// The test a sweep belongs to: its package, its module path and its name,
/// and a part where one test, or one helper serving several, runs several sweeps.
pub struct Test { /* package, module, name, part */ }

/// `this_test!("name")` or `this_test!("name", "part")`: the calling test,
/// its package and module taken where it is written.
macro_rules! this_test { /* ... */ }

/// Judge each of a sweep's cases (4.2): its seeds at the gate, fresh ones in a
/// campaign, one under a replay. The first case whose check returns a finding
/// ends the sweep, and comes back with its test and seed.
pub fn cases<S, E, F>(test: Test, seeds: S, check: F) -> Result<(), Case<E>>
where
    S: IntoIterator<Item = u64>,
    E: core::fmt::Display,
    F: FnMut(u64) -> Result<(), E>;

/// Whether the cases are the gate's seeds, for a check over all of them, and
/// for a sweep's configuration (section 6).
pub fn gate() -> bool;
```

- **`Test`** is the case's identity. A helper serving several tests takes its caller's, as walking's and follows' shards do.
- **`seeds`** is a range or a hand-picked list. Its length is the case count, at the gate and in a campaign, unless the sweep asks a campaign for more (section 6).
- **A check returns its finding,** as jig's core sweeps return theirs today. A check that panics is named too (4.3).
- **A case's seed is the world's seed.** A world on streams takes it as `Streams::new(seed)`, and one that isn't takes it as it took its loop's seed. A draw made some other way, as a fixture picked by `seed % n`, takes it from the case's seed too, so that a campaign moves every draw.
- **`each_seed` stays** as it is, for the loops that keep fixed seeds, focused tests' among them.

### 4.2 Its modes

- **The gate's,** with neither variable set: the seeds as given, in order. It is the same work as today.
- **A campaign's,** `SKEIN_TEST_FUZZY_CAMPAIGN=SEED`, sixteen hex digits: the case count's fresh seeds. Case k's seed is `mix(mix(SEED ^ fnv(TEST)) ^ k)`, with `mix` and `fnv` as `skein/docs/design/testing/sweeps.md`, 2 fixes them, and TEST the identity written `package::module::name`, then `/part` where it has one. A case depends on the campaign, its test and its number alone, never on another test or on the order tests run.
- **A replay's,** `SKEIN_TEST_FUZZY_CASE=SEED`: that one seed, once, in the test its run selects. A replay is of a campaign's case, so it runs the campaign's configuration: `gate()` is false under it.
- **The variables** are skein's test set (`skein/docs/foundation/cli.md`, 5). They are declared in skein's test table, in skein-world, with its generated test-env document (the same, 6 and 8), and read once in a process, through that table. Empty is unset. A value that isn't sixteen hex digits, or both set at once, is a setup error: the test fails before its first case, naming the variable.
- **Out of the gate's way:** `tools/tiers/one-process` removes both variables from the environment of what it runs. So the gate, the delivery check and anyone's run of the suites take the gate's mode, whatever the caller's shell holds. Only `tools/tiers/campaign` sets them, for its pair's runs.

### 4.3 A failing case names itself

- **A finding a check returns** comes back from `cases` as `Case`, printed `fuzzy case: TEST seed SEED: FINDING`.
- **A check that panics:** the seam installs a panic hook once in a process. While a case runs, it holds the case's test and seed, for that case alone. The hook prints `fuzzy case: TEST seed SEED` and then calls the hook before it. Nothing catches the panic (`skein/docs/foundation/programming-model.md`, 10.4).
- **Outside the gate's mode,** a sweep that ends prints `fuzzy cases: TEST N`, the cases it ran, so a campaign can tell a sweep that ran from a test that took no case.
- **At the gate too,** a failing case names its seed, where today it does only if its test prints it.
- **The replay command** is the campaign's to write, since the campaign knows the binary and the full test name (5.4).

### 4.4 What a case checks

- **A case's own checks** run in every mode: contracts as it goes, invariants once it settles, its referee, its goals' expectations (the strategy's section 6).
- **A check over the gate's seeds** holds only for those seeds: `Sweep::check_fell`, an exact count of what was left out, a coverage array filled across the loop. Outside the gate's mode, `check_fell` passes and prints its reach, `fuzzy reach: SWEEP FAMILY/KIND N` for each configured kind, its count included where it is nought. A test's own checks over its loop run only where `gate()` holds.
- **Sweep counts:** `Sweep::ran` adds one for each kind that fell in a case, since `FaultLedger::fell` names each kind once. `check_fell` reads that count as it read the set.
- **Every seed a valid case.** A world adopts the seam only once any seed draws a case it can run. A draw its world can't run is the world's to make valid: drawn again from the case's own stream, or skipped and counted as ineligible in the sweep's reach. Such a draw is a configured kind with no eligible target, say, or a hold drawn with no release. It is never failed as the product's. temper's composed workload is such a world today: its faults panic where no goal is eligible for a configured kind, and a seed can hold a train member it never releases. So it adopts once its draws are total (section 9).
- **A full-width seed:** a case's seed is any `u64`. A world that does arithmetic on its seed (walking's `seed + 7100`) wraps it or derives from it, and keeps the draws of the gate's seeds.
- **A fixed scenario keeps its seed.** A test that asserts what its seed was chosen to show (a pinned seed's named outcome, a golden, a digest) takes no case from the seam, and a campaign runs it as the gate does (5.2).
- **The gate's counts stay:** an adopting sweep keeps its binary's checks counts as they were (the `seed` counts that `each_seed` and `Sweep::ran` make), and its instructions within the growth allowance (`tools/tiers/budgets.md`, 8), as its change's gate measures them.

## 5. A campaign

### 5.1 Rounds

- **A campaign is rounds.** A round runs a tree's campaign pair once, with one campaign seed, every case of every adopted sweep fresh.
- **`tools/tiers/campaign TREE [--rounds N] [--seed SEED] [--last DIR]`:**
  - gives each round its seed and prints it before the round starts: drawn from the system's entropy, or, with `--seed`, round r's is `mix(mix(SEED) ^ r)`;
  - runs `tools/tiers/run --cpu DIR TREE campaign` with `SKEIN_TEST_FUZZY_CAMPAIGN` set;
  - reads the round's report, then replays its failing cases (7.2).

  `--last` names the last campaign's directory, for 7.3.
- **A round is a replay:** its seed and its commit give the same cases again.
- **Why rounds** (notes, 4): a round is the unit a runner schedules, stops and reads. A test's case count stays the test's own.

### 5.2 What a round runs

- **The pair `campaign-TREE`** in `.config/nextest.toml`: the tree's clause of the `fuzzy` profile's filter, and'ed with the binaries listed for it, as the real pairs list theirs (`tools/tiers/layers.md`, 9.2). They are binaries of the layers that replay whose sweeps take their cases from the seam. The real loop and end to end are never listed, since a case there doesn't replay.
- **Registered as every pair is:** its packages sorted and never empty in `tools/tiers/packages.toml`, generated and verified (`tools/tiers/design.md`, 6), and its prerequisite binaries listed, even where there are none (the same, 2.1). Its macOS overrides are as every profile's. The boundary check holds its list to `tools/tiers/layers.toml`'s layers that replay. A tree's pair comes with its first adopting sweep.
- **Its report** keeps every test's output, passed or failed, since the reach and case lines come from passing tests too.
- **A test that takes no case from the seam,** in a listed binary, runs as at the gate, and the round's report names it, with its time. A binary whose fixed tests outweigh its sweeps moves them to a binary of their own, or stays off the list.

### 5.3 Its cost

- **The gate's figures are a comparison,** not a forecast. A round runs a process per test, its cases draw other work than the gate's seeds, and a wider catalogue changes each case's work. `--cpu` reports each layer of the round against the tree's fuzzy ceilings alone, as a warning, never red.
- **Its wall:** the pair takes the default profile's backstop until the user sets the campaign's.
- **A campaign's envelope,** in rounds a day per tree, is the user's figure, as its hardware is.

### 5.4 Its report

- **Per round:**
  - its commit and campaign seed;
  - each failing case: its binary, its full test name, its seed, the failure's first line, and the command that replays it, `SKEIN_TEST_FUZZY_CASE=SEED tools/tiers/run TREE campaign -- -E 'binary_id(=BINARY) & test(=NAME)'`;
  - each failure outside a case (a setup error, an assertion after the loop, a crash before any case line), by test, as a verdict of its own;
  - each test it ran, with its time and the cases it ran, or none;
  - each sweep's reach: every configured kind with the cases it fell in;
  - what it left unobserved: where the wall or a crash cut it short, the tests that finished and those that didn't.
- **Folded over the campaign:**
  - the failing cases, one finding per signature (7.1);
  - each sweep's reach, summed;
  - each configured kind that fell in none of the campaign's cases, named as a question for its world: a finite sample proves no kind unreachable.
- **A build that fails** ends the campaign, naming its step.
- **Its files:** `campaign.json` and a text summary in the campaign's directory, beside each round's own run directory.

### 5.5 Where it runs

The options, best first:

1. **Once a day,** on a builder apart from the gate and the landings, for the rounds its envelope holds. A day's rounds make one report to read, and the campaign competes with no gate. The user's pick, 11-26.
2. **At each landing,** a round or two after landing. That ties its findings to landings they mostly aren't owed to, and its load to main's pace.
3. **At the gate, with rotating seeds:** no (`test-budgets.md`, 4).

Which builder, the envelope and the wall are the user's figures still (status, **Open**). Until a runner runs campaigns there, a campaign runs by hand after landing.

## 6. Wider in a campaign

- **Its catalogue, kind by kind.** A sweep that configures one kind of each family at the gate, so that the fewest seeds make each fall (`temper/docs/design/testing/workloads-sweeps.md`, 1.2), may configure more in a campaign, where seeds aren't scarce. A kind joins a sweep's campaign catalogue once its world can inject it, give it an eligible target, exclude what its open problems need, and say what each goal expects of it. A kind that needs another workload stays in the cheapest world that reaches it (1.1 there). The gate's list doesn't change. Each kind's draw keeps its own place in its part, so a kind added moves no other kind's draw (1.2 there).
- **More cases.** A sweep may ask a campaign for more cases than it runs at the gate. temper's composed sweep runs one case at the gate, and a campaign is where it runs many.
- **Generated agents** (the misbehaving agents draft) widen what a case can be. In a campaign they are one more sweep, and their table's cells are kinds of a family of their own, counted as any kind (5.4).

## 7. Findings

### 7.1 What a finding is

- **A failing case at its commit:** its test, its seed, the configuration it ran (the gate's or a campaign's), and its failure's first line.
- **Its signature:** its binary, its test, and the failure's first line with each run of digits read as one `N`, so that two cases failing the same way are one finding whatever values they print. A finding lists its cases' seeds, at most 8, the first first.
- **Unattributed:** it may be an old fault or a recent landing's, until 7.3's replay or a person says more.

### 7.2 Replayed before it is reported

- **Each failing case is replayed alone,** by its replay command, at the round's commit, through the same pair. The replay must select exactly one test and run exactly one case. Failing again, the case is a finding.
- **Passing alone,** at the same commit, seed and runner, the case doesn't replay, so its world breaks the strategy's section 1, where a seed replays to the same run. That is a finding of its own, against the world, reported apart, with the case's two runs named.
- **A replay that selects nothing or runs no case** says so, and its case is kept as unreplayed.

### 7.3 At the last campaign's commit

- **Each finding's first case is replayed once** at the commit of the tree's last campaign, from `--last`. The report gives what it saw: the case passed at that commit, failed the same way, failed otherwise, or ran no case there (the test or the seam absent).
- **It speaks for that case alone,** not for the signature's others. A case's draws are its test's code at that commit, so its input is the same only where its world prints a digest of what it drew and both digests match. Without that, the report says the input wasn't compared. Bisecting over the range is later (section 11).

### 7.4 Its words, and where it goes

- **Its title:** `TEST fails on a fresh seed: FIRST LINE`, within 200 bytes.
- **Its body:**
  - its commit, the campaign's seed and round, and its cases' seeds;
  - the replay command;
  - what 7.3's replay saw;
  - the failure's lines, within the bound;
  - the capture of its generated answers, where its world keeps one (the misbehaving agents draft).
- **Where it goes:** the campaign's report, for the user to route. Filed by hand as a temper finding, it follows findings' rules, closed once the landing that fixes it is cleared (`temper/docs/design/domain/findings.md`, 7).
- **Filed by the campaign's runner** is later (section 11). It needs a source, a matching rule and a closing rule of its own. Ordinary red findings aren't matched by signature (`findings.md`, 4 there). A landing tier's red finding would be cleared by the next round, which draws other cases (`temper/docs/design/domain/red-findings.md`, 6).

### 7.5 Repaired and pinned

- **Fixed, then pinned in the lowest tier that shows it** (the strategy's section 9), as the strategy's section 8 keeps a seed:
  - as a scenario, where it shows behaviour worth naming;
  - among its sweep's seeds at the gate only where the gate's configuration reproduces it;
  - else as a scenario running its case in the configuration it failed under.
- **A generated agent's case is pinned as its capture,** a fixed scenario: each answer keyed by its conversation, its turn and its request's attempt, with its finish and its tokens (the misbehaving agents draft). Its faults and timing still replay from the case's seed. Once a generator's menu or weights change, its seed draws other answers (notes, 6).
- **A finding that can't be fixed yet** is an ignored test replaying it, its reason naming the work that fixes it (the strategy's section 8). Where its sweep must go on meanwhile, it takes the narrowest exclusion its world reads (`temper/docs/design/testing/workloads-sweeps.md`, 1.3).

## 8. An escape widens its world

- **An escape** is a fault found outside the worlds, on dev.fabrig.ai, by a live run or by a person, that the nearest fuzzy world's draws couldn't reach. Its repair does three things:
  1. it fixes the fault;
  2. it pins its case in the lowest tier that shows it (the strategy's section 9);
  3. it widens that world's draws so that the fault's kind is drawn: a menu body, a kind in a family, a reference to an earlier id, a wake.

  The repair's change names the world and what it now draws, and the widened sweep passes at the gate on the fixed code.
- **11-26's crash** shows the rule. Its fix pinned its case in a session step test and in the run world's scripted stories, but widened no world (a0f6f9d91b). The misbehaving agents draft's first step gives smith's session world's menu a `shell` body with `yield_seconds` and a `job` body, the widening this rule would have asked for.
- **Why:** a pinned instance guards one shape. The class's next instance comes on another seed or in a neighbouring shape, which only a widened draw reaches, and campaigns then explore it.
- **It goes in the strategy's section 8,** beside the seed kept.

## 9. Adoption

- **A world adopts in its own change:**
  - its loops take their cases through `cases`, with their test's identity and their gate seeds, unchanged;
  - its draws are valid at any seed (4.4);
  - its checks over the loop wait for `gate()`;
  - its binary is listed in its tree's campaign pair, which its tree's first adopter adds.

  It keeps its gate counts (4.4).
- **First, temper's engine-domain walking sweep** (`temper/tests/engine/domain/tests/suite/fuzzy/walking.rs`), already on `each_seed`, its eight shards each passing its own test's identity. Then opening and follows.
- **Then, by the yield their draws promise:**
  - jig's core workload sweeps, whose checks already return their findings;
  - temper's forge world's sweeps, around landings and their queue (the strongest record of findings, notes, 2);
  - temper's composed workload, once its draws are total (4.4), and then its catalogue, kind by kind (section 6);
  - smith's session world, once the misbehaving agents draft's menu bodies and its answers drawn per request have landed, since the second re-picks that world's seeds;
  - hearth's and skein's machine worlds.
- **Hearth's render properties keep `each_seed`,** since their helper serves focused tests, which keep fixed seeds.

## 10. What it changes

- **`skein/docs/foundation/testing-strategy.md`:**
  - 1: the same work at every run of the suites, with a campaign beside them, whose cases still replay;
  - 8: campaigns after landing, a failing case naming its seed, and an escape widening its world;
  - 8.1: the campaign pairs' CPU reported as warnings, and their wall the user's.
- **`skein/docs/design/testing/sweeps.md`,** 4: `cases`, `Test`, the modes, the case's derivation and lines, `gate()`, Sweep's counts and its reach lines.
- **skein's test table and its generated test-env document** (`skein/docs/foundation/cli.md`, 8): skein's two test variables.
- **`tools/tiers/design.md`:** `tools/tiers/campaign`, and one-process removing the variables (11.5 there).
- **`tools/tiers/layers.md`,** 9, and **`tools/tiers/budgets.md`,** 4.2: the campaign pairs beside the real pairs, and their layers' rows.
- **`temper/docs/design/testing/workloads.md`,** 8: a sweep's campaign configuration, and the composed workload's draws valid at any seed.
- **`.config/gate/README.md`,** After landing: the campaign.
- **`test-budgets.md`,** 11: its sweep of fresh seeds is this draft's, changed with it.

## 11. Not in this draft

- **What a seed can draw:** generated agents, the table of wakes against parks, conservation checks at a domain's transitions. These are the misbehaving agents draft's.
- **A generated producer for landings and goals,** drawn from a grammar of its own: temper's workload kit already draws goals, trains and parties' acts from the seed (`temper/docs/design/testing/workloads-kit.md`, 1), and neither this draft nor the misbehaving agents draft schedules more yet.
- **Filing a campaign's findings into temper** by its runner, with a source, a matching rule and a closing rule of their own (7.4).
- **Coverage-guided fuzzing of decoders** with `cargo fuzz` (the strategy's section 6), which no target does yet. A campaign's runner could host its runs later.
- **Conversion across two releases under a crash** (`layouts-both-ways.md`), a sweep of its own.
- **Bisecting a finding** over the landings since the last campaign, and **shrinking** a case.
- **Shrinking a sweep's sample at the gate** once campaigns cover it: it stays whole (the user, 11-26).
- **The campaign's runner,** outside this repository: its schedule and its builder (5.5).

**Tests stay cheap:** the suites are the check. A milestone runs a test it adds or changes by name, alone, at most. **A plan runs the focused and fuzzy suites alone and is complete when it lands (`AGENTS.md`, 5).** No milestone runs a campaign or a slow tier. The first campaign runs after landing, on the user's word. `walk = false`.
