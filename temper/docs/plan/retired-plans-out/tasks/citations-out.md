+++
id = "citations-out"
part = "citations-out"
kind = "docs"
needs = []
files = [
  "temper/docs/design/draft/reference-code-production.status.md",
  "temper/docs/design/protocol/configuration.status.md",
  "temper/docs/design/protocol/credentials.md",
  "temper/docs/design/draft/first-run-projects.status.md",
  "temper/docs/design/draft/first-release.status.md",
  "temper/docs/development/pre-pr-numbers.md",
  "temper/docs/design/draft/reference-case.execution.md",
  "temper/docs/design/draft/configuring.execution.md",
  "jig/crates/jig-protocol-config/src/charters.rs",
]
effort = "high"
minutes = 45
+++

# The citations into the retired plans, rewritten as the facts they stood for

## Goal

Before the 49 directories leave main, every citation into them from outside them becomes the
fact it stood for, with its date or commit (the request's Task 1, second half). The survey at
this plan's cut (`ae878830b8`) found no `docs/plan/<name>/` path outside the plans; the
citations name the plans instead. They are the places of `## Steps`, 1 to 9, in nine files.

Acceptance:

- Each place of `## Steps` reads as its step says, naming no plan and nothing of the lab
  (`notes/common.md`, "What text added outside `docs/plan/` may say"); nothing else in those
  files changes but the date lines of the status files edited.
- `## Steps`, 10's search prints only the lines it lists as staying.
- The checks of `notes/common.md`: citations at 0;
  `heavy cargo nextest run -p fabrig-boundary --test outward` passes. For the comment line
  (`## Steps`, 9): `cargo fmt --check -p jig-protocol-config` (builds nothing) is clean, and
  `python3 -I -B scripts/old_forms.py` run from `jig/` exits 0 with no `--write`.
- Pushed.

## Inputs

- `temper/docs/plan/retired-plans-out/notes/common.md`: the standing rules.
- `temper/docs/plan/retired-plans-out/inputs/request.md`, "Task 1" (the citations' list and its
  example forms).
- `DESIGN.md`, 2 and 3: status files and execution files.
- The sources each step names, in the 49 directories, read only for the fact the step needs.

## Owns and may change

- **The nine files above,** only at the places `## Steps` names, plus each edited status
  file's date line (`notes/common.md`). `configuration.status.md`'s line 6 is one long line that
  another draft's trunk edits around (its lines 5 and 7): change only the words the step names.
- **`jig/crates/jig-protocol-config/src/charters.rs`:** the one comment line of `## Steps`, 9,
  and nothing else; the request allows documentation only, and a comment changes no code. This
  departs from the implementer's text: no clippy or test run for it, the checks above instead.
- **The landing commit** of a done cell is the first-parent merge of that plan's trunk on main
  (`git log --first-parent --format='%h %cI %s' origin/main | grep 'Merge <trunk>'`). If you
  cannot find one, the cell keeps its date and time alone, and the report says which.
- **A wording the step sketches** may be tightened to fit its sentence; the fact, its date and
  the absence of the plan's name are what bind.
- **A further citation into the 49** that Step 10's search shows and this spec does not list:
  rewrite it the same way and name it in the report; if it is in a file outside the
  allowed targets of `notes/common.md`, leave it and list it under `## Left`.

## Steps

1. **`temper/docs/design/draft/reference-code-production.status.md`** (a draft's status;
   **Design departs** entries):
   - line 18, "Decided by reference-m9's cut, 2026-11-03." becomes "Decided 2026-11-03.";
   - lines 24 and 25, "Decided by reference-m5's cut, 2026-11-03: the reference sets the
     foreman's model on its charter ..." becomes "Decided 2026-11-03: the reference sets ...",
     the rest as it is.

2. **`temper/docs/design/protocol/configuration.status.md`**, line 6, the 13.9 sentence's end:
   "who writes them, and the runner's budget they take, are the user's (plan `reference-m10`'s
   question `case-workflows`)." The question is open; it keeps its options (the request). Read
   them in `temper/docs/plan/reference-m10/plan.toml`, `[[question]]` `case-workflows`, and
   write them without the lab's names and without the recommendation, e.g.: "who writes them,
   and the runner's budget they take, are the user's, open since 2026-11-03: the walk's copy of
   fabrig carries them, written for the box that runs the case, CI running the gate's nextest
   profiles whole, `tier-goal.yml` the gate without its slowest suites and `tier-slow.yml` the
   slow tiers, within a runner budget the user sets; or fabrig carries them under
   `.forgejo/workflows/` once that budget is set; or the case runs on a smaller repository
   than fabrig, whose workflows are cheap."

3. **`temper/docs/design/protocol/credentials.md`**, lines 274 and 275: "(the director's
   answer of 2026-11-03, `forgejo-connect`)" becomes "(decided 2026-11-03)". A design
   document: no date line to change.

4. **`temper/docs/design/draft/first-run-projects.status.md`**, line 6: "(plan
   `first-run-projects`, `inputs/request.md`)" becomes "(2026-11-03)", so that the sentence
   reads "For now, while the user is away (2026-11-03), Ready opens ...".

5. **`temper/docs/design/draft/first-release.status.md`:**
   - line 11, "Decided by the cut (plan `public-reading`, 11-03) while the user is away"
     becomes "Decided 2026-11-03 while the user is away";
   - line 15, "decided conservatively while the user is away (plan `public-reading`,
     `inputs/request.md`)" becomes "decided conservatively while the user is away
     (2026-11-03)".

6. **`temper/docs/development/pre-pr-numbers.md`**, lines 5 to 7: the sample "in 14 plans,
   2026-10-30 to 11-03 (reference-m1 to m9, configuring-p1, first-run-projects,
   withdrawn-forms, playground-reset r4 and r6)" becomes "in 14 plans landed on main from
   2026-10-30 to 11-03", the list of names dropped; the rest of the paragraph stays (its "fab's
   `<draft>/integration`" included: lab names in it are another step's).

7. **`temper/docs/design/draft/reference-case.execution.md`** (an execution file; `DESIGN.md`,
   3), section 3's table, the State cells of M1 to M6 and M8 to M10 (lines 95 to 100 and 102
   to 104): "(plan `reference-mN`[, n tasks])" becomes the landing merge's short hash in
   backticks, e.g. M1's "done: landed 2026-11-03 02:21Z (plan `reference-m1`, 24 tasks)"
   becomes "done: landed 2026-11-03 02:21Z (`d1d49bd206`)". The rest of each cell stays ("its
   walk folds into M3's", "first walks blocked by a chat finding (finding-111)", "its walk is
   rung C"). M7's cell names no plan and stays. The merges at the cut, for checking:

   | M1 `d1d49bd206` | M2 `b909b01fd7` | M3 `224965929f` | M4 `0ca3509cbf` | M5 `916b6dbfdb` |
   |---|---|---|---|---|
   | **M6** `fcbde2d586` | **M8** `2841e59d0a` | **M9** `677e405ca1` | **M10** `4c739315c4` | |

   Line 223, "P1 (plan `configuring-p1`, started 2026-11-02)" becomes "P1 (started
   2026-11-02)".

8. **`temper/docs/design/draft/configuring.execution.md`**, section 3's table, the State cells
   of P1 to P3 (lines 48 to 50) in the same form: P1 `be16409150`; P2 "(plans `configuring-p2`,
   design, and `configuring-p2-build`, 14 tasks)" becomes "(`bfa287bbc4`)", the build's merge,
   the design having landed with it; P3 `a6a8155d35`. P4's cell (line 51, "plan
   `configuring-p4`", whose directory left main at `6bed982725`) takes the same form,
   `062fc5fb1f`, so the table reads alike.

9. **`jig/crates/jig-protocol-config/src/charters.rs`**, line 122:
   `// Its format carries no mask yet (configuring-p3, contracts C1).` becomes
   `// Its format carries no mask yet: every family (jig/docs/design/domain/engine.md, 7.1).`
   The fact: a charter's `tools` is a `ToolFamilies` mask (engine.md, 7.1), and until a format
   carries it a decoder fills `RunPolicy::EVERY_FAMILY`. If the line is no longer there or says
   otherwise, leave the file and report it.

10. **The search.** From the repository root, after committing:

    ```sh
    names='wasm32-jig|playground-skein|wasm32-skein|live-smith-manual|live-smith-reds|live-smith-reds-2|reference-m4|citation-burndown|collector-starved|configuring-p1|configuring-p2|configuring-p2-build|configuring-p3|fab3-fixes|first-run-projects|forgejo-connect|github-repair-race|interrupted-calls|live-temper|live-temper-stories|live-temper-stories-2|look-welcome-chats|me-retention|open-default|openai-manual|phase-case|playground-decisions|playground-temper|public-reading|readiness-eintr|reference-m1|reference-m2|reference-m3|reference-m5|reference-m6|reference-m7a|reference-m7b|reference-m8|reference-m9|reference-m10|repo-links|status-gaps|status-lines|wasm32-list|wasm32-more|wasm32-rustls|wasm32-temper|wasm32-temper-2|wasm32-temper-3'
    git grep -nE "(^|[^a-z0-9-])($names)([^a-z0-9-]|$)" -- . ':!*/docs/plan/*'
    ```

    What stays, being no citation of these directories:
    - `live-temper`, the nextest profile: `.config/nextest.toml`, `.config/fixture-readers.toml`,
      `tools/tiers/packages.toml`, `tools/tiers/test_cpu.py`, `temper/scripts/proof-live*.sh`,
      `temper/crates/temper/tests/suite/forge_land.rs`, `temper/docs/design/domain/testing.md`,
      `temper/docs/development/v0.1.0-ladder.md`;
    - `first-run-projects`, the draft `draft/first-run-projects.md` and its status file's own
      name: `temper/docs/design/STATUS.md`, `client/README.md`, `client/README.status.md`,
      `client/forge.md`, `client/mockups/STATUS.md`, `protocol/setup.md`, and lines 1 and 3 of
      `draft/first-run-projects.status.md`;
    - `phase-case` in `temper/docs/design/draft/notes.md` (lines 103 and 153 to 156, section
      2's landings on record, and line 424, N16's "phase-case's retire"): events of the queue's
      records and main's history, not the directory; a draft's notes are not this task's;
    - `public-reading` in `temper/docs/design/draft/dashboards.md`, line 423: the draft's
      question on its branch, which the directory does not hold.

    Any other line is a citation the spec missed: `## Owns and may change` says what to do.

11. Commit (one per tree is enough: temper's docs, jig's comment), each saying why, and push.

## Report

`Done:` and the places changed, by file and line. Then the search's output beside the list of
what stays; any citation found beyond this spec and what you did; the checks' results (the
outward test's count); your minutes.
