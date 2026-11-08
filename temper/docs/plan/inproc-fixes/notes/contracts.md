# inproc-fixes: the contracts

The run's binding shapes where its two tasks meet. Everything this note does not
name stays as `temper/docs/plan/inproc-runner/notes/contracts.md`, sections 2 and
3, give it, and as `tools/tiers/design.md`, 10.2 and 11.8, give it.

## 1. A test without a process in `results.json` (task `runner` writes, task `summary` reads)

This revises `temper/docs/plan/inproc-runner/notes/contracts.md`, 2, in one point
(`tools/tiers/design.md`, 11.7, *A forced kill*, and 11.8):

- `processes` holds each process with a record, densely, in the order of their
  indexes; a test's `process` is the index of its `"tests"` or `"fresh"` process in
  that list, **or `null`** where a forced kill left that process without its record.
- A `null` `process` occurs only when `cancelled` is non-null, and only with an
  `outcome` of `"passed"`, `"failed"` or `"starved"`: what libtest reported before the
  kill. Its other fields are as for any test (`before` for a failed one, `alone`
  null, `output` where it did not pass).
- After a forced kill, `run.json` has `finished` true, `starved` true and `returncode`
  the lanes' exit status as `Popen` gives it, `-9`; `results.json`'s `cancelled` is
  `"wall-hard"`.

Example, the only process killed after its first test failed:

```json
{"profile": "default", "selected": 3, "no_tests": "fail", "cancelled": "wall-hard",
 "processes": [],
 "tests": [{"binary_id": "pkg::suite", "test_name": "a", "outcome": "failed",
            "wall_s": 0.01, "process": null, "before": [], "alone": null,
            "output": "thread 'a' panicked at ..."}],
 "unobserved": [{"binary_id": "pkg::suite", "test_name": "b"},
                {"binary_id": "pkg::suite", "test_name": "c"}]}
```

## 2. What `summary` gives for it (task `summary`)

`summary`'s keys and meanings stay as `tools/tiers/design.md`, 10.2 gives them. A
test with a `null` `process` is read as any other: it is in `tests`, a failed one in
`failures` (so `execute_one_process` returns 100), a starved one counted in
`suite.starved`. `processes` lists the processes with a record, and the suite's
`cpu_s`, `user_s` and `system_s` are their sums, a lower bound (11.7).
