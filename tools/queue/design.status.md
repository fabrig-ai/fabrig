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

The controller, transport, durable train state, independent result ingestion,
ordered promotion, full-check reservations, notification replay, conditional
ready deletion, remote command supervision, persistent legacy startup guards,
reconciliation tooling, drain and train-aware restart are absent (section 4).
An unresolved worker clone requires explicit operational reconciliation; the
local ownership marker cannot establish heavy quiescence. No production
validation or real Cargo/systemd run is claimed by the disposable fake suites.
