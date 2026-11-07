# forge-github: status

2026-11-07. Where skein's code lags or departs from [forge-github.md](forge-github.md).

- **Not yet:** 11: `GET /app`'s `owner` isn't decoded, nor served by the dialect's server half. 13: no fact names that `owner`, and no live run has observed it. 4.5: the GitHub routes of `Runs`, `Run`, `Artifact`, `Dispatch`, `Rerun`, `Tree`, a fast-forward-only `Merge` and a rebase `Update`, which the dialect answers `Refused` (`skein-forge/src/github/tests.rs`, `turn_calls_are_refused_without_a_request`); of `UpdateRef` and the labels, which no dialect has (forge.md, 3.2); `Protection`'s `pulls_required` and `pulls_bypass`; and the stories of 4.5 on the fake GitHub.
