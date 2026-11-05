+++
id = "chats-base"
part = "chats-base"
kind = "migration"
needs = ["chat-state"]
files = []
effort = "medium"
minutes = 60
+++

# What ui-chats has by now

## Goal

Every task that draws builds on `ui-chats`' byline, conversation and tool lines. This task merges `origin/ui-chats/integration` into this plan's trunk, so that they do, as far as they've merged by now.

Acceptance:

- Your branch holds `origin/ui-chats/integration`'s tip at the time of your run, by one merge commit, `git merge --no-ff`, whose second parent it is. If `origin/main` already holds `ui-chats`' landing, say so and merge `origin/main` instead.
- Conflicts are resolved keeping both sides' meaning: this plan's jig, skein and temper engine work stays, and `ui-chats`' client stays as it is there. Where a generated file conflicts, regenerate it (`notes/common.md`).
- `cargo fmt`; `heavy cargo nextest run -p temper-client-domain -p temper-client-view -p temper-client-shell-terminal -p temper-client-gallery`; `heavy cargo check --tests -p temper-client-world`.
- **Your report's first section says which of `chat-byline`, `conversation-turns`, `tool-lines`, `chat-stories`, `playground-recording` and `proof` your base now holds.** (`git log origin/ui-chats/integration --oneline | rg ...`, or `fab status` isn't yours: read their merge commits.) A later task reads that.

## Inputs

- `temper/docs/plan/ui-chat-draft/notes/common.md`.
- `temper/docs/plan/ui-chat-draft/plan.md`, *How it sits on ui-chats*.
- `temper/docs/plan/ui-chats/plan.md`.

## Owns and may change

Owns nothing, and may change whatever the merge names, to settle it.

Options, each pre-approved:

- **`ui-chats`' tasks not merged yet:** that's no failure, and you don't wait: merge what's there, and name what isn't. Each later task of this plan has its own option for what it finds.
- **A test of `ui-chats`' already red on `origin/ui-chats/integration`:** `## Left`, with its log, not yours.
- **Budgets:** the merge adds no tests of its own.

## Steps

1. `git fetch origin`, the merge.
2. The conflicts; the checks above; push.

## Report

As `notes/common.md`, Your report, with the list above.

## Rebase (2026-11-05, step merge, foreman ui-chat-draft/chats-base/foreman-1)

Reconcile the ui-chats merge with the integration trunk’s bounds and client changes.
The merge failed on nine conflicting paths; no test failure is reported.
Rebase onto the landing trunk while preserving the task’s required merge ancestry; do not flatten the ui-chats merge.
The overlapping trunk change is `6ade26aff4` (“Merge main while preserving chat documents and whole-document bounds”); configuration status also overlaps main’s `09e237da15`.
- `jig/crates/jig-client-protocol-link/src/limits.rs`: retain grouped `narrow` bounds, including `client.long_text` for `specification_words` and `edit_note_body`; remove duplicate trailing assignments.
- `temper/crates/temper-client-view/src/look.rs`: retain both sets of styles, including chat thread, typing, delegation and tool styles. Assign unique tokens consistent with `NAMES` and `src/tests/look.rs`.
- `temper/crates/temper-client-view/src/words.rs`: retain both vocabularies, reconcile duplicate phrase IDs, and derive `ALL`, phrase, segment and text-byte bounds from the combined declarations.
- `temper/docs/design/client/README.md`: preserve both plans’ meaning; the remaining terminal-inbox overlap is formatting.
- `temper/docs/design/client/browser.md`: retain the ui-chats sample description without restoring a composer to Chats.
- `temper/docs/design/protocol/configuration.status.md`: retain the trunk’s outstanding coordinator-note scope gap from `09e237da15`.
- `temper/tools/gallery/src/lib.rs`: preserve both the chat byline disclosure opening and Needs “Saw” opening.
- `temper/tools/gallery/src/samples.rs`: preserve `Sample::open_disclosure`, both sample sets and their fixtures; derive the combined sample count.
- `temper/docs/design/client/journeys/welcome-playground/recording`: retain a complete ui-chats NewChat recording, not mixed recording streams. This is source; regenerate its HTML prints with `heavy cargo run -p temper-client-journeys`.
Regenerate any stale gallery expectations, names and terminal goldens using the repository tools; inspect their meaning.
Run `cargo fmt`, the spec’s nextest and world checks through heavy, and the citation check for resolved design text.
The report’s first section must identify all six requested ui-chats task states from merge ancestry, including whether `proof` is present.
