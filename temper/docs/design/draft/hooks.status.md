# hooks: status

2026-11-27.

## Design lags

- 1, 3: `../domain/agent/charter.md`, section 4, and smith's `domain/run-start.md`, 1.1, and `domain/run-delivery.md`, 1.1: the delivery check is `.temper/pre-pr`, one path in the conventions, run at a `deliver` and at a `finish` that delivers alike.
- 1, 3, 4: `../protocol/tooling.md`, sections 1, 2, 4.1 and 4.2, `../domain/landing-settings.md`, 1.1, `../protocol/configuration/engine.md` and `../domain/forge/branches.md`, section 2: the merge and close jobs' commands are paths the project names, and a setting's presence switches its job on.
- 3.4: `../protocol/tooling.md`, section 2, and `../domain/agent/charter.md`, section 4: a job's time is its kind's deadline, a constant (ten minutes for `close`), and a check's bound the project's `check_timeout_ms`; `../domain/landing-settings.md`, 1.1, has no ceiling for a hook's time.
- 6: `../protocol/configuration/reference.md`: the reference's `.temper/pre-pr` and `.temper/merge`, and `merge_job` and `close_job` in its landing profile. `tools/pre-pr/design.md`: `.temper/pre-pr`, `goal-pre-pr`, `goal-child-pre-pr` and `checked-close` as fabrig's commands.

## Design departs

None.

## Open

- 3.4: the ceiling's default, 3,600 seconds, is a figure of the drafter's, for the user to confirm or change.
- `ways.md`, 8.4, 10 and 11, still says the first pass's hooks: three of them, `close` run from the landing branch's tip where its refusal holds a landing, a fallback moment by moment, and hooks after `.temper/`'s guarded read. It stays as it is on the user's word (2026-11-27) until its next pass, which takes this draft's.
