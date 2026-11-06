+++
id = "view-tab"
part = "tab-root"
kind = "feature"
needs = ["sign-in-holds"]
files = ["temper/crates/temper-client-view/src/pages/sign_in_wait.rs", "temper/crates/temper-client-view/src/tests/sign_in_wait.rs", "temper/crates/temper-client-view/src/pages/connect_model.rs", "temper/crates/temper-client-view/src/pages/welcome_model.rs"]
effort = "medium"
minutes = 25
+++

## Goal

The note reads the page's own fact of a tab refused: "ChatGPT’s sign-in", no "in the new tab", and "Open ChatGPT’s sign-in ›" as the primary button, the person's own click. Today `sign_in_wait::tab_refused` is false, the one function this plan feeds.

Acceptance:

- `sign_in_wait::tab_refused(page: &ConnectModel) -> bool` reads `ConnectModel::tab_refused`, and both callers pass their page.
- A view test gives the root `Event::TabRefused` after a press and draws the refused note, then, with the answer, its primary link at the address. Another view test with no refusal draws "Waiting for ChatGPT’s sign-in, in the new tab" and "Open it again ›".
- The existing `refused_tab_keeps_the_plain_title_and_a_primary_external_link` stays, without its `assert!(!tab_refused())`.

## Inputs

- `temper/docs/plan/sign-in-new-tab/notes/contracts.md`, *The root*, *The view*.
- `temper/docs/plan/sign-in-new-tab/notes/common.md`.
- `temper/docs/design/client/README.md`, 3.5, the welcome's Model step, *The sign-in under way*; `ConnectModel`, *The sign-in's tab*.

## Owns and may change

- Owns the files in the header.
- Keep the hunks small: `pages/connect_model.rs` is being changed by the draft `first-usable-model`, and fabrig/ui has priority there. Change the one call in each caller.
- Remove the sentence "The view has no fact of a tab refused: `sign_in_wait::tab_refused` is false, the one function that draft feeds." from `temper/docs/design/client/README.status.md`'s first **Not yet** entry. This task may make that one edit, against the implementer's text.

## Steps

1. The tests, failing first.
2. The function and its two callers. Break the rule by hand once: return `false`, see the refused test fail for the title's words, restore.
3. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-client-view --all-targets -- -D warnings`; `heavy cargo nextest run -p temper-client-view`; `heavy cargo check -p temper-client-shell-web -p temper-client-gallery --tests`. Commit and push.

## Report

As the implementer's text asks.

## Rebase (2026-11-06, step merge, foreman sign-in-new-tab/view-tab/foreman-1)

resolve the status conflict by preserving both tasks’ sentence removals.
The merge conflicts only in `temper/docs/design/client/README.status.md`, in the first **Not yet** entry.
Commit `87a765c242` removes the obsolete claim that `sign_in_wait::tab_refused` is false.
Integration commit `0cf31fbabb` removes the obsolete sentence about `Act::SignInElsewhere` and `ConnectModel::elsewhere`.
Resolve by retaining both removals. After the existing section citations, the entry’s body should read:
“A page reloaded holds no address, so it draws no "Open it again".”
Preserve the remaining status entries and the task’s Rust changes.
