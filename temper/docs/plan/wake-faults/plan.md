# wake-faults: plan

Three faults a goal met on a real deployment (`7638040201`, the third again
at `b4555e6785`), and a fourth the user added: a goal's first run ending
invalid after successful reads, a repair's attempts ending invalid before
any turn, a parked goal woken again and again by an escalation it had
passed to its person, and a run woken as a result came due missing the
part for results. The request ranks five wants; all five are in the plan,
in five tasks. Nothing was dropped.

## What each fault was

Read-only surveys traced each from the code; the evidence is the request's
words alone.

1. **Invalid after successful reads: not `program-burst-files`.** That
   fault's panic aborts the agent process, which jig classes as *the
   agent's* (`jig-domain-host`'s `gone`, `Failure::Agent(Exited)`), never
   invalid. The goal's contract is a report (template 4). The path that
   fits every word: on the program backend a run with a contract ends only
   through Claude Code's `StructuredOutput` (smith's
   `protocol/programs.md`, 5.5); the goal's model ended its cut with a
   report, a minute's writing at xhigh after its last read, while the
   delegates it had just made were live. The tasks child refuses a finish
   while delegates are live (`Problem::live_delegates`), but the run has
   ended, so the core's refused-answer route fails the attempt
   `Invalid` with no reason (`route_refused_payload`), and the next run is
   told nothing; so it did the same, twice more. `RunEndFailed` would have
   said `invalid: unreadable_result` (a `None` reason). The direct backend
   has the same path. Fixed by the cause: such a finish parks the run.
2. **Invalid before any turn:** every start a host refuses before the
   run's first turn, other than for its transcript, reaches the engine as
   `Failed(Invalid)` at one line (`jig-domain-hub-link`'s
   `payload::ending`), and temper's worker sends smith's own *busy* (no
   room for an agent or a run) the same way. The design already says such
   a start is *permanent* (an assignment its host found invalid) and a busy
   one no failure; the code departed. Which refusal the repair met can't
   be read from here: its words reach serve's log
   (`temper/docs/design/domain/worker.status.md`). The retries came fast
   because every class's backoff starts at 0.5 to 1 s and doubles (the
   small profile's `RETRY`, base 1 s; `failures::backoff`); the hold at count 4
   is 1 attempt plus 3 retries, as designed. The fix classes the start
   permanent, or busy.
3. **A parked goal woken 19 times:** the read escalation's person-answer
   wait (`0bf4dca720`) held only for a *direct* person delegate and a read
   kept in memory; the goal's run evidently missed one of them. Replaced
   by the user's settled rule, which needs no stored field: the task
   already keeps the kinds its last run was offered.
4. **A due result's part missed:** reasons are taken at preparation from
   every waiting kind, but a word arriving between preparation and claim
   that the prepared parts don't offer is kept back, and the prepared
   reasons put back (`jig-domain-core`'s `preparation_current`,
   `preparation_facts`). The story
   `a_question_arriving_after_preparation_waits_for_the_next_run` pinned
   that; it flips.

## Design

Written by this cut, released with the specs:

- jig's `domain/tasks.md`: 5.5 (*invalid* is never a refused start), 5.6
  (*A task ends after its delegates*: an agent's finish parks), 8.1 and
  8.3 (an entry's wake), 8.6 (*Every reason waiting at its claim*; *An
  entry left undecided waits for a change, once a run covered it*,
  replacing the person-answer wait), section 9 (*Waking*), and the open
  question on other waits removed as settled.
- jig's `domain/engine.md`: 7.1 step 2, 7.4 (*Finished while its
  delegates are live, it parks*; *An entry left undecided*), section 8 (a
  refused start's class).
- jig's `domain/hosts.md`: 6.3 and section 8 (`FromAgent::StartBusy`).
- temper's `domain/worker.md`, 4.3.
- The draft `per-wake.md`, section 4, as the request asks, and its reasons
  in `draft/notes.md`, section 8.
- **Not yet** entries in `tasks.status.md`, `engine.status.md`,
  `hosts.status.md` and `worker.status.md`; each task deletes its own.

## Decisions

- **A charter with no parts covers every kind** (and a task with its own
  instructions): its runs read their instructions whole, and are offered
  `Reasons::ALL`. So the covered rule is one test, `offered.meets(kind)`,
  and a no-parts task left undecided isn't woken again by itself either,
  as before per-wake's re-wake existed. A refused entry still wakes.
- **"A new revision or a message about it"** is what the code already
  wakes on: a new hold or a new holder makes a new entry, which wakes as
  an arrival does, and any message the task's policy lets wake it brings
  the entry along (its kind joins the reasons).
- **Item 1's fix parks rather than names the reason.** Naming it would
  need a stored value (an `InvalidResult` member or a note for the next
  run): the request makes that a question first, so it is the plan's
  question `parked-finish-told`, recommending no. Parking is what the
  model meant, spends no try, and is what `engine.md`, 7.4 already does
  for a finish while kept. It lives in the tasks child, beside that rule;
  only agent executors park, so the tasks tier's refusal for others
  stands.
- **A refused start spends a permanent try, with no new value:** every
  class and retry policy exists. Its words still go to serve's log, not
  the retry notice: carrying them would change the stored `tasks::End`.
- **Every reason at the claim prepares the run again** when a new reason
  arrives, at most once per kind. A question arriving between preparation
  and claim now joins the run too: the request asks for every kind
  pending when a run starts.
- **Found, not in scope:** smith caps a report's text at 1,024 bytes
  (`jig-protocol-charter`'s `CONTRACT_TEXT_MAX`) where template 4 promises
  `tasks.result_bytes`, 63,738; it fails safe (smith tells the model), so
  it is no part of these faults. A figure, which the request leaves out.

## Parts and tasks

| Part (slice) | Task | Needs | Minutes |
|---|---|---|---|
| `finish-parks` (1) | `finish-parks`: the channel world's failing row with a scripted `claude`; the park in `jig-domain-tasks`; the tasks tier's pins | none | 75 |
| `covered-entries` (2) | `covered-entries`: per-wake rows 868, 869; the rule in `wake::after_run` and both restores; the exception removed; `offering.rs` rows; the help page | none | 100 |
| `late-reasons` (3) | `late-reasons`: per-wake rows 866 (rewritten), 870, a result part on the test charter; `preparation_current` prepares again | none | 80 |
| `start-classes` (4) | `start-refusals`: the hub link's table and mapping, `FromAgent::StartBusy`, jig's test worker | none | 60 |
| | `worker-busy`: temper's worker sends `StartBusy` for smith's busy | `start-refusals` | 45 |

All released in the first push; contracts in `notes/contracts.md`, rules in
`notes/common.md`.

## Critical path

`start-refusals` (60) then `worker-busy` (45): 105 minutes. Beside it,
`covered-entries` (100), `late-reasons` (80) and `finish-parks` (75) start
at once. The plan lands when all five have merged.

## Overlaps

- **Within the plan:** `finish-parks` and `covered-entries` both write
  `jig-domain-tasks`, on disjoint lines (`notes/contracts.md`, section 4):
  only `run.rs` is shared, its terminal admission against its claim's
  reset. Running them one after the other would put 175 minutes on the
  path for one line's distance. `covered-entries` and `late-reasons` both
  add rows to the per-wake story table and its kit, at the places
  section 3 assigns. `tasks.status.md` and `engine.status.md` carry
  several tasks' entries, each deleting its own sentence.
- **With the drafts beside this one:** `goal-call-faults` (its cut in
  flight) will likely touch jig's `domain/tasks.md` and `tasks.status.md`
  (its item 2, a task subscribing to its own children) and perhaps
  `jig-domain-tasks`; its files aren't known yet. No other draft beside
  this one names a file of this plan's.
