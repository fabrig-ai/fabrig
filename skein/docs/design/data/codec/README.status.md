# codec: status

2026-11-18. Where skein's code lags or departs from [README.md](README.md).

- **Departs:** 6: an option whose value is an option generates `Option<Option<T>>`, which `clippy::option_option` rejects, so generated code does not pass the lints there. Only the pin, `every-v3`, nests options; its module declaration in `skein/tests/codec`'s library carries a scoped `#[expect(clippy::option_option, ...)]`, and the generated text carries none. A family that nests options in earnest takes the generator's fix first. Decided by the goal's coordinator, 2026-11-18, to keep the pin off a generator change.
