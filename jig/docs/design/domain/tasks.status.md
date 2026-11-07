# tasks: status

2026-11-07. Where jig's code lags or departs from [tasks.md](tasks.md).

Section 3 has no known gap in model indices retaining their catalogue
numbers and a task checking against its own revision's charter resolution.
Unlisted sections have no known gap.
Section 9's replacement of a run's own pending settings proposal has no
known gap: refusal preserves the old set, late acceptance changes nothing,
and a restart retains only the fresh pending proposal. The deployment
proposal bound remains as specified in `engine.md`, 17.6 and section 13 here.
Section 7's next-turn LLM amendments have no known gap, including silent
LLM-only amendments, the durable live-run mark and a stop whose terminal
parks without spending a try.

- **Not yet:** 3, 8.6: a task keeps no reasons, and its claim no offered
  kinds (`jig-domain-tasks/src/boundary.rs`, `TaskRecord`).
- **Not yet:** 8.2, 8.3, 8.6, 9: every message waiting is offered to a run, whatever its
  charter's parts; a turn's fence takes every message before it, offered or
  not (`jig-domain-tasks/src/inbox.rs`, `take`); and an entry left undecided
  when a run ends doesn't wake the task again (`jig-domain-tasks/src/wake.rs`,
  `after_run`).
- **Not yet:** 3, 7: a task's LLM choice has no `later` (`jig-domain-tasks/src/llm.rs`).
- **Not yet:** 5.9: a drain; a project's scope; a
  party's proposal of a release; a hold parked until an event.
  7: a widening funded by whoever gives it (the task's existing funder is
  charged); moving a task under another task; successors; questions that
  outlive their asker. 8.3: markers.
  8.5: a conversation's questions.
  12: the stories of these.
