# outward-zero: plan

Request (`inputs/request.md`, step 12 of 12 of a user-approved plan): the outward check becomes
absolute. The inventory is empty, so it leaves, and any match fails.

## Parts and tasks

One part, one slice, one task, as the request asks, all in the first push.

- **absolute:** `tasks/absolute.md`, no needs, ~30 min, medium. `tests/boundary/tests/outward.rs`
  loses the inventory and gains a plain verdict; `tests/boundary/outward.txt` is deleted; the
  gate's README, the package description and two design lines say the same.

**Critical path:** `absolute`, alone.

## What was decided

- **What the survey found beyond the request's four changes** (its change 5):
  `tests/boundary/Cargo.toml`'s description says "beyond its inventory", and
  `temper/docs/design/domain/testing.status.md` says the test "counts" the `~/` form. Both are in
  the task. `outward.txt` is otherwise cited only under `*/docs/plan/`: `qa-out` and
  `retired-plans-out`. Those stay; the outward check and the citations checker both leave them out.
- **`*/docs/plan/<draft>/` is read strictly:** one directory, `docs/plan`, a draft, something inside
  it. The check today leaves out any path holding `/docs/plan/`, and a root `docs/plan/`. No tracked
  file sits in the difference, so the narrowing passes at the base.
- **Symlinks:** the check skips them today, and the request says nothing is excluded but plan
  directories, the test's own file and binary files. The task scans a link's target path, which is
  the text git stores, rather than keep a fourth exclusion. The one tracked link holds no name.
- **`the polls of real runs`** replaces `the dogfood's polls` in `smith/docs/design/shell.md`, 6.5.
- **The design lines go in the task,** word for word, rather than in a separate cut: the request asks
  for one task, and neither line closes a status entry or a draft's gap. The spec states this as its
  departure from the implementer's text.
- **`medium` effort:** the work is removal and small edits, with the code sketched in the spec.

## The base

A replay of the matcher over the tracked files at `d4bd15e25b` found no match in 16,905 text files,
the link's text included, so the absolute rule holds from the start. If a merge brings a match, the
task cleans that file. `citations.py --all` reports 0.

## Overlaps with the drafts beside this one

- **`sign-in-wait` (queue):** its spec adds a `sign_in_time` row to `smith/docs/design/shell.md`, 6.5,
  the table holding the `job_wait` row. The base already holds a `sign_in_time` row, twelve lines
  below. One-line edits that far apart merge cleanly. This overlap could not be avoided: the request
  names the row.
- No other listed task of `fabrig-scale`, `debt-5`, `bugs-temper` or `debt-6` touches this task's
  files. `v0.1.0-ladder` lists no tasks yet.
