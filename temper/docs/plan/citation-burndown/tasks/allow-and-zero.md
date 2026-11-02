+++
id = "allow-and-zero"
part = "zero"
kind = "docs"
needs = ["jig-store-items", "skein-forge-readme", "temper-citations", "checker-readings"]
files = ["tools/citations/allow.txt"]
effort = "medium"
minutes = 15
+++
# allow.txt for what must stay, and the count at zero

## Goal

The last task of the burndown: list in `tools/citations/allow.txt` the 17 findings that are no
citation and must stay (examples and data quoted as such, evidence outside the repository,
history kept on purpose), each with its reason, and check that
`python3 -I -B tools/citations/citations.py --all` exits 0 on your branch, which starts from the
trunk with the other four tasks merged.

Acceptance:

- `allow.txt` holds the lines below, under its two header lines, in the format the checker reads
  (`FILE TARGET # reason`; the checker refuses a line without a reason, exit 2).
- After your commit, `citations.py --all` prints `citations: 0 broken in HEAD` and exits 0.
- `python3 -I -B tools/citations/test_citations.py` passes.

## Inputs

- `temper/docs/plan/citation-burndown/notes/common.md`, then `notes/findings.md`, whose
  "allow-and-zero" list these lines excuse.
- `tools/citations/README.md` (`allow.txt`).
- The other tasks' reports, where fab gives them, for anything they left under `## Left`.

## Owns and may change

- Owns `tools/citations/allow.txt`.
- Options, pre-approved, for a finding still printed once the 17 lines are in:
  - Not a citation, and it must stay (data, an example, evidence outside the repository): one
    more `allow.txt` line with its reason.
  - A real break (a dead path or a missing section) in a Markdown file that
    `.config/gate/read-by-tests.txt` does not list, or in a code comment: fix its text, naming
    the file and section that carry the content now, text only (`notes/common.md`), whatever
    the tree; list each edit in the report.
  - Anything else (a misreading of the checker, or a real break in a file tests read): stop with
    a `Question:`, giving the finding and what you would do.

## Steps

1. Append to `tools/citations/allow.txt`, keeping its two header lines:

   ```text
   # Data, examples and names quoted as such.
   smith/docs/design/benchmarks.md CODEX_HOME/AGENTS.md # codex-cli's file under CODEX_HOME, not a document here
   temper/crates/temper-engine-domain-forge-landing/src/tests/key.rs crates.md # a test's path, as data
   temper/docs/design/client/README.md process.md # a charter's document in an example, quoted
   temper/docs/design/domain/landing.md crates/x/README.md # an ignored path, as an example
   temper/docs/help/landing.md crates/x/README.md # an ignored path, as an example
   temper/docs/design/domain/testing.md changes/1.md # the walk repository's file, as data
   temper/docs/design/draft/configuring.md process.md # a charter's document in an example, quoted
   temper/docs/design/draft/dashboards.md status-page.md # an adopted repository's document, the test's data
   # Evidence outside the repository.
   jig/docs/development/quality-review.md agents/backlog.md # the agents repository's backlog
   skein/crates/skein-fake-forge-protocol/src/answers.rs ladder-b01/g2/report-2.md # a ladder walk's report, kept outside the repository
   temper/crates/temper/tests/support/csv_repair.rs ladder-b01/g1/report.md # a ladder walk's report, kept outside the repository
   temper/crates/temper/tests/support/csv_repair.rs qa-focused/rung-b1-brief.md # a QA brief, kept outside the repository
   temper/docs/design/draft/archive/first-run.md /src/rust/tmp/temper-first-time-ux-2026-10-20.md # a survey kept outside the repository
   temper/docs/design/draft/macos.md /src/rust/tmp/macos-survey-2026-10-22.md # a survey kept outside the repository
   temper/docs/design/draft/upgrades.md /src/rust/tmp/roadmap-3/store-upgrades.md # an investigation kept outside the repository
   # History kept on purpose.
   temper/docs/design/draft/notes.md status.md # the earlier pipeline's file names, as history
   temper/docs/design/draft/notes.md landing-model.md # the retired draft, readable at the commit it names
   ```

   TARGET is the checker's, as it prints it: the `~/src/rust/tmp/...` paths print from their
   first `/`.
2. Commit; run `citations.py --all`. For each finding left, apply the options above; commit
   again.
3. Run `test_citations.py`; push.

## Report

`notes/common.md`'s report: `--all`'s last line, and each line or text fix beyond the 17. The
first line, once `--all` exits 0: `Done: citations.py --all exits 0 on the trunk`.
