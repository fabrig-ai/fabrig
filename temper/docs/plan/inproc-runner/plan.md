# inproc-runner: the plan

The focused and fuzzy suites gain a second mode of the CPU collector: each test
binary's selected tests in one process, binaries in parallel on as many lanes as the
machine has CPUs, CPU measured per process from `wait4` and per test not at all
(`inputs/request.md`). Nextest's path, `cpu-wrapper.c` and every slow tier stay as they
are; the gate switches to the new mode afterwards, in a change of its own.

Contracts: `notes/contracts.md`; standing rules: `notes/common.md`.

## Parts and tasks

1. `starved` (skein): under `TIERS_ONE_PROCESS` a starved pass writes its line and
   unwinds, ending only its test (`skein/docs/design/testing.md`, 5). Needs nothing.

The runner, its summary and command, the repairs item 10 of the request finds, and the
plan's close follow in the next batch.

## Critical path

`starved` runs beside the runner and the summary; the repairs need all three.
