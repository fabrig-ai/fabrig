# Merge queue status

Provisional, 2026-10-29. Legacy selection/check/outcome policy remains the
plain default. Reviewed controller/authority invariants in 3.8–3.10 remain:
immutable P/C/tips, two outstanding records, ordered decisions, persistent
bisection, INFRA rejection of no input, ordinary explicit-C main pushes,
deletion-only leases and stable at-least-once outbox IDs.

Section 3.7.1 is implemented by the opt-in pinned repair triplet and restart's
external runtime selection. `test_repair_priority.py` covers disposable local
origins, full singleton ordering, normal batches of 16, durable tried tips,
pin refusal/advancement, red/conflict/error audit and simulated CLI/restart
launches. This enables no production action and changes no check or timeout.
It is integrated with section 3.10's authority and restart interfaces. External
runtime validation covers all fourteen frozen module/list files, refuses changes
before update and launch, and loads its authority dependencies from that copy.
The 47 priority cases cover dependency drain, persistent ownership, cross-clone
exclusion and real external-child handoff.
Review repairs cover signal/137 heavy uncertainty with durable global and
checkout refusal receipts, no input recovery/pruning and unsupported release;
consumed-pin restart with trusted state revalidation; PATH/plan refusal audit;
and opt-in adjudicated repair hold. Real disposable daemons prove official
restart through held consumed pins, same authority inode, stop drain and a
later restart without hold resuming ordinary work. No production activation
or unknown-heavy reconciliation is supplied.

Sections 3.11–3.12 now implement explicit local/SSH endpoint transport,
Linux subreaper supervision, matching heavy completion-query evidence,
controller-owned result/log/supervision return, opt-in CLI, stop/drain/resume,
same-authority restart and audited retirement. Snapshot/request versions remain
1; incompatible runtime/config/unknown versions refuse without migration.
Production rollout remains OFF until independent review and real dedicated
hetzner8/hetzner2 proof. This work activates no queue or second prover.

Offline tests use real disposable Git/process/flock/pidfd/inotify mechanisms
and injected SSH/heavy neighbors. They cover lost acknowledgements, controller
and supervisor death, surviving commands, unresolved heavy, command descendants,
corrupt artifacts, exact ordered pushes, stopped bisection, restart timeout and
handoff, enclosing drain deadlines, completed-preparation dispatch replay,
retained retirement audit and generation revival refusal. All 63
reviewed controller cases remain in their unchanged 19/22/22 split, with the
existing isolation/AST checks. Budgets and existing timeouts are unchanged.

The read-only heavy query is new runtime behavior; the prior migration's
identical-runtime proof does not validate it. Real SSH, heavy/systemd recovery,
dedicated-gate nonempty checking and rollout still require Ops validation in
transport-operations.md. No real service, SSH, Cargo/build, installation, post,
main/ready push or production operation was performed for this task.

Limits remain explicit: same-boot lost-supervisor recovery is blocked; the
supported operator path requires separately authorized host reboot plus matching
heavy release evidence. Incomplete preparation stays blocked. Frozen runtime
updates require drain/retirement; a new train generation in a retired layout is
not implemented. Cooperative ownership does not sandbox arbitrary hostile code
or unrecorded detached external jobs. There is no artifact GC, arbitrary
machine-power-loss claim, or silent reconciliation by deleting state/markers.
