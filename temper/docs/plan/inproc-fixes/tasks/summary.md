+++
id = "summary"
part = "summary"
kind = "feature"
needs = []
files = ["tools/tiers/cpu.py", "tools/tiers/test_cpu.py"]
effort = "medium"
minutes = 30
+++

# inproc-fixes: `summary` reads a forced kill's results, a test without a process

## Goal

After a forced kill, `results.json` may hold a test whose `process` is `null`
(`tools/tiers/design.md`, 11.7, *A forced kill*;
`temper/docs/plan/inproc-fixes/notes/contracts.md`, 1). `cpu.py`'s
`one_process_summary` reads it as any other test, and refuses a `null` `process`
anywhere else. 10.2's keys and meanings stay as they are.

Acceptance:

- A test with `process` `null` is accepted only when `results.json`'s `cancelled` is
  non-null and its `outcome` is `passed`, `failed` or `starved`; otherwise `summary`
  raises `ValueError` (and `cpu-summary` exits 2), as for any invalid index now.
- Such a test is in `tests` as any other (its `before` and `alone` read as now); a
  failed one is in `failures`, so `execute_one_process` returns 100; a starved one is
  counted in `suite.starved`. `processes` and the suite's `cpu_s`, `user_s` and
  `system_s` are the recorded processes', as now.
- Rows in `tools/tiers/test_cpu.py`'s `OneProcessTests`, each seen failing before the
  fix where it can (the accepting row; the refusing rows pass today and are kept as
  the rule's guard), and `heavy python3 -I -B tools/tiers/test_cpu.py` green.

## Inputs

- `tools/tiers/design.md`, 11.7 (*A forced kill*), 11.8 (`results.json`), 10.2
  (`summary`'s one-process result, `execute_one_process`'s statuses).
- `temper/docs/plan/inproc-fixes/notes/contracts.md`, 1 and 2;
  `temper/docs/plan/inproc-fixes/notes/common.md`.

## Owns and may change

- Owns `tools/tiers/cpu.py`'s `one_process_summary` and `tools/tiers/test_cpu.py`. No
  other function of `cpu.py` changes: nextest's path stays as it is.
- `one_process.py`, `test_one_process.py` and `design.status.md` are task `runner`'s,
  beside this one: not edited here. It writes such results; this task's rows write
  them by hand, as `OneProcessTests` does.

## Steps

1. Row `test_forced_kill_keeps_reported_outcomes_without_a_process`, from `setUp`'s
   results: `cancelled` `wall-hard`, `run.json` `starved` true; `processes` the one
   recorded process (`self.process(['z'])`), tests `z` passed in process 0 and
   `self.entry('a', outcome='failed', process=None)`, `unobserved` `b` (and `selected`
   3). `summary`: `failures` `['pkg::suite a']`, `unobserved` `b`, `suite.cpu_s` the
   recorded process's, `processes` one entry; through the `execute_one_process`
   stand-in (as `test_execute_status_precedence_and_appended_summary` does) the status
   is 100. Today it raises on the index; see it fail.
2. In `one_process_summary`, accept `process` `None` under the rule above, skipping
   the index checks for it; nothing else reads a test's process.
3. Refusing rows in `test_invalid_results_raise_and_cli_exits_two`: `null` process
   with `cancelled` null; with `cancelled` set and outcome `crashed`; and `cancelled`.
   Break the rule once by hand (accept every `null`), see them fail, restore.
4. `heavy python3 -I -B tools/tiers/test_cpu.py`; `python3 -I -B
   tools/citations/citations.py --base origin/main`. Commit and push.

## Report

As the implementer's text gives it, with each row and what it showed before the fix
and with the rule broken by hand.
