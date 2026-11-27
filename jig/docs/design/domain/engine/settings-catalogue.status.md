# settings-catalogue: status

2026-11-27. Where the tree's code lags or departs from [settings-catalogue.md](settings-catalogue.md).

- **Not yet:** 1: the root's `Ask::Catalogue` (temper's engine and jig's
  test domain) writes the store's names in list order, not one per
  number, so a model past a gap lands in the wrong slot. [in flight: change 20459]
- **Not yet:** 1: the settings check takes any number below the highest
  for a model: an authority's bit or a template's choice naming a hole
  passes. [in flight: change 20460]
- **Not yet:** 2: a read changing only the displayed omitted count
  does not mark `me`. `routing/lists.rs`, `answered`, compares good-read
  presence and `names`, but not `left_out`; `documents/small.rs`, `frame`,
  adds the visible accounts' largest `left_out` to each project's count.
  Settings can therefore keep the old count until another Frame mark. The field is computed correctly on a fresh read.
  The mark table has no row for this case; this finding is by reading,
  with no runtime result supplied. No reason for the omission is recorded.
