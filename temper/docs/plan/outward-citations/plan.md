# outward-citations: the plan

From `inputs/request.md` (survey/agents-refs, step 2 of 12, approved by the user): the
citation checker stops excusing another repository's documents but hearth's, reads what it
skipped, and the citations it then shows are said in fabrig's own words. One slice, two direct
parts, one task each, both released in the first push and run at once.

## Parts and tasks

1. **checker** (`tools/citations/README.md`), slice 1.
   - `tasks/strict-checker.md`, high, ~50 minutes, needs nothing: `EXTERNAL` keeps hearth's
     branch alone; a new `OUTSIDE` form refuses a path straight after "<name> repository's"
     (no article or determiner, not hearth) as a finding `outside`, never resolved, since the
     suffix fallback would accept a same-named local file; `walk` reads the root `DESIGN.md`
     and `tools/**/*.md` and leaves every file under `docs/plan/` out; five tests (the agents
     test flipped); the README rewritten to match; `allow.txt` loses its "Evidence outside the
     repository" block and gains one for the README's own examples.
2. **wording** (`AGENTS.md, 2`), slice 1.
   - `tasks/outward-wording.md`, medium, ~35 minutes, needs nothing: the 16 lines of
     `notes/findings.md` reworded as the spec gives them, word for word (six scripts' comments,
     `AGENTS.md` 8, two drafts' lines, the dashboards draft's four, two Rust comments with the
     date of their measurement, three drafts' home-directory survey paths).

## Decisions

- **Measured, not guessed:** a scratch copy of the checker with the change applied, run at
  `4c830578a7`, gave the 29 findings of `notes/findings.md` (23 with `allow.txt` unchanged); each
  has one owner. Nothing else in the repository shows: the root `DESIGN.md`, the other
  `tools/` Markdown and every `plan.toml` comment are clean.
- **The refusal names no repository.** The request wants the refusal explicit, since `resolve`
  falls back by suffix. Matching the second repository's own phrasings would name it in the
  checker, and its bare possessive ("agents'") also reads "its agents' questions": so `OUTSIDE`
  is generic ("<name> repository's" before a path, articles and determiners excluded). Measured:
  it fires on the three lines that cite that repository and on none of the 40-odd "a
  repository's" phrases. A bare possessive ("the probe's `x.md`") is now resolved like any
  path, so it fails when the file is not here.
- **"The Evidence outside the repository" block goes whole:** its six entries are all records
  kept outside the repository (two walk reports, a QA brief, two surveys and an investigation
  in a home directory). The two Rust comments state the observed fact with the date of the
  commit that recorded it (2026-10-27); the three drafts already date their survey in the same
  sentence, so their path goes. The other blocks stay, `dashboards.md status-page.md` included:
  the draft keeps `status-page.md` as its test's data, described in its own words.
- **The README's examples** (`x.md`, `../../a.md` and their sections) are listed in `allow.txt`,
  scoped to the README with their printed targets, rather than reworded: the README teaches
  those forms. The task may reword one instead where an entry would excuse more.
- **Two tasks in parallel, no needs:** the wording task's acceptance is today's checker at 0 and
  its lines as given; the checker task's is `--all` printing only the wording task's findings
  until that merges. `allow.txt` has one writer (`strict-checker`), so the two never touch one
  file. Neither builds anything: no Rust changes, so no heavy, clippy or nextest
  (`notes/common.md`).
- **No design document changes:** the checker's design is its README, which its task rewrites to
  the spec's text; the drafts' lines change in wording only, no section moves, so no status
  entry changes.

## Overlaps with other drafts

None of the drafts beside this one names a file these tasks change. Two files are shared with
running work in practice: `temper/docs/design/draft/reference-case.execution.md`, whose section
3 the reference walks update as milestones land (this plan edits a line of section 2), and
`AGENTS.md`, every agent's source (this plan edits one parenthesis in section 8). Neither touches
the same lines.

## Critical path

`strict-checker` (~50 minutes) beside `outward-wording` (~35), then the trunk's end check:
`citations.py --all` at 0 and `test_citations.py` green.
