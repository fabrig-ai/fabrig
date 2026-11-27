# design: status

2026-11-27. The worker delivery path and its offline acceptance target are
described in [design.md](design.md), sections 1 and 9. The full accumulated
goal check and close run candidate assertions from the snapshot. Close selects
the published budget baseline in the source checkout. The full diagnostic is available by explicit dispatch. Sections 2–8 retain historical
detail for interpreting earlier receipts.

- **Not yet:** 1 and 4.1: a close whose checkout lacks main's counts still fetches them from the public repository into scratch, and an evaluation of budgets alone reads `/policy-counts.json`; without them close breaks, rather than starting from main's newest goal landing in `HEAD`, running its checks and judging no budget (*Without main's counts*). [in flight: change 21481]
