# codec: status

2026-11-18. Where skein's code lags or departs from [README.md](README.md).

- **Not yet:** 4: the CLI's generation writes golden files beside the records' module [in flight: change 7631].
- **Not yet:** 4, 6: `Output` also holds `rust`, `tests` and `goldens`, and `Tests`, `tests_main` and the test emitter remain [in flight: change 7639].
- **Not yet:** 6: skein-codegen has no `goldens`, `Construct`, `CONSTRUCTS`, `constructs` or `unpinned` [in flight: change 7631].
- **Not yet:** 6, 9.3: `every` has no version holding every construct, no test that it does, and no digest literal of its current version; its tests are generated [in flight: change 7633].
- **Not yet:** 6: skein's other codec fixtures, `skein-oauth` and the UI example keep golden files and generated tests [in flight: change 7633].
- **Not yet:** 6: jig's families' drift compares golden files and generated tests, which jig's codec crates keep [in flight: change 7634].
- **Not yet:** 6: the jig store's memory, fuzzy and hand-written tests read golden files [in flight: change 7637].
- **Not yet:** 6: the jig client wire's, public wire's and engine cursor's tests read golden files [in flight: change 7638].
- **Not yet:** 6: smith's families keep golden files and generated tests, which smith's and jig's tests read [in flight: change 7635].
- **Not yet:** 6: temper's families keep golden files and generated tests, which temper's tests read [in flight: change 7636].
- **Departs:** 6: an option whose value is an option generates `Option<Option<T>>`, which `clippy::option_option` rejects, so generated code does not pass the lints with no allowances there. Only the pin, `every-v3`, nests options; its module declaration in `skein/tests/codec`'s library carries a scoped `#[expect(clippy::option_option, ...)]`, and the generated text carries none. A family that nests options in earnest takes the generator's fix first. Decided by the goal's coordinator, 2026-11-18, to keep the pin off a generator change.

