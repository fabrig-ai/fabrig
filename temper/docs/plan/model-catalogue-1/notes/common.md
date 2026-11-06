# model-catalogue-1: the rules every task names

Read this note, then your spec. Where this note and the implementer's text differ, this note wins.

## Departures from the implementer's text

- **Design text:** the cut wrote the design. A task removes from a status file the **Not yet** entry
  it closes, when its `Owns and may change` names that entry, and edits no other `docs/design/` line;
  what the code taught goes in the report.
- **Nothing reaches a real host.** No test here runs a real `claude`, a real provider or a real
  sign-in.

## Beside this plan

Other drafts are building beside this one, and `fabrig/ui` has priority on `temper-client-*`: it lands
the look `model-picker` itself and this plan takes its pages by catch-up, so a task here draws the
picker's facts in today's look and adds no CSS. In `temper/crates/temper-client-view/src/words.rs`,
`accounts/words.rs`, `look.rs`, `limits.rs`, `pages/chat.rs`, `pages/chats.rs`, `pages.rs` and
`temper-client-domain/src/pages.rs`:

- keep each hunk as small as the change allows: add lines, reorder and reformat nothing;
- `cargo fmt` only what you changed, and take back any hunk it makes elsewhere;
- name in your report each of these files you changed.

## The stock's names

`claude-opus-5-5`, `claude-sonnet-5-5`, `claude-fable-5-1`, `gpt-6.1-sol`, `gpt-6-astra`,
`gpt-6.1-terra` (OpenAI lists no `gpt-6-terra`), `gpt-6-luna`. Their table is
`temper/docs/design/draft/model-catalogue.md`, section 2.
