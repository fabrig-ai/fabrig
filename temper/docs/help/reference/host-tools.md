# Host tools

Kind: reference

Every tool temper adds to its runs' own: what it does, as a run reads it, whether it reads or writes, and the members of its input.

A run calls only the tools in its own list: a tool here that your list lacks is not yours.

Members are optional unless marked required. Numeric bounds in brackets are inclusive.

File, pull-file and compare replies fit min(core.call_answer_bytes minus tool name, agent.json profile.declared.tool_payload); raising the former cannot override the latter. Oversized comparisons give measured size and recovery. Forge refusals name the bound and exact size or at least bytes read; unknown checks say an unnamed bound, and figureless forge bounds say a bound of the forge's. All refusals say how to read on. Fitted file prefixes keep [file truncated] and size/recovery feedback if room permits. max_bytes selects a raw prefix without offset; cuts are incomplete. If metadata leaves no content room, shorten the path or raise the allowance. Oversized pull-file pages keep paths as room permits, distinguishing omitted contents from absent sides. Recover omitted paths by listing head/base directories; read paths at head/base commits, resolving base separately. Later pages list other files.

## finding

file: kind severity title body [area]; hand: number area why; route: number goal; backlog: number; drop: number reason; list: [state] [mine]; show: number

A write.
Offered where the run's charter names the delegate family and its authority holds it.

- `act`: one of `file|hand|route|backlog|drop|list|show`; required.
- `kind`: one of `qa|flake|red|review|feedback`.
- `severity`: one of `low|medium|high`.
- `area`: text.
- `title`: text.
- `body`: text.
- `number`: integer.
- `why`: text.
- `goal`: integer.
- `reason`: text.
- `state`: text.
- `mine`: boolean.

## red

A red is a failure a goal's branch carries that no live change owns. record: key at why; end: key why; list

A write.
Offered where the run's charter names the delegate family and its authority holds it.

- `act`: one of `record|end|list`; required.
- `key`: text.
- `at`: text. 40 or 64 lowercase hex digits of the check's base.
- `why`: text.

## follow

start: follow commit; hold/skip: follow commit reason; merge: follow commits (2+, queue order); list: project's follows; show: follow. Acts are owner's.

A write.
Offered where the run's charter names the delegate family and its authority holds it.

- `act`: one of `start|hold|skip|merge|list|show`; required.
- `follow`: integer [1, 4294967295].
- `commit`: text.
- `commits`: array.
- `reason`: text.

## decided

Ask whether you may decide for your person. Carry on only if decided; otherwise the answer says how to ask.

A write.
Offered where the run's charter names the control family and its authority holds it.

- `question`: text; required. Question, up to 400 bytes.
- `options`: array; required. 2-6 options, each up to 120 bytes.
- `took`: integer [0, 5]; required. Chosen index, from 0.
- `why`: text; required. Why, up to 400 bytes.
- `kind`: one of `wording|retry|implementation|tests|dependency|departure|money|irreversible|rules|other|findings`; required.
- `paths`: array. Up to 16 repository-relative paths, each 1-256 bytes, no leading /.
- `under`: integer [1, 20]. Covering item in the project's classes.

## goal

Have one goal tracked, only for work your person wants agreed first (set agree) or a survey that changes nothing. For a change to a repository, call change.

A write.
Offered where the run's charter names the delegate family and its authority holds it.

- `words`: text; required.
- `advisor`: boolean. Set by a goal alone, for a survey that changes nothing.
- `agree`: boolean. Set by the person's chat when they want the work agreed first: the goal asks its plan before any change.

## change

Have one change written on a branch of a repository of this project, opened as a pull request and merged once its checks pass; you hear its result.

A write.
Offered where the run's charter names the delegate family and its authority holds it.

- `words`: text. Change's complete brief.
- `repository`: text. owner/name; omit if sole writable.
- `name`: text. A name for after.
- `landing`: text. goal/name; omit: held goal/base.
- `branch`: text. mine: goal into base; omit: a new. the `branch` form, under Shared forms.
- `merge`: text. Catch-up: 40/64 hex commit.
- `after`: array. Live delegate IDs/earlier names.
- `producer`: integer >=1. Producer charter; omit: forge default.
- `llm`: object. producer/review:{main:{model,effort},sub_agents:[{model,effort}]}.
- `changes`: array. Batch: put all options inside each item.
  - `words`: text; required. Change's complete brief.
  - `repository`: text. owner/name; omit if sole writable.
  - `name`: text. A name for after.
  - `landing`: text. goal/name; omit: held goal/base.
  - `branch`: text. mine: goal into base; omit: a new. the `branch` form, under Shared forms.
  - `merge`: text. Catch-up: 40/64 hex commit.
  - `after`: array. Live delegate IDs/earlier names.
  - `producer`: integer >=1. Producer charter; omit: forge default.
  - `llm`: object. producer/review:{main:{model,effort},sub_agents:[{model,effort}]}.
  - `repairs`: text. Red landing's 40/64 hex merge commit.
  - `questions`: text. up, none; omit: project setting. the `questions` form, under Shared forms.
- `repairs`: text. Red landing's 40/64 hex merge commit.
- `questions`: text. up, none; omit: project setting. the `questions` form, under Shared forms.

## delegate

Batch direct delegates within authority. tracked {priority}: goals, lower first. Repository changes: call change.

A write.
Offered where the run's charter names the delegate family and its authority holds it.

- `batch`: array; required. The tasks: each with executor, spec, contract, authority; optionally deadline_seconds, tracked, dependencies, wake, llm.
  - `executor`: object; required. {kind: agent, charter}, {kind: procedure, connector, code}, {kind: person, person} or {kind: role, role}.
    - `kind`: one of `agent|procedure|person|role`; required.
    - `charter`: integer >=0.
    - `connector`: integer >=0.
    - `code`: integer >=0.
    - `person`: integer >=0.
    - `role`: integer >=0.
  - `spec`: object; required. the `spec` form, under Shared forms.
  - `contract`: object; required. {kind: report, words}, {kind: change, connector, change_kind, words} or {kind: verdict, choices: [{code, words}]}.
    - `kind`: one of `report|change|verdict`; required.
    - `words`: integer >=0.
    - `connector`: integer >=0.
    - `change_kind`: integer >=0.
    - `choices`: array.
      - `code`: integer >=0; required.
      - `words`: integer >=0; required.
  - `authority`: object; required. Whole: restate every member you keep. the `authority` form, under Shared forms.
  - `deadline_seconds`: integer >=1. It must end this many seconds after it is made. Give this or authority.budget.deadline; with neither, it ends with you.
  - `tracked`: object. Track it as a goal at this priority, the lower first. Leave it out for an untracked task. the `tracked` form, under Shared forms.
  - `dependencies`: array.
    - `kind`: one of `batch|existing`; required.
    - `number`: integer >=0; required.
  - `wake`: object. the `wake` form, under Shared forms.
  - `llm`: object. main {model, effort} names the model, effort optional; subagents and account are optional. the `llm` form, under Shared forms.
  - `symbolic_grants`: array. the `grants` form, under Shared forms.

## amend

Amend a live delegate at its spec revision within authority. Omitted parameters remain; lists replace. Authority and instructions are whole.

A write.
Offered where the run's charter names the control family and its authority holds it.

- `target`: integer >=1; required. The delegate's number.
- `revision`: integer >=0. The spec revision you last read. Leave it out only to change a tracked delegate's priority alone.
- `amendment`: object; required. The reason, which its live run hears, and what changes: tracked, instructions, spec, authority, dependencies, wake, llm.
  - `reason`: text; required. What the delegate's live run hears, unless new instructions replace it.
  - `tracked`: object or string. {priority} tracks it as a goal, the lower first; untracked stops tracking it.
    - Alternative: object. the `tracked` form, under Shared forms.
    - Alternative: one of `untracked`.
  - `instructions`: text. Its own instructions, whole; empty leaves it none.
  - `spec`: object. the `spec` form, under Shared forms.
  - `authority`: object. Whole: restate every member you keep. A deadline is budget.deadline. the `authority` form, under Shared forms.
  - `dependencies`: array. The whole list of the existing tasks it waits on.
  - `wake`: object. the `wake` form, under Shared forms.
  - `llm`: object. main {model, effort} names the model, effort optional; subagents and account are optional. the `llm` form, under Shared forms.

## message

Send words, a question or an answer to a referenced task; its run reads it next turn.

A write.
Offered where the run's charter names the message family and its authority holds it.

- `target`: integer >=1; required. Task.
- `form`: one of `words|question|answer`; required. An answer names its question.
- `words`: text; required. Text.
- `question`: integer >=1. Answer: question ID.

## cancel

Cancel a live delegate and every task below it, with a reason.

A write.
Offered where the run's charter names the control family and its authority holds it.

- `target`: integer >=1; required. The delegate's number.
- `reason`: text; required. Why it is cancelled.

## release

Release a held delegate, naming its hold revision. One a party's hold holds is theirs to release: propose it.

A write.
Offered where the run's charter names the control family and its authority holds it.

- `target`: integer >=1; required. The held delegate's number.
- `revision`: integer >=0; required. The hold's revision, as you last read it.
- `again`: boolean. For an effect held uncertain alone: true makes it again, false does not.

## introduce

Give two referenced tasks references to each other, so each can message the other.

A write.
Offered where the run's charter names the message family and its authority holds it.

- `left`: integer >=1; required. Task number.
- `right`: integer >=1; required. Other task.

## subscribe

Hear of a referenced task held/ended, or a timer once/periodically. Returns subscription number.

A write.
Offered where the run's charter names the subscribe family and its authority holds it.

- `kind`: one of `task|timer`; required. task or timer.
- `target`: integer >=1. task: task number.
- `held`: boolean. task: hear when held; omit: false.
- `result`: boolean. task: hear result; omit: true.
- `at`: integer >=0. timer: Unix-epoch wall nanoseconds.
- `period`: integer >=1. timer: repeat interval, nanoseconds; omit: once.

## unsubscribe

Remove one of your subscriptions.

A write.
Offered where the run's charter names the subscribe family and its authority holds it.

- `subscription`: integer >=1; required. The subscription's number, as subscribe answered it.

## decide

Decide a proposal waiting for you: accept it, reject it with a reason, or pass it up to the holder above.

A write.
Offered where the run's charter names the decide family and its authority holds it.

- `proposer`: integer >=1; required. The proposing task's number.
- `proposal`: integer >=1; required. The proposal's number among its proposer's.
- `decision`: one of `accept|reject|pass`; required. accept, reject or pass.
- `reason`: text. Why; required to reject.

## decide_escalation

Decide a held descendant's escalation: release it, reject it with a reason, or pass it up to the holder above.

A write.
Offered where the run's charter names the decide family and its authority holds it.

- `task`: integer >=1; required. The held task's number.
- `revision`: integer >=1; required. Its hold's revision, as you last read it.
- `decision`: one of `release|reject|pass`; required. release, reject or pass.
- `reason`: text. Why; required to reject.

## propose

Ask the holder to act beyond authority; party-held releases await their owner.

A write.
Offered where the run's charter names the propose family and its authority holds it.

- `action`: one of `batch|effect|amend|widen|release`; required. Act.
- `task`: integer >=1. amend/widen/release: task.
- `revision`: integer >=0. amend: spec; release: hold.
- `hold`: integer >=1.
- `batch`: array. delegate's batch.
- `amendment`: object. amend's amendment.
- `authority`: object. Whole authority.
- `reason`: text; required. Why.
- `as_holder`: boolean. batch: accepter holds them.
- `repository`: object. effect: effect_forge.repository.
- `resource`: object. effect: effect_forge.resource.
- `write`: object. effect: effect_forge.write.

## withdraw

Withdraw one of your pending proposals.

A write.
Offered where the run's charter names the decide family and its authority holds it.

- `proposal`: integer >=1; required. The proposal's number, as propose answered it.

## note

Write or revise a note, whole or by lines, at the revision read.

A write.
Offered where the run's charter names the note family and its authority holds it.

- `scope`: object; required. {kind}: deployment; project:+project; goal:+project,goal; resources:+project,connector,pattern.
- `description`: text; required. What a search reads.
- `body`: text; required. New text or range replacement.
- `references`: array. Task numbers.
- `name`: integer >=1. Revise: name; omit: new.
- `recalled`: integer >=1. Revision read.
- `first`: integer >=1. With name,recalled,remove: first line (1-based); remove=0 past end appends.
- `remove`: integer >=0. With first: lines removed; 0 inserts.
- `join`: boolean. With first,remove: join preceding line without newline.

## recall

Recall a note by its name, or search the descriptions of the notes in some scopes, a page at a time.

A read.
Offered where the run's charter names the recall family and its authority holds it.

- `kind`: one of `name|search`; required. name, by its name; search, by words in scopes.
- `name`: integer >=1. For name: the note's name.
- `scopes`: array. For search: the scopes, each as note's scope.
- `query`: text. For search: the words.
- `page`: integer >=0. The page, from 0; 0 when absent.

## subscribe_forge

Hear forge news: landings on a branch, CI on heads, pull request state, or comments and reviews on an issue or pull request.

A write.
Offered where the run's charter names the subscribe family and its authority holds it.

- `topic`: object; required. {kind,repository?}: owner/name|{forge,repository}, else sole readable; landings+branch,ci+head,pull/participation+number
- `own_change`: integer >=1. For landings: your change task's number; its own landing does not wake you.
- `paths`: array. For landings: a landing wakes you when it changes one of these paths, or a file below one.

## effect_forge

Make one write on an adopted repository through the forge's connector: an issue, a comment or a status, among the forge's writes.

A write.
Offered where the run's charter names the effect family and its authority holds it.

- `repository`: object; required. The repository, as {forge, repository}: the forge's number and the repository's.
- `resource`: object; required. What it writes on: {kind: repository}, {kind: pull, number}, {kind: issue, number} or {kind: branch, segments}.
- `write`: object; required. {kind: create_issue, key, title, body}, {kind: post, number, key, body} or {kind: status, commit, context, check}.

## read_task

Specs/results; from=next. For an answer: durable question numbers and answerers.

A read.
Offered where the run's charter names the read family and its authority holds it.

- `task`: integer >=1; required.
- `field`: one of `specification|result`.
- `from`: integer >=0, 0 when absent.

## settings

Read settings: no members gives inventory; setting, revision or history selects one. Long answers continue from an offset.

A read.
Offered where the run's charter names the settings family and its authority holds it.

- `setting`: object. One setting's whole value, named as a settings file names it.
  - `kind`: text; required. The setting's kind. the `kind` form, under Shared forms.
  - `number`: integer [1, 4294967295]. Setting number; template-goal: its template's. Omit for rules and unnumbered project settings.
  - `project`: integer [1, 4294967295]. Project settings: required; landing-rule: optional (deployment if absent); charter-words and template-goal: none.
- `revision`: integer [1, 4294967295]. Alone, that revision's record: its items and who decided it. With a charter, the charter as that held revision has it.
- `history`: object. A page of the revisions, the newest first.
  - `before`: integer [1, 4294967295]. The page before this revision; the newest page when absent.
- `from`: integer [0, 4294967295]. Where to go on in the answer's text: the offset after next: on its last line; 0 when absent.

## propose_settings

Propose whole settings from items or a note revision. One owner acceptance makes a revision; keep your document separate.

A write.
Offered where the run's charter names the settings family and its authority holds it.

- `against`: integer [1, 4294967295]; required. The newest revision you read. A proposal against an older one is stale, and you hear the newest.
- `reason`: text; required. Why; the owners read it.
- `note`: object. {name, revision}: goal note work: settings with {items:[...]}; build a few KB per write with first/remove/join.
  - `name`: integer >=1; required.
  - `revision`: integer [1, 4294967295]; required.
- `items`: array. The changes, at most one per setting. Supply exactly one of items or note.
  - `setting`: object; required. The setting, named as a settings file names it.
    - `kind`: text; required. The setting's kind. the `kind` form, under Shared forms.
    - `number`: integer >=1. Setting number; template-goal: its template's. Omit for rules and unnumbered project settings.
    - `project`: integer [1, 4294967295]. Project settings: required; landing-rule: optional (deployment if absent); charter-words and template-goal: none.
  - `set`: any value. Its whole new value, as a settings file holds it; forge-charter takes a charter number. Read the setting first.
  - `edit`: one of `reset|removed`. In place of set: reset, to follow its default again, or removed, no longer offered.

## read_items

Issues/pulls, oldest updates first; forge.client.rows/answer_bytes bounded. Advance page.

A read.
Offered where the run's charter names the read family and its authority holds it.

- `repository`: text. The repository, as owner/name; leave it out, or empty, when the project has one.
- `since`: integer [0, 18446744073709551615], 0 when absent. Only items updated at or after this time, in nanoseconds.
- `page`: integer [1, 4294967295], 1 when absent. The page.
- `item_kind`: one of `issue|pull`. Issues or pull requests alone; both when absent.

## read_item

Issue/pull and comments within forge.client.rows/answer_bytes. Advance after for comments; read_items for summaries.

A read.
Offered where the run's charter names the read family and its authority holds it.

- `repository`: text. The repository, as owner/name; leave it out, or empty, when the project has one.
- `number`: integer [1, 18446744073709551615]; required. The issue's or pull request's number.
- `after`: integer [0, 18446744073709551615], 0 when absent. Only the comments after this comment's id.

## read_pull

Pull within forge.client.rows/answer_bytes. Files: read_pull_files by page, or read_file by path at head/base.

A read.
Offered where the run's charter names the read family and its authority holds it.

- `repository`: text. The repository, as owner/name; leave it out, or empty, when the project has one.
- `number`: integer [1, 18446744073709551615]; required. The pull request's number.

## read_pull_for

Find a pull by head/base within forge.client.rows/answer_bytes. Files: read_pull_files by page or read_file by path at head/base.

A read.
Offered where the run's charter names the read family and its authority holds it.

- `repository`: text. The repository, as owner/name; leave it out, or empty, when the project has one.
- `head`: text; required. The branch it merges from.
- `base`: text; required. The branch it merges into.

## read_reviews

Pull reviews within forge.client.rows/answer_bytes. Next page uncut; same page/skip if cut.

A read.
Offered where the run's charter names the read family and its authority holds it.

- `repository`: text. owner/name; omit or empty for sole project repo.
- `number`: integer [1, 18446744073709551615]; required. Pull #.
- `page`: integer [1, 4294967295], 1 when absent. Page.
- `skip`: integer [0, 4294967295], 0 when absent. Rows read.

## read_statuses

Commit statuses within forge.client.rows/answer_bytes. Advance page for other rows; it skips an oversized page.

A read.
Offered where the run's charter names the read family and its authority holds it.

- `repository`: text. The repository, as owner/name; leave it out, or empty, when the project has one.
- `commit`: text; required. The commit's id: 40 or 64 lowercase hexadecimal digits.
- `page`: integer [1, 4294967295], 1 when absent. The page.

## read_remarks

Review remarks within forge.client.rows/answer_bytes. Next page uncut; same page/skip if cut.

A read.
Offered where the run's charter names the read family and its authority holds it.

- `repository`: text. owner/name; omit or empty for sole project repo.
- `number`: integer [1, 18446744073709551615]; required. Pull #.
- `review`: integer [1, 18446744073709551615]; required. read_reviews id.
- `page`: integer [1, 4294967295], 1 when absent. Page.
- `skip`: integer [0, 4294967295], 0 when absent. Rows read.

## read_branch

Branch head within forge.client.answer_bytes; read_branches with a longer prefix lists names.

A read.
Offered where the run's charter names the read family and its authority holds it.

- `repository`: text. The repository, as owner/name; leave it out, or empty, when the project has one.
- `branch`: text; required. The branch's name.

## read_branches

List the branches whose names start with a prefix. The names fit forge.client.answer_bytes; a longer prefix reads fewer.

A read.
Offered where the run's charter names the read family and its authority holds it.

- `repository`: text. The repository, as owner/name; leave it out, or empty, when the project has one.
- `prefix`: text; required. A nonempty prefix of the names, as temper/.

## read_pull_files

Whole pull files within forge.client.rows/answer_bytes. Next page uncut; same page/skip if cut. read_file at head/base.

A read.
Offered where the run's charter names the read family and its authority holds it.

- `repository`: text. owner/name; omit or empty for sole project repo.
- `number`: integer [1, 18446744073709551615]; required. Pull #.
- `head`: text; required. Head: 40/64 lowercase hex.
- `page`: integer [1, 4294967295], 1 when absent. Page.
- `skip`: integer [0, 4294967295], 0 when absent. Rows read.

## read_compare

Compare commits by names, counts and containment, without contents. No raw reads. Read a path's contents with read_file at each commit; Missing means absent.

A read.
Offered where the run's charter names the read family and its authority holds it.

- `repository`: text. The repository, as owner/name; leave it out, or empty, when the project has one.
- `before`: text; required. The earlier commit's id: 40 or 64 lowercase hexadecimal digits.
- `after`: text; required. The later commit's id: 40 or 64 lowercase hexadecimal digits.

## read_checks

Combined checks within forge.client.rows/answer_bytes; read_statuses by page, then read_job for a failed job.

A read.
Offered where the run's charter names the read family and its authority holds it.

- `repository`: text. The repository, as owner/name; leave it out, or empty, when the project has one.
- `commit`: text; required. The commit's id: 40 or 64 lowercase hexadecimal digits.

## read_file

File prefix or directory within forge.client.answer_bytes and max_bytes. A cut is incomplete; lower max_bytes or select directory paths.

A read.
Offered where the run's charter names the read family and its authority holds it.

- `repository`: text. The repository, as owner/name; leave it out, or empty, when the project has one.
- `head`: text; required. The commit's id: 40 or 64 lowercase hexadecimal digits.
- `path`: text; required. A file's or a directory's path; "." or empty for the root.
- `max_bytes`: integer [1, 4294967295]. Raw prefix bytes; default: read limit. Leave host room for metadata and escaping (up to 4x); a cut is incomplete.

## read_job

Failed-step end or job tail within forge.client.answer_bytes. Lower max_bytes for the same attempt to fit.

A read.
Offered where the run's charter names the read family and its authority holds it.

- `repository`: text. The repository, as owner/name; leave it out, or empty, when the project has one.
- `head`: text; required. The commit's id the CI ran on: 40 or 64 lowercase hexadecimal digits.
- `run`: integer [1, 18446744073709551615]; required. The CI run's id, as read_checks names it; not a link's run number.
- `job`: integer [0, 18446744073709551615]; required. The CI job's id, as read_checks names it; not a link's job index.
- `attempt`: integer [0, 4294967295]; required. The attempt's number, which may be 0 on Forgejo.
- `max_bytes`: integer [1, 4294967295]. At most this many bytes; the read's limit when absent.

## read_protection

Branch protection within forge.client.rows/answer_bytes. No pages; read_branch or read_items for narrower context.

A read.
Offered where the run's charter names the read family and its authority holds it.

- `repository`: text. The repository, as owner/name; leave it out, or empty, when the project has one.
- `branch`: text; required. The branch's name.

## read_settings

Repository settings within forge.client.answer_bytes. No pages; read_branch or read_items for narrower context.

A read.
Offered where the run's charter names the read family and its authority holds it.

- `repository`: text. The repository, as owner/name; leave it out, or empty, when the project has one.

## read_collaborators

Collaborators within forge.client.rows/answer_bytes; advance page. read_permission reads one user.

A read.
Offered where the run's charter names the read family and its authority holds it.

- `repository`: text. The repository, as owner/name; leave it out, or empty, when the project has one.
- `page`: integer [1, 4294967295], 1 when absent. The page.

## read_permission

User permission within forge.client.answer_bytes. No pages; read_collaborators lists users by page.

A read.
Offered where the run's charter names the read family and its authority holds it.

- `repository`: text. The repository, as owner/name; leave it out, or empty, when the project has one.
- `user`: integer [1, 18446744073709551615]; required. The forge user's id.

## read_docs

Read pages: {topic,from} gives text then next: OFFSET or end; {search} finds sections holding every word.

A read.
Offered to every run but an asking one.

- `topic`: text. Page, e.g. index.
- `from`: integer >=0. Offset after next:; default 0.
- `search`: text. Up to 8 words, all required.

## see

Read temper by address, as your person, in your project: its work, changes, settings, proposals, pages and files. Each answer ends with what to read next.

A read.
Offered to an asking run, where its authority holds the asking family.

- `address`: text; required. One line, as your instructions list addresses: settings charter 1, help, more TOKEN.

## document

Keep this chat's one document: write it whole first, its first line # TITLE; then append a section, or replace the one under the same ## heading.

A write.
Offered to an asking run, where its authority holds the asking family.

- `action`: one of `write|append|replace`; required. write: the whole document; append: a section at its end; replace: the section under body's first ## heading.
- `body`: text; required. Markdown: the whole document, or one section beginning with its ## heading.

## remember

Keep a line about your person across their chats, only when they ask: add one, or change or drop one by its number. Each answer is the whole list.

A write.
Offered to an asking run, where its authority holds the asking family.

- `action`: one of `add|change|drop`; required.
- `line`: integer [1, 4294967295]. The line's number, for change and drop.
- `text`: text. One line, at most 120 bytes.

## Shared forms

### authority

- `tools`: integer >=0; required. The tool families, as a bit set.
- `grants`: array; required. the `grants` form, under Shared forms.
- `delegation`: object; required.
  - `kinds`: array; required.
    - `kind`: one of `agent|procedure|role`; required.
    - `number`: integer >=0; required.
  - `tasks`: integer >=0; required.
  - `depth`: integer >=0; required.
- `budget`: object; required.
  - `spend`: integer >=0; required.
  - `deadline`: integer >=0. Wall-clock nanoseconds since the Unix epoch.
- `notes`: integer [0, 255]; required.
- `note_resources`: array.
  - `connector`: integer >=0; required.
  - `pattern`: object; required.
    - `segments`: array; required.
    - `terminal`: text; required. the `terminal` form, under Shared forms.
    - `last`: text; required.

### wake

- `words`: object; required. the `words` form, under Shared forms.
- `notices`: object; required. the `words` form, under Shared forms.
- `news`: object; required. the `words` form, under Shared forms.
- `results`: one of `never|each|last_or_failure`; required.
- `questions`: boolean; required.
- `answers`: boolean; required.
- `timers`: boolean; required.

### spec

- `words`: text; required.
- `parameters`: array. {name, kind: number, value}, {name, kind: bytes, value} or {name, kind: resource, connector, resource}.
  - `name`: integer >=0; required.
  - `kind`: one of `number|bytes|resource`; required.
  - `value`: any value.
  - `connector`: integer >=0.
  - `resource`: integer >=0.
- `inputs`: array.

### llm

- `main`: object; required. the `main` form, under Shared forms.
- `subagents`: array. the `main` form, under Shared forms.
- `account`: integer >=0.

### grants

- `connector`: integer >=0; required.
- `kind`: integer >=0; required.
- `segments`: array; required.
- `terminal`: text; required. the `terminal` form, under Shared forms.
- `last`: text; required.

### kind

one of `charter|template|policy|rules|model|landing-rule|forge-charter|template-goal|project-policy|project-ways|project-questions|project-classes|charter-words|project-words|follow|dependency|project-areas|project-triager`.

### words

- `kind`: one of `never|immediate|batch`; required.
- `count`: integer >=0.
- `age`: integer >=0. Nanoseconds.

### main

- `model`: integer [0, 255]; required.
- `effort`: integer [0, 255].

### tracked

- `priority`: integer [0, 4294967295]; required.

### questions

one of `none|up`.

### terminal

one of `exact|open`.

### branch

one of `mine`.
