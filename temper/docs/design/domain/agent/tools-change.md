# A change in the run's words

This document holds the run's change tool, split from [`tools.md`](tools.md).

## 1. A change, in the run's words

The stock chat first reads the code a repository change touches with the
forge's `read_forge what=branch` for its head, then `what=file`. It calls `change`
with the person's request and only what it read, never describing code
it has not read, its words opening with a title line written for a
person (`charter.md`, 4, *Words a person reads first*). Repository changes go through `change`, never through
writes to the chat's workspace. After delegation the chat says only that
work started until the delegate reports. When a change reports successful
`NoChange`, the chat tells the person that nothing changed and explains why
using the delegate's conclusion, without claiming landing, implementation
or tests. For a fix, if the named fault is absent, it changes nothing and
says what the code has. The stock coder likewise writes
nothing and finishes with the successful `no_change` verdict saying what the code has when the
named fault is absent, never adding a feature in a fix's place. Where
the change's words describe the code otherwise than it is, the coder
builds what they ask to add in the code there is, under its names,
finishing with `no_change` only when it cannot be built there.
An error that prevents production is a failure and keeps its recovery hold. When a change
lands, the chat names the pull request and merge commit from its result's
`Landed` line, and describes what changed only as the result or pull request says,
never by restating the request as done.

A chat cannot know what a change's `delegate` needs: the procedure's
numbers, the repository's resource, its landing branch, the contract's
kind and the grants a change makes. `change` takes the words alone, and
the root builds the one delegate from what it knows, then decides it as
any `delegate` (jig's authority, named and decided once):

- **the repository:** among the project's adopted repositories it may
  write, on forges not ended, the one named, or with no name the only
  one; with several and no name, or a name none has, refused, naming
  them. Never the home by default: a project's home may be another
  forge's (the demo's);
- **the delegate:** the change procedure (../forge/change/README.md, 1) with the words,
  the repository's resource and its landing branch: the one the call
  names (below), or with none the applicable held goal branch, otherwise
  the repository's landing branch, the one its adoption names
  (`../forge/adoption.md`, 8), which is its default branch only where
  the adoption named that; a
  change's contract, its words bounded by the deployment's
  `tasks.result_bytes`;
- **its authority, the run's own narrowed:** the run's grants of the
  forge's reads (kind 1, ../forge/README.md, section 2) and of the kinds a change
  makes (../forge/effects.md, section 1: a push, open, merge, edit, update, close
  or reopen a pull request), on the change's repository, which every
  agent of the change holds, so that its reviewer and its repairs read
  the pull request, its files and CI's logs; the change's charters, its
  producer's and its reviews' (../forge/change/gates.md, 3), as the only kinds it may
  delegate, the merge kind, job 3, where the run's authority delegates
  to it (../forge/change/repairs.md), and a person's role where a landing rule
  covering the branch names a person gate; at most
  `forge.change_policy.tasks` tasks (`../../protocol/configuration/limits.md`
  : 6 in `small`, a producer, a review, and two repairs each
  followed by its review; 80 in `standard`), one fewer than the run has left, and a depth of at most 2; no notes; half of
  what the run's budget has left, so that it can still read the result
  and answer, refused when that is no more than a run's minimum. That
  half is a default, open to tuning. Several changes in one call share
  that half equally.

A `delegate` refused for an executor or a grant says, too, that a
repository change is `change`'s.

**A person's report has a floor.** jig refuses a report contract bound at
0 words addressed to a person or a role (jig's `domain/people/README.md`,
7), and the run learns it before and after the call:

- **The schema says it:** `delegate`'s `contract` description ends
  `A person's or a role's report: words at least 1.`; its `words` keeps
  `minimum` 0, which an agent's report and a verdict's choices still
  take.
- **The refusal names the bound:** a batch refused as `Refusal::Contract`
  reads `task <N> result contract (words at most the result bound; a
  person's or a role's report at least 1)`, whatever the tool that made
  the batch, so that a run can mend the member it names.

**What a plan decides.** Beside the words and the repository, a call
names what a plan decides of each change, and the root still builds the
delegate from what it knows:

```
{"words": "...", "repository": "owner/name",
 "name": "parser", "landing": "goal", "after": [41, "lexer"]}
{"changes": [{"name": "lexer", "words": "...", "landing": "goal"},
             {"name": "parser", "words": "...", "landing": "goal", "after": ["lexer"]}]}
{"words": "...", "branch": "mine"}
```

- **`landing`:** `"goal"`, the nearest goal branch held by the requester
  or one of its ancestors in the change's repository (../forge/branches.md, 1);
  or a branch's name in the repository. Left out, that held goal branch,
  otherwise the repository's landing branch. Ordinary child changes,
  including calls from the goal's subtree, cannot name a destination
  outside that held branch: the whole call is refused before any delegate
  is made. `"branch": "mine"` deliberately lands the requester's held goal
  branch whole, defaulting to the repository's landing branch. `"goal"`
  with no applicable held goal branch is refused, saying so; a
  name that is not a branch's (an empty segment) is refused, naming it;
  a branch the repository lacks holds the change at its opening, the
  forge refusing its pull request (../forge/change/README.md, 2).
- **`branch`:** `"mine"`, its requester's held branch: the goal branch
  the run's task holds in the change's repository (../forge/branches.md, 1),
  which the change lands whole as it stands, with no producer (../forge/change/README.md,
  1); left out, the change's own branch, which its producer pushes.
  `"mine"` from a run whose task holds no goal branch in that repository
  is refused, saying so; with `"landing": "goal"`, a branch into
  itself, refused, saying so; while another live change of the task is
  on that branch, refused, naming its task. Any other value refused,
  naming it, `"run"` among them: a goal's runs deliver their design
  straight to its goal branch (`tools.md`; ../forge/branches.md, 3), so no change lands
  a run branch. The schema offers `"enum": ["mine"]`, described `mine:
  goal into base; omit: a new.`
- **`name`:** a name for the change within the call, at most 64 bytes,
  unique in it, which another change of the same call names in its
  `after`; it names nothing once the call is decided.
- **`after`:** the change's dependencies (jig's `domain/tasks/README.md`,
  section 4): a task number, of one of the run's task's live delegates,
  a task it references, or any task of its project that the party who
  requested its tree can read, another goal's among them, which jig's
  admission checks as the edge is made (`../../draft/bounded-waits.md`,
  6.3); or the `name` of an earlier change in the same call. So a change
  waits on another goal's task by its `after` alone, with no question
  held open to keep its run alive. The schema describes it `Task
  IDs/earlier names.` The change waits on
  them as any task does, so it starts once each has ended done; a name
  the call does not give before it, or a task jig refuses, refuses the
  call, naming it. Its producer, and every other task it delegates to, may
  read each of them with `read_task`, live or retained, read only (jig's
  `domain/engine/executors-tools.md`, 1): a dependency is no reference, and
  carries no message.
- **`merge`:** a catch-up (../forge/updates.md, section 1): the commit to take, as
  40 or 64 hexadecimal digits, which the merge job merges into a branch
  made at the landing branch's tip, as the change's producer in place of
  an agent; the words say what the commit brings and why. Refused,
  naming the setting, where the change's project names no `merge_job`
  (`../../protocol/configuration/engine.md`), and naming the kind,
  where the run's authority does not delegate to the merge kind; a
  commit in another form is refused, naming it. Whether the repository
  has the commit is the job's to find: one the forge cannot fetch
  fails the job, and holds the change.
- **`changes`:** several changes in one call, each an object of the
  members above but `changes`, made as one batch (jig's
  `domain/tasks/README.md`, section 4): all of them or none, the call's answer
  naming each change's task in the call's order. A call names either
  `words` or `changes`, never both; `changes` holds at least one.
  A batch's change takes every member from its own object alone, the
  members below `changes` and those to come too, and nothing from the
  call's: a call with `changes` names no other member. One that does is
  refused as it is decoded, before any change is made, naming the first
  such member in the declaration's order (`repository`, `name`,
  `landing`, `branch`, `merge`, `after`, `producer`, `llm`, `repairs`,
  `questions`) and saying it belongs in each change: "host call member
  belongs in each change, not beside changes: landing". A landing, a
  producer, a route for questions or an LLM choice given once for a
  whole batch is never dropped unheard, and the run gives it again in
  each change's object. A member's own value is the change's, as in the
  single form; `words` beside `changes` is refused naming `changes`, as
  above.
- **`producer`:** the number of the charter the change's producing
  task, its repairs, its resolutions and its checks' gates run under
  (../forge/change/README.md, 1); left out, the forge's change charter, kind 3's number
  1 (`../../protocol/configuration/settings-kinds.md`, 1). A charter the newest
  settings do not offer (jig's `domain/engine/settings.md`, 3), or one the
  run's authority may not delegate to (jig's `domain/authority.md`,
  section 3), refuses the call, naming it, as a `delegate` naming it
  would be refused.
- **`llm`:** an LLM choice for each of the change's agents' roles, each
  in jig's form, its models and efforts by their configured names and no
  account (jig's `protocol/configuration-templates.md`, 1):

  ```
  {"words": "...", "landing": "goal", "producer": 6,
   "llm": {"producer": {"main": {"model": "gpt-6.1-sol", "effort": "high"}},
           "review": {"main": {"model": "gpt-6.1-sol", "effort": "medium"}}}}
  ```

  `producer` for its producing task, its repairs, its resolutions and
  its checks' gates; `review` for its agents' gates. The root checks
  each as jig checks a batch's (jig's `domain/tasks/README.md`, section 4),
  against the charter its role runs under at the newest revision and
  the run's authority: a model not configured or named twice, an effort
  its model does not declare, a model, main or sub-agent, outside that
  charter's models or the authority's refuses the call, naming the
  model and why. A role left out names none, and each of its agents takes
  its models at each run's check (jig's `domain/tasks/README.md`, section 3).
  A role's choice carries no `later` (`../../draft/per-wake.md`, section
  5): a change's producers resume or finish in one activation, so each
  of its agents takes the one choice at every run, and the root makes
  each with no `later`.
- **`repairs`:** the red landing the change repairs
  (`../after-landing.md`, section 7), by its merge commit's full
  hexadecimal digits, as the red's news names it: a landing of temper's
  in the change's repository, on the branch the change lands into, red
  in some tier as the root decides the delegate. A value that is not a
  commit's digits is refused, naming it; a commit no live landing of
  temper's there has, or one red in no tier, refused, naming the commit
  and why. The change keeps it, and its own landing clears the red one
  when a later tier run passes the failures blamed on it
  (`../after-landing.md`, 5.3); left out, the change repairs nothing.
- **Who may set them:** the members past `words` and `repository` are a
  plan's, and only a run whose task's authority holds jig's control
  family (amend and cancel, with `release`: jig's `domain/authority.md`,
  section 3) may set them: a plan's maker amends, cancels and releases
  what it makes. The stock policy's families give it
  (`../../protocol/configuration/stock.md`), so a chat or a goal under the
  stock sets them. `change`'s declaration offers them only where the run's
  authority holds that family (`tools.md`, the host part's budget), and the root
  refuses a call naming one from a run without it, beyond authority,
  naming the family, as it decides the delegate. What each member sets is
  checked as the rest of the batch is: its landing branch by the change's
  grants on the repository, its dependencies by jig's batch.
- **`questions`:** how the change takes its agents' questions (../forge/change/questions.md,
  1): `"none"`, a change that asks no one (`agents.md`, 2); `"up"`, each
  passed straight to the requester; left out, the
  project's setting (`../../protocol/configuration/settings-kinds.md`, 1, kind 6), whose
  default is `"none"`. Any other value is refused, naming it, `"foreman"`
  among them, alone and in a batch's member alike: that route is retired
  (`../../draft/stock-charters.md`, 3), and the schema offers `"none"` and
  `"up"` alone.
- **Members to come** join each change's object, each optional and
  checked by the root as it decides that change's delegate, leaving
  these as they are.

## 2. Open questions

- **Who may name a plan's members of `change`:** the control family
  says it (`tools-change.md`, 1), since a plan's maker amends, cancels and releases what
  it makes. A family of its own would let a policy grant control without
  plans, or plans without control.

- **A change produced by a job of any kind.** `producer` names a
  charter, and `merge` the merge job, a catch-up's producer (`tools-change.md`, 1):
  every producer a reference change needs
  (`../../draft/reference-code-production.md`, section 3). Default: no
  other job kind produces a change; a project whose tooling makes
  changes with no model would name its kind as a member of `change`.

- **A real chat choosing `change`** (`tools.md`): the owned-forge
  landing story requires the chat's own successful `change`, rather than
  a goal's change. Observe whether `goal`'s declaration steers a plain
  repository change there, and whether the branch-read guidance gives a
  real run its next read. Default: declarations and bounded refusal
  guidance, without a new admission restriction; the existing live story
  runs after landing (`../../testing/live-tier.md`). A scripted call establishes
  neither model choice nor recovery.
