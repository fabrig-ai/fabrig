# after-landing-runs: status

2026-11-25. Where the tree's code lags or departs from [after-landing-runs.md](after-landing-runs.md).

- **Not yet:** 2, by plan `forge-polls-quiet`: a tier polls its run in
  flight every `poll`, not at the client's `poll_at`.
- **Not yet:** 2, *A run whose tier is no longer configured*: a
  restart builds a follower for such a run and panics reading its
  tier's settings; nothing ends it. [in flight: change 17682]
