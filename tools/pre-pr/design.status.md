# design: status

2026-11-27. The worker delivery path and its offline acceptance target are
described in [design.md](design.md), sections 1 and 9. The full accumulated
goal check and close run candidate assertions from the snapshot. Close selects
the published budget baseline in the source checkout. The full diagnostic is available by explicit dispatch. Sections 2–8 retain historical
detail for interpreting earlier receipts.

- **Not yet:** 4.1, *Every failure at once*: the close passes
  `--fail-fast` to both runs, skips the fuzzy run after a failed focused
  one, and stops every test at a format refusal or a failed helper
  build, `cargo check` or listing. [in flight: change 21054]
