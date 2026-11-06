# store: status

2026-11-05. Where jig's code lags or departs from [store.md](store.md).

- **Not yet:** 7.4: the lines after a position. 7.3, 11.1: retention: no horizon shrinks an ended task to its summary or drops its transcript; a page of history, spend or notes cut before its first row. 10.5, 14: rehearsing and going back: `jig/crates/jig-protocol-store` has no `Request::Rehearse`, `Request::Back`, `Event::Back`, `BackRefusal` or `levels`, and `jig/tests/store` no story of them. 14: the reference binary killed around each phase of a turn's commit (`jig/testing/jig-test/tests/process.rs` kills and restarts it once); `jig/tests/ops` and `jig/testing/jig-conformance` run `jig-fake-store`, where no world fakes the store.
- **Departs:** 14: `jig/tests/store/src/model.rs` reads a copy as a settings revision's and panics on any other row, because the core copies only that one, so no story reaches it; `jig/crates/jig-protocol-store/src/tests/commits.rs` holds a copy of another row and section 6's two rules of a copy.
