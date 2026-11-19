# Status

2026-11-16. The first milestone is built. Engine, worker, agent and client
wires version their layouts, stated and checked at channel opening;
builds speak from each wire's floor, jig's channel bridging versions
shared by engine and worker (`protocol/wires.md`; `draft/redeploys.md`, R1 and R4).
The engine restarts alone while workers keep runs: stop settles placing
nothing; workers' grace covers an upgrading start; engine grace exceeds
their stop bound. `upgrade --check` names workers that may stay from
openings kept by the engine (`protocol/wires.md`, 10; `domain/root.md`, 8.2; R5, goal 2288).
`temper-redeploy` uses the operator's kept description: stage every host,
checkpoint while the old build serves, check the new build over it,
and, before the engine goes alone, read the new build's verdict on the
workers over it too, refusing with nothing stopped a build that cannot;
then restart the engine alone if workers may stay, using its settled
stop's checkpoint as backup and rereading the workers' verdict over it
by links and small copies;
restart workers in turn when asked, otherwise all parts together,
draining each worker while the old engine serves. Each redeploy is
recorded and refereed: workers' agreed versions and returns against
their waits, no try spent or task held since except for a run's own
failure (`redeploy.md`; R6, goal 2487; C3, goal 7552; its follow-up,
goal 8397).
The engine's settled stop tells workers it restarts: those told keep
runs through the restart wait (default twenty minutes), relayed calls
and completions waiting within deadlines; the next start holds their
claims as long. Redeploy names who was told (`domain/root.md`, 8.2;
`domain/worker/README.md`, section 2; jig's `protocol/channel/restart.md`;
R8, goal 4843). Deployment drills follow landing, each an operator-started
redeploy.
A model provider unreachable for minutes holds no task: its run waits
as an infrastructure failure; a model-call failure's hold names the call
from structured fields alone (`draft/run-provider-outage.status.md`;
`domain/worker/README.md`, 4.3).
Failed workspace preparation says where and why: worker, workspace,
repository, stage, git command (check out distinct from clean), operation
and prepare times; the worker logs it
(`draft/checkout-failure-words.status.md`; `domain/worker/checkouts.md`, 1).
A real-loop story failing on an unasked kill names target, killer, time
and reason, with service notes from the run's last seconds
(`draft/real-loop-kill-cause.status.md`; `testing/real-loop.md`).
A story pairs git kills caused by its own cancel with the worker's
cancelled-attempt note and counts them as asked for. The three-worker
reference story adds s1 once the director holds ws, without assuming
which of two started workers the fleet picks first
(`draft/real-loop-cancel-kills.status.md`; `testing/real-loop.md`;
`testing/reference-case.md`).
Goal 6833's real-loop turn-landing story checks only the ctl-released
batch reaching the engine within its 2 s guide: one ctl answer to a
person's latch releases both waiting tasks together. The engine's domain
world checks two goals' changes forming one `turn` landing on one green
gate (`draft/turn-landing-placement.status.md`; `testing/real-loop.md`;
`domain/landing.md`, 16).
Remaining: landing/finding workflows, client surfaces, deployment,
releases, verification and cost. Status files below record code's gaps
and departures from design, and design's from drafts; documents keep
deferred work and open questions. Team appears at a project's second
person or first standing role. P4b, goal 713: standing-role rows and cards
with Pause/Resume in the head, Now, rules, wakes and this week's sessions;
two gallery states. Forge-topic words, kept charter words and session
lines await reads. P4c, goal 1983: areas and owners, the triager as Who
routes, a person's areas, card's "owns AREA", "Talk to it", "Give the area
back" and weekly report, "+ Hand an area to a role", and talk on a role's
page. Line sign-offs await a read of what another person can still sign
(`client/pages/team.status.md`; `draft/surfaces.status.md`, 6).
P5b, goal 2173: Tell temper at a project's Needs you foot, phone `Tell`
page and below Ask temper's composer; "About" and "Goes to" before "File
it". Words told on a standing role's page, card or talk are filed with
its held area; others with none and the page in the body, for the triager
to hand on. Terminal `/tell` sends its screen. What became of a word in
the brief awaits design
(`client/pages/needs-you.status.md`; `draft/surfaces.status.md`, 9).
Home for a person's H1, goal 4272: Needs you before Decided for you;
"findings" headline when all open cards are findings; severity in a
three-bar glyph and chip ground, counted in Needs you's head; finding
rows; person-held cards with acts and why, ordered by severity then
oldest; "One more" and its line; shared why said once in the plural;
Decided for you rows with object alone, tick, no agent's why; goal rows
with bar, number words, titles cut at 80. Walk follows landing
(`draft/home-for-a-person.status.md`; `client/pages/home.md`, 1).
Its milestone F1, goal 4585, draws the triager's groups: a question
whose line form names findings, with `Findings:` and `Group:` lines, is
a group's card on Needs you and on Home, its chip "3 bugs" on its
highest severity's ground, its phrase and its oldest age, a row a
finding, its context and its choices; the stock triager groups what it
keeps and asks one question a group; the headline and Needs you's head
count the findings the open cards concern, each once. Its walk follows
landing (`draft/home-for-a-person.status.md`; `client/pages/needs-you.md`, 1).
Its milestone F2, goal 4747, says why a group reached its person: a
kind of decision of its own, `findings`, starting work on findings,
which the stock's Agents review and Draft first tables ask unless the
person is away; the triager's question over a group names it, so its
line reads "Reached you because it starts work on findings, which asks
you unless you’re away.", said once on Home, with its Stop asking, "Let
the triager start goals for findings, then tell you." Its walk follows
landing (`draft/home-for-a-person.status.md`; `domain/forge/attention.md`, 6).
Its milestone M3, goal 5517, draws milestones on the pages: a goal that
waits says on what, "waits on S6", or "waits for its resources"; a
milestone is one row on Home with its count, state, strip of its goals and
headline, one card in Work's In flight holding its goals and their changes,
a group under the Goals filter, and a page of its own; Needs you's cards,
Decided for you's rows and Landed this week name a goal's milestone, and a
goal's page says "in Store shape". Its walk follows landing
(`draft/milestones.status.md`; `client/pages/milestone.md`).
Home and Work hold still between reads, goal 8538: redraw from whole
answers; busy needs reads keep the old headline, "N+" means a cut inbox
list, and the rail uses the page count. Parts not made keep their last
decoded values: milestone goals/headlines stay on Home and Work's In
flight, whose count stands (`client/pages/README.md`;
`client/pages/work.md`, 1, *In flight*; `client/forge/parts.md`, 5).

Landed drafts, whole story P8, goal 638: projects keep drafts landed in
their repositories and marked ready by the writer with `temper ctl draft ready`,
read at landing-branch head, with commit, title, chat and eventual goal.
Work's Drafts lists ready ones, each starting a goal at the project's
way; render check follows landing (`domain/forge/drafts.md`;
`client/pages/work.md`, 1; `draft/surfaces.status.md`, 5). People use
`role talk` with standing roles; instructions for every case become
rules by settings proposals. Roles report via Now (in Home's brief's
words), a line per session and weekly `report` note. Areas/routing:
R2, goal 684; Team talk/report: P4c, goal 1983; terminal talk: T3,
goal 1984 (`draft/standing-roles.status.md`). Being told, whole story
N3b, goal 492, is built against fakes alone: devices, jig's need arrival,
root pushes, service Web Push, `temper ctl push add`, browser Telling
you, service worker and manifest. Mail/digest are designed, unbuilt;
real push needs a named HTTPS deployment and its VAPID key
(`domain/forge/attention.status.md`, 7.4; `draft/what-reaches-you.status.md`, 9).


QA's hands are landed-qa's milestone Q2,
by goal 1000: a drive session on a fake box from a given build, taking a
person's steps and looks a line at a time; recipes and their walk; the
fake model's given scripts and strict switch; a store this build cannot
read refused by name; and a box started again with a newer build, whose
process-tier story runs after landing (`client/drive.md`;
`client/recipe.md`; `protocol/setup/box.md`, 1 and 5; `protocol/store.md`,
5; `draft/landed-qa.status.md`).

Landed QA's Q1, Q3, Q4 and Q5: `temper-qa` watches main's ref, triages
goal landings into cases in main's order, and keeps deployment builds
of landed tips until no pass can need them, in one ledger. Goal 1210's
pass takes the newest reached build and all its cases, restores each
setup at a place from copy or recipe, drives the box line by line,
records outcomes and findings, files open findings as one request and
goal, then resumes on a build holding its landing, two goals at most.
Goal 1334 adds TUI cases. Places have separate ports; `temper-qa tune`
gives their and the bounds' tuning figures. Box fake forges retain state
across restarts; a head's CI is as declared. Fake-model scripts use
conversation values, so B0/B1 twins replay as setups. Goal 1676 keeps
QA on fakes alone: each setup uses the box's fake Forgejo, connected
by ctl lines naming box values; recipe `forge` lines commit held files,
naming no credential. Goals 3014 and 5217 add deployments, ctl first:
triage writes and passes run ctl cases first, in browserless ctl/`forge`
sessions. A deployment-kind place starts the kept build's engine,
worker and agent from setup `config/` at its ports beside fake Forgejo
and model; each is saved and restored with a newer build. `deployment patch`
gives the project's landing to the recipe's result. Fake Forgejo
dispatches setup-declared, content-decided gates. The first deployment
setup lands by `turn`, `batch` two: scripts ready two goals' changes
together with a red top on the real loop; the engine's domain world
checks both landing on one green gate. Passes and the deployment-place
process story follow landing. QA gaps go to people without goals,
reopened for judgement by a closing landing. Goal 5214's ctl-only box
setup `ctl-hello` connects the fake model as the person's account and
scripts approving review verdicts, so a ctl change lands by stock rule.
Goal 5641 adds web/TUI at deployment places: non-ctl sessions sign the
browser in through the engine's fresh link and drive both clients on
its origin with the kept build's bundle. Triage and passes take web/tui
cases after ctl on deployment setups; `turn-landing-clients` sees
landing through turn's state in both. Goal 5747 unifies box and
deployment behind one drive-session/QA-tool interface, differing only
in kind values. Copies omit the deployment's pinned agent as the box's;
each box setup is weighed and stays box-only. `stock-landing` reaches
`ctl-hello`'s state at a deployment place; triage offers setups of the
kinds the places run
(`testing/qa/README.md`; `testing/qa/deployment.md`;
`testing/qa/kinds.md`; `testing/qa/setups.md`; `testing/qa/gaps.md`;
`draft/landed-qa.status.md`). Goal 8591's drive line `web requests`
lists the requests a page has sent since it last loaded, each with its
wave, from the browser kit's reports; triage may write it as a step for
a page whose draft promises a request budget, and a pass compares its
last line as a see (`client/drive.md`, 3.2; `testing/qa/triage.md`, 3;
`testing/qa/passes.md`, 4).

Merge queue M1–M3, goals 683, 2175 and 2336: `turn` landing queues
(`domain/landing.md`), a train per repository/branch, one landing each
at a time. Up to the project's `batch` waiting units stack in the
worker's merge train, a branch per member, proved once at the top,
landed by one fast-forward of the landing's own pull request. Red
multi-unit rounds narrow by prefixes: repair the blamed unit, land the
green prefix, carry higher units to the next round. `temper_probe`
lets probe gates stop at first red (`domain/landing-search.md`).
Probes by test T1, goal 4045, gives `temper_tests` to probe only cases
the top's gates report failed, landing nothing, with one run per
workflow/commit in flight
(`domain/landing-search-by-test.md`; `draft/probes-by-test.status.md`).
Engine worlds check the behaviours; the process tier keeps two stories
each on fake forge and owned Forgejo, run by choice
(`testing/process-tier.md`); staging walks follow landing
(`draft/merge-queue.status.md`).

Landing history H1, goal 2474: change/landing steps survive their rows
as events (`domain/landing-history.md`), written by the deciding or
observing step under `queue` and `turn`, one change per event. Families
are demand-loaded through the root by the forge, omitted at restart;
swept by change past `forge.history_days` or `forge.history_events`,
including idle/deleted projects; read by `temper ctl landing history`
and `change show` after the row goes. H2, goal 2649, reads events alone
for timing (`domain/landing-times.md`): a fold splits ready-to-landed/ended
spans into non-overlapping states, held first, waiting last, partial if
history starts after ready; landing rounds, probes and proof time;
paged by change with running totals in the cursor, read by
`temper ctl landing times` (`draft/landing-history.status.md`).

A held landing recovers (`draft/held-landing-recovery.status.md`): a
release takes effect once, fenced, read afresh and decided on those
reads, never on the ended delegate it retires, so a failed resolution's
release checks the head its branch now holds, or asks one fresh
resolution pinned to it (`domain/forge/change/repairs.md`, *A
release*); a change owns only the pull request it opened or whose
number it stored, never a closed one found by its branches
(`domain/forge/change/README.md`, 2); an ended change's pull request,
branch and history are frozen (`domain/forge/keeping-up.md`, 3); and a
job's attempt names why it ended (`domain/landing-history.md`, 2.2).

A landing's prepare survives a restart
(`draft/prepare-restart.status.md`): a round at preparing reads at each
step whether its prepare lives, has answered or is absent, from jig's
context and its job record, never from its own marks; the forge's start
reads and wakes each landing, deciding nothing; a lost attempt is
retried in the same child once its writer slots settle; proven absence
asks once again, fenced by jig's step, keeping the prefixes recovered;
and a release at preparing reconciles the same way
(`domain/landing-preparing.md`).

A follow across projects is the whole story's milestone F1
(`domain/forge/follows.md`): kind 12, a follow of a branch of a
repository its project adopts as context, each project's context
adoption kept beside the writer's; the forge hearing the source's
landings into a queue kept across a restart; a follow task making one
follow change per landing in order, its words made by code, parameter
18 naming what it follows; the owner's start, hold, skip and merge in
`temper ctl`, keeping the chain whole; the follow's check dispatched by
its change with the followed commit; and lag and parity in the forge's
`follows` document. A standing role may own one: its follow task is its
delegate, funded by it, its producers' questions relayed to it and its
changes' holds reaching it as escalations; it acts with the agents'
`follow` tool; a red after landing of a follow change is filed with the
area its owner owns; and Team draws a follow on its owner's row and a
role's card. The follow change's words carry no diff, and a person's
follow's holds reach the project's escalation role, each asked of the
user (`domain/forge/follows.status.md`; `draft/follows.status.md`). A
project's dependency on another is milestone F2
(`domain/forge/dependencies.md`): kind 13, naming another project of
the deployment, carried in the `follows` document beside the follows
and printed by `temper ctl follow list`, "dependency 1: fabrig depends
on vsr-rs"; nothing acts on it, and Home on All projects' What follows
what draws it beside the follows, from the first four projects'
`follows` documents (`client/pages/home.md`, 1), within a shipped
client's 26 watches (`protocol/configuration/limits.md`). A goal's reds
are temper's records, milestones B1 to B3 of `draft/branch-reds.md`
(`domain/forge/reds.md`). A goal's coordinator records and ends them with
the `red` tool, at most `forge.branch_reds` a goal. At each check's start
the worker asks the engine for them through an application's ask on
jig's channel (jig's `domain/hosts/hub.md`, 10), and writes those whose
commit the check's base contains to `temper/reds`, which fabrig's
delivery check reads, its cache keyed on the file; the tree keeps no
list of known reds. The check runs a failing test's process again, through
one-process's `--again`, and a test that passes then is a flake, refusing
nothing; at its end it writes `temper/check-results`, which the worker
reports to the engine, passed or refused. A change keeps the reds its
passed checks passed, and they end when it lands into the goal's branch;
each flake is filed or counted as a `flake` finding (`domain/findings.md`,
4). Reds end too by the tool and with their goal (`draft/branch-reds.status.md`).
A full forge outbox is a wait, not a hold: a change's effects, a
landing's writes, release cleanup and goal branches' makings that find
no room wait in one line, first come first served, their places kept on
their records across a restart, each deciding again when asked; an
agent's call is answered `Unavailable` at once, its room reserved from
its check to its entry; a waiting change's panel says "Waiting for room
in the forge outbox." on client wire 26 (`domain/forge/room.md`;
`draft/outbox-room-wait.status.md`).
Before its summary, the check writes every failure it found, with its
key and class, to `target/temper/pre-pr-failures.json` under the
checkout's root, outside the git directory, whose paths an agent's `read`
refuses, and the summary's second line names that path and counts the
failures by class, so that a key its capped summary had no room for stays
readable (`draft/pre-pr-failure-list.status.md`). A delivery of Markdown
under `docs/` alone that no test declares as its input is prose only: it
runs citations and the outward scan as a text check, which reads one list
of names with the boundary crate's Rust test and gives its verdict, and
no cargo command; and a person's or a role's report contract bound at 0
words is refused, as no answer could meet it
(`draft/small-fixes-1116.status.md`).

A landing's gate red only from known flakes runs once more
(`domain/landing-regate.md`; `draft/flake-regate.status.md`): where every
test a gate's run failed is named by an open flake finding of the gate
or of a delivery's check, its failed jobs run again once before any
receipt, kept on the landing's row across a restart, and only that
attempt's verdict makes the receipt, its green counting the findings. A
landing repair whose failed tests are known may end with verdict 1,
`flake`, on its change contract's choice (jig's
`domain/tasks/README.md`, section 3); its
change holds, and a person's release names the failed receipt as the
change's regate, so the next landing at that key dispatches the gate
afresh at the same content and its receipt replaces the failed one, with
no commit made. A red again is repaired and counted as ever.

A task's delivery stays delivered across its runs
(`domain/worker/checkouts.md`, *A task's own delivery outlives its
hold*; `draft/delivered-across-runs.status.md`): a change keeps, beside
its last checked head, the delegate task whose push set it, and that
task's next assignment names that head as its own delivery, so a
producer, a repair or a resolution that delivered, asked and parked
finishes as a change in its next run. Its new hold, starting on that
head, pushes it again, leased on what prepare fetched, and answers
delivered; a clean tree with no delivery of its own is still nothing,
and smith still refuses it.

A project's tiers after landing (`domain/after-landing.md`) are held and
released by a person, `temper ctl tier hold` and `tier release`, the hold
kept on the tier's record and said on a verifying change's line; a red
range is bisected over its changed-key landings; each verdict is posted
on the forge, `temper/verdict/<tier>` on the run's tip and a comment on
each member's pull request, `Repaired by #M` once a repair passes. A
project whose kind 4 says `wait: landed` ends each change at its
landing, so its goal and the goals waiting on it go on then, while the
tiers judge the landing from the landing log, which keeps each landing
with its members' goals and milestones until every tier has judged it,
within `forge.landings_pending`; a red there is the project's finding,
once per landing, and switching a project ends its verifying and red
changes landed and uncleared (`draft/after-landing-repairs.status.md`). A
change's stall on CI and a landing's gate run past its bound name each
pending job's state, read by skein's `JobState`, and count only the
time CI runs, by an execution clock read each poll, a wait for a runner
bounded apart (`domain/forge/change/stalls.md`, 2); a tier's run that never
ends is held by no bound (`draft/tier-controls.status.md`).

T1's repair, goal 761, reads the launchpad's brief, Where you left off
and Needs you across the person's projects, says why a walked need
reached the person and when each need asked, dates a left-off row and
draws the blank row under the header (`client/tui/pages/launchpad.md`).
Its walk follows landing, and Stop asking in the terminal remains the
web's (`draft/surfaces.status.md`, 11). T3, goal 1984, talks with a
standing role in the terminal: `/talk`, the role's head and Now, its
rule question's choices and its proposal accepted in place, leaving by
Esc; Team on PROJECT on the launchpad; and `/tell`, filing a person's
word as a `feedback` finding (`client/pages/talk.md`;
`client/tui/pages/talk.md`). Its journey and walk follow landing.

How work runs' What follows a landing and a role's budget are the whole
story's milestone P3d, by goal 1016 (`client/pages/settings.md`, 2): a box
a follow of the project's, from the forge's `follows` document, its words
made from its setting, its lag and parity and who acts' chips, then a box
of the project's path's tiers after landing; Budgets' rows for its live
standing roles under its open period; and Roles the charters its standing
roles and follows run (`domain/forge/change/branch-path.md`). Their render
check and walk follow landing (`client/pages/settings.md`, 3).

Each binary's
package, the client's included, links `temper-version` and hands its
build identity to its crate; no library links it, a row of the boundary
test's direction rule holds it, and a new commit relinks the binaries
and rebuilds no library or world below them (`release.status.md`;
`draft/version-rebuilds.status.md`).

The stock from files is the stock charters' milestone S1, by goal 1144
(`draft/stock-charters.execution.md`): the stock's charters 5, the goal's
agent, and 2, the implementer, assembled from five Markdown files in
`temper/docs/charters/stock/` by the step crate `temper-charter-set`
(`domain/agent/charter.md`, 1), a stock box's `root.journal.run_bytes`
65,536 to hold them, and the foreman retired with its whole reach, the
reference's charter 6 among it: a change's questions go `up` or nowhere,
and a stored foreman is refused naming its setting
(`domain/forge/change/questions.md`, 1; `protocol/configuration/settings.md`,
2). Its walk on a fresh box follows landing.

Overlays are the stock charters' milestone S2, by goal 1386: a charter's
overlay kept over its default, the fields it replaces and the text it
adds, laid over each start's defaults (jig's `domain/engine/overlays.md`);
`settings apply`'s overlay items, their directories of charter files and
files stacked in one call, and `settings show`'s marks and literal export
(`client/ctl/overlays.md`); the room judged on the charter an overlay
makes, its parts counted (`domain/agent/room.md`, 4); the reference's
charters 5 and 2 as overlays of its fields, its charters 7 and 8 gone
(`protocol/configuration/reference.md`), stage one's golden keeping the
whole charters on a profile of its own (`testing/checks.md`, 5); and
fabrig's set, `temper/docs/charters/fabrig/`, documentation
(`domain/agent/charter.md`, 1). The switch on dev.fabrig.ai follows
deployment (`draft/stock-charters.execution.md`).

`temper box` in a terminal runs the TUI at once and the box behind
it, by the look request `tui-box-front`: it finds, or downloads once,
`temper-box` of its own build and runs `temper-box --front` as its
child, signed in by the link the box hands it on a pipe, the welcome
following the box's start and then the browser's steps, "Signed in as
the owner" from the forges document's `linked`, the playground's header
and quiet line, a quit that asks while work runs and stops the box,
showing it stopping, the box's state and `/log`, and `/web`, which
opens temper in the browser by the box's printed link while it is
live and the web's Home once a browser signed in by it, the link in a
note to copy where no browser opens; `temper-box` alone
runs the box with no TUI (`client/tui/box.md`; `client/box-binary.md`;
`protocol/setup/box.md`, 6). Its journey waits
for `again` to start the box in the pseudo-terminal
(`client/tui/journeys.status.md`, 8.3), and its real-terminal check
follows landing (`client/README.md`, section 5). The user's requests
`tui-11` to `tui-14` of 2026-11-15, goal 3416, give a box set up before
a starting screen until the TUI has read `me`, a group starting in the
box's state ("Forgejo starting…", then the notice "Forgejo started")
from the box's `starting`, `started` and `dormant` lines, a first run's
start steps on the quiet line rather than in the welcome's body, a
cleared screen when the terminal is lent to Claude Code's sign-in, and
the TUI's two-line header after pi's, fabrig.ai's mark, the build and
the place (`client/tui/pages/README.md`, 3). The requests `tui-15` to
`tui-19` of the same day, goal 3597, add blank rows around temper's
line when the terminal is lent, draw the header and the welcome as
their mockups (a blank row on top, the mark at column 1, the link at
"Welcome."'s column, "In the browser" on one line), go back to the
launchpad once `/close` is done, and give each recording run its own
file, the last ten kept (`client/tui/journeys.md`, 3.1). The requests
`tui-20` and `tui-21` of the same day, goal 4172, take `/ask`,
`/change`, `/draft` and `/tell` alone to a chat not yet started, its
box empty with a placeholder, started by the first words sent and left
by Esc, in the place of `tui-17`'s name left in the box
(`client/tui/pages/README.md`, 2, *Not yet started*), and put the
link's state first on every screen's bottom row, at its first column
(`client/tui/pages/README.md`, *The link's state*).

The deployment's client is `temper`, at `bin/temper`, with its capture,
`skein_shell::Table` and dispatch in `temper-client-shell`. Its public rows
are `[ADDRESS]` and `help`; `ctl` is internal. It runs the TUI and ctl from
shared shells, and `--version` reads `temper-version`. Its ctl story signs
in to a `temper-engine` and runs after landing. Its closure is held by the
boundary test's row, with five shared vocabulary crates as named
exceptions (`protocol/binaries.status.md`; `client/README.status.md`, 2).
The compiled `commands` page renders the client's table first.
The upgrade commands and a start's job lines are built; the help walk
`upgrade` is not written (`protocol/documentation.status.md`). The
check reports whole rehearsal/tool time, but each job's time is
unavailable (`protocol/binaries.status.md`).

temper's command lines, `draft/cli.execution.md`'s C2, C3, C4 and C6: the box's
binary is `temper-box`, a package under `bin/` whose crate is
`temper-box-shell`, and the engine's, the worker's and the agent's,
`temper-engine`, `temper-worker` and `temper-agent`, are too, their crates
`temper-engine-shell`, `temper-worker-shell` and `temper-agent-shell`;
each holds a table on skein's, reached by `Table::reach`, with its help,
`--version`, exit 2 for refused arguments, its pre-table modes as rows,
and its reference documents generated beside `cli/README.md`. The
internal variables are renamed `TEMPER_INTERNAL_*` and the twins
dropped. The test set is `temper-test-env`'s table, its variables
`TEMPER_TEST_*`, through which temper's tests read the environment, with
its `test-env` generated beside the others. The client is `temper`, a
package under `bin/` whose crate is `temper-client-shell`, with its
table; `temper box` runs its TUI in front of a `temper-box --front`
child, which it finds beside it, in its cache or downloads once
(`client/box-binary.md`), and the box runs no TUI of its own
(`cli/README.status.md`; `draft/cli.status.md`).

Claude Code's sign-in through the terminal temper holds is being built:
the TUI in front of a box lends its terminal to `temper-box
program-login` on the welcome's "Sign in to Claude Code", and the page
follows the login to its end (`client/tui/box.status.md`,
`client/pages/welcome.status.md`; `draft/claude-sign-in.status.md`).

A deleted project's repositories are released whole: each, through the
one helper every path erasing a repository takes, goes with every
record that names it, its reads, watches and pending work retired and
its writes paged so that a restart part-way starts and resumes, while a
context another project adopts stays (`domain/forge/adoption.md`, 1;
`protocol/store.md`, 2; `draft/project-delete-cascade.status.md`).

A person opens a change from a branch they pushed with `temper ctl
change open`: the root expands the client's `OpenChange` into one change
task of their own, requested and funded by them within their role's
creating offer, and the change's first state, taking, reads the branch's
head once, keeps it and makes the change's own branch at it, then lands
it as any change, under `turn` or `queue`, leaving the pushed branch
untouched; the client's wire moved its version for it
(`domain/forge/change/opened.md`; `draft/mechanical-changes.status.md`,
M1). Its walk on dev.fabrig.ai follows landing.

A signed-in person's client asks a fresh sign-in link, `temper ctl
DIRECTORY link --out FILE`, written `0o600` and never printed, so that
a browser on the host signs in as that person once the printed links
are spent, with no restart; the link signs in the asker's party alone,
no longer than the asker, and ctl's refusals of a spent, expired or
unknown link name both ways to a fresh one (`client/ctl/README.md`, 6.1
and 7; `protocol/sign-in.md`, 2.3; jig's `protocol/client/sign-in.md`;
`draft/asked-link.status.md`).

A journey's recording holds placeholders for what varies between walks
of the same screens and isn't a secret: the TUI's build, every loopback
port and each approval code, put in by `add` after its proof and checked
by a replay of the result. The committed recordings were normalised
once, without being walked again, and the print's test holds each to its
own normalisation; a browser's print numbers skein-ui's ids afresh on
each page. Times stand, and a framed field takes a placeholder of the
original's width alone (`client/journey-placeholders.md`;
`draft/journey-placeholders.status.md`).

`again` walks a journey on one fixed start, 2026-01-06T09:00:00Z and
seed 7: the fake box's engine, worker and fake peers, the page and the
TUI take it, and each one's clock moves only to its own deadlines, so
two walks of one journey make the same recording and prints but where
real processes decide. The box's own loop, which fetches over TLS, and
its agents keep the host's clock; a recording walked by hand keeps its
times (`client/journey-clock.md`; `draft/journey-clock.status.md`).

Design documents:
- [`README.md`](README.status.md)
- [`cli/README.md`](cli/README.status.md)
- [`client/README.md`](client/README.status.md)
- [`client/box-binary.md`](client/box-binary.status.md)
- [`client/ctl/README.md`](client/ctl/README.status.md)
- [`client/ctl/areas.md`](client/ctl/areas.status.md)
- [`client/ctl/milestones.md`](client/ctl/milestones.status.md)
- [`client/ctl/results.md`](client/ctl/results.status.md)
- [`client/ctl/testing.md`](client/ctl/testing.status.md)
- [`client/ctl/tools.md`](client/ctl/tools.status.md)
- [`client/ctl/tools-crates.md`](client/ctl/tools-crates.status.md)
- [`client/ctl/workers.md`](client/ctl/workers.status.md)
- [`client/domain-crates.md`](client/domain-crates.status.md)
- [`client/drive.md`](client/drive.status.md)
- [`client/engine.md`](client/engine.status.md)
- [`client/forge/README.md`](client/forge/README.status.md)
- [`client/forge/deployment.md`](client/forge/deployment.status.md)
- [`client/forge/family.md`](client/forge/family.status.md)
- [`client/forge/pages.md`](client/forge/pages.status.md)
- [`client/forge/parts.md`](client/forge/parts.status.md)
- [`client/forge/records.md`](client/forge/records.status.md)
- [`client/forge/repositories.md`](client/forge/repositories.status.md)
- [`client/forge/served.md`](client/forge/served.status.md)
- [`client/journey-clock.md`](client/journey-clock.status.md)
- [`client/journey-placeholders.md`](client/journey-placeholders.status.md)
- [`client/line.md`](client/line.status.md)
- [`client/pages/README.md`](client/pages/README.status.md)
- [`client/pages/accounts.md`](client/pages/accounts.status.md)
- [`client/pages/ask-temper.md`](client/pages/ask-temper.status.md)
- [`client/pages/chat/README.md`](client/pages/chat/README.status.md)
- [`client/pages/chat/composer.md`](client/pages/chat/composer.status.md)
- [`client/pages/chat/conversation.md`](client/pages/chat/conversation.status.md)
- [`client/pages/chat/document.md`](client/pages/chat/document.status.md)
- [`client/pages/chat/terminal.md`](client/pages/chat/terminal.status.md)
- [`client/pages/chats.md`](client/pages/chats.status.md)
- [`client/pages/goal.md`](client/pages/goal.status.md)
- [`client/pages/home.md`](client/pages/home.status.md)
- [`client/pages/milestone.md`](client/pages/milestone.status.md)
- [`client/pages/needs-you.md`](client/pages/needs-you.status.md)
- [`client/pages/remembered.md`](client/pages/remembered.status.md)
- [`client/pages/settings.md`](client/pages/settings.status.md)
- [`client/pages/task.md`](client/pages/task.status.md)
- [`client/pages/team.md`](client/pages/team.status.md)
- [`client/pages/welcome.md`](client/pages/welcome.status.md)
- [`client/pages/workers.md`](client/pages/workers.status.md)
- [`client/pages/work.md`](client/pages/work.status.md)
- [`client/recipe.md`](client/recipe.status.md)
- [`client/testing.md`](client/testing.status.md)
- [`client/tui/README.md`](client/tui/README.status.md)
- [`client/tui/box.md`](client/tui/box.status.md)
- [`client/tui/journeys.md`](client/tui/journeys.status.md)
- [`client/tui/pages/README.md`](client/tui/pages/README.status.md)
- [`client/tui/pages/chat.md`](client/tui/pages/chat.status.md)
- [`client/tui/pages/design-session.md`](client/tui/pages/design-session.status.md)
- [`client/tui/pages/launchpad.md`](client/tui/pages/launchpad.status.md)
- [`client/tui/pages/sign-in.md`](client/tui/pages/sign-in.status.md)
- [`client/tui/theme.md`](client/tui/theme.status.md)
- [`client/view.md`](client/view.status.md)
- [`client/view-crates.md`](client/view-crates.status.md)
- [`client/web/README.md`](client/web/README.status.md)
- [`client/web/gallery.md`](client/web/gallery.status.md)
- [`client/web/journeys.md`](client/web/journeys.status.md)
- [`client/web/journey-redaction.md`](client/web/journey-redaction.status.md)
- [`client/web/look-requests.md`](client/web/look-requests.status.md)
- [`client/web/look.md`](client/web/look.status.md)
- [`client/web/public.md`](client/web/public.status.md)
- [`client/web/testing.md`](client/web/testing.status.md)
- [`domain/README.md`](domain/README.status.md)
- [`domain/after-landing.md`](domain/after-landing.status.md)
- [`domain/agent/README.md`](domain/agent/README.status.md)
- [`domain/agent/agents.md`](domain/agent/agents.status.md)
- [`domain/agent/ask-temper.md`](domain/agent/ask-temper.status.md)
- [`domain/agent/charter.md`](domain/agent/charter.status.md)
- [`domain/agent/declarations.md`](domain/agent/declarations.status.md)
- [`domain/agent/room.md`](domain/agent/room.status.md)
- [`domain/agent/standing.md`](domain/agent/standing.status.md)
- [`domain/agent/tools.md`](domain/agent/tools.status.md)
- [`domain/findings.md`](domain/findings.status.md)
- [`domain/forge/README.md`](domain/forge/README.status.md)
- [`domain/forge/dependencies.md`](domain/forge/dependencies.status.md)
- [`domain/forge/follows.md`](domain/forge/follows.status.md)
- [`domain/forge/adoption.md`](domain/forge/adoption.status.md)
- [`domain/forge/attention.md`](domain/forge/attention.status.md)
- [`domain/forge/branches.md`](domain/forge/branches.status.md)
- [`domain/forge/change/README.md`](domain/forge/change/README.status.md)
- [`domain/forge/change/branch-path.md`](domain/forge/change/branch-path.status.md)
- [`domain/forge/change/gates.md`](domain/forge/change/gates.status.md)
- [`domain/forge/change/opened.md`](domain/forge/change/opened.status.md)
- [`domain/forge/change/questions.md`](domain/forge/change/questions.status.md)
- [`domain/forge/change/repairs.md`](domain/forge/change/repairs.status.md)
- [`domain/forge/change/stalls.md`](domain/forge/change/stalls.status.md)
- [`domain/forge/drafts.md`](domain/forge/drafts.status.md)
- [`domain/forge/issues.md`](domain/forge/issues.status.md)
- [`domain/forge/keeping-up.md`](domain/forge/keeping-up.status.md)
- [`domain/forge/queues.md`](domain/forge/queues.status.md)
- [`domain/forge/reds.md`](domain/forge/reds.status.md)
- [`domain/forge/room.md`](domain/forge/room.status.md)
- [`domain/forge/walkthrough.md`](domain/forge/walkthrough.status.md)
- [`domain/landing.md`](domain/landing.status.md)
- [`domain/landing-settings.md`](domain/landing-settings.status.md)
- [`domain/landing-history.md`](domain/landing-history.status.md)
- [`domain/landing-preparing.md`](domain/landing-preparing.status.md)
- [`domain/landing-times.md`](domain/landing-times.status.md)
- [`domain/landing-search.md`](domain/landing-search.status.md)
- [`domain/landing-search-by-test.md`](domain/landing-search-by-test.status.md)
- [`domain/landing-regate.md`](domain/landing-regate.status.md)
- [`domain/milestones.md`](domain/milestones.status.md)
- [`domain/root.md`](domain/root.status.md)
- [`domain/routing.md`](domain/routing.status.md)
- [`domain/trial.md`](domain/trial.status.md)
- [`domain/worker/README.md`](domain/worker/README.status.md)
- [`domain/worker/checkouts.md`](domain/worker/checkouts.status.md)
- [`domain/worker/jobs.md`](domain/worker/jobs.status.md)
- [`playground.md`](playground.status.md)
- [`protocol/README.md`](protocol/README.status.md)
- [`protocol/floor.md`](protocol/floor.status.md) — floor 1, dev.fabrig.ai's settled store; explicit owner levels, registered layouts converted at read, fixed formats outside the registry and named refusals below the floor. The one-process floor deployment opens and serves; its floorless copy refuses. Upgrade stories use a registry change and a job, with a current settled image. No gap.
- [`protocol/binaries.md`](protocol/binaries.status.md)
- [`protocol/channel.md`](protocol/channel.status.md)
- [`protocol/checkpoints.md`](protocol/checkpoints.status.md)
- [`protocol/cluster.md`](protocol/cluster.status.md)
- [`protocol/components.md`](protocol/components.status.md)
- [`protocol/configuration/README.md`](protocol/configuration/README.status.md)
- [`protocol/configuration/derived.md`](protocol/configuration/derived.status.md)
- [`protocol/configuration/directories.md`](protocol/configuration/directories.status.md)
- [`protocol/configuration/engine.md`](protocol/configuration/engine.status.md)
- [`protocol/configuration/figures.md`](protocol/configuration/figures.status.md)
- [`protocol/configuration/limit-settings.md`](protocol/configuration/limit-settings.status.md)
- [`protocol/configuration/limits.md`](protocol/configuration/limits.status.md)
- [`protocol/configuration/moved.md`](protocol/configuration/moved.status.md)
- [`protocol/configuration/providers.md`](protocol/configuration/providers.status.md)
- [`protocol/configuration/reference.md`](protocol/configuration/reference.status.md)
- [`protocol/configuration/settings.md`](protocol/configuration/settings.status.md)
- [`protocol/configuration/stock.md`](protocol/configuration/stock.status.md)
- [`protocol/credentials.md`](protocol/credentials.status.md)
- [`protocol/documentation.md`](protocol/documentation.status.md)
- [`protocol/forge/README.md`](protocol/forge/README.status.md)
- [`protocol/forge/engine.md`](protocol/forge/engine.status.md)
- [`protocol/git.md`](protocol/git.status.md)
- [`protocol/llm.md`](protocol/llm.status.md)
- [`protocol/openid-provider.md`](protocol/openid-provider.status.md)
- [`protocol/services.md`](protocol/services.status.md)
- [`protocol/setup/README.md`](protocol/setup/README.status.md)
- [`protocol/setup/box.md`](protocol/setup/box.status.md)
- [`protocol/setup/github.md`](protocol/setup/github.status.md)
- [`protocol/setup/init.md`](protocol/setup/init.status.md)
- [`protocol/setup/owned.md`](protocol/setup/owned.status.md)
- [`protocol/setup/playground.md`](protocol/setup/playground.status.md)
- [`protocol/setup/place.md`](protocol/setup/place.status.md)
- [`protocol/setup/reconcile.md`](protocol/setup/reconcile.status.md)
- [`protocol/setup/testing.md`](protocol/setup/testing.status.md)
- [`protocol/sign-in.md`](protocol/sign-in.status.md)
- [`protocol/store.md`](protocol/store.status.md) — the current registered schema is v25, with no map patch history; `temper-secrets` is fixed at v1. Other implementation gaps are listed in its status.
- [`protocol/tooling.md`](protocol/tooling.status.md)
- [`protocol/wires.md`](protocol/wires.status.md)
- [`redeploy.md`](redeploy.status.md)
- [`release.md`](release.status.md)
- [`testing/README.md`](testing/README.status.md)
- [`testing/checks.md`](testing/checks.status.md)
- [`testing/fakes.md`](testing/fakes.status.md)
- [`testing/live-tier.md`](testing/live-tier.status.md)
- [`testing/process-tier.md`](testing/process-tier.status.md)
- [`testing/qa/README.md`](testing/qa/README.status.md)
- [`testing/qa/builds.md`](testing/qa/builds.status.md)
- [`testing/qa/deployment.md`](testing/qa/deployment.status.md)
- [`testing/qa/findings.md`](testing/qa/findings.status.md)
- [`testing/qa/gaps.md`](testing/qa/gaps.status.md)
- [`testing/qa/kinds.md`](testing/qa/kinds.status.md)
- [`testing/qa/landings.md`](testing/qa/landings.status.md)
- [`testing/qa/ledger.md`](testing/qa/ledger.status.md)
- [`testing/qa/passes.md`](testing/qa/passes.status.md)
- [`testing/qa/places.md`](testing/qa/places.status.md)
- [`testing/qa/setups.md`](testing/qa/setups.status.md)
- [`testing/qa/triage.md`](testing/qa/triage.status.md)
- [`testing/real-loop.md`](testing/real-loop.status.md)
- [`testing/reference-case.md`](testing/reference-case.status.md)

Drafts:
- [`draft/after-landing-repairs.md`](draft/after-landing-repairs.status.md)
- [`draft/after-landing-reds-1114.md`](draft/after-landing-reds-1114.status.md)
- [`draft/agent-docs.md`](draft/agent-docs.status.md)
- [`draft/ask-temper.md`](draft/ask-temper.status.md)
- [`draft/asked-link.md`](draft/asked-link.status.md)
- [`draft/bench-without-sqlite.md`](draft/bench-without-sqlite.status.md)
- [`draft/box-link-first.md`](draft/box-link-first.status.md)
- [`draft/branch-reds.md`](draft/branch-reds.status.md)
- [`draft/brief-per-person.md`](draft/brief-per-person.status.md) — P1, goal 8126: `PersonBrief` per party/project, run at their look with them payer through their project pool; their Home's words span their last brief. No account/room: "No brief: no account pays for your work here." Old `ProjectFacts` brief/run dropped on read; client wire 36→37 (38 with landing wait), store 25 unchanged. Walk pending.
- [`draft/brief-pull-fits-deadline.md`](draft/brief-pull-fits-deadline.status.md)
- [`draft/bundle-loading.md`](draft/bundle-loading.status.md)
- [`draft/change-landing-fixes.md`](draft/change-landing-fixes.status.md)
- [`draft/change-stall-after.md`](draft/change-stall-after.status.md)
- [`draft/chat-reading.md`](draft/chat-reading.status.md)
- [`draft/checkpoints.md`](draft/checkpoints.status.md) — C1, goal 5099: barrier, store/transcript holds, immutable links/appended-prefix copies, record/manifest, atomic rename/open sweep, newest `store.checkpoints.keep`, held-lock skip; stopped `temper-engine checkpoint`, `--hold NAME -- COMMAND`, `--remove NAME`, owner's `temper ctl DIR deployment checkpoint`. C2, goal 6983: stopped restore, safety checkpoint, staged generation/transcripts with no previous generation, durable intent rolled forward at restore/open (`protocol/checkpoints.md`, 2.5; jig's `protocol/store/restoring.md`). C3, goal 7552: live checkpoint/full check; negative verdict drains before old engine stops; settled-stop backup; `upgrade --workers` over maps alone, no record walks/jobs; rollback with new restore then old build (`redeploy.md`, 4; `protocol/binaries.md`, section 2; jig's `protocol/store/opening.md`, 5). Goal 8397: pre-stop verdict refuses unsupported builds before stopping; rehearsal links snapshot/sealed segments, copies newest/secrets; first-redeploy prerequisites (`redeploy.md`, 2.3 and 4.1); restore pins previous levels (jig's `protocol/store/restoring.md`, 4.3). Walks after landing.
- [`draft/ci-on-forgejo-17.md`](draft/ci-on-forgejo-17.status.md)
- [`draft/ci-runner-wait.md`](draft/ci-runner-wait.status.md)
- [`draft/claude-api.md`](draft/claude-api.status.md)
- [`draft/claude-provider.md`](draft/claude-provider.status.md)
- [`draft/claude-sign-in.md`](draft/claude-sign-in.status.md)
- [`draft/cli.md`](draft/cli.status.md)
- [`draft/codec-shape.md`](draft/codec-shape.status.md) — K2, goal 7547: `src/generated/<stem>.rs` per family, version constant inside; no item-count arrays, `LIMIT_NAMES` a slice, named booleans, functions split by declaration/field; fingerprints from schema texts. Removed jig's store and smith's local-store `generated/layouts.rs`, jig's layouts row (key `00 03` reserved), skein-oauth's `FINGERPRINT`. Merge job regenerates clean schemas with `skein-codegen` (`protocol/tooling.md`, 4.1); forges/searches mark generated files and `AGENTS.md` forbids hand merges. K3: lean tasks records, `schema/lean`, `every-v3` both ways, completeness, `missing`/`roots`/`largest`, pinned bounds, largest-value memory tests and line/compile/decode measurements against draft 3.3.
- [`draft/commit-failed-why.md`](draft/commit-failed-why.status.md)
- [`draft/config.md`](draft/config.status.md)
- [`draft/configuring.md`](draft/configuring.status.md)
- [`draft/confinement.md`](draft/confinement.status.md)
- [`draft/crate-graph.md`](draft/crate-graph.status.md)
- [`draft/dashboards.md`](draft/dashboards.status.md)
- [`draft/deleted-repository-rows.md`](draft/deleted-repository-rows.status.md) — goal 3678: temper's job at open `deleted-repository-forge-rows` drops the forge rows a deleted project left naming a repository with no `Repository` row, and refuses, naming it, on any row it must keep; jobs at open count what they drop, printed on the `ran` line (`protocol/store.md`, 4; `protocol/binaries.md`, 2).
- [`draft/delivered-across-runs.md`](draft/delivered-across-runs.status.md)
- [`draft/delivery-checks.md`](draft/delivery-checks.status.md)
- [`draft/deployment-binaries.md`](draft/deployment-binaries.status.md)
- [`draft/design-economy.md`](draft/design-economy.status.md)
- [`draft/dogfood-hacks.md`](draft/dogfood-hacks.status.md)
- [`draft/early-start-checked.md`](draft/early-start-checked.status.md)
- [`draft/early-start-squash.md`](draft/early-start-squash.status.md)
- [`draft/fetch-progress.md`](draft/fetch-progress.status.md)
- [`draft/first-release.md`](draft/first-release.status.md)
- [`draft/first-run-projects.md`](draft/first-run-projects.status.md)
- [`draft/flake-regate.md`](draft/flake-regate.status.md)
- [`draft/focused-pyramid.md`](draft/focused-pyramid.status.md)
- [`draft/follows.md`](draft/follows.status.md)
- [`draft/forge-identity.md`](draft/forge-identity.status.md)
- [`draft/forge-polls-quiet.md`](draft/forge-polls-quiet.status.md)
- [`draft/forge-reads-targeted.md`](draft/forge-reads-targeted.status.md)
- [`draft/fuzzy-context.md`](draft/fuzzy-context.status.md)
- [`draft/gate-compile.md`](draft/gate-compile.status.md)
- [`draft/gate-units.md`](draft/gate-units.status.md) — A1, goal 7991: base checks build only their flagged targets (`tools/tiers/references.md`, 5); the agent and landing fixtures and the process guard are kits of their own (`testing/README.md`, 5).
- [`draft/github.md`](draft/github.status.md)
- [`draft/goal-as-designer.md`](draft/goal-as-designer.status.md)
- [`draft/goal-runs-itself.md`](draft/goal-runs-itself.status.md)
- [`draft/goal-start-flags.md`](draft/goal-start-flags.status.md)
- [`draft/held-landing-recovery.md`](draft/held-landing-recovery.status.md)
- [`draft/home-for-a-person.md`](draft/home-for-a-person.status.md) — A1, goal 4273: the stock charters' words for a goal's or a change's title line, `decided`'s question and options, and a goal's report headline, written for a person, and `help goal start`'s for a goal's title.
- [`draft/home-for-a-person.md`](draft/home-for-a-person.status.md) — B1: the status agent's words carry the project's open findings, counted by kind and severity with the high ones' titles, after the goals and before the roles' Nows; a sitting starts a run when they changed since the last brief, kept as the brief's mark, a project that has landed nothing included; its instructions name findings by what they are about (`domain/forge/attention.md`, 2 and 3).
- [`draft/journey-clock.md`](draft/journey-clock.status.md)
- [`draft/journey-placeholders.md`](draft/journey-placeholders.status.md)
- [`draft/journey-walks.md`](draft/journey-walks.status.md)
- [`draft/kept-change-answers.md`](draft/kept-change-answers.status.md)
- [`draft/landed-qa.md`](draft/landed-qa.status.md)
- [`draft/landing-history.md`](draft/landing-history.status.md)
- [`draft/landing-update-loop.md`](draft/landing-update-loop.status.md) — a stale update replans instead of spinning: the client names a moved base apart from a moved head, a refusal before any send replans against the base the forge now has or backs off within `stall`, spending no update, and the history's update lines carry the forge's base beside the pin (`domain/forge/README.md`, 6; `domain/landing-history.md`, 2.2; skein's `forge/client.md`, 7.4).
- [`draft/long-text-pages.md`](draft/long-text-pages.status.md) — landed whole into main in pulls #157 and #181; plan complete, vision folded into design, read-only text retained pending proposed removal.
- [`draft/look-requests.md`](draft/look-requests.status.md)
- [`draft/macos.md`](draft/macos.status.md)
- [`draft/mechanical-changes.md`](draft/mechanical-changes.status.md)
- [`draft/merge-deletions.md`](draft/merge-deletions.status.md)
- [`draft/merge-queue.md`](draft/merge-queue.status.md)
- [`draft/milestones.md`](draft/milestones.status.md) — M1, goal 4276: jig's gathering, `Conclude` and a party's `Delegate`. M2, goal 4793: temper's milestone, the stock's template 8, label 4, on jig's gathering; its goals asking its maker; `temper ctl milestone new`, `whole`, `list` and `show`, and `goal start`'s `--milestone`, `--after` and `--no-deadline`; the forge family's read `Milestone` and the forge's row of each milestone, its last landing and its end; the status agent's milestones (`domain/milestones.md`; `client/ctl/milestones.md`). M3, goal 5517: the pages' milestones, a goal's waits, Home's grouped Goals, Work's milestone card, a milestone's page and its goals named by it on Needs you, Decided for you and a goal's page (`client/pages/milestone.md`).
- [`draft/model-catalogue.md`](draft/model-catalogue.status.md)
- [`draft/model-check.md`](draft/model-check.status.md)
- [`draft/one-owner.md`](draft/one-owner.status.md)
- [`draft/outbox-room-wait.md`](draft/outbox-room-wait.status.md)
- [`draft/own-way.md`](draft/own-way.status.md)
- [`draft/page-round-trips.md`](draft/page-round-trips.status.md) — R1, goal 6234: jig's batch, many reads and watches in one exchange, its replies streamed as each is released; the client root settling before its link sends; a stream naming the one it replaces; jig's client wire 34 (jig's `protocol/client/wire.md`, 2; `protocol/client/default-http.md`, 6).
- [`draft/page-round-trips.md`](draft/page-round-trips.status.md) — R3, goal 6236: `me` with the stream, its hello holding the served ranges and then `me`, every stream watching it (jig's `protocol/client/wire.md`, 2); what names no party asked with the hello (`client/README.md`, 3.4); no narrower twin of a list's watch; the needs' selection read again only as `me`'s `needs_total` or the inbox moves (`client/pages/work.md`, 1); a setting's read answering with its revision (`protocol/configuration/settings.md`, 2).
- [`draft/page-round-trips.md`](draft/page-round-trips.status.md) — R4, goal 7902: Work, Goals and Changes on their budget, a cold open drawn whole in two waves of two requests, the stream and then one batch, its constant beside the page and its focused test on a small project and one at the client's bounds; nothing of the frame's or a page's before the hello, no chips or `deployment` watch on Work; Work's sections drawn top down as their lists come, each row whole when first drawn (`client/pages/work.md`, 1; `client/README.md`, 3.4; `client/forge/family.md`, 7).
- [`draft/panics.md`](draft/panics.status.md)
- [`draft/per-wake.md`](draft/per-wake.status.md)
- [`draft/pre-pr-failure-list.md`](draft/pre-pr-failure-list.status.md)
- [`draft/prepare-restart.md`](draft/prepare-restart.status.md)
- [`draft/probes-by-test.md`](draft/probes-by-test.status.md)
- [`draft/producer-reach.md`](draft/producer-reach.status.md)
- [`draft/project-delete-cascade.md`](draft/project-delete-cascade.status.md)
- [`draft/project-open-superseded.md`](draft/project-open-superseded.status.md) — a made project's `Home` opens only while the person stays where they asked it: a later navigation or project choice supersedes the opening, the creation still finishing; the native client shows the address it last applied, and the process tier's person goes on once the creation's own `Home` is applied (`client/view.md`, 3; `testing/process-tier.md`).
- [`draft/proposal-reach.md`](draft/proposal-reach.status.md)
- [`draft/queue-smooth.md`](draft/queue-smooth.status.md)
- [`draft/real-journeys.md`](draft/real-journeys.status.md)
- [`draft/real-loop-cancel-kills.md`](draft/real-loop-cancel-kills.status.md)
- [`draft/real-loop-kill-cause.md`](draft/real-loop-kill-cause.status.md)
- [`draft/record-anywhere.md`](draft/record-anywhere.status.md)
- [`draft/reference-case.md`](draft/reference-case.status.md)
- [`draft/reference-code-production.md`](draft/reference-code-production.status.md)
- [`draft/redeploys.md`](draft/redeploys.status.md)
- [`draft/repository-copies.md`](draft/repository-copies.status.md)
- [`draft/run-provider-outage.md`](draft/run-provider-outage.status.md)
- [`draft/serve-without-box.md`](draft/serve-without-box.status.md)
- [`draft/settings-numbers.md`](draft/settings-numbers.status.md)
- [`draft/sign-in-new-tab.md`](draft/sign-in-new-tab.status.md)
- [`draft/small-fixes-1113.md`](draft/small-fixes-1113.status.md)
- [`draft/small-fixes-1116.md`](draft/small-fixes-1116.status.md)
- [`draft/squash-in.md`](draft/squash-in.status.md) — S1, goal 2176: a change's commit into its goal's branch, its title, description and trailers; S2, goal 2231: the stock charters' words for the coding agent, the designer and the coordinator, a change's commits its maker's and main taken only for a change that needs it.
- [`draft/standing-roles.md`](draft/standing-roles.status.md)
- [`draft/stock-ways.md`](draft/stock-ways.status.md)
- [`draft/store-floor.md`](draft/store-floor.status.md) — F0, goal 4032: dev.fabrig.ai's store settled once at open: the tasks' milestones, the whole live tasks, the projections' whole forms and temper's whole change records dropped from the schema, value types 10 and 12 retired with their ranges, each set's settling job putting every value at the binary's layout and retiring the other layouts by proof, before `task-logs`; skein's plans with their drops refused; the turns' versions, ended tasks' variants and empty round branches counted; the levels held explicitly by each set; dev.fabrig.ai redeployed with it onto `3d1c717d37`. F2, goal 4623: the map patches' history and the walk gone: jig's and temper's patches, old registered texts and temper's old goldens, the eager walk with S2's rewrite, the fresh mark, the effects-in-flight check, seeding from code, the baseline recognizer and the rehearsal's decoding at patch levels; the owner sets (`OwnerSet`) keep each current family, value type, job, level and the frames' texts and patches; a store behind the levels or holding values with no registry refused, naming the bridge build; the upgrade fixture a registry change and a job. F1, goal 4999: the open refuses a store with no floor row or one below `FLOOR`, after its levels and before any plan, job, rehearsal decoding or registry read, naming the bridge build `3d1c717d379e`, and a non-empty store with no registry instead of seeding it; the floor row written by a store's first commit alone; every done job, the settling mechanism, F0's counts and the retired declarations gone, history routed to the task logs in every commit; the one-process golden store made at the floor. F3, goal 5600: the words rewritten to the floor: temper's baseline document rewritten as its floor's, jig's store documents, skein's codec documents and the drafts that promised patches for stores behind the build; the redeploy's backup taken before a check that ran a job. F1's and F2's walks not yet run.
- [`draft/store-starts.md`](draft/store-starts.status.md) — B1, goal 7562: a live zero-budget task outlives its closed ledger across a restart, its end posting to its project's current period; an escalation's decider stored and linked as a task or a person, older stores typed once by jig's `escalation-deciders` job, and shown with its kind in the task's document, the client wire still naming its number alone (open); `upgrade --check` pinned to refuse, naming its record, a store a start refuses.
- [`draft/store-shape.md`](draft/store-shape.status.md) — S1, goal 2651: the census, counted by skein-kv per keyspace, named and reported by jig's component, shown in the deployment's document, temper's Limits page and `temper ctl DIR deployment census`; S2, goal 2652: landed, with no head-only walk remaining; older stores refused by name, naming bridge build `3d1c717d379e`; S3, goal 2653: generations by hard links: skein-io's `HardLink` on every backend and in the file layer, and an upgrade's new generation linking the previous one's snapshot and sealed segments, copying its newest segment and secrets. S4, goal 2826: layout ids: defaults and former names in skein's schemas, a value type's layout fingerprint, derived plans over a value's bytes, the command passing over registered families; jig's store registry of layouts, its heads mapped at the store's edges and every value converted as read, the open's check, and jobs at open routed to their owners, written changes among them. S5, goal 3361: milestones as attempts: the tasks child keeps a row per attempt and per held phase, no row per phase change; escalations, history pages, goals' projections and temper's goal issues read them; client wire 23 carries them. S6, goal 3362: retention: an ended task's summary written when it ends, and its turns, full record and summary each erased past a horizon of the deployment's limits (14, 30 and 365 days) by the core's hourly pass, deferred while live work names it or its requester has not read it; forgotten tasks' remains and decided person proposals collected; a shrunk task listed from its summary; a change record dropped with its task's record. S8, goal 3363: hot records apart from cold: live tasks, goals' projections and temper's change records each kept as a part an act changes and a part that changes on most saves, a save putting only the part that changed. S7, goal 3523: history in task logs: skein-kv's payload logs, compressed by skein's DEFLATE; a task's history and its turns in two logs in the store's generation, read through a view of the map and the logs, linked into a new generation and dropped whole by retention. S1 to S8 are landed; no done upgrade job remains.
- [`draft/surfaces.md`](draft/surfaces.status.md)
- [`draft/template-places.md`](draft/template-places.status.md)
- [`draft/test-budgets.md`](draft/test-budgets.status.md) — B1, goal 5270: every binary in a pyramid layer (`tools/tiers/layers.toml`); library/binary/generated-codec tests are step tests, repository checks separate; boundary validates coverage and names (`tools/tiers/layers.md`). B2, goal 5567: focused/fuzzy CPU budgets by tree/layer from 11-16 gate runs plus 10%, references and ideals; one-process totals red under `--gate`, warn otherwise (`tools/tiers/budgets.md`). B3, goal 5618: selection judged whole using main's folded ledger; held budget only falls, min(user ceiling, lowest main +10%); explicit downward transfers; red belongs to change if main fits or growth exceeds noise, naming growing tests/binaries (same, 7 to 10). B4, goal 5722: new or doubled-wall tests run alone, flagged tests at base too; CPU minus empty binary, reasons past reference, CPU-alone ledger (`tools/tiers/references.md`). B6, goal 6324: delivery uses gate rules with `TIERS_LEDGER`, ledger-keyed cache; missing ledgers warn; no builder keeps them (`tools/pre-pr/design.md`, 4 and 7). B7, goal 6661: over-budget main uses binaries' last-three CPU medians in ledger v3, including unrun head binaries and growth; folded lowest; noise max(1 s, main/20) (`tools/tiers/budgets.md`, 7 to 9). B8, goal 8104: all figures user's; `tools/tiers/figures.py` names/refuses raised figures or added reasons at gate/delivery, except figures-only landings. Every budget/reference red/refusal points to full testing strategy and 8.1's three questions; gate runner pending (same, 10.1; `tools/pre-pr/design.md`, 4). B9, goal 8135: main-ledger tests whose last three alone figures differ by over half again are named with figures; warning only (`tools/tiers/references.md`, 6). B10, goal 8139: 25 non-jig tests furthest past reference on 11-18 (temper simulated/real-loop/end-to-end/system/domain and repository checks): correct tier, cheaper cause, or measurements to user (`skein/docs/foundation/testing-strategy.md`, 8.1).
- [`draft/tier-controls.md`](draft/tier-controls.status.md)
- [`draft/to-begin.md`](draft/to-begin.status.md)
- [`draft/tools-by-role.md`](draft/tools-by-role.status.md) — T1, goal 5198: unencodable assignments name fields/figures in engine log and held reason; generated paths name themselves; charter fixed costs checked by family at apply and against defaults at start, including local agents. T2, goal 5617: authority-filtered tools by charter family, `read_docs` except asking runs; ≤1,024-byte short declarations, whole on `host-tools`; 18 forge reads via `read_forge`/`read_issues`/`read_ci` with `what`; contract 4, floor 1, adapters 1–3; stock goal/coder offers ≤14,336/4,096 bytes; brief floor checked at apply/start. Goal 8709: short forms name each `what`'s inputs; omitted branch defaults, `file` at branch head; sentence refusals; contract 10, three-read adapters 6–9.
- [`draft/tui-design-session.md`](draft/tui-design-session.status.md)
- [`draft/tui-remote.md`](draft/tui-remote.status.md)
- [`draft/tui.md`](draft/tui.status.md)
- [`draft/turn-landing-placement.md`](draft/turn-landing-placement.status.md)
- [`draft/upgrades.md`](draft/upgrades.status.md)
- [`draft/v0.1.0.md`](draft/v0.1.0.status.md)
- [`draft/version-rebuilds.md`](draft/version-rebuilds.status.md)
- [`draft/what-reaches-you.md`](draft/what-reaches-you.status.md)
- [`draft/whole-story.md`](draft/whole-story.status.md)
- [`draft/widening-funded.md`](draft/widening-funded.status.md)

Archived drafts:
- [`draft/archive/reference-landing.md`](draft/archive/reference-landing.status.md)
- [`draft/archive/worker-reply-bound.md`](draft/archive/worker-reply-bound.status.md)

Goal 684 carries R2: areas in words and their owners, the project triager
(kinds 14 and 15), findings held by those owners with news or Needs you,
ctl filing, handing and triage, outside service parties, and stock
triager, prover and porter charters (`domain/areas.status.md`,
`domain/findings.status.md`, `draft/standing-roles.status.md`).
