# settings-catalogue: status

2026-11-23. Where the tree's code lags or departs from [settings-catalogue.md](settings-catalogue.md).

- **Not yet:** 2: a read changing only the displayed omitted count
  does not mark `me`. `routing/lists.rs`, `answered`, compares good-read
  presence and `names`, but not `left_out`; `documents/small.rs`, `frame`,
  adds the visible accounts' largest `left_out` to each project's count.
  Settings can therefore keep the old count until another Frame mark. The field is computed correctly on a fresh read.
  The mark table has no row for this case; this finding is by reading,
  with no runtime result supplied. No reason for the omission is recorded.
