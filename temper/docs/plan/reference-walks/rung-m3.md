# Rung M3, a goal for a draft: surveys as sub-agents, designers as changes, the close-out, all into the goal's branch

With `kit.md` beside this file (the machine, the login's move, the configuration, findings and
the report). Backends: **ChatGPT with an owned Forgejo**; the goal's work at `gpt-6.1-sol`.

## What this is

The walk of the reference case's milestone M3, designers and the cut
(`temper/docs/design/draft/reference-case.execution.md`; the story:
`temper/docs/design/draft/reference-case.md`, steps 1 to 6 and 22; the design:
`temper/docs/design/draft/reference-code-production.md`, sections 4 to 6 and 14). The person
starts a goal for a draft; the goal surveys with inspect-only sub-agents, makes the shared shape
and a designer's change per family, then each family's batch; every change lands into the goal's
own branch, and the goal's close-out lands last. The happy path only: no questions, no early
starts (each designer starts once the shared shape has landed), no restarts. `temper help` and
`temper ctl help` are the person's guide. This brief is the rung's acceptance: it is rerun
unchanged until it passes.

## The scenario

1. **The box,** as rung 0 walks it: configured, the four checks `true`, started as a user unit,
   signed in. The budget raised before any forge is set up, sized as `temper help budget` says for
   a goal of up to a hundred changes; say the figures.
2. **The repository:** an owned Forgejo set up through ctl with a first repository `fabrig` in a
   new project (`forge setup --org fabrig --repository fabrig`, or as the help names it), then, as
   the person, its `main` set to fabrig at the walk's commit (`git push` from REPO's commit to the
   repository with the person's Forgejo login; kit.md's hands). Read it through the API with the
   read header: `main`'s head is the walk's commit, and
   `temper/docs/design/draft/archive/playground-reset.md` is in it.
3. **The reference's configuration:** the settings of
   `temper/docs/examples/reference/settings.json` (the coordinator's charter 5, the designer's
   charter 6, the coding agent's charter 2, template 4 "goal for a draft" and the landing rules for
   `main` and goal branches), applied through ctl as `temper help configure` walks it, adapted only
   where the help says: their models on what the box offers (`gpt-6.1-sol` for the goal's work,
   made offered as the help says), their landing rules naming this box's `fabrig/fabrig`. Then
   `settings show` names each, and `settings show template 4` names *its own goal branch*.
4. **The login in:** ctl's account import of `W/account.json`, `W/give-back` written at once, the
   account usable.
5. **The goal** (step 1): the person starts a goal from template 4 in the `fabrig` project,
   through ctl, its words exactly:

   > Implement `temper/docs/design/draft/archive/playground-reset.md`.

   Expected: `task show` names template 4's charter and LLM choice, tracked; the goal holds its own
   branch g, `temper/<G>/g<G>`, which the forge shows at `main`'s tip.
6. **Surveys as sub-agents** (step 2): the goal's first run reads the draft and the designs it
   touches and asks inspect-only sub-agents, side by side, what the code does today. Read through
   ctl (the goal's activity, its run's sub-agent sessions): each sub-agent's LLM choice, what it
   read, that it changed nothing, none refused.
7. **The plan and its designers** (steps 3, 4 and 6, without step 5's question): the goal makes S,
   the shared shape, and one designer's change per family that needs one, each after S, its
   producer under the designer's charter 6 with its LLM choice, and the changes it specifies itself.
   S lands into g; each designer then writes its family's design into g, lists its changes in its
   report, and lands into g; each result wakes the goal, which makes that family's batch. Follow
   each through ctl (`task show`, `change show`, the goal's tree) and the forge (g's head, the pull
   requests into g, their merges). `main` does not move.
8. **The close-out** (step 22): once every change has landed into g, the goal makes X, the
   close-out, under charter 6; X lands into g. The goal reports and ends; g is kept, unlanded.
9. **Clean up:** the login back, the box stopped. Keep all of `W`.

## What passes

Read on both sides: the goal from template 4, on its own branch g made at `main`'s tip; its
surveys as inspect-only sub-agents side by side, none refused; S, the designers' changes and each
family's batch made and landed into g as steps 3, 4 and 6 say, each producer under its charter at
its LLM choice; X landed into g, last; the goal's report; g kept and `main` unmoved; temper's
account and the forge's agreeing at each step; the login back before the stop; every step from
`temper help` and `temper ctl help`, or each place they did not get you there filed as a finding.
Where a step has no way through ctl (a step the help says ctl does not do), file it as a blocking
finding, then go on by the nearest way the help names, if any, and say which.

Say in the report: the figures, each task's number, charter and LLM choice, the changes and their
pull requests into g, the minutes from the goal's start to X's landing, and the goal's spend.
