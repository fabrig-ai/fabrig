# codec: status

2026-11-18. Where skein's code lags or departs from [README.md](README.md).

- **Not yet:** 4, 6: generated modules hold no `VERSION`, their versioned records naming the version as a literal; `LIMIT_NAMES` is a `[&str; N]` and `Limits::values` returns a `[u32; N]`; a record of more than three booleans holds them in an array; a split's helpers and decoded fragments are numbered, and family-wide functions are split in chunks by count [in flight: change 7918].
- **Departs:** 6: an option whose value is an option generates `Option<Option<T>>`, which `clippy::option_option` rejects, so generated code does not pass the lints there. Only the pin, `every-v3`, nests options; its module declaration in `skein/tests/codec`'s library carries a scoped `#[expect(clippy::option_option, ...)]`, and the generated text carries none. A family that nests options in earnest takes the generator's fix first. Decided by the goal's coordinator, 2026-11-18, to keep the pin off a generator change.
