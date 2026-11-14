# store: status

2026-11-14. Where temper's code lags or departs from [store.md](store.md).


- **Not yet:** 4: a failed commit's cause: `Stop::CommitFailed`'s `why`,
  `CommitStop`, `RouteStop`, `Up::Failed` and the stop line's words
  [in flight: change 61].
- **Not yet:** 2: family 41, `LandedDraft`, neither kept nor read
  [in flight: change 663].
- **Not yet:** 2: family 42, `Follow`, a follow's record [in flight: change 105].

- **Not yet:** 2: the `Change` row's `follows` [in flight: change 99].


Section 2 has no known implementation gap in provider 0's subject.
The forge's codec writes the 10-byte forge and user pair, reads it only
for provider 0 and rejects other lengths. Its table holds the exact bytes,
both integer maxima and invalid forms. There is no migration or store-version
change, as no supported deployment requires compatibility.

Section 4's `Error` record changed in place, its `too_large` carrying an
`Excess`, by goal 64: the deployment's store, written before it, is made
afresh at its next redeploy, which waits for the running goals to settle.

Section 2's family 25 stores the optional `ForgeLink` in store v11 in
place, without a version bump or migration. Both numbers must be nonzero;
the record codec and its fuzzy round trip include the field. Links for
several members remain section 7's deferred work.

Section 2's family 20 keeps kind 11, a goal template's work, in store
v11 in place: `TemplateGoal`, `design` and `close_out`, and
`SettingValue`'s `goal`, with no version bump or
migration; the store's codec reads and writes it and its slot table
holds it (`temper-protocol-store`'s `tests/settings.rs`).

Section 2's families 20, 34, 38 and 39 keep what reaches a person in
store v11 in place, by plan `ui-reaches`, with no version bump or
migration: kind 8, `SettingValue`'s `classes`, a `ProjectClasses`;
`KeepAsking`, a `PersonFact` keyed 2; `Away` with no kinds; and
`Decided` with its reach, an asked row's `took` none, and `CallDecided`'s
reach. The codec's round trips hold them
(`away_and_decided_rows_round_trip_their_options_and_bounds`,
`tests/settings.rs`, `tests/keys.rs`), a table of one item and one set
among the settings, and the codecs world's
`store_decoders_never_panic_and_reencode_the_same` reads them.

Section 2's family 20 keeps kinds 9 and 10, a charter's words and a
project's words, in store v11 in place: `CharterWords`, `line`, and
`ProjectWords`, `headline` and `lede`, and `SettingValue`'s `line` and
`words`, with no version bump or migration; the schema's ceilings bound
them, the codec refuses words that are empty or hold a control
character on reading, and its round trip at the bounds and the goldens
hold it (`tests/settings.rs`).

Sections 2 and 4 have no known gap in a repository's CI workflows:
family 2's record, the schema's `Adopted`, holds `workflows` in store
v11 in place, its ceilings the store's limits `adopted_workflows` and
`adopted_workflows_item`, taken from the forge's `WORKFLOWS` and
`WORKFLOW_BYTES`.

Section 4's `CompareAnswer` changed in place, by goal 64: the deployment's store, written before it, is made afresh at its next redeploy, which waits for the running goals to settle.

- **Not yet:** 4: `ItemsAnswer`'s `cut`, in store v11 in place.
- **Not yet:** 2: family 22 `Delivery` (no `Key` or `Stored` variant); family 30 `LandingLog`'s trailers and tier verdicts, until `../domain/after-landing.md`, sections 6, 8 and 9 need them; the families of findings, `Finding`, `Mark` and `QaPass`.
- **Design departs:** 2, from the request `ui-away` of 2026-11-05, decided by its plan's cut: `Away` keeps `off` where the request has `on`, and `Decided` keeps `party`, a field it doesn't name (`../domain/forge/attention.status.md`, 4 and 5).

- **Not yet:** 4, the tier rows' ceilings of 257: nothing checks `forge.changes` against them before the store's sizing, so a value past 128 refuses the start as past the codec's bounds, naming no limit.

- **Not yet:** 4, the tier rows' ceilings of 2049: the schema still holds 257, so `forge.changes` is at most 128.
