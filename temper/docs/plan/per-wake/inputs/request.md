# per-wake: a run reads the instructions for why it runs

`temper/docs/design/draft/per-wake.md` gives a charter `parts`, each for some reasons a run runs, and a task's LLM choice a `later`, so the goal's answers read an answer's instructions at an answer's effort, as the pipeline we develop with does. This plan builds the draft's jig half: its sections 2 to 5, in the order of its section 9, item 1. The draft wins where other designs differ, and its status file names the sections that lag. temper's half (its settings form, `temper ctl settings`, the reference's charter 5 in parts and template 4's `later`) is the next plan, so keep this one to jig and what jig's own worlds need.

## What's wanted, ranked. Drop from the bottom if it passes eight tasks, and say what you dropped

1. **The story first, in jig's lowest world that holds a task with its inbox and runs:** a task whose charter has a shared text and two parts, one `on` `first`, one `on` `question`, and an LLM choice with a `later`.
   - Its first run reads the shared text and the first part, at the first choice.
   - A question arriving during that run isn't offered to it, keeps its place, and wakes the next run.
   - The next run reads the shared text and the question part, at `later`.
   - An escalation left undecided when a run ends wakes the next run.
   - A person's `words` and an `amendment` reach any run.
   - A charter without parts behaves as today.
   Each a row of a table where one fits.

2. **The design says so, before code:** the draft's section 9, item 1 names the sections: jig's `domain/tasks.md` sections 3, 8 and 9; `domain/engine.md` 7.1, 7.2, 7.4 and section 9; `domain/hosts.md` 6.4; `protocol/configuration.md` sections 3 and 6; `protocol/store.md`; `client/domain/contract.md` 5.1. Bring the draft's status entries for 2 to 5 current.

3. **A run's reasons** (draft, section 2): `first`, the kinds that woke it as jig's store names them, kept on the task's claim across a continuation and a retry.

4. **A charter's `parts`** (section 3): their shape in configuration, store and client records; `charters.parts` 8 in every profile; refusal at the path of an unknown kind, an always-offered kind, an empty `on`, a part past the limit, and parts on a charter whose runs resume. A run's instructions are the shared text, then each matching part once, in order. A charter's stored policy, parts included, stays within the store's run bound.

5. **What a run is offered** (section 4): the kinds its parts cover plus the always-offered set. A turn's fence takes only offered messages up to it, and a kept message is judged again when the run ends. The same for a message a pace keeps back. A message arriving between a run's preparation and its claim follows the prepared parts.

6. **An entry left undecided wakes the next run,** with that entry's kind as its reason.

7. **`later`** (section 5): on a task's LLM choice, taken by every run whose reasons aren't `first`; checked as the first choice is, at batch, amendment, each run's check and restore; refused on a task whose charter resumes and on a change's `llm`; a provider change at the first later run re-chooses the kept account; the task's document shows the next run's choice.

## Not in this plan

- temper's settings form, `temper ctl`, the reference's charters, template 4 and section 8's refusal words: the next plan.
- An LLM choice per kind of wake, parts for resumed runs, and documents per part: the draft's Later.
- **A chat keeps one choice.** A chat's picker writes its task's LLM choice, and its next turn takes it. Only the goal's template gives a `later`, so nothing a chat does changes. If the mechanism would make a choice written without `later` keep an older `later`, say so in your report: an amendment that gives the first choice alone must leave the task with no `later` unless it says otherwise.

**Tests stay cheap:** the focused and fuzzy limits are being tightened, and the gate will reject a plan that adds cost. A new case of one behaviour is a row of an existing table test, and a new story goes in a world below the real loop. Failing tests first; no test reaches a real host. **A plan runs the focused and fuzzy suites alone and is complete when it lands (`AGENTS.md`, 5).** This plan changes stored formats and the client's charter and task records, which their other owners have been told of; any further stored format or protocol change is a question first. Formats change in place before v0.1.0. jig never names temper, and skein's interface changes get a heads-up. `walk = false`.
