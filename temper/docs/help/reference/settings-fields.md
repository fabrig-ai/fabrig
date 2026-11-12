# Settings fields

Kind: reference

Every member of a settings value: its type, its bound and what it does to work, as a settings file sets it and `settings show` prints it.

Each section is a JSON object shape; bullets name members, types, bounds and effects on work. A settings file is `file`. Read `settings show KIND N`, edit and set it whole. Ctl sends these bounds; the deployment may refuse it, naming its fault: `shape`, `count`, `reference`, `rules`, `budget`, `grace` or `owner`. Spend uses deployment model-price units. Hex is lower-case hexadecimal bytes: `666f726765` is `forge`.

## file

A settings file, as `settings apply` and `settings propose` read it.

- `items`: list of at most 1,024, each `item`. One item per changed setting, each setting once.

## item

A `setting` and exactly one of `set`, `edit` and `overlay`.

- `setting`: `setting`. The setting the item changes.
- `overlay`: `overlay`. A charter overlay, instead of set or edit.
- `set`: the setting's kind's value: `charter`, `template`, `policy`, `rules`, `model`, `landing-rule`, `forge-charter`, `template-goal`, `project-policy`, `project-ways`, `project-questions`, `project-classes`, `charter-words`, `project-words`, `follow` or `dependency`. Whole replacement value.
- `edit`: `"removed"` or `"reset"`. An edit instead of a value. `"removed"`: No longer offered, its default included; the rules cannot be removed. `"reset"`: Erase its change; follow its default again.

## setting

Which setting: its kind and number, or project for project settings.

- `kind`: `"model"`, `"charter"`, `"template"`, `"policy"`, `"project-policy"`, `"rules"`, `"landing-rule"`, `"forge-charter"`, `"template-goal"`, `"follow"`, `"dependency"`, `"project-ways"`, `"project-questions"`, `"project-classes"`, `"charter-words"`, `"project-words"`, `"project-areas"` or `"project-triager"`. The setting's kind. `"project-policy"`: Project policy: `set` only, once per file alongside other settings.
- `number`: integer, 0 to 4,294,967,295. Setting number, from 1; omit for rules and unnumbered project settings.
- `project`: integer, 0 to 4,294,967,295. Required for project settings, follow and dependency; absent for charter-words. Landing rule: from 1, or deployment-wide if absent.

## charter

Run policy from the task's creation revision.

- `rest`: true or false. Offer remaining catalogue models after named choices; default false.
- `number`: integer, 0 to 4,294,967,295. Same as `setting.number`.
- `instructions`: text, at most 1,048,576 bytes. Agent's first read, before the task's brief.
- `waiting`: nanoseconds. Idle run's wait for words or news before parking.
- `resume`: true or false. Resume the task's transcript if any, rather than start afresh.
- `turns`: integer, 0 to 4,294,967,295. Most model turns; more than `reserve_turns`, else `budget` refusal.
- `time`: nanoseconds. Run cancellation time; more than `reserve_time`, else `budget` refusal.
- `reserve_turns`: integer, 0 to 4,294,967,295. Of `turns`, reserve to finish and report.
- `reserve_time`: nanoseconds. Of `time`, reserve to finish and report. With `modify`, at least `check_timeout` milliseconds in every applicable current or held charter; otherwise worker admission refuses. See Settings help for held-revision recovery.
- `model`: `model`. Default model: first payable choice among `model` and `alternatives`.
- `alternatives`: list of at most 32, each `model`. Models after `model`, each once; also available to sub-agents.
- `inspect`: true or false. May the agent read, list and search its workspace?
- `modify`: true or false. May the agent write, edit and remove workspace files?
- `shell`: true or false. May the agent run workspace commands?
- `agents`: true or false. May the agent open sub-agents with no more tools than its own?
- `call_timeout`: nanoseconds. Longest call to temper's tools.
- `tools`: integer. Host families need charter and task leave. Asking (2048): see and document; with settings (1024), propose_settings too. Without asking, 1024 grants settings tools. Default 1023 excludes both.
- `hosts`: `"workers"`, `"engine"` or `"both"`. Which hosts may run it. `"workers"`: The deployment's workers alone. `"engine"`: The engine's own slots alone. `"both"`: Either.
- `spend`: integer. >= (`reserve_turns` + 1) costliest default-model completions, else `budget` refusal. Runs get remaining task funds, capped by `rules.maximum_run_spend`.
- `wake`: `wake`, or null. Wake policy for tasks made under this charter whose batch names none. Null/absent: jig's default, all rules `"immediate"`, `results` `"last_or_failure"`, questions, answers and timers waking.
- `documents`: list of at most 8, each `document`. Brief documents; at most `tasks.charter_documents`, else `shape` refusal.
- `parts`: list of at most 8, each `charter-part`. Ordered parts chosen by run reasons, at most tasks.charter_parts; default [].

## overlay

Optional default replacements and added text. Use charters or literal instructions/parts, never both.

As `charter`'s fields, replacing the default whole: `rest`, `waiting`, `resume`, `turns`, `time`, `reserve_turns`, `reserve_time`, `model`, `alternatives`, `inspect`, `modify`, `shell`, `agents`, `call_timeout`, `tools`, `hosts`, `spend`, `documents`.

- `instructions`: text, at most 1,048,576 bytes. Optional added text, empty when absent; not beside charters.
- `parts`: list of at most 8, each `charter-part`. Optional added text, empty when absent; not beside charters.
- `charters`: list of at most 4,294,967,295, each text, at most 4,294,967,295 bytes. Optional nonempty directories, relative to settings file; instead of instructions/parts, only charters 5 and 2. Markdown counts toward file bound.

## charter-part

Instructions after shared text, read when run reasons meet `on`.

- `on`: list of at most 14, each `"first"`, `"question"`, `"question_answered"`, `"question_ended"`, `"question_owed"`, `"escalation"`, `"proposal"`, `"person_stalled"`, `"result"`, `"notice"`, `"nothing_awaited"`, `"news"`, `"timer"` or `"words"`. Nonempty unique names; refuses those offered to every run.
- `instructions`: text, at most 1,048,576 bytes. Text appended after the shared instructions when this part is chosen.

## document

Brief file at preparation branch head.

- `forge`: text, at most 32,768 bytes. Copy the repository resource's `host`, including port, from `see settings` under `project repositories`, or `temper ctl DIR forge list`; neither forge number nor web address.
- `repository`: text, at most 65,537 bytes. OWNER/NAME; if no project adopts it, the section says so.
- `branch`: text, at most 131,075 bytes. Branch head to read; segments joined by `/`.
- `path`: text, at most 256 bytes. The file's path in the repository, at most 256 bytes.
- `heading`: text, at most 80 bytes, or null. Markdown heading: read through the next peer or ancestor heading; null reads the whole file.

## catalogue

Numbered runtime model and its display facts.

- `number`: integer, 0 to 4,294,967,295. Permanent number, 1 to 64, matching setting.number.
- `model`: `model`. The runtime model, whole.
- `shown`: text, at most 4,096 bytes, or null. Its name for showing, or null.
- `description`: text, at most 512 bytes, or null. Its description, or null.
- `picker`: true or false. Whether a model picker shows it.
- `withdrawn`: true or false. Whether it is withdrawn; its number and name stay.
- `source`: `"stock"`, `"list"` or `"person"`. Where the model came from. `"stock"`: The binary. `"list"`: A provider list. `"person"`: A person.
- `estimated`: true or false. Whether its prices are provider defaults.

## model

Runtime model. Except `effort`, charter members must match its configured row or refuse `reference`. Model settings supply it whole.

- `provider`: integer, 0 to 4,294,967,295. The configured LLM provider that serves it; 1 when absent.
- `dialect`: integer, 0 to 4,294,967,295. Configured model protocol.
- `endpoint`: integer, 0 to 4,294,967,295. Configured serving endpoint's number.
- `program`: text, at most 32 bytes, or null. Configured program running it, or null for none.
- `name`: text, at most 4,096 bytes. Provider's model name.
- `window`: integer, 0 to 4,294,967,295. The most tokens of input it reads.
- `output`: integer, 0 to 4,294,967,295. Most tokens per completion, thinking included.
- `input_price`: integer. Fresh input cost: spend units per `price_unit` tokens.
- `cached_price`: integer. Cached input cost: spend units per `price_unit` tokens.
- `cache_write_price`: integer, or null. Cache write cost per `price_unit` tokens; null uses fresh-input price.
- `output_price`: integer. Output cost: spend units per `price_unit` tokens.
- `price_unit`: integer, 0 to 4,294,967,295. The number of tokens each price is for.
- `efforts`: list of at most 16, each text, at most 32 bytes. Configured efforts, in order.
- `effort`: text, at most 32 bytes, or null. One of `efforts`; null: default effort.

## wake

Wake by message kind; proposals, escalations and cancels always wake.

- `words`: `wake_rule`. Task and deployment words; a person's always wake it at once.
- `notices`: `wake_rule`. Notices: referenced task done/held, proposal passing through, awaited hold.
- `news`: `wake_rule`. News its connectors class as waking for it.
- `results`: `"never"`, `"each"` or `"last_or_failure"`. Its delegates' results. `"never"`: None wakes it. `"each"`: Each delegate's result wakes it. `"last_or_failure"`: The last delegate's result, or any failure.
- `questions`: true or false. Whether a delegate's question wakes it.
- `answers`: true or false. Whether an answer to its own question wakes it.
- `timers`: true or false. Whether its timers wake it.

## wake_rule

Wake by message kind: a word or one-member object.

- `"never"`: It waits for the task's next run.
- `"immediate"`: It wakes the task at once.
- `batch`: `wake_batch`. Once enough arrive or the oldest waits long enough.
- `paced`: nanoseconds. At most once per span from the last run's start; first after a quiet span wakes at once. Above zero, else `shape` refusal.

## wake_batch

Message batch waking a task once.

- `count`: integer, 0 to 4,294,967,295. Wake at this count; above zero, else `shape` refusal.
- `age`: nanoseconds. Or at this age of the oldest; above zero, else `shape` refusal.

## template

Executor and defaults for new tasks.

- `resources`: list of at most 32, each `symbol`. Optional, default []: distinct resource-symbol pairs resolved for the new task. Forge connector 1, symbol 1: its own goal branch.
- `number`: integer, 0 to 4,294,967,295. Template number matching `setting.number`; chat: 1, goal: 2.
- `name`: `text`. Its name, as clients show it.
- `executor`: `executor`. Who runs the task.
- `contract`: `contract`. The result the task must give.
- `shallower`: integer, 0 to 4,294,967,295. Delegation depth below creator role; 0 makes the template unusable.
- `budget`: `budget`. What the task may spend.
- `deadline`: `deadline`. By when the task must end.
- `tracked`: integer, 0 to 4,294,967,295, or null. Its default priority as tracked work, lower first, or null for untracked.
- `flags`: `flags`. How the task behaves and is kept.
- `label`: integer, 0 to 4,294,967,295. Number grouping ended tasks in lists; temper reads nothing else from it.
- `grace`: nanoseconds, or null. Chat closing time or null; unfinished runs then cancel. Past deployment limit: `grace` refusal.
- `llm`: `llm`, or null. The models the task asks for, or null for its charter's default.

## symbol

Resource resolved at creation.

- `connector`: integer, 0 to 65,535. The connector that resolves the symbol; 1 is the forge.
- `symbol`: integer, 0 to 65,535. The connector's resource symbol; the forge's 1 is its own goal branch.

## text

A text, whole.

- `bytes`: text, at most 65,536 bytes. The text.
- `cut`: true or false. Always false: a value is set whole.

## executor

Task executor.

- `"chosen"`: The creator chooses a charter or procedure their role permits.
- `charter`: integer, 0 to 4,294,967,295. Agent under this charter; must be offered, else `reference` refusal.

## contract

One required task result.

- `approval`: `approval`. Person's approval of exact state; agents can only fail.
- `report`: `report`. A report.
- `verdict`: `verdict`. One of a set of verdicts.
- `change`: `change`. A connector's result.

## approval

An approval of an exact state.

- `state`: hex, at most 32,768 bytes. Approved state; approval must return it byte for byte.
- `part`: `part`. What a client shows the approval with.

## part

A thing a connector owns, which a client shows.

- `connector`: integer, 0 to 65,535. The connector that owns it: 1 is the forge.
- `kind`: `"resource"`, `"pool"`, `"result"`, `"procedure"`, `"effect"`, `"outbox"` or `"hold"`. What kind of thing it is.
- `key`: `key`. Which one.

## key

Which thing: a one-member object.

- `path`: list of at most 9, each hex, at most 32,768 bytes. Hex resource segments: `forge`, host, owner, repository, `branch`, branch segments.
- `number`: integer. A number the connector knows it by.

## report

A report in words.

- `words`: integer, 0 to 4,294,967,295. The most bytes the report may hold.

## verdict

A verdict: one of its choices, with the fields that choice asks for.

- `choices`: list of at most 1,048,576, each `choice`. The choices the task gives one of.
- `cut`: true or false. Always false: a value is set whole.

## choice

A verdict choice.

- `code`: integer, 0 to 4,294,967,295. Its number, unique among the choices.
- `words`: integer, 0 to 4,294,967,295. The most bytes of the verdict's explanation.
- `followups`: integer, 0 to 4,294,967,295. Max verdict follow-up delegates.
- `fields`: list of at most 32, each `choice_field`. The fields the verdict must fill, each named once.

## choice_field

A field a verdict fills.

- `name`: hex, at most 32,768 bytes. Its name as hex, not empty.
- `max`: integer, 0 to 4,294,967,295. The most bytes of its value.

## change

A connector's result: for the forge, a change.

- `connector`: integer, 0 to 65,535. The connector that gives the result: 1 is the forge.
- `kind`: integer, 0 to 65,535. Result kind: forge 1 landed change, 2 change made but not landed.
- `words`: integer, 0 to 4,294,967,295. Most report bytes, per change title and body.

## budget

Task spend.

- `"given"`: The creator gives it, within their allotment.
- `amount`: integer. This much.
- `as_allowed`: `as_allowed`. Creator's remaining funds at task creation.

## as_allowed

As much as fits.

- `cap`: integer, or null. The most it may be, or null for no cap.

## deadline

Task end time.

- `"none"`: No deadline.
- `"given"`: The creator gives one, or none.
- `span`: nanoseconds. This long after the task is made.

## flags

How a task behaves and is kept.

- `conversation`: true or false. Conversation: requester words are turns; next run uses newest revision.
- `listed`: true or false. List by project and label once ended?
- `retention`: `"kept"` or `"forgotten_once_read"`. How long it is kept. `"kept"`: Until the deployment's retention horizon. `"forgotten_once_read"`: Until its requester has read it and no live task names it.
- `ended_by`: `ended_by`. What ends it.

## ended_by

How a task ends.

- `"result"`: Its run's result.
- `"requester"`: Requester alone; needs `conversation` and charter `executor`, else `shape`.
- `"standing"`: Only cancel; an agent with no conversation or tracking.
- `closer`: `closer`. A named procedure ends it.

## closer

Closes the task once nothing remains to await.

- `connector`: integer, 0 to 65,535. The closing procedure's connector.
- `code`: integer, 0 to 4,294,967,295. The procedure's code.

## llm

The models a task asks for.

- `account`: integer, 0 to 4,294,967,295, or null. Paying account; null: any able to pay.
- `main`: `model_effort`. The main model and its effort.
- `subagents`: list of at most 64, each `model_effort`. The models its runs' sub-agents may use, each named once.
- `later`: `llm-later`. Optional, left out none: the choice after the task's first activation.

## llm-later

Model choice after the first activation.

- `main`: `model_effort`. The later main model and its effort.
- `subagents`: list of at most 64, each `model_effort`. Absent: []. Models for later runs' sub-agents, each named once.

## model_effort

A model, by name, and its effort.

- `model`: text, at most 4,096 bytes. A configured model's name, one the task's charter offers.
- `effort`: text, at most 32 bytes, or null. Model's effort; null if it takes none.

## policy

Project policy copied at creation; changes affect later projects.

- `escalation_role`: integer, 0 to 4,294,967,295, or null. Final escalation role or null; must accept proposals and decide escalations.
- `ceiling`: `authority`. Project task ceiling; within rules, else `rules` refusal.
- `projections`: list of at most 32, each `grant`. Goal projection writes to project homes; each grant within `ceiling`.
- `period_spend`: integer. What the project may spend in a period; within `rules.period_spend`.
- `roles`: list of at most 32, each `role`. The project's roles: what a party in each may do.
- `requirements`: list of at most 32, each `requirement`. Project effect needs, beside rules' requirements.
- `permissions`: list of at most 32, each `permission`. Project role given by forge collaborator permission at adoption.
- `templates`: list of at most 16, each `template_override`. The project's overrides of templates.

## role

A role in a project.

- `number`: integer, 0 to 4,294,967,295. 0 owner, 1 maintainer, 2 member, 3 observer; others policy-defined. 0 must allow task creation, else `owner` refusal.
- `authority`: `authority`. Max party task allotment, within policy `ceiling`.
- `period_spend`: integer. Party allotment per period, within policy `period_spend`.
- `requests`: integer, 0 to 65,535. Party request bits: 1 create, 2 allot, 4 accept, 8 amend, 16 cancel, 32 release, 64 move, 128 watch, 256 change policy.
- `decides`: integer, 0 to 255. Proposals a party may decide, bits: 1 task batch, 2 effect, 4 widening, 8 amendment, 16 escalation.

## template_override

Project template override; null keeps the template's member.

- `template`: integer, 0 to 4,294,967,295. The template's number; offered, and once in the policy.
- `executor`: `executor`, or null. Its executor, or null.
- `budget`: `budget`, or null. Its budget, or null.
- `deadline`: `deadline`, or null. Its deadline, or null.
- `priority`: integer, 0 to 4,294,967,295, or null. Its default priority, or null; an untracked template ignores it.

## permission

Forge collaborator permission and its project role at adoption.

- `connector`: integer, 0 to 65,535. The connector: 1 is the forge.
- `permission`: integer, 0 to 65,535. The permission: 0 none, 1 read, 2 write, 3 admin.
- `role`: integer, 0 to 4,294,967,295. The role it gives, one of the policy's `roles`.

## rules

Deployment ceilings, reread per decision. Narrowing affects existing tasks; widening never widens them.

- `ceiling`: `authority`. The most any project may have.
- `period_spend`: integer. The most a project's period spend may be.
- `minimum_run_spend`: integer. A run starts only with more than this to spend.
- `maximum_run_spend`: integer. Maximum run funds; above `minimum_run_spend`, within `ceiling` spend.
- `implies`: list of at most 32, each `implication`. Kinds of a connector's whose grant grants other kinds too.
- `requirements`: list of at most 32, each `requirement`. What an effect needs, in every project.

## implication

One kind whose grant grants another.

- `connector`: integer, 0 to 65,535. The connector: 1 is the forge.
- `kind`: integer, 0 to 65,535. The kind whose grant also grants `implies`.
- `implies`: integer, 0 to 65,535. The kind it grants.

## authority

Authority: tools, models, grants, delegation, spend and deadline.

- `models`: `models`. The models its runs may use.
- `tools`: integer. Engine tool bits: 1 delegate, 2 message, 4 amend/cancel/release, 8 decide/withdraw, 16 propose, 32 subscribe, 64 effect, 128 note, 256 recall, 512 forge reads, 1024 settings, 2048 asking.
- `grants`: list of at most 1,048,576, each `grant`. Permitted connector effects and reads on resources.
- `delegation`: `delegation`. The delegates it may make.
- `spend`: integer. The most its tree may spend.
- `deadline`: nanoseconds since 1970 UTC, or null. The latest it may end, or null for no deadline.
- `notes`: integer, 0 to 255. Note scope bits: 1 its goal, 2 its project, 4 deployment.
- `note_resources`: list of at most 1,048,576, each `resource`. The resources it may write notes on.

## models

Available models.

- `"all"`: Every configured model, those configured later included.
- `named`: list of at most 64, each hex, at most 4,096 bytes. These models alone, each name as hex; an empty list allows none.

## grant

Leave for a connector effect or read on resources.

- `connector`: integer, 0 to 65,535. The connector: 1 is the forge.
- `kind`: integer, 0 to 65,535. Forge kinds: 1 read, 2 push, 3 open PR, 4 merge, 5 review, 6 status, 7 comment, 8 open issue, 9 branch, 10 edit PR, 11 update PR from base, 12 close/reopen PR, 13 edit issue, 14 reviewers, 15 delete branch.
- `pattern`: `pattern`. The resources it covers.

## pattern

Exact `segments`, then `last`. Repository: `forge`, host, owner, repository. Branch: append `branch`, branch segments.

- `segments`: list of at most 9, each hex, at most 32,768 bytes. The first segments, each as hex, each matched exactly.
- `last`: `last`. The segment after them.

## last

The last segment: an object with one of these members.

- `exact`: hex, at most 32,768 bytes. Exactly this segment, as hex, and nothing beneath it.
- `open`: hex, at most 32,768 bytes. Hex segment prefix and descendants; empty covers every segment.

## delegation

The delegates a task may make.

- `kinds`: list of at most 22, each `delegate`. Who its delegates may be.
- `tasks`: integer, 0 to 4,294,967,295. Tree's lifetime task count, not concurrent.
- `depth`: integer, 0 to 4,294,967,295. How far below it its tree may go; 0 allows no delegate.

## delegate

A kind of delegate: a one-member object.

- `charter`: integer, 0 to 4,294,967,295. An agent under this charter, which must be offered.
- `procedure`: integer, 0 to 4,294,967,295. This procedure: 2 is the forge's change.
- `role`: integer, 0 to 4,294,967,295. A person in this role.
- `job`: integer, 0 to 4,294,967,295. This job: 3 is the project's merge job.

## resource

Resources it may write notes on.

- `connector`: integer, 0 to 65,535. The connector: 1 is the forge.
- `pattern`: `pattern`. The resources.

## requirement

Requirements for a connector effect on resources, independent of authority.

- `connector`: integer, 0 to 65,535. The connector whose effect it constrains: 1 is the forge.
- `kind`: integer, 0 to 65,535. The kind of effect; implications do not apply.
- `pattern`: `pattern`. The resources it applies to.
- `judge`: `judge`. Who judges it.
- `guard`: `guard`. When it is judged.
- `must_be_guarded`: true or false. Require guarding as the effect acts; unguardable effects refuse.

## judge

The connector that judges a requirement, and its criterion.

- `connector`: integer, 0 to 65,535. The judging connector: 1 is the forge.
- `requirement`: integer, 0 to 65,535. Forge criteria: 1 CI passed at head, 2 head up to date with base, 3 gate, 4 approvals.
- `parameters`: integer, 0 to 4,294,967,295. A number the judging connector reads the criterion's terms by.

## guard

When to judge.

- `"guarded"`: As the effect is made, by the system that makes it.
- `observed`: `observed`. Before acting, from recent facts.

## observed

A requirement judged from facts read before the effect.

- `freshness`: nanoseconds. How old those facts may be.

## landing-rule

Criteria for every covered landing.

- `pattern`: `landing-pattern`. `forge`, host, repository owner and name, then `branch` and its segments; project rules may use `branch` and its segments alone for that branch in every project repository.
- `enforce`: true or false. True: every covered landing needs these criteria. False: only the change procedure checks.
- `ci`: true or false. CI must have passed at the change's head.
- `up_to_date`: true or false. The change's head must contain its base's tip.
- `gates`: list of at most 64, each `landing-gate`. The gates every change it covers asks for.
- `approvals`: list of at most 64, each `landing-approval`. Distinct people per role required to approve by forge review.
- `sets`: list of at most 6, each integer, 0 to 255. Unique project path sets 1 to 6; default []. Omitted if empty; none for deployments.

## landing-gate

Each change needs agent review, or a repository check without CI.

- `number`: integer, 0 to 4,294,967,295. The agent review's or repository check's gate number.
- `blocking`: true or false. Whether this gate must pass before the change lands.
- `freshness`: `"exact"` or `"clean"`. At which later heads a verdict still counts. `"exact"`: Verdict counts at its given head. `"clean"`: Verdict also counts at later heads from cleanly merged updates.

## landing-approval

Distinct project role holders approve by forge reviews.

- `role`: integer, 0 to 4,294,967,295. A role in the project's policy whose people approve the change.
- `people`: integer, 0 to 4,294,967,295. Distinct role holders required to approve the change.
- `freshness`: `"exact"` or `"clean"`. At which later heads an approval still counts. `"exact"`: Verdict counts at its given head. `"clean"`: Verdict also counts at later heads from cleanly merged updates.

## landing-pattern

Base segments and final segment/prefix. Project rules may start at `branch` across their repositories.

- `segments`: list of at most 9, each hex, at most 4,096 bytes. The exact base segments of the resources the rule covers.
- `last`: `landing-last`. Final exact segment or open prefix, and descendants.

## landing-last

Final resource segment, in hex.

- `exact`: hex, at most 4,096 bytes. One last segment equal to these bytes.
- `open`: hex, at most 4,096 bytes. A final segment beginning with these bytes, with all below it.

## forge-charter

Charter number, 0 to 4,294,967,295. Setting 1: change; setting 2: reviews.

## dependency

Project dependency: select by project and number (1 to forge.dependencies). No stock default or automatic action.

- `project`: integer, 0 to 4,294,967,295. Required project number.

## follow

Context-to-owned follow: select by project and number (1 to forge.follows); no stock default.

- `source`: `follow-source`. Context source; repository and branch fixed once accepted.
- `target`: `follow-target`. Owned target; repository and branch fixed once accepted.
- `owner`: `follow-owner`. Follow changes' owner; required.
- `start`: `"owner"` or `"code"`. When changes start; absent/empty: owner. `"owner"`: Wait for the owner; the default. `"code"`: Start after the previous change lands.
- `producer`: text, at most 8,192 bytes. Change parameter 11, C;P;R. Absent/empty: change charter and its defaults.
- `check`: `follow-check`. The follow check; required.

## follow-source

Context repository. Empty branch: adopted default; empty paths: all.

- `forge`: integer, 0 to 65,535. The forge number.
- `repository`: integer, 0 to 4,294,967,295. The repository id on that forge.
- `branch`: text, at most 4,096 bytes. Source branch; absent/empty: adopted default.
- `paths`: list of at most 256, each text, at most 256 bytes. At most forge.follow_paths nonempty segment prefixes; no edge or doubled slash.

## follow-target

Owned, writable repository. Absent/empty branch: landing branch.

- `forge`: integer, 0 to 65,535. The forge number.
- `repository`: integer, 0 to 4,294,967,295. The repository id on that forge.
- `branch`: text, at most 4,096 bytes. Target branch; absent/empty: landing branch.

## follow-owner

A person in any role of this project.

- `person`: integer. Project person's number.

## follow-check

Target workflow and report artifact: each required, 1 to 256 bytes.

- `workflow`: text, at most 256 bytes. Target workflow file; required, nonempty.
- `artifact`: text, at most 256 bytes. Uploaded artifact name; required, nonempty.
- `format`: `"junit"` or `"lines"`. The report format; required. `"junit"`: JUnit XML. `"lines"`: Line-format report.

## template-goal

Goal design-first and close-out switches. Absent: plans, no close-out.

- `design`: true or false. Coordinator delivers design before planning changes? Required.
- `close_out`: true or false. Coordinator makes close-out after goal lands? Required.

## project-ways

Project display and goal start. Absent: drafts true, others false, way 0.

- `drafts`: true or false. Whether chats can write drafts.
- `goals`: true or false. Whether the rail shows Goals.
- `changes`: true or false. Whether the rail shows Changes.
- `way`: integer, 0 to 4,294,967,295. Goal's starting template: project-offered, tracked set; 0 for the first.

## project-questions

The route a project's change takes when its maker names none.

- `"none"`: No question route; the stock's value.
- `"up"`: A change's questions go up to its parent.

## project-classes

Classes by kind and path set; absent: Agents review's table.

- `items`: list of at most 20, each `class-item`. The table's class items.
- `sets`: list of at most 6, each `path-set`. The path sets its items may name.

## class-item

Kind/set, optional words, class; one wordless item per subject. Money, irreversible, rules and other always wait, with no item.

- `kind`: `"wording"`, `"retry"`, `"implementation"`, `"tests"`, `"dependency"` or `"departure"`. Exactly one of kind or set is required.
- `set`: integer, 0 to 255. A set in this table, from 1 to 6; exactly one of kind or set.
- `words`: text, at most 120 bytes. Optional wording for this item, empty when absent.
- `class`: `"decide"`, `"ask"` or `"ask-away"`. Wait is never a table class. `"decide"`: Agents decide and list it. `"ask"`: Ask the person; Away may decide. `"ask-away"`: Ask the person even when away.

## path-set

Repository-relative path patterns, uniquely numbered 1 to 6.

- `number`: integer, 0 to 255. Distinct within the table, from 1 to 6.
- `name`: text, at most 40 bytes. A nonempty name.
- `patterns`: list of at most 6, each text, at most 80 bytes. Nonempty repository-relative patterns: * within segments, ** for a whole segment.

## charter-words

Charter purpose: 1 to 120 UTF-8 bytes, no controls. Defaults to instructions' first sentence. Select by charter number.

## project-words

How work runs headline and lede; absent uses the way's words.

- `headline`: text, at most 120 bytes. Required: 1 to 120 UTF-8 bytes holding no control character.
- `lede`: text, at most 480 bytes. Required: 1 to 480 UTF-8 bytes holding no control character.

## project-areas

A project's areas, at most 16.

- `areas`: list of at most 16, each `area`. Required; order is kept; names are unique.

## project-triager

A project's triager.

- `owner`: `area-owner`. Required.

## area

A named scope of project work and its owner.

- `name`: text, at most 32 bytes. Required: 1 to 32 ASCII lowercase letters, digits or -, starting with a letter.
- `words`: text, at most 480 bytes. Required: 1 to 480 UTF-8 bytes with no control character.
- `owner`: `area-owner`. Required.

## area-owner

One owner: task, party, role or charter.

- `task`: integer. A live task of the project.
- `party`: integer. A party with a role in the project.
- `role`: integer, 0 to 4,294,967,295. Below authority.roles; 0 is the owners.
- `charter`: integer, 0 to 4,294,967,295. From 1 through tasks.charters.
