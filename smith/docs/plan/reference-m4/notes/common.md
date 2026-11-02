# Every reference-m4 task

- Read `AGENTS.md` and `smith/AGENTS.md` first; follow their checks and style. The request is
  `smith/docs/plan/reference-m4/inputs/request.md`. The design is
  `smith/docs/design/protocol/programs.md`, section 2 and 5.10 (sub-agents on a program backend),
  with `smith/docs/design/domain/run.md`, 5.3 (what a sub-agent is on either backend). The shapes
  that cross tasks are `smith/docs/plan/reference-m4/notes/contracts.md`: build to them exactly.
- **smith alone.** No task changes jig, temper or skein code. A skein gap you meet is a `Question:`.
- **Never run the real program.** Never run `claude`, `codex` or `smith login`, never set
  `SMITH_LIVE`, `SMITH_LIVE_CLAUDE` or `ANTHROPIC_API_KEY`, never run a `live` test binary, and send
  nothing to a real service. The worlds run smith's fake `claude` (`smith/tests/fake-claude`). A live
  test you write or edit is checked by `heavy cargo check -p <its package> --tests` and clippy only;
  fab's after-landing pair runs it on main.
- **Lowest world first.** A behaviour is tested in the lowest place that shows it: the crate's unit
  tests, then `smith/tests/agent-process` (the spawned agent on the fake program), then
  `smith/tests/end-to-end`'s fake-backed binaries.
- **Not yours:** any `docs/design/` (what the code taught goes in the report), drafts, the root
  `.config/nextest.toml`, fab's code, the shelf's logins. Change only your spec's `files` and the
  neighbour edits it lists, plus what the implementer's text always allows (compiler-forced arms,
  fields and literals; a scoped `#[expect]`); anything else is a `Question:`.
- **The strict subset** holds in production crates (`smith/AGENTS.md`): exhaustive own-enum matches,
  skein's bounded containers, checked arithmetic, no `if let`/`let else` on own enums.
- Focused tests run in the `default` profile (`heavy cargo nextest run -p PKG ...`), one command at a
  time; check the count selected is not zero.
- Your report's first line is `Done: ...`, `Question: ...`, `Infra: ...` or `Failed: ...`.
