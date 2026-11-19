# journey-writer: status

2026-11-19. Where temper's code lags or departs from [journey-writer.md](journey-writer.md).

- **Not yet:** 3: `pages` takes no `--within` and no `--check`, prints no `fresh NAME` line, sends the reprint's own lines to stdout, orders the stale journeys by name across both roots, and makes each walk's scratch inside the run's built inputs, which go only when no walk stopped [in flight: change 9579].
- **Not yet:** 4: `scripts/journey-pages.sh` needs `--project`, takes no `--branch`, `--push` or `--within`, holds no lock, keeps no commit, always pushes `journey-pages` by force, and ends its stdout with `temper ctl`'s answer [in flight: change 9581].

No other gap.
