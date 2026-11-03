+++
id = "stage-one-judge"
part = "judge"
kind = "feature"
needs = ["stage-one-surface", "stage-one-check"]
files = ["temper/tools/stage-one/src/judge.rs", "temper/tools/stage-one/src/tests/judge.rs", "temper/tools/stage-one/src/tests/fixtures/judge/**"]
effort = "high"
minutes = 120
+++

## Goal

Build the judge's side and the stage's verdict (`temper/docs/design/domain/testing.md`, 6.5.4):
`temper-stage-one judge EVIDENCE` prints the judge's brief whole; `temper-stage-one verdict
EVIDENCE [--json]` runs the check, decodes the judge's `judge.json` and gives the stage's verdict.
No LLM is called by code: the pass gives the brief to a session of its own.

Acceptance:

- The brief for the profile's evidence (`stage-one-check`'s `profile_evidence`, with a
  description, the person's documents and each named document added) matches its golden text.
- The verdict's decoder takes a whole verdict and refuses each bad one below, naming it; the
  stage passes only as 6.5.4 says.
- `heavy cargo nextest run -p temper-stage-one` passes; clippy clean.

## Inputs

- `temper/docs/design/domain/testing.md`, 6.5.4 (and 6.5.1's rules, 6.5.3's evidence).
- `temper/docs/plan/configuring-p4/notes/common.md`; `temper/docs/plan/configuring-p4/notes/contracts.md`
  (K3, K4, K5, K6).
- On the trunk: `golden.rs`, `evidence.rs` (`stage-one-surface`), `check.rs` and
  `src/tests/check.rs`'s `profile_evidence` (`stage-one-check`).

## Owns and may change

- Owns: `src/judge.rs` (the stub replaced whole), `src/tests/judge.rs`,
  `src/tests/fixtures/judge/**`.
- `src/tests/check.rs`: `profile_evidence` made `pub(crate)` if it is not; nothing else.
- Options, each pre-approved:
  - The golden text's bless: follow the convention temper's other golden-text tests use (find it
    with `rg` in `temper/crates/temper-client-tools/src/tests/`); if there is none to reuse, the
    test compares with `include_str!` and the report says how the fixture was made.

## Steps

1. **The brief**, `brief(golden, evidence, report) -> Result<String, JudgeError>`, Markdown, in
   this order:
   1. *Who you are:* the judge of configuring's stage 1, reading what code cannot; no tool use is
      needed; answer with one JSON object in the form at the end, nothing else.
   2. *The person's process:* `person/reference-code-production.md` and
      `person/reference-landing.md`, whole, each under its name.
   3. *What was accepted, by role:* the check's binding; for each charter role bound, its number,
      its instructions whole, and each document it names (its forge, repository, branch, path and
      heading) with its text from `documents/` (cut to the heading's section where it names one:
      from the heading's line to the next heading of its level or above).
   4. *The rules:* the golden's rules, a line each: id, role, words, source.
   5. *The agent's description* of the settings back as a workflow (`description.txt`), and the
      reference's section 1, "In one page", from the person's document, to read it against.
   6. *The proposal and the chat* (`proposal.txt`, `chat.txt`), for what needs a restart: the
      box's prerequisites (the golden's box obligations' words) proposed rather than said.
   7. *Your answer:* K6's JSON form, its `rules` listing every rule id once, with `verdict` one of
      `reaches` (its role's runs get it from the charter's instructions or a document it names),
      `missing`, `elsewhere` (another role's only), `where` and a `quote`.
   A missing `description.txt`, person document or document a bound charter names is a
   `JudgeError` naming it.
2. **The verdict:** `decode_verdict(bytes, golden)`, strict as K6 says; `stage(report, verdict)`
   with each reason a line (`rule coding.push-early: missing`, `description: differs: ...`,
   `restart: proposed: "..."`, `check: failed`).
3. **The commands:** `judge::main` (`EVIDENCE`: the brief on stdout, exit 0; 2 on an error);
   `judge::verdict_main` (`EVIDENCE [--json]`: `check::check` over the evidence, `judge.json`
   decoded, the stage's reasons and `passed` or `failed`; exit 0, 1, or 2 when `judge.json` is
   absent or refused, naming it).
4. **Tests** in `src/tests/judge.rs`:
   - `the_brief_for_the_profile_matches_its_golden` (`src/tests/fixtures/judge/brief.md`; the
     added files short and hand-written under `src/tests/fixtures/judge/`);
   - `a_brief_names_what_it_lacks` (no description; a named document absent);
   - `a_section_is_cut_at_its_heading` (a document with two `## ` sections, one named);
   - `a_verdict_decodes`, and refused: an unknown member, a rule the golden lacks, a rule missing,
     a rule twice, a verdict word unknown;
   - `the_stage_passes_only_whole`: passed; then each of a rule `missing`, the description
     `differs`, the restart `proposed`, and a failed report, failing with its reason;
   - `verdict_main_without_a_judge_file_exits_2`.
5. `cargo fmt`; old forms; clippy; `heavy cargo nextest run -p temper-stage-one`; commit
   ("temper-stage-one: the judge's brief and the stage's verdict") and push.

## Report

As the implementer's text says; and: the brief's size for the profile's evidence in bytes, and
the test count.
