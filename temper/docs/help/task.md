# Tasks

Kind: concept

A task is one numbered piece of work in a project, done by an agent, a procedure or a person, within an authority, toward a result of a declared kind; tasks make trees of delegates.

## What it is

- **A task** has a spec (words and inputs), executor (agent under a charter,
  procedure, person or role), authority (what it may do, delegate and spend,
  and by when), and a contract: report, change, verdict or approval.
  `task show` prints these.
- **A person task** prints `taken by:` naming its holder, or `none`.
  Once an answer has ended the task, `answered by:` takes its place.
  A closing person task with no holder prints neither line.
- **Delegates:** an agent's run makes a tree of child tasks (delegates).
  Results reach their parent; its run hears them under its waking rules.
  A change task is a delegate run by the forge, returning a landed
  change (temper help box) or why none applies; it starts with half its
  parent's remaining spend. Questions go `up` to its requester (for a
  goal, the goal's agent, then you), or nowhere (`none`). With `up`, it
  relays producer, repair and resolution questions, holds answers until
  dependencies land, and holds past its question bound.
  To answer an escalation, get its answer number with `read_task` on the
  change, then `message` with `form=answer`. Words and amendment reasons
  leave questions open; a party's words are taken unanswered.
  A change lacking authority to proceed (tasks exhausted, depth
  used, too little spend) holds, saying what it lacks; released unchanged,
  it holds again. For exhausted tasks, grant more with
  `task amend TASK --rev REV --tasks N REASON`, then
  `task release TASK --hold REV`, or cancel it. `N` replaces the lifetime
  allowance; release keeps tasks already made. Widening
  stays within the giver's authority and funding.
  A change runs nothing itself: `task release --again` is refused;
  release its hold for a fresh decision.
- **Its parts:** tree (live delegates), activity (events, newest last),
  turns (model words, calls and results per step), and run (current attempt).
- **Its settings revision:** `task show` prints the task's
  `settings: revision R`, plus the run's own and newest where they differ.
- **Its model choice:** when the task carries an LLM choice, `task show`
  prints `main model: MODEL/EFFORT`, then `sub-agent model: MODEL/EFFORT`
  for each sub-agent choice in order. A model without an effort prints
  just its name.

## How it changes

- **Live phases,** as `task show` prints them, then as a wait names them:
  Waiting (`waiting`), Waiting for news (`idle`), Due to run (`due`),
  Preparing a host (`preparing`), Claimed by a host (`claimed`),
  Running (`running`), Retrying (`backing-off`), Executor active
  (`active`), Closing (`closing`), Held (`held`).
- **Ended:** Completed, Failed, Cancelled, or Ended (`ended` to a wait).
  `task show` prints the result; verdicts name `verdict CODE` with their
  reason, also in requester wakes and `task activity`.
- **An invalid run's end** in `task activity` says why: `run 3 ended:
  its result broke its contract (verdict)`, naming `form`, `verdict`,
  `words`, `change` or `followups`. These mean a different result form,
  an unoffered verdict code, too many word bytes, a different change kind
  or connector, or too many follow-up tasks. `run 3 ended: its result
  couldn't be read` means no contract kind was recorded: the adapter
  could not decode or translate the result. `--json` gives the cause in
  the failed end's optional `invalid_reason`. The cause describes that
  run; retry classification and counters are unchanged.
- A start past a host's limits is refused and frees its reservation.
- **What it waits on** prints after `waiting on:`: `Kept by its
  dependencies`, `Waiting for delegates`, `Waiting for news`, `Nothing
  awaited`, or a party and what it must answer: `you: question 3 of task 44 · 1 waiting`.
  For a held task that awaits nothing else, `waiting on:` prints `held`.
  When no account serves its next run, `next run:` stands alone above the lines
  that say what to do, and `waiting on:` prints `an account`.
  Its header then says `Waiting for an account` for `Due to run`, and `--json`'s
  `waits_on` says `account`; to a wait it is still `due`. Moving a task gives it
  no account: `move` is offered on a task another party requests, and not on your
  own.
- **Held:** a held task runs no more until someone decides. Its hold
  has a reason (out of tries, stopped, its deadline passed, an
  insufficient budget, an effect whose outcome is uncertain, among
  others) and a revision of its own; a hold a party placed is theirs to
  release (temper help hold). Releasing it runs it again;
  leaving it held, or passing it up to the next holder of authority,
  are the other decisions. `task show` prints `held:` and its reason, then `way out:`, what to mend and the call that releases it: `task release TASK --hold REV`. An effect's hold names the effect, its operation,
  resource and outcome; a procedure's names the step refused and why,
  in its connector's words. Only an uncertain effect asks whether to
  repeat it: a release with `--again`, once you have checked whether it happened.
- **Awaiting a person:** an agent whose instructions covered a waiting
  decision and that parks leaves it waiting. That decision does not wake
  it again until it changes or something else wakes the agent, when the
  decision is offered again. A run with no instructions for the decision,
  or one that refused it, is woken again. The held task and decision stay,
  and the decision's stall time keeps running.
- **Stopped:** stopping places a hold of yours over the task and every
  task below it, those made later included, and ends their runs;
  releasing the stopped task runs them all again (temper help hold).
- **Cancelled:** cancelling ends the task and its tree: at once, or
  after a grace the canceller gives, in which its run may finish. A
  task that has ended refuses a cancel, saying how it ended; the same
  cancel again, under its key, gets its first answer. A change's early
  start may end its producer `Cancelled · its early start conflicts with
  SHA`: a dependency's head did not merge. The change goes on with a new
  producer (temper help change).
- **Amended:** an amendment replaces the controls it names (priority
  and tracking, deadline, instructions, spec, dependencies, model) and
  keeps the rest; all but a tracked task's priority name the spec
  revision the amender saw. Words a read cut short are replaced whole
  at that revision.

## What temper guarantees

- **One result.** A task ends once, with one result of its contract's
  kind, or failed or cancelled.
- **Words between turns:** a message reaches the agent at a turn's
  boundary and steers its next turn.
- **Decisions on what was seen:** a release names the hold's
  revision and an amendment the spec's; a moved one is refused, and a
  read shows the new one.
- **A task's runs keep their settings revision** until it is moved to
  the newest (temper help settings).
- **Spend counts up the tree:** a delegate's budget is reserved from
  its parent's (temper help budget).

## Acting on it

- **Reading:** `task list` lists a project's tasks; `task show` reads
  one; `task tree`, `task activity`, `task turn` and `task run` read its parts; `task show`,
  `task tree` and `task run` wait with `--until` or `--since`.
- **Acting:** `task show`'s `acts:` include
  `task message`, `task answer`, `task amend`, `task stop`, `task cancel`, `task release`, `hold lift`, `task decide` and `proposal decide`.
  `task amend`'s line is its call alone; `--json` carries the bounds it is offered within.
- **temper's runs** act on their delegates with the host tools
  `delegate`, `change`, `goal`, `decided`, `amend`, `message`, `cancel`, `release`,
  `subscribe`, `introduce`, `decide_escalation` and `propose`.
  `read_task`: specs and retained results of any task below through a proven same-project requester chain, and tasks its requester waits on; `field=result` reads reports and Left lists, including changes and reviews.
  Verdicts print `Verdict: CODE`; offsets count reason bytes.
  `from=next` continues; restart revised specs. Durable question numbers and
  answerers survive delivery: use `message` to the printed target with
  `form=answer` and that `question` number.
  Only runs with authority for the message family get `message`: a change's
  coder, repairer and resolver where it relays their questions; not its reviewer.

An “invalid results” hold may mean an assignment was refused before any model turn.
For the precise refusal reason or failed-run detail, read serve's stderr or
`DATA/box/logs/serve.log`.

## Limits

Restart settings: `temper help configuration`.

- temper does not let ctl take a task, hand it back, move it to
  another party, reorder it or mark it seen: those acts print `(no tool)`.
- temper does not interrupt a turn in progress for a message, an
  amendment or a priority.
- A task's deadline is checked when a run starts, not while it runs
  (temper help budget).
- `task activity` and `task list` read 32 lines a page; a turn's part prints
  65536 bytes at a time.

An overview may cut `asked:`, `instructions:` and its report. Follow
`read specification:`, `read instructions:` or `read result:`;
JSON and `task activity` give them under `text_reads`.
`task text TASK --field result` reads the full report; follow
continuations until `no more`, including after it ends.
A cut page of `task text`, or of `task turn --part`, names its page, of how
many, and the call that reads on in its first line as well as its last.
`task text --json` gives `task`, `field`, `from`, `total`, `text` and `next`;
null `next` ends the read. Printed offsets keep UTF-8 characters whole.
After an amendment or a settings revision change, restart at zero.
