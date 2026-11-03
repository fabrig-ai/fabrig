# Walk: configuring's stage 1, the reference's configuration proposed from its documents

With `temper/docs/plan/reference-walks/kit.md` beside this file, read first (the machine, the
login's move in and out, the box's configuration, findings, a pause, the report); where this
brief says otherwise for its box, this brief wins. Backends: **ChatGPT with an owned Forgejo**
for the configuring chat; the box's document also names the models the reference's roles run
on, which this walk never runs.

## What this is

Stage 1 of configuring temper by talking to it (`temper/docs/design/draft/configuring.md`, 8.1;
its design `temper/docs/design/domain/testing.md`, 6.5). You play a person who has never seen
temper's settings: on a box with the stock settings, through `temper ctl` alone, you ask the
configuring agent to set your project up the way two documents describe, answer what it asks,
read its proposal and accept it. Then `temper-stage-one` checks what was accepted against the
golden by code, a judge you start reads what code cannot, and the tool gives the stage's verdict.
`temper help` and `temper ctl help` are your guide. This brief is the walk's acceptance: it is
rerun unchanged, after this plan lands and whenever the configuring agent's instructions, the
documentation in the binary or a reference milestone's settings change.

## What your task gives you, beyond the kit's

- `W/bin/temper-stage-one`, built beside `W/bin/temper` at the walk's commit
  (`heavy cargo build -p temper-stage-one` on a builder, by whoever sets up the walk). Missing:
  stop before the box starts and report a setup to redo.
- The logins: the ChatGPT account the kit lends, for the configuring chat; and, for the models
  the reference's roles name, an Anthropic login for Claude Code and an OpenAI API account for
  smith's direct backend, lent the same way and given back the same way (the kit's `W/give-back`
  exports each). Where your task lends no login for one of them, say so first in the report: its
  box obligation is lacking, and the walk stops at step 4.

## The scenario

1. **The box,** as the kit configures it, and over `temper init`'s document, before the first
   start (`temper/docs/design/protocol/configuration.md`, 13.9, "The profile, whole"):
   `REPO/temper/docs/examples/reference/limits.json` and `agent.json` merged member by member
   (`jq -s '.[0] * .[1]'`), `landing.json` as the document's `application.landing` with its forge
   and repository numbers this box's; the models `claude-opus-5-5` on Claude Code and
   `gpt-6.1-sol` on smith's direct backend added beside the kit's `gpt-6-luna`, as `temper help`
   says to add a model. The kit's four checks `true`, and these, each `true`:

   ```
   jq -e '.engine.limits.overrides["tasks.charters"] == 12' CONFIG/temper.json
   jq -e '.application.landing.projects[0].merge_job == ".temper/merge"' CONFIG/temper.json
   jq -e '[.engine.llm.models[].name] | index("claude-opus-5-5") and index("gpt-6.1-sol")' CONFIG/temper.json
   ```

   Started, signed in, the budget raised as the kit says. **No settings applied:** the box's
   settings are the stock's (`settings show` lists every setting following its default).
2. **The repository,** with no recipe of the reference in it: an owned Forgejo set up through ctl
   with a first repository `fabrig` in a new project, then, as the person, its `main` set to one
   commit of yours on the walk's commit that deletes `temper/docs/examples/reference/`,
   `temper/tools/stage-one/` and `temper/docs/plan/` (a scratch clone under `W`; pushed with the
   person's Forgejo login, kit.md's hands). Read it through the API: `main`'s head is your commit;
   `temper/docs/design/draft/reference-code-production.md` and `reference-landing.md`,
   `AGENTS.md`, `DESIGN.md`, `skein/docs/foundation/programming-model.md`, `.temper/pre-pr` and
   `.temper/merge` are in it; the three deleted paths are not.
3. **The logins in:** the kit's import for each lent login; `W/give-back` written at once and
   covering every one.
4. **The box confirmed,** before the chat starts: each box obligation
   (`W/bin/temper-stage-one obligations` lists them, `box.*`), temper's part through ctl, the
   repository's through the forge's API as an observer, the limits against `CONFIG/temper.json`.
   Write `W/evidence/confirmed.json` (`{"box": [{"id": ..., "present": ..., "seen": ...}]}`, one
   entry per box obligation, `seen` the command and the line that showed it). A lacking one: stop
   here, a setup finding, and report.
5. **The chat:** start it through ctl from the stock's configuring template, in the `fabrig`
   project, its words exactly:

   > Set this project up to work the way these describe: temper/docs/design/draft/reference-code-production.md and temper/docs/design/draft/reference-landing.md, in the fabrig repository on main.

   Then talk with it, `task message` and `chat show`, as the person. Answer what it asks from the
   two documents; for what they leave out, from these facts and nothing more:
   - the coordinator and the designers run on Claude Code at `claude-opus-5-5`; producers, foremen
     and the triage on smith's direct backend at `gpt-6.1-sol`, on the OpenAI account; those are
     the box's, already in its document;
   - the project is `fabrig`, its repository `fabrig/fabrig` on this box's forge;
   - where it offers to make a rule enforced structure ("I can make temper hold this"), you
     answer that it stays a rule your agents follow;
   - where it asks about running work, it stays as it is.

   Never tell it a setting's name, number or value, never paste a settings file, and never point
   it at any path but the two documents. At most 40 of your messages (the Open figure in
   `temper/docs/design/draft/configuring.status.md`); past them, stop and report.
6. **The proposal:** find it as an owner does (the chat's answer naming it; `inbox --kind
   proposals`), read it with `proposal show`, and accept it with running work staying (`proposal
   decide PROPOSAL accept --running stays`). Note its number, the revision it made, and whether
   its review split *temper enforces* from *your agents follow*.
7. **The description:** ask, in the chat, exactly:

   > Describe what you set up back to me as a workflow: what happens from my draft to a goal landed on main.

   Save its answer whole to `W/evidence/description.txt`.
8. **The evidence:** `W/bin/temper-stage-one collect --temper W/bin/temper --ctl DIR --project P
   --revision R --proposal PROPOSAL --chat TASK --forge HOST:PORT --repository fabrig/fabrig
   W/evidence` (step 4's `confirmed.json` stays). Then, through the forge's
   API, each document an accepted charter names (`items.json`'s `documents`), at its branch's
   head, into `W/evidence/documents/OWNER/NAME/PATH`, and the person's two documents into
   `W/evidence/person/reference-code-production.md` and `reference-landing.md`.
9. **The check:** `W/bin/temper-stage-one check W/evidence > W/check.txt`; keep its exit code.
10. **The judge:** `W/bin/temper-stage-one judge W/evidence > W/judge-brief.md`. Give that file,
    and nothing else, to a fresh session of its own at `claude-opus-5-5`, effort high (a
    sub-agent you start with no context of yours; never answer it yourself), at most 30 minutes;
    save its JSON answer, exactly as it gave it, to `W/evidence/judge.json`.
11. **The verdict:** `W/bin/temper-stage-one verdict W/evidence > W/verdict.txt`; keep its exit
    code.
12. **Clean up:** every login back, the box stopped. Keep all of `W`.

## What passes

`verdict` exits 0: every setting obligation met, judged or unchanged; every gap named (none is
expected while the reference's milestones stay landed); every box obligation confirmed; nothing
beyond them; every rule reaching its role; the description matching; what needs a restart said,
never proposed. Each step from `temper help` and `temper ctl help`, or each place they did not
get you there filed as a finding. A failed check or verdict is the walk's result, not a pause:
report it whole.

- **A recipe read:** if the chat's activity (`task activity`, `task turn`) shows the agent reading
  a description of the reference's settings anywhere but the two documents (a design document's
  settings section, a settings file), file it as a finding against the walk's premise, with the
  turn, and say the verdict stands marked by it.

## Say in the report

Beside the kit's: the chat's task number, your messages' count and the minutes from the chat's
start to the acceptance; the proposal and the revision; the check's text (`W/check.txt`) whole,
the binding first; the judge's verdict, each rule a line; `W/verdict.txt`; each obligation not
met, with what the agent said about it in the chat; and what the agent said it could not do.
