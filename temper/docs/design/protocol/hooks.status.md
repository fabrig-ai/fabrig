# hooks: status

2026-11-27. Where temper's code lags or departs from [hooks.md](hooks.md).

- **Not yet:** 2.2: the worker checks a hook's link by reading its target with skein's `Op::ReadLink`, not by the sibling open [in flight: change 21655].
- **Not yet:** 2.3: no reader of `.temper/hooks/config.toml` [in flight: change 21654].
- **Not yet:** 2.3, 4.3: a `merge` or `close` hook runs for its job's deadline, its file unread, and a hook past its time fails as the job's deadline [in flight: change 21665].
- **Not yet:** 2.3: no `hook_timeout_max_seconds`; a merge or close job's deadline is ten minutes [in flight: change 21666].
- **Not yet:** 2.3, 4.1: a `deliver` or `finish` hook runs within the check's bound, its file unread [in flight: change 21667].
- **Not yet:** 4.2: temper reads `.temper/hooks/` at the job's commit before making a merge job or a close step, and sends no `find` or `fallback` [in flight: change 21664].
- **Not yet:** 4.2, 4.3: the worker answers no none: a hook missing from the checkout fails the job [in flight: change 21665].
