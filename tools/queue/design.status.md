# Merge queue status

Provisional, 2026-10-29. The tools discovery policy in
`tools/queue/design.md, 2.3` is implemented by `mq` and checked by
`test_mq.py`: tracked Python suites, executable shebangs, and the exact
manual-only heavy recovery exclusion. Other queue behavior is unchanged
by this migration.
