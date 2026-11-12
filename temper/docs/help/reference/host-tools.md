# Host tools

Kind: reference

Every tool temper adds to its runs' own: what it does, as a run reads it, whether it reads or writes, and the members of its input.

File, pull-file and compare replies fit the smaller of core.call_answer_bytes less the tool name and the receiving agent allowance (agent.json profile.declared.tool_payload). Raising core.call_answer_bytes alone cannot overcome a smaller agent allowance. Oversized rendered comparisons return measured-size feedback and recovery instructions; forge size refusals name the bound passed and the exact size, or at least the size read. Unnamed checks say an unnamed bound; a forge bound without figures says a bound of the forge's. Each refusal says how to read on. Host-fitted file prefixes keep [file truncated] and give size and recovery feedback when room permits. max_bytes selects a raw prefix, with no offset; a cut is incomplete. Metadata that leaves no content room needs a shorter path or a larger allowance. Oversized pull-file pages retain changed paths as room permits and distinguish omitted contents from absent sides. Read selected paths at the head and base commits, resolving the base separately; recover omitted paths by listing directories at both commits. Later pages only list other files.

## finding

file: kind severity title body [area]; hand: number area why; route: number goal; backlog: number; drop: number reason; list: [state] [mine]; show: number

A write.

- `act`: one of `file`, `hand`, `route`, `backlog`, `drop`, `list` or `show`; required.
- `kind`: one of `qa`, `flake`, `red`, `review` or `feedback`; optional.
- `severity`: one of `low`, `medium` or `high`; optional.
- `area`: text; optional.
- `title`: text; optional.
- `body`: text; optional.
- `number`: an integer; optional.
- `why`: text; optional.
- `goal`: an integer; optional.
- `reason`: text; optional.
- `state`: text; optional.
- `mine`: true or false; optional.

## decided

Ask whether you may decide this for your person. Carry on only if decided; otherwise ask them with message, as a question.

A write.

- `question`: text; required. The question, at most 400 bytes.
- `options`: a list; required. Two to six options, each at most 120 bytes.
- `took`: an integer from 0 to 5; required. The chosen option's index, starting at zero.
- `why`: text; required. Why you chose it, at most 400 bytes.
- `kind`: one of `wording`, `retry`, `implementation`, `tests`, `dependency`, `departure`, `money`, `irreversible`, `rules` or `other`; required.
- `paths`: a list; optional. At most 16 repository-relative paths the choice would change, each 1 to 256 bytes with no leading /.
- `under`: an integer from 1 to 20; optional. The item with words of settings project-classes that covers this choice.

## goal

Have one goal tracked, only for work your person wants agreed first (set agree) or a survey that changes nothing. For a change to a repository, call change.

A write.

- `words`: text; required.
- `advisor`: true or false; optional. Set by a goal alone, for a survey that changes nothing.
- `agree`: true or false; optional. Set by the person's chat when they want the work agreed first: the goal asks its plan before any change.

## change

Have one change written on a branch of a repository of this project, opened as a pull request and merged once its checks pass; you hear its result.

A write.

- `words`: text; required. What to change: the files, and the exact text where it matters. It is all the change's coder reads.
- `repository`: text; optional. The repository, as owner/name; leave it out when the project has one you may write.

## delegate

Make a batch of direct delegates within your authority. tracked {priority} makes one a goal, the lower first. For a change to a repository, call change.

A write.

- `batch`: a list; required. The tasks: each with executor, spec, contract, authority; optionally deadline_seconds, tracked, dependencies, wake, llm.

## amend

Amend a live delegate at its spec revision within authority. Omitted parameters remain; lists replace. Authority and instructions are whole.

A write.

- `target`: an integer of at least 1; required. The delegate's number.
- `revision`: an integer of at least 0; optional. The spec revision you last read. Leave it out only to change a tracked delegate's priority alone.
- `amendment`: an object; required. The reason, which its live run hears, and what changes: tracked, instructions, spec, authority, dependencies, wake, llm.

## message

Send words, a question, or the answer to a question, to a task you reference. Its run reads it at its next turn.

A write.

- `target`: an integer of at least 1; required. The task's number.
- `form`: one of `words`, `question` or `answer`; required. words; a question, which its run may answer; or an answer, naming the question.
- `words`: text; required. What you say.
- `question`: an integer of at least 1; optional. For an answer: the question's number.

## cancel

Cancel a live delegate and every task below it, with a reason.

A write.

- `target`: an integer of at least 1; required. The delegate's number.
- `reason`: text; required. Why it is cancelled.

## release

Release a held delegate, naming its hold revision. One a party's hold holds is theirs to release: propose it.

A write.

- `target`: an integer of at least 1; required. The held delegate's number.
- `revision`: an integer of at least 0; required. The hold's revision, as you last read it.
- `again`: true or false; optional. For an effect held uncertain alone: true makes it again, false does not.

## introduce

Give two tasks you reference a reference to each other, so that each can message the other.

A write.

- `left`: an integer of at least 1; required. One task's number.
- `right`: an integer of at least 1; required. The other task's number.

## subscribe

Hear of a task you reference, held or ended, or of a timer, once or periodic. Answers the subscription's number.

A write.

- `kind`: one of `task` or `timer`; required. task, to hear of a task; timer, to hear at a time.
- `target`: an integer of at least 1; optional. For task: the task's number.
- `held`: true or false; optional. For task: true to hear when it is held; false when absent.
- `result`: true or false; optional. For task: true to hear its result; true when absent.
- `at`: an integer of at least 0; optional. For timer: when, in wall-clock nanoseconds since the Unix epoch.
- `period`: an integer of at least 1; optional. For timer: again every this many nanoseconds; once when absent.

## unsubscribe

Remove one of your subscriptions.

A write.

- `subscription`: an integer of at least 1; required. The subscription's number, as subscribe answered it.

## decide

Decide a proposal waiting for you: accept it, reject it with a reason, or pass it up to the holder above.

A write.

- `proposer`: an integer of at least 1; required. The proposing task's number.
- `proposal`: an integer of at least 1; required. The proposal's number among its proposer's.
- `decision`: one of `accept`, `reject` or `pass`; required. accept, reject or pass.
- `reason`: text; optional. Why; required to reject.

## decide_escalation

Decide a held descendant's escalation: release it, reject it with a reason, or pass it up to the holder above.

A write.

- `task`: an integer of at least 1; required. The held task's number.
- `revision`: an integer of at least 1; required. Its hold's revision, as you last read it.
- `decision`: one of `release`, `reject` or `pass`; required. release, reject or pass.
- `reason`: text; optional. Why; required to reject.

## propose

Ask the covering holder to act beyond your authority. A release from a party's hold names task, hold and the hold's revision, and waits for its owner.

A write.

- `action`: one of `batch`, `effect`, `amend`, `widen` or `release`; required. The act; each takes the members named for it below.
- `task`: an integer of at least 1; optional. For amend, widen and release: the task's number.
- `revision`: an integer of at least 0; optional. The spec revision you last saw, for amend; the hold's revision, for release.
- `hold`: an integer of at least 1; optional.
- `batch`: a list; optional. For batch: the tasks, as delegate's batch.
- `amendment`: an object; optional. For amend: what changes, as amend's amendment.
- `authority`: an object; optional. For widen: the authority wanted, whole.
- `reason`: text; required. Why; the holder reads it.
- `as_holder`: true or false; optional. For batch: true makes the tasks the accepter's delegates rather than yours.
- `repository`: an object; optional. For effect: as effect_forge's repository.
- `resource`: an object; optional. For effect: as effect_forge's resource.
- `write`: an object; optional. For effect: as effect_forge's write.

## withdraw

Withdraw one of your pending proposals.

A write.

- `proposal`: an integer of at least 1; required. The proposal's number, as propose answered it.

## note

Write a note in a scope for later runs to recall, or revise one, whole or by a line range, naming the revision you read.

A write.

- `scope`: an object; required. {kind}: deployment; project, with project; goal, with project and goal; resources, with project, connector, pattern.
- `description`: text; required. What the note is about: a search reads it.
- `body`: text; required. The note's text, or the text that replaces the range.
- `references`: a list; optional. The numbers of the tasks it refers to.
- `name`: an integer of at least 1; optional. To revise a note: its name. Leave it out for a new note.
- `recalled`: an integer of at least 1; optional. To revise a note: the revision you read, from your brief, recall or your last write's answer.
- `first`: an integer of at least 1; optional. To change lines, with name, recalled and remove: the 1-based first line replaced; the note's lines + 1 to append.
- `remove`: an integer of at least 0; optional. With first: how many lines go; 0 to insert.
- `join`: true or false; optional. With first and remove: true continues the line before first without a newline, for long lines.

## recall

Recall a note by its name, or search the descriptions of the notes in some scopes, a page at a time.

A read.

- `kind`: one of `name` or `search`; required. name, by its name; search, by words in scopes.
- `name`: an integer of at least 1; optional. For name: the note's name.
- `scopes`: a list; optional. For search: the scopes, each as note's scope.
- `query`: text; optional. For search: the words.
- `page`: an integer of at least 0; optional. The page, from 0; 0 when absent.

## subscribe_forge

Hear forge news: landings on a branch, CI on heads, pull request state, or comments and reviews on an issue or pull request.

A write.

- `topic`: an object; required. {kind,repository?}: owner/name|{forge,repository}, else sole readable; landings+branch,ci+head,pull/participation+number
- `own_change`: an integer of at least 1; optional. For landings: your change task's number; its own landing does not wake you.
- `paths`: a list; optional. For landings: a landing wakes you when it changes one of these paths, or a file below one.

## effect_forge

Make one write on an adopted repository through the forge's connector: an issue, a comment or a status, among the forge's writes.

A write.

- `repository`: an object; required. The repository, as {forge, repository}: the forge's number and the repository's.
- `resource`: an object; required. What it writes on: {kind: repository}, {kind: pull, number}, {kind: issue, number} or {kind: branch, segments}.
- `write`: an object; required. {kind: create_issue, key, title, body}, {kind: post, number, key, body} or {kind: status, commit, context, check}.

## read_task

Specs/results, from=next. Durable question numbers/answerers: message form=answer, question

A read.

- `task`: an integer of at least 1; required.
- `field`: one of `specification` or `result`; optional.
- `from`: an integer of at least 0, 0 when absent; optional.

## settings

Read the deployment's settings: the inventory with no member, or a setting, a revision's record or a page of the history; a long answer goes on from an offset.

A read.

- `setting`: an object; optional. One setting's whole value, named as a settings file names it.
- `revision`: an integer from 1 to 4294967295; optional. Alone, that revision's record: its items and who decided it. With a charter, the charter as that held revision has it.
- `history`: an object; optional. A page of the revisions, the newest first.
- `from`: an integer from 0 to 4294967295; optional. Where to go on in the answer's text: the offset after next: on its last line; 0 when absent.

## propose_settings

Propose whole settings from items or a note revision to owners. One owner acceptance makes one revision. Keep your document separate.

A write.

- `against`: an integer from 1 to 4294967295; required. The newest revision you read. A proposal against an older one is stale, and you hear the newest.
- `reason`: text; required. Why; the owners read it.
- `note`: an object; optional. {name, revision}: goal note work: settings with {items:[...]}; build a few KB per write with first/remove/join.
- `items`: a list; optional. The changes, at most one per setting. Supply exactly one of items or note.

## read_items

Issues/pulls, oldest updates first; forge.client.rows/answer_bytes bounded. Advance page.

A read.

- `repository`: text; optional. The repository, as owner/name; leave it out, or empty, when the project has one.
- `since`: an integer from 0 to 18446744073709551615, 0 when absent; optional. Only items updated at or after this time, in nanoseconds.
- `page`: an integer from 1 to 4294967295, 1 when absent; optional. The page.
- `item_kind`: one of `issue` or `pull`; optional. Issues or pull requests alone; both when absent.

## read_item

Issue/pull and comments within forge.client.rows/answer_bytes. Advance after for comments; read_items for summaries.

A read.

- `repository`: text; optional. The repository, as owner/name; leave it out, or empty, when the project has one.
- `number`: an integer from 1 to 18446744073709551615; required. The issue's or pull request's number.
- `after`: an integer from 0 to 18446744073709551615, 0 when absent; optional. Only the comments after this comment's id.

## read_pull

Pull within forge.client.rows/answer_bytes. Files: read_pull_files by page, or read_file by path at head/base.

A read.

- `repository`: text; optional. The repository, as owner/name; leave it out, or empty, when the project has one.
- `number`: an integer from 1 to 18446744073709551615; required. The pull request's number.

## read_pull_for

Find a pull by head/base within forge.client.rows/answer_bytes. Files: read_pull_files by page or read_file by path at head/base.

A read.

- `repository`: text; optional. The repository, as owner/name; leave it out, or empty, when the project has one.
- `head`: text; required. The branch it merges from.
- `base`: text; required. The branch it merges into.

## read_reviews

Pull reviews within forge.client.rows/answer_bytes. Next page uncut; same page/skip if cut.

A read.

- `repository`: text; optional. owner/name; omit or empty for sole project repo.
- `number`: an integer from 1 to 18446744073709551615; required. Pull #.
- `page`: an integer from 1 to 4294967295, 1 when absent; optional. Page.
- `skip`: an integer from 0 to 4294967295, 0 when absent; optional. Rows read.

## read_statuses

Commit statuses within forge.client.rows/answer_bytes. Advance page for other rows; it skips an oversized page.

A read.

- `repository`: text; optional. The repository, as owner/name; leave it out, or empty, when the project has one.
- `commit`: text; required. The commit's id: 40 or 64 lowercase hexadecimal digits.
- `page`: an integer from 1 to 4294967295, 1 when absent; optional. The page.

## read_remarks

Review remarks within forge.client.rows/answer_bytes. Next page uncut; same page/skip if cut.

A read.

- `repository`: text; optional. owner/name; omit or empty for sole project repo.
- `number`: an integer from 1 to 18446744073709551615; required. Pull #.
- `review`: an integer from 1 to 18446744073709551615; required. read_reviews id.
- `page`: an integer from 1 to 4294967295, 1 when absent; optional. Page.
- `skip`: an integer from 0 to 4294967295, 0 when absent; optional. Rows read.

## read_branch

Branch head within forge.client.answer_bytes; read_branches with a longer prefix lists names.

A read.

- `repository`: text; optional. The repository, as owner/name; leave it out, or empty, when the project has one.
- `branch`: text; required. The branch's name.

## read_branches

List the branches whose names start with a prefix. The names fit forge.client.answer_bytes; a longer prefix reads fewer.

A read.

- `repository`: text; optional. The repository, as owner/name; leave it out, or empty, when the project has one.
- `prefix`: text; required. A nonempty prefix of the names, as temper/.

## read_pull_files

Whole pull files within forge.client.rows/answer_bytes. Next page uncut; same page/skip if cut. read_file at head/base.

A read.

- `repository`: text; optional. owner/name; omit or empty for sole project repo.
- `number`: an integer from 1 to 18446744073709551615; required. Pull #.
- `head`: text; required. Head: 40/64 lowercase hex.
- `page`: an integer from 1 to 4294967295, 1 when absent; optional. Page.
- `skip`: an integer from 0 to 4294967295, 0 when absent; optional. Rows read.

## read_compare

Compare commits by names, counts and containment, without contents. No raw reads. Read a path's contents with read_file at each commit; Missing means absent.

A read.

- `repository`: text; optional. The repository, as owner/name; leave it out, or empty, when the project has one.
- `before`: text; required. The earlier commit's id: 40 or 64 lowercase hexadecimal digits.
- `after`: text; required. The later commit's id: 40 or 64 lowercase hexadecimal digits.

## read_checks

Combined checks within forge.client.rows/answer_bytes; read_statuses by page, then read_job for a failed job.

A read.

- `repository`: text; optional. The repository, as owner/name; leave it out, or empty, when the project has one.
- `commit`: text; required. The commit's id: 40 or 64 lowercase hexadecimal digits.

## read_file

File prefix or directory within forge.client.answer_bytes and max_bytes. A cut is incomplete; lower max_bytes or select directory paths.

A read.

- `repository`: text; optional. The repository, as owner/name; leave it out, or empty, when the project has one.
- `head`: text; required. The commit's id: 40 or 64 lowercase hexadecimal digits.
- `path`: text; required. A file's or a directory's path; "." or empty for the root.
- `max_bytes`: an integer from 1 to 4294967295; optional. Raw prefix bytes; default: read limit. Leave host room for metadata and escaping (up to 4x); a cut is incomplete.

## read_job

Failed-step end or job tail within forge.client.answer_bytes. Lower max_bytes for the same attempt to fit.

A read.

- `repository`: text; optional. The repository, as owner/name; leave it out, or empty, when the project has one.
- `head`: text; required. The commit's id the CI ran on: 40 or 64 lowercase hexadecimal digits.
- `run`: an integer from 1 to 18446744073709551615; required. The CI run's id, as read_checks names it; not a link's run number.
- `job`: an integer from 0 to 18446744073709551615; required. The CI job's id, as read_checks names it; not a link's job index.
- `attempt`: an integer from 0 to 4294967295; required. The attempt's number, which may be 0 on Forgejo.
- `max_bytes`: an integer from 1 to 4294967295; optional. At most this many bytes; the read's limit when absent.

## read_protection

Branch protection within forge.client.rows/answer_bytes. No pages; read_branch or read_items for narrower context.

A read.

- `repository`: text; optional. The repository, as owner/name; leave it out, or empty, when the project has one.
- `branch`: text; required. The branch's name.

## read_settings

Repository settings within forge.client.answer_bytes. No pages; read_branch or read_items for narrower context.

A read.

- `repository`: text; optional. The repository, as owner/name; leave it out, or empty, when the project has one.

## read_collaborators

Collaborators within forge.client.rows/answer_bytes; advance page. read_permission reads one user.

A read.

- `repository`: text; optional. The repository, as owner/name; leave it out, or empty, when the project has one.
- `page`: an integer from 1 to 4294967295, 1 when absent; optional. The page.

## read_permission

User permission within forge.client.answer_bytes. No pages; read_collaborators lists users by page.

A read.

- `repository`: text; optional. The repository, as owner/name; leave it out, or empty, when the project has one.
- `user`: an integer from 1 to 18446744073709551615; required. The forge user's id.

## read_docs

Read temper's pages: {topic, from} gives a page from an offset, then next: OFFSET or end; {search} the sections holding every word, a line each.

A read.

- `topic`: text; optional. A page's name, such as index.
- `from`: an integer of at least 0; optional. The offset to read from, as next: gave it; 0 when absent.
- `search`: text; optional. The words a section must hold, every one: at most 8.

## see

Read temper by address, as your person, in your project: its work, changes, settings, proposals, pages and files. Each answer ends with what to read next.

A read.

- `address`: text; required. One line, as your instructions list addresses: settings charter 1, help, more TOKEN.

## document

Keep this chat's one document: write it whole first, its first line # TITLE; then append a section, or replace the one under the same ## heading.

A write.

- `action`: one of `write`, `append` or `replace`; required. write: the whole document; append: a section at its end; replace: the section under body's first ## heading.
- `body`: text; required. Markdown: the whole document, or one section beginning with its ## heading.

## remember

Keep a line about your person across their chats, only when they ask: add one, or change or drop one by its number. Each answer is the whole list.

A write.

- `action`: one of `add`, `change` or `drop`; required.
- `line`: an integer from 1 to 4294967295; optional. The line's number, for change and drop.
- `text`: text; optional. One line, at most 120 bytes.
