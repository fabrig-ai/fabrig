# Settings fields

Kind: reference

JSON types, bounds and effects for settings files and `settings show`.

Read/edit/set whole: `settings show KIND N`. Refusals=shape/count/reference/rules/budget/grace/owner/linked/accounts. Spend=deployment model-price units. uN=unsigned integer 0..2^N-1; bool=true/false; t[N]/h[N]=text/lowercase hex <=N bytes; ns=nanoseconds; [N] T<=N entries. ?=optional; !=required; |null=nullable. Choices=JSON strings; A|B=alternatives; choice=described alternatives. set=setting.kind; project-policy=policy.

## file
`settings apply`/`settings propose` file.
- items:[1024] item;One item per changed setting.
- format:u64;? min 1, default 1.

## item
A `setting`; exactly one `set`/`edit`/`overlay`.
- setting:setting;setting the item changes.
- overlay:overlay;charter overlay instead of set/edit.
- remove:bool;true instead of set/edit/overlay; withdraw.
- reset:bool;true instead of set/edit/overlay; follow default.
- set:the value for `setting.kind`;Whole replacement.
- edit:choice;edit instead of a value. `removed`: Withdraw, default included; rules cannot be removed. `reset`: Erase change; follow default.

## setting
Kind and number, or project for project settings.
- kind:`model|worker|provider|charter|template|policy|project-policy|rules|landing-rule|forge-charter|template-goal|follow|dependency|project-ways|project-questions|project-classes|charter-words|project-words|project-landing|project-gates|project-areas|project-triager|agents|welcome|limits|temper-limits`;setting's kind. `project-policy`: Project policy: `set` only, once per file alongside other settings. `project-landing`: Next start. `project-gates`: Next start.
- number:u32;Number >=1; omit for rules/welcome/limit kinds and unnumbered project settings.
- project:u32;! project settings/follow/dependency. Absent: charter-words/welcome/agents/limit kinds. Landing rule: >=1; absent: deployment-wide.

## charter
Task-creation revision's run policy.
- rest:bool;Offer remaining catalogue models after named choices; default false.
- number:u32;Same as `setting.number`.
- instructions:t[1048576];Agent's first read, before task brief.
- waiting:ns;Idle wait for words/news before parking.
- resume:bool;Resume any task transcript rather than start fresh.
- turns:u32;Most model turns; more than `reserve_turns`, else `budget` refusal.
- time:ns;Run cancellation time; more than `reserve_time`, else `budget` refusal.
- reserve_turns:u32;Of `turns`, finish/report reserve.
- reserve_time:ns;Of `time`, finish/report reserve. With `modify`: >= check_timeout ms in each applicable current or held charter, else worker admission refuses. Held-revision recovery: settings help.
- model:model;Default: first payable of `model`/`alternatives`.
- alternatives:[32] model;After `model`, each once; sub-agents may use them.
- inspect:bool;May read/list/search workspace?
- modify:bool;May write/edit/remove workspace files?
- shell:bool;May run workspace commands?
- agents:bool;May open sub-agents with at most its own tools?
- call_timeout:ns;Longest call to temper's tools.
- tools:u64;Host families need charter/task leave. Asking (2048): see/document; with settings (1024), propose_settings too. Without asking, 1024 grants settings tools. Default 1023 excludes both.
- hosts:choice;Run hosts. `workers`: Deployment workers only. `engine`: Engine slots only. `both`: Either.
- spend:u64;>= (reserve_turns + 1) costliest default-model completions, else budget refusal. Runs get task funds left, half if delegation is allowed; <=rules.maximum_run_spend.
- wake:wake|null;Wake for tasks made under this charter if batch names none. Null/absent: jig's default, all rules `"immediate"`, `results` `"last_or_failure"`, questions, answers and timers waking.
- documents:[8] document;Brief documents; at most `tasks.charter_documents`, else `shape` refusal.
- parts:[8] charter-part;Ordered parts by run reasons; <=tasks.charter_parts; default [].

## overlay
Optional default replacements/added text: charters or literal instructions/parts, not both.
Same as `charter`, replacing each default whole: `rest`, `waiting`, `resume`, `turns`, `time`, `reserve_turns`, `reserve_time`, `model`, `alternatives`, `inspect`, `modify`, `shell`, `agents`, `call_timeout`, `tools`, `hosts`, `spend`, `documents`.

- instructions:t[1048576];? added text; absent=empty; excludes charters.
- parts:[8] charter-part;? added text; absent=empty; excludes charters.
- charters:[4294967295] t[4294967295];Optional nonempty settings-file-relative directories replace instructions/parts; charters 5/2 only. Markdown counts in file bound.

## charter-part
Instructions after shared text, read when run reasons meet `on`.
- on:[14] `first|question|question_answered|question_ended|question_owed|escalation|proposal|person_stalled|result|notice|nothing_awaited|news|timer|words`;Nonempty unique names; refuses those offered to every run.
- instructions:t[1048576];Append after shared instructions when chosen.

## document
Brief file at preparation branch head.
- forge:t[32768];Resource `host` with port: `see settings`/`project repositories` or `temper ctl DIR forge list`; not forge number/web address.
- repository:t[65537];OWNER/NAME; if no project adopts it, the section says so.
- branch:t[131075];Branch head to read; segments joined by `/`.
- path:t[256];Repository-relative file path, <=256 bytes.
- heading:t[80]|null;Heading: read until next peer/ancestor heading; null reads whole file.

## provider
LLM provider/endpoints; next engine start. Remove withdraws its name; reset needs a default.
- number:u32;Same as setting.number; 1..16.
- name:t[256];!
- endpoint:t[4096];?
- direct:t[4096];?
- signed_in_plan:bool;! with sign_in; absent=false.
- manage_usage:t[2048];? public HTTPS.
- keep:[16] t[64];? needs endpoint; * and ? patterns.
- drop:[16] t[64];Needs keep; absent=[].
- sources:provider-sources;!
- endpoints:[64] provider-endpoint;!
- withdrawn:bool;Absent=false.

## provider-sources
Sources optional; {}=none. Programs unique: 1..32 lowercase ASCII letters/digits/-/_.
- keys:bool;Absent=false.
- programs:[64] t[32];Absent=[].
- sign_in:provider-registration;?

## provider-registration
Public sign-in, no secret. issuer/client/authorization/token/keys nonempty. redirect_path: no leading /, query or fragment. TLS needs server_name; plaintext omits server_name/trust.
- issuer:t[4096];!
- client:t[4096];!
- authorization:t[4096];!
- token:t[4096];!
- keys:t[4096];!
- redirect_path:t[4096];!
- scopes:[64] t[4096];!
- must_grant:[64] t[4096];!
- name_hint:t[4096];?
- revocation:t[4096];?
- resource:t[4096];?
- redirect_port:u16;?
- transport:`tls|plaintext`;!
- server_name:t[4096];!
- trust:t[4096];?

## provider-endpoint
provider implicit; program optional. TLS needs server_name; plaintext no server_name/trust, loopback only. _ms=milliseconds; 64 endpoints total.
- name:t[4096];!
- number:u32;!
- dialect:u32;!
- program:t[4096];?
- address:provider-address;!
- transport:`tls|plaintext`;!
- server_name:t[4096];!
- trust:t[4096];?
- headers:[16] provider-header;!
- connect_ms:u64;!
- handshake_ms:u64;!

## provider-address
One of socket/host; host needs port.
- socket:t[128];IPv4:PORT or [IPv6]:PORT.
- host:t[253];DNS name.
- port:u16;With host.

## provider-header
No credential headers (case-insensitive): Authorization, Proxy-Authorization, X-Api-Key, Api-Key, Cookie.
- name:t[4096];!
- value:t[4096];!

## worker
Name/places/secret digest; remove withdraws name; reset needs a default.
- number:u32;Absent: the item's number.
- name:t[128];ASCII letters/digits/-/_/., first alphanumeric; also bounded by fleet.worker_name_bytes.
- places:u32;Concurrent runs: 1..fleet.slots; effective next opening.
- digest:h[32]|null;Secret's SHA-256: 64 lowercase hex digits; absent only over a digest-less in-process default.
- withdrawn:bool;Absent: false; true is read-only. Withdraw by a removed edit.

## catalogue
Numbered model and its display facts.
- number:u32;Keeps setting.number (1..64).
- model:model;Whole model.
- shown:t[4096]|null;Display name.
- description:t[512]|null;Description.
- picker:bool;Show in picker?
- withdrawn:bool;Withdrawn? Number and name stay.
- source:choice;Origin. `stock`: binary. `list`: provider list. `person`: person.
- estimated:bool;Provider default prices?

## model
Whole runtime model. Spend units per `price_unit` tokens. Charter fields except `effort` match config (`reference`).
- provider:u32;Serving LLM provider; absent: 1.
- dialect:u32;Model protocol.
- endpoint:u32;Endpoint number.
- program:t[32]|null;Serving program, if any.
- name:t[4096];Provider's model name.
- window:u32;Input token limit.
- output:u32;Completion tokens, thinking included.
- input_price:u64;Fresh input price.
- cached_price:u64;Cached input price.
- cache_write_price:u64|null;Cache write price; null uses fresh-input price.
- output_price:u64;Output price.
- price_unit:u32;Tokens per price.
- efforts:[16] t[32];Efforts, in order.
- effort:t[32]|null;One of `efforts`; null: default.
- speeds:[4] setting-speed;Ordered; absent: [].
- speed:t[32]|null;Default from `speeds` (else `shape`); absent/null: standard.

## setting-speed
Faster speed, model price unit.
- name:t[32];Provider speed name.
- also:[2] t[32]|null;Reported aliases; absent/null: [].
- input_price:u64;Fresh input cost.
- cached_price:u64;Cached input cost.
- cache_write_price:u64|null;Cache writes; absent/null: input price.
- output_price:u64;Output cost.

## wake
Wake by message kind; proposals, escalations and cancels always wake.
- words:wake_rule;Task/deployment words; a person's wake at once.
- notices:wake_rule;Referenced task done/held; passing proposal; awaited hold.
- news:wake_rule;News its connectors class as waking.
- results:choice;delegates' results. `never`: None wakes it. `each`: Each delegate's result wakes it. `last_or_failure`: last delegate's result, or any failure.
- questions:bool;Wake for delegate's question?
- answers:bool;Wake for answer to own question?
- timers:bool;Wake for its timers?

## wake_rule
Wake by message kind: a word or one-member object.
- `never`:It waits for the task's next run.
- `immediate`:It wakes the task at once.
- batch:wake_batch;Once enough arrive or the oldest waits long enough.
- paced:ns;At most once per positive span from run start. First after quiet wakes now. Nonpositive: `shape`.

## wake_batch
Message batch waking a task once.
- count:u32;Wake at this count; above zero, else `shape` refusal.
- age:ns;Or at this age of the oldest; above zero, else `shape` refusal.

## template
Executor and defaults for new tasks.
- resources:[32] symbol;Optional, default []: distinct resource-symbol pairs resolved for new task. Forge connector 1, symbol 1: its own goal branch.
- number:u32;Template number matching `setting.number`; chat: 1, goal: 2.
- name:text;name, as clients show it.
- executor:executor;Who runs the task.
- contract:contract;result the task must give.
- shallower:u32;Delegation depth below creator role; 0 makes the template unusable.
- budget:budget;What the task may spend.
- deadline:deadline;By when the task must end.
- tracked:u32|null;default priority as tracked work, lower first, or null for untracked.
- flags:flags;How the task behaves and is kept.
- label:u32;Number grouping ended tasks in lists; temper reads nothing else from it.
- grace:ns|null;Chat closing time/null; unfinished runs cancel then. Beyond deployment limit: `grace` refusal.
- llm:llm|null;Asked models; null: charter's default.

## symbol
Resource resolved at creation.
- connector:u16;Symbol resolver; 1: forge.
- symbol:u16;connector's resource symbol; the forge's 1 is its own goal branch.

## text
text, whole.
- bytes:t[65536];text.
- cut:bool;Always false: set whole.

## executor
Task executor.
- `chosen`:Creator chooses a charter/procedure their role permits.
- charter:u32;Agent under this charter; must be offered, else `reference` refusal.

## contract
One required task result.
- approval:approval;Person's approval of exact state; agents can only fail.
- report:report;report.
- verdict:verdict;One of a set of verdicts.
- change:change;connector's result.

## approval
Approval of an exact state.
- state:h[32768];Approved state; approval must return it byte for byte.
- part:part;What a client shows the approval with.

## part
Connector-owned thing shown by a client.
- connector:u16;connector that owns it: 1 is the forge.
- kind:`resource|pool|result|procedure|effect|outbox|hold`;What kind of thing it is.
- key:key;Which one.

## key
Which thing: a one-member object.
- path:[9] h[32768];Hex resource segments: `forge`, host, owner, repository, `branch`, branch segments.
- number:u64;number the connector knows it by.

## report
Report in words.
- words:u32;most bytes the report may hold.

## verdict
One verdict choice and its required fields.
- choices:[1048576] choice;choices the task gives one of.
- cut:bool;Always false: set whole.

## choice
A verdict choice.
- code:u32;number, unique among the choices.
- words:u32;most bytes of the verdict's explanation.
- followups:u32;Max verdict follow-up delegates.
- fields:[32] choice_field;fields the verdict must fill, each named once.

## choice_field
A field a verdict fills.
- name:h[32768];name as hex, not empty.
- max:u32;most bytes of its value.

## change
A connector's result: for the forge, a change.
- connector:u16;connector that gives the result: 1 is the forge.
- kind:u16;Result kind: forge 1 landed change, 2 change made but not landed.
- words:u32;Most report bytes, per change title and body.

## budget
Task spend.
- `given`:creator gives it, within their allotment.
- amount:u64;This much.
- as_allowed:as_allowed;Creator's remaining funds at task creation.

## as_allowed
As much as fits.
- cap:u64|null;most it may be, or null for no cap.

## deadline
Task end time.
- `none`:No deadline.
- `given`:creator gives one, or none.
- span:ns;This long after the task is made.

## flags
Task behavior and retention.
- conversation:bool;Conversation: requester words are turns; next run uses newest revision.
- listed:bool;List by project and label once ended?
- retention:choice;Retention. `kept`: Deployment retention horizon. `forgotten_once_read`: Until read by requester and named by no live task.
- ended_by:ended_by;What ends it.

## ended_by
Task ending.
- `result`:run's result.
- `requester`:Requester alone; needs `conversation` and charter `executor`, else `shape`.
- `standing`:Only cancel; an agent with no conversation or tracking.
- closer:closer;named procedure ends it.

## closer
Closes task when nothing remains to await.
- connector:u16;closing procedure's connector.
- code:u32;procedure's code.

## llm
Task model choice.
- account:u32|null;Paying account; null: any able to pay.
- main:model_effort;Main model choice.
- subagents:[64] model_effort;Sub-agent models, each named once.
- later:llm-later;Absent: none. Choice after first activation.

## llm-later
Model choice after the first activation.
- main:model_effort;Later main model choice.
- subagents:[64] model_effort;Absent: []. Later sub-agent models, each named once.

## model_effort
model, by name, with its effort and speed.
- model:t[4096];Named model the charter offers.
- effort:t[32]|null;Model's effort; null if it takes none.
- speed:t[32];declared speed by name or `standard`; absent: the charter's speed for the model.

## policy
Project policy copied at creation; changes affect later projects.
- escalation_role:u32|null;Final escalation role or null; must accept proposals and decide escalations.
- ceiling:authority;Project task ceiling; within rules, else `rules` refusal.
- projections:[32] grant;Goal projection writes to project homes; each grant within `ceiling`.
- period_spend:u64;What the project may spend in a period; within `rules.period_spend`.
- roles:[32] role;project's roles: what a party in each may do.
- requirements:[32] requirement;Project effect needs, beside rules' requirements.
- permissions:[32] permission;Project role given by forge collaborator permission at adoption.
- templates:[16] template_override;project's overrides of templates.

## role
role in a project.
- number:u32;0 owner, 1 maintainer, 2 member, 3 observer; others policy-defined. 0 must allow task creation, else `owner` refusal.
- authority:authority;Max party task allotment, within policy `ceiling`.
- period_spend:u64;Party allotment per period, within policy `period_spend`.
- requests:u16;Party request bits: 1 create, 2 allot, 4 accept, 8 amend, 16 cancel, 32 release, 64 move, 128 watch, 256 change policy.
- decides:u8;Proposals a party may decide, bits: 1 task batch, 2 effect, 4 widening, 8 amendment, 16 escalation.

## template_override
Project template override; null keeps the template's member.
- template:u32;template's number; offered, and once in the policy.
- executor:executor|null;executor, or null.
- budget:budget|null;budget, or null.
- deadline:deadline|null;deadline, or null.
- priority:u32|null;default priority, or null; an untracked template ignores it.

## permission
Forge collaborator permission and its project role at adoption.
- connector:u16;connector: 1 is the forge.
- permission:u16;permission: 0 none, 1 read, 2 write, 3 admin.
- role:u32;role it gives, one of the policy's `roles`.

## rules
Deployment ceilings read each decision; narrow existing tasks, never widen them.
- ceiling:authority;most any project may have.
- period_spend:u64;most a project's period spend may be.
- minimum_run_spend:u64;run starts only with more than this to spend.
- maximum_run_spend:u64;Maximum run funds; above `minimum_run_spend`, within `ceiling` spend.
- implies:[32] implication;Kinds of a connector's whose grant grants other kinds too.
- requirements:[32] requirement;What an effect needs, in every project.

## implication
One kind whose grant grants another.
- connector:u16;connector: 1 is the forge.
- kind:u16;kind whose grant also grants `implies`.
- implies:u16;kind it grants.

## authority
Authority: tools, models, grants, delegation, spend and deadline.
- models:models;models its runs may use.
- tools:u64;Engine tool bits: 1 delegate, 2 message, 4 amend/cancel/release, 8 decide/withdraw, 16 propose, 32 subscribe, 64 effect, 128 note, 256 recall, 512 forge reads, 1024 settings, 2048 asking.
- grants:[1048576] grant;Permitted connector effects and reads on resources.
- delegation:delegation;delegates it may make.
- spend:u64;most its tree may spend.
- deadline:ns since 1970 UTC|null;latest it may end, or null for no deadline.
- notes:u8;Note scope bits: 1 its goal, 2 its project, 4 deployment.
- note_resources:[1048576] resource;resources it may write notes on.

## models
Models offered.
- `all`:Every configured model, those configured later included.
- named:[64] h[4096];These models alone, each name as hex; an empty list allows none.

## grant
Leave for a connector effect or read on resources.
- connector:u16;connector: 1 is the forge.
- kind:u16;Forge kinds: 1 read; 2 push; 3/10/11/12 PR open/edit/update from base/close-reopen; 4 merge; 5 review; 6 status; 7 comment; 8 open issue; 9 branch; 13 edit issue; 14 reviewers; 15 delete branch.
- pattern:pattern;resources it covers.

## pattern
Exact `segments`, then `last`. Repository: `forge`/host/owner/repository; branch: append `branch`/segments.
- segments:[9] h[32768];first segments, each as hex, each matched exactly.
- last:last;segment after them.

## last
Last segment: a one-member object.
- exact:h[32768];Exactly this segment, as hex, and nothing beneath it.
- open:h[32768];Hex segment prefix and descendants; empty covers every segment.

## delegation
The delegates a task may make.
- kinds:[22] delegate;Who its delegates may be.
- tasks:u32;Tree's lifetime task count, not concurrent.
- depth:u32;How far below it its tree may go; 0 allows no delegate.

## delegate
A kind of delegate: a one-member object.
- charter:u32;agent under this charter, which must be offered.
- procedure:u32;This procedure: 2 is the forge's change.
- role:u32;person in this role.
- job:u32;Jobs: 1 prepares a landing, 3 merges, 4 closes a goal.

## resource
Resources it may write notes on.
- connector:u16;connector: 1 is the forge.
- pattern:pattern;resources.

## requirement
Requirements for a connector effect on resources, independent of authority.
- connector:u16;connector whose effect it constrains: 1 is the forge.
- kind:u16;kind of effect; implications do not apply.
- pattern:pattern;resources it applies to.
- judge:judge;Who judges it.
- guard:guard;When it is judged.
- must_be_guarded:bool;Require guarding as the effect acts; unguardable effects refuse.

## judge
Requirement's judging connector and criterion.
- connector:u16;judging connector: 1 is the forge.
- requirement:u16;Forge criteria: 1 CI passed at head, 2 head up to date with base, 3 gate, 4 approvals.
- parameters:u32;number the judging connector reads the criterion's terms by.

## guard
When to judge.
- `guarded`:As the effect is made, by the system that makes it.
- observed:observed;Before acting, from recent facts.

## observed
Requirement judged from facts read before the effect.
- freshness:ns;How old those facts may be.

## landing-rule
Criteria for every covered landing.
- pattern:landing-pattern;`forge`/host/owner/repository/`branch`/branch segments. Project rules may start at `branch`, matching it in every project repository.
- enforce:bool;True: criteria for every covered landing. False: change procedure alone checks.
- ci:bool;CI must have passed at the change's head.
- up_to_date:bool;change's head must contain its base's tip.
- gates:[64] landing-gate;gates every change it covers asks for.
- approvals:[64] landing-approval;Distinct people per role required to approve by forge review.
- sets:[6] u8;Unique project path sets 1-6; default []. Empty omitted; none for deployments.

## landing-gate
Each change needs agent review, or a repository check without CI.
- number:u32;agent review's or repository check's gate number.
- blocking:bool;Whether this gate must pass before the change lands.
- freshness:choice;Heads where a verdict counts. `exact`: Given head only. `clean`: Also later heads from cleanly merged updates.

## landing-approval
Distinct project role holders approve by forge reviews.
- role:u32;role in the project's policy whose people approve the change.
- people:u32;Distinct role holders required to approve the change.
- freshness:choice;Heads where an approval counts. `exact`: Given head only. `clean`: Also later heads from cleanly merged updates.

## landing-pattern
Base segments and final segment/prefix. Project rules may start at `branch` across their repositories.
- segments:[9] h[4096];exact base segments of the resources the rule covers.
- last:landing-last;Final exact segment or open prefix, and descendants.

## landing-last
Final resource segment, in hex.
- exact:h[4096];One last segment equal to these bytes.
- open:h[4096];final segment beginning with these bytes, with all below it.

## forge-charter
Charter number, 0 to 4,294,967,295. Setting 1: change; setting 2: reviews.

## dependency
Dependency by project/number (1 to forge.dependencies); no stock default or automatic action.
- project:u32;! project.

## follow
Context-to-owned follow by project/number (1 to forge.follows); no stock default.
- source:follow-source;Context source; repository and branch fixed once accepted.
- target:follow-target;Owned target; repository and branch fixed once accepted.
- owner:follow-owner;Follow changes' owner; required.
- start:choice;When changes start; absent/empty: owner. `owner`: Wait for owner (default). `code`: Start after the previous change lands.
- producer:t[8192];Change parameter 11, C;P;R. Absent/empty: change charter and its defaults.
- check:follow-check;! follow check.

## follow-source
Context repository. Empty branch: adopted default; empty paths: all.
- forge:u16;Forge number.
- repository:u32;Repository id on this forge.
- branch:t[4096];Absent/empty: adopted default branch.
- paths:[256] t[256];At most forge.follow_paths nonempty segment prefixes; no edge/doubled slash.

## follow-target
Owned, writable repository. Absent/empty branch: landing branch.
- forge:u16;Forge number.
- repository:u32;Repository id on this forge.
- branch:t[4096];Absent/empty: landing branch.

## follow-owner
project person or live standing role.
- person:u64;Project person's number.
- task:u64;Standing role task number.

## follow-check
Target workflow and report artifact: each required, 1 to 256 bytes.
- workflow:t[256];! nonempty target workflow file.
- artifact:t[256];! nonempty uploaded artifact name.
- format:choice;! report format. `junit`: JUnit XML. `lines`: Line-format report.

## template-goal
Goal design-first/close-out switches. Absent: plans, no close-out.
- design:bool;Coordinator delivers design before planning changes? Required.
- close_out:bool;Coordinator makes close-out after goal lands? Required.

## project-ways
Project display and goal start. Absent: drafts true, others false, way 0.
- drafts:bool;Whether chats can write drafts.
- goals:bool;Whether the rail shows Goals.
- changes:bool;Whether the rail shows Changes.
- way:u32;Goal's starting template: project-offered, tracked set; 0 for the first.

## project-questions
route a project's change takes when its maker names none.
- `none`:No question route; the stock's value.
- `up`:change's questions go up to its parent.

## project-classes
Classes by kind and path set; absent: Agents review's table.
- items:[20] class-item;table's class items.
- sets:[6] path-set;path sets its items may name.

## class-item
Kind/set, optional words, class; one wordless item per subject. Money, irreversible, rules and other always wait, with no item.
- kind:`wording|retry|implementation|tests|dependency|departure|findings`;Exactly one of kind or set.
- set:u8;set in this table, from 1 to 6; exactly one of kind or set.
- words:t[120];? item wording; absent=empty.
- class:choice;Wait is never a table class. `decide`: Agents decide and list it. `ask`: Ask the person; Away may decide. `ask-away`: Ask the person even when away.

## path-set
Repository-relative path patterns, uniquely numbered 1 to 6.
- number:u8;Distinct within the table, from 1 to 6.
- name:t[40];nonempty name.
- patterns:[6] t[80];Nonempty repository-relative patterns: * within segments, ** for a whole segment.

## charter-words
Purpose by charter number: 1-120 UTF-8 bytes, no controls; default: instructions' first sentence.

## project-words
How work runs headline and lede; default: way's words.
- headline:t[120];! 1-120 UTF-8 bytes, no controls.
- lede:t[480];! 1-480 UTF-8 bytes, no controls.

## project-landing
Landing at the next start.
- by:choice;? `queue`: Default. `turn`: Prepare, prove, turn.
- wait:choice;? `cleared`: Default. `landed`: Ends at landing.
- combine:landing-combine-settings;? main merge, other squash.
- batch:u16;? 1 to forge.landing_members; default 1.
- landing_attempts:u8;? min 1; default 3.
- gate_retries:u8;? default 2.
- tier_retries:u8;? default 2.
- flakes:landing-flakes;? default rerun at checkout.rerun_processes.
- check_timeout_ms:u64;? 1000 to 86400000; default none.
- gate_timeout_ms:u64;? 60000 to 86400000; default 3600000.
- verification_bound_ms:u64;? ms; default 7200000.
- merge_job:t[4096];? relative, no NUL or .. segments; see merge command in configuration.
- close_job:t[4096];? relative, no NUL or .. segments.
- repairs:repairs;? no repairs by default; needs wait landed.

## landing-flakes
String `none` or object `{"rerun": N}`.
- `none`:No known-flake gate reruns or delivery flake findings.
- rerun:u32;0 to checkout.rerun_processes.

## repairs
Repair goals.
- repair_goals:u8;? 1 to 16; default 2.
- repair_attempts:u8;? 1 to 8; default 2.
- repair_deadline_ms:u64;? 3600000 to 604800000; default 86400000 (24h).
- goal:repair-goal;!

## repair-goal
Agent goal.
- charter:u32;! deployment agent charter.
- llm:llm|null;? template llm; null/absent: charter default.
- priority:u32;! tracked priority.
- budget:u64;! above rules.minimum_run_spend.

## project-gates
Gates and tiers at next start.
- repositories:[1114240] repository-gates;! unique project repos; encoding fits settings carrier.

## landing-combine-settings
How trains combine changes.
- main:`merge|squash`;? default merge.
- other:`merge|squash`;? default squash.

## repository-gates
A repository's test gates and tiers.
- forge:u16;! the adopted repository.
- repository:u32;! the adopted repository.
- gates:[1114240] gate-check;? at most forge.gates.
- tiers:[1114240] tier-check;? at most forge.tiers.tiers.

## gate-check
A workflow gate proving a landing.
- name:t[32];! letters/digits/-/_; unique per repository.
- workflow:t[255];! workflow file, no slash.
- ignored:[1114240] t[1024];? <=forge.ignored_paths whole-segment paths, each <=forge.name_bytes.
- blocking:bool;! blocks landing.
- base:bool;? reads base; default false.
- report:gate-report;? artifact holding failed test ids.

## gate-report
Failed tests uploaded by a gate.
- artifact:t[32];! artifact, no slash.
- format:choice;! `junit`: JUnit report. `lines`: One test id per line.
- fuzzy:t[64];? nonempty fuzzy-test name prefix.

## tier-check
A slow workflow and the branch it follows.
- name:t[32];! letters/digits/-/_; unique per repository.
- workflow:t[255];! workflow file, no slash.
- ignored:[1114240] t[1024];? <=forge.ignored_paths whole-segment paths, each <=forge.name_bytes.
- artifact:t[32];! artifact, no slash.
- format:choice;! `junit`: JUnit report. `lines`: One test id per line.
- branch:t[1048576];? default landing branch; `goal`: goal branches.
- span_ms:u64;? ms between starts; default 0, no span.
- description:t[120];? nonempty UTF-8, no controls.

## project-areas
A project's areas, at most 16.
- areas:[16] area;! order is kept; names are unique.

## welcome
Deployment welcome; default shown in box, hidden elsewhere.
- shown:bool;! Shows the welcome and offers the playground.

## project-triager
A project's triager.
- owner:area-owner;!

## area
A named scope of project work and its owner.
- name:t[32];! 1-32 ASCII lowercase letters/digits/-, first a letter.
- words:t[480];! 1-480 UTF-8 bytes, no controls.
- owner:area-owner;!

## area-owner
Owner: task, party, role or charter.
- task:u64;live task of the project.
- party:u64;party with a role in the project.
- role:u32;Below authority.roles; 0 is the owners.
- charter:u32;From 1 through 64.

## agents
Agent profile and endpoint limits, taken at each worker's next start.
- profile:agent-profile;Overrides standard profile.
- endpoints:agent-endpoints;By endpoint name.
- fallback:agent-limits;For unlisted endpoints.

## agent-profile
Map: <=128 unique names (<=64 UTF-8 bytes) to unsigned integers within each type; profile declared/policy overrides over standard.

## agent-endpoints
Map: <=64 unique endpoint names (<=64 UTF-8 bytes) to agent-limits.

## agent-limits
Endpoint limits; all integers >0.
- window:u32;Input tokens.
- output:u32;Output tokens, reasoning included.
- reasoning_item:u32;Reasoning item bytes.
- head_ms:u64;Milliseconds.
- idle_ms:u64;Milliseconds.
- oversized:choice;Oversized reasoning behavior. `fail`: Refuse an oversized reasoning item. `drop`: Drop an oversized reasoning item.

## limits
Core limits: whole list; default=[]; no number/project.
Value: list[256] limit-override.

## temper-limits
Application kind 22; as limits.
Value: list[256] limit-override.

## limit-override
Reasoned override.
- name:t[64];! 1-64 bytes: a-z/0-9/_/.; ascending, unique.
- value:u64;! limit's unit.
- reason:t[120];! 1-120 UTF-8 bytes: why override stock.

## Formats

Newest format `temper ctl` reads: 1; no `format`: 1.
