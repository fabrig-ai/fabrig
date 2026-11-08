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
