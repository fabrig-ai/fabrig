+++
id = "help-words"
part = "help-words"
kind = "feature"
needs = []
files = ["temper/crates/temper-service/src/startup/reference.rs", "temper/crates/temper-service/src/startup/members.rs", "temper/docs/help/change.md", "temper/docs/help/task.md", "temper/docs/help/reference/configuration.md", "temper/crates/temper-docs/src/tests/rules.rs", "temper/docs/design/protocol/documentation.status.md"]
effort = "medium"
minutes = 30
+++

# The help says what a merge command is given and must do, and what a cancelled early start means

## Goal

`temper help configuration` gains the merge command's contract, `temper help change`'s step `resolving` points to it, and both `temper help change` and `temper help task` say what `Cancelled · its early start conflicts with SHA` means and that the change goes on with a new producer. The words are written: you apply them, render the section and hold them with tests.

A member's line holds at most 160 bytes (`the_rows_have_unique_names_and_bounded_lines`), which the contract cannot fit, so it is a section of the page and `merge_job`'s line points to it. `change.md` and `task.md` sit 12 and 2 bytes under their bound of 8,192, so `notes/help-words.md` also tightens them; every word it removes is a fact the page says elsewhere.

Acceptance:

- `temper help configuration` ends with the section `## The merge command`, as `notes/help-words.md` has it, its bounds and `PATH` read from `limits.rs`.
- `merge_job`'s line is the note's, and the phrase `run from the repository root` is gone from the page: the job's working directory is `TEMPER_JOB`, not the repository.
- `change.md` and `task.md` are the note's, each within 8,192 bytes.
- The tests below pass, and `each_page_keeps_its_head_template_links_and_voice` and `the_pages_configuration_are_what_the_tables_render` still do.
- The **Not yet** entry on 8 in `temper/docs/design/protocol/documentation.status.md` is gone.

## Inputs

- `temper/docs/plan/clearer-words/notes/common.md`.
- `temper/docs/plan/clearer-words/notes/help-words.md`: the exact words.
- `temper/docs/design/protocol/documentation.md`, 8: the last bullet, which this cut wrote.
- `temper/docs/design/protocol/tooling.md`, 4.1: what the section says. The words were checked against it and against the worker's code (`temper-worker-domain-tooling/src/domain.rs`, `written_job`).
- `temper/docs/design/domain/forge.md`, 8.11, *Early starts*: what the pages' sentences say.

## Owns and may change

Owns: `temper-service/src/startup/reference.rs`; the `merge_job` row's `line` in `temper-service/src/startup/members.rs`, nothing else in that file; `temper/docs/help/change.md` and `task.md`, a departure from the implementer's text, which closes `docs/help/`; `temper/docs/help/reference/configuration.md`, regenerated; three new tests in `temper-docs/src/tests/rules.rs`; the one entry in `documentation.status.md`.

Options, each pre-approved:

- If an old text in the note no longer occurs once because the trunk moved, apply the same change to what stands, keep the page within its bound and say so in the report. If the page cannot fit without losing a fact, tighten another sentence that repeats a fact the page says elsewhere, and name it in the report.
- If a fact in the note is untrue of the code, correct the page's sentence to the code, keep it brief, and name it in the report: the design is the lead's.
- If `MERGE_DEADLINE` is not whole seconds or a limit is unreachable from `reference.rs`, make the one `pub(crate)` or helper change `limits.rs` needs and say so.

## Steps

1. **The section.** In `reference.rs`, `configuration_reference` calls `render` and then a new private `merge_command(out: &mut Writer) -> Result<(), Overflow>`, which writes the note's section after a blank line. It cites `temper/docs/design/protocol/tooling.md`, 4.1 at the function. The engine page's renderer is untouched. Its three values, each through `skein_lib::Decimal` or as bytes:

   ```rust
   // {BYTES}: crate::limits::MERGE_RESULT_BYTES
   // {SECONDS}: crate::limits::MERGE_DEADLINE.as_secs()
   // {PATH}: crate::limits::tooling(0).path
   ```

   Rendering takes the measured-then-exact path every part of this page takes: `render` is called twice by the page's test, and one byte short must overflow.
2. **The line.** `merge_job`'s `line` in `members.rs` becomes the note's.
3. **The pages.** Apply the note's replacements to `change.md` and `task.md`, in its order.
4. **Regenerate** `configuration.md`, then run the same test again without the variable, to see the compiled page match:

   ```
   SKEIN_CODEGEN_WRITE=1 heavy cargo nextest run -p temper --lib -E 'test(the_pages_configuration_are_what_the_tables_render)'
   heavy cargo nextest run -p temper --lib -E 'test(the_pages_configuration_are_what_the_tables_render)'
   ```

   Its diff is the section and the one line, nothing else.
5. **The tests,** in `temper-docs/src/tests/rules.rs`, beside `task_help_explains_refused_invalid_results`, reading each page with its white space collapsed as that test does:

   ```rust
   #[test]
   fn configuration_help_gives_the_merge_commands_contract() {
       // After "## The merge command": `TEMPER_JOB`, `request.json`, `result.json`, `git add`,
       // **Exit 0:**, **Exit 3:**, **Any other exit,**.
       // Before it: "see The merge command, below." on merge_job's line.
   }
   #[test]
   fn change_help_points_resolving_to_the_merge_command() {
       // Between "- `resolving`:" and "- `repairing`:": temper help configuration.
   }
   #[test]
   fn change_and_task_help_say_what_a_cancelled_early_start_means() {
       // Each of change and task: `Cancelled · its early start conflicts with SHA`
       // and "goes on with a new producer".
   }
   ```
6. **The status.** Delete the **Not yet** entry on 8 from `documentation.status.md`.
7. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `python3 -I -B tools/citations/citations.py --base origin/main`; `heavy cargo clippy -p temper-service -p temper-docs --all-targets -- -D warnings`.
8. Tests, one command at a time: `heavy cargo nextest run -p temper-docs`; `heavy cargo nextest run -p temper --lib`; `heavy cargo nextest run -p temper-service --lib -E 'test(the_rows_have_unique_names_and_bounded_lines)'`. A departure from the implementer's text: only words move, so the other dependents of these crates wait for the trunk's end check.
9. Commit, say why, and push.

## Report

As the implementer's text asks. Name each option you took, and the byte size of `change.md` and `task.md`.
