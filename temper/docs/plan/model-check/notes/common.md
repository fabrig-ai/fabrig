# model-check: the rules every task names

Read this note, then your spec. Where this note and the implementer's text differ, this note wins.

## Departures from the implementer's text

- **Design text:** the cut wrote the design. A task removes from a status file the **Not yet** entry it
  closes, when its `Owns and may change` names that entry, and edits no other `docs/design/` line; what the
  code taught goes in the report.
- **Nothing reaches a real host.** No test here runs a real `claude`, a real provider or a real sign-in, and
  no test spends a real turn: a test uses the fake LLM. A real turn is only for the walk (the request's
  *Tests*).

## Failing tests first

The request asks for this, at the lowest tier that can show it. Commit each story's test with its fix, and
say in your report that you saw it fail on the old code, and how: a test's first run before the fix counts
as the *new test must bite* check.

## Beside this plan

`fabrig/ui`'s plans have priority on `temper-client-*`: `look-round-12`, `ui-walk-3`, `chat-reading-c`,
`sign-in-new-tab`. In `temper/crates/temper-client-view/src/words.rs`, `accounts/words.rs`, `look.rs`,
`limits.rs`, `pages/home*.rs`, `pages.rs`, `temper-client-domain/src/pages.rs`, `route.rs`, `domain.rs`,
and `temper-client-shell-web/temper.css`:

- keep each hunk as small as the change allows: add lines, reorder and reformat nothing;
- `cargo fmt` only what you changed, and take back any hunk it makes elsewhere;
- name in your report each of these files you changed.

**Goldens, prints and the gallery's `names.txt` and `tests.rs`** conflict across plans: on a merge,
regenerate rather than hand-merge, and keep `names.txt` sorted as it is.

## Words

Comments and words say "trial" for what runs and "the try" for what the person reads, as
`temper/docs/design/client/README.md`, 3.5, does. A trial is never a chat: no word of a chat's on it.
