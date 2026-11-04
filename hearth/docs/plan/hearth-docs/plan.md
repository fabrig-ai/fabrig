# hearth-docs: the plan

The request (`inputs/request.md`) brings hearth's documents to the repository's conventions and takes private names out of its current
files. Nothing of hearth's behaviour or design changes. This file grows with each push.

## Parts

- **family:** request 4. `readmes`.
- **citations:** request 3. `citations-code`, `citations-docs`: two tasks, since the files don't meet and neither needs the other.
- **names:** request 1. `names-hosts` first.
- **status:** request 2, with the rest of request 4. In the next push.

## Calls

- **What hearth prints stays.** 32 printed errors name a section as `protocol/format.md, 9.1`, and the `doc` strings of task JSON feed
  digests. "Nothing of hearth's behaviour changes" settles it: they keep their form, and `hearth/docs/design/README.status.md` records
  the departure from `hearth/docs/design/README.md`, 6. `notes/contracts.md`, 2.2 lists them.
- **`clippy.toml` and `[lints]` blocks don't change.** Their comments are rooted already, and what isn't rooted is a copy of
  `tools/lints/clippy/*.toml`, outside the request.
- **Citations go first.** Every later task writes on rooted citations, so no later diff carries them.
- **Other trees' documents** take the same root path (`notes/contracts.md`, 2.1), since the rule is the root's. The request names the
  foundation and hearth's own; leaving "skein's `io.md`" would keep the convention it drops.
- Names that look private and aren't in the request are in `notes/contracts.md`, 1.3, left for the user.

## Critical path

`citations-docs`, then the status tasks, then `status-index`. `readmes`, `citations-code` and `names-hosts` run beside it.

## Overlaps

None known with the drafts beside this one: their listed tasks hold no hearth file. `skein/docs/design/{io,json,kernel}.md` and the four
tree READMEs take one line or a few here, and no listed task changes them.
