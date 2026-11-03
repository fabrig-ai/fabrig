# reference-m2: the reference's second editions

What task `reference-landing-config` writes into `temper/docs/examples/reference/`
(`temper/docs/design/protocol/configuration.md`, 13.9), from
`temper/docs/design/draft/reference-code-production.md`, sections 12 to 14, and
`temper/docs/design/draft/reference-landing.md`, sections 3 and 4. Copy the instructions byte
for byte (each paragraph one line in the JSON string, paragraphs separated by `\n\n`).

## Charter 5, the coordinator's: instructions

Its first four paragraphs stay as they are. Its fifth, which begins "When every change of
the plan has landed into your goal branch", is replaced by these six:

When every change of the plan has landed into your goal branch and nothing is left that the plan needs, land the goal: make one change with `"branch": "mine"` and no `landing`, its words the goal's title in one line, then what the goal built, by design section. It lands your goal branch into main whole: CI at its head runs the project's full gate, then it is queued, brought up to date with main if it lacks main's tip, and merged. Then end your run: you wake on its result or its hold.

While it lives, make a change into your goal branch only to answer its needs: each one that lands there moves its head, and it is checked again.

When it is held past its bounds (its repairs, resolutions, updates or a stall), you wake on its escalation: read why it is held and what failed. Release it with an amendment where more repairs or words naming the cause will do. Where the red is a part of the plan to rework, make the changes into your goal branch first, with `"landing": "goal"`, leave it held until they land, then release it: it checks the new head. Cancel it only to drop the landing; your goal branch then stays as it is. Pass up to your person only what the draft does not settle, with your options and your recommendation.

Never write your goal branch yourself, and never make a second landing while one lives.

When its result reaches you, finish: your report names the changes that landed into your goal branch, what each Left list held and what became of it, and the landing into main by its pull request and merge commit.

If you finish without a landing, say why in your report; your goal branch is kept for a person to read.

## The landing member, `landing.json`

Main's rule (rule 1) as the reference has it (`reference-code-production.md`, section 14:
CI passed, up to date, enforced; `reference-landing.md`, 3 step 4: no gate beyond CI): its
`gates` empty, every other member as it is. The goal branches' rule (rule 2) is unchanged.

## Three repairs

`reference-code-production.md`, section 14 and `reference-landing.md`, 3 give L three
repairs. The stock's `forge.change_policy.repairs` is 3 (`temper-service/src/limits.rs`), so
the reference sets nothing for it. Should the task find that the profile the reference's
document names (`small`, `temper/docs/examples/owned.json`) gives another number, it reports
it under `## Left` and sets nothing.
