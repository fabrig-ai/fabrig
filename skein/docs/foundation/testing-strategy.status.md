# testing-strategy: status

2026-11-26. Where the repository's code lags or departs from [testing-strategy.md](testing-strategy.md).

- **Not yet:** 6: no tree runs transition coverage with `cargo llvm-cov`.
- **Not yet:** 6: no tree fuzzes a machine or decoder with `cargo fuzz`.
- **Not yet:** 1, 8 and 8.1: no campaign runs. No sweep takes its cases through skein's seam, so a failing case names its seed only where its test prints one, and no tree has a campaign pair or `tools/tiers/campaign` (`temper/docs/design/draft/fuzzy-campaigns.md`, 4, 5 and 9).
- **Not yet:** 8: no world draws agents, so none keeps a capture to pin (`temper/docs/design/draft/misbehaving-agents.md`, 4.4).
