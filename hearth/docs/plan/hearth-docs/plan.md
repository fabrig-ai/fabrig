# hearth-docs: the plan

The request (`inputs/request.md`) brings hearth's documents to the repository's conventions and takes private names out of its current
files. Nothing of hearth's behaviour or design changes. This file grows with each push.

## Parts

- **family:** request 4. `readmes`.
- **citations:** request 3. `citations-code`, `citations-docs`: two tasks, since the files don't meet and neither needs the other.
- **names:** request 1. `names-hosts`, then `names-account`, since both move the same goldens; `names-docs` for the documents.
- **status:** request 2 and the last of request 4. `status-format`, `status-store`, `status-testing`, `status-overview`,
  `status-machine`, `status-run`, `status-draft`, each owning its documents, then `status-index`.

`notes/common.md` holds what every task holds, `notes/contracts.md` the mapping, the citation forms and what stays, `notes/status.md`
how a status task works.

## Calls

- **What hearth prints stays.** 32 printed errors name a section as `protocol/format.md, 9.1`, and the `doc` strings of task JSON feed
  digests. "Nothing of hearth's behaviour changes" settles it: they keep their form, and `hearth/docs/design/README.status.md` records
  the departure from `hearth/docs/design/README.md`, 6. `notes/contracts.md`, 2.2 lists them.
- **`clippy.toml` and `[lints]` blocks don't change.** Their comments are rooted already, and what isn't rooted is a copy of
  `tools/lints/clippy/*.toml`, outside the request.
- **Citations go first.** Every later task writes on rooted citations, so no later diff carries them.
- **Other trees' documents** take the same root path (`notes/contracts.md`, 2.1), since the rule is the root's. The request names the
  foundation and hearth's own; leaving "skein's `io.md`" would keep the convention it drops.
- **`Open questions`** closes every document: each "What is open" is a list of open questions, whichever the milestone.
- **`alice` sorts before `ci` and `free` after,** so names change orders as well as digests. The specs call that the rename's own effect and
  have it reported; any other failure is a finding, which stops the task with a question.
- **`free` becomes `bob` where `alice` already stands beside it,** as in `hearth/tests/config/examples/mail.json`, so no two accounts merge.
- Names that look private and aren't in the request are in `notes/contracts.md`, 1.3, left for the user, with a question in `plan.toml`.

## Critical path

`citations-docs` (50), `names-docs` (20), `status-testing` (60), `status-index` (20): 150 implementer-minutes. `citations-code` and
`names-hosts` then `names-account` run beside it, and nothing after them waits on them. `names-account` waits for `citations-code` too:
the survey found no adjacent lines, but rustfmt reflows what a citation lengthens.

## What needed no task

- **`clippy.toml` and manifests:** no change, as Calls says.
- **`scripts/check.sh`, a git dependency on skein:** nothing in hearth says either. The separate repository lived in the READMEs alone
  (`readmes`). `hearth/docs/design/domain/testing.md`'s own profiles and budgets are design, and its status records where the root differs.
- **`hearth/docs/development/debt/`:** left. `D002`, `D003` and the debt `README.md` cite `hearth/docs/design/STATUS.md`, which stays;
  this plan renames no document.
- **`draft/archive/factory.md`:** temper's archive holds the file alone, with no status and no index entry, so this one does too.

## Overlaps

None known with the drafts beside this one: their listed tasks hold no hearth file. `skein/docs/design/{io,json,kernel}.md` and the four
tree READMEs take one line or a few here, and no listed task changes them.
