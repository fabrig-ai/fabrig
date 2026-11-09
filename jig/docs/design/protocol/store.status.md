# store: status

2026-11-09. Where jig's code lags or departs from [store.md](store.md).

Section 5.1's tasks-v9 failed ends retain every optional invalid-result
kind through the store codec. The generated layout fingerprint and golden
bytes agree with the schema; no gap is known in this field.

Section 5 has no known gap in stored charter parts, task reasons and
claimed offered kinds, or `later` in task, people and template choices.
Their codecs and bounded restore checks carry these fields in the
existing family versions, under the pre-release format rule.

Sections 4.1 and 7.2 have no known gap in settings key order:
`Setting` orders Application, ProjectPolicy and Model as their stored
tags 4, 5 and 6 do. Stored bytes are unchanged. The cursor and key
tables cover these boundaries, and the key generator includes settings.
The consuming application's startup table walks an application setting
and a model setting one row at a time on open and after a crash.
This proves the order defect; the reported cancellation's cursor and
range are unknown, so it does not establish that incident's exact state.

Sections 4.4, 7.2 and 15.9 have no known gap in facts' key tag 5, their
bounded codec, restoration after the accounts and sources, or removal
with the account. The accounts family changes in place under section 5.

Section 4.4 has no known implementation gap in account list records,
model first-read times or the provider-list revision decider. The list
record and its catalogue decision share the commit and restart boundary.

Sections 5 and 5.1 have no known gap in loading model-setting names before
family restoration and taking catalogue updates before their commit, so a
person's stored model retains its number across restart.
Section 14's seven whole generation, patch, check and frame sweeps are
under `fuzzy::`, with the recovery bodies shared by their focused cuts.
The focused stray-cleanup cut's mismatch with `../domain/testing.md`, 9
is recorded in `../domain/testing.status.md`; it does not remove a fuzzy
cut or a recovery check. Whole-suite budget proof remains open there.

- **Not yet:** 7.3, 11.1: retention: no horizon shrinks an ended task to its summary or drops its transcript; a page of history, spend or notes cut before its first row. 10.5, 14: rehearsing and going back: `jig/crates/jig-protocol-store` has no `Request::Rehearse`, `Request::Back`, `Event::Back` or `BackRefusal`, and `jig/tests/store` no story of them. 14: the reference binary killed around each phase of a turn's commit (`jig/testing/jig-test/tests/process.rs` kills and restarts it once); `jig/tests/ops` and `jig/testing/jig-conformance` run `jig-fake-store`, where no world fakes the store.
- **Departs:** 14: `jig/tests/store/src/model.rs` reads a copy as a settings revision's and panics on any other row, because the core copies only that one, so no story reaches it; `jig/crates/jig-protocol-store/src/tests/commits.rs` holds a copy of another row and section 6's two rules of a copy.
- **Not yet:** 14: the core's world on the simulated disk: no row kills the
  engine at a file operation of a decision's commit and starts it again
  through its link pass; `jig-test-store-files` selects its directory-backed
  store only for patch sets and crashes it at a seed, never at a chosen
  operation.
