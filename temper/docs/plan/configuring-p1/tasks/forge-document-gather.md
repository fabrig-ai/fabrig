+++
id = "forge-document-gather"
part = "documents-temper"
kind = "feature"
needs = []
files = ["temper/crates/temper-engine-domain-forge/src/boundary.rs", "temper/crates/temper-engine-domain-forge/src/domain.rs", "temper/crates/temper-engine-domain-forge/src/domain/**", "temper/crates/temper-engine-domain-forge/src/brief.rs", "temper/crates/temper-engine-domain-forge/src/held.rs", "temper/crates/temper-engine-domain-forge/src/brief_document.rs", "temper/crates/temper-engine-domain-forge/src/lib.rs", "temper/crates/temper-engine-domain-forge/src/tests.rs", "temper/crates/temper-engine-domain-forge/src/tests/brief_document.rs", "temper/docs/design/protocol/configuration.status.md"]
effort = "high"
minutes = 120
+++

# temper: the forge gathers a charter's document as a brief section

## Goal

temper's forge connector gathers a charter's document for a run: the file at the branch's
head, or its part under a heading, cut at a line end to the section's allotment, one commit
for the section's whole life; and a section saying why when the repository is not adopted, or
the branch, file or heading is not there (`temper/docs/design/protocol/configuration.md`,
13.7). This task is the forge's half alone: the root binds a planned document to it in
`root-document-binding`; until then nothing sends `PlanDocument`.

Acceptance:

- `BriefDocument`, `Event::PlanDocument { section, document }` and `BriefSource::Document`, as
  `temper/docs/plan/configuring-p1/notes/contracts.md`, section 3 gives them; `BriefSource`
  stays `Copy`. `PlanDocument` registers the section in `brief_planned` with
  `BriefSource::Document` and keeps its `BriefDocument` by the section's token until
  `GatherPlanned` takes it, or `DropBrief` drops it; the table is bounded by the forge's
  `brief_sections`.
- `GatherPlanned` for a document resolves the repository (adopted in any project under the
  document's host, owner and name, its forge not ended), reads the branch's head
  (`skein_forge::api::Read::Branch`), then the file at that head (`Read::File`, `max_bytes`
  the forge's `client.answer_bytes`, so a cut can count what it leaves out), and holds the
  section's words as contracts.md, section 3 writes them: `At BRANCH's head COMMIT`, the text
  or the heading's part, and `[N bytes cut]` (or `[more than N bytes cut]`) when cut at the
  last line end within the allotment. `CutBrief` re-renders the held words to the smaller
  allotment the same way, never reading again. The four "saying why" words are the section's
  text (`BriefRead::Got`); a failed or refused read is `BriefRead::Failed`. Busy reads wait and
  retry as the other sources' do, within the gather's deadline.
- The heading's part: from the first Markdown heading line whose text, after its `#`s and one
  space, equals the heading byte for byte, to the line before the next heading of its level or
  above, or the file's end; a `#` line inside a fenced code block is no heading.
- The gather's code sits in a new module `src/brief_document.rs` (doc comment citing
  `temper/docs/design/protocol/configuration.md, 13.7`), called from `held.rs`/`brief.rs` where
  the other sources' gathers are; `held.rs`'s cut for a document cuts at a line end, every
  other source's cut stays as it is.
- Step tests in a new `src/tests/brief_document.rs` (declared in `src/tests.rs`), built as
  `src/tests/brief.rs`'s `planned_read` builds a section, with the forge's fake reads:
  - `a_document_section_is_the_file_at_the_branch_head`: the words start
    `At main's head <commit>` and hold the file whole;
  - `a_heading_takes_its_part_to_the_next_heading_of_its_level`: `## B` under `# A` up to the
    next `##` or `#`, not a deeper `###`; a `#` inside a fence ignored;
  - `a_long_document_is_cut_at_a_line_end_and_counts_what_it_left_out`;
  - `a_document_cut_again_keeps_its_commit`: a `CutBrief` after the branch moved re-renders the
    held words, its commit unchanged, and reads nothing;
  - `a_document_of_a_repository_not_adopted_says_so`, `..._of_a_missing_branch_says_so`,
    `..._of_a_missing_file_says_so` (a directory too), `..._of_a_missing_heading_says_so`;
  - `a_failed_document_read_is_missing`.
  `heavy cargo nextest run -p temper-engine-domain-forge --lib -E 'test(/brief_document::/)'`
  passes, its count in the report; then the crate's lib tests and the forge world
  (`heavy cargo nextest run -p temper-engine-forge-world`, the package name as
  `temper/tests/engine/forge/Cargo.toml` gives it).
- The first 13.7 sentence leaves `temper/docs/design/protocol/configuration.status.md`: delete
  "13.7: the forge gathers no document." and nothing else.

## Inputs

- `temper/docs/plan/configuring-p1/notes/common.md` and `notes/contracts.md`, section 3 (the
  forge's half).
- `temper/docs/design/protocol/configuration.md`, 13.7; `temper/docs/design/domain/forge.md`,
  section 2 (`read_file`, the resource names); `temper/docs/design/domain/root.md`, 7.2 (a
  connector's brief section).
- Today's code to follow: `src/held.rs` (`gather`, `cut`, `render_edge`, `take`), `src/brief.rs`
  (the pull section's stages and `brief_answer`; it names its head with `crate::commit_text`),
  `src/domain.rs`'s `PlanBrief`/`GatherPlanned`/`DropBrief` arms, and `src/tests/brief.rs`.

## Owns and may change

- Owns the files in the header (and the forge's own unit-test fixtures under `src/tests/`
  where a story needs an adopted repository with a file at a branch head; list each).
- Readers of `BriefSource` and `Event` in the forge crate: compiler-forced arms only.
  `temper-engine-domain`'s `assemble.rs` matches `BriefSource` (`forge_brief_kind`): add the
  forced arm there as `BriefSource::Document => unreachable!("documents are planned by
  PlanDocument")` only if the compiler forces it, and list it; the root's binding is not yours.
- Pre-approved: if the forge has no lookup of adopted repositories by host, owner and name
  across projects, add one in `src/brief_document.rs` over the forge's repositories; if a
  branch name with `/` needs a form the fake forge does not take, follow `skein_forge`'s
  `Read::Branch` as the fake reads it and say so.
- The plan beside this one (`reference-m1/change-words`) edits `boundary.rs` and `domain.rs`
  too: keep your hunks to the new variant, the new event and the two arms.

## Steps

1. The types and the event; the table keeping a planned document; the `GatherPlanned` arm.
2. `src/brief_document.rs`: resolution, the two reads, the heading's part, the words, the cut.
3. The step tests; run them alone, then the crate's lib tests and the forge world.
4. The status sentence.
5. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`;
   `heavy cargo clippy -p temper-engine-domain-forge --all-targets -- -D warnings`;
   `heavy cargo check -p temper-engine-domain --tests`; the citations check.
6. Commit (why: the forge gathers a charter's document at its branch's head); push your branch.

## Report

As the implementer's text says, and also: how a busy or late read ends, the words of each
"saying why" section as tested, and the tests' count.
