# tasks: status

2026-11-11. Where jig's code lags or departs from [README.md](README.md).

- **Not yet:** 5, 6 and 7: a task that stands, its renewal per period and its exemption from the stall's bound (`standing.md`, sections 2 and 4) [in flight: change 103].
- **Not yet:** 3: a standing task's session (`standing.md`, 5.1) [in flight: change 106].

`lifecycle.md`, section 6's classified activation cause has no known gap. `FailedWithReason`
uses the existing reason bound and retry policy, and keeps its UTF-8 words
on the retry notice and exhausted hold through task restoration. The job
failure table checks the notice, hold and restored source.

`lifecycle.md`, section 2 has no known gap in an agent claim interrupted by its
engine's orderly stop: `End::Recheck` keeps committed turns, calls and
saved work, spends no try, adds no backoff and makes the task due again.
A crash retains the lost-or-not-started recovery rule. The core restart
table and one-process drain story check the distinction.

`lifecycle.md`, sections 6 and 7 have no known gap in a start its host refused
before the run's first turn, or in an agent's finish while its task's
delegates are live. A refused transcript is transient, any other invalid
start permanent, and an agent's busy refusal no failure
(`jig-domain-hub-link`'s `payload::ending`). A finish not asking to
cancel live delegates parks with no try spent (`run/ends.rs`,
`run_parks`), on the program backend too, as the channel world's
`a_program_finish_with_live_delegates_parks_without_a_try` shows with a
scripted program.

`README.md`, section 3 and `changes.md` and `messages.md`, sections 1, 2, 3 and 6 and `proposals.md` have no known gap in run reasons,
their retention across kept continuations and retries, offered-message
fences, a run prepared again for every reason waiting at its claim, or an
entry left undecided: one its run covered waits for a change and is
offered at the next wake, one it did not cover or refused wakes the task
again (`wake.rs`, `after_run`), and a restart wakes an idle holder only
for an entry its last run wasn't offered. A choice's `later` is checked with
its first choice, and an amended choice without it removes the old one.
Resumed parts, documents per part, choices per wake kind and offering a
message with its part remain `README.md`, section 8's later work.

Section 3 has no known gap in model indices retaining their catalogue
numbers and a task checking against its own revision's charter resolution.
A task given no LLM choice takes each model's supported charter-selected
effort, or its catalogue default when none is selected. The default-LLM
table covers main and sub-agent selections and an explicit task choice
retains its own effort.
Unlisted sections have no known gap.
`proposals.md`'s replacement of a run's own pending settings proposal has no
known gap: refusal preserves the old set, late acceptance changes nothing,
and a restart retains only the fresh pending proposal. The deployment
proposal bound remains as specified in `../engine/settings.md`, 6, and `../engine/README.md`, 7 here.
`changes.md`'s next-turn LLM amendments have no known gap, including silent
LLM-only amendments, the durable live-run mark and a stop whose terminal
parks without spending a try.

- **Not yet:** `messages.md`, 6, *A live run hears its results*: a live run is relayed
  only what its parts offer, so a coordinator's first run, whose part
  offers no `result`, hears none of its changes end, and a closer-ended
  task's last success is never relayed, even while its run lives
  (`jig-domain-tasks/src/wake.rs`, `after_message` and `rule`;
  `waiting.rs`, `next`) [in flight: change 100].

- **Not yet:**  the drain, placed-hold scope, release proposal and parked hold remain `lifecycle.status.md`, 10.
   widening, moves, successors and surviving questions remain `changes.status.md` (Not yet, the whole document).  markers remain `messages.status.md`, 3.
   conversation questions remain `messages.status.md`, 5.
  7: the stories of these.
