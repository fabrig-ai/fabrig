# store: status

2026-11-03. Where jig's code lags or departs from [store.md](store.md).

- **Not yet:** 5.1: the tasks family's dependencies marked started from (`../domain/connectors.md`, 6.1): `tasks-v9`'s `TaskRecord` keeps none. 7.3, 11.1: retention: no horizon shrinks an ended task to its summary or drops its transcript; a page of history, spend or notes cut before its first row. 10.5, 14: rehearsing and going back: `jig/crates/jig-protocol-store` has no `Request::Rehearse`, `Request::Back`, `Event::Back`, `BackRefusal` or `levels`, and `jig/tests/store` no story of them. 14: the reference binary killed around each phase of a turn's commit (`jig/testing/jig-test/tests/process.rs` kills and restarts it once); `jig/tests/ops` and `jig/testing/jig-conformance` run `jig-fake-store`, where no world fakes the store.
