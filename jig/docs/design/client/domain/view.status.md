# view: status

2026-11-06. Where jig's code lags or departs from [view.md](view.md).

- **Not yet:** 4.1: `pages::owed` as a primitive of its own (`pages.md`, 12.1, and its status). 7: the helper `words` holds no executor or outcome phrases. 9: cards for the objects a turn's calls made. 17.1, 17.5: no view root routes `draw`, jig has no `draw` of its own, and no golden scenes test one.
- **Departs:** 4: `jig-client-view` keeps 27 `_limits`, `_env` and `_out` parameters no function reads, and two of its fragment tests widen a length with `u64::try_from`, against `skein/docs/design/lib.md`, 6, because the `debt-28` request of 2026-11-06 names both items and keeps out of this crate (the code's side is `jig/docs/development/debt/D041-unused-parameters.md` and `jig/docs/development/debt/D043-length-widening.md`).
