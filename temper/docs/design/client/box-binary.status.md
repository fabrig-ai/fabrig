# box-binary: status

2026-11-25. Where temper's code lags or departs from [box-binary.md](box-binary.md).

- **Not yet:** 2, 3 and 6: the download fetches `temper-box.zst`, checked against its sums' line, stages it in `temper-box.zst.part` and unpacks it with `ruzstd`'s decoder, refusing a truncated, damaged, too long or trailed frame; the hand-fetch line pipes through `zstd -dc`. 7: the unpacking's rows. [in flight: change 17931]
