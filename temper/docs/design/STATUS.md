# Status

2026-11-13. The first milestone stands in the code. Remaining work spans
landing and finding workflows, client surfaces, deployment and releases,
and verification and cost. The status files below record where temper's
code lags or departs from its design, and its design from its drafts;
each document keeps its deferred work and open questions. Team shows from a
project's second person or its first standing role. P4b, goal 713, draws
the project's standing roles, a row each, and a role's card: its head
with Pause and Resume, its Now, its rules, what wakes it and its
sessions this week, with two gallery states; a forge topic's words, a
charter's kept words and a session's line wait for reads. P4c, goal 1983,
draws the project's areas and their owners, the triager as Who routes, a
person's areas, a card's "owns AREA", "Talk to it", "Give the area back"
and weekly report, and "+ Hand an area to a role"; a role's page talks
with it. A line's sign-offs wait on a read of what another person can
still sign (`client/pages/team.status.md`; `draft/surfaces.status.md`, 6).
P5b, goal 2173, draws Tell temper at a project's Needs you's foot, on the
phone's `Tell` page and under Ask temper's composer, saying "About" and
"Goes to" before "File it": a word told on a standing role's page, card or
talk is filed with the area the role holds, any other with none and the
page in its body, for the triager to hand on; the terminal's `/tell` sends
its screen. What became of a word in the brief waits for a design
(`client/pages/needs-you.status.md`; `draft/surfaces.status.md`, 9).

Landed
drafts are the whole story's milestone P8, by goal 638: a project keeps
each draft landed in its repositories that its writer says is ready, by
`temper ctl draft ready`, read at the landing branch's head and kept with
its commit, title, chat and, once started, its goal; Work lists the ready
ones under Drafts, each starting a goal at the project's way; its render
check follows landing (`domain/forge/drafts.md`; `client/pages/work.md`, 1;
`draft/surfaces.status.md`, 5). A person talks with a standing role through
`role talk`, with instructions for every case becoming rules by settings
proposals. A role reports through its Now, a line per session and a weekly
`report` note, with its Now in Home's brief's words; areas and routing are R2, goal 684; Team's talk and report are P4c's, goal 1983, and the
terminal's talk is T3's, goal 1984 (`draft/standing-roles.status.md`).
Being told is the whole story's milestone N3b, goal 492, built
against fakes alone: a person's devices, jig's need's arrival, the
root's pushes and the service's Web Push, `temper ctl push add`, and
Telling you in the browser, its service worker and manifest; mail and
the digest are designed and not built, and a real push needs a deployment
at a name over HTTPS and its VAPID key (`domain/forge/attention.status.md`,
7.4; `draft/what-reaches-you.status.md`, 9).


QA's hands are landed-qa's milestone Q2,
by goal 1000: a drive session on a fake box from a given build, taking a
person's steps and looks a line at a time; recipes and their walk; the
fake model's given scripts and strict switch; a store this build cannot
read refused by name; and a box started again with a newer build, whose
process-tier story runs after landing (`client/drive.md`;
`client/recipe.md`; `protocol/setup/box.md`, 1 and 5; `protocol/store.md`,
5; `draft/landed-qa.status.md`).

Landed QA's milestones Q1, Q3, Q4 and Q5: `temper-qa`
watches main's ref, triages each goal's landing into cases in main's
order, and keeps the deployment's builds of landed tips until no pass
can need them, all in one ledger; a pass, by goal 1210, takes the newest
reached build with every case it holds, restores each case's setup
at a place from its copy or its recipe, drives the box a line at a time,
records outcomes and findings as it goes, files its open findings as one
request and one goal, and resumes on a build holding that goal's
landing, two goals at most. By goal 1334 it takes the TUI's cases too.
Its places have their own ports, and `temper-qa tune` gives the figures
bounds and places are tuned from. The box's fake forges keep their state
across a restart, and a head's CI there is what it declares.
Fake-model scripts take values from the conversation, so B0's and B1's
twins replay as setups. By goal 1676 QA runs on the fakes alone: every
setup's forge is the box's fake Forgejo, connected through ctl lines that
name the box's values, and a recipe's `forge` lines commit files it
holds, with no credential named. Its passes follow landing
(`testing/qa/README.md`; `testing/qa/setups.md`;
`draft/landed-qa.status.md`).

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
client's 26 watches (`protocol/configuration/limits.md`).

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
makes, its parts counted (`domain/agent/tools.md`); the reference's
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
showing it stopping, and the box's state and `/log`; `temper-box` alone
runs the box with no TUI (`client/tui/box.md`; `client/box-binary.md`;
`protocol/setup/box.md`, 6). Its journey waits
for `again` to start the box in the pseudo-terminal
(`client/tui/journeys.status.md`, 8.3), and its real-terminal check
follows landing (`client/README.md`, section 5).

The deployment's client is `temper`, at `bin/temper`, with its capture,
`skein_shell::Table` and dispatch in `temper-client-shell`. Its public rows
are `[ADDRESS]` and `help`; `ctl` is internal. It runs the TUI and ctl from
shared shells, and `--version` reads `temper-version`. Its ctl story signs
in to a `temper-engine` and runs after landing. Its closure is held by the
boundary test's row, with five shared vocabulary crates as named
exceptions (`protocol/binaries.status.md`; `client/README.status.md`, 2).
The compiled `commands` page renders the client's table first
(`protocol/documentation.status.md`).

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

Design documents:
- [`README.md`](README.status.md)
- [`cli/README.md`](cli/README.status.md)
- [`client/README.md`](client/README.status.md)
- [`client/box-binary.md`](client/box-binary.status.md)
- [`client/ctl/README.md`](client/ctl/README.status.md)
- [`client/ctl/areas.md`](client/ctl/areas.status.md)
- [`client/ctl/results.md`](client/ctl/results.status.md)
- [`client/ctl/testing.md`](client/ctl/testing.status.md)
- [`client/ctl/tools.md`](client/ctl/tools.status.md)
- [`client/drive.md`](client/drive.status.md)
- [`client/engine.md`](client/engine.status.md)
- [`client/forge/README.md`](client/forge/README.status.md)
- [`client/forge/deployment.md`](client/forge/deployment.status.md)
- [`client/forge/family.md`](client/forge/family.status.md)
- [`client/forge/pages.md`](client/forge/pages.status.md)
- [`client/forge/records.md`](client/forge/records.status.md)
- [`client/forge/repositories.md`](client/forge/repositories.status.md)
- [`client/forge/served.md`](client/forge/served.status.md)
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
- [`client/pages/needs-you.md`](client/pages/needs-you.status.md)
- [`client/pages/remembered.md`](client/pages/remembered.status.md)
- [`client/pages/settings.md`](client/pages/settings.status.md)
- [`client/pages/task.md`](client/pages/task.status.md)
- [`client/pages/team.md`](client/pages/team.status.md)
- [`client/pages/welcome.md`](client/pages/welcome.status.md)
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
- [`client/web/README.md`](client/web/README.status.md)
- [`client/web/gallery.md`](client/web/gallery.status.md)
- [`client/web/journeys.md`](client/web/journeys.status.md)
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
- [`domain/forge/change/questions.md`](domain/forge/change/questions.status.md)
- [`domain/forge/change/repairs.md`](domain/forge/change/repairs.status.md)
- [`domain/forge/drafts.md`](domain/forge/drafts.status.md)
- [`domain/forge/issues.md`](domain/forge/issues.status.md)
- [`domain/forge/keeping-up.md`](domain/forge/keeping-up.status.md)
- [`domain/forge/queues.md`](domain/forge/queues.status.md)
- [`domain/forge/walkthrough.md`](domain/forge/walkthrough.status.md)
- [`domain/landing.md`](domain/landing.status.md)
- [`domain/root.md`](domain/root.status.md)
- [`domain/routing.md`](domain/routing.status.md)
- [`domain/trial.md`](domain/trial.status.md)
- [`domain/worker/README.md`](domain/worker/README.status.md)
- [`domain/worker/checkouts.md`](domain/worker/checkouts.status.md)
- [`domain/worker/jobs.md`](domain/worker/jobs.status.md)
- [`playground.md`](playground.status.md)
- [`protocol/README.md`](protocol/README.status.md)
- [`protocol/baseline.md`](protocol/baseline.status.md)
- [`protocol/binaries.md`](protocol/binaries.status.md)
- [`protocol/channel.md`](protocol/channel.status.md)
- [`protocol/cluster.md`](protocol/cluster.status.md)
- [`protocol/components.md`](protocol/components.status.md)
- [`protocol/configuration/README.md`](protocol/configuration/README.status.md)
- [`protocol/configuration/directories.md`](protocol/configuration/directories.status.md)
- [`protocol/configuration/engine.md`](protocol/configuration/engine.status.md)
- [`protocol/configuration/limits.md`](protocol/configuration/limits.status.md)
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
- [`protocol/setup/reconcile.md`](protocol/setup/reconcile.status.md)
- [`protocol/setup/testing.md`](protocol/setup/testing.status.md)
- [`protocol/sign-in.md`](protocol/sign-in.status.md)
- [`protocol/store.md`](protocol/store.status.md)
- [`protocol/tooling.md`](protocol/tooling.status.md)
- [`release.md`](release.status.md)
- [`testing/README.md`](testing/README.status.md)
- [`testing/checks.md`](testing/checks.status.md)
- [`testing/fakes.md`](testing/fakes.status.md)
- [`testing/live-tier.md`](testing/live-tier.status.md)
- [`testing/process-tier.md`](testing/process-tier.status.md)
- [`testing/qa/README.md`](testing/qa/README.status.md)
- [`testing/qa/builds.md`](testing/qa/builds.status.md)
- [`testing/qa/findings.md`](testing/qa/findings.status.md)
- [`testing/qa/landings.md`](testing/qa/landings.status.md)
- [`testing/qa/ledger.md`](testing/qa/ledger.status.md)
- [`testing/qa/passes.md`](testing/qa/passes.status.md)
- [`testing/qa/places.md`](testing/qa/places.status.md)
- [`testing/qa/setups.md`](testing/qa/setups.status.md)
- [`testing/qa/triage.md`](testing/qa/triage.status.md)
- [`testing/real-loop.md`](testing/real-loop.status.md)
- [`testing/reference-case.md`](testing/reference-case.status.md)

Drafts:
- [`draft/agent-docs.md`](draft/agent-docs.status.md)
- [`draft/ask-temper.md`](draft/ask-temper.status.md)
- [`draft/bench-without-sqlite.md`](draft/bench-without-sqlite.status.md)
- [`draft/box-link-first.md`](draft/box-link-first.status.md)
- [`draft/bundle-loading.md`](draft/bundle-loading.status.md)
- [`draft/change-stall-after.md`](draft/change-stall-after.status.md)
- [`draft/chat-reading.md`](draft/chat-reading.status.md)
- [`draft/ci-on-forgejo-17.md`](draft/ci-on-forgejo-17.status.md)
- [`draft/claude-api.md`](draft/claude-api.status.md)
- [`draft/claude-provider.md`](draft/claude-provider.status.md)
- [`draft/claude-sign-in.md`](draft/claude-sign-in.status.md)
- [`draft/cli.md`](draft/cli.status.md)
- [`draft/configuring.md`](draft/configuring.status.md)
- [`draft/crate-graph.md`](draft/crate-graph.status.md)
- [`draft/dashboards.md`](draft/dashboards.status.md)
- [`draft/delivery-checks.md`](draft/delivery-checks.status.md)
- [`draft/deployment-binaries.md`](draft/deployment-binaries.status.md)
- [`draft/dogfood-hacks.md`](draft/dogfood-hacks.status.md)
- [`draft/fetch-progress.md`](draft/fetch-progress.status.md)
- [`draft/first-release.md`](draft/first-release.status.md)
- [`draft/first-run-projects.md`](draft/first-run-projects.status.md)
- [`draft/follows.md`](draft/follows.status.md)
- [`draft/forge-identity.md`](draft/forge-identity.status.md)
- [`draft/forge-polls-quiet.md`](draft/forge-polls-quiet.status.md)
- [`draft/forge-reads-targeted.md`](draft/forge-reads-targeted.status.md)
- [`draft/github.md`](draft/github.status.md)
- [`draft/goal-as-designer.md`](draft/goal-as-designer.status.md)
- [`draft/goal-runs-itself.md`](draft/goal-runs-itself.status.md)
- [`draft/landed-qa.md`](draft/landed-qa.status.md)
- [`draft/long-text-pages.md`](draft/long-text-pages.status.md) — landed whole into main in pulls #157 and #181; plan complete, vision folded into design, read-only text retained pending proposed removal.
- [`draft/look-requests.md`](draft/look-requests.status.md)
- [`draft/macos.md`](draft/macos.status.md)
- [`draft/merge-queue.md`](draft/merge-queue.status.md)
- [`draft/model-catalogue.md`](draft/model-catalogue.status.md)
- [`draft/model-check.md`](draft/model-check.status.md)
- [`draft/own-way.md`](draft/own-way.status.md)
- [`draft/per-wake.md`](draft/per-wake.status.md)
- [`draft/queue-smooth.md`](draft/queue-smooth.status.md)
- [`draft/real-journeys.md`](draft/real-journeys.status.md)
- [`draft/record-anywhere.md`](draft/record-anywhere.status.md)
- [`draft/reference-case.md`](draft/reference-case.status.md)
- [`draft/reference-code-production.md`](draft/reference-code-production.status.md)
- [`draft/redeploys.md`](draft/redeploys.status.md)
- [`draft/repository-copies.md`](draft/repository-copies.status.md)
- [`draft/serve-without-box.md`](draft/serve-without-box.status.md)
- [`draft/sign-in-new-tab.md`](draft/sign-in-new-tab.status.md)
- [`draft/squash-in.md`](draft/squash-in.status.md) — S1, goal 2176: a change's commit into its goal's branch, its title, description and trailers; S2, goal 2231: the stock charters' words for the coding agent, the designer and the coordinator, a change's commits its maker's and main taken only for a change that needs it.
- [`draft/standing-roles.md`](draft/standing-roles.status.md)
- [`draft/stock-ways.md`](draft/stock-ways.status.md)
- [`draft/surfaces.md`](draft/surfaces.status.md)
- [`draft/template-places.md`](draft/template-places.status.md)
- [`draft/to-begin.md`](draft/to-begin.status.md)
- [`draft/tui-design-session.md`](draft/tui-design-session.status.md)
- [`draft/tui.md`](draft/tui.status.md)
- [`draft/upgrades.md`](draft/upgrades.status.md)
- [`draft/v0.1.0.md`](draft/v0.1.0.status.md)
- [`draft/version-rebuilds.md`](draft/version-rebuilds.status.md)
- [`draft/what-reaches-you.md`](draft/what-reaches-you.status.md)
- [`draft/whole-story.md`](draft/whole-story.status.md)

Archived drafts:
- [`draft/archive/reference-landing.md`](draft/archive/reference-landing.status.md)
- [`draft/archive/worker-reply-bound.md`](draft/archive/worker-reply-bound.status.md)

Goal 684 carries R2: areas in words and their owners, the project triager
(kinds 14 and 15), findings held by those owners with news or Needs you,
ctl filing, handing and triage, outside service parties, and stock
triager, prover and porter charters (`domain/areas.status.md`,
`domain/findings.status.md`, `draft/standing-roles.status.md`).
