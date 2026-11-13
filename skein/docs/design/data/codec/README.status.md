# codec: status

2026-11-13. Where skein's code lags or departs from [README.md](README.md).


- **Not yet:** 2: `skein-oauth` and `skein-ui-example-view` hold their generated record code beside code with other workspace dependencies; each gets its codec crate in G3 of `temper/docs/design/draft/crate-graph.execution.md`, and until then is an exception of the codec check (`tools/crates/design.md`, 3.2).
- **Not yet:** 4, 6: `generate` writes the tests inside the records' module, with no tests' module apart and no `tests_main`, and skein's codec fixtures have no `tests/generated` target [in flight: change 2450].
- **Not yet:** 6: temper's codec crates' generated tests are inside their records' code, and `temper-protocol-store-codec`'s library builds a test target [in flight: change 2452].
- **Not yet:** 6: jig's codec crates' generated tests are inside their records' code, and the libraries of jig's store and codec crates that hold no other tests build a test target [in flight: change 2454].
- **Not yet:** 6: smith's codec crates', `skein-oauth`'s and the UI example's generated tests are inside their records' code [in flight: change 2455].
- **Not yet:** 4: `Output` still has `rust`, the records' module with its tests inside, for the drift tests not yet writing the two apart [in flight: change 2456].
