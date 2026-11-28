# Sweeps of faults and acts

The sweeps cover each family in the cheapest world that reaches it and mix
all families in the composed world (`workloads.md`, 5).

## 1. The sweeps

### 1.1 One per family, in the cheapest world

A family whose faults fall in different worlds takes a sweep in each
world that is cheapest for some of them, as smith's providers take two
(`smith/docs/design/faults.md`, 5). The sweeps the milestone's other
goals built stay as they are, each its family's in its world: the
forge's and stale state's in the forge world (`blame.md`;
`fakes.md`, 2), the providers' and the worker's in smith's worlds and
the worker's (`smith/docs/design/faults.md`, 5; `worker.md`, 4), the
accounts' in the system world (`checks.md`, 6.3), and the restarts' in
each world (`restarts.md`, 9). This goal adds:

| Sweep | World | Family | Configures | Takes |
|---|---|---|---|---|
| `people_act_on_goals_at_any_step` | the engine's domain world | people and parties | `CANCELS_A_GOAL`, `CANCELS_A_CHANGE`, `CANCELS_A_LANDING`, `HOLDS_A_MEMBER`, `RELEASES_A_MEMBER`, `ANSWERS_LATE`, `NEVER_ANSWERS`, `ANSWERS_TWICE`, `ANSWERS_AN_ENDED_RUN`, `WORDS_TO_AN_ENDED_PRODUCER`, jig's `AMENDS_A_PASSED_DEADLINE` | `open::exclude`; jig's S15 exclusion |
| `people_act_on_the_landing_train` | the engine's forge world | people and parties | `CANCELS_A_LANDING`, `HOLDS_A_MEMBER`, `RELEASES_A_MEMBER`, `CANCELS_A_CHANGE` | `open::exclude` |
| `the_engines_tables_reach_their_bounds` | the engine's domain world | the engine's own bounds | `FORGE_WRITE_ENTRIES`, `FORGE_READ_ADMISSIONS`, `BRANCH_HOLDS`, `LANDING_ROWS` | `open::exclude` |
| `a_restart_changes_no_outcome` | the engine's domain world | restarts, compared (`workloads.md`, section 7) | `temper_faults::restarts::ENGINE` | `open::exclude` |
| `worker_runs_park_resume_and_move` | the worker's domain world | runs that park, resume and move | `DELIVERED_THEN_PARKED`, `RESUMED_ON_ANOTHER_WORKER` | none |
| `the_workers_tables_reach_their_bounds` | the worker's domain world | the engine's own bounds | `WORKSPACES`, `PREPARE_PERMITS`, `PER_RUN_SCRATCH`, `RUN_SLOTS`, `BUILD_PLACES` | none |
| jig's four | jig's core and engine-clients worlds | people and parties; runs that park, resume and move; the engine's own bounds | `jig/docs/design/domain/testing-workloads.md`, 7 | jig's exclusions |
| `every_family_falls_in_the_composed_workload` | the composed world | every family (1.2) | 1.2 | 1.3 |
| `a_restart_changes_no_composed_outcome` | the composed world | restarts, compared (`workloads.md`, section 7) | `temper_faults::restarts::MID_TURN` | none |

- **The loop** is skein's (`skein/docs/design/testing/sweeps.md`, 4): for
  each seed it writes the seed to stderr, runs the world with
  `Streams::new(seed)`, folds the run's ledger and returns its findings
  at once; after the last seed, `check_fell`. A comparison's seed folds
  the ledger of its run with restarts.
- **A kind no draw of the world can reach** at its limits is not
  configured: it is named, with why, in a comment beside its sweep's
  `configure` calls, as smith's sweeps name theirs
  (`smith/docs/design/faults.md`, 5), and the design's status says so.
- **`A_CHANGE_WAITS_ON_ANOTHER_GOAL` takes no sweep:** the workload's
  milestone starts a goal only once the goal it depends on has ended, so
  the `after` its first change names on the earlier goal's change never
  waits (`workloads-kit.md`, 1.1), and a draw that started the later goal while the earlier
  lives would change what a milestone is. The family's other kinds fall
  in the worker's sweep and jig's.
- **A sweep's checks** are its world's as they stand, with `Ends` over
  every goal where the world runs goals, and its acts' expectations.

### 1.2 The composed sweep mixing every family

`every_family_falls_in_the_composed_workload` configures one kind of each
family, the one production met most, so that every seed can mix them
all and the fewest seeds make each fall:

| Family | Kind | Drawn from |
|---|---|---|
| busy | `temper_faults::forge::TIMED_OUT` | `Faults`, `forge` |
| lagging | `temper_faults::forge::HEAD_BEHIND` | `Faults`, `forge` |
| bounds | `temper_faults::forge::JOB_LOG` | `Faults`, `forge` |
| refusing | `temper_faults::forge::CLOSED_BY_ANOTHER` | `Faults`, `forge` |
| ci | `temper_blame::LOST` | `Faults`, `forge` |
| stale | `temper_faults::stale::MOVED_BASE` | `Faults`, `forge` |
| provider | `skein_llm_kit::faults::RATE_WINDOW` | `Faults`, `llm` |
| worker | `temper_worker_inspect::faults::DISK_FULL` | `Faults`, `worker` |
| people and parties | `HOLDS_A_MEMBER` and `RELEASES_A_MEMBER`, on the train | `Parties` |
| runs that park, resume and move | `DELIVERED_THEN_PARKED` | `Faults`, `worker` |
| a session's state at a park | `smith_faults::parks::JOB_RUNNING` | `Faults`, `cells`, armed with the cell |
| parks against wakes | `jig_core_kit::cells::JOB_RUNNING_PARKS_ON_NOTHING` | `Faults`, `cells` |
| the engine's own bounds | `FORGE_READ_ADMISSIONS` | observed (`workloads.md`, 2.4) |
| restarts | `ENGINE_CRASH_MID_TURN` | `Faults`, `restarts` |

- **Each seed** draws every family's kind, its moment a drawn step of the
  goal it strikes, from its own part, so that dropping a family leaves the
  others' draws as they were. Each fault stops by a moment the run names,
  so every goal expects `Ends` past the faults' last moment, but a held
  member's goal until its release (`workloads.md`, 6.2).
- **Where the composed world can't make a kind** (its worker keeps no fake
  disk, say), the sweep takes the family's next kind the world reaches,
  named in its comment with why: `PLACEMENT_BUSY` for the worker; another
  table the workload fills for the bounds. Every family keeps one kind.
  Three of the table's kinds are such replacements: `JOB_LOG` stands for
  `PULL_FILES`, since the composed workload's repairs read their gate's
  evidence and never a pull's files; `MOVED_BASE` stands for `SQUASHED`,
  since a dependent goal starts once its predecessor has landed, so no
  checkout taken early is left with a squashed parent;
  `FORGE_READ_ADMISSIONS` stands for `FORGE_WRITE_ENTRIES`, since the
  composed workload makes too few concurrent writes to fill the stock
  write table, and its faulted world bounds the read admissions to two,
  recorded at its quiet points (`workloads.md`, 2.4).
- **The kinds are a list the caller gives,** and each family's draw keeps
  a fixed place in its part, so that leaving one kind out moves no other
  kind's draw.
- **The two halves' kinds of the cell** are recorded as they fall: smith's
  `JOB_RUNNING` is its family's kind here, and jig's `PARKS_ON_NOTHING`
  falls beside `DELIVERED_THEN_PARKED`, which stays its family's kind. The
  cell is the production incident's (`../draft/misbehaving-agents.notes.md`,
  *The incident and its fix*), and is configured only where the gate's
  seed shows it falls (`../draft/misbehaving-agents.execution.md`, 4).
- **The draft's failure table** (`../draft/failure-worlds.md`, 3.3) holds
  for runs where the provider's fault falls, by the account runs' judge
  (`checks.md`, 6.3); for gates where CI's falls, by `temper_blame`
  (`blame.md`).

### 1.3 Exclusions

- **Each sweep takes the exclusions** of the open problems its kinds
  strike, as their owners give them: the engine's
  `temper_engine_forge_world::open::exclude` (`engine-checks.md`, 5.2),
  jig's `jig_core_kit::open::EXCLUSIONS` and
  `jig_engine_clients_kit::open::EXCLUSIONS`
  (`jig/docs/design/domain/testing-fakes.md`, 1.3), each acting before its
  fault falls, its conditions read through the adapters or the world's
  state at that moment, and nothing broader (`../draft/failure-worlds.md`, 6).
- **P124's first combination,** cancelling a landing while its gate's
  dispatch is in flight, takes no exclusion: its fix landed, and
  `cancelled_landing_frees_its_write_entries` is active in the forge
  world's suite (`engine-checks.md`, 5.1).
- **A finding** fails its sweep, as any red. One that is the sweep's own
  defect is repaired in its change. One that is the product's and outside
  the draft's catalog is asked up before anything is written for it; once
  confirmed, it takes its scenario in its lowest harness, `#[ignore =
  "failure worlds: <what failed>, repaired as a red"]`, and the narrowest
  exclusion its world reads, so that the sweep holds everything else and
  the problem is repaired as a red (the request, *Out*).

### 1.4 Room

- **The user's figures:** each sweep runs in its world's fuzzy suite, in
  its binary's layer, within `tools/tiers/budgets.toml` as it stands,
  with the moves the user made from F7's room on 2026-11-22
  (`../../development/failure-worlds-room.md`, *The moves for the user*).
  This design raises nothing.
- **Seeds:** the fewest at which every kind it configures falls, at
  least one, pinned in the test.
- **Measured:** each change gives its new sweeps' CPU and instructions
  from the suites' own reports. A layer its sweeps take over its budget,
  or past its growth allowance, is red (`tools/tiers/budgets.md`, 8), and
  is repaired as `skein/docs/foundation/testing-strategy.md`, 8.1 says:
  the same checks for less where they can be made faster; otherwise the
  measurements go to the user as a question. No sweep takes fewer seeds
  or checks to fit.

### 1.5 In a campaign

The engine domain world's walking, opening and follows sweeps, and the
forge world's sweeps, take their cases from skein's seam,
`skein_world::sweep::cases` (`skein/docs/design/testing/sweeps.md`, 4.2 to 4.5),
so that a campaign runs them with fresh seeds after landing
(`temper/docs/design/draft/fuzzy-campaigns.md`, 9). `campaign-temper`
lists their binaries, `temper-engine-domain-world::suite` and
`temper-engine-forge-world::suite` (`tools/tiers/layers.md`, 9.3). At the
gate each does the work it did before:

- **Its identity:** each test passes its own, `this_test!` where it is
  written, and a shard's is its test's. A test that judges several worlds
  at each seed, as the CI faults' sweep judges three, gives each world a
  part and its own `cases` over the same seeds, so that each world's case
  replays alone.
- **Its gate seeds** as before, in their order, with the same draws. A
  sweep that offsets its seed (walking's, opening's and follows') wraps
  the sum, so that any `u64` draws a case, and its fuzzy module pins the
  gate seeds' draws by a digest and draws one at `u64::MAX`. A choice
  made by the seed's value (the CI sweep's `seed == 1`) takes its parity
  instead, which keeps the gate's choices and gives a campaign both.
- **Every seed a valid case** (skein/docs/design/testing/sweeps.md, 4.5): a draw its world can't
  run at a fresh seed, a push past the fake forge's commit room or a task
  past the task kit's table, is skipped, and the case goes on. The gate's
  seeds never reach one, so they draw as before.
- **Its checks over the loop** wait for `gate()`: what a sweep fills or
  counts across its seeds, as the opens' mask of what was exercised, the
  queue histories' kinds of event, the pull cutoff's counts, the forge
  restarts' gate dispatches and a count an exclusion left out.
  `check_fell` is skein's, and writes the reach outside the gate. A
  case's own checks, its referee, its replay's comparison and its
  invariants, run in every mode.
- **Its counts:** each case one `seed` check (skein/docs/design/testing/sweeps.md, 4.2). A sweep
  that counted its seeds through `each_seed` or `Sweep::ran` counts the
  same, and one whose loop was its own, counting nothing, gains its
  cases' count: its binary loses none.
- **Fixed scenarios keep their seeds** and take no case: the forge
  world's `people_act_on_the_landing_train`,
  `answers_past_bounds_leave_repairs_their_evidence` and
  `refusals_keep_the_batches_invariants` script what their seeds were
  chosen to show. A campaign runs them as the gate does.

The composed workload's two sweeps, `every_family_falls_in_the_composed_workload`
and `a_restart_changes_no_composed_outcome`, take their cases from the
seam too, now that its draws are valid at any seed (`workloads.md`, section 5), and
`campaign-temper` lists their binary, `temper-system-world::suite`:

- **Their gate seeds** as before, 6 for the mixed sweep and 0 and 29 for
  the comparison, each case's checks those of 1.2 and `workloads.md`, section 7, its
  `Ends` and its oracle.
- **More cases in a campaign** (skein/docs/design/testing/sweeps.md, 4.2), since seeds aren't
  scarce there (`../draft/fuzzy-campaigns.md`, 6): the mixed sweep asks 16 and the
  comparison 8, each of the comparison's cases two runs. A count whose
  cases, at the gate case's measured time, would pass 400 s, half the
  pair's backstop of 810 s, halves until they fit.
- **Its catalogue in a campaign:** where `gate()` holds, the mixed sweep
  configures 1.2's kinds; otherwise those and its catalogue's (1.6).
- **Its counts:** each case one `seed` check. The mixed sweep counted
  its seed twice, by `each_seed` and by `Sweep::ran` outside a case, and
  counts it once: the lost check takes a `duplicate` reason
  (`tools/tiers/budgets.md`, 13), as smith's run world's did. The
  comparison counted each seed once, by `Sweep::ran`, and counts the
  same.
- **The binary's other fuzzy tests,** the accounts', the seeded system
  runs', the green light's, the repair's and the mid-turn restarts', take
  no case yet and run in a round as the gate runs them, each named with
  its time (`tools/tiers/campaign.md`, 3). The composed sweeps' cases are
  the bulk of the binary's round; should a round's times show the others
  outweighing them, those move to the seam or to a binary of their own
  (`workloads.md`, section 11).

### 1.6 The composed sweep's catalogue

In a campaign, `every_family_falls_in_the_composed_workload` configures
more kinds than 1.2's one a family: its catalogue, a kind at a time as
the composed world learns to inject it, give it an eligible target,
exclude what its open problems need and say what each goal expects of
it (`../draft/fuzzy-campaigns.md`, 6). The catalogue, its draws and its kinds
are in `composed-catalogue.md`. The gate's list doesn't change.

## 2. Open questions and later work

The question of room in instructions and the tests still to move to the
campaign seam stay with the workloads (`workloads.md`, 10 and 11).
