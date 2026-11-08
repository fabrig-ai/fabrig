# codec: status

2026-11-08. Where skein's code lags or departs from [codec.md](codec.md).

- **Not yet:** 11.8: the patcher's step tests
  (`skein/crates/skein-codec/src/patch/run/tests.rs`) compare tables'
  addresses, not the counting allocator's count around a reused pass,
  and `nested_walks_grow_both_stacks_and_reuse_them` reads its stacks
  before the first walk deepens them.
