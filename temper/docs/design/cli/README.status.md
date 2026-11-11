# README: status

2026-11-11. Where temper's code lags or departs from [README.md](README.md).

`temper` takes its table in `../draft/cli.execution.md`'s C3; until then
the client's table is `temper-shell`'s `commands` rows, and its help is
today's (`../protocol/binaries.status.md`).

- **Not yet:**
  - 11.1 and 11.2: no `temper-test-env`, no test table and no generated
    `test-env` [in flight: change 446]
  - 11.2: temper's tests that check a document or a golden read
    `SKEIN_CODEGEN_WRITE` themselves [in flight: change 447]
  - 11.2 and 11.3: `temper-box`'s tests read the process themselves, and
    they, temper's proof scripts, CI's Forgejo lines and the gate's
    README name the live, managed, Forgejo, wrapper and parent variables
    by their names before 11.3 [in flight: change 448]
  - 11.2 and 11.3: the rate suite, the confirm tests, the browser world
    and the real loop read the process themselves, `TEMPER_RATE` and
    `TEMPER_RECORD_RELEASE` keep their names, and `tools/tiers/` and CI
    still set `TEMPER_TEST_CLAUDE` [in flight: change 449]
