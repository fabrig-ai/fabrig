# The composed sweep's catalogue

The kinds that temper's composed sweep, `every_family_falls_in_the_composed_workload`, configures in a campaign beyond its gate's one kind a family (`workloads.md`, 8.2 and 8.6): how a campaign's case draws them, which kinds the composed world injects, each with its injector, eligible target and exclusions, and what each goal expects of them (`temper/docs/design/draft/fuzzy-campaigns.md`, 6).

## 1. The catalogue

- **The gate's list, then the catalogue:** at the gate the mixed sweep configures 8.2's thirteen kinds; in a campaign it configures those and the catalogue's (section 2). A replay, whose `gate()` is false, configures the campaign's (`skein/docs/design/testing/sweeps.md`, 4.3).
- **A case's mix:** a campaign's case plans 8.2's kinds as the gate does, each on a goal eligible for it (`workloads.md`, 5), and each catalogue kind on a coin, so that a case mixes every family's kind with a drawn handful of the rest, rather than every kind at once. A bounds kind bounds its table low only in a case that plans it.
- **Its own part:** each catalogue kind draws its coin, its goal and its moment from `Stream::Faults`, part `FAMILY/NAME`, its family and name as its `Kind` holds them (`busy/unavailable`), the coin first. So adding a kind moves no other kind's draw, and 8.2's kinds draw as they did, in their places of their parts (8.2).
- **An observed kind** that the world injects nothing for, `WRONG_TREE`, takes no coin: configured, it falls wherever its target's change goes red, as the world records it today, and a case with no target for it records it ineligible.
- **Valid at any seed,** as `workloads.md`, 5 makes every draw: a planned kind with no eligible goal, or whose moment has gone when it would fall, isn't injected, and its ledger records it ineligible.
- **What each goal expects:** every kind here is finite: it stops by a moment the run names, as 8.2's do, so every goal still ends done with its changes landed, as the composed world's oracle reads them (`workloads.md`, 5), its `Ends` past the faults' last moment. Each family's judge holds where it has one: CI's kinds by `temper_blame`, as `blame.md` blames each, a runner's fault retried with no repair and a change's fault repaired, but a silent gate, which is held, not retried: its run past `gate_timeout` holds the landing, naming the runner (`../domain/landing-proof.md`, 2.1), the world cancels that run as another user and its party releases the hold, and the new round proves its keys as any round does, by a receipt or a fresh dispatch (`../domain/landing.md`, 4.2), with no repair; the provider's by the account runs' judge (`checks.md`, 6.3); the bounds' by their saturation, recorded at quiet points (`workloads.md`, 2.4). The landing rows' table is the one bound whose fall never ends as 2.4 reads it: the engine keeps a landing's row once it has ended and replaces one only for a new admission (`../domain/landing-turn.md`, 2, Bounded), so a table the workload fills stays at its capacity once every goal has landed. The composed world starts its saturation as 2.4 does, by the table's count, and ends it at the first quiet point after the workload's goals have all ended at which every row the table keeps has ended and is reclaimable, its entry `Release::Kept` (`engine-checks.md`, 2.3): from there no landing waits for its room. Nothing in the product changes for it, and no exclusion is taken.
- **Its exclusions** are those its other sweeps take for it, the same `Exclusion`, never a copy: one a forge world's sweep registers for itself moves to `temper_engine_forge_world::open::exclude` (`engine-checks.md`, 5.2), which each sweep calls with the kinds it configures. The composed world reads each exclusion's conditions as the fault would fall, and leaves the fault out only while they hold (`skein/docs/design/testing/sweeps.md`, 5).
- **A kind joins in a change of its own,** once the composed world injects it, gives it its target, takes its exclusions and holds its expectation, as section 3 shows.

## 2. Its kinds

Each is injected as the kind of 8.2 it names is, or as its other world injects it, so the composed world keeps one injector for each mechanism. The busy, lagging, refusing and stale kinds strike the forge's calls through the composed world's forge reply; CI's, its gate runs; the provider's, its party's account; the worker's, its fake checkout.

| Kind | Injected as | Its target | Exclusions |
|---|---|---|---|
| `forge::UNAVAILABLE` | `TIMED_OUT`, a forge call answered unavailable for a stretch | any goal | none |
| `forge::LIMITED` | `TIMED_OUT`, a forge call answered rate-limited for a stretch | any goal | none |
| `forge::BASE_BEHIND` | `HEAD_BEHIND`, the base's view lagging its pushes for a stretch | any goal | none |
| `forge::MERGE_LATE` | `HEAD_BEHIND`, a merge's record arriving late | the train goal | the forge world's lagging sweep's `a_late_manual_merge_record_finishes_the_landed_batch`, while a squashed member already landed on its base is recorded |
| `forge::CLOSE_REFUSED` | `CLOSED_BY_ANOTHER`, the engine's close of a pull refused for a stretch | any goal | `open::exclude`'s `a_failed_capability_refresh_reads_again`, while the refused close occupies a slot in a full forge client call table |
| `forge::MERGED_BY_ANOTHER` | `CLOSED_BY_ANOTHER`, another user merging a pull | a goal that isn't the train | the forge world's refusing sweep's `a_member_merged_by_another_lands_like_any_landing`, while its change is ready or queued for landing, or a batch member, until its record |
| `stale::REOPENED` | `CLOSED_BY_ANOTHER`'s outside write, another user reopening a pull the engine closed | any goal | none |
| `temper_blame::WRONG_TREE` | observed as today, a candidate holding a wrong tree, with no coin (section 1) | a goal with a `red_once` change | none |
| `temper_blame::SET_UP` | `LOST`, the fake forge's gate job failing its set-up | any goal | none |
| `temper_blame::SILENT` | `LOST`, a gate that never reports | any goal | none |
| `temper_blame::MISSING_REPORT` | as `blame.md` gives it, a failed run of a gate that reads a report completing with none uploaded | a goal with a `red_once` change | none |
| `skein_llm_kit::faults::SPENT_CREDITS` | `RATE_WINDOW`, the party's account answering with spent credits until restored, as the system world's account sweep plans it | any goal | none |
| `skein_llm_kit::faults::EXPIRED_SIGN_IN` | `RATE_WINDOW`, the party's sign-in expired until renewed, as the account sweep plans it | any goal | none |
| `temper_worker_inspect::faults::FETCH_CUT` | `DISK_FULL`, the worker's fake checkout cutting a workspace's fetch partway, as the worker's domain world cuts it | any goal | none |
| `bounds::BRANCH_HOLDS` | `FORGE_READ_ADMISSIONS`, the branch holds' table bounded low, its saturation recorded at quiet points | observed (`workloads.md`, 2.4) | `open::exclude`'s `a_prepare_is_admitted_when_branch_holds_are_released` |
| `bounds::LANDING_ROWS` | `FORGE_READ_ADMISSIONS`, the landing rows' table bounded low, its saturation ended once every kept row is reclaimable (section 1) | observed | none |
| `restarts::ENGINE_RESTART_MID_TURN` | the comparison's settled stop at a held call (`workloads.md`, 7), as the mixed sweep's `ENGINE_CRASH_MID_TURN` is its crash | any goal | none |

## 3. Testing

Focused and fuzzy suites only, and nothing at the gate's cost: the gate's sweep runs 8.2's kinds alone, so a catalogue kind adds no work there.

- **Its draw** (`temper-system-world`'s step tests, with no world): over a range of seeds and at `u64::MAX`, the kind plans on its coin, on a goal eligible for it, or is recorded ineligible; it leaves every other kind's plans as they were; seed 6's gate draw is unchanged.
- **The gate's sweep** passes, unchanged, at seed 6.
- **A case that plans it,** replayed by name with `SKEIN_TEST_FUZZY_CASE` set to a seed whose draw plans it on an eligible goal (sweeps.md, 4.3): its reach line counts the kind in one case, and the case's checks hold. The change names the seed and the lines; no campaign runs it.
- **No focused composed case** pins a catalogue kind, which would put its cost at the gate. The gate reaches a catalogue kind in the cheapest world that injects it, where one does: the forge world's refusing sweep configures `CLOSE_REFUSED` among `REFUSING` (`workloads.md`, 8.1). And a fault the composed world finds is pinned in the lowest tier that shows it, never in the composed world (`skein/docs/foundation/testing-strategy.md`, 8 and 9): the capability refresh's, by `open::exclude`'s `a_failed_capability_refresh_reads_again` (`engine-checks.md`, 5.2). So `CLOSE_REFUSED` falls in the composed world in a campaign's cases alone (`temper/docs/design/draft/fuzzy-campaigns.md`, 6).

## 4. Later

- **Kinds whose injector is a mechanism the composed world lacks:** busy's `NO_ROOM`, `LANDING` and `LATE`; CI's `RUNNER_WAIT`, `CANCELLED`, `STARVED` and `BASE_RED`; the provider's hangs, `HANG_AFTER_HEAD` and `IDLE_MID_STREAM`, and smith's `WINDOW_ELSEWHERE`; the worker's `TRASH_STUCK`, `PLACEMENT_BUSY` and `CHECK_WAITS`; the worker's tables; the engine's restarts beyond mid-turn, the forge's and the worker's; `RESUMED_ON_ANOTHER_WORKER`, which needs a second worker.
- **People's acts beyond the train's hold and release:** cancelling a goal, a change or a landing, answering late, twice or never, words to an ended producer, amending a passed deadline. Each changes what its goal expects (`workloads.md`, 6.2), so the composed world's oracle first takes each goal's expectation rather than every goal done.
- **Not reachable by the composed workload,** each with why: `PULL_FILES` and `SQUASHED`, as 8.2 says; `COMPARISON`, whose read the workload's repairs never make; `A_PATH_DELETED_ON_THE_BASE`, with no scripted resolution; `FORGE_WRITE_ENTRIES`, whose table the workload never fills; `A_CHANGE_WAITS_ON_ANOTHER_GOAL`, as 8.1 says; the provider's cuts and `UNSERVED_MODEL`, which wait on their open problems' fixes; `BILLING_REFUSAL`, `EDGE_REFUSAL` and `UNTOOLED_PROGRAM`, with no program backend in the composed world; the restarts with no cut the composed world exposes, `STOP_BEFORE_OPENING`, `UNIT_SIGNALLED` and `OLDER_LAYOUT`.
