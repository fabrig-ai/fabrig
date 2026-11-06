# D034: The core observer rebuilds its snapshot at each commit

- **Kind:** tests
- **Where:** `jig/testing/jig-conformance/src/core.rs`, `jig/testing/jig-core-kit/src/conformance.rs`
- **Seen:** 2026-10-18 at `86971be`, in jig's tech-debt list ("Test cost": conformance and `ops`'s alert story; the core world's observer)
- **Seen:** 2026-10-31 at `1b5c98834`, in the quality run's debt list ("Ops (A12)": `alert_cuts` replays its baseline; "Tech-debt cross-checks still open": the core observer rebuilds its snapshot per commit)
- **Checked:** 2026-11-06
- **Value:** medium (the worlds' CPU spent projecting durable state)
- **Size:** M
- **Status:** open
- **Paid when:** the core world's observer updates from each commit's rows.

## What

The adapters skip view construction when the observer has already applied the
store's commit. On a new commit, `durable_at` still rebuilds
`snapshot(&store.rows)` whole. A temporary thread-CPU timer measured its share
of the suites' test-process CPU after that guard:

| Suite | `durable_at` CPU | Suite CPU | Share |
| --- | ---: | ---: | ---: |
| `jig-core-world`, focused | 2.214 s | 27.926 s | 7.9% |
| `jig-conformance-world`, fuzzy | 1.420 s | 14.309 s | 9.9% |
| `jig-ops-world`, fuzzy | 0.320 s | 17.245 s | 1.9% |

These are one run each, including timer overhead. Feeding only a commit's rows
would leave the whole `snapshot` rebuild and the referee's walk of every task
per commit. The snapshot's unkeyed `Vec`s, including writers and balances, and
summed person spend need keyed replacement and erasure accounting before they
can consume a row feed. The observer's `discarded` set is cleared and rebuilt,
`reached` receipts are collected from the current inboxes, and the applied
commit generation advances once per commit: replaying these operations per row
is not idempotent. A feed must preserve those commit boundaries. `Got` also
changes the store's rows outside commits, so cold observation needs the current
rows even when `applied` is unchanged.

`Observer::inbound` also builds the full view for every core event
(`jig/testing/jig-core-kit/src/conformance.rs`'s `inbound`,
`jig/tests/ops/src/conformance/mod.rs`'s `inbound`). And
`Neighbours::busy_events` keeps `{event:?}` strings
(`jig/tests/conformance/src/lib.rs`).

`jig/tests/ops/tests/suite/fuzzy/joint.rs`'s six acceptance and move tests keep
their own baselines on purpose: each runs its own seed (531 through 536).

## Why it matters

Each commit projects the whole durable state and validates every task. That
work grows with both a story's length and its retained rows.

## What pays it back

Feed the observer each commit's rows, with keyed snapshot accounting and
commit-level observations, and check equivalence against whole-store projection.
Typed replay equality reports the first differing field; the alert story's cuts
share one baseline inside their table test. Nextest gives each test a process,
so separate tests cannot share a process-local baseline.
