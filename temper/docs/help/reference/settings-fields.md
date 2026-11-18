# Settings fields

Kind: reference

Settings value types, bounds and effects, for settings files and `settings show`.

Settings-file JSON (`file`), read whole by `settings show KIND N`. Bullets give fields, types, bounds and effects. Ctl sends these bounds; deployment faults: `shape`, `count`, `reference`, `rules`, `budget`, `grace`, `owner`. Spend: deployment model-price units. Lowercase hex: 666f726765 encodes forge. u8/u16/u32/u64: unsigned 8/16/32/64-bit integers (0 to 2^bits-1). bool: true/false. text[N]/hex[N]: at most N bytes; list[N]: at most N entries. ? optional; ! required. Values follow setting.kind's shape; project-policy uses policy.

## file
settings file, as `settings apply` and `settings propose` read it.
- `items`: list[1,024] `item`. One item per changed setting, each setting once.

## item
`setting` and exactly one of `set`, `edit` and `overlay`.
- `setting`: `setting`. setting the item changes.
- `overlay`: `overlay`. charter overlay, instead of set or edit.
- `set`: the value for `setting.kind`. Whole replacement value.
- `edit`: `"removed"` or `"reset"`. edit instead of a value. `"removed"`: No longer offered, its default included; the rules cannot be removed. `"reset"`: Erase its change; follow its default again.

## setting
Which setting: its kind and number, or project for project settings.
- `kind`: `"model"`, `"charter"`, `"template"`, `"policy"`, `"project-policy"`, `"rules"`, `"landing-rule"`, `"forge-charter"`, `"template-goal"`, `"follow"`, `"dependency"`, `"project-ways"`, `"project-questions"`, `"project-classes"`, `"charter-words"`, `"project-words"`, `"project-landing"`, `"project-gates"`, `"project-areas"`, `"project-triager"`, `"welcome"`, `"limits"` or `"temper-limits"`. setting's kind. `"project-policy"`: Project policy: `set` only, once per file alongside other settings. `"project-landing"`: At the next start. `"project-gates"`: At the next start.
- `number`: u32. Setting number, from 1; omit for rules, welcome and limit kinds, and unnumbered project settings.
- `project`: u32. Required for project settings, follow and dependency; absent for charter-words, welcome and limit kinds. Landing rule: from 1, or deployment-wide if absent.

## charter
Run policy from the task's creation revision.
- `rest`: bool. Offer remaining catalogue models after named choices; default false.
- `number`: u32. Same as `setting.number`.
- `instructions`: text[1,048,576]. Agent's first read, before the task's brief.
- `waiting`: nanoseconds. Idle run's wait for words or news before parking.
- `resume`: bool. Resume the task's transcript if any, rather than start afresh.
- `turns`: u32. Most model turns; more than `reserve_turns`, else `budget` refusal.
- `time`: nanoseconds. Run cancellation time; more than `reserve_time`, else `budget` refusal.
- `reserve_turns`: u32. Of `turns`, reserve to finish and report.
- `reserve_time`: nanoseconds. Of `time`, reserve to finish and report. With `modify`, at least `check_timeout` milliseconds in every applicable current or held charter; otherwise worker admission refuses. See Settings help for held-revision recovery.
- `model`: `model`. Default model: first payable choice among `model` and `alternatives`.
- `alternatives`: list[32] `model`. Models after `model`, each once; also available to sub-agents.
- `inspect`: bool. May the agent read, list and search its workspace?
- `modify`: bool. May the agent write, edit and remove workspace files?
- `shell`: bool. May the agent run workspace commands?
- `agents`: bool. May the agent open sub-agents with no more tools than its own?
- `call_timeout`: nanoseconds. Longest call to temper's tools.
- `tools`: u64. Host families need charter/task leave. Asking 2048: see/document; add settings 1024 for propose_settings. Without asking, 1024 grants settings tools. Default 1023 excludes both.
- `hosts`: `"workers"`, `"engine"` or `"both"`. Which hosts may run it. `"workers"`: deployment's workers alone. `"engine"`: engine's own slots alone. `"both"`: Either.
- `spend`: u64. >= (`reserve_turns` + 1) costliest default-model completions, else `budget` refusal. Runs get remaining task funds, capped by `rules.maximum_run_spend`.
- `wake`: `wake`, or null. Wake policy for tasks made under this charter whose batch names none. Null/absent: jig's default, all rules `"immediate"`, `results` `"last_or_failure"`, questions, answers and timers waking.
- `documents`: list[8] `document`. Brief documents; at most `tasks.charter_documents`, else `shape` refusal.
- `parts`: list[8] `charter-part`. Ordered parts chosen by run reasons, at most tasks.charter_parts; default [].

## overlay
Optional default replacements and added text. Use charters or literal instructions/parts, never both.
As `charter`'s fields, replacing the default whole: `rest`, `waiting`, `resume`, `turns`, `time`, `reserve_turns`, `reserve_time`, `model`, `alternatives`, `inspect`, `modify`, `shell`, `agents`, `call_timeout`, `tools`, `hosts`, `spend`, `documents`.

- `instructions`: text[1,048,576]. Optional added text, empty when absent; not beside charters.
- `parts`: list[8] `charter-part`. Optional added text, empty when absent; not beside charters.
- `charters`: list[4,294,967,295] text[4,294,967,295]. Optional nonempty directories, relative to settings file; instead of instructions/parts, only charters 5 and 2. Markdown counts toward file bound.

## charter-part
Instructions after shared text, read when run reasons meet `on`.
- `on`: list[14] `"first"`, `"question"`, `"question_answered"`, `"question_ended"`, `"question_owed"`, `"escalation"`, `"proposal"`, `"person_stalled"`, `"result"`, `"notice"`, `"nothing_awaited"`, `"news"`, `"timer"` or `"words"`. Nonempty unique names; refuses those offered to every run.
- `instructions`: text[1,048,576]. Text appended after the shared instructions when this part is chosen.

## document
Brief file at preparation branch head.
- `forge`: text[32,768]. Repository resource's `host`, port included: copy from `see settings`' `project repositories` or `temper ctl DIR forge list`, not forge number or web address.
- `repository`: text[65,537]. OWNER/NAME; if no project adopts it, the section says so.
- `branch`: text[131,075]. Branch head to read; segments joined by `/`.
- `path`: text[256]. file's path in the repository, at most 256 bytes.
- `heading`: text[80], or null. Markdown heading: read through the next peer or ancestor heading; null reads the whole file.

## catalogue
Numbered runtime model and its display facts.
- `number`: u32. Permanent number, 1 to 64, matching setting.number.
- `model`: `model`. runtime model, whole.
- `shown`: text[4,096], or null. Its name for showing, or null.
- `description`: text[512], or null. Its description, or null.
- `picker`: bool. Whether a model picker shows it.
- `withdrawn`: bool. Whether it is withdrawn; its number and name stay.
- `source`: `"stock"`, `"list"` or `"person"`. Where the model came from. `"stock"`: binary. `"list"`: provider list. `"person"`: person.
- `estimated`: bool. Whether its prices are provider defaults.

## model
Runtime model, whole in model settings. Charter members except `effort` must match its configured row, else `reference`.
- `provider`: u32. configured LLM provider that serves it; 1 when absent.
- `dialect`: u32. Configured model protocol.
- `endpoint`: u32. Configured serving endpoint's number.
- `program`: text[32], or null. Configured program running it, or null for none.
- `name`: text[4,096]. Provider's model name.
- `window`: u32. most tokens of input it reads.
- `output`: u32. Most tokens per completion, thinking included.
- `input_price`: u64. Fresh input cost: spend units per `price_unit` tokens.
- `cached_price`: u64. Cached input cost: spend units per `price_unit` tokens.
- `cache_write_price`: u64, or null. Cache write cost per `price_unit` tokens; null uses fresh-input price.
- `output_price`: u64. Output cost: spend units per `price_unit` tokens.
- `price_unit`: u32. number of tokens each price is for.
- `efforts`: list[16] text[32]. Configured efforts, in order.
- `effort`: text[32], or null. One of `efforts`; null: default effort.

## wake
Wake by message kind; proposals, escalations and cancels always wake.
- `words`: `wake_rule`. Task and deployment words; a person's always wake it at once.
- `notices`: `wake_rule`. Notices: referenced task done/held, proposal passing through, awaited hold.
- `news`: `wake_rule`. News its connectors class as waking for it.
- `results`: `"never"`, `"each"` or `"last_or_failure"`. Its delegates' results. `"never"`: None wakes it. `"each"`: Each delegate's result wakes it. `"last_or_failure"`: last delegate's result, or any failure.
- `questions`: bool. Whether a delegate's question wakes it.
- `answers`: bool. Whether an answer to its own question wakes it.
- `timers`: bool. Whether its timers wake it.

## wake_rule
Wake by message kind: a word or one-member object.
- `"never"`: It waits for the task's next run.
- `"immediate"`: It wakes the task at once.
- `batch`: `wake_batch`. Once enough arrive or the oldest waits long enough.
- `paced`: nanoseconds. At most once per span from the last run's start; first after a quiet span wakes at once. Above zero, else `shape` refusal.

## wake_batch
Message batch waking a task once.
- `count`: u32. Wake at this count; above zero, else `shape` refusal.
- `age`: nanoseconds. Or at this age of the oldest; above zero, else `shape` refusal.

## template
Executor and defaults for new tasks.
- `resources`: list[32] `symbol`. Optional, default []: distinct resource-symbol pairs resolved for the new task. Forge connector 1, symbol 1: its own goal branch.
- `number`: u32. Template number matching `setting.number`; chat: 1, goal: 2.
- `name`: `text`. Its name, as clients show it.
- `executor`: `executor`. Who runs the task.
- `contract`: `contract`. result the task must give.
- `shallower`: u32. Delegation depth below creator role; 0 makes the template unusable.
- `budget`: `budget`. What the task may spend.
- `deadline`: `deadline`. By when the task must end.
- `tracked`: u32, or null. Its default priority as tracked work, lower first, or null for untracked.
- `flags`: `flags`. How the task behaves and is kept.
- `label`: u32. Number grouping ended tasks in lists; temper reads nothing else from it.
- `grace`: nanoseconds, or null. Chat closing time or null; unfinished runs then cancel. Past deployment limit: `grace` refusal.
- `llm`: `llm`, or null. models the task asks for, or null for its charter's default.

## symbol
Resource resolved at creation.
- `connector`: u16. connector that resolves the symbol; 1 is the forge.
- `symbol`: u16. connector's resource symbol; the forge's 1 is its own goal branch.

## text
text, whole.
- `bytes`: text[65,536]. text.
- `cut`: bool. Always false: a value is set whole.

## executor
Task executor.
- `"chosen"`: creator chooses a charter or procedure their role permits.
- `charter`: u32. Agent under this charter; must be offered, else `reference` refusal.

## contract
One required task result.
- `approval`: `approval`. Person's approval of exact state; agents can only fail.
- `report`: `report`. report.
- `verdict`: `verdict`. One of a set of verdicts.
- `change`: `change`. connector's result.

## approval
approval of an exact state.
- `state`: hex[32,768]. Approved state; approval must return it byte for byte.
- `part`: `part`. What a client shows the approval with.

## part
thing a connector owns, which a client shows.
- `connector`: u16. connector that owns it: 1 is the forge.
- `kind`: `"resource"`, `"pool"`, `"result"`, `"procedure"`, `"effect"`, `"outbox"` or `"hold"`. What kind of thing it is.
- `key`: `key`. Which one.

## key
Which thing: a one-member object.
- `path`: list[9] hex[32,768]. Hex resource segments: `forge`, host, owner, repository, `branch`, branch segments.
- `number`: u64. number the connector knows it by.

## report
report in words.
- `words`: u32. most bytes the report may hold.

## verdict
verdict: one of its choices, with the fields that choice asks for.
- `choices`: list[1,048,576] `choice`. choices the task gives one of.
- `cut`: bool. Always false: a value is set whole.

## choice
verdict choice.
- `code`: u32. Its number, unique among the choices.
- `words`: u32. most bytes of the verdict's explanation.
- `followups`: u32. Max verdict follow-up delegates.
- `fields`: list[32] `choice_field`. fields the verdict must fill, each named once.

## choice_field
field a verdict fills.
- `name`: hex[32,768]. Its name as hex, not empty.
- `max`: u32. most bytes of its value.

## change
connector's result: for the forge, a change.
- `connector`: u16. connector that gives the result: 1 is the forge.
- `kind`: u16. Result kind: forge 1 landed change, 2 change made but not landed.
- `words`: u32. Most report bytes, per change title and body.

## budget
Task spend.
- `"given"`: creator gives it, within their allotment.
- `amount`: u64. This much.
- `as_allowed`: `as_allowed`. Creator's remaining funds at task creation.

## as_allowed
As much as fits.
- `cap`: u64, or null. most it may be, or null for no cap.

## deadline
Task end time.
- `"none"`: No deadline.
- `"given"`: creator gives one, or none.
- `span`: nanoseconds. This long after the task is made.

## flags
How a task behaves and is kept.
- `conversation`: bool. Conversation: requester words are turns; next run uses newest revision.
- `listed`: bool. List by project and label once ended?
- `retention`: `"kept"` or `"forgotten_once_read"`. How long it is kept. `"kept"`: Until the deployment's retention horizon. `"forgotten_once_read"`: Until its requester has read it and no live task names it.
- `ended_by`: `ended_by`. What ends it.

## ended_by
How a task ends.
- `"result"`: Its run's result.
- `"requester"`: Requester alone; needs `conversation` and charter `executor`, else `shape`.
- `"standing"`: Only cancel; an agent with no conversation or tracking.
- `closer`: `closer`. named procedure ends it.

## closer
Closes the task once nothing remains to await.
- `connector`: u16. closing procedure's connector.
- `code`: u32. procedure's code.

## llm
models a task asks for.
- `account`: u32, or null. Paying account; null: any able to pay.
- `main`: `model_effort`. main model and its effort.
- `subagents`: list[64] `model_effort`. models its runs' sub-agents may use, each named once.
- `later`: `llm-later`. Optional, left out none: the choice after the task's first activation.

## llm-later
Model choice after the first activation.
- `main`: `model_effort`. later main model and its effort.
- `subagents`: list[64] `model_effort`. Absent: []. Models for later runs' sub-agents, each named once.

## model_effort
model, by name, and its effort.
- `model`: text[4,096]. configured model's name, one the task's charter offers.
- `effort`: text[32], or null. Model's effort; null if it takes none.

## policy
Project policy copied at creation; changes affect later projects.
- `escalation_role`: u32, or null. Final escalation role or null; must accept proposals and decide escalations.
- `ceiling`: `authority`. Project task ceiling; within rules, else `rules` refusal.
- `projections`: list[32] `grant`. Goal projection writes to project homes; each grant within `ceiling`.
- `period_spend`: u64. What the project may spend in a period; within `rules.period_spend`.
- `roles`: list[32] `role`. project's roles: what a party in each may do.
- `requirements`: list[32] `requirement`. Project effect needs, beside rules' requirements.
- `permissions`: list[32] `permission`. Project role given by forge collaborator permission at adoption.
- `templates`: list[16] `template_override`. project's overrides of templates.

## role
role in a project.
- `number`: u32. 0 owner, 1 maintainer, 2 member, 3 observer; others policy-defined. 0 must allow task creation, else `owner` refusal.
- `authority`: `authority`. Max party task allotment, within policy `ceiling`.
- `period_spend`: u64. Party allotment per period, within policy `period_spend`.
- `requests`: u16. Party request bits: 1 create, 2 allot, 4 accept, 8 amend, 16 cancel, 32 release, 64 move, 128 watch, 256 change policy.
- `decides`: u8. Proposals a party may decide, bits: 1 task batch, 2 effect, 4 widening, 8 amendment, 16 escalation.

## template_override
Project template override; null keeps the template's member.
- `template`: u32. template's number; offered, and once in the policy.
- `executor`: `executor`, or null. Its executor, or null.
- `budget`: `budget`, or null. Its budget, or null.
- `deadline`: `deadline`, or null. Its deadline, or null.
- `priority`: u32, or null. Its default priority, or null; an untracked template ignores it.

## permission
Forge collaborator permission and its project role at adoption.
- `connector`: u16. connector: 1 is the forge.
- `permission`: u16. permission: 0 none, 1 read, 2 write, 3 admin.
- `role`: u32. role it gives, one of the policy's `roles`.

## rules
Deployment ceilings, reread per decision. Narrowing affects existing tasks; widening never widens them.
- `ceiling`: `authority`. most any project may have.
- `period_spend`: u64. most a project's period spend may be.
- `minimum_run_spend`: u64. run starts only with more than this to spend.
- `maximum_run_spend`: u64. Maximum run funds; above `minimum_run_spend`, within `ceiling` spend.
- `implies`: list[32] `implication`. Kinds of a connector's whose grant grants other kinds too.
- `requirements`: list[32] `requirement`. What an effect needs, in every project.

## implication
One kind whose grant grants another.
- `connector`: u16. connector: 1 is the forge.
- `kind`: u16. kind whose grant also grants `implies`.
- `implies`: u16. kind it grants.

## authority
Authority: tools, models, grants, delegation, spend and deadline.
- `models`: `models`. models its runs may use.
- `tools`: u64. Engine tool bits: 1 delegate, 2 message, 4 amend/cancel/release, 8 decide/withdraw, 16 propose, 32 subscribe, 64 effect, 128 note, 256 recall, 512 forge reads, 1024 settings, 2048 asking.
- `grants`: list[1,048,576] `grant`. Permitted connector effects and reads on resources.
- `delegation`: `delegation`. delegates it may make.
- `spend`: u64. most its tree may spend.
- `deadline`: nanoseconds since 1970 UTC, or null. latest it may end, or null for no deadline.
- `notes`: u8. Note scope bits: 1 its goal, 2 its project, 4 deployment.
- `note_resources`: list[1,048,576] `resource`. resources it may write notes on.

## models
Available models.
- `"all"`: Every configured model, those configured later included.
- `named`: list[64] hex[4,096]. These models alone, each name as hex; an empty list allows none.

## grant
Leave for a connector effect or read on resources.
- `connector`: u16. connector: 1 is the forge.
- `kind`: u16. Forge kinds: 1 read, 2 push, 3 open PR, 4 merge, 5 review, 6 status, 7 comment, 8 open issue, 9 branch, 10 edit PR, 11 update PR from base, 12 close/reopen PR, 13 edit issue, 14 reviewers, 15 delete branch.
- `pattern`: `pattern`. resources it covers.

## pattern
Exact `segments`, then `last`. Repository: `forge`, host, owner, repository. Branch: append `branch`, branch segments.
- `segments`: list[9] hex[32,768]. first segments, each as hex, each matched exactly.
- `last`: `last`. segment after them.

## last
last segment: an object with one of these members.
- `exact`: hex[32,768]. Exactly this segment, as hex, and nothing beneath it.
- `open`: hex[32,768]. Hex segment prefix and descendants; empty covers every segment.

## delegation
delegates a task may make.
- `kinds`: list[22] `delegate`. Who its delegates may be.
- `tasks`: u32. Tree's lifetime task count, not concurrent.
- `depth`: u32. How far below it its tree may go; 0 allows no delegate.

## delegate
kind of delegate: a one-member object.
- `charter`: u32. agent under this charter, which must be offered.
- `procedure`: u32. This procedure: 2 is the forge's change.
- `role`: u32. person in this role.
- `job`: u32. Jobs: 1 prepares a landing, 3 merges, 4 closes a goal.

## resource
Resources it may write notes on.
- `connector`: u16. connector: 1 is the forge.
- `pattern`: `pattern`. resources.

## requirement
Requirements for a connector effect on resources, independent of authority.
- `connector`: u16. connector whose effect it constrains: 1 is the forge.
- `kind`: u16. kind of effect; implications do not apply.
- `pattern`: `pattern`. resources it applies to.
- `judge`: `judge`. Who judges it.
- `guard`: `guard`. When it is judged.
- `must_be_guarded`: bool. Require guarding as the effect acts; unguardable effects refuse.

## judge
connector that judges a requirement, and its criterion.
- `connector`: u16. judging connector: 1 is the forge.
- `requirement`: u16. Forge criteria: 1 CI passed at head, 2 head up to date with base, 3 gate, 4 approvals.
- `parameters`: u32. number the judging connector reads the criterion's terms by.

## guard
When to judge.
- `"guarded"`: As the effect is made, by the system that makes it.
- `observed`: `observed`. Before acting, from recent facts.

## observed
requirement judged from facts read before the effect.
- `freshness`: nanoseconds. How old those facts may be.

## landing-rule
Criteria for every covered landing.
- `pattern`: `landing-pattern`. `forge`, host, owner, repository, `branch`, branch segments; project rules may use only `branch` and segments across all project repositories.
- `enforce`: bool. True: every covered landing needs these criteria. False: only the change procedure checks.
- `ci`: bool. CI must have passed at the change's head.
- `up_to_date`: bool. change's head must contain its base's tip.
- `gates`: list[64] `landing-gate`. gates every change it covers asks for.
- `approvals`: list[64] `landing-approval`. Distinct people per role required to approve by forge review.
- `sets`: list[6] u8. Unique project path sets 1 to 6; default []. Omitted if empty; none for deployments.

## landing-gate
Each change needs agent review, or a repository check without CI.
- `number`: u32. agent review's or repository check's gate number.
- `blocking`: bool. Whether this gate must pass before the change lands.
- `freshness`: `"exact"` or `"clean"`. At which later heads a verdict still counts. `"exact"`: Verdict counts at its given head. `"clean"`: Verdict also counts at later heads from cleanly merged updates.

## landing-approval
Distinct project role holders approve by forge reviews.
- `role`: u32. role in the project's policy whose people approve the change.
- `people`: u32. Distinct role holders required to approve the change.
- `freshness`: `"exact"` or `"clean"`. At which later heads an approval still counts. `"exact"`: Verdict counts at its given head. `"clean"`: Verdict also counts at later heads from cleanly merged updates.

## landing-pattern
Base segments and final segment/prefix. Project rules may start at `branch` across their repositories.
- `segments`: list[9] hex[4,096]. exact base segments of the resources the rule covers.
- `last`: `landing-last`. Final exact segment or open prefix, and descendants.

## landing-last
Final resource segment, in hex.
- `exact`: hex[4,096]. One last segment equal to these bytes.
- `open`: hex[4,096]. final segment beginning with these bytes, with all below it.

## forge-charter
Charter number, 0 to 4,294,967,295. Setting 1: change; setting 2: reviews.

## dependency
Project dependency: select by project and number (1 to forge.dependencies). No stock default or automatic action.
- `project`: u32. Required project number.

## follow
Context-to-owned follow: select by project and number (1 to forge.follows); no stock default.
- `source`: `follow-source`. Context source; repository and branch fixed once accepted.
- `target`: `follow-target`. Owned target; repository and branch fixed once accepted.
- `owner`: `follow-owner`. Follow changes' owner; required.
- `start`: `"owner"` or `"code"`. When changes start; absent/empty: owner. `"owner"`: Wait for owner (default). `"code"`: Start after the previous change lands.
- `producer`: text[8,192]. Change parameter 11, C;P;R. Absent/empty: change charter and its defaults.
- `check`: `follow-check`. Required follow check.

## follow-source
Context repository. Empty branch: adopted default; empty paths: all.
- `forge`: u16. Forge number.
- `repository`: u32. Repository id on this forge.
- `branch`: text[4,096]. Absent/empty: adopted default branch.
- `paths`: list[256] text[256]. At most forge.follow_paths nonempty segment prefixes; no edge/doubled slash.

## follow-target
Owned, writable repository. Absent/empty branch: landing branch.
- `forge`: u16. Forge number.
- `repository`: u32. Repository id on this forge.
- `branch`: text[4,096]. Absent/empty: landing branch.

## follow-owner
project person or live standing role.
- `person`: u64. Project person's number.
- `task`: u64. Standing role task number.

## follow-check
Target workflow and report artifact: each required, 1 to 256 bytes.
- `workflow`: text[256]. Required nonempty target workflow file.
- `artifact`: text[256]. Required nonempty uploaded artifact name.
- `format`: `"junit"` or `"lines"`. Required report format. `"junit"`: JUnit XML. `"lines"`: Line-format report.

## template-goal
Goal design-first and close-out switches. Absent: plans, no close-out.
- `design`: bool. Coordinator delivers design before planning changes? Required.
- `close_out`: bool. Coordinator makes close-out after goal lands? Required.

## project-ways
Project display and goal start. Absent: drafts true, others false, way 0.
- `drafts`: bool. Whether chats can write drafts.
- `goals`: bool. Whether the rail shows Goals.
- `changes`: bool. Whether the rail shows Changes.
- `way`: u32. Goal's starting template: project-offered, tracked set; 0 for the first.

## project-questions
route a project's change takes when its maker names none.
- `"none"`: No question route; the stock's value.
- `"up"`: change's questions go up to its parent.

## project-classes
Classes by kind and path set; absent: Agents review's table.
- `items`: list[20] `class-item`. table's class items.
- `sets`: list[6] `path-set`. path sets its items may name.

## class-item
Kind/set, optional words, class; one wordless item per subject. Money, irreversible, rules and other always wait, with no item.
- `kind`: `"wording"`, `"retry"`, `"implementation"`, `"tests"`, `"dependency"`, `"departure"` or `"findings"`. Exactly one of kind or set.
- `set`: u8. set in this table, from 1 to 6; exactly one of kind or set.
- `words`: text[120]. Optional wording for this item, empty when absent.
- `class`: `"decide"`, `"ask"` or `"ask-away"`. Wait is never a table class. `"decide"`: Agents decide and list it. `"ask"`: Ask the person; Away may decide. `"ask-away"`: Ask the person even when away.

## path-set
Repository-relative path patterns, uniquely numbered 1 to 6.
- `number`: u8. Distinct within the table, from 1 to 6.
- `name`: text[40]. nonempty name.
- `patterns`: list[6] text[80]. Nonempty repository-relative patterns: * within segments, ** for a whole segment.

## charter-words
Charter purpose: 1 to 120 UTF-8 bytes, no controls. Defaults to instructions' first sentence. Select by charter number.

## project-words
How work runs headline and lede; absent uses the way's words.
- `headline`: text[120]. ! 1 to 120 UTF-8 bytes holding no control character.
- `lede`: text[480]. ! 1 to 480 UTF-8 bytes holding no control character.

## project-landing
Landing at the next start.
- `by`: `"queue"` or `"turn"`. ? queue by default; at the next start. `"queue"`: Default. `"turn"`: Prepare, prove, turn.
- `combine`: `landing-combine-settings`. ? main merge, other squash.
- `batch`: u16. ? 1 to forge.landing_members; default 1.
- `landing_attempts`: u8. ? default 3. At least 1.
- `gate_retries`: u8. ? default 2.
- `tier_retries`: u8. ? default 2.
- `check_timeout_ms`: u64. ? 1000 to 86400000; default none.
- `gate_timeout_ms`: u64. ? 60000 to 86400000; default 3600000.
- `verification_bound_ms`: u64. ? u64 milliseconds; default 7200000.
- `merge_job`: text[4,096]. ? relative path, no NUL or .. segments; The merge command in configuration.
- `close_job`: text[4,096]. ? relative path, no NUL or .. segments.

## project-gates
project's repository gates and tiers; at the next start.
- `repositories`: list[1,114,240] `repository-gates`. ! unique repositories of this project; whole encoding within the settings carrier; at the next start.

## landing-combine-settings
How trains combine changes.
- `main`: `"merge"` or `"squash"`. ? default merge.
- `other`: `"merge"` or `"squash"`. ? default squash.

## repository-gates
One repository's test gates and tiers.
- `forge`: u16. ! the adopted repository.
- `repository`: u32. ! the adopted repository.
- `gates`: list[1,114,240] `gate-check`. ? at most forge.gates.
- `tiers`: list[1,114,240] `tier-check`. ? at most forge.tiers.tiers.

## gate-check
workflow gate proving a landing.
- `name`: text[32]. ! letters, digits, dash or underscore; unique in the repository.
- `workflow`: text[255]. ! workflow filename without slash.
- `ignored`: list[1,114,240] text[1,024]. ? at most forge.ignored_paths, each within forge.name_bytes; whole path segments.
- `blocking`: bool. ! whether the gate blocks.
- `base`: bool. ? default false; whether the gate reads the base.
- `report`: `gate-report`. ? artifact holding failed test ids.

## gate-report
failed tests uploaded by a gate.
- `artifact`: text[32]. ! artifact name without slash.
- `format`: `"junit"` or `"lines"`. ! `"junit"`: JUnit report. `"lines"`: One test id per line.
- `fuzzy`: text[64]. ? nonempty name prefix of fuzzy tests.

## tier-check
slow workflow and the branch it follows.
- `name`: text[32]. ! letters, digits, dash or underscore; unique in the repository.
- `workflow`: text[255]. ! workflow filename without slash.
- `ignored`: list[1,114,240] text[1,024]. ? at most forge.ignored_paths, each within forge.name_bytes; whole path segments.
- `artifact`: text[32]. ! artifact name without slash.
- `format`: `"junit"` or `"lines"`. ! `"junit"`: JUnit report. `"lines"`: One test id per line.
- `branch`: text[1,048,576]. ? default landing branch; goal follows goal branches.
- `span_ms`: u64. ? milliseconds between starts; default 0, no span.
- `description`: text[120]. ? nonempty UTF-8 without control characters.

## project-areas
project's areas, at most 16.
- `areas`: list[16] `area`. ! order is kept; names are unique.

## welcome
deployment's welcome. Default: shown under the box, not shown otherwise.
- `shown`: bool. ! Shows the welcome and offers the playground.

## project-triager
project's triager.
- `owner`: `area-owner`. !

## area
named scope of project work and its owner.
- `name`: text[32]. ! 1 to 32 ASCII lowercase letters, digits or -, starting with a letter.
- `words`: text[480]. ! 1 to 480 UTF-8 bytes with no control character.
- `owner`: `area-owner`. !

## area-owner
One owner: task, party, role or charter.
- `task`: u64. live task of the project.
- `party`: u64. party with a role in the project.
- `role`: u32. Below authority.roles; 0 is the owners.
- `charter`: u32. From 1 through tasks.charters.

## limits
Core limit overrides: ascending names; whole list, empty default; no number or project.
Whole value: list[256] `limit-override`.

## temper-limits
As limits, for application limits (kind 21).
Whole value: list[256] `limit-override`.

## limit-override
One override, with a required reason.
- `name`: text[64]. ! 1-64 bytes: a-z, 0-9, _ or .; strictly ascending, no duplicates.
- `value`: u64. ! limit's unit.
- `reason`: text[120]. ! 1-120 UTF-8 bytes: why it departs from stock.
