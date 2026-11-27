# hooks: status

2026-11-27.

## Design lags

- 5: `../protocol/hooks.md`, 5, `../domain/landing-settings.md`, 1.1, `../protocol/configuration/engine.md` and `../domain/agent/charter.md`, section 4: `.temper/pre-pr`, `merge_job` and `close_job` are still read for a repository without `.temper/hooks/`, and `temper ctl` takes both settings. `../protocol/configuration/reference.md` and `tools/pre-pr/design.md`: fabrig keeps `.temper/pre-pr`, `merge`, `close` and `checked-close` as wrappers, and `goal-pre-pr` and `goal-child-pre-pr` in `.temper/`. Goal H2's.

## Design departs

- 3.3: temper's conventions name the hooks directory, `.temper/hooks`, whose `deliver` and `finish` smith takes as the check for each moment, rather than a path for each (smith's `protocol/charter.md`, section 2; `../protocol/hooks.md`, 4.1). A moment's hook is named for smith's own tool, so the directory says both, and the rule by repository needs the directory anyway. Decided by goal 20277's designer, 2026-11-27, keeping the draft's intent where its letter names two paths.

## Open

- `ways.md`, 8.4, 10 and 11, still says the first pass's hooks: three of them, `close` run from the landing branch's tip where its refusal holds a landing, a fallback moment by moment, and hooks after `.temper/`'s guarded read. It stays as it is on the user's word (2026-11-27) until its next pass, which takes this draft's; with it, a way's path, which still shows a goal's catch-up by `merge_job` alone (`../protocol/hooks.md`, 7).
