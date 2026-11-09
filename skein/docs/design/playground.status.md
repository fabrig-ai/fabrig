# playground: status

2026-11-09. Where skein's code lags or departs from [playground.md](playground.md).

- **Not yet:** 6.2, *What a difference says*: a replay's `Differs` names
  its line, its build and whether a moment or a digest differed, but not
  the line's input, the two digests or the replay's last lines
  (`skein-playground`'s `replay.rs`).
- **Departs:** 2: `skein-playground`'s manifest names two crates past
  those 2 and README.md, 3 give it: `skein-shell`, for the `Host` that
  `skein-sim-world` re-exports, and `skein-sim`, which none of its code
  uses; no reason is recorded.
