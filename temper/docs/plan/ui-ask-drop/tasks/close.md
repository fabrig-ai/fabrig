+++
id = "close"
part = "close"
kind = "docs"
needs = ["domain-configuring", "domain-settings", "joined", "client-names", "gallery", "stage-one", "help", "stock", "journeys"]
files = ["temper/docs/design/STATUS.md", "temper/docs/design/draft/ask-temper.execution.md", "temper/docs/design/**/*.status.md"]
effort = "medium"
minutes = 25
+++

# close: the drop's state in the design's records

## Goal

Once every task of this plan has merged, the design's records say the configuring chat
is gone: no **Not yet** entry this plan's design wrote is left, `ask-temper.execution.md`'s
M1 says the configuring chat went, and `STATUS.md`'s summary names no charter 3 or
template 3 in the stock.

Acceptance:

- `rg -n 'charter 3|template 3|configuring chat|Configure temper|configuring agent' temper/docs/design --glob '*.status.md' temper/docs/design/STATUS.md`
  finds no line that says the code still holds or runs the configuring chat; each hit
  left is a departure or a draft's lag that stays true, named in the report.
- The **Not yet** entries this plan's design batch wrote (listed below) are gone: each
  task deleted its own; any left because its task didn't is deleted here only if the
  merged trunk holds what it says is not yet (check the code), and otherwise left with the
  task named in the report.
- `temper/docs/design/draft/ask-temper.execution.md`, section 3's M1 row and its entry
  say charter 3, template 3, `Configure` and Settings' offer are gone, by the plan
  `ui-ask-drop`.
- `python3 -I -B tools/citations/citations.py --base origin/main` lists nothing new.

## Inputs

- `DESIGN.md`, sections 2 and 3 (status files hold only gaps; execution files keep each
  milestone's state current). This task departs from the implementer's text, which
  closes `docs/design/`: its files are these records alone.
- `temper/docs/plan/ui-ask-drop/plan.md` (what each task built).

## Owns and may change

- `temper/docs/design/STATUS.md` (its paragraphs on Ask temper and the configuring
  agent), `temper/docs/design/draft/ask-temper.execution.md` (M1's row and entry), and
  in any `*.status.md` under `temper/docs/design/` only the entries the acceptance names.
- Never a design document's own text, a draft's text, or code.

## Not yet entries of this plan

Built by tasks whose specs were released before these entries were written, so they are
this task's to delete once the trunk holds what they say is not yet:

- `temper/docs/design/domain/testing.status.md`: the bullet beginning "**Not yet:** 6.4,
  the engine's domain world's configuring by Ask temper" (`domain-configuring`,
  `domain-settings`); the bullet beginning "**Not yet:** 6.4, the client and the engine,
  joined" (`joined`).
- `temper/docs/design/client/README.status.md`: the bullet beginning "**Not yet:** 10,
  the joint world's Ask temper chat" (`joined`).
- `temper/docs/design/client/README.status.md`'s bullet beginning "**Not yet:** 5.3: the
  rename's form asks the focus" names `configure.rs` among the pages that guard their
  focus: name the module `client-names` renamed it to (its report; `rg` the trunk), the
  entry otherwise kept.

Deleted by their own tasks, checked here: `protocol/configuration.status.md`'s two and
`domain/agent.status.md`'s two (`stock`), `client/browser.status.md`'s 7.2 gallery names
(`gallery`), `client/journeys.status.md`'s and `client/tui/journeys.status.md`'s 9
(`journeys`).

## Steps

1. For each entry above, check the trunk (`rg` the code it names) and delete it, or
   leave it and name the task in the report.
2. Edit M1's row and entry and `STATUS.md`'s paragraphs.
3. Run the citations check; commit ("Record the configuring chat's drop") and push.

## Report

Each entry deleted or left (and why); the lines changed in `STATUS.md` and the
execution file; the citations check's result.

## Rebase (2026-11-08, step merge, foreman ui-ask-drop/close/foreman-1)

reconcile the six merge conflicts while preserving the configuring-chat removal and main’s landed journey and recorder updates.
The merge failed on content conflicts, not a test or infrastructure failure.
In `temper/docs/design/STATUS.md`, retain close’s removal statements and main’s Ask temper placement, sign-in-tab, redaction and recorder-key updates.
Main’s relevant commits are `16893e054a` (look-round retirement) and `8a91d0b749` (recorder-key retirement).
In `temper/docs/design/client/journeys.status.md`, preserve main’s placement and sign-in-tab facts and its welcome-playground redaction correction.
Keep the remaining chat-draft stock-removal gap unless inspection of the resulting recording proves it resolved; offering Ask temper does not prove template 3 and charter 3 are absent.
Main regenerated chat-draft in `6eafc361d3`; the journeys task’s answer explicitly permits leaving its stock-removal renewal until after landing.
In `temper/docs/design/client/tui/journeys.status.md`, preserve main’s Alt+m/Alt+s coverage and manual-terminal-check caveat; keep the branch’s deletion of the completed template-3 gap and remove “but the one below.”
For `temper/docs/design/client/journeys/project-landing/{01.html,recording}` and `temper/docs/design/client/journeys/welcome-playground/recording`, retain coherent recording/print sets from branch commit `6075e6c6e4`.
Those recordings were generated after stock removal and already show Ask temper; main independently regenerated them in `1f505044d3` and `263e3f1309`.
Do not splice recording events or mix a recording with another generation’s prints.
Run the focused `temper-client-journeys` test `tests::the_committed_journeys_are_their_print` through heavy.
If combined code makes a recording stale, regenerate it with `temper-client-journeys again NAME`, report the generated changes, and rerun focused replay.
Preserve the auto-merged M1 execution update and run `python3 -I -B tools/citations/citations.py --base origin/main`.
