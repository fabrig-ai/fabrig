# temper's conformance world

The concrete engine root runs on jig's conformance harness and outside
referee. Store rows come from `jig-fake-store`; workers retain their
attempts, charges and unacknowledged calls across engine crashes.
Restoration uses the engine's real bounded load requests. The world plays
the protocol layer with the engine world's stand: the hub's link records,
tool decodes and renders, ended outcomes and accounts; an input that admission
hands back is sent again. The adapter's
timer probes drive the root between external inputs; its bookkeeping adds
no task or forge decisions. The fixture uses bounded journal cohorts, a
one-second landing queue window and write lifetime, with restore pages
bounded to 32 rows and 64 KiB. Each shared scenario grants its real push
workspace and the effects it exercises; issue projections are not configured
for stories without goals.

The independent fake forge receives translated native API operations. Its
observations retain actual pre-write state, actor ownership, server replies and
native changes. Released writes require durable outbox entries and attempts.
The root contributes no effect verdicts. Recovery declarations follow the
native client's classes and `docs/design/domain/forge.md`, section 6.

| Shared scenario | temper's binding |
|---|---|
| LateCopy | An idempotent commit status set and a retained transport copy |
| OtherHand | A real producer push, an opened pull, a decided merge and another actor's push |
| UncertainRestarts | An idempotent status flight whose absolute attempt deadline survives three cold starts |
| ChangedJudge | Real CI changes after a merge decision; the committed flight remains decided and another change cannot merge |
| NarrowingPolicy | An authenticated owner adds a requirement; a committed status flight resolves while a pending proposal and a fresh call are checked again |
| StandingTask | Recurring agent batches under successive funding periods, with duplicate period notifications |
| RunBudget | Twelve independently charged completions sharing one run allowance |
| ShrinkingPool | Not applicable: temper's forge has no pooled holds |

| Native story | Independent ending and focused cuts |
|---|---|
| ReviewedChange | CI and a human approval at the merged head; cuts 1 and 17 |
| RepairCi | Failed CI, a real repair push and approval at its new head; cuts 1 and 23 |
| GoalIssue | Owned write-only issue, subtree milestones and closure; cuts 21 and 23 |
| OtherHand | Another actor's branch movement holds the change without overwriting it; cuts 1 and 17 |
| LostMerge | Lost successful merge answer found at its head after restart, once; cuts 1 and 18 |
| PersonWords | Client words delivered live or retained in the next activation's inbox after a lost relay; cuts 1 and 9 |
| IssueCreation | Lost native creation answer found by its retained marker, once; cuts 1 and 7 |

The forge kinds follow each row of that section. In particular,
creation is kind 8, issue editing/closing/reopening is kind 13, pull opening
is kind 3, editing is kind 10, updating from the base is kind 11, merging is
kind 4 and closing/reopening is kind 12. Branch deletion is kind 15. Fixtures
grant the writes they exercise; the order has no implications between these
kinds. The native client and the independently mapped fake writes agree on
their recovery classes.

The fake retains actual lost-answer and timeout uncertainty. Cold-start
observations also identify durable attempted entries whose acknowledgements
may have been lost. Subsequent durable erasure, failed projection rows and
named held outcomes establish settlement or a hold for that retained entry.
Already settled keys cannot be relabeled by a later projection failure; no
live root verdict is consulted. GoalIssue accepts a held projection only
with independently retained uncertainty for an unrecoverable creation or
milestone comment. Client words
supply delivery obligations as well as durable inbox and actual worker receipts.

Focused tests run each baseline and the configured cuts above. Shared stories
keep three selected entry, middle and closing receipt cuts; their baselines run
separately instead of being repeated to calculate those cut numbers. Each
focused test asserts its selected cut fired. The fuzzy tests crash every
applicable shared and native baseline at every durable commit, one test per
baseline. Both suites run
the same outside referee after every harness iteration. The narrow engine-domain
world retains CI job-log contents/truncation and race/permission checks that
these stories do not exercise.

The world's allocation, measured serially (`--profile measure -j 1`), is 4
seconds (2 seconds before the user's doubling of 10-22, provisional) focused
and 40 seconds (20 seconds before the user's doubling of 10-22, provisional)
fuzzy. Both suites keep all fourteen applicable stories, all 392 shared and
native commit cuts, and the same outside observations and referee.