# Every live-smith-reds task

- Read `AGENTS.md` and `smith/AGENTS.md` first; follow their checks and style. The request is
  `smith/docs/plan/live-smith-reds/inputs/request.md`; the live stories' design is
  `smith/docs/design/testing.md`, 2.3.
- **The live stories never run in your slot.** `smith`'s binary `live` and
  `smith-agent-process-world`'s binary `live` need logins lent to fab's after-landing pair, which
  your slot does not have: never run them, never set `SMITH_LIVE`, `SMITH_LIVE_CLAUDE` or
  `ANTHROPIC_API_KEY`, never run `claude`, `codex` or `smith login` yourself, and send nothing to a
  real service. A live test you edit is checked by `heavy cargo check -p <its package> --tests`,
  clippy, and the offline test your spec names; the after-landing pair runs it on main.
- **Lowest world first.** A cause is fixed with a test in the lowest world that shows it (the fake
  `claude` in `smith/tests/fake-claude`, the crate's unit tests, `smith/tests/agent-process`,
  then `smith/tests/end-to-end`'s fake-backed binaries), red before your fix and green after:
  run it once before the fix and say in the report what it failed with.
- **No credential bytes in a message.** A live smith's standard error and a record file may hold
  credential bytes: never print them, never read a `tokens/` record or a Claude configuration
  directory's files. Report typed facts (exit codes, `smith-events` enums and counts, lengths,
  generations) as your spec says.
- **Not yours:** any `docs/design/` (what the code taught goes in the report), the API-key story
  `an_anthropic_key_delivers_a_commit_in_place`, fab's after-landing code, the shelf's logins,
  the root `.config/nextest.toml`. Change only your spec's `files` and the neighbour edits it
  lists; anything else is a `Question:`.
- Focused tests run in the `default` profile (`heavy cargo nextest run -p PKG ...`); check the
  count selected is not zero.
- Your report's first line is `Done: ...`, `Question: ...`, `Infra: ...` or `Failed: ...`.
