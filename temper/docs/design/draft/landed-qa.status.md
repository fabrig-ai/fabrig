# landed-qa: status

2026-11-12. Where the design lags or departs from [landed-qa.md](landed-qa.md).

Milestone Q1 is designed in `../testing/qa/README.md` and its documents: the watcher on main's ref, the deployment's kept builds, triage, and the ledger's landings, cases and builds.

Milestone Q2, what the first passes need of temper, is designed and
built: the drive session (`../client/drive.md`), recipes and their walk
(`../client/recipe.md`), the fake model's given scripts and strict
switch (`../protocol/setup/box.md`, 5; skein's `testing/fakes/llm.md`,
2.5), a box started again with a newer build (`../protocol/setup/box.md`,
1) and a store refused by name (`../protocol/store.md`, 5). The draft
leaves `../domain/findings.md`, 8, temper's own pass, as it is (section
2).

Milestone Q3, passes on the web client, is designed in
`../testing/qa/places.md` (places, copies, their making again and the
drive session held open), `../testing/qa/passes.md` (a pass's start,
session, end, resume and report) and `../testing/qa/findings.md`
(findings, their request and its goal), with the ledger's passes,
findings and case outcomes (`../testing/qa/ledger.md`, 2.7 to 2.9 and 4.8
to 4.16) and the first recipes in `../../development/qa/setups/`.

Milestone Q4, the TUI and deeper states, is designed in
`../testing/qa/setups.md` (the setups' catalogue: the TUI's recipe and
B0's and B1's twins as setups), with a pass taking `tui` cases by default
(`../testing/qa/README.md`, 3.2; `../testing/qa/passes.md`, 4), places
with their own ports (`../testing/qa/places.md`, 2), the figures bounds
and places are tuned from (`../testing/qa/ledger.md`, 2.10, 4.17 and
6), fake forges that keep their state across a restart (skein's
`testing/fakes/forge.md`, 2.7 and 5, `testing/fakes/github.md`, 1.7;
`../testing/fakes.md`, 2; `../protocol/setup/box.md`, 5), and scripts
that take values from the conversation (skein's `testing/fakes/llm.md`,
2.6; `../protocol/setup/playground.md`; `../testing/fakes.md`, 1), and
the drive session's `forge` lines, a person's commit and the wait on its
checks on the box's own Forgejo (`../client/drive.md`, 3.5).

- **Design departs:**
  - 6, 10: a take reads the whole conversation, the agent's brief in its system text among it, rather than a tool's output alone, since a review's brief is where its pull request and head are named (skein's `testing/fakes/llm.md`, 2.6). The goal's designer, 2026-11-12.
  - 2, 12: bounds and places are tuned by the deployment from `temper-qa tune`'s figures and suggestions, never by the loop itself, since `qa.json` is the deployment's (`../testing/qa/ledger.md`, 6). More than one place gives each place its own ports (`../testing/qa/places.md`, 2). The goal's designer, 2026-11-12.
  - 3, 10: the client a build keeps is the TUI's recording build (`temper-client` with `temper-client-shell-tui/record`, `../client/tui/journeys.md`, 2), beside the bundle's recording build: the drive session reads the TUI's frames from its saves, replayed natively, since nothing in the tree reads a pseudo-terminal's screen (`../client/drive.md`, 2.1 and 3.3). The goal's designer, 2026-11-12.
  - 10: the journeys' `again` keeps building from the checkout and recording into it, a person's command (`../client/web/journeys.md`, 8.1); the drive session and the recipe walk are what walks and drives with a given build in a given scratch directory (`../client/drive.md`, 2; `../client/recipe.md`, 5), since `again` writes into the checkout, which a QA session never touches. The goal's designer, 2026-11-12.
  - 7: the session signs the TUI in itself, by the browser's approval, when a line first needs it, so that no line and no recipe approves and a recipe walks the same from a fresh box or a copy (`../client/drive.md`, 3.3). The goal's designer, 2026-11-12.
  - 5, 7, 10: drive sessions and recipes take a fourth client, `forge`: one commit of the files a line names to a repository of the box's owned Forgejo, as the person, with the forge's `person_write.header`, and a wait on its checks, so that B1's twin reaches its gate as its walk did (`../client/drive.md`, 3.5; `../testing/qa/setups.md`, 4.4). The user, 2026-11-12.
  - 6: B1's twin commits the gate its fixture holds, not acceptance.md, 7's later text, which the twin never met (`../testing/qa/setups.md`, 4.4). The goal's coordinator, 2026-11-12.
  - 3: a kept build's `bin/fake-claude`, smith's fake of the coding agent's command line, prints no temper commit as its version; it is held by the build's manifest alone, and every temper executable of the build prints the commit (`../testing/qa/builds.md`, 3). The goal's designer, 2026-11-12.
  - 5, 7: a recipe's step that broke with no landing meaning it to fails the cases on that setup, with its finding, as section 7 has it, where section 5 says they are blocked by it: failed, they come back at the pass's resume (`../testing/qa/places.md`, 7). The goal's designer, 2026-11-12.
  - 8: a pass's request is a report, `temper/docs/development/qa/requests/pass-<P>-<R>.md`, rather than a draft in `draft/`: it lists findings, not a vision, and needs no status file (`DESIGN.md`, section 4). The deployment's command puts it on main and starts its goal, since how a request reaches main and how a goal starts are the deployment's (`../testing/qa/findings.md`, 4 and 5). The goal's designer, 2026-11-12.
  - 9: a pass session that records nothing counts its cases as blocked, so that a session that does nothing is not taken again without end; cases with no outcome go back uncounted beside a session that recorded something (`../testing/qa/ledger.md`, 4.12). The goal's designer, 2026-11-12.
