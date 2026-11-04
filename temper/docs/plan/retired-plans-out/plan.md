# retired-plans-out: plan

Request (`inputs/request.md`, step 4 of 12 of a user-approved plan): the 49 retired plan
directories under `*/docs/plan/` leave main, after one pass that keeps in the design any
decision, contract or open choice recorded only there, and rewrites every citation into them
as the fact it stood for. Documentation only; no live plan changes.

## Parts and tasks

All in slice 1; released together in the first push.

- **citations-out:** `tasks/citations-out.md`, no needs, ~45 min. The places outside the
  plans that cite the 49, in nine files: two draft status files' "decided by <plan>'s cut",
  `configuration.status.md`'s `case-workflows` (its open choices kept), `credentials.md`'s
  answer, the first-run and first-release status lines, `pre-pr-numbers.md`'s sample, the two
  execution tables' done cells (the landing merge's hash in place of the plan), and one
  comment line in `jig-protocol-config/src/charters.rs`.
- **audit:** four readers, no needs, in parallel, each writing a kept note in `notes/`
  (`notes/contracts.md`, K1) and nothing else:
  - `tasks/audit-goal.md` (~90 min): reference-m1, m2, m3, m4, m6, phase-case,
    github-repair-race, status-gaps, status-lines, readiness-eintr, interrupted-calls;
  - `tasks/audit-landing.md` (~95 min): reference-m5, m7a, m7b, m8, m9, m10;
  - `tasks/audit-configuring.md` (~90 min): configuring-p1, p2, p2-build, p3,
    first-run-projects, forgejo-connect, open-default, look-welcome-chats, public-reading,
    me-retention, repo-links, fab3-fixes;
  - `tasks/audit-playground.md` (~95 min): playground-skein, playground-temper,
    playground-decisions, the eight wasm32 plans, live-smith-manual, live-smith-reds,
    live-smith-reds-2, live-temper, live-temper-stories, live-temper-stories-2, openai-manual,
    citation-burndown, collector-starved.
- **keep-decisions:** `tasks/keep-decisions.md`, needs the four readers and citations-out,
  ~100 min. Writes each finding into the design document or status file it concerns, dated,
  and lists the additions in its commits.
- **plans-out:** `tasks/plans-out.md`, needs keep-decisions, ~20 min, medium. Deletes the 49
  present; citations at 0, `fabrig-boundary` green, old-forms unchanged.

**Critical path:** a reader (~95 min) → keep-decisions (~100) → plans-out (~20), about 3.6
hours of implementer time plus fab's merges; citations-out runs beside the readers.

## Decisions

- **The request's Task 1 runs as four readers and one writer.** What it names to read is
  about 12,300 lines: 2,554 lines of `## Answers` in 97 task files, about 4,000 of `plan.md`,
  4,100 of `notes/contracts.md`, 1,700 of `notes/common.md`, 400 of other notes, five
  `plan.toml` questions. One implementer cannot weigh that against the design with care in one
  context. Four readers, one per family of plans (2,800 to 3,200 lines each), work at once and
  write findings only. They do not write the design: its status files hold long one-line
  bullets, so four writers into the same files would conflict at each merge. One writer,
  needing all four, writes the design. The kept notes stay in this plan's directory and leave
  main with it. The order of the request (the audit, then the deletion needing it) holds.
- **The kept note's form and the rules for what is kept** are `notes/contracts.md`, K1 to K3:
  each item Held, Superseded, Not design (how the work was run, or a code shape the code now
  carries) or Kept (a product decision, a boundary contract, an open choice); a finding carries
  its source, what was checked, its target and its text, ready to paste. Unsure is Kept; the
  writer decides.
- **The citations are a task of their own,** beside the readers: the survey at the cut
  (`ae878830b8`) found every one. None names a `docs/plan/<name>/` path; they name the plans.
  The places: `draft/reference-code-production.status.md` 18 and 25;
  `protocol/configuration.status.md` 6; `protocol/credentials.md` 275;
  `draft/first-run-projects.status.md` 6; `draft/first-release.status.md` 11 and 15 (11 was
  not in the request's list); `temper/docs/development/pre-pr-numbers.md` 6;
  `draft/reference-case.execution.md` 95 to 100, 102 to 104 and 223;
  `draft/configuring.execution.md` 48 to 50; `jig/crates/jig-protocol-config/src/charters.rs`
  122. Small and certain, it lands first and the writer starts from it.
- **Not citations, and staying:** `live-temper` the nextest profile; `first-run-projects` the
  draft; `phase-case` in `draft/notes.md` (fab's records of a landing and a retire, not the
  directory); `public-reading` in `draft/dashboards.md` 423 (the draft's question on its
  branch; the directory holds no such question); the path patterns
  `temper/docs/plan/configuring-<phase>` and `temper/docs/plan/<milestone>` in the execution
  files, which say where a running plan sits.
- **The one non-Markdown edit** is a comment in jig's `charters.rs` that cites
  "configuring-p3, contracts C1": rewritten to cite `jig/docs/design/domain/engine.md`, 7.1,
  with no build (a comment changes no code; old-forms checked).
- **Done cells name the landing merge** on main's first-parent history (M1 `d1d49bd206` and
  so on; the spec lists them). P4's cell, whose plan directory already left main
  (`6bed982725`), takes the same form so the table reads alike.
- **What added text may say** (`notes/common.md`): no plan, no role of the lab, no name of the
  lab; "the user" where the source says the user decided, else the date alone. The outward test
  guards the lab's names; a grep in each spec guards the rest.
- **Measured at the cut:** citations `--all` 0; deleting the 49 in the cut's worktree (then
  restored) left every tree's old-forms count as it was (old-forms counts dead Markdown paths
  in Rust comments);
  `outward.rs` and every check leave `docs/plan/` out, so nothing else changes with the
  deletion; no other trunk adds a mention of the 49.
- **No design section is written by this cut.** The design writes are the tasks'
  (citations-out, keep-decisions), from the plans' own records.
- **No question for the user.**

## Overlaps with the drafts beside this one

- **`temper/docs/design/protocol/configuration.status.md`, line 6** (citations-out, step 2):
  fabrig-scale's trunk edits lines 5 and 7 (`brief-room`, `init-standard`, `reference-trim`,
  `stock-scale` list the file; debt-1's `temper-reconcile-gone` too, merged on its trunk).
  Adjacent lines: whichever trunk lands second meets a conflict, a catch-up. Unavoidable: the
  request names the line. The spec keeps the edit to the sentence's last words.
- **`temper/docs/design/draft/reference-case.execution.md`:** outward-citations'
  `outward-wording` (merged on its trunk) edits section 2; this plan edits section 3's table
  and line 223: separate hunks.
- **keep-decisions' targets** are known only once the notes are written. Many likely ones are
  files fabrig-scale, debt-1, playground-browser and own-rules are changing (`notes/common.md`
  lists them); the writer adds lines of its own there and rewrites none.
- **`public-reading-build`** (landed on main 2026-11-03 23:42Z, `a26016f215`; its retire not
  yet run) names `temper/docs/plan/public-reading/plan.md` and `notes/next-cut.md` among its
  `plan.toml` sources. plans-out deletes them, as the request says, which states that the
  plans' runner keeps its own copies. If fab reads a live plan's sources from main after this
  lands, those two are gone.
- **plans-out** changes only the 49 directories, which no other draft touches.
