# Every live-smith-reds-3 task

- Read `AGENTS.md` and `smith/AGENTS.md` first; follow their checks and style. The request is
  `smith/docs/plan/live-smith-reds-3/inputs/request.md`; the live stories' design is
  `smith/docs/design/testing.md`, 2.3. The failing logs (main df41529d3, hetzner8) are quoted in
  your spec: you never need to fetch them.
- **The live stories never run in your slot.** `smith`'s binary `live` needs the logins lent to
  fab's after-landing pair, which your slot does not have: never run it, never set `SMITH_LIVE`,
  `SMITH_LIVE_CLAUDE` or `ANTHROPIC_API_KEY`, never run `claude`, `codex` or `smith login`
  yourself, and send nothing to a real service. A live test you edit is checked by
  `heavy cargo check -p smith --tests`, clippy, and the offline test your spec names; the
  after-landing pair runs it on main.
- **No credential bytes in a message.** A record file may hold credential bytes: never print
  them, never read a `tokens/` record. A failing story's message holds typed facts only
  (testing.md, 2.3, "A failure says why"): generations, whole seconds, fixed words.
- **Not yours:** any `docs/design/` (what the code taught goes in the report), the API-key story
  `an_anthropic_key_delivers_a_commit_in_place`, fab's after-landing code, the shelf's logins,
  the root `.config/nextest.toml`, `smith/crates/**` (the product meets the design here: no
  product change), and `smith/crates/smith-local-shell/src/credentials/login/**` and
  `product.rs` in particular (plan `sign-in-wait`'s). Change only your spec's `files` and the
  neighbour edits it lists; anything else is a `Question:`.
- Focused tests run in the `default` profile (`heavy cargo nextest run -p PKG --test BIN -E
  'test(=NAME)'`); check the count selected is not zero.
- Your report's first line is `Done: ...`, `Question: ...`, `Infra: ...` or `Failed: ...`.
