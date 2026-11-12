# worker-reply-bound: status

2026-11-09. Where the design lags or departs from [worker-reply-bound.md](worker-reply-bound.md).

Archived: the design carries the draft's vision in full. No design lag,
departure or open decision remains.

The hello's optional reply bound is `jig/docs/design/protocol/channel/records.md`,
1 and `jig/docs/design/domain/hosts/README.md`, section 2; the fleet keeping it
by a run's live attempt is `jig/docs/design/domain/engine/fleet.md` and `jig/docs/design/domain/engine/settings.md`, section 8
. A temper worker naming `host_reply_text` is
`temper/docs/design/domain/worker/README.md`, section 5; a task read's room and
its rendering are `temper/docs/design/domain/routing.md`, 2 and
`temper/docs/design/domain/agent/tools.md`. Where the worker names none,
the local agent's bound applies, else `documents.text_bytes`.

The six-row `task_reads_take_their_workers_reply_bound` table in
`temper/tests/engine/domain/tests/suite/host_calls.rs` covers the larger
worker-only page, a smaller worker bound, both fallbacks, continuation
and the journal ceiling. No implementation gap is known for these parts.
Real-deployment savings remain unobserved
(`temper/docs/design/domain/root.status.md`, **Not yet:** 5.2).

A charter's words and any figure remain outside the draft's scope; neither
is follow-up work for this plan. The separate `see` rendering question
remains in `temper/docs/design/domain/agent/README.md`, section 8.
