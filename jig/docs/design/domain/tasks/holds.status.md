# holds: status

2026-11-27. Where the tree's code lags or departs from [holds.md](holds.md).

- **Not yet:** 3, *Each use checks its writer*: taking a slot doesn't assert that no writer held it, and a release names no expected writer, freeing whatever slot the resource has or nothing, silently (`jig-domain-tasks`, `writers.rs`, `take`, `free`, `effect_settled`, `read_afresh`). [in flight: change 19637]
