# Status

2026-11-11. The first milestone stands in the code. Remaining work spans
landing and finding workflows, client surfaces, deployment and releases,
and verification and cost. The status files below record where temper's
code lags or departs from its design, and its design from its drafts;
each document keeps its deferred work and open questions. Team shows from a
project's second person or its first standing role. P4b, goal 713, draws
the project's standing roles, a row each, and a role's card: its head
with Pause and Resume, its Now, its rules, what wakes it and its
sessions this week, with two gallery states; a forge topic's words, a
charter's kept words and a session's line wait for reads. Areas, owners,
the triager as Who routes and a card's "Talk to it" are P4c's, and a
line's sign-offs wait on sign-offs by path and kept sign-offs
(`client/pages/team.status.md`; `draft/surfaces.status.md`, 6).

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
`report` note, with its Now in Home's brief's words; areas and routing remain
a later milestone, and Team's and the terminal's talk and report surfaces
remain the pages' (`draft/standing-roles.status.md`).

QA's hands are landed-qa's milestone Q2,
by goal 1000: a drive session on a fake box from a given build, taking a
person's steps and looks a line at a time; recipes and their walk; the
fake model's given scripts and strict switch; a store this build cannot
read refused by name; and a box started again with a newer build, whose
process-tier story runs after landing (`client/drive.md`;
`client/recipe.md`; `protocol/setup/box.md`, 1 and 5; `protocol/store.md`,
5; `draft/landed-qa.status.md`).

Landed QA's milestones Q1, Q3 and Q4: `temper-qa`
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
across a restart. Fake-model scripts take values from the conversation,
so B0's twin replays as setups on a box's own Forgejo; B1's waits on how
its gate's commit is made. Its passes follow landing
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
`follows` document. A standing role as owner waits on R1; the follow
change's words carry no diff, and a person's follow's holds reach the
project's escalation role, each asked of the user
(`domain/forge/follows.status.md`; `draft/follows.status.md`). A
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
Its walk follows landing; team lines, `/talk` and `/tell` remain T3's,
and Stop asking in the terminal remains the web's
(`draft/surfaces.status.md`, 11).

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
2). Its walk on a fresh box follows landing; overlays are S2
(`draft/stock-charters.status.md`).

Design documents:
- [`README.md`](README.status.md)
- [`cli/README.md`](cli/README.status.md)
- [`client/README.md`](client/README.status.md)
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
- [`draft/box-link-first.md`](draft/box-link-first.status.md)
- [`draft/bundle-loading.md`](draft/bundle-loading.status.md)
- [`draft/chat-reading.md`](draft/chat-reading.status.md)
- [`draft/ci-on-forgejo-17.md`](draft/ci-on-forgejo-17.status.md)
- [`draft/claude-api.md`](draft/claude-api.status.md)
- [`draft/claude-provider.md`](draft/claude-provider.status.md)
- [`draft/cli.md`](draft/cli.status.md)
- [`draft/configuring.md`](draft/configuring.status.md)
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
- [`draft/reference-case.md`](draft/reference-case.status.md)
- [`draft/reference-code-production.md`](draft/reference-code-production.status.md)
- [`draft/repository-copies.md`](draft/repository-copies.status.md)
- [`draft/serve-without-box.md`](draft/serve-without-box.status.md)
- [`draft/sign-in-new-tab.md`](draft/sign-in-new-tab.status.md)
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
