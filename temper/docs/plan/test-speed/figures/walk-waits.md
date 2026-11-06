# Fake-box walk waits

Base: `70c981f961377cc5373dc068683b32c87e8d3c42`. Measurements on
2026-11-06, one builder slot, `debian-32gb-hel1-14`. The process profile
runs one test at a time. CPU means user plus system CPU of the test and
its descendants, collected by `tools/tiers/run --cpu`.

The short-paces diagnosis holds for the status walks' landing ranges.
Goal and Away instead wait primarily for a retained chat activation and,
for Away, the designed Undo periods. They already wake on observations;
shortening the forge paces did not remove those waits and was not kept.

## Timeline before the repair

Captured before changing the walks: ctl start/completion times and native
client page transitions, correlated with each run's `box.log` (UTC,
whole-second log stamps). Offsets below are seconds from the test's first
box start in the second base sample. Temporary tracing was removed.
The ranges partition the whole walk, including CPU work; they do not
pretend that wall minus aggregate descendant CPU measures sleeping time
on a multicore machine. The named blocking ranges cover more than 90%
of each walk's idle wall, with startup and shutdown included separately.

### Away

`a_goal_decides_while_away_then_waits_on_home_after_overturn_and_off`.
Box log: start 10:43:02Z; stopping and stopped 10:43:27Z.

| Offset | Wall | What the range waits for |
| --- | ---: | --- |
| 0–0.44 | 0.44 | Empty box start and owner sign-in |
| 0.44–0.83 | 0.39 | Connect, key, usable-account watch, settings show/apply |
| 0.83–2.81 | 1.98 | First native visit: Skip, settings selection, Away enabled |
| 2.81–3.30 | 0.49 | Start chat, watch its reply, read chat and goal |
| 3.30–13.55 | 10.25 | Second native visit: goal's carried-on text; the chat retains the only worker slot until its 10 s wait ends |
| 13.55–14.57 | 1.02 | Home, decision, overturn, Away off |
| 14.57–19.84 | 5.27 | Next question after answering the overturned question; the client's 5 s Undo hold |
| 19.84–25.03 | 5.19 | Answer reaches goal and Home loses the goal; the second 5 s Undo hold |
| 25.03–25.19 | 0.16 | Ended-task watch, transcript read, healthy box stop |

The three long second-visit ranges total 20.71 s. The first contains the
stock charter's 10 s wait while the chat occupies the only worker slot;
the next two contain Home's deliberate 5 s Undo holds. The native client
already waits on changed pages. These are a person's waits too, with
reasons that still hold, not missed forge wakes.

### Goal report

`a_chats_goal_on_the_fake_box_shows_its_report_in_the_goals_list`.
Box log: start 10:43:27Z; stopping and stopped 10:43:39Z.

| Offset | Wall | What the range waits for |
| --- | ---: | --- |
| 0–0.39 | 0.39 | Empty box start and sign-in |
| 0.39–0.71 | 0.32 | Connect, key, usable account, start chat |
| 0.71–1.09 | 0.38 | Chat reply watch and conversation read |
| 1.09–11.48 | 10.39 | Goal-ended watch; the chat retains the only worker slot for its 10 s wait |
| 11.48–11.55 | 0.07 | Settings show/apply |
| 11.55–11.63 | 0.08 | Native Goals list, landed row, report page |
| 11.63–11.67 | 0.04 | Client drain and box stop |

The single 10.39 s range holds essentially all idle wall. The ended-task
watch remains the oracle. Short forge paces do not remove the retained
activation's wait, so this walk keeps its empty first start and its
shipped paces.

### Personal-plan status

`a_plans_project_has_its_brief_on_a_return_past_the_sitting`.
Box logs: preparation 10:43:39Z; working box 10:43:39–10:44:27Z;
return box 10:44:28–10:44:32Z. Owned Forge 1 starts at 10:43:40Z,
is started at 10:43:43Z, and starts again at 10:44:28–29Z.

| Offset | Wall | What the range waits for |
| --- | ---: | --- |
| 0–0.73 | 0.73 | Preparation, reopen, owner sign-in |
| 0.73–6.12 | 5.39 | `forge setup` provisions and starts the owned Forgejo and runner |
| 6.12–6.49 | 0.37 | Plan connect, browser return, usable account, chat start |
| 6.49–16.94 | 10.45 | Chat-idle watch; retained chat activation releases the one worker slot after 10 s |
| 16.94–37.00 | 20.06 | Change-landed watch: real git, owned forge, checks and review |
| 37.00–38.10 | 1.10 | Project/account reads and Home's landing text |
| 38.10–42.16 | 4.06 | Deliberately cross the 3 s sitting plus its 1 s wall-clock margin |
| 42.16–47.58 | 5.42 | Status agent awaits the worker slot held by the chat resumed on landing, then writes its brief |
| 47.58–48.69 | 1.11 | Owned box shutdown |
| 48.69–53.41 | 4.72 | Restart owned forge, sign in, read retained personal account, stop |

These ranges account for the whole walk. The sitting is intentional; its bound is not shortened. The final trace
confirms the second chat activation's 10 s wait, overlapping the first
Home visit and the deliberate sitting gap. The brief's remaining 5.42 s
is that activation's remaining wait, not a forge poll. This walk
provisions an owned Forgejo and runner and runs real git in its long
setup, landing and restart ranges; the exact per-process CPU split is
unknown. Their CPU implementation is outside this task.

### Home stored count

`home_on_the_fake_box_keeps_a_landing_count_after_box_and_client_restarts`.
Box logs: preparation 10:44:32–33Z; working box 10:44:33–55Z;
restart 10:44:55–59Z. No owned Forgejo or runner starts in this walk.

| Offset | Wall | What the range waits for |
| --- | ---: | --- |
| 0–0.68 | 0.68 | Preparation and reopened owner sign-in |
| 0.68–1.14 | 0.46 | Connect, re-adopt checked repository, key, usable account |
| 1.14–2.16 | 1.02 | First Home visit and Skip |
| 2.16–2.21 | 0.05 | Start chat |
| 2.21–12.51 | 10.30 | Chat-idle watch; the retained chat activation holds the only worker slot for 10 s |
| 12.51–20.99 | 8.48 | Landed-change watch, through fake forge and real child processes; sensitive to forge polling load |
| 20.99–23.07 | 2.08 | Two client restarts showing computed count; stop box |
| 23.07–24.64 | 1.57 | Restart box and sign in on the retained store |
| 24.64–26.71 | 2.07 | Two more Home visits and healthy stop |

Home is slow even with 100 ms polling: the 18.78 s task/landing ranges
dominate. The final trace separates the invariant 10 s chat wait from
the landing watch, which falls from 8.48 s to 0.80 s on the 1 s pace.
The four client visits already use changed-page observations. No
assertion or deadline changes.

### Coverage and causal checks

The tables partition the entire measured wall, leaving no unclassified
long gap. The named long waits cover at least 90% of the idle-wall proxy
used by the survey (wall less descendant CPU): Away's 20 s of charter
and Undo waits against 20.24 s; goal's 10 s against 8.01 s; plan's
10 s first activation, 4 s sitting and roughly 5 s remaining resumed
activation against 17.06 s; Home's 10 s activation and roughly 7.68 s
poll-sensitive landing delay against 18.52 s, with its four page waits
accounted for separately. A sum of parallel descendants' CPU is not a
measurement of an individual interval's idle time; these are coverage
checks, not additive per-interval CPU claims.

The causal trace then checked agent starts, Waiting and exits beside
ctl acts and UI transitions. For example, Away's chat reported Waiting
at 11:12:20.793Z, exited at 11:12:30.793Z, and the goal spawned at
11:12:30.913Z. The box's one worker slot is the init template's, not a
changed bound. In the answer trace, UI choices at 11:02:05.483Z and
11:02:10.744Z reached the engine at 11:02:10.489Z and 11:02:15.750Z:
the designed Undo hold (`temper/docs/design/client/README.md`, 3.5),
not a missing wake. The charter wait is
`temper/docs/design/domain/agent.md`, 4.6.

A trial giving Away the status walks' original 100/500/1000/100 ms
paces took 24.648 wall / 5.878 CPU s, against the base median
25.080 / 4.887. Its wall gain was smaller than its CPU increase, so it
was removed. A separate 100 ms issue-projection interval trial took
25.488 / 6.463 and was also removed. A native-socket close experiment
passed a lower jig test but left Away at 24.474 / 6.403; it was not this
walk's cause and all its product/test edits were removed. No product
repair is claimed, and no new test remains in this branch.

### Pace and first-start decisions

Both status walks share `Walk::short_paces`: normal, maximum and slow
poll paces of 1000 ms, hinted reads at 100 ms. Their first empty run
prepares the box, then they reopen the retained public configuration.
That keeps the first-start contract; importing a source configuration
would change it. The preparation costs 0.38 s in the base Home trace
and 0.40 s in the base plan trace, about 0.48 s in each final trace.
No extra start is added to the two status walks. Goal and Away do not
pay for an unhelpful reopen.

The status paces are deliberately longer than the prior 100 ms normal
pace: the walk has to leave room for its journal and request budget
between reads. With 500/1000/1000/100 ms, plan took 44.322 wall /
34.960 CPU s and Home 19.300 / 7.306. With
1000/1000/1000/100 ms, they took 45.927 / 36.494 and
18.745 / 6.221. The latter retains the gain with fewer normal reads;
repeated figures below carry more weight than either single trial.
The change adjusts these explicitly permitted paces only: no capacity,
CPU budget, action deadline, charter wait, Undo hold or sitting is raised
or shortened. Every assertion of the four walks stands.

## Timelines after the repair

The same ctl/page boundaries in the first final sample, before and after
in seconds. The other ranges keep their startup, account, configuration,
client and shutdown observations. All four walks passed.

| Range | Base | Final | Interpretation |
| --- | ---: | ---: | --- |
| Away: second native visit | 21.73 | 21.74 | Retained chat and two Undo holds remain |
| Goal: goal-ended watch | 10.39 | 10.79 | Retained chat still owns the single worker slot |
| Plan: chat-idle watch | 10.45 | 11.50 | Stock waiting period and scheduling remain |
| Plan: landed-change watch | 20.07 | 11.95 | Longer paces reduce polling load during the real change |
| Plan: deliberate sitting gap | 4.06 | 4.04 | Same 3 s sitting plus 1 s margin |
| Plan: brief page watch | 5.42 | 5.49 | Resumed chat must release the worker slot |
| Home: chat-idle watch | 10.30 | 10.36 | Stock waiting period remains |
| Home: landed-change watch | 8.48 | 0.80 | The dominant avoidable wait disappears |
| Home: four client page watches | about 4 | about 4 | Same first look, returns and restart assertions |

The final plan trace has its chat Waiting at 11:13:06.638Z and exiting
at 11:13:16.633Z before the coder starts. After landing, its resumed chat
is Waiting at 11:13:29.044Z and exits at 11:13:39.042Z; the status agent
starts at 11:13:39.085Z and exits at 11:13:39.162Z. Thus the brief's wait
has an observed owner and end event. Home's box log continues to show
only the fake peers and serve, not an owned Forgejo or runner.

## Repeated figures

Base code: `70c981f961377cc5373dc068683b32c87e8d3c42`.
Final code: `a52522563cae60574766e6d9844fdf8b91f596d4`.
All on the same slot and machine, with 16 CPUs online. CPU is the sum of
user and system time of the test and its descendants; build time is
excluded. Each cell is **wall / CPU seconds**.

B1 is the full base tier; B2 and B3 are four-walk samples with temporary
ctl/page tracing. F1 is a four-walk sample with that tracing plus agent
Waiting/start/exit markers; F2 is uninstrumented. F3 is the required
12-walk selection, F4 the full final tier. All final samples use the
same code change. The diagnostic status sample at base additionally
measured Plan 51.174 / 26.258 and Home 26.758 / 9.119.

| Walk | B1 | B2 | B3 | Median |
| --- | ---: | ---: | ---: | ---: |
| Away | 25.080 / 4.887 | 25.205 / 4.965 | 25.005 / 3.911 | 25.080 / 4.887 |
| Goal | 11.809 / 4.084 | 11.688 / 3.678 | 12.735 / 2.490 | 11.809 / 3.678 |
| Plan | 52.043 / 40.956 | 53.430 / 36.369 | 63.468 / 28.153 | 53.430 / 36.369 |
| Home | 26.957 / 8.728 | 26.728 / 8.211 | 28.149 / 6.047 | 26.957 / 8.211 |

| Walk | F1 | F2 | F3 | F4 | Median |
| --- | ---: | ---: | ---: | ---: | ---: |
| Away | 25.556 / 3.825 | 24.068 / 4.738 | 28.014 / 4.094 | 24.321 / 4.998 | 24.939 / 4.416 |
| Goal | 12.405 / 2.339 | 11.617 / 3.446 | 12.287 / 2.829 | 11.785 / 4.112 | 12.036 / 3.138 |
| Plan | 49.852 / 23.232 | 41.182 / 32.621 | 57.264 / 25.732 | 44.820 / 33.711 | 47.336 / 29.176 |
| Home | 19.120 / 6.924 | 19.522 / 5.655 | 19.447 / 5.785 | 19.599 / 6.355 | 19.484 / 6.070 |

Home's median falls by 7.472 wall s
(27.7%) and
2.141 CPU s. Plan's falls by
6.094 wall s
(11.4%) and
7.192 CPU s. Neither retained change
trades more CPU for less wall. Away and Goal are unchanged; differences
in their figures are run variation, not a claimed improvement.

### Whole process tier

| Code | Tests | Wall s | CPU s | Result |
| --- | ---: | ---: | ---: | --- |
| Base | 51 | 498.394317 | 369.235141 | 50 passed; one unrelated assertion flaked; over CPU budget |
| Final | 51 | 505.768167 | 433.672669 | 51 passed; over CPU budget |

The CPU budget remains 360 s, the per-test CPU bound 60 s, and wall hard
limit 1080 s. Neither run starved, leaked a descendant, had incomplete
CPU records or left a test unobserved. The final tier is still red for
CPU, despite all assertions passing. The two changed status tests use
9.619240 fewer CPU s between these full runs;
all other tests together use
74.056768 more CPU s.

The largest increases are in untouched `process.rs` stories:
runner-cancellation/restarts +20.283 CPU s, the personal-plan landing
+9.608, and the CSV landing +8.989. They do not call the new helper.
This task neither changes their oracles nor raises the tier budget.
The base failure is
`a_persons_plan_change_lands_on_the_owned_forge_and_is_found_after_a_restart`,
`process.rs:704`: two printed sign-in links where one was expected.
It passed alone (1/1, 38.808 s) and in the final tier (37.623 s), on the
same machine.

Base load observed at 10:33:00Z was 10.37 / 7.94 / 8.23; the B2 start
observation was 1.88 / 6.92 / 8.65. During F1 it reached
20.18 / 11.51 / 11.35; during F3 it reached
26.20 / 20.66 / 14.80. The final tier started at 11:24:50Z with
20.04 / 22.95 / 17.03 and load was 8.78 / 8.05 / 11.58 near its end.
These are load observations, not isolated-machine comparisons. One tier
run at each end cannot establish a whole-tier speedup; the repeated
walk figures and matching act ranges support the local change.

Commands: `heavy python3 -I -B tools/tiers/run --cpu DIR temper process`
for each full tier; append `-- --test process -E
 'test(/walk::(away|goal|status)::/)'` for the four walks. The required
filter `test(/walk|fake_backends/)` selected 12 process tests, all passed.
`heavy cargo nextest run -p temper` selected 444 focused tests, all
passed. Clippy on all temper targets, old-form counts and citations also
passed. No assertions, capacities, charter waits, Undo periods or
sitting bounds are changed. The remaining waits stand in
`temper/docs/design/domain/testing.status.md`, 2.6.
