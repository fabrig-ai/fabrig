# milestones: status

2026-11-18. Where the code lags or departs from [milestones.md](milestones.md).

- **Not yet:** 8: a milestone's landings are counted on the forge's
  default branch rather than the repository's landing branch [in flight:
  change 8087].
- **Not yet:** 7 and 9: `MilestoneTotals` has no `reds`, and the status agent's milestone line no `; R reds owed`.

- **Design departs:** 7: an ended milestone's totals come from the forge's row alone; jig's ended record is not read, since its archive loader serves no forge read. One whose row was dropped (section 8's bound) answers `not_milestone`.
