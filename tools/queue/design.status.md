# Merge queue status

Provisional, 2026-10-29. Sections 1–3.7 retain the one-gate queue. Explicit
parent/candidate inputs, writing ratchets, offline shebang discovery and
batch-size restart prerequisites are present and remain covered by
`test_mq.py`.

Section 3.8 is an implemented **local-only review proposal**, tested by
`test_worker.py`: explicit opt-in, independent clone ownership, immutable
candidate checking, frozen runtime/list identity, check-only ratchets and
private atomic result/log evidence. It is not installed or called by the
queue/restart. It activates no second prover. The request/result contracts follow the approved section 3.8 design;
production deployment remains disabled. Ops validated a documentation-only
empty-plan job on the dedicated second gate: exact bundle import, candidate
identity, checkout exclusion and private atomic evidence passed. This smoke
ran no Cargo checks and does not validate a second prover.

Section 3.9 is implemented as a **disabled, unready library** in `train.py`,
with 63 disposable tests in `test_train.py`. It prepares exact chained Git
candidates in a separate clone, writes ratchets before freezing, pins/exports
objects, reserves names/full cadence, records dispatch ownership, independently
validates complete worker/log evidence, and adjudicates in order. Sequential
bisection, stop/drain state, version-1 atomic snapshots, explicit promotion
intents, applied-once accounting, stable at-least-once outbox and injected
reconciliation/effect interfaces are present. There are no changes to Queue's
CLI/default behavior and no production adapter or train entry point.

Partial conflicts block successor speculation while the unaffected merged
subset is checked in its original attempt against pinned P/C. Conflict tried
marks and rejections remain provisional through transport, checking and result
validation. Green lands that exact candidate and applies both merged-input and
conflict decisions atomically; red bisects only the merged inputs with its full
requirement preserved. INFRA at any stage retains no branch rejection/tried
mark and does not release uncertain ownership. All-conflict batches can still
be adjudicated without worker checks. Replay tests cover combined decisions
and outbox acknowledgements as well as stop/drain and ancestor invalidation.

The train compares the full parent/candidate trees, including generated lower
baselines; this can conservatively escalate to full rather than retaining the
one-gate pre-baseline selection in 3.3. Ratchet preparation reds conservatively
force subsequent bisection full. Exact worker plan validation remains intact.

Part 4 remains required: common legacy/train/restart authority lock and
persistent startup guards; production fresh-parent/explicit-OID effects and
conditional cleanup/post adapters; preparation and worker reconciliation;
state structural hardening, artifact repair/retention and recovery validation.
The core's injected guard and explicit evidence hooks do not supply those
production guarantees. Unknown state versions refuse without migration.

Part 5 remains required: SSH and supervised job transport with proved lifetime
ownership, operational stop/drain/resume/restart integration and rollout. Both
live train gates stay off; production activation is prohibited. An unresolved
worker clone requires explicit operational reconciliation; a local marker or
process absence cannot establish heavy quiescence. Fake tests include real
local Git bundles/merges/lease deletions and injected publication failures,
not production services, remote supervision, real Cargo/systemd or arbitrary
machine-crash proof.
