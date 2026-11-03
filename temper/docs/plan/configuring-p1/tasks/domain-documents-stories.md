+++
id = "domain-documents-stories"
part = "documents-temper"
kind = "feature"
needs = ["root-document-binding", "jig-documents-core", "jig-documents-store"]
files = ["temper/tests/engine/domain/tests/suite/forge_routes/documents.rs", "temper/tests/engine/domain/tests/suite/forge_routes.rs", "temper/tests/engine/domain/src/forge_routes.rs", "temper/docs/design/domain/agent.status.md", "temper/docs/design/domain/testing.status.md"]
effort = "high"
minutes = 90
+++

# temper: the domain world's four stories of a charter's documents

## Goal

Prove, through jig's core plan, temper's root binding and the forge's gather together, that a
charter's documents reach its runs (`temper/docs/design/domain/testing.md`, 6.4, the row "the
engine's domain world, a charter's documents"; temper's
`temper/docs/design/protocol/configuration.md`, 13.7; jig's
`jig/docs/design/domain/engine.md`, 17.9). This closes P1's documents.

Acceptance, a new `tests/suite/forge_routes/documents.rs` (declared in
`tests/suite/forge_routes.rs`), four stories in the world's usual cost (about 0.1 s together;
their time in the report):

- `a_charters_document_reaches_its_runs_brief_at_the_branch_head`: the charter the world's runs
  use names `AGENTS.md` on `main` of an adopted repository; a run's assignment holds a section of
  kind `Forge(Document(..))` whose smith title is `Document OWNER/NAME AGENTS.md` and whose text
  starts `At main's head ` and the commit, then holds the file whole.
- `a_landed_change_to_the_document_reaches_the_next_run`: a change that edits `AGENTS.md` lands on
  `main` (or the fake forge advances the branch, as `forge_routes.rs`'s CI stories push a file);
  the next run's section has the new text and the new commit, with no change of settings.
- `a_documents_heading_gives_its_part_alone`: a document with heading `Rules` reads from
  `## Rules` to the next `##`, the section titled `Document OWNER/NAME AGENTS.md, "Rules"`.
- `a_missing_document_says_so_and_the_run_goes_on`: three documents' cases, in one story or each
  its own `#[test]` (a missing file; a missing heading; a repository adopted in no project), each a
  section in the contracts' words (`temper/docs/plan/configuring-p1/notes/contracts.md`, section
  3), and the run is prepared and starts.
- The world's limits for these stories hold the brief: `brief.sections` 8 and
  `tasks.charter_documents` at least the documents a story names, set in the stories' own limits
  (as the world's other stories set theirs), never in the shared fixtures.
- `heavy cargo nextest run -p temper-engine-domain-world --test suite -E 'test(/documents::/)'`
  passes, its count in the report (check the binary's name in the package's `Cargo.toml`); then
  the package's focused tests.
- The statuses this closes: delete "4.1: a charter's documents, each a `Document` section of its
  runs' briefs (`../protocol/configuration.md`, 13.7)." from
  `temper/docs/design/domain/agent.status.md`'s **Not yet**, and "6.4: the four stories of a
  charter's documents." from `temper/docs/design/domain/testing.status.md`'s, and nothing else;
  if a status's first sentence after `**Not yet:**` then starts with a lowercase section entry,
  leave it as it reads.

## Inputs

- `temper/docs/plan/configuring-p1/notes/common.md` and `notes/contracts.md`, sections 1 and 3.
- `temper/docs/design/domain/testing.md`, 6.4 and 6.1 (a story's cost);
  `temper/docs/design/protocol/configuration.md`, 13.7; `temper/docs/design/domain/agent.md`, 4.1.
- Stories to follow: `tests/suite/forge_routes.rs`'s
  `a_failed_ci_repair_brief_names_the_failed_step_and_its_logs_end` and its helpers
  (`failed_ci_repair`, `ci_words`: a section found in `assignment.sections` by its kind), and
  `src/forge_routes.rs` (the world's configuration, its charters from `walking::charter()`, a file
  pushed with `skein_fake_forge_domain`'s `scenario::advance`).

## Owns and may change

- Owns the files in the header; `src/forge_routes.rs` gains only the helpers these stories need (a
  charter with documents, a world whose limits hold them), each listed.
- Read only: every crate. A story that fails because the code does not do what the contracts say
  is a `Question:` with the assignment's sections as found; never bend an assertion to fit.
- Pre-approved: if the world's charters cannot take documents in its defaults, give them by a
  `PartyInput::ChangeSettings` of the charter before the run, as jig's settings stories do; if a
  landing is costly in this world, advance the branch through the fake forge instead and say so.

## Steps

1. The helpers; the four stories; run them alone.
2. The status lines.
3. `cargo fmt`; `heavy cargo clippy -p temper-engine-domain-world --all-targets -- -D warnings`;
   `heavy cargo nextest run -p temper-engine-domain-world`; the citations check.
4. Commit (why: a charter's documents reach its runs, end to end); push your branch.

## Report

As the implementer's text says, and also: the stories' time, how each document's file was put on
its branch, and the tests' count.
