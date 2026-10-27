# Host tools

Kind: reference

Every tool temper adds to its runs' own: what it does, as a run reads it, whether it reads or writes, and the members of its input.

## read_items

Read one page of forge items.

A read.

- `repository`: text; optional. owner/name of one of the project's repositories; leave out when it has one
- `since`: an integer from 0 to 18446744073709551615, 0 when absent; optional. a time in nanoseconds; defaults to 0
- `page`: an integer from 1 to 4294967295, 1 when absent; optional. a page number, from 1; defaults to 1
- `item_kind`: one of `issue` or `pull`; optional. issue or pull; leave out for both

## read_item

Read an issue or pull and its comments.

A read.

- `repository`: text; optional. owner/name of one of the project's repositories; leave out when it has one
- `number`: an integer from 1 to 18446744073709551615; required. an issue or pull request number
- `after`: an integer from 0 to 18446744073709551615, 0 when absent; optional. a comment id; defaults to 0

## read_pull

Read a pull request.

A read.

- `repository`: text; optional. owner/name of one of the project's repositories; leave out when it has one
- `number`: an integer from 1 to 18446744073709551615; required. an issue or pull request number

## read_pull_for

Find a pull by head and base.

A read.

- `repository`: text; optional. owner/name of one of the project's repositories; leave out when it has one
- `head`: text; required. a branch name: the pull request's head
- `base`: text; required. a branch name: the pull request's base

## read_reviews

Read reviews on a pull request.

A read.

- `repository`: text; optional. owner/name of one of the project's repositories; leave out when it has one
- `number`: an integer from 1 to 18446744073709551615; required. an issue or pull request number
- `page`: an integer from 1 to 4294967295, 1 when absent; optional. a page number, from 1; defaults to 1

## read_statuses

Read statuses on a commit.

A read.

- `repository`: text; optional. owner/name of one of the project's repositories; leave out when it has one
- `commit`: text; required. a commit id, 40 or 64 lowercase hex digits
- `page`: an integer from 1 to 4294967295, 1 when absent; optional. a page number, from 1; defaults to 1

## read_remarks

Read review remarks.

A read.

- `repository`: text; optional. owner/name of one of the project's repositories; leave out when it has one
- `number`: an integer from 1 to 18446744073709551615; required. an issue or pull request number
- `review`: an integer from 1 to 18446744073709551615; required. a review id
- `page`: an integer from 1 to 4294967295, 1 when absent; optional. a page number, from 1; defaults to 1

## read_branch

Read a branch head.

A read.

- `repository`: text; optional. owner/name of one of the project's repositories; leave out when it has one
- `branch`: text; required. a branch name

## read_branches

List branches.

A read.

- `repository`: text; optional. owner/name of one of the project's repositories; leave out when it has one
- `prefix`: text; required. a branch name prefix

## read_pull_files

Read changed files on a pull.

A read.

- `repository`: text; optional. owner/name of one of the project's repositories; leave out when it has one
- `number`: an integer from 1 to 18446744073709551615; required. an issue or pull request number
- `head`: text; required. a commit id, 40 or 64 lowercase hex digits
- `page`: an integer from 1 to 4294967295, 1 when absent; optional. a page number, from 1; defaults to 1

## read_compare

Compare two commits.

A read.

- `repository`: text; optional. owner/name of one of the project's repositories; leave out when it has one
- `before`: text; required. a commit id, 40 or 64 lowercase hex digits
- `after`: text; required. a commit id, 40 or 64 lowercase hex digits

## read_checks

Read a commit's combined checks.

A read.

- `repository`: text; optional. owner/name of one of the project's repositories; leave out when it has one
- `commit`: text; required. a commit id, 40 or 64 lowercase hex digits

## read_file

Read a bounded file at one head.

A read.

- `repository`: text; optional. owner/name of one of the project's repositories; leave out when it has one
- `head`: text; required. a commit id, 40 or 64 lowercase hex digits
- `path`: text; required. a file path in the repository
- `max_bytes`: an integer from 1 to 4294967295; optional. the maximum bytes to read; defaults to the read's limit

## read_job

Read one bounded CI job log.

A read.

- `repository`: text; optional. owner/name of one of the project's repositories; leave out when it has one
- `head`: text; required. a commit id, 40 or 64 lowercase hex digits
- `run`: an integer from 1 to 18446744073709551615; required. a CI run id
- `job`: an integer from 0 to 18446744073709551615; required. a CI job id, which may be 0 on Forgejo
- `attempt`: an integer from 0 to 4294967295; required. a CI attempt number, which may be 0 on Forgejo
- `max_bytes`: an integer from 1 to 4294967295; optional. the maximum bytes to read; defaults to the read's limit

## read_protection

Read branch protection.

A read.

- `repository`: text; optional. owner/name of one of the project's repositories; leave out when it has one
- `branch`: text; required. a branch name

## read_settings

Read repository settings.

A read.

- `repository`: text; optional. owner/name of one of the project's repositories; leave out when it has one

## read_collaborators

Read repository collaborators.

A read.

- `repository`: text; optional. owner/name of one of the project's repositories; leave out when it has one
- `page`: an integer from 1 to 4294967295, 1 when absent; optional. a page number, from 1; defaults to 1

## read_permission

Read a user's repository permission.

A read.

- `repository`: text; optional. owner/name of one of the project's repositories; leave out when it has one
- `user`: an integer from 1 to 18446744073709551615; required. a forge user's id

## effect_forge

Ask the connector for a keyed forge write.

A write.

- `repository`: an object; required.
- `resource`: an object; required.
- `write`: an object; required.

## subscribe_forge

Hear matching forge news.

A write.

- `topic`: an object; required.
- `own_change`: an integer of at least 1; optional.
- `paths`: a list; optional.

## change

Have one change made to a repository of this project and landed: temper writes it on a branch, opens a pull request, waits for CI and merges it; you hear its result. words is all the coder sees: say what to change, which files and the exact text where it matters. Name the repository as owner/name when the project has more than one; the answer names them if you don't.

A write.

- `words`: text; required.
- `repository`: text; optional.

## delegate

Create an authorized batch of direct child tasks. tracked {priority} makes one a goal, the lower first; omit it for untracked. Each llm choice names main {model, effort?}, optional subagents and an optional account.

A write.

- `batch`: a list; required.

## amend

Amend a live delegate within authority, naming only what changes. A tracked delegate's priority alone needs no revision. A deadline is authority.budget.deadline, with the rest of authority restated as it is. An llm choice names main {model, effort?}, optional subagents and an optional account.

A write.

- `target`: an integer of at least 1; required.
- `revision`: an integer of at least 0; optional. The spec revision you last saw; omit only to change a tracked delegate's priority alone.
- `amendment`: an object; required.

## message

Send words, a question or an answer to a referenced task.

A write.

- `target`: an integer of at least 1; required.
- `form`: one of `words`, `question` or `answer`; required.
- `words`: text; required.
- `question`: an integer of at least 1; optional.

## cancel

Cancel a live delegate and its subtree.

A write.

- `target`: an integer of at least 1; required.
- `reason`: text; required.

## release

Release a held delegate.

A write.

- `target`: an integer of at least 1; required.
- `revision`: an integer of at least 0; required.
- `again`: true or false; optional.

## introduce

Give two referenced tasks reciprocal references.

A write.

- `left`: an integer of at least 1; required.
- `right`: an integer of at least 1; required.

## unsubscribe

Remove a standing subscription.

A write.

- `subscription`: an integer of at least 1; required.

## withdraw

Withdraw one of your pending proposals.

A write.

- `proposal`: an integer of at least 1; required.

## decide_escalation

Release, reject or pass a held descendant.

A write.

- `task`: an integer of at least 1; required.
- `revision`: an integer of at least 1; required.
- `decision`: one of `release`, `reject` or `pass`; required.
- `reason`: text; optional.

## decide

Accept, reject or pass a pending proposal.

A write.

- `proposer`: an integer of at least 1; required.
- `proposal`: an integer of at least 1; required.
- `decision`: one of `accept`, `reject` or `pass`; required.
- `reason`: text; optional.

## subscribe

Watch a referenced task or a timer.

A write.

- `kind`: one of `task` or `timer`; required.
- `target`: an integer of at least 1; optional.
- `held`: true or false; optional.
- `result`: true or false; optional.
- `at`: an integer of at least 0; optional.
- `period`: an integer of at least 1; optional.

## propose

Ask the covering holder to act beyond your authority.

A write.

- `action`: one of `batch`, `effect`, `amend`, `widen` or `release`; required.
- `task`: an integer of at least 1; optional.
- `revision`: an integer of at least 0; optional.
- `batch`: a list; optional.
- `amendment`: an object; optional.
- `authority`: an object; optional.
- `reason`: text; required.
- `as_holder`: true or false; optional.
- `repository`: an object; optional.
- `resource`: an object; optional.
- `write`: an object; optional.

## note

Write a scoped note, or revise one at the revision you recalled.

A write.

- `scope`: an object; required.
- `description`: text; required.
- `body`: text; required.
- `references`: a list; optional.
- `name`: an integer of at least 1; optional.
- `recalled`: an integer of at least 1; optional.

## recall

Recall a note by name or search scoped descriptions.

A read.

- `kind`: one of `name` or `search`; required.
- `name`: an integer of at least 1; optional.
- `scopes`: a list; optional.
- `query`: text; optional.
- `page`: an integer of at least 0; optional.

## read_docs

Read temper's own documentation: setting it up, its concepts, recipes and limits. {"search": "WORDS"} answers the sections holding every word, a line each, NAME@OFFSET: HEADING: LINE. {"topic": NAME, "from": OFFSET} answers a page of a topic from OFFSET (0 when absent), then next: OFFSET to read on, or end.

A read.

- `topic`: text; optional.
- `from`: an integer of at least 0; optional.
- `search`: text; optional.
