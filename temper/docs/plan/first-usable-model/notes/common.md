# first-usable-model: the rules every task names

Read this note, then your spec. Where this note and the implementer's text differ, this note wins.

## Departures from the implementer's text

- **Design text:** the cut wrote the design. A task removes from a status file the **Not yet** entry
  it closes, when its `Owns and may change` names that entry, and edits no other `docs/design/` line;
  what the code taught goes in the report.
- **Nothing reaches a real host.** No test here runs a real `claude`, a real provider or a real
  sign-in (the request's *Tests*).

## Beside this plan

Other drafts are building beside this one, and `fabrig/ui` has priority on `temper-client-*`. In
`temper/crates/temper-client-view/src/words.rs`, `accounts/words.rs`, `look.rs`, `limits.rs`,
`pages/chat.rs`, `pages/chats.rs`, `pages.rs` and `temper-client-domain/src/pages.rs`:

- keep each hunk as small as the change allows: add lines, reorder and reformat nothing;
- `cargo fmt` only what you changed, and take back any hunk it makes elsewhere;
- name in your report each of these files you changed.

## Failing tests first

The request asks for this. Commit each story's test with its fix, and say in your report that you
saw it fail on the old code, and how: a test's first run before the fix counts as the *new test must
bite* check.
