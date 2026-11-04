# Request: the retired plans leave fabrig

From survey/agents-refs, 2026-11-04; the user approved the plan (step 4 of 12). The plan directories
under `*/docs/plan/` are run records of the lab that runs fabrig's development. Retired plans are
no longer read from fabrig (their runner keeps its own copies), and from now on each plan's
directory leaves main when it retires. This step removes the 49 retired ones still on main, after
one pass that keeps any decision recorded only there. Two tasks: the audit first, the deletion
needing it.

## The 49 directories
jig/docs/plan/wasm32-jig; skein/docs/plan/playground-skein, wasm32-skein;
smith/docs/plan/live-smith-manual, live-smith-reds, live-smith-reds-2, reference-m4;
temper/docs/plan/: citation-burndown, collector-starved, configuring-p1, configuring-p2,
configuring-p2-build, configuring-p3, fab3-fixes, first-run-projects, forgejo-connect,
github-repair-race, interrupted-calls, live-temper, live-temper-stories, live-temper-stories-2,
look-welcome-chats, me-retention, open-default, openai-manual, phase-case, playground-decisions,
playground-temper, public-reading, readiness-eintr, reference-m1, reference-m2, reference-m3,
reference-m5, reference-m6, reference-m7a, reference-m7b, reference-m8, reference-m9,
reference-m10, repo-links, status-gaps, status-lines, wasm32-list, wasm32-more, wasm32-rustls,
wasm32-temper, wasm32-temper-2, wasm32-temper-3.
Delete only those that exist at the task's base; **keep every other** `docs/plan/` directory
(live ones: reference-walks, v0.1.0-ladder, sign-in-wait, live-smith-reds-3, outward-rule,
public-reading-build, withdrawn-forms, and any plan started since, including this one).

## Task 1 (a docs task): keep what only a plan records
For each directory: its tasks' `## Answers`, its `plan.toml` `[[question]]`s and their answers,
`plan.md`'s decisions, `notes/contracts.md` and `notes/common.md`. A decision, a contract or an
open choice that the design (`<tree>/docs/design/`, its status files, `DESIGN.md`) does not already
hold goes into the design document or status file it concerns, dated, in that document's words,
with no reference to the plan or to who ran it. An open choice stays open (say its options). Most
are already in the design; list in the commit what was added and where.
Then every citation **into** these directories from outside them is rewritten as the fact it
stood for, with its date or commit, e.g.:
- status files that say "decided by <plan>'s cut" or cite `docs/plan/<x>/...`
  (`temper/docs/design/reference-code-production.status.md` ~l.11, 18,
  `temper/docs/design/protocol/configuration.status.md` ~l.6 (reference-m10's case-workflows
  question: keep its open choices), `temper/docs/design/protocol/credentials.md` ~l.275,
  `temper/docs/design/first-run-projects.status.md` ~l.6, `temper/docs/development/pre-pr-numbers.md` ~l.6);
- the execution drafts' tables citing plans as done records
  (`temper/docs/design/draft/reference-case.execution.md` ~l.95-104,
  `temper/docs/design/draft/configuring.execution.md` ~l.48-49): a done cell says the commit or date;
- any other found by `git grep -n 'docs/plan/<name>'` for each name above.

## Task 2: delete the directories
Remove the 49 directories (only those present). `tools/citations/citations.py --all` at 0 and
`tests/boundary/tests/outward.rs` green (it excludes `docs/plan/`; nothing else should change).

## Bounds
- Only documentation changes; no code.
- Nothing under a live plan's directory changes.
