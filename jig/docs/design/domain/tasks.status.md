# tasks: status

2026-11-08. Where jig's code lags or departs from [tasks.md](tasks.md).

Section 5.2 has no known gap in an agent claim interrupted by its
engine's orderly stop: `End::Recheck` keeps committed turns, calls and
saved work, spends no try, adds no backoff and makes the task due again.
A crash retains the lost-or-not-started recovery rule. The core restart
table and one-process drain story check the distinction.

Sections 3, 7, 8.2, 8.3, 8.6 and 9 have no known gap in run reasons,
their retention across kept continuations and retries, offered-message
fences or wakes for undecided entries, including a read escalation
awaiting an unanswered direct person delegate. Exact reads are kept for
the attempt; a restart reoffers unresolved entries without changing their
holder, entry or stall time. A choice's `later` is checked with
its first choice, and an amended choice without it removes the old one.
Resumed parts, documents per part, choices per wake kind and offering a
message with its part remain section 13's later work.

Section 3 has no known gap in model indices retaining their catalogue
numbers and a task checking against its own revision's charter resolution.
A task given no LLM choice takes each model's supported charter-selected
effort, or its catalogue default when none is selected. The default-LLM
table covers main and sub-agent selections and an explicit task choice
retains its own effort.
Unlisted sections have no known gap.
Section 9's replacement of a run's own pending settings proposal has no
known gap: refusal preserves the old set, late acceptance changes nothing,
and a restart retains only the fresh pending proposal. The deployment
proposal bound remains as specified in `engine.md`, 17.6 and section 13 here.
Section 7's next-turn LLM amendments have no known gap, including silent
LLM-only amendments, the durable live-run mark and a stop whose terminal
parks without spending a try.

- **Not yet:** 5.9: a drain; a project's scope; a
  party's proposal of a release; a hold parked until an event.
  7: a widening funded by whoever gives it (the task's existing funder is
  charged); moving a task under another task; successors; questions that
  outlive their asker. 8.3: markers.
  8.5: a conversation's questions.
  12: the stories of these.
