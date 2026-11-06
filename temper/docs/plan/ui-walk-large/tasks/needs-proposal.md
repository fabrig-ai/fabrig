+++
id = "needs-proposal"
part = "w3-needs"
kind = "feature"
needs = ["proposal-projects", "home-true"]
files = [
  "temper/crates/temper-client-domain/src/pages/inbox.rs",
  "temper/crates/temper-client-domain/src/pages/needs.rs",
  "temper/crates/temper-client-domain/src/tests/pages/inbox.rs",
  "temper/crates/temper-client-view/src/cards/proposal.rs",
  "temper/crates/temper-client-view/src/words.rs",
  "temper/crates/temper-client-view/src/tests/cards.rs",
  "temper/crates/temper-fake/src/backends.rs",
  "temper/tests/client/tests/suite/configuring_chat.rs",
  "temper/docs/design/client/journeys/**",
]
effort = "high"
minutes = 90
+++

# A settings proposal reaches the project's Needs you

## Goal

W3's client half, and its proof. With `projects` on the row, the project's Needs you shows a settings proposal, the rail counts it, Accept works from there and the rail falls, while the chat's card stays. Prove it with a world test and a browser walk of Configure temper's own path, which also takes in W6's return. Then reprint the journeys once over the plan's changes.

Acceptance:

- A configuring chat's proposal of project 1's ways: the card on project 1's Needs you and Home, and the rail "1". Accept from there makes the revision; the rail's count goes; the chat's card says accepted.
- A proposal of the deployment's charter: the same in each project the person owns.
- A proposal of two projects' items: in both; its title names both.
- The card's title never says "0", and its waiting line never says "Owners of project 0".
- A browser walk at 1440 and 420 on a fresh fake box, with its evidence named in your report.
- `temper ctl inbox` and `proposal show`, with `--json`, carry `projects`.
- The journeys' prints are fresh.

## Inputs

- `temper/docs/plan/ui-walk-large/inputs/request.md`, W3.
- `temper/docs/plan/ui-walk-large/notes/common.md`; `notes/contracts.md`, 2 and 5.
- `temper/docs/design/client/README.md`, 3.5, `Inbox`, *The rail's count* and *Needs you*; 5.13, *The card in a chat*.
- `temper/docs/design/domain/agent.md`, 8.1.
- `jig/docs/design/client/domain/contract.md`, 5.1; `jig/docs/design/domain/engine.md`, 17.6.
- `temper/docs/plan/ui-final-walk/findings.md`, W3 reproduction (`origin/ui-final-walk/integration` if the trunk lacks it).

## Owns and may change

Owns the `files` above.

May also change, each only so:

- `temper/crates/temper-client-tools/src/jig/**` and its tests: `projects` in the `inbox` and `proposal` records and their words, where a person reads which project. `temper/docs/help/reference/*`, by its own tool. `temper/tools/stage-one`'s fixtures, by theirs, if they print those records.
- `temper/crates/temper-fake/src/tests/*`, for the new cue.
- `temper/tools/gallery/**`: rows for the card of two projects and of the deployment, `names.txt` kept sorted.
- `temper/docs/design/client/README.status.md`: remove the **Not yet** on 3.5 that starts "*The rail's count*, `Inbox`, and 5.13's card". Only that.

Don't touch `jig/` or `stock.rs`; a change `projects` needs in jig is a question.

Options, each pre-approved:

- **`Card.project` stays the row's `project`, 0,** since `acts.rs` sends it as the decision's scope and jig refuses any other. Filtering and counting read `shows_in`, never `Card.project`.
- **Accept with `running`:** the Needs card offers what the chat's card does. If the flow card doesn't draw that choice, it takes the chat's default and says so in your report.
- **The cue's `against`:** the fresh box's newest revision. The walk proposes before changing any setting; if the box's newest differs between runs, read how `settings` numbers a fresh box and fix the walk's order.
- **A walk step only a person's real session can give:** drive it through CDP as a person would, by role and name. Don't reach into the page's state.

## Steps

1. **The domain.** `pages/inbox.rs`: `add` takes the row (or a `shows` flag) and filters with `row.shows_in(filter)`. `pages/needs.rs`, `count`: `row.shows_in(project)`. A test in `tests/pages/inbox.rs` for each; break each once (back to `row.project`), see it fail.

2. **The card,** `cards/proposal.rs`: the title from `proposal.projects`, by `notes/contracts.md`, 5; `HistoryHolder::Policy` with project 0. A test in `tests/cards.rs` for none, one and two, and for the waiting line.

3. **The fake model.** A `cued` entry in `model_document`, as "Write this up as a draft beside us." is: the words a walk types into Configure temper, then a turn calling `propose_settings` with one `project-ways` item for project 1, and a closing turn of words. Its arguments are `jig-protocol-tools/src/decode.rs`'s; the item as `temper/tests/client/tests/suite/settings_tools.rs`'s `PROJECT_WAYS_FILE`.

4. **The world,** `temper/tests/client/tests/suite/configuring_chat.rs`, beside `propose`:

   ```rust
   #[test] fn a_configuring_proposal_of_a_projects_ways_reaches_its_needs_and_its_rail()
   // the run proposes project-ways for 1; the front opens Address::Inbox with the project's
   // filter: the card, `needs().count(client, 1)` 1; Act::Decide Accept; revision 2,
   // count 0; the chat's page shows the card accepted.
   #[test] fn a_deployments_proposal_shows_in_each_project_its_owner_owns()
   // a charter's item; two projects owned, one only a member of.
   ```

5. **The browser walk,** a throwaway CDP script, untracked and gone before the push. `temper/docs/plan/ui-final-walk/driver/cdp.py` is the base (`git show origin/ui-final-walk/integration:` if the trunk lacks it). At 1440, then 420 through `Emulation.setDeviceMetricsOverride`, on a fresh fake box with `forge.sitting_ms` a few seconds in its document:
   1. The fake ChatGPT sign-in and the welcome.
   2. Home's "Start": Configure temper, the cue's words, the proposal's card in the chat.
   3. The rail's count 1; Needs you, the card; Accept; the rail with no count; back to the chat, its card accepted.
   4. A chat that lands; a look at Home; the page closed, past the sitting; Home again, the agent's sentence and byline (W6).

   Each step: a screenshot and the page's text, under `~/src/fabrig/ui-walk-large/walk-1440/` and `walk-420/`, beside the mock's `needs.html` where a step has one.

6. **A change of meaning runs its readers' tests:** a project's needs. Besides the implementer's text, run `temper-client-domain`, `temper-client-view`, `temper-client-tools`, `temper-client-shell-terminal`, `temper-client-world`, `temper-fake`, `temper-client-gallery`.

7. **Reprint,** `heavy cargo run -p temper-client-journeys`, over `proposal-projects`' recordings and `home-true`'s words. This task's reads don't change the exchange. If replay says otherwise, `again NAME` by `notes/common.md`.

8. fmt, `old_forms`, clippy, `python3 -I -B tools/citations/citations.py --base origin/main`; commit and push.

## Report

As `notes/common.md`, *Your report*, and each walk step's evidence path, at both widths, with anything it showed against the mock.
