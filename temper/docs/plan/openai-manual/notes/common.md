# openai-manual: standing rules

Every task of this plan names this note.

- The tree is skein; read `AGENTS.md`, then `skein/AGENTS.md`.
- **Never run a provider story.** No task runs `openai_streamed_text_and_history`,
  `openai_tool_round_trip`, `openai_unauthorized` or the three `anthropic_*` stories of
  `skein/tests/llm/tests/live.rs`, with or without `--run-ignored`: they reach the real
  providers. Under the `live-skein` profile a task may list the `live` binary
  (`heavy cargo nextest list ...`) and run its offline login tests by name (the spec's
  filter); nothing else of it.
- Never read, copy or name a login: no `~/.codex`, no `auth.json`, no key file, no shelf
  path, nothing under `~/src/fabrig/fab/`; never set `SKEIN_TEST_LIVE_OPENAI_AUTH` or
  `SKEIN_TEST_LIVE_ANTHROPIC_KEY`.
- **The design is written:** `skein/docs/design/llm.md, 5` is unchanged; the gap is a
  **Departs** entry in `skein/docs/design/llm.status.md`, whose text the spec gives whole. The
  status edits the spec names are its one stated departure from the implementer's text
  ("Not yours: any `docs/design/`"); no other design, draft or `docs/help/` edit.
- No new crates, no dependency changes, no new items: attributes and prose only.
- Old-form counts only fall (`skein/scripts/old-forms.txt`): new text carries no history
  and no "actual" or "genuine".
- Every build and test through `heavy`, one at a time, and check the count each selected or
  listed.
