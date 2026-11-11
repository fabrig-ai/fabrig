# opening: status

2026-11-11. Where the tree's code lags or departs from [opening.md](opening.md).

- **Not yet:** 5: rehearsing and going back: `jig/crates/jig-protocol-store` has no `Request::Rehearse`, `Request::Back`, `Event::Back` or `BackRefusal`, and `jig/tests/store` no story of them [in flight: change 670]
- **Not yet:**  retention: no horizon shrinks an ended task to its summary or drops its transcript; a page of history, spend or notes cut before its first row.  the reference binary killed around each phase of a turn's commit (`jig/testing/jig-test/tests/process.rs` kills and restarts it once); `jig/tests/ops` and `jig/testing/jig-conformance` run `jig-fake-store`, where no world fakes the store.
