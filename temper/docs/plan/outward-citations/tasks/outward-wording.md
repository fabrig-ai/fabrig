+++
id = "outward-wording"
part = "wording"
kind = "docs"
needs = []
files = ["AGENTS.md", "skein/scripts/browser.sh", "jig/scripts/browser.sh", "jig/scripts/live-sign-in.sh", "temper/scripts/proof-live.sh", "temper/scripts/proof-live-github.sh", "temper/scripts/proof-live-github-managed.sh", "temper/docs/design/draft/dashboards.md", "temper/docs/design/draft/reference-case.execution.md", "temper/docs/design/draft/macos.md", "temper/docs/design/draft/upgrades.md", "temper/docs/design/draft/archive/first-run.md", "skein/crates/skein-fake-forge-protocol/src/answers.rs", "temper/crates/temper/tests/support/csv_repair.rs"]
effort = "medium"
minutes = 35
+++
# Citations of documents kept outside the repository, said in fabrig's words

## Goal

Sixteen lines cite documents kept outside the repository: another repository's design
documents, a walk's reports, a QA brief, surveys in a home directory. Once the citation
checker stops excusing them (the `strict-checker` task, in parallel), each is a finding. Reword
each as below: the citation goes, and where the sentence leaned on it, it states the fact itself
with its date. Wording only: no behaviour of any script or test changes.

Acceptance:

- Each of the 16 `outward-wording` findings in `notes/findings.md`, and `dashboards.md` lines 58
  and 370, reads as step 1 gives it; `git diff` changes comments and prose only.
- After your last commit, `python3 -I -B tools/citations/citations.py --all` on your branch
  reads 0, with the checker at your base. If `strict-checker` has merged into your base, it
  reads 0 with that checker too.
- No line you edited names another repository, a path outside the repository or a record kept
  there. Other lines of these files do (script code that reads `$HOME/src/rust/tmp/`, `AGENTS.md`
  3 and 8, `reference-case.execution.md` 3's old code, "its agents' questions"): they stay.
- `cargo fmt --check -p skein-fake-forge-protocol -p temper` passes, `sh -n` passes on each
  script you changed, and `python3 -I -B scripts/old_forms.py`, run from the `skein`, `jig` and
  `temper` roots, raises no count.

## Inputs

- `temper/docs/plan/outward-citations/notes/common.md`, then `notes/findings.md`.
- `temper/docs/plan/outward-citations/inputs/request.md`, items 3 and 4: the request.
- `AGENTS.md`, 2: how a comment speaks.
- `DESIGN.md`, 4: a section keeps its number; these edits renumber nothing.

## Owns and may change

- Owns the lines named in step 1 of the files in `files`, and no other line of them.
- **Departures from your implementer's text,** each because the request (approved by the user)
  asks for exactly these edits: you edit `AGENTS.md`, drafts (`temper/docs/design/draft/`) and
  one archived draft, in the lines below only. No Rust changes, so no build, clippy or nextest:
  `notes/common.md` gives the checks.
- Not yours: `tools/citations/**` (`strict-checker`'s, `allow.txt` included); any other line
  that names the other repository without citing a document (`AGENTS.md` 5 and 8 name it, for
  instance): leave them.
- Options, pre-approved:
  - **A line moved at your base:** find it by its text; if its text changed, apply the same
    edit to its current words and say so.
  - **Rewrapping:** rewrap the edited paragraph or comment to its neighbours' width; nothing
    outside it.

## Steps

1. Reword, each as given (the paragraph rewrapped to its width):
   - **`skein/scripts/browser.sh`, 2-5:**

     ```sh
     # The browser tier, outside the gate: the UI example's bundle and server,
     # then the browser suite (ui-web.md, 14.1). Its arguments go to nextest,
     # so that one test runs alone: scripts/browser.sh -E 'test(=NAME)'.
     ```

   - **`jig/scripts/browser.sh`, 2-5:**

     ```sh
     # The browser tier (protocol/browser.md, 15), outside the gate: jig's test
     # application's bundle and jig-test, then the browser suite. Its arguments
     # go to nextest, so that one test runs alone:
     # scripts/browser.sh -E 'test(=NAME)'.
     ```

   - **The build-queue comment,** two lines in each of `jig/scripts/live-sign-in.sh` (7-8),
     `temper/scripts/proof-live.sh` (6-7), `proof-live-github.sh` (6-7) and
     `proof-live-github-managed.sh` (8-9), becomes one:

     ```sh
     # The machine's build queue where it has one, plain cargo where it hasn't.
     ```

   - **`AGENTS.md`, 71 (section 8):** "fab runs the pipeline (the agents repository's
     `docs/design/fab.md`): a cut, ..." becomes "fab runs the pipeline: a cut, ..."; the rest of
     the line as it is.
   - **`temper/docs/design/draft/reference-case.execution.md`, 43-45 (section 2):** "under
     fab's rules for a run's lead and its designers (the agents repository's
     `fab/roles/cut-run.md`):" becomes "under fab's rules for a run's lead and its designers:".
   - **`temper/docs/design/draft/dashboards.md`**, where the status page's document is the
     draft's example and its test's input:
     - 9-10: "The page pointed at is https://status.fabrig.ai, a status page whose description
       is the test's input (section 9)."
     - 58-60: "- **Its test:** given `status-page.md`, a description of status.fabrig.ai's
       page, the agent proposes a dashboard; what it cannot show is the list of facts temper
       does not keep (section 9)."
     - 197-200: "- **The projection is the status page's pace:** the rise over the look-back,
       then either full before the period's end, at when, or what is left at its end."
     - 370-371: "- **The status page as a golden.** `status-page.md`, the description of
       status.fabrig.ai's page, goes through a real model as the person's description, ..."
       and the rest of the item as it is.
   - **`skein/crates/skein-fake-forge-protocol/src/answers.rs`, 368-369:**

     ```rust
         // Forgejo 17 combines only the requested page: a walk against it, recorded on
         // 2026-10-27, saw a success hide a job still pending on a later page.
     ```

   - **`temper/crates/temper/tests/support/csv_repair.rs`, 21-22:**

     ```rust
     // The release-note gate a walk against Forgejo proved, recorded byte for byte on
     // 2026-10-27. No action or external service.
     ```

     (Both dates are those of the commits that wrote the measurements into the repository:
     `7ddc7f8620` and `b152011097`.)
   - **The drafts' surveys,** whose sentences already carry their date; the home-directory
     path goes:
     - `temper/docs/design/draft/macos.md`, 5-7: "which started from a survey of what stands
       between temper and macOS (every claim with its file and line)."
     - `temper/docs/design/draft/upgrades.md`, 6-7: "It comes from the investigation of
       2026-10-23 (Codex, read-only) and a session with the user that day, ..."
     - `temper/docs/design/draft/archive/first-run.md`, 5-7: "which started from the
       assessment of where the first run stands once the accounts and GitHub work land. It
       crosses ..."
2. Run the checks of the acceptance; commit ("Say outward citations in fabrig's words"); run
   `--all` (acceptance); push.

## Report

`notes/common.md`'s report; each edit's `file:line` after it; any line found moved or changed
at your base and what you did; the outward check's result if it exists at your base.
