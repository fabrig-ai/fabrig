# skein-limit-40: the plan

The request (`inputs/request.md`): skein's focused suite budget in
`tools/tiers/budgets.toml` goes from 30 to 40 CPU seconds, the user's
figure of 11-08; the header's skein line says so. Nothing else changes.

## Parts and tasks

- `skein-limit` (direct): `tasks/raise-figure.md`, needs nothing. One
  value (`[gate.skein.default]` `cpu_suite_s = 40`) and one header
  comment line; checked by `tools/tiers/test_run.py` and
  `tools/tiers/test_cpu.py`, which need no build.

One slice: the gate accounts skein's focused suite against 40.

## Critical path

`raise-figure` alone, about 10 implementer-minutes.

## Overlaps

None: no draft beside this one names `tools/tiers/budgets.toml`.

## Left outside, by the request

The request changes no other file. These design lines still name skein's
focused budget as 30 CPU s, and lag the figure once it lands:

- `skein/docs/design/testing.md`, 7 ("CPU budgets of 30 s and 120 s in
  `tools/tiers/budgets.toml`");
- `skein/docs/design/testing.status.md`, the **Not yet** entries under 7
  (the 30 CPU s budget and its 2.5 CPU s margin);
- `skein/docs/design/STATUS.md` (the focused repairs' "within 30 CPU s").

`skein/docs/foundation/testing-strategy.md`, 8.1 keeps 30 s as the
default figure until a tier's budget is set; it stays true.
