# Settings fields

Kind: reference

Every member of a settings value: its type, its bound and what it does to work, as a settings file sets it and `settings show` prints it.

A settings file is a `file`, below; each section is one shape of JSON object, and each bullet one of its members, with its type, its bound and what it does. Read a value with `settings show`, given its kind and number, edit it, and set it whole. A bound here is what ctl sends; the deployment may still refuse a value, naming its fault: `shape`, `count`, `reference`, `rules`, `budget`, `grace` or `owner`. Spend is in the deployment's units, those of its models' prices. Hex is bytes in lower-case hexadecimal: `666f726765` is `forge`.

## file

A settings file, as `settings apply` and `settings propose` read it.

- `items`: list of at most 1,024, each `item`. The change: an item for each setting it touches, each setting once.

## item

One setting, and what becomes of it: `setting`, then exactly one of `set` and `edit`.

- `setting`: `setting`. The setting the item changes.
- `set`: the setting's kind's value: `charter`, `template`, `policy`, `rules`, `model`, `landing-rule`, `forge-charter`, `template-goal`, `project-policy`, `project-ways`, `project-questions`, `project-classes`, `charter-words` or `project-words`. Its new value, whole, which replaces the one before.
- `edit`: `"removed"` or `"reset"`. What becomes of the setting, in place of a value. `"removed"`: No longer offered, its default included; the rules cannot be removed. `"reset"`: Its change erased, so that it follows its default again.

## setting

Which setting: its kind and number, or project for project settings.

- `kind`: `"model"`, `"charter"`, `"template"`, `"policy"`, `"project-policy"`, `"rules"`, `"landing-rule"`, `"forge-charter"`, `"template-goal"`, `"project-ways"`, `"project-questions"`, `"project-classes"`, `"charter-words"` or `"project-words"`. The setting's kind. `"project-policy"`: A project's own policy: only `set`, at most once per file; may share a file with other settings. Its value is a policy.
- `number`: integer, 0 to 4,294,967,295. The setting's number, from 1; left out for rules, project-policy, project-ways, project-questions, project-classes and project-words.
- `project`: integer, 0 to 4,294,967,295. Required for project-ways, project-questions, project-classes and project-words; absent for charter-words. For a landing rule, a project's number, from 1, the deployment's when left out; for a project's policy, its project, required.

## charter

An agent run's instructions, models, tools, hosts and budget. Runs use the charter from the task's creation revision.

- `rest`: true or false. Whether the other catalogue models follow the named first choices; false when absent.
- `number`: integer, 0 to 4,294,967,295. The charter's number, the same as `setting.number`.
- `instructions`: text, at most 1,048,576 bytes. What the run's agent reads first, before its task's brief.
- `waiting`: nanoseconds. How long a run with nothing to do waits for words or news before it parks.
- `resume`: true or false. Whether a run resumes its task's transcript, when there is one, rather than start afresh.
- `turns`: integer, 0 to 4,294,967,295. The most model turns a run takes; more than `reserve_turns`, else refused as `budget`.
- `time`: nanoseconds. The longest a run lasts before it is cancelled; more than `reserve_time`, else refused as `budget`.
- `reserve_turns`: integer, 0 to 4,294,967,295. Of `turns`, those kept for the run to finish its work and report.
- `reserve_time`: nanoseconds. Of `time`, the part kept for finishing and reporting. With `modify` true, at least the worker's check timeout (`check_timeout` in milliseconds). A worker refuses admission if any applicable current or held charter version has less reserve. See settings help for recovery through held revisions.
- `model`: `model`. The default model: a task that names none takes the first of `model` and `alternatives` whose provider has an account that can pay.
- `alternatives`: list of at most 32, each `model`. The other models the charter offers, in order after `model`, each named once; a run's sub-agents may use them too.
- `inspect`: true or false. Whether the run's agent may read, list and search its workspace.
- `modify`: true or false. Whether the run's agent may write, edit and remove files in its workspace.
- `shell`: true or false. Whether the run's agent may run commands in its workspace.
- `agents`: true or false. Whether the run's agent may open sub-agents, each with no more tools than its own.
- `call_timeout`: nanoseconds. The longest one of the run's calls to temper's tools may take.
- `tools`: integer. Host families offered where both charter and task allow them. Asking (2048) alone or with settings (1024) declares only see, document and, with settings, propose_settings; other charters get settings tools by 1024. Default 1023: every family but settings and asking.
- `hosts`: `"workers"`, `"engine"` or `"both"`. Which hosts may run it. `"workers"`: The deployment's workers alone. `"engine"`: The engine's own slots alone. `"both"`: Either.
- `spend`: integer. A spend the charter must hold: at least `reserve_turns` plus one of the default model's completions at their costliest, else refused as `budget`. A run is given its task's funds left, at most `rules.maximum_run_spend`.
- `wake`: `wake`, or null. Which messages wake a task made under the charter whose batch names no wake policy; null, or left out, for jig's default: every rule `"immediate"`, `results` `"last_or_failure"`, and questions, answers and timers waking.
- `documents`: list of at most 8, each `document`. The documents the charter's runs read, each as a brief section of its own; at most `tasks.charter_documents`, else refused as `shape`.
- `parts`: list of at most 8, each `charter-part`. Optional, left out []: ordered parts chosen by a run's reasons; at most tasks.charter_parts. A charter that resumes takes no parts.

## charter-part

Instructions a run reads after the charter's shared text when its reasons meet this part's on.

- `on`: list of at most 14, each `"first"`, `"question"`, `"question_answered"`, `"question_ended"`, `"question_owed"`, `"escalation"`, `"proposal"`, `"person_stalled"`, `"result"`, `"notice"`, `"nothing_awaited"`, `"news"`, `"timer"` or `"words"`. Nonempty, each name once. Names offered to every run choose no part and are refused.
- `instructions`: text, at most 1,048,576 bytes. Text appended after the shared instructions when this part is chosen.

## document

A file in each charter run's brief, read at the branch head during preparation; landed rules reach the next run.

- `forge`: text, at most 32,768 bytes. The canonical `host` in Ask temper's `see settings` inventory under `project repositories`, alongside OWNER/NAME; people can also read `temper ctl DIR forge list` (or `forge list --json`). It is the host in repository resource names, including its port; copy it unchanged, not the forge number or web address.
- `repository`: text, at most 65,537 bytes. The repository as OWNER/NAME; one adopted in no project gives a section saying so.
- `branch`: text, at most 131,075 bytes. The branch whose head is read, its segments joined by `/`.
- `path`: text, at most 256 bytes. The file's path in the repository, at most 256 bytes.
- `heading`: text, at most 80 bytes, or null. A Markdown heading's text: the run reads from that heading to the next of its level or above; null reads the whole file.

## catalogue

A whole numbered model setting: its runtime model, display facts, availability and origin.

- `number`: integer, 0 to 4,294,967,295. Its permanent number, from 1 to 64, matching setting.number.
- `model`: `model`. The runtime model, whole.
- `shown`: text, at most 4,096 bytes, or null. Its name for showing, or null.
- `description`: text, at most 512 bytes, or null. Its description, or null.
- `picker`: true or false. Whether a model picker shows it.
- `withdrawn`: true or false. Whether it is withdrawn; its number and name stay.
- `source`: `"stock"`, `"list"` or `"person"`. Where the model came from. `"stock"`: The binary. `"list"`: A provider list. `"person"`: A person.
- `estimated`: true or false. Whether its prices are provider defaults.

## model

A runtime model. In a charter, every member but `effort` must match its configured row, else refused as `reference`; a model setting supplies the row whole.

- `provider`: integer, 0 to 4,294,967,295. The configured LLM provider that serves it; 1 when absent.
- `dialect`: integer, 0 to 4,294,967,295. The protocol the model is spoken to in, as configured.
- `endpoint`: integer, 0 to 4,294,967,295. The number of the configured endpoint that serves it.
- `program`: text, at most 32 bytes, or null. The program that runs it, as configured, or null for none.
- `name`: text, at most 4,096 bytes. The model's name, as its provider knows it.
- `window`: integer, 0 to 4,294,967,295. The most tokens of input it reads.
- `output`: integer, 0 to 4,294,967,295. The most tokens it writes in one completion, thinking included.
- `input_price`: integer. What fresh input costs, in spend units per `price_unit` tokens.
- `cached_price`: integer. What cached input costs, in spend units per `price_unit` tokens.
- `cache_write_price`: integer, or null. What writing its cache costs, in spend units per `price_unit` tokens, or null to price it as fresh input.
- `output_price`: integer. What output costs, in spend units per `price_unit` tokens.
- `price_unit`: integer, 0 to 4,294,967,295. The number of tokens each price is for.
- `efforts`: list of at most 16, each text, at most 32 bytes. The efforts it takes, in order, as configured.
- `effort`: text, at most 32 bytes, or null. The effort a run asks of it, one of `efforts`, or null for its default.

## wake

What wakes a task, kind of message by kind; a proposal, an escalation and a cancel always wake it.

- `words`: `wake_rule`. Words from tasks and from the deployment; a person's words always wake it at once.
- `notices`: `wake_rule`. Notices: a task it references done or held, a proposal passing through, a hold it waits for.
- `news`: `wake_rule`. News its connectors class as waking for it.
- `results`: `"never"`, `"each"` or `"last_or_failure"`. Its delegates' results. `"never"`: None wakes it. `"each"`: Each delegate's result wakes it. `"last_or_failure"`: The last delegate's result, or any failure.
- `questions`: true or false. Whether a delegate's question wakes it.
- `answers`: true or false. Whether an answer to its own question wakes it.
- `timers`: true or false. Whether its timers wake it.

## wake_rule

How a kind of message wakes a task: a word, or an object with one member.

- `"never"`: It waits for the task's next run.
- `"immediate"`: It wakes the task at once.
- `batch`: `wake_batch`. Once enough have arrived, or the oldest has waited long enough.
- `paced`: nanoseconds. At most once in this span from the task's last run's start; the first after a quiet span wakes it at once. Above zero, else refused as `shape`.

## wake_batch

A batch of messages that wakes a task once.

- `count`: integer, 0 to 4,294,967,295. Wake once this many have arrived; above zero, else refused as `shape`.
- `age`: nanoseconds. Or once the oldest has waited this long; above zero, else refused as `shape`.

## template

New tasks of one shape: executor, result, budget, deadline and flags. Creation values stay; changes affect later tasks.

- `resources`: list of at most 32, each `symbol`. Optional; absent means an empty list. No pair twice. Resource symbols resolved for the new task; forge connector 1, symbol 1 is its own goal branch.
- `number`: integer, 0 to 4,294,967,295. The template's number, the same as `setting.number`; a chat is filled from template 1 and a goal from template 2.
- `name`: `text`. Its name, as clients show it.
- `executor`: `executor`. Who runs the task.
- `contract`: `contract`. The result the task must give.
- `shallower`: integer, 0 to 4,294,967,295. How many levels of delegation the task has below its creator's role: 1 is one level; 0 leaves the template unusable.
- `budget`: `budget`. What the task may spend.
- `deadline`: `deadline`. By when the task must end.
- `tracked`: integer, 0 to 4,294,967,295, or null. Its default priority as tracked work, lower first, or null for untracked.
- `flags`: `flags`. How the task behaves and is kept.
- `label`: integer, 0 to 4,294,967,295. A number that groups the task in the lists of ended tasks; temper reads nothing else from it.
- `grace`: nanoseconds, or null. How long a chat's close lets its runs finish before it cancels them, or null; past the deployment's limit, refused as `grace`.
- `llm`: `llm`, or null. The models the task asks for, or null for its charter's default.

## symbol

A connector resource resolved when the task is created.

- `connector`: integer, 0 to 65,535. The connector that resolves the symbol; 1 is the forge.
- `symbol`: integer, 0 to 65,535. The connector's resource symbol; the forge's 1 is its own goal branch.

## text

A text, whole.

- `bytes`: text, at most 65,536 bytes. The text.
- `cut`: true or false. Always false: a value is set whole.

## executor

Who runs a task: a word, or an object with one member.

- `"chosen"`: The creator chooses a charter or procedure their role may create with.
- `charter`: integer, 0 to 4,294,967,295. An agent under this charter, which must be offered, else refused as `reference`.

## contract

The result a task must give: an object with one of these members.

- `approval`: `approval`. A person's approval of an exact state; an agent given one can only fail.
- `report`: `report`. A report.
- `verdict`: `verdict`. One of a set of verdicts.
- `change`: `change`. A connector's result.

## approval

An approval of an exact state.

- `state`: hex, at most 32,768 bytes. The state approved, which the approval must give back byte for byte.
- `part`: `part`. What a client shows the approval with.

## part

A thing a connector owns, which a client shows.

- `connector`: integer, 0 to 65,535. The connector that owns it: 1 is the forge.
- `kind`: `"resource"`, `"pool"`, `"result"`, `"procedure"`, `"effect"`, `"outbox"` or `"hold"`. What kind of thing it is.
- `key`: `key`. Which one.

## key

Which thing: an object with one of these members.

- `path`: list of at most 9, each hex, at most 32,768 bytes. A resource's name, its segments as hex: `forge`, the host, the owner and the repository, then `branch` and the branch's segments.
- `number`: integer. A number the connector knows it by.

## report

A report in words.

- `words`: integer, 0 to 4,294,967,295. The most bytes the report may hold.

## verdict

A verdict: one of its choices, with the fields that choice asks for.

- `choices`: list of at most 1,048,576, each `choice`. The choices the task gives one of.
- `cut`: true or false. Always false: a value is set whole.

## choice

One verdict a task may give.

- `code`: integer, 0 to 4,294,967,295. Its number, unique among the choices.
- `words`: integer, 0 to 4,294,967,295. The most bytes of the verdict's explanation.
- `followups`: integer, 0 to 4,294,967,295. The most follow-up delegates the verdict may ask for.
- `fields`: list of at most 32, each `choice_field`. The fields the verdict must fill, each named once.

## choice_field

A field a verdict fills.

- `name`: hex, at most 32,768 bytes. Its name as hex, not empty.
- `max`: integer, 0 to 4,294,967,295. The most bytes of its value.

## change

A connector's result: for the forge, a change.

- `connector`: integer, 0 to 65,535. The connector that gives the result: 1 is the forge.
- `kind`: integer, 0 to 65,535. The kind of result: the forge's 1 is a landed change, its 2 a change made but not landed.
- `words`: integer, 0 to 4,294,967,295. The most bytes of the result's report, each of the change's title and body.

## budget

What a task may spend: a word, or an object with one member.

- `"given"`: The creator gives it, within their allotment.
- `amount`: integer. This much.
- `as_allowed`: `as_allowed`. As much as the creator has left when the task is made.

## as_allowed

As much as fits.

- `cap`: integer, or null. The most it may be, or null for no cap.

## deadline

By when a task must end: a word, or an object with one member.

- `"none"`: No deadline.
- `"given"`: The creator gives one, or none.
- `span`: nanoseconds. This long after the task is made.

## flags

How a task behaves and is kept.

- `conversation`: true or false. Whether the task is a conversation: its requester's words are its turns, and it moves to the newest revision at its next run.
- `listed`: true or false. Whether it is listed, by project and label, once it has ended.
- `retention`: `"kept"` or `"forgotten_once_read"`. How long it is kept. `"kept"`: Until the deployment's retention horizon. `"forgotten_once_read"`: Until its requester has read it and no live task names it.
- `ended_by`: `ended_by`. What ends it.

## ended_by

What ends a task: a word or one-member object.

- `"result"`: Its run's result.
- `"requester"`: Its requester alone; requires `conversation` and a charter `executor`, else `shape` refusal.
- `"standing"`: Only cancel; an agent with no conversation or tracking.
- `closer`: `closer`. A named procedure ends it.

## closer

Procedure closing the task when nothing remains to await.

- `connector`: integer, 0 to 65,535. The closing procedure's connector.
- `code`: integer, 0 to 4,294,967,295. The procedure's code.

## llm

The models a task asks for.

- `account`: integer, 0 to 4,294,967,295, or null. The account to pay with, or null for any that can.
- `main`: `model_effort`. The main model and its effort.
- `subagents`: list of at most 64, each `model_effort`. The models its runs' sub-agents may use, each named once.
- `later`: `llm-later`. Optional, left out none: the choice after the task's first activation.

## llm-later

The choice every run after the task's first activation takes.

- `main`: `model_effort`. The later main model and its effort.
- `subagents`: list of at most 64, each `model_effort`. Optional, left out []: the models later runs' sub-agents may use, each named once.

## model_effort

A model, by name, and its effort.

- `model`: text, at most 4,096 bytes. A configured model's name, one the task's charter offers.
- `effort`: text, at most 32 bytes, or null. One of the model's efforts, or null for a model that takes none.

## policy

A project's initial policy, copied and kept. Changes affect later projects, never an existing project's policy.

- `escalation_role`: integer, 0 to 4,294,967,295, or null. The role an escalation reaches last, or null; that role must accept proposals and decide escalations.
- `ceiling`: `authority`. The most any task in the project may have; within the rules, else refused as `rules`.
- `projections`: list of at most 32, each `grant`. The grants under which temper writes goals' projections into the project's homes; each within `ceiling`.
- `period_spend`: integer. What the project may spend in a period; within `rules.period_spend`.
- `roles`: list of at most 32, each `role`. The project's roles: what a party in each may do.
- `requirements`: list of at most 32, each `requirement`. What an effect in the project needs, beside the rules' requirements.
- `permissions`: list of at most 32, each `permission`. The project role each forge collaborator's permission gives, at adoption.
- `templates`: list of at most 16, each `template_override`. The project's overrides of templates.

## role

A role in a project.

- `number`: integer, 0 to 4,294,967,295. Its number: 0 is owner, 1 maintainer, 2 member, 3 observer; others are the policy's own. Role 0 must be able to create tasks, else refused as `owner`.
- `authority`: `authority`. The most a party in it may give a task; within the policy's `ceiling`.
- `period_spend`: integer. What a party in it may allot in a period; within the policy's `period_spend`.
- `requests`: integer, 0 to 65,535. What a party in it may ask, a bit each: 1 create, 2 allot, 4 accept, 8 amend, 16 cancel, 32 release, 64 move, 128 watch, 256 change the policy.
- `decides`: integer, 0 to 255. The proposals a party in it may decide, a bit each: 1 a batch of tasks, 2 an effect, 4 a widening, 8 an amendment, 16 an escalation.

## template_override

A project's override of a template; a member left null keeps the template's own.

- `template`: integer, 0 to 4,294,967,295. The template's number; offered, and once in the policy.
- `executor`: `executor`, or null. Its executor, or null.
- `budget`: `budget`, or null. Its budget, or null.
- `deadline`: `deadline`, or null. Its deadline, or null.
- `priority`: integer, 0 to 4,294,967,295, or null. Its default priority, or null; an untracked template ignores it.

## permission

A forge collaborator's permission, and the project role it gives at adoption.

- `connector`: integer, 0 to 65,535. The connector: 1 is the forge.
- `permission`: integer, 0 to 65,535. The permission: 0 none, 1 read, 2 write, 3 admin.
- `role`: integer, 0 to 4,294,967,295. The role it gives, one of the policy's `roles`.

## rules

Deployment ceilings bound every project's policy, read afresh for each decision. Narrowing reaches existing tasks; widening never widens them.

- `ceiling`: `authority`. The most any project may have.
- `period_spend`: integer. The most a project's period spend may be.
- `minimum_run_spend`: integer. A run starts only with more than this to spend.
- `maximum_run_spend`: integer. The most a run is given of its task's funds; more than `minimum_run_spend`, and within `ceiling`'s spend.
- `implies`: list of at most 32, each `implication`. Kinds of a connector's whose grant grants other kinds too.
- `requirements`: list of at most 32, each `requirement`. What an effect needs, in every project.

## implication

One kind whose grant grants another.

- `connector`: integer, 0 to 65,535. The connector: 1 is the forge.
- `kind`: integer, 0 to 65,535. The kind whose grant also grants `implies`.
- `implies`: integer, 0 to 65,535. The kind it grants.

## authority

What a task, a role or a ceiling may do: its tools, models, grants, delegation, spend and deadline.

- `models`: `models`. The models its runs may use.
- `tools`: integer. The engine's tool families its runs may call, a bit each: 1 delegate, 2 message, 4 amend, cancel and release, 8 decide and withdraw, 16 propose, 32 subscribe, 64 effect, 128 note, 256 recall, 512 the forge's reads, 1024 settings, 2048 asking.
- `grants`: list of at most 1,048,576, each `grant`. The effects and reads it may make, each a connector's kind on some resources.
- `delegation`: `delegation`. The delegates it may make.
- `spend`: integer. The most its tree may spend.
- `deadline`: nanoseconds since 1970 UTC, or null. The latest it may end, or null for no deadline.
- `notes`: integer, 0 to 255. The scopes it may write notes in, a bit each: 1 its goal, 2 its project, 4 the deployment.
- `note_resources`: list of at most 1,048,576, each `resource`. The resources it may write notes on.

## models

Which models: a word, or an object with one member.

- `"all"`: Every configured model, those configured later included.
- `named`: list of at most 64, each hex, at most 4,096 bytes. These models alone, each name as hex; an empty list allows none.

## grant

Leave to make one kind of a connector's effect or read, on some resources.

- `connector`: integer, 0 to 65,535. The connector: 1 is the forge.
- `kind`: integer, 0 to 65,535. The kind; the forge's are 1 read, 2 push, 3 open a pull request, 4 merge, 5 review, 6 set a status, 7 comment, 8 open an issue, 9 make a branch, 10 edit a pull request, 11 update it from its base, 12 close or reopen it, 13 edit an issue, 14 set reviewers, 15 delete a branch.
- `pattern`: `pattern`. The resources it covers.

## pattern

Resources by name: the exact `segments`, then `last`. A forge's repository is `forge`, the host, the owner and the repository; a branch adds `branch` and its segments.

- `segments`: list of at most 9, each hex, at most 32,768 bytes. The first segments, each as hex, each matched exactly.
- `last`: `last`. The segment after them.

## last

The last segment: an object with one of these members.

- `exact`: hex, at most 32,768 bytes. Exactly this segment, as hex, and nothing beneath it.
- `open`: hex, at most 32,768 bytes. Any segment that begins with this, as hex, and everything beneath it; empty covers every segment.

## delegation

The delegates a task may make.

- `kinds`: list of at most 18, each `delegate`. Who its delegates may be.
- `tasks`: integer, 0 to 4,294,967,295. How many tasks its tree may make over its life, not at once.
- `depth`: integer, 0 to 4,294,967,295. How far below it its tree may go; 0 allows no delegate.

## delegate

A kind of delegate: an object with one of these members.

- `charter`: integer, 0 to 4,294,967,295. An agent under this charter, which must be offered.
- `procedure`: integer, 0 to 4,294,967,295. This procedure: 2 is the forge's change.
- `role`: integer, 0 to 4,294,967,295. A person in this role.
- `job`: integer, 0 to 4,294,967,295. This job: 3 is the project's merge job.

## resource

Resources it may write notes on.

- `connector`: integer, 0 to 65,535. The connector: 1 is the forge.
- `pattern`: `pattern`. The resources.

## requirement

What one kind of a connector's effect needs on some resources, whatever any authority says.

- `connector`: integer, 0 to 65,535. The connector whose effect it constrains: 1 is the forge.
- `kind`: integer, 0 to 65,535. The kind of effect; implications do not apply.
- `pattern`: `pattern`. The resources it applies to.
- `judge`: `judge`. Who judges it.
- `guard`: `guard`. When it is judged.
- `must_be_guarded`: true or false. Whether the system that makes the effect must check it as it does; an effect that system cannot guard is refused.

## judge

The connector that judges a requirement, and its criterion.

- `connector`: integer, 0 to 65,535. The judging connector: 1 is the forge.
- `requirement`: integer, 0 to 65,535. Its criterion; the forge's are 1 CI passed at the head, 2 the head up to date with its base, 3 a gate, 4 approvals.
- `parameters`: integer, 0 to 4,294,967,295. A number the judging connector reads the criterion's terms by.

## guard

When a requirement is judged: a word, or an object with one member.

- `"guarded"`: As the effect is made, by the system that makes it.
- `observed`: `observed`. Before the effect is made, from facts read recently enough.

## observed

A requirement judged from facts read before the effect.

- `freshness`: nanoseconds. How old those facts may be.

## landing-rule

A landing rule for the deployment or one project. Its criteria apply to every landing it covers.

- `pattern`: `landing-pattern`. `forge`, the forge's host, repository owner and name, then `branch` and its segments; or, for a project's rule, `branch` and its segments alone, covering that branch in each of the project's repositories.
- `enforce`: true or false. True makes each criterion a requirement every landing it covers must meet; false leaves them to the change procedure alone.
- `ci`: true or false. CI must have passed at the change's head.
- `up_to_date`: true or false. The change's head must contain its base's tip.
- `gates`: list of at most 64, each `landing-gate`. The gates every change it covers asks for.
- `approvals`: list of at most 64, each `landing-approval`. The distinct people holding each role who must approve the change by a forge review.

## landing-gate

A gate every change covered by the rule asks for: an agent's review, or a repository's check when it has no CI.

- `number`: integer, 0 to 4,294,967,295. The agent review's or repository check's gate number.
- `blocking`: true or false. Whether this gate must pass before the change lands.
- `freshness`: `"exact"` or `"clean"`. At which later heads a verdict still counts. `"exact"`: The verdict counts at the head it was given at. `"clean"`: The verdict also counts at later heads reached by updates that merged cleanly.

## landing-approval

Distinct people holding a project role approve the change, each by a review on the forge.

- `role`: integer, 0 to 4,294,967,295. A role in the project's policy whose people approve the change.
- `people`: integer, 0 to 4,294,967,295. How many distinct people holding the role must approve the change.
- `freshness`: `"exact"` or `"clean"`. At which later heads an approval still counts. `"exact"`: The verdict counts at the head it was given at. `"clean"`: The verdict also counts at later heads reached by updates that merged cleanly.

## landing-pattern

The exact base segments, then an exact last segment or an open prefix and all below it. A project's rule may begin at `branch`, covering that branch in each of its repositories.

- `segments`: list of at most 9, each hex, at most 4,096 bytes. The exact base segments of the resources the rule covers.
- `last`: `landing-last`. One more segment, equal to an exact value or beginning with an open value, with all below it.

## landing-last

The final resource segment, in the same hex form as a pattern's last segment.

- `exact`: hex, at most 4,096 bytes. One last segment equal to these bytes.
- `open`: hex, at most 4,096 bytes. A final segment beginning with these bytes, with all below it.

## forge-charter

An integer from 0 to 4,294,967,295: the charter's number. Setting 1 selects the change's charter; setting 2 selects its reviews'.

## template-goal

A goal template's work: whether its coordinator designs first and makes a close-out. With none kept, it plans and makes no close-out.

- `design`: true or false. Whether the coordinator delivers a design before planning its changes; required.
- `close_out`: true or false. Whether the coordinator makes a close-out after the goal lands; required.

## project-ways

What a project shows and the way its goals start. With none stored, drafts is true, the other switches false and way 0.

- `drafts`: true or false. Whether chats can write drafts.
- `goals`: true or false. Whether the rail shows Goals.
- `changes`: true or false. Whether the rail shows Changes.
- `way`: integer, 0 to 4,294,967,295. The template a goal starts by: one the project offers with tracked set, or 0 for the first.

## project-questions

The route a project's change takes when its maker names none.

- `"none"`: No question route; the stock's value.
- `"foreman"`: A change's questions go to its foreman.
- `"up"`: A change's questions go up to its parent.

## project-classes

What agents decide for a person and what asks them, by kind and by path set. With none stored, Agents review's table applies.

- `items`: list of at most 20, each `class-item`. The table's class items.
- `sets`: list of at most 6, each `path-set`. The path sets its items may name.

## class-item

Exactly one of kind or set, optional words and a class. Only one item without words per subject. Money, irreversible, rules and other always wait and have no item.

- `kind`: `"wording"`, `"retry"`, `"implementation"`, `"tests"`, `"dependency"` or `"departure"`. Exactly one of kind or set is required.
- `set`: integer, 0 to 255. A set in this table, from 1 to 6; exactly one of kind or set.
- `words`: text, at most 120 bytes. Optional wording for this item, empty when absent.
- `class`: `"decide"`, `"ask"` or `"ask-away"`. Wait is never a table class. `"decide"`: Agents decide and list it. `"ask"`: Ask the person; Away may decide. `"ask-away"`: Ask the person even when away.

## path-set

A named set of repository-relative path patterns, numbered distinctly from 1 to 6.

- `number`: integer, 0 to 255. Distinct within the table, from 1 to 6.
- `name`: text, at most 40 bytes. A nonempty name.
- `patterns`: list of at most 6, each text, at most 80 bytes. At least one nonempty repository-relative pattern: * within a segment, ** as a whole segment.

## charter-words

What the charter does: a JSON string of 1 to 120 UTF-8 bytes, no control characters. If absent, Roles uses its instructions' first sentence. Selector: charter number, no project.

## project-words

Both headline and lede for the project's own way in How work runs. If absent, it uses the way's words. Selector: project, no number.

- `headline`: text, at most 120 bytes. Required: 1 to 120 UTF-8 bytes holding no control character.
- `lede`: text, at most 480 bytes. Required: 1 to 480 UTF-8 bytes holding no control character.
