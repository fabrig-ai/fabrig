# The waits views' contracts

Provisional, 2026-11-26. The shapes the bounded-waits milestone's last
goal, the views (`../draft/bounded-waits.md`, 9), shares between its
changes: jig's client contract for a project's waits and an inbox need's
bound, which temper's tools and pages read. They bind while the goal
runs; its last change deletes this document once the code holds them
(`../draft/design-economy.md`, 4.3). The lasting design is jig's
`client/domain/contract/reads.md`, 1.2, and `documents.md`, 1, *an
inbox*.

## 1. A project's waits

In `jig-client-domain`'s contract, beside `Read::Tasks` and
`ReadValue::Tasks`:

- `Read::Waits { project: u32, kind: Option<WaitKind>, party: Option<u64>,
  due: Option<u64>, after: Option<Cursor> }`: `due` in nanoseconds, as
  the wire carries walls; `party` a party's number, as
  `EscalationHolder::Person` holds one; `after` the page's cursor, none
  for the first.
- `ReadValue::Waits(Box<WaitPage>)`.
- `pub struct WaitPage { pub now: Wall, pub rows: List<WaitRow>, pub
  after: Option<Cursor> }`: `now` the first page's, carried on every page;
  `after` none after the last page.
- `pub struct WaitRow { pub chip: Chip, pub wait: Wait }`: the task's chip
  and one of its waits, `Wait` the record a task's document carries.

The client wire carries them as `Read`'s and `ReadValue`'s new variants,
`ReadWaits` and `WaitPage`, as the repository's rules take an added
variant: a new layout named by its fingerprint, no version moved.

## 2. An inbox need's bound

- `InboxEntry` gains `pub bound: Option<Bound>`: on a need, its `party`
  wait's bound, `Bound` the waits' own (`jig-client-domain`'s
  `contract::waits::Bound`), and its `at` that wait's *since*; on an
  update, none. On the client wire an optional field with its default,
  none.
- A client judges a need overdue where its bound is `Bound::At { at,
  then: AtBound::Overdue }` and `at` is at or before its clock.
