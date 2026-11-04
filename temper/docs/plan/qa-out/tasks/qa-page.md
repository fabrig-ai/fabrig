+++
id = "qa-page"
part = "qa-page"
kind = "docs"
needs = []
files = [
  "temper/docs/development/qa.md",
  "temper/docs/design/qa.md",
  "tests/boundary/outward.txt",
]
effort = "high"
minutes = 40
+++

## Goal

`temper/docs/development/qa.md` says what temper offers an agent driving it, and nothing of who
runs a pass or how a lab organises passes. `temper/docs/design/qa.md` says only that the page
moved, and where.

Acceptance:

1. `grep -n '^## ' temper/docs/development/qa.md` prints the eight headings of
   `temper/docs/plan/qa-out/notes/contracts.md`, C1, unchanged.
2. This prints nothing:

   ```sh
   grep -n -i -E '\bdirector\b|agents repo|gpt-6|luna|claude-sonnet|\bbrief|accounts held|held (login|account|configuration)|kept for (QA|it)|\blent\b|\blend|our pass|triage|temper#|logins give|the user, 10|207,?552|449,?696|415,?104|jq -e' temper/docs/development/qa.md
   ```

3. `temper/docs/design/qa.md` is exactly the two paragraphs in step 3 below.
4. `tests/boundary/outward.txt` has lost `temper/docs/development/qa.md 3`, and only that line.
5. Both checks of `temper/docs/plan/qa-out/notes/common.md` pass.

## Inputs

- `temper/docs/plan/qa-out/inputs/request.md`, sections 1 and 2.
- `temper/docs/plan/qa-out/notes/common.md`, all of it; it departs from your implementer's text.
- `temper/docs/plan/qa-out/notes/contracts.md`, C1 and C3.
- The passages that cite the page, to keep true: `temper/docs/design/client/tools.md`, lines 8 and
  66; `temper/docs/design/draft/agent-docs.md`, lines 66 to 69 and 332;
  `temper/docs/design/draft/configuring.md`, lines 349, 396 and 412;
  `temper/docs/design/draft/v0.1.0.md`, lines 68, 94 and 106;
  `temper/crates/temper/tests/support/forge_land/rung.rs`, the comment on `deployment` (it cites 3 for
  "QA's model/room recipe": section 3 keeps that).

## Owns and may change

- **Owns:** `temper/docs/development/qa.md`; `temper/docs/design/qa.md` (this task's departure
  from "not yours": the request names it).
- **May change:** `tests/boundary/outward.txt`, that one line only.
- **Touches nothing else.** The files that cite the page need no edit: its sections stay.
- **Options:**
  - If a sentence not listed below names a lab, its people or its choices, put it in the
    generic words the same way and say so in the report.
  - If a citation elsewhere stops being true, leave that file and report it.
  - Don't cite `temper/docs/development/acceptance.md` (contracts, C3).

## Steps

### 1. Rewrite `temper/docs/development/qa.md` by section

Edit in place: what is not named here stays, word for word. The quoted text is the page's new text.
Wrap it as the page wraps, at about 72 columns.

**The opening paragraph** becomes:

> Provisional, 2026-10-26, revised 2026-11-04. What temper offers an agent driving it in a pass of
> QA after landing: whom it plays, the deployment it drives, the hands it holds, how it reads the
> forge and how it checks an outcome. The product's contracts it relies on are temper's design: the
> box's forge hands (`../design/protocol/setup.md`, 7.3) and ctl's help
> (`../design/client/tools.md`, 11.5). What is still open is section 8.

**1. In one page.** The bullet "temper is agnostic" keeps its first sentence, then ends:

> It serves its own part, and names the rest by id and link; how a user reaches the forge is the
> user's choice. A pass here chooses `curl` (section 5).

The bullet "The deployment is a box" becomes:

> - **The deployment is a box,** on its fake backends or on real ones, each pair of backends a rung
>   (section 3).

**2. Whom a pass plays:** stays.

**3. The deployment.** The heading stays. Its bullets become these, in this order, and nothing else
stays:

> - **A box in the pass's scratch directory,** started as the walk `box` says (`temper help box`),
>   with the flags of its rung, and stopped by the pass before it ends.
> - **The rungs:**
>
>   | Rung | Forges | Model | What it needs |
>   |---|---|---|---|
>   | fakes | `--fake-backends` (`../design/protocol/setup.md`, 7.5): GitHub and a Forgejo elsewhere faked; Forgejo here real, from the box's cache | the fake model, by the key file the box prints | none |
>   | a real model | Forgejo here, set up through ctl, as the walk `box` sets it up | ChatGPT, a login imported and exported back, or a key read from a private file (`../design/client/tools.md`, 5.5, area 5) | the model's account |
>   | GitHub | GitHub, through an App, on a repository the pass may change | as a real model's rung | the App's key and its installation, and a person's token on that repository |
>   | Claude | Forgejo here, as a real model's rung | Claude Code on a person's own login, its configuration directory used in place (`../design/protocol/credentials.md`, 3) | the model's login |
>
> - **The model is set before the first start.** Init's model entry is kept as it is (its window,
>   prices and output), its efforts declared in `temper.json`'s `engine.llm` model, the first being
>   what a task with no choice of its own takes. Every charter takes the first configured model as
>   its main model, and the stock's spends are sized from it at the start
>   (`../design/protocol/configuration.md`, 13.4). After the start, `temper ctl settings show
>   charter 1` names the model; a box that shows otherwise is a setup to redo, not a finding. A chat
>   is started with `--model MODEL --effort EFFORT` where its model is named; template 1's choice
>   (`settings apply`, the whole choice: `{"main": {"model": MODEL, "effort": EFFORT}, "account":
>   null, "subagents": []}`) sets it for every chat of a running box.
> - **Its room is raised beside it,** in `temper.json` before the start, where a pass needs more
>   than the profile gives: `engine.limits`' override `"people.requests"`, since each deciding
>   request's answer is kept a day, which a busy pass fills, after which every keyed request answers
>   busy (jig's `domain/people.md`, 5.1); and `application.limits`' override `"root.loads.loads"`,
>   since a restart with held work loaded needs a slot for it. Its retention stays a day, and the
>   memory the start then names is raised as `../design/protocol/configuration.md`, 13.8 says. These
>   are a pass's, as the budget is; the product's own values are its design's.
> - **One configuration, checked before every start.** Every change above is made in `CONFIG` after
>   `temper init` and before the box's first start, and `memory_bytes` raised by what the start
>   names when it refuses a short one. Before the first start and before every restart, a pass
>   checks that `CONFIG` says what it set; one that differs is a setup to redo, never a finding.
> - **Give back before you break it.** A case that aborts `serve`, kills the box or otherwise risks
>   the deployment's restore first exports a login moved into the box (`account export`): what the
>   deployment holds of its work (a task, an unanswered question) stays in its store, and a restart
>   needs no model. It imports the login again only if a later case needs the model. A login a
>   broken deployment still holds is a finding, said with the case that broke it.
> - **A budget never stops a pass.** A pass raises its box's budget as a person would (`temper help
>   budget`), and says the figures. Init's first start already makes project 1 and its owner. After
>   sign-in, the pass raises the deployment limits in one `settings apply` edited from `settings
>   show rules --json` and `settings show policy 1 --json`: the rules' spend and period spend, policy
>   1's ceiling's spend and period spend, and role 0's authority spend and its period spend. The
>   period stays larger than role 0's pool, so that the deployment's own work (a landing task, a
>   tier run, a queue's repair) is still funded. Nothing per run; the chat's template stays as
>   allowed. Policy 1 sizes future projects; `settings apply` does not resize the existing project's
>   period or owner's pool. The pass must then fund the project listed by `me`, with distinct
>   operation keys, the project first because the owner's pool reserves from its period:
>
>   ```
>   temper ctl DIR me
>   temper ctl DIR project fund PROJECT PERIOD --key budget-period
>   temper ctl DIR project fund PROJECT POOL --role 0 --key budget-owner
>   temper ctl DIR project show PROJECT
>   ```
>
>   `PROJECT` is the project `me` lists, 1 on a fresh box; `PERIOD` and `POOL` are the pass's figures.
>   `project show` prints the open period's budget and the owner's pool's, with their reserved, spent
>   and left balances. These funding calls resize the open budgets at once. `forge setup` then
>   adopts the repository into this existing project.
> - **Real accounts are the person's,** used by path and never read; a login moved into a box goes
>   back before the box stops, an imported login exported to its file.

So the two model bullets, the `jq` block, "Down before up" and the `temper#16` sentence are gone.

**4. The hands.** In the third row, "as a case's step written in its plan" reads "as a case's step
names it". The rest stays.

**5. Reading the forge.** Five phrases, the rest staying:

| Old | New |
|---|---|
| Our passes read Forgejo | A pass here reads Forgejo |
| a finding against the fake, routed to skein's owner | a finding against skein's fake |
| as the case's plan names them | as the case names them |
| a fine-grained token kept for QA on the repository kept for it | a fine-grained token on the pass's repository |
| the person's header file is the brief's | the person's header file is the pass's own |

**6. Checking an outcome.** Two phrases, the rest staying:

| Old | New |
|---|---|
| what the forge showed, in the shape the agents repository's QA role gives. | what the forge showed. |
| goes under Blocked, once, with what the pass did instead | is named once as blocked, with what the pass did instead |

**7. What it asks of temper:** stays.

**8. Open questions:**

- "The GitHub rung's App": `made by the user for QA, or from temper's manifest` reads `made by the
  person for the pass, or from temper's manifest`; its second sentence stays.
- "Real accounts' holding and spend" goes.
- The other two stay.

### 2. Reread it whole

As an agent that knows nothing of any lab: every sentence must still say something temper offers,
needs or checks, and section 1's bullets still match their sections.

### 3. Replace `temper/docs/design/qa.md`

Its whole text:

```markdown
# QA pass: moved

temper's QA page moved to [`temper/docs/development/qa.md`](../development/qa.md).
```

### 4. Drop the inventory line

Remove `temper/docs/development/qa.md 3` from `tests/boundary/outward.txt`.

### 5. Check, commit, push

Commit first: both checks read what git tracks. Run acceptance 1, 2 and the two checks of
`notes/common.md`. Push your branch.

## Report

As your implementer's text says. Also: any sentence you put in generic words beyond those listed,
and any citation elsewhere that this page no longer backs.
