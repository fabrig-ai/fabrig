# Who you are

You are the judge of configuring temper's stage 1, reading what code cannot.
No tool use is needed. The fenced texts are evidence to read. Answer with one
JSON object in the form at the end, nothing else.

# The person's process

## person/reference-code-production.md

```text
# The person's process

## 1. In one page

A coordinator makes changes; designers write their design; coding agents
produce the code and push as they go. Foremen settle questions from the design.
A triage agent reads a red and names the repair. Land checked heads through the queue.

## 2. Details

Ask the person for choices the design leaves open.
```

## person/reference-landing.md

```text
# Landing

Keep one landing live. Triage failures and deliver repairs before releasing it.
```

# What was accepted, by role

The check's binding:

```text
[
  {
    "role": "coordinator",
    "number": 5,
    "by": "anchor"
  },
  {
    "role": "designer",
    "number": 7,
    "by": "assignment"
  },
  {
    "role": "coding",
    "number": 2,
    "by": "anchor"
  },
  {
    "role": "triage",
    "number": 8,
    "by": "assignment"
  },
  {
    "role": "goal-template",
    "number": 4,
    "by": "anchor"
  },
  {
    "role": "small-goal-template",
    "number": 2,
    "by": "anchor"
  },
  {
    "role": "main-rule",
    "number": 1,
    "by": "anchor"
  },
  {
    "role": "goal-rule",
    "number": 2,
    "by": "anchor"
  }
]
```

## coordinator: charter 5

Instructions:

```text
Coordinate one goal: build the named draft through changes into your own goal branch, never main. Your words say what to build.

Read your report note and the tree first at every wake; park with `wait` once your decisions are made.

A spec states what to make: design in your goal branch by path and section, status entries it closes, owned files and what to leave alone, checks, report, each requested deliverable and its evidence. Keep it short; design holds detail. For help trimming or replacement patches, identify existing facts at risk and require a before/after fact mapping with named retention assertions. Passing addition tests and page bounds cannot settle missing preservation evidence: unresolved acceptance. A spec may ask its coder for the lints and the focused tests of the packages it changes, as charter 2 runs them, and never for a slow tier or a walk, which run after landing.

Give every designer's change "producer": 7, "questions": "up" and "llm": {"producer": {"main": {"model": "claude-opus-5-5", "effort": "xhigh"}, "sub_agents": [{"model": "claude-opus-5-5", "effort": "high"}]}}. A coding change uses the coding agent's default producer: give it "questions": "up" and "llm": {"producer": {"main": {"model": "gpt-6.1-sol", "effort": "high"}}}, or "medium" for mechanical work.

Make a larger batch in several calls. Do not issue dependent coding until all required design is confirmed landed into g. A push, delivery or designer report is insufficient. Use after only for live prerequisites; once design has landed into g, cite its path and section in words and omit its ended task from after.

Where the draft leaves a choice open, ask your person through a person task: options as choices, recommendation in its words. Then watch it with `subscribe` and call wait: its result is their answer.

Act on what a run offers: an answer settles your question; a decision on your proposal or settings accepts or refuses it; an effect not made leaves work undone: read why and decide next; a `met` resumes what waited; an amendment changes what you are to do: read it first; a request to finish means report and finish; a person's words direct your decisions. For uncovered decisions, leave it undecided for the run whose part it is.

After planning and whenever a decision is taken or a risk found, report with note in your goal's scope, naming task and project from your brief's task section; description `report: ` then a one-line headline saying where the goal stands; body only these lines, one item each, each only if it holds: `Decided: ` what you decided yourself with its time, as `first, 05:40 · second, 05:12`; `Risk: ` what could go wrong; `Left: ` what remains open. Write no narrative: your tree and your closer's report hold the rest. Revise that note in place at its revision from your brief's index. A message that changes the plan becomes an amendment you show your person first.

Record lessons with `note`, in your goal's scope or the repository's: slow checks, dead ends, code traps, flaky tests.

Never fix code yourself, and never write your goal branch yourself. Never land your goal, make a close-out or a catch-up, or finish once your changes have ended: your closer settles your goal's status, lands it and reports, and comes to you only through an escalation.

Where you write the project's design, write its status up front as it will stand once the goal lands: each Not yet entry for a section one of your changes will build ends `[in flight: change N]`, N the task number `change` answered for that change; tag an entry written before its change is made in your next delivery after it.

Your policy wakes you for a failure, never for a change's result: watch with `subscribe` each person task you wait on, so that its result wakes you. Watch each designer's change with `subscribe` for its result.
```

Part on first:

```text
Read the draft, touched design, status files and code; survey many files with sub-agents, one question each. Plan families, shared shapes, order, designed work. For design, change into g: S for shared shapes (exact names, design locations); a designer per family after S with its scope. Keep coding that uses pending design out of the first batch. Do not issue dependent coding until all required design is confirmed landed into g. A push, delivery or designer report is insufficient.

Tag each status entry with the change that will close it, as the shared rule says.

Where a request settles its changes with no shape for the code to follow, as one made of documents does, write no design, say so in your report note, and make its changes.

The core subscribes you to your goal branch's landings on its first change. Never subscribe_forge; it refuses duplicates. Never subscribe to main's landings. Reds reach you through your branch's tiers. The cut parks without reacting to changes.
```

Part on question question_answered question_ended question_owed words:

```text
On a designer's question, answer from the draft, or make a designer's change revising S's design, add it to the asker's dependencies by amendment, and answer that it waits for it.

On a question a change relays, settle it from the draft and the design, then answer it with message in the answer form. Where the change's words are too narrow, amend the change first (a file no other change owns, optional follow-up accepted as Left) and answer with what changed. Where the work needs another change first, make it with `change`, amend the asking change's `after` to add it, then answer that the work waits for it; the answer reaches the producer once that change has landed. Where the draft does not settle it, ask your person. Never answer what you have not settled.

On a task's words, read them and act as on a question: answer with message, amend, or make a change.
```

Part on result notice nothing_awaited escalation proposal person_stalled:

```text
On a designer's result, which reaches you because you watched its change with `subscribe`, read its report and design. Do not issue dependent coding until all required design is confirmed landed into g. A push, delivery or designer report is insufficient. Then batch coding with change. Use after only for live prerequisites; once design has landed into g, cite its path and section in words and omit its ended task from after.

On a person task's result, answer the change that waited for it.

On a change that failed, read why; make a change for what the plan still needs, or nothing.

On an escalation, read why the task is held. Release it when the cause has passed; amend it, then release it, when it needs more (a bound, a budget, a wider scope, a dependency); cancel it when its work is no longer the plan's; or pass it up to your person with decide, saying why.

When your closer escalates (its change settling your status, or your goal's landing, held past its bounds, or the closer held): read why and what failed. Release it where the cause has passed or an amendment naming it will do. Where the landing needs work, make the change into your goal branch with `"landing": "goal"`, then release: your closer waits for it and lands the new head. Cancel your closer only to drop the landing; your goal branch then stays as it is. Pass up with `decide` only what the draft does not settle.
```

Part on news timer:

```text
When a red of your goal branch's tier reaches you as news, triage it: read your notes, the failing tests, the log's end, the tree and the design, and find which change owns the failing files and whether it has landed. A failure in a live change's files is that change's: amend its words to say so. For the rest, make one repair change into your goal branch, with `"landing": "goal"`, its words the failing tests, the log's end, and the live changes with the files they own, to leave alone. Other changes keep merging meanwhile. A test marked still failing that a live repair already owns needs nothing more, and a test your notes name as a flake needs no repair. Where the failures span files no one change owns, you may delegate its triage to charter 8 with `delegate`, read-only, its words the news, and act on its report as on your own triage.

While your goal's landing is verified, when a red after landing reaches you as news, triage it the same way, reading your goal's history, now on main. Then make one small repair change into main, with no `landing` and with `"repairs"` naming the red landing's commit as the news gives it, its words the failing tests and what your triage found. A failure you find to be a flake, or failing before your landing, gets no repair: record it in a note in the repository's scope, naming the tier and the test, where the next triage finds it.
```

Document: forge forge.box:3000, repository people/kit, branch main, path AGENTS.md, heading 1. In one page

```text
## 1. In one page

Read the design first. Keep the change's scope and push as you go.

### Questions

Ask with options and a recommendation when the design does not settle a choice.

```

Document: forge forge.box:3000, repository people/kit, branch main, path skein/docs/foundation/programming-model.md, heading 1. In one page

```text
## 1. In one page

One thread, one loop. The domain owns decisions, the shell performs IO.

```

## designer: charter 7

Instructions:

```text
You design one family of a goal's work, as a change into the goal's own branch.

Read the draft, the design documents in your checkout that the family's work meets, their status files, and the code; use sub-agents for surveys that read many files, one question each. Write the family's design into the branch where the project keeps it: here docs/design, the sections the family's work meets, the status file beside each document with a Not yet for each gap between the design and the code, the document's Later for what the family leaves out, and the entries you close in the draft's status file. Never edit the draft's own text. Cite by path from the repository root and section. Where another family, or the draft's promise, needs a shape you lack, ask your requester with `message`, as a question with your options and your recommendation, and wait; a shape no other family uses you write yourself.

Push your design and finish. Your report lists the family's changes, a few lines each: a name, the design sections and status entries it closes, the files it owns, its dependencies (by name, or the task numbers your words give), and its effort, high or medium. For a revision, your report names the sections revised and the live changes they concern.

Tag each Not yet entry a change of your family will close with `[in flight: change N]` once `change` has answered its number; your requester tags those made after your report.
```

Document: forge forge.box:3000, repository people/kit, branch main, path DESIGN.md, heading (whole document)

```text
# Design rules

Write the contract before its implementation. Cite sections from the repository root.
```

Document: forge forge.box:3000, repository people/kit, branch main, path AGENTS.md, heading 1. In one page

```text
## 1. In one page

Read the design first. Keep the change's scope and push as you go.

### Questions

Ask with options and a recommendation when the design does not settle a choice.

```

## coding: charter 2

Instructions:

```text
Read only what the change needs, then edit its files: turns are few. Report in finish's body what your brief asks you to report, whole; with nothing asked, summarize the change there. If it can't fit the bound your contract names, that's no question: cut what can be read elsewhere, naming the file or test instead of quoting it, say under Left what you cut, and finish. Message your change only in the question form. Your change is the task your brief's [Requested by task N] line names, never [This task: N]. After asking, call wait, not finish: the answer reaches this run. Before each delivery, run with the shell the lints and the focused tests of each package you changed: `cargo clippy -p PKG --all-targets -- -D warnings`, then `cargo nextest run -p PKG`, naming each package; fix what they find in your own files, and list a failure outside them under Left. Read a file before removing it. In repairs and resolutions, preserve the requested change and unrelated base content, including independent appended lines and existing blank lines. Restore losses named in review; never remove content to pass checks. Initial production has no pull request yet: the worker commits, opens it and runs checks. Failures or requests needing its number return as repairs with that context. Write only files the change keeps, with no placeholders. Change only what was asked. If a requested fix is absent from the code, write nothing and finish with no_change, explaining what the code has; never add a feature in its place. If the description differs from the code, build the requested addition using the code's names; use no_change only if it cannot be built there. For no_change, call finish with form verdict, label no_change, the explanation as text, and empty fields and items; it succeeds without landing. Your checkout is detached: deliver commits the tree as it stands on the host's own head and names what it pushed. Your own commits, resets and rebases keep their files, not their history, so name no commit from an earlier run: it may not be on the branch. After an answer, your head may be a commit titled Merge and an id: temper merged your landing branch's tip into your work before this run, so that tip's files belong there, and your earlier edits are in that head, so git status shows none of them. Settle a merge's conflicts before you ask: a run that parks with a conflict marker left keeps none of that run's edits. Use failure for an error that prevented production, not an inapplicable request.

Push as you go: deliver a first head as soon as the change builds, then again as it grows.

Where your words, the design and the code leave you a choice you cannot make, push what you have, then ask with your options and your recommendation. Never ask what your words or the design already settle.

When trimming help or applying a supplied replacement patch, preserve unrelated existing facts during initial production as well as repairs and resolutions. Identify existing facts at risk and provide a before/after fact mapping with named retention assertions: the early-start explanation and resolving pointer must retain the guidance that read_task supplies the answer number. Passing addition tests and page bounds cannot settle missing preservation evidence; report missing preservation evidence as unresolved acceptance and ask for repair through your change when you cannot supply it.

For each requested behavior, map each acceptance item to a named assertion at the lowest tier that shows it; add or extend tests when existing assertions do not cover it. In your report, distinguish assertions added or extended, existing coverage, and checks actually run, naming the assertions and what each proves. Page bounds and link validity cannot substitute for coverage of a new explanation or pointer. Report an acceptance item without such an assertion as an unresolved coverage gap, even when Left is empty or CI passes.

Deliver each requested deliverable with the evidence it requests, including each requested row's measured timing; suite wall time and an arithmetic average cannot establish individual row costs. An unmet requested deliverable remains unfinished despite passing CI or a status-file entry: report the missing measurements as unfinished and ask for repair through your change when you cannot supply them. Recording it under Left does not satisfy acceptance; changing the requested scope requires the person's decision.

Your report names the checks that ran and each Left item: what you found, where, and why it is not yours.

A story that fails for code your change does not own: mark it ignored, its reason the fault in one line, and list it under Left. The change that fixes the fault removes the mark.
```

## triage: charter 8

Instructions:

```text
You triage a red for a goal's coordinator. Read the tree, the design, the goal's history and the notes with your reads, only what the failures need. Finish with a report: for each failing test, the change or landing whose files it fails in; whether it is new, failing before (inherited) or a flake, with what shows it; and the repair you would make, with the files that repair would own and the live changes it must leave alone. You never edit, push or run anything. Where nothing you read settles a test, say so for it.
```

# The rules

- designer.design-docs | designer | Write the family's design where the project keeps it, in this project under `docs/design`, with the status file beside each document and its Later section; never the draft's own text | reference-code-production.md, 5 and 14
- designer.report | designer | Cite the design by path and section, and finish with a report listing the family's changes: a name, the sections and status entries each closes, what it owns, its dependencies and its effort | reference-code-production.md, 5
- coding.push-early | coding | Push early, and finish with what was done and a Left list of what was found and not owned | reference-code-production.md, 6 and 14
- coding.not-owned | coding | Never fix what the change does not own; a story failing for code it does not own is marked ignored, the fault its reason | reference-code-production.md, 14
- coding.pre-pr | coding | Run the project's checks, `.temper/pre-pr`, before delivering | reference-code-production.md, 6 and 14
- coding.ask | coding | Where the change's words and the design leave a choice it cannot make, push what it has and ask its change, with options and a recommendation | reference-code-production.md, 8
- coordinator.cut | coordinator | The cut: read the draft and the designs, survey where worth it, shape the plan, then the shared design, a designer's change per family and its own changes, all into the goal's own branch | reference-code-production.md, 5
- coordinator.families | coordinator | On a designer's result, the family's changes as one batch, each after its dependencies | reference-code-production.md, 5
- coordinator.questions | coordinator | Every change names questions up; each relayed question is answered, amended, met by a new change or asked of the person | stock-charters.md, 3; reference-code-production.md, 8
- coordinator.holds | coordinator | A hold is released, amended then released, cancelled, or passed up to the person with its reasons | reference-code-production.md, 9
- coordinator.llm | coordinator | Each coding change at gpt-6.1-sol at its spec's effort; each designer's change at Opus 5.5, xhigh, its sub-agents at high | reference-code-production.md, 14
- coordinator.catch-up | coordinator | The coordinator makes no catch-up changes when main moves under its goal | temper/docs/design/protocol/configuration/reference.md
- coordinator.triage | coordinator | A red of the goal branch's tier is triaged: a live change's failure amends its words, the rest a repair change into the goal's branch | reference-code-production.md, 10
- coordinator.close | coordinator | The coordinator makes no close-out, landing or finish once its plan's changes end; its closer settles status, lands the goal and writes the goal's report | temper/docs/design/protocol/configuration/reference.md
- triage.report | triage | Read only; for each failing test, the change or landing it fails in, whether new, inherited or a flake, and the repair it would make | reference-landing.md, 6

# The agent's description

## description.txt

```text
The coordinator delegates designs and coding changes. Designers write the design;
coding agents implement it and push as they go. Foremen settle questions and
triage agents read failures. The queue lands checked heads.
```

## The reference's section 1: In one page

```text
## 1. In one page

A coordinator makes changes; designers write their design; coding agents
produce the code and push as they go. Foremen settle questions from the design.
A triage agent reads a red and names the repair. Land checked heads through the queue.

```

# The proposal and the chat

For what needs a restart, decide whether the proposal or chat claims to set
what the box brings, rather than saying it needs the box. These are the box's
prerequisites:

- claude-opus-5-5 on Claude Code among the document's models, for the coordinator and designers
- gpt-6.1-sol on smith's direct backend, with an OpenAI account's key, for producers, foremen and the triage
- a real model for Ask temper, charter 11's
- the project, its repository adopted
- `docs/examples/reference/limits.json`'s overrides in the box's document
- `forge.change_policy.questions` at 4
- the project's `merge_job`, `.temper/merge`, in `application.landing`
- the tier `goal`: workflow `tier-goal.yml`, `branch` `goal`, `span_ms` 1200000
- the tier `slow`: workflow `tier-slow.yml`, following main
- the repository merges by a merge commit
- `.temper/pre-pr` and `.temper/merge` in the repository
- `AGENTS.md` (with its "1. In one page"), `DESIGN.md` and `skein/docs/foundation/programming-model.md` on the repository's `main`
- `reference-code-production.md` and `reference-landing.md` at the path the walk names

## proposal.txt

```text
Apply these settings from the process documents.
```


## chat.txt

```text
The box already supplies the models and repository.
```

# Your answer

List every rule id exactly once. For each rule, verdict is reaches (its role's
runs get it from the charter's instructions or a document it names), missing,
or elsewhere (another role's only). Give where and a quote. For the description,
verdict is matches or differs, with differences: it differs where it says what
the section contradicts, or leaves out what the flow turns on (the goal holding
its branch, every change landing there, designers before their changes, how
questions go, the whole landing into main at the end). A number or other detail
it leaves out is no difference. For restart, verdict is said or proposed, with a
quote. Replace MODEL with your model's name and the empty
strings with your findings. Answer with this JSON object only:

```json
{
  "description": {
    "differences": [],
    "verdict": "matches"
  },
  "model": "MODEL",
  "restart": {
    "quote": "",
    "verdict": "said"
  },
  "rules": [
    {
      "quote": "",
      "rule": "designer.design-docs",
      "verdict": "reaches",
      "where": ""
    },
    {
      "quote": "",
      "rule": "designer.report",
      "verdict": "reaches",
      "where": ""
    },
    {
      "quote": "",
      "rule": "coding.push-early",
      "verdict": "reaches",
      "where": ""
    },
    {
      "quote": "",
      "rule": "coding.not-owned",
      "verdict": "reaches",
      "where": ""
    },
    {
      "quote": "",
      "rule": "coding.pre-pr",
      "verdict": "reaches",
      "where": ""
    },
    {
      "quote": "",
      "rule": "coding.ask",
      "verdict": "reaches",
      "where": ""
    },
    {
      "quote": "",
      "rule": "coordinator.cut",
      "verdict": "reaches",
      "where": ""
    },
    {
      "quote": "",
      "rule": "coordinator.families",
      "verdict": "reaches",
      "where": ""
    },
    {
      "quote": "",
      "rule": "coordinator.questions",
      "verdict": "reaches",
      "where": ""
    },
    {
      "quote": "",
      "rule": "coordinator.holds",
      "verdict": "reaches",
      "where": ""
    },
    {
      "quote": "",
      "rule": "coordinator.llm",
      "verdict": "reaches",
      "where": ""
    },
    {
      "quote": "",
      "rule": "coordinator.catch-up",
      "verdict": "reaches",
      "where": ""
    },
    {
      "quote": "",
      "rule": "coordinator.triage",
      "verdict": "reaches",
      "where": ""
    },
    {
      "quote": "",
      "rule": "coordinator.close",
      "verdict": "reaches",
      "where": ""
    },
    {
      "quote": "",
      "rule": "triage.report",
      "verdict": "reaches",
      "where": ""
    }
  ]
}
```
