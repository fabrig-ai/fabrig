# subagent-burst: rules every task follows

- **Read first:** fabrig's `AGENTS.md`, your tree's `AGENTS.md`
  (`smith/AGENTS.md` or `skein/AGENTS.md`), your spec, and
  `smith/docs/plan/subagent-burst/notes/contracts.md`, which binds the
  shapes that cross tasks. Where a spec and the contracts note differ, the
  note wins; say so in the report.
- **The request:** `smith/docs/plan/subagent-burst/inputs/request.md`,
  read whole (it is short). Its evidence is a deployment outside this
  repository: its words are all there is of it.
- **Only focused and fuzzy run.** Never `tools/tiers/run`, a
  `<tier>-<tree>` profile, a live test or `scripts/browser.sh`. No walk,
  render check or throwaway driver: the plan is complete when it lands.
- **Tests stay cheap.** A new case of a behaviour is a row of an existing
  table test where one fits; a new test only where none does, beside its
  siblings' fixture. smith's and skein's focused suites have 30 CPU
  seconds each in the gate and their fuzzy suites 120
  (`tools/tiers/budgets.toml`); a row that costs more than a second of
  CPU is named in the report with its measurement. Failing tests first:
  a row that pins a fix is seen failing before the fix.
- **Fakes, never the real thing:** no test reaches a real host, a real
  LLM or a real Claude Code. The program is `smith-fake-claude`
  (`smith/tests/fake-claude`), in the simulated agent-process world
  (`smith/tests/agent-process`, package `smith-agent-process-world`) or
  as its binary in the shipped product's tests (`smith/tests/end-to-end`,
  package `smith`, binary `end_to_end`); a model is skein's fake LLM.
- **A test may read its own descendants' `/proc/<pid>/stat`, `comm` and
  `cmdline`, never any process's `environ`** (`AGENTS.md`, 3).
- **Design files are the cut's,** with one departure each spec states:
  the task deletes the **Not yet** entries it closes from the status
  files it names, the phrase it closes in its tree's
  `docs/design/STATUS.md`, and touches no other design text. What the
  code taught that the design does not say goes in the report.
- **Before calling a focused or fuzzy failure "outside the change",** run
  it once on the pristine base. If the base passes and your branch fails,
  it is yours.
- **No figure changes:** no budget, timeout, room (`conversations`),
  bound or profile value of production changes. A test world may set its
  own declared values, as its rows need, and says so.
- **Not to touch:** `smith/docs/plan/**` but your report, drafts,
  `docs/help/`, and every other task's owned files below but the lines
  your spec names.

| Task | Owns |
|---|---|
| `burst` | the program world's rows and harness for a burst of `sub_agent` calls (`program_children.rs`, `program_world.rs`, the fake's `print.rs` and its `Step` enum), and the fix in whichever crate its test finds the cause |
| `report-end` | `skein-fail-stop`'s report, its test world (`skein/tests/fail-stop`), and skein's `shell.status.md` 6 entry |
| `detail-row` | one row of `smith-host-protocol`'s kept-detail test, and `smith/docs/design/domain/host.status.md`'s panic entry |
| `program-child` | the agent binary's `--program-child` mode, the program's launch through it (`smith-protocol-program`'s launch, `ProgramBackend.parent`), the shipped product's abort test, and the programs and shell status entries |
