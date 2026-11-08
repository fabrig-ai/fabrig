+++
id = "mechanics"
part = "smith-start"
kind = "feature"
needs = []
files = ["smith/crates/smith-domain-run/src/prompt.rs", "smith/crates/smith-domain-run/src/prompt/**", "smith/crates/smith-domain-run/src/tests/opening.rs", "smith/crates/smith-domain-run/src/tests/workspace.rs", "smith/crates/smith-domain-run/src/tests/finite_system.txt", "smith/crates/smith-domain-run/src/tests/open_system.txt", "smith/docs/design/domain/run.status.md"]
effort = "medium"
minutes = 60
+++

# ui-ask-m1: a run with no workspace says nothing of one

## Goal

smith's system text for a run with no workspace has no `## Tools` section ("You have no
tools that act on the checkout.") and no `## Checkout` section ("There is no
checkout."): its instructions, its brief, then the mechanics it has. A run with a
workspace is unchanged, byte for byte.

Acceptance:

- `render_system` (`smith-domain-run/src/prompt.rs`) writes `render_tools` and
  `render_checkout` only where the run has a workspace. The fixed-prose measure that
  sizes the system text (`fixed_prompt_bytes`, `budget_prompt_bytes`) stays an upper
  bound for both cases; if it counted those sections unconditionally, it may keep doing
  so (a measure past the text is allowed), and the report says which it does.
- The tests that pin the absent-workspace text now pin their absence:
  `prompt.rs`, `absent_workspace_and_empty_field_rules_are_said_plainly` and
  `empty_role_and_brief_emit_no_prefix_but_empty_sections_are_still_rendered`;
  `tests/workspace.rs`, `absent_workspace_suppresses_workspace_families_and_preserves_main_services`;
  `tests/opening.rs`, `open_first_user_message_is_the_whole_literal_brief` (its
  731-byte figure falls by the two sections' bytes); and the fixtures
  `tests/finite_system.txt` and `tests/open_system.txt` where their world has no
  workspace (regenerate them by the test that reads them if it writes them; otherwise
  edit only the two sections out). Each asserts that neither heading appears, and a
  workspace row beside it that both still do.
- `smith/docs/design/domain/run.status.md`: delete the **Not yet** sentence "3.3: a run
  with no workspace still has `## Tools` ... (`smith-domain-run/src/prompt.rs`,
  `render_system`)." and nothing else (a departure from the implementer's text: the
  status must say what is built). The **Departs** entry on the word "checkout" stays.

## Inputs

- `temper/docs/plan/ui-ask-m1/notes/common.md`.
- `smith/docs/design/domain/run.md`, 3.3 ("No workspace, no word of one").
- `temper/docs/design/draft/ask-temper.md`, 4.2, for why (smith's mechanics for an open
  run with no workspace about 0.6 KB).

## Owns and may change

- Owns the files in the header.
- A change of meaning: every no-workspace run's system text changes (temper's status
  agent, charter 9, and trials, charter 10, among them). Run the readers' tests:
  `heavy cargo nextest run -p smith-domain-run`, `-p smith-agent-world` (or the package
  of `smith/tests/agent`, with `no_workspace.rs`), and `rg` the repository for
  "There is no checkout" and "no tools that act on the checkout" (the survey found them
  only in the files above). A temper test that pins smith's whole system text for a
  run with no workspace is a reader: its expected text loses the two sections, edited
  only for that, and named in the report.

## Steps

1. Flip the pinned rows to absence, with the workspace row beside; run; see them fail.
2. Gate the two sections in `render_system` on the workspace.
3. `heavy cargo nextest run -p smith-domain-run`, then the readers above; `cargo fmt`;
   `python3 -I -B smith/scripts/old_forms.py`; `heavy cargo clippy -p smith-domain-run
   --all-targets -- -D warnings`.
4. Commit and push.

## Report

As the implementer's text says; give the system text's bytes for the opening fixture
before and after.
