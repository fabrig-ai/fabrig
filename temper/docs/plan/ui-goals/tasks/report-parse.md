+++
id = "report-parse"
part = "report-parse"
kind = "feature"
needs = []
files = [
  "temper/crates/temper-client-domain/src/pages/report.rs",
  "temper/crates/temper-client-domain/src/tests/pages/report.rs",
]
effort = "high"
minutes = 40
+++

# A goal's report, read

## Goal

A pure helper reads a goal's report note. Nothing draws it yet, and no page reads it yet.

Acceptance:

- `pages::report` holds `OPENING`, `Report`, `parse`, `read` and `newest`, exactly as
  `temper/docs/plan/ui-goals/notes/contracts.md`, The report, gives them.
- Its tests below pass, each one bitten once by hand.

## Inputs

- `temper/docs/plan/ui-goals/notes/common.md`, `temper/docs/plan/ui-goals/notes/contracts.md` (The report).
- `temper/docs/design/domain/agent.md`, 8.3.
- `temper/docs/design/client/README.md`, 3.5, *A goal's report*.
- `jig/crates/jig-client-domain/src/contract/documents.rs`, `Note`; `contract/reads.rs`, `Read::Notes`.

## Owns and may change

Owns, as `files` lists them.

May also change, one line each: `temper/crates/temper-client-domain/src/pages.rs` (`pub mod report;`, in
its order), `temper/crates/temper-client-domain/src/tests/pages.rs` (`mod report;`).

Options, each pre-approved:

- If `Text::new` needs a bound for `OPENING`, use the description's own: `read` must not be fallible for
  its caller, so `expect` it where the subset allows, with a message saying the opening fits.
- If a fixture for `c::Note` exists in `src/tests/fixtures*`, use it; if not, the test file makes its own.

## Steps

1. **`pages/report.rs`,** a step module under the subset; its module comment cites
   `temper/docs/design/client/README.md`, 3.5. Bytes alone: never assume a body is whole UTF-8 past what
   `DocumentText` gives. No panic on any note: a description without the opening reads as its first line.
2. **Tests, `tests/pages/report.rs`:**
   - `a_whole_report_reads_in_its_parts`: headline, two paragraphs, `Left:`, `Risk:`, and `Decided: first,
     05:40 · second, 05:12` as two items; `at` the note's `changed`.
   - `every_line_is_optional`: an empty body, and each of the three alone.
   - `only_a_lines_start_makes_a_field`: "We saw Left: inside a line." stays a paragraph.
   - `a_cut_body_reads_as_far_as_it_goes`: a body marked cut, ending inside a `Left:` line, and one ending
     inside a paragraph.
   - `a_description_without_the_opening_reads_whole`.
   - `newest_takes_the_opening_and_the_latest_change`: a note naming `report: ` mid-description is passed
     over, `changed` none is oldest, no row is `None`.
   - `its_read_names_the_goal_and_the_opening`.
3. **Check:** `cargo fmt`; `heavy cargo clippy -p temper-client-domain --all-targets -- -D warnings`;
   `heavy cargo nextest run -p temper-client-domain --lib`. Commit and push.

## Report

As `notes/common.md`, Your report.
