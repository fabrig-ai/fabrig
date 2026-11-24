# A turn told in parts

Provisional, 2026-11-21, from temper's `draft/worker-restarts.md`,
sections 3, 6 and 7 (a crash, 2026-11-24). A turn is told whole once its last call has answered
(run.md, section 4, step 6). A host that keeps a run's place across its
own restart needs more: the LLM's answer kept as soon as it arrives, and
each call's answer as it ends, so that a drain need only wait for the
answer in flight, and the next activation runs again what the drain cut
rather than planning the turn afresh. This document is that: a turn's
parts, the drain at the answer, and a cut turn resumed. The checks'
results directory, which lets a check cut by a drain skip what passed,
is `run-delivery.md`, 1.1's.

## 1. In one page

- **Parts, beside whole turns** (section 2): a completion's answer, told
  when it arrives and before its calls run; each call's result, told
  when the call ends; and a sub-agent's window, told when its
  compaction opens one. Each part's body is a turn's record
  (protocol/transcript.md, section 2). The whole turn is still told as
  before, and only it enters the transcript.
- **Only for a host that asks** (section 3): the start's `tell_parts`.
  A host that keeps no parts gets none and drains at the end of the
  turn, as run.md, section 9, has it.
- **Every conversation's,** main's and each sub-agent's, each part
  naming its thread, and a sub-agent's the call it answers.
- **A drain at the answer** (section 4): with a grace in the drain, the
  run starts no completion, lets the answers in flight arrive, gives the
  calls in flight the grace, and then cuts them. A cut command's output
  so far and how long it ran are told as its result; nothing else is
  told for the cut turn, which stays open.
- **The cut turn resumed** (section 5): the next activation rebuilds the
  turn from the window and its parts, asks the LLM nothing for it, and
  answers each call that has no result by its kind: a check runs again,
  a host call or a delivery is answered from what the host decided or
  asked anew, a sub-agent's conversation goes on from its own parts
  (section 6), a call never started runs now. The model sees each
  answer as if the call had run once, and a command cut midway as
  interrupted, for it to judge.
- **A crash takes the same path** (sections 3 and 5): a command and a
  `job` wait tell a start as they start, an `interrupted` result with
  no time and no output, which their end's result supersedes. A crash
  tells nothing more, so the next activation finds each command it cut
  interrupted, after an unknown time, and runs again only what never
  started.

## 2. Parts

A part is one of three kinds, told on its own record
(protocol/channel.md, section 4), its body the bytes of a turn's record
(protocol/transcript.md, section 2) as the conversation's turns are
encoded:

- **an answer:** the conversation's messages since the last turn or
  answer it told, up to and including the assistant message the
  completion answered, with that completion's usage, the run's spend
  so far and the conversation's jobs. For main, that is everything its
  whole turn will hold before the calls' results, since main tells each
  whole turn before its next completion. For a sub-agent, which tells no
  whole turn, it includes the previous answer's results message and
  anything appended after it. A conversation's first answer is an
  opening turn: its affinity, account, backend and system text, as the
  conversation's first turn carries them (protocol/transcript.md,
  section 3);
- **a result:** one user message holding one call's result block, as
  the whole turn will keep it: the call's provider id and what it
  returned (protocol/transcript.md, section 2), and the run's spend so
  far. A call may tell more than one: a command's start, then its
  end's or a drain's (section 3); the last told stands;
- **a window:** a sub-agent's window turn, which its compaction made
  (`session-context.md`, 1.2). Main's compactions are told as window turns, as
  before.

Beside its body, a part carries:

- **its number,** from one in each activation, its own sequence apart
  from the turns';
- **the turn it belongs to:** the number main's turn in flight will
  have in this activation, the last turn told plus one, as a live
  piece's `turn` (protocol/channel.md, section 4). A sub-agent's part
  carries main's, since a sub-agent lives within one call of main's
  turn;
- **its thread:** main's zero, a sub-agent's its ordinal (run.md, 5.3;
  session.md, section 2);
- **its call:** for a sub-agent's part, the position of the
  `sub_agent` call it answers among main's answer's blocks, as that
  call's name carries it (run.md, 5.2); zero for main's, and unread.

The run's domain tells a part as a request of its own:

```rust
// smith-domain-session::record
pub enum PartKind { Answer, Result, Window }
pub struct Part { pub thread: u32, pub call: u32, pub kind: PartKind, pub body: Turn }
/// A part as a start hands it back: the activation and turn it was told in.
pub struct Resumed { pub activation: u64, pub turn: u32, pub part: Part }

// smith-domain
pub enum Request { /* as before */ Part { host_run: Token, number: u32, turn: u32, part: record::Part } }
pub enum Event {
    /* as before */
    Drain { run: Token, grace: Option<Duration> }, // none: at the turn's end (run.md, section 9)
    PartKept { run: Token, number: u32 },          // that part and every one before it are durable
}
pub struct Start { /* as before */ pub tell_parts: bool, pub parts: Box<[record::Resumed]> }
```

## 3. Telling them

- **When the start asks:** its `tell_parts` (protocol/channel.md,
  section 3). Without it the run tells no part, and a drain carries no
  grace from such a host.
- **An answer before its calls.** A completion's answer is told as it
  arrives, before any of its calls is scheduled: the session's place
  between the assistant message and its calls' scheduling. A completion
  answered while its session closes, whose calls never run, is told as
  an answer too, and each of its calls then has a result (`not run`).
- **A result as each call ends,** in the order calls end, with the
  result the whole turn will keep: a workspace tool's outcome, text the
  domain wrote, invalid, not run or withdrawn. A call answered before
  its turn's answer was told, such as an invalid call, is told after
  the answer, in its block order.
- **A start for a command or a `job` wait.** As a command, or a `job`
  call that waits for a job's output, starts running, once the tools
  have taken it rather than answered it at their entrance, the run
  tells a result part for it: `interrupted`, its time absent and its
  output empty (protocol/transcript.md, section 2). Its result when it
  ends, `running` included, or a drain's `interrupted` (section 4), is
  told after it and supersedes it. It goes up as any part, and the
  command does not wait for its host to keep it: a crash before the
  host has kept it leaves the call with no part, which section 5 runs
  as never started. No other call tells a start: a check, a `save`, a
  host call and a `sub_agent` call are run again or answered from the
  host's record whether or not they started (section 5), and any other
  workspace tool ends in moments, so one a crash cut with no part runs
  again. A sub-agent's starts are told on its thread, as its results.
- **Every conversation's.** A sub-agent's answers and results are
  told as main's, on its thread, though its whole turns are not; its
  last answer's result in main is the `sub_agent` call's, told on main's
  thread when the child has ended.
- **Main's whole turn as before,** told when its last call has
  answered (run.md, section 4), after its parts.
- **Held like turns.** Parts go up while the run lives, in the order
  told, interleaved with turns, and count with them against the start's
  window of unacknowledged turns and bytes (protocol/channel.md,
  section 7). The host acknowledges them with `part kept`, a number
  covering that part and every one before it. A run with no room waits,
  as for a turn.
- **A host reads only their numbers, sizes and the fields beside the
  body,** never the body, and keeps them until its parent has, then
  acknowledges (host.md, section 6). A host that keeps them durably
  keeps those told since the run's last whole turn it kept, and hands
  them back at the next start (section 5); a whole turn kept makes every
  earlier part of its turn and before it moot.

## 4. A drain at the answer

A drain whose record carries a grace (protocol/channel.md, section 3),
from a host that asked for parts, ends the run at its answer rather than
at its turn's end:

- **No new completion** of any conversation starts once the run has
  heard it, main's and each sub-agent's, a finishing one of a wind-down
  included, as run.md, section 9, has it for main.
- **Completions in flight answer.** Each goes on to its answer, which is
  told as a part (section 3), and its calls start as ever.
- **Then the grace.** Once no completion of any conversation is in
  flight, or at once when none was, the calls in flight have the grace
  to end. Each that ends within it tells its result. A sub-agent that
  ends within it answers its call; one that would ask another completion
  waits, and is cut with the rest.
- **At the grace's end, the cut.** Each call still running is stopped as
  a cancel stops it (run.md, section 9): a command and a `job` wait
  cancelled, a check aborted, a host call withdrawn, a sub-agent's
  conversation closed with its own calls cut the same way. Then:
  - **a command or a `job` wait** cut midway tells a result part,
    `interrupted`: how long it ran, and its output read before the cut
    as a running command's is kept (protocol/transcript.md, section 2);
  - **a check-bearing call** (`deliver`, `finish` with a change), **a
    `save`, a host call and a `sub_agent` call** tell nothing: section 5
    runs them again or answers them from the host's record;
  - **a call never started** tells nothing;
  - **a delivery in flight is never abandoned** (`run-delivery.md`, 1.2): it is not
    cut, and its result is told when its host answers.
- **No whole turn for the cut turn.** The run tells no turn whose calls
  have not all answered: its answer and results so far stay parts, and
  the turn stays open for the next activation.
- **The run answers *failed: cancelled*,** with every turn it told and
  its fence, as run.md, section 9, has a drained run answer.
- **A turn whose calls all end within the grace** is told whole, and the
  run ends as a drain without a grace ends it.
- **The grace is the host's figure,** carried by the drain; a drain
  without one is run.md, section 9's, whole turn and all. A cancel after
  a drain cuts at once, as ever, telling nothing more.

## 5. Resuming a cut turn

A run whose start resumes a transcript and carries parts (section 2's
`Resumed`, in order: by activation, then number) resumes the cut turn
before it asks any completion:

- **Main's answer leads.** The cut turn is the one whose answer is
  main's earliest answer part: its body's place follows the window's
  last turn's (protocol/transcript.md, section 2). Results, and
  sub-agents' parts, are those sharing an activation and turn with
  main's parts of that place. A start whose parts hold no such answer,
  or whose parts do not decode, resumes without them: the turn is
  planned afresh, as before parts.
- **Rebuilt, not asked.** The answer's messages follow the window in
  the session's history, as told; no completion is asked for them.
- **Each call of the answer, in block order, gets its result:**

  | The call | Its result |
  |---|---|
  | one with a result part | its last, by activation then number; a command's start left last, as a crash leaves it, is `interrupted` after an unknown time with no output |
  | a host call, a delivery or a save that the start's answered calls hold (protocol/channel.md, section 3) | that answer, as the call's own result |
  | `deliver` or `finish` with a change, its delivery unanswered | its checks run again on the same tree, given the same results directory (`run-delivery.md`, 1.1), then its delivery asked anew; a `finish` is judged as ever |
  | a `save` or another host call, unanswered | asked anew, under this activation's name for it (run.md, 5.2) |
  | `sub_agent` | its conversation goes on (section 6), and its answer is the call's result |
  | `wait` | answered as a parked run's withheld `wait` is, with the messages that woke the run (run.md, section 6) |
  | any other, with no part | run now: a drain tells a part for each command it cut and a command tells its start, so it never started, or it is a workspace tool a crash cut, run again |

  A host call is asked anew under this activation's name only when no
  host decided it, since a name carries its activation and no later one
  reuses it (run.md, 5.2). A call a host decided in any earlier activation
  is among the start's answered calls under its own name (run.md, section
  6), whether a drain settled its relay or the engine settled it after
  losing the worker, and an engine holding a decided effect it has not
  yet settled places the run again only once it has.
- **The model sees each as if the call had run once:** a rerun's or a
  recorded answer is the call's result, in its place, and an
  `interrupted` result is rendered for the model to judge
  (protocol/llm.md, section 5).
- **The turn ends as any turn,** when its last call has answered: the
  results message holds the results in block order, then the waking
  text and the messages that woke the run (`run-start.md`, 1.4, and `run-messages.md`, 1),
  and the whole turn is told, at the answer's place, as this
  activation's first, its calls keeping the names and positions the
  answer gave them. Its answers the start's answered calls held are not
  told again in the waking prompt.
- **Cut again,** the resumed turn tells its new parts in this
  activation, and the next start carries both activations' parts:
  results told in either count.
- **Its budget.** Calls run again count against this activation's
  budget; the answer's completion was counted where it ran.

## 6. A sub-agent resumed

- **Its parts** are those whose call is the `sub_agent` call's position,
  from its last window part, else from its first answer, in order: its
  history is their messages, then its last answer's results by section
  5's rules, but that a sub-agent has no host call, delivery, `save`,
  `wait` or `sub_agent` of its own (run.md, 5.3).
- **Its opening is its first answer's:** the system text and affinity
  that answer kept, so it is told what it was told at its open. A
  sub-agent with no answer part opens afresh from the call's input, as
  at first.
- **Its share** is what it would get asking now (run.md, 5.3), its turns
  less the answers its parts hold; its time starts again with this
  activation's clocks.
- **Its new parts** carry a new ordinal, this activation's, and the
  same call position, so the next resume finds both activations' parts
  of it by that position.

## 7. The world

- **Telling,** in the run world (`smith/tests/run`): with `tell_parts`,
  an answer part before its calls' first result, a result part per call
  in the order they end, a sub-agent's on its thread with its call's
  position, and main's whole turn after its parts; without it, none.
- **The drain at the answer:** with a grace, a drain while main's
  completion is in flight tells its answer, gives a read call its result
  and ends the run cancelled with no whole turn for the cut turn; while
  a check runs, the check is aborted at the grace's end and no part is
  told for the `deliver`; while a command runs, it is cancelled at the
  grace's end and its `interrupted` part holds the time it ran and its
  output so far; calls that end within the grace tell their results and
  a turn whose calls all end is told whole.
- **Resuming:** from a transcript and the parts above, the run asks no
  completion for the cut turn; the check runs again, its results
  directory the same; the interrupted command's result reaches the
  model as interrupted, with its output so far, and is not run again; a
  host call the start's answered calls hold is not asked; an unanswered
  one is asked once under the new activation's name; a call with no part
  runs; the whole turn is told at the answer's place; parts that do not
  follow the window are ignored.
- **A sub-agent resumed** from its parts asks no completion for an
  answer its parts hold, and its new parts carry the same call
  position.
- **A crash:** a command's start is told after its answer and before
  its result, which follows it, main's and a sub-agent's on its thread;
  a call the tools answer at their entrance tells no start. Resumed from
  the parts a first activation told before it stopped with no drain:
  while its answer was in flight, none for the cut turn, which is
  planned afresh; while a check ran, the answer alone, and the check
  runs again with the same results directory; while a command ran, the
  answer and the command's start, and the model reads it interrupted
  after an unknown time, with no output, and it is not run again. A
  drained command's resume reads its time, its drain's part told after
  its start.

## 8. Open questions

- **What a resumed turn's host calls made:** a host keys what a turn's
  calls made by their names in the turn's activation (jig's
  `domain/engine/executors.md`, 2); a call answered from an earlier
  activation's record names nothing there. Default: so, as a call told
  in a waking prompt names nothing today.
- **Parts for a local host,** which resumes its chats in one process:
  none asked. Default: none until a local host keeps its place across a
  restart.
