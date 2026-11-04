# Request: fabrig keeps temper's acceptance, not a lab's QA runbook

From survey/agents-refs, 2026-11-04; the user approved the plan (step 10 of 12). fabrig names nothing of
whoever runs its development. The lab that walks temper's ladders now keeps its own kits, rung
briefs, backlogs and runbook. fabrig keeps what temper promises: what an agent driving temper gets,
and temper v0.1.0's acceptance stories. Documents only. Two or three direct tasks.

## 1. temper/docs/development/qa.md: the contract stays, the runbook goes
Keep what temper offers an agent driving it, which `tools.md`, `agent-docs.md`, `configuring.md`
and `crates/.../rung.rs` cite: whom a pass plays, the hands, reading the forge, checking an outcome,
what it asks of temper. **Keep section numbers 3, 5 and 6 as they are** (a document outside the
repository cites them): section 3 stays, in generic words, as what a pass needs of a deployment.
Remove who marks, chooses, starts and triages a pass; rungs gated by anyone; lending logins;
references to another repository's QA flow or role (~l.7, 98, 135, 176, 265, 300); machine names.

## 2. temper/docs/design/qa.md
Stays as a pointer page: drop the sentence naming a plan's kit and when the page goes; it says only
that the QA page moved, and where.

## 3. temper v0.1.0's acceptance stories
Before the plan directories go, write temper's own acceptance stories from their rung briefs
(`temper/docs/plan/v0.1.0-ladder/rung-*.md`, `temper/docs/plan/reference-walks/rung-*.md`): a page
`temper/docs/development/acceptance.md`, one section per story: what a user does and what must
hold, with its source twin or fixture where `temper/docs/design/domain/testing.md` 2.7 needs one.
No host, workspace variable, lent login, session or walker. `temper/docs/design/draft/v0.1.0.md`
(~l.24, 28, 94, 118, "a QA session walks it", "the director triages") states the bar and cites the
page, without saying who walks it.

## 4. What leaves
Delete `temper/docs/development/v0.1.0-ladder.md` and `reference-walks.md` (generated backlogs, now
kept outside the repository), `temper/docs/plan/v0.1.0-ladder/` and `temper/docs/plan/reference-walks/`
whole; drop their lines from `tests/boundary/outward.txt` and any citation of them.

## Bounds
- Documents only (a code comment citing a removed path may change).
- `tools/citations/citations.py --all` at 0; the outward check green; these files' entries gone.
