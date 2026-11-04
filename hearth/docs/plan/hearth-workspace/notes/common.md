# hearth-workspace: standing rules

Every task of this plan reads this note and `notes/contracts.md`. The request is
`hearth/docs/plan/hearth-workspace/inputs/request.md`; these are its rules, as they bind a task.

- **A port, not new work.** No behaviour of hearth changes, and no design does. Tests move and
  are ported; they don't change what they check.
- **A red test after the port is a finding about the port first.** Don't weaken a bound or a
  test to get past it. A bound that can only be set by measuring is set by measuring, with the
  measurement in the commit message (`notes/contracts.md`, 5).
- **If a test shows a skein bug, stop and ask.** A change to skein that hearth seems to need is a
  question, not a task: `skein/`, `smith/`, `jig/` and `temper/` change only where a spec says.
- **Take the new forms where a rename gets you there, and go no further.** hearth's remaining
  `worst_case` and `MAX_OUT` uses are allowed old forms that may only fall
  (`skein/docs/foundation/programming-model.md`, 6.3). Adopting the newer model in full is out of
  scope. Lower the counts with `python3 -I -B scripts/old_forms.py --write`, from `hearth/`, for
  what you removed.
- **`hearth/docs/design/` stays as it is,** but for the citations a move breaks and the status
  lines a spec names. `docs/design/` is otherwise not yours, as the implementer's text says.
- **Budgets and timeouts are the user's.** `tools/tiers/budgets.toml` doesn't change. The only
  timeouts this plan touches are the two the request sets (`notes/contracts.md`, 2).
- **`python3 -I -B tools/citations/citations.py --all` stays at 0,** and
  `tests/boundary/tests/outward.rs` passes. It reads every tracked file outside `*/docs/plan/`, so
  no file you change names the lab: `AGENTS.md`, 1.
- **hearth's packages in one flag:** every one of them starts with `hearth`, so `-p 'hearth*'`
  (quoted) selects those that are members. If a command refuses the glob, name each with `-p`.
