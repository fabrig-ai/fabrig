# Every live-smith-reds-2 task

- Read `AGENTS.md` and `smith/AGENTS.md` first; follow their checks and style. The request is
  `smith/docs/plan/live-smith-reds-2/inputs/request.md`; the live stories' design is
  `smith/docs/design/testing.md`, 2.3. The failing logs (main 46e6e70d9, hetzner8, Claude Code
  2.1.312, codex-cli 0.162.1) are quoted in your spec: you never need to fetch them.
- **The live stories never run in your slot.** `smith`'s binary `live` and
  `smith-agent-process-world`'s binary `live` need logins lent to fab's after-landing pair, which
  your slot does not have: never run them, never set `SMITH_LIVE`, `SMITH_LIVE_CLAUDE` or
  `ANTHROPIC_API_KEY`, never run `claude`, `codex` or `smith login` yourself, never read Claude
  Code's installed files, and send nothing to a real service. A live test you edit is checked by
  `heavy cargo check -p <its package> --tests`, clippy, and the offline test your spec names; the
  after-landing pair runs it on main.
- **Lowest world first.** A cause is fixed with a test in the lowest world that shows it (a
  crate's unit tests, the fake `claude` in `smith/tests/fake-claude`, `smith/tests/tools`,
  `smith/tests/local-process`, `smith/tests/agent-process`, then `smith/tests/end-to-end`'s
  fake-backed binaries), red before your fix and green after: run it once before the fix and say
  in the report what it failed with. A test that pins a shape already right is green before and
  after; say so.
- **No credential bytes in a message.** A live smith's standard error and a record file may hold
  credential bytes: never print them, never read a `tokens/` record or a Claude configuration
  directory's files. A failing story's message holds typed facts only (testing.md, 2.3, "A
  failure says why"): exit codes, `smith-events` enums and counts, tool names, lengths,
  generations; never a prompt, a completion, a tool's input or result, or a file's bytes.
- **`smith/tests/end-to-end/live.rs` and `support/live_report.rs` have one writer at a time:**
  the tasks that edit them are chained in `plan.toml`; edit only the functions your spec names.
- **Not yours:** any `docs/design/` (what the code taught goes in the report), the API-key story
  `an_anthropic_key_delivers_a_commit_in_place`, fab's after-landing code, the shelf's logins,
  the root `.config/nextest.toml`, `smith/crates/smith-local-shell/src/credentials/login/**` and
  `product.rs` (the `sign-in-wait` plan's). Change only your spec's `files` and the neighbour
  edits it lists; anything else is a `Question:`.
- Focused tests run in the `default` profile (`heavy cargo nextest run -p PKG --test BIN -E
  'test(=NAME)'`); check the count selected is not zero.
- Your report's first line is `Done: ...`, `Question: ...`, `Infra: ...` or `Failed: ...`.
