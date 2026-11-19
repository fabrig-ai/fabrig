# macos-tiers: macOS tested after landing, its reds filed and repaired as Linux's are

Draft, 2026-11-18, with the user. macOS support becomes a first-class
citizen: GitHub's macOS runners test fabrig automatically after landing,
and what they find is filed and repaired as a red of Linux's tiers is.
The end-to-end stories on fakes (browser, TUI, the process tier's owned
Forgejo) run often, the focused and fuzzy suites rarely, more often at
first, and a few stories run on a real model, on the user's key held as
a GitHub Actions secret. It builds on `macos.md`, the port, and on
`../domain/after-landing.md`, the tiers.

The user's rule for it (2026-11-18): **best effort, and last in
priority order**, at least for now. macOS's work goes after everything
else, and what is Linux's may still be degraded there, by name. The
frequent run takes less than three hours.

## 1. In one page

- **Two tiers of fabrig's own on macOS** (section 3): `macos-e2e`,
  the end-to-end stories on fakes, every three hours, each run within
  three; and `macos-suites`, the focused and fuzzy suites, every six
  hours at first, then daily. Each is a tier as `tier-main.yml` is, so
  temper does the rest: one run per tier at a time over every landing
  since the last, failures rerun alone, new ones bisected to the
  landing that brought them, and each red filed and repaired
  (`../domain/after-landing.md`, sections 2 to 7). Nothing in temper
  changes.
- **The tiers relay** (section 5): temper dispatches only on fabrig's
  own forge, and the Macs are GitHub's. Each macOS tier's workflow
  there hands its run to GitHub's `macos.yml` at the same commit, waits,
  and brings back its report. Its job only waits, on a runner label of
  its own, so it never takes a gate's place.
- **`macos.yml` becomes dispatch-only** (section 4): a set, a commit,
  the tests to run alone and a name for the run; one job per part, in
  parallel; one JUnit report of the failing tests, by the ids tier
  main's report uses. Its schedule goes: GitHub dropped about half of
  its scheduled runs, and nothing read the rest.
- **A build that fails on macOS is a red,** blamed and bisected as a
  test is; a runner that fails is a fault, retried and never blamed
  (4.4).
- **Live on macOS, once a day** (section 6): temper's key landing story
  and skein's Anthropic stories, on the cheapest stock model, with the
  user's key in a GitHub environment only main reaches. Not a tier: as
  the live tier's, its red goes to triage, never to a landing.
- **Bring-up before the tiers** (section 7): one full run by hand, then
  each failure fixed or listed as Linux's own, by name. A tier is turned
  on once its set is green, the end-to-end one first.
- **macOS last** (section 8): its repairs take the lowest priority, its
  runs have no budgets yet, and a red blocks nothing.

## 2. Where it starts

Read on 2026-11-18 through GitHub's public API, which gives each run
and its failing step's annotations without a login (`../testing/macos.md`,
section 1).

- **What runs:** `.github/workflows/macos.yml`, on `macos-26`, every
  target compiled, then the one story,
  `a_csv_change_passes_ci_and_a_blocking_agent_review_on_the_owned_forge_then_lands_and_survives_restart`
  (`../testing/macos.md`, section 2). A run takes 7 to 13 minutes, its
  dependencies cached.
- **Half its schedule is dropped.** Its cron asks for four runs a day;
  GitHub started about two.
- **It has been red, and nothing read it.** The story failed in six runs
  from 2026-11-13 to 11-15, then passed again. Since 2026-11-17 22:54Z
  the run stops before the story: `temper-box forge fetch` refuses a
  forge with no setup, "run forge prepare 1 --listen ADDR --person LOGIN
  EMAIL", since the forge's places are kept at set-up. A repair landed
  on 2026-11-19, 803ec4abac: the step prepares the forge first, and the
  next run shows whether the story passes.
- **Every target compiles,** so the port's sections 3 and 4 hold at the
  compiler (`macos.md`); the suites, the browser tier and the TUI's
  stories have never run on a Mac.
- **Linux's after landing** is one tier, `main`, its workflow
  `tier-main.yml` on fabrig's forge, whose job runs the runner's
  `$CI_TIER` and uploads `junit.xml` as the artifact `junit`
  (`../domain/after-landing.md`, section 4). The live stories run once a
  day apart from it (`../testing/live-tier.md`).

## 3. The sets

| Set | Tier | What | How often | Within |
|---|---|---|---|---|
| `e2e` | `macos-e2e` | the process tier's stories, its owned Forgejo real Forgejo built for macOS (`macos.md`, 4.3), the model fake; the browser tiers of temper, jig and skein, with their TUI and browser stories on fakes | every 3 hours | 150 minutes a run |
| `suites` | `macos-suites` | the focused and the fuzzy suites of skein, smith, jig and temper | every 6 hours, then daily | 300 minutes a run |
| `live` | none | temper's key landing story and skein's Anthropic stories, on a real model and the owned Forgejo | daily | 60 minutes |

- **How often is the tier's span** (`../domain/after-landing.md`,
  section 2): the least time between the starts of its runs. A run
  waits for main to move, then takes its newest tip.
- **Within is the run's bound,** dispatch to report, the relay's wait
  included. `e2e`'s keeps a run inside its span, as the user asks.
  Past it, the relay cancels the run on GitHub and the tier's run is a
  fault (5.2).
- **Why `e2e` is the frequent one:** its stories are what a person on a
  Mac does, and the process and readiness code under them is where
  macOS differs. Most of the suites test logic Linux's gate already
  holds.
- **`suites` moves to daily** once it has been green a week, at a
  restart of the reference deployment the user approves, since a tier's
  span is in its configuration (`../domain/after-landing.md`, section
  11).
- **Left out:** hearth, whose tests are Linux's machines; tier main's
  `conformance-hearth`; the live tier's Claude Code story, which needs a
  program's login no runner holds, and skein's OpenAI stories, which
  need a ChatGPT sign-in (section 11).

## 4. The workflow on GitHub

### 4.1 Its inputs

`.github/workflows/macos.yml`, on `workflow_dispatch` alone:

- **`set`:** `e2e`, `suites`, `live`, or `all`, which is `e2e` and
  `suites` together, for a run by hand.
- **`commit`:** the forty hex digits to test. The run is dispatched on
  `main` and checks this commit out.
- **`tests`:** ids one a line, as tier main's report names them; empty
  runs the whole set. Given, it is a rerun or a bisect's probe
  (`../domain/after-landing.md`, section 2).
- **`request`:** the run's title, `run-name`, so that whoever
  dispatched it finds it among the workflow's runs.

The commit must be on `main`: the first step refuses one that
`origin/main` doesn't hold, before anything is built or any secret is
reached.

### 4.2 Its parts

| Part | Set | Runs |
|---|---|---|
| `process` | `e2e` | the pair `process-temper`, through `tools/tiers/run` |
| `browser` | `e2e` | the pairs `browser-temper`, `browser-jig` and `browser-skein`, through `tools/tiers/run` |
| `skein`, `smith`, `jig`, `temper` | `suites` | the tree's focused suite, then its fuzzy one, as the gate runs them |
| `live` | `live` | the stories of section 6 |

- **One job per part,** in parallel, so that each builds only its own
  targets and its disk stays within the runner's 14 GB.
- **A part runs only its ids** when `tests` is given, and a part holding
  none of them is skipped, its build too. Which part holds an id is
  read from the id's binary, never from a listing that needs a build.
- **One report:** a last job gathers each part's nextest JUnit into the
  run's artifact `junit`, and each part's own file stays beside it with
  every case's duration.
- **Each part reports its disk and its wall time,** as the workflow's
  one job does today, and its failures annotate the run as now.

### 4.3 What it installs

- **As today:** the toolchain `rust-toolchain.toml` pins, and
  `cargo-nextest` checked by its SHA-256; the registry cached by the
  lock.
- **Per part:** the dependencies' build cached by the lock, the
  toolchain and the part, all the parts' caches within GitHub's 10 GB a
  repository, line tables only for debug information.
- **The browser part:** the `wasm32-unknown-unknown` target, and Chrome
  for Testing at the version Linux's browser tiers run, its download
  checked by its SHA-256 and named by `SKEIN_TEST_CHROMIUM`.
- **The process and live parts:** Forgejo and its runner through
  `temper-box forge fetch` (`../protocol/setup/owned.md`, section 6), as
  the one story takes them today.
- **Timeouts for a slower machine:** each nextest profile the parts run
  gains an override for `cfg(target_os = "macos")` that doubles its
  slow timeout, in `.config/nextest.toml`, rather than a profile of
  macOS's own.

### 4.4 Reds and faults

- **A failing test is a red,** by its id.
- **A build that fails is a red too:** the part reports one failing
  case, `build.<part>` (`build.temper`, say), with the compiler's first
  errors, so that a landing that breaks macOS's build is blamed and
  bisected as one that breaks a test. A rerun or a probe of
  `build.<part>` builds that part.
- **Anything else is a fault,** never a red: a runner lost, a job past
  its timeout, a disk full, a download refused. The part fails with no
  report, and the run with it.

### 4.5 What stays

- **The one story stays in `process`,** by its name, and its support on
  macOS stays as `../testing/macos.md`, section 2 has it.
- **`release-macos.yml`** is unchanged.
- **No concurrency group by ref:** temper runs one run per tier at a
  time, and GitHub queues what passes its limit on macOS jobs at once.

## 5. The relay

### 5.1 The tiers' workflows

- **`tier-macos-e2e.yml` and `tier-macos-suites.yml`,** on fabrig's
  forge beside `tier-main.yml`, dispatch-only with the same inputs,
  `temper_commit`, `temper_key` and `temper_tests`. Each runs the
  runner's `$CI_TIER --tier NAME COMMIT` with `CI_TIER_TESTS`, and
  uploads `junit.xml` as the artifact `junit`, as tier main's does.
- **Their label is their own,** `relay`, never `dogfood`: their job
  waits for hours and builds nothing, and `dogfood`'s places are the
  gate's and tier main's. The runner that takes `relay` runs several
  jobs at once.

### 5.2 What `$CI_TIER` does for a macOS tier

It lives on the runner's host, outside this repository, as tier main's
does. Its contract:

- **The commit reaches GitHub,** whose repository mirrors fabrig's main:
  it waits until GitHub has the commit, within 15 minutes, else a fault.
- **It dispatches `macos.yml`** on `main`, with the tier's set, the
  commit, the tests and a request naming the tier and its run, and finds
  the run by its title.
- **It waits** within the set's bound (section 3), reading the run
  every minute, and cancels it past the bound.
- **It writes the report:** the failing cases of the run's `junit`, as
  tier main's report has them. A run cancelled, past its bound, or
  ended with no report is a fault, so temper retries it and then holds
  the tier, saying why (`../domain/after-landing.md`, section 4).
- **Its token** is GitHub's, fine-grained, for this repository alone,
  reading and writing its Actions, and kept by the runner's host in a
  private file. It is never in this repository, nor among the
  workflow's secrets on GitHub.
- **A tier held** after its retries reaches whoever keeps the relay, as
  a runner down does today.

## 6. Live on macOS

- **Once a day at main's tip,** beside the live tier's own run, the
  relay dispatches `macos.yml` with the set `live` and reads its report
  beside the live tier's stories. Its red goes to whoever triages reds
  after landing, never to a landing: a day's range is too wide to
  blame, and a live red is often the provider's
  (`../testing/live-tier.md`).
- **The stories:** temper's
  `a_person_asks_claude_on_an_api_key_to_land_a_change_on_the_owned_forge`,
  as `temper/scripts/proof-live.sh forge-land-claude-key` runs it, and
  skein's Anthropic stories, the profile `live-skein`, each on
  `claude-haiku-5-5`, the cheapest stock model
  (`../testing/live-tier.md`).
- **The key** is the user's: the Actions secret `ANTHROPIC_API_KEY` in
  the environment `macos-live`, whose deployment branches are `main`
  alone. Only the `live` job names the environment.
- **The key's file:** after the commit is checked to be on main, the job
  writes the key to a private file in the runner's temporary directory,
  which the stories take by its path, `TEMPER_TEST_LIVE_ANTHROPIC_KEY`
  and `SKEIN_TEST_LIVE_ANTHROPIC_KEY`, as the live tier's stories do. No
  step prints it, and the job removes the file as it ends.
- **The risk, the user's to accept (2026-11-18):** code on main runs in
  the `live` job, where the key can be reached. The key serves these
  stories alone, and its own spend limit bounds what a run can spend.

## 7. Bring-up

1. **The fetch repaired** (section 9): the one story runs again.
2. **The workflow in its new shape,** and one run by hand of the set
   `all` at main's tip: its failures are the inventory.
3. **Each failure fixed or listed:** a test that only Linux can run is
   built for Linux alone, with its reason, and listed in its tree's list
   (`../testing/macos.md`, section 3 for temper; skein's
   `runtime/shell.md`, section 8; smith's and jig's lists made in their
   testing design as their first entries come). One goal per tree. Such
   a goal can't run its tests on a Mac: it lands on its reading of the
   failures, and the next run by hand judges it.
4. **A tier turned on once its set is green,** so that its baseline
   blames no old red on a new landing (`../domain/after-landing.md`,
   section 2): `macos-e2e` first, since its one story already passes;
   `macos-suites` tree by tree, its failing trees' parts left out of
   the set until each is green.

## 8. Reds, priority and budgets

- **A macOS red is a red after landing,** as tier main's: its failures
  run again alone, those that pass are flakes, new ones are bisected and
  blamed on the landing that brought them, and each is filed and
  repaired (`../domain/after-landing.md`, sections 5 and 7;
  `after-landing-repairs.md`).
- **Its repair can't run on a Mac** before it lands: it reads the
  failure, lands, and the tier's next run judges it.
- **Last in priority:** a goal that repairs a macOS red, or brings a
  tree up, takes the lowest priority, and waits for places while marked
  work holds them (the user, 2026-11-18).
- **A red blocks nothing:** no gate reads macOS, as now.
- **Bisection is costly here:** each probe builds its part whole, so it
  costs most of a run. A day's range of about a hundred landings takes
  about seven probes.
- **No budgets on macOS yet:** a run's durations are in its artifact,
  and the user sets macOS's figures from them later.

## 9. Order

The milestones are in `macos-tiers.execution.md`. In short:

1. **The fetch repaired,** the one story running again: landed on
   2026-11-19.
2. **The workflow** of section 4 and the tiers' workflows of 5.1, with
   `../testing/macos.md` rewritten to match.
3. **The relay,** outside this repository, from 5.2's contract: its
   label's runner and its token.
4. **Bring-up** (section 7), tree by tree.
5. **The tiers on,** in the reference deployment's configuration, at a
   restart the user approves: `macos-e2e`, then `macos-suites`.
6. **Live on macOS:** the environment and its key, then the daily run.
7. **`suites` to daily,** after a week green.

## 10. Open questions

- **The key's spend limit:** the user's figure. Default: set on the key
  before the first live run.
- **GitHub's macOS jobs at once:** the organisation's plan's limit.
  Default: five, the Free and Team plans', so that `e2e`'s two parts,
  `suites`' four and `live` queue a little when they meet.
- **A probe's cost:** a bisect over a wide range of `suites` may outlast
  its span. Default: bisection as Linux's; `tell all`
  (`../domain/after-landing.md`, section 11) only if probes pile up.

## 11. Later

- **A repair that runs its macOS test before it lands,** alone and by
  name, as a repair may on Linux: GitHub's mirror holds main alone, so
  it needs its branch there and the relay for a goal.
- **The live tier's Claude Code story and skein's OpenAI stories on
  macOS,** once a runner can hold a program's login or a sign-in.
- **macOS's budgets,** from the runs' durations, the user's figures.
- **A real Mac,** `macos.md`, 2.3.
