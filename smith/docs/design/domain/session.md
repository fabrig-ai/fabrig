# Sessions

Provisional, 2026-10-05, revised 2026-10-20. What a smith session is, as
a domain layer: one conversation with an LLM, opened by a run. It is the
child domain `smith-domain-session`, whose own child is the tools domain
(tools.md). What is still open is listed in section 11.

## 1. In one page

- **A session is a conversation, only.** It knows the provider-neutral
  vocabulary of a conversation and nothing of runs, charters, results or
  hosts.
- **Turn by turn.** It calls the LLM, runs the tools the LLM asks for,
  sends their results back, and repeats until the LLM yields, a limit
  ends it, or its opener closes it.
- **Each turn is told** to its opener as it ends, with every name
  resolved, so that a turn means the same outside the session that made
  it: the main session's turns are its run's transcript.
- **One prefix per window.** Within a window the prompt only grows: the
  model, the affinity, the system text and the tools stay as they were,
  and history is appended to, never edited. Near the model's context
  window the session compacts, and a summary opens a new window.
- **Opened from a transcript,** a session goes on where an earlier one
  stopped, from its last window, with the same affinity. Whether it is
  offered a window from another endpoint or backend is its opener's
  decision (run-messages.md, section 1). A window that ran on another endpoint,
  dialect, account or backend is carried over: compacted on the
  charter's LLM, and a new window opened (`session-context.md`, 1.2).
- **Provider-neutral.** Nothing in a session depends on which provider
  answers; each provider's API is skein's shared LLM client's.
- **Bounded:** turns, tokens, spend, time and bytes held, each a budget
  its opener gives it; and each prompt by its model's window.

## 2. A session

- **Turn by turn.** A session that yields has not ended: its opener
  continues it with a new message, or closes it. Calls in the answer it
  yielded with are answered as not run when it continues.
- **A conversation's affinity.** Its opener gives each session an
  affinity: the run tree's identity as its key, and the conversation's
  thread, its ordinal in the run's activation, main's being zero (run-start.md, section 1.2, and run.md, section 5.3
  ). The session puts it unchanged into every prompt. skein's
  dialect renders it, or ignores it where the provider has nothing to
  render it as (skein's llm/README.md). It is fixed for the conversation's life:
  across its windows (`session-context.md`, section 1), and across activations through its
  transcript, which records it (`session.md`, section 3). A session opened from a
  transcript keeps the affinity the transcript records.
- **Owned and delegated tools.** A session runs the tools it owns
  through its tools child domain, and delegates the rest to its opener
  (`finish`, `deliver`, `wait`, sub-agents and host tools, run.md,
  section 5) or, later, out of the domain (MCP calls, as opaque
  payloads). To the session they differ only in who answers: a delegated
  call is a ticket and an effect, and its answer is the result's bytes.
- **Everything in flight can be stopped.** Every tool call has a
  deadline, and an opener can abort its session, which cancels what is
  in flight and waits for it to settle (programming-model.md, 5.3).
- **No chains within a step.** An answer that comes back in the step that
  asked for it (a call the tools refuse at their entrance, one the opener
  answers at once) waits on the ready list (programming-model.md,
  section 2), so what one step emits and holds stays bounded.

## 3. Turns and transcripts

- **What a turn is:** one completion, its surrounding user messages, the
  provider's blocks in position, the calls with their provider ids,
  names and input as written, their results as concrete bytes, and what
  the completion used and cost. A withdrawn call has a result of its own
  kind, and a call cut off at the output limit is kept as a note of it,
  with its error result (section 4). A historical call holds no ticket
  that means something only inside the session that made it.
- **Supplied system text** is retained verbatim on the first and each window
  opening turn, separately from replay messages. It includes the run's
  selected instructions and fitted brief. Older records without it keep
  that context unavailable; a reader never rebuilds it from settings.
- **A turn has a kind:** an ordinary turn, as above, or a window turn,
  the compaction that closed one window and opened the next, holding the
  new window's opening as its messages (`session-context.md`, section 1.2).
- **Told as it ends.** Closing waits for actual terminal answers before
  telling the last turn, including answers that win a cancellation. The
  one exception is a delegated call whose answer its opener withholds
  and closes the session without giving, as a run parks with its `wait`
  withheld: the last turn is told with that call unanswered, and the
  opener answers it when it opens the conversation again (run-messages.md,
  section 1). A turn's messages are those new since the turn before, so
  each message is told once and an ordinary turn's prompt is its
  transcript up to it. A trace records prompts the same way
  (protocol/events.md).
- **The stable prefix** is a contract, so that a provider can serve each
  prompt's prefix from its cache:
  - Within a window, the model, the endpoint's options, the affinity, the
    system text and the tools offered are fixed. The reasoning effort is
    the conversation's, from its charter, and a later activation may ask
    for another without a window boundary, since no block depends on it.
    History is append-only: each completion's prompt is the one before
    it, then that completion's blocks, then what was appended after them.
  - The system text and the tools offered are fixed for the
    conversation's life. A session never changes them; a run that resumes
    the conversation composes them again from its charter (run-start.md,
    section 1.3). A completion's tool choice narrows what the model may
    call (`session.md`, section 5) and changes no definition.
  - Dynamic guidance is appended as messages, never placed in the system
    text or a tool's description: budget and threshold notes, the
    wind-down and final-turn notes, and messages passed on between
    completions (run-messages.md, section 1 and budget.md).
  - A window boundary is the only exception (`session-context.md`, section 1.2). There, history
    is replaced whole; the system text, the tools and the affinity stay.
  - A prompt's encoding is byte-deterministic, so an unchanged prefix is
    unchanged bytes (protocol/llm.md, section 2.2).
- **Calls per response.** A completion's block bound is the declared
  calls per response; skein refuses a completion with more as a typed
  limit (section 4). A turn's messages are bounded by the transcript's
  blocks per message, derived from the same declared number and never
  below it (protocol/limits.md), so every completion the session accepts
  can be told.
- **Versioned.** Turns and transcripts are domain values with a version;
  their bytes are the protocol layer's (`smith-transcript`), so a host
  can keep them opaque and a later agent can resume them or know it
  cannot. A transcript records its conversation's affinity once, in its
  opening turn (protocol/transcript.md, section 3).
- **Opening from a transcript.** A session may open with earlier turns:
  its conversation's current window, from the conversation's first turn
  or from its last window turn, as they were told, including the
  provider's opaque blocks, which go back to the same provider and
  endpoint verbatim. History is checked before a completion starts:
  version, and one endpoint and dialect; contiguous turns; the affinity,
  account and backend in the
  opening turn; message and block bounds; roles and call and result ids;
  and no unresolved ticket. A window that ran on another endpoint,
  dialect, account or backend than the charter's main LLM is carried
  over, not refused (`session-context.md`, 1.2). It must fit the session's byte limits with
  the waking prompt and room for a completion. A history that fits them
  but fails the window check (`session-context.md`, section 1.1) is compacted before its first
  ordinary completion. An unanswered call in the tail gets the same
  not-run result as an ordinary continuation, unless the opener answers
  it as the session opens, as a run answers the `wait` it withheld when
  it parked; answers the history does not hold go to the LLM as text in
  the waking prompt (run-messages.md, section 1). The opener names in its opening
  the one call it may have withheld, by its tool's name, and the session
  restores that call alone as withheld: it knows none of its opener's
  tools by name.
- **Refused transcripts.** Another version, a malformed, unresolved or
  oversized history are distinct refusals; the run treats every one as
  a transient failure (run-messages.md, section 1).

## 4. Providers and retries

- **Any provider.** A session talks to the endpoint and model it was
  opened with; skein's shared client speaks each provider's API (HTTP,
  server-sent events, JSON) and its errors, and the protocol layer
  translates the session's vocabulary and tool schemas to it.
- **Retries are policy.** A session classifies a failed call
  (overloaded, rate limited, unavailable, timed out, context too long,
  invalid, unauthorised, an account exhausted or refused, a request
  unsupported, a local limit), retries the transient ones after a
  jittered exponential backoff, and gives up when its retries run out.
  A rate limit is retried too, but spends none of them (below).
  An exhausted account, a refused one and an unsupported request are
  never retried: the same request on the same account would meet them
  again, and the agent never moves a run to another account
  (protocol/programs.md, section 3). The protocol layer runs the attempt and its
  deadlines. A provider's failure carries its HTTP status where it had
  one.
- **Timeouts carry their phase, and are retried by it:** connecting, the
  handshake, the response's head, idleness between events, or the whole
  call.
  - The head and idle deadlines are the model's, which its opener gives
    with it. The whole call's bound is the session's remaining time.
    There is no per-attempt ceiling: a stream that keeps making progress
    is cut only when the session's time runs out.
  - A timeout in any phase but the whole call is transient, and retried
    like any transient failure. A whole-call expiry is not retried: the
    session's time is up (section 6).
- **A full pool means waiting, not failure.** A call that finds the LLM
  pool at capacity, in connections or in memory, waits below the session
  for room, within its deadline. It is not a failed call and spends none
  of the session's retries.
- **A rate limit means waiting, not a spent retry.** A call failed *rate
  limited*, whatever its HTTP status (a refusal inside a stream whose
  head said 200 included), is made again after its wait, and spends
  none of the session's retries: they count the other transient
  failures alone, so a run that meets one limit after another is never
  failed for them.
  - **Its wait** is the backoff its attempt would take, and at least
    the delay its provider gave: skein's `RateLimited { retry_after }`,
    the head's `retry-after`, the error's reset, its message's `try
    again in` figure, else its limit's window (skein's
    `llm/rate-limits.md`). A delay of zero leaves the ordinary backoff,
    capped at `backoff_max`.
  - **Each try is an attempt.** The attempt number goes on rising, and
    a session that later fails for another class counts every attempt
    it made, rate limited ones included.
  - **The host hears of it.** While the session waits, its root tells
    its host of the wait as the completion's long operation,
    `completion`, its span ending at the retry, and says it done as the
    retry is asked or the session ends: so the host's watchdog stretches
    over a wait longer than its no-progress deadline (host.md, section
    4; protocol/channel.md, section 4). The run's budgets keep counting.
  - **A wait past the session's time,** a delay that would end at or
    after its expiry, is not taken on a call with an account: the
    session ends as an exhausted account's does, its root raising
    `limited` for the call's account with that delay as its reset, and
    the run ends limited for it (run.md, section 9), so its task waits
    for the account and spends no try (jig's `domain/accounts-sources.md`,
    2). With no account to name, the session waits as for any delay,
    and its expiry ends it (`session.md`, section 6).
- **A local limit is typed:** `Limit { which, bound }`, naming the limit
  that fired and its bound. It is not retried, since the same request
  would meet it again.
- **A request too large for skein's request bound** does not follow a
  passed admission. The session's own bounds on a window's messages,
  blocks and bytes are derived within skein's request and history
  bounds, with the system text and the tools at their largest, and are
  checked before each completion with the window (`session-context.md`, 1.1), so a history
  that would pass skein's is compacted first (../protocol/limits-derivation.md, section
  1.2). Should one fire anyway, a fault in that derivation, it is taken
  as *context too long*: one compaction, and a second in the same window
  fails the session.
- **A request past the line,** which the guard refuses before it is
  sent (`session-context.md`, 1.1), is taken as *context too long*, unsent: one compaction, and
  a second in the same window fails the session.
- **Context too long** compacts the window once; a second in the same
  window fails the session (`session-context.md`, section 1.2).
- **A call cut off at the output limit,** which skein marks, never enters
  history as written: its input is incomplete, and a provider would
  refuse it on replay. The session replaces it in history with a note of
  it (its provider id, its tool and how many bytes of input it had) and
  answers it with an error result, the invalid call's problem *cut off*
  (protocol/llm.md, section 4), so the model can make it again with
  less. The session then goes on as after any answered call.
- **An oversized reasoning item** fails the completion with a limit
  naming the reasoning item. Dropping it instead is an explicit opt-in
  of the agent's configuration: the completion is kept without it, so
  the item never enters history, and the drop is reported as a fact.
- **The last failure's detail is the operator's.** The session keeps
  the detail of the last failure it met (the provider's text, clipped,
  or the limit and its bound) within a bound its limits give, each later
  failure replacing it. It goes with the session's end to its opener,
  and on to the agent's log and, as the capture policy allows, its trace
  (protocol/events.md). It never enters the model's history.
- **Credentials** are the protocol layer's. An unauthorised credential,
  an exhausted account and a refused one are each a notice the session
  raises, which the agent sends to its host as unauthorized, limited and
  refused (host.md, section 7). An unauthorised failure is transient when
  a refreshed credential may answer the retry: its call's grant was not
  a key, or a newer generation is already held. An actual unauthorised
  response on a key with none newer fails at once, after its notice,
  with the same class, evidence, report and account: no host renews a key
  (protocol/channel.md, section 6). The call retains its issued grant's
  renewal policy even if a later generation arrives while it is in
  flight. A missing or lapsed grant that fails unsent keeps its recovery
  retries. The others end the session as a failure of its model, naming
  the account.
- **A completion waits for a grant that will come** (temper's
  `draft/redeploys.md`, 5.3). While its host says its renewals are held
  (host.md, section 7: its link to what grants them is lost, as when
  its engine is away), a completion whose account's grant has lapsed,
  or is missing, and is renewable (a bearer's or a program source's,
  `Grant.renewable`) is not issued and fails nothing: the session waits,
  its reservation released and no retry spent, for the next grant of
  that account (`Event::Grant`). A newer generation that is valid
  issues it; one no newer, or lapsed, once the host has said its
  renewals resume, fails it unsent as before, keeping its recovery
  retries: so a source that must be signed in again fails as it does
  now. It waits within the conversation's expiry and the run's
  deadline, past which those deadlines end it; the run's budgets keep
  counting. The agent tells its host of the wait as a long operation,
  `grant`, bounded by those deadlines, and of its end as any long done
  (protocol/channel.md, section 4), so the host's watchdog knows of it
  (host.md, section 4). With renewals not held, and for a key, which no
  host renews, a lapsed or missing grant fails unsent as before; a
  completion already issued runs on under its provider's deadlines. A model with no
  account raises none of them: no grant can come for it, so each ends
  the session as a failure of its model, with its provider's failure
  class and no account (protocol/charter.md, section 2). An unsupported
  request raises no notice: it names the field, and fails the session
  (protocol/llm.md, section 6).

## 5. Tools in a session

- **One tools entrance is dispatched before the next batch starts.**
  `tools_requests_dispatched` requires every request made by the tools
  entrance to be handed on before another batch starts, including
  cancellations and job polls. The session world holds this under load in
  `smith-session-world`, `orders::tools_requests_dispatched_before_the_next_batch`.
- **A successor batch starts after reclaim.**
  `batch_promoted_after_reclaim` makes an immediately answered batch's
  successor ready in the next iteration; a close at that boundary
  prevents the unstarted tail from running. The session world holds this
  in `smith-session-world`,
  `orders::immediate_batches_are_promoted_after_reclaim_before_close`.
- **Batch admission preserves effects and result order.**
  `batch_accepts(effect)` admits adjacent reads together and a write
  alone; `batch_settled` requires every call's terminal before the next
  batch starts, with owned and delegated results kept in call order.
  The session world reverses read terminals under load in
  `smith-session-world`,
  `orders::reverse_read_terminals_preserve_write_isolation_and_result_order`.

- **Tools run in parallel where that is safe.** Each tool has an effect,
  read or write. Adjacent reads in one turn run together, up to
  `parallel_tools` at once, or `calls_per_response` where that is
  smaller (protocol/limits.md, 2.3); the reads past it make the next
  batch, which starts once every call of the one before has answered. A
  write runs alone, and results go back in call order. A sub-agent's
  effect follows from its tools; a host tool's is as its charter
  declares; `finish` and `deliver` are writes; `job` is a read, and a
  write with `stop` (tools.md, section 5).
- **A job's wait gives way to the rest of its response.** A `job` call
  waits as it asks only in a response whose calls are all `job` calls.
  Beside any other call, it waits for its job's end no longer than the
  other calls of its batch run: once every call of its batch that is not
  a `job` call has answered, it answers at once, `running` if its job
  still runs; in a batch of `job` calls only, as after a write or a
  command, it answers as soon as it starts (tools.md, section 5). A poll
  beside other work costs at most that work's time, never its own wait,
  whatever the work's effect, so the session never needs to know what a
  command changes.
- **A failed write stops the writes after it.** When a write of a
  completion fails, the completion's later writes and commands are
  answered as not run, naming that call; its later reads still run. So
  a change made in several calls of one completion is never checked
  half made.
  - **A write fails** when its change is not known to be made whole:
    - **a call answered `invalid`,** whatever its tool and its effect:
      its arguments did not decode, or the tools refused them, so what it
      would have done is not known;
    - **a `write` or an `edit` answered anything but `written` or
      `edited`,** by protocol/events.md, 3.6's verdicts: a path refused,
      a version not read or stale, an `old` that matched nothing or more
      than once, edits that together leave the file as it was
      (`unchanged`: like every failed edit, it applies none of the
      call's edits, tools.md, section 3), a size
      refused, `failed`, which may have stored part of its
      change, and every other.
  - **Nothing else stops them:** a `shell` or a `job` call's answer,
    whatever it is but `invalid`, an exit that is not zero and
    `timed_out` included; a host tool's `error`; a sub-agent's answer; a
    call not run or withdrawn.
- **Jobs run beside the turns.** A `shell` call stays a write until it
  answers, `running` or ended. A job that is running holds no place in
  any later batch: the session's later reads, writes and commands run by
  these rules while it runs, and its completions go on.
  - **They are the session's writes in progress.** Its opener cancels
    them, and waits for them to settle, before another writer takes the
    lease or a check or a delivery runs (run.md, section 5.3, and run-delivery.md, section 1.1).
  - **They end with the session.** A session closing, for any reason,
    cancels its running jobs and waits for them to settle, then tells a
    `job.ended` for each job whose end no answer reported, before it
    ends (tools.md, section 5).
  - **None outlives its activation.** A session resumed in a later
    activation has no jobs.
- **Tool choice.** Each completion is asked with a tool choice: any tool
  offered (`Auto`), none (`None`), or only some (`Only(names)`). The
  tools offered never change (section 3); the choice narrows what may be
  called.
  - The opener gives the choice when it continues the session, as a run
    winding down offers `finish`, granted `deliver` and `wait`, and host tools
    marked for the reserve (budget.md), never `save` (run-delivery.md, 1.5).
    The session sets `None` on its own final turn (`session.md`, section 7), and on a
    compaction (`session-context.md`, 1.2), whatever the conversation's choice.
  - A call outside the choice, one a dialect that cannot express `Only`
    let through or one the model wrote anyway, is answered as not run.
    The session then goes on by its ordinary rules.
- **A call too large to take** arrives as an oversize block with its
  size. The session answers it as an invalid call the model can fix,
  never as a failed completion (protocol/llm.md, section 4).
- **The workspace is shared, knowledge is not** (tools.md, section 4):
  what a session's LLM has read, and at which version, is its own state.

## 6. Budgets and prices

- **Budgets:** turns, tokens (input, output, cache reads and writes, as
  the provider counts them), spend in the host's unit, and time, given by
  the opener at open; bytes held, against the agent's limits. Crossing
  a turn, token or byte budget stops the next completion, not the turn
  in flight: the calls of the completion that crossed it still run and
  settle. For spend, the session reserves each completion's maximum first
  (budget.md), so spend never crosses it. Time is the exception:
  when it runs out, the session closes at once. A compaction is a
  completion like any other: it counts as a turn, and is reserved,
  charged and counted against these budgets (`session-context.md`, section 1.2). A job
  charges no turn, token or spend; its session's time runs on while it
  runs, as it does while any call runs, and a session that closes as its
  time runs out cancels its jobs with its other calls (`session.md`, section 5).
- **Context is a dimension of its own.** The window bounds one prompt
  (`session-context.md`, section 1.1); the token budgets bound what every completion used,
  summed. Neither stands for the other: a session well within its token
  budgets may have to compact, and one far from its window may cross a
  budget.
- **Usage** is what the provider counted: input, output, cache reads,
  cache writes and reasoning tokens. Each count is optional, and *not
  reported* is distinct from zero. Input, cache reads and cache writes
  are disjoint, as skein reports them: input is what was neither read
  from the cache nor written to it, so the three sum to the prompt's
  size. A token budget counts what was reported. Reasoning tokens are
  counted within output; they are reported for what they are, never
  charged again. A completion that
  reports no input or no output count is charged the reservation made
  for it, so spend is never under-counted (section 11).
- **Prices** are integer input, cached, cache-write and output amounts
  per positive `unit` tokens. New input uses the input rate, cache reads
  the cached rate, and cache writes the cache-write rate, which is the
  input rate when the charter gives none. The combined rational charge
  of **each completion** is rounded upwards once, using checked
  arithmetic; the result and the cumulative spend must fit `u64`.
  Overflow is a typed failure ending the session with a report of it,
  never a saturated charge: the charge that cannot be added is not, and
  the spend reported is what was charged before it.
- **Spend is cumulative** within an activation, sub-agents included. A
  delegated terminal, answered or withdrawn, carries the sub-agent's
  cumulative spend once under its call's identity; a duplicate or stale
  delivery cannot charge it again. Restoring history does not charge old
  activations again. A run counts each child once, through the parent's
  terminal answer, never adding both. Enforcing one aggregate budget
  across concurrently open sessions is the run's (budget.md).

## 7. Sub-agents

A sub-agent is a session its run opens to answer a served call (run.md,
5.3). As a session it is like any other: its own tools, budget and model.
Its turns are not the run's transcript; they are summarised in the call
that asked for it, whose result is the child's final message.

- **The final turn.** A child's opener gives it, at open, a final-turn
  note and a wrap-up allowance of time (run.md, section 5.3). When the
  child's own turns are down to one, or its remaining time to that
  allowance, its next completion is its final turn: the session appends
  the note and asks with `ToolChoice::None`. Calls the model writes
  anyway are answered as not run. The session then yields that answer as
  its last, its budget spent, and the run gives it to the parent with
  `stop: budget` (run.md, section 5.3), so a capped child answers instead
  of ending unanswered.
- **A compaction due near the end.** The final-turn trigger counts a
  compaction that is due:
  - by turns, a child whose window needs compacting compacts with two
    turns left and answers with its last;
  - by time, the final sequence starts at twice the allowance, so the
    compaction and the final turn both fit what is left.
- **Its answer is bounded,** and its system text states the bound
  (run-start.md, section 1.3).
- **A main session has no final turn of its own.** Its run's wind-down
  gives it its last completions, with a narrowed tool choice (budget.md
  ).

## 8. Context management

Moved to [`session-context.md`](session-context.md), section 1.

## 9. Below the domain

Requests enter uncapped queues, without an output allowance for one
entrance. Facts are lossless in an uncapped queue; `pending_facts()` gives
the parent its drain count. The session gathers its tools' facts by the
tools' `pending_facts()` count after each child step.

- **LLM providers:** skein's shared client (`skein-llm`) for each
  provider's API and its failures, rendering the affinity and the tool
  choice in each dialect; smith's protocol layer for tool schemas and
  decoding; credentials lent by the host (host.md, section 7).
- **Turns' encoding** (`smith-transcript`): a turn's bytes, with a
  version, every provider's opaque blocks kept verbatim with the dialect
  and endpoint they came from, and a transcript's affinity, account and
  backend, once.
- **Limits** derived from declared quantities (protocol/limits.md): the
  window's threshold and headroom, the history's bounds, the
  completion's block bound and the failure detail's.

## 10. The world

skein's fake LLM provider, whose completions a script draws (tool calls,
malformed input, calls cut off at the output limit, oversized reasoning
items, usage reported and not, every class of failure with each timeout
phase, streaming), the tools domain with the machine's faces, and a
scripted opener that continues, closes, aborts, narrows the tool choice
and answers delegated calls. Windows are small, so a conversation
compacts within a few turns and long stories stay within the suites'
budgets.

- **Its stories:**
  - a conversation that yields and is continued;
  - reads in parallel and a write alone; more adjacent reads than
    `parallel_tools`, the rest run in the next batch;
  - edits of several files and a command in one completion, run in call
    order; an edit among them that does not match, the writes and the
    command after it answered as not run, naming it by its position and
    its tool, and a read after it run; the same after an edit answered
    `unchanged` and after a read answered `invalid`; a command that exits
    non-zero, the writes after it run;
  - a command answered `running` while the session reads, edits and
    completes, then read to its end with `job`; a `job` call with the
    default wait beside reads, answered `running` once they have
    answered; a `job` call before an edit and one after a command, in one
    response, each answered at once; a `job` call alone, and `job` calls
    alone in their response, each waiting for its job's end; a `job`
    with `stop` alone in its batch; jobs cancelled by the opener before another
    writer, `cancelled` at their next read; a session closing with a job
    running, telling its `job.ended`;
  - every failure class retried or given up; head and idle timeouts
    retried, a whole-call expiry not; a full pool waited for;
  - a call cut off at the output limit, answered and made again; an
    oversized reasoning item failing, and dropped where opted in;
  - a budget crossed mid-turn, and one that leaves no room for a
    compaction;
  - a session opened from a transcript, and each refusal of one; one
    opened from a window turn; one whose tail holds a call its opener
    withheld at closing, answered by the opener as it opens; a window
    carried over from another dialect, account and backend, compacted on
    the charter's LLM with no opaque block sent;
  - an exhausted account, a refused one and an unsupported request,
    each failing at once with no retry, and an unauthorised credential
    answered by a refreshed grant; an unauthorised key failing at once,
    except when a newer generation is already held;
  - a conversation compacted at the threshold, and after a context too
    long; context too long on the compaction, failing; calls in a
    compaction's answer not run; a request skein finds too large, taken
    once as context too long;
  - the appended count at the window's measured ratio: a window whose
    prompts the provider counts at several bytes a token, then a turn
    whose results it counts at one byte a token, admitted by the check
    and refused unsent by the guard, then compacted within its window;
    a turn whose new part measures just within the line, sent; a
    completion of the most output just within the line, then its
    compaction fitting; a ratio below one byte a token, or above
    `TOKEN_BYTES`, taken at that bound;
  - a compaction asked with `None` and the window's tools; one in a
    conversation narrowed to `Only`, whose new window asks with `Only`
    again; a child's final turn falling due during a compaction;
  - a summary of several text blocks, opening its window after the task
    and the preamble, separated by blank lines; a summary cut at the
    output limit, opening its window as written; a refused compaction,
    and ones with no text, cut and not, each failing with its fault and
    telling no turn;
  - a wind-down offering only some tools, with a call outside them; a
    child's final turn answered with `stop: budget`;
  - a sub-agent's spend counted once; cache writes priced at their own
    rate; usage not reported, told apart from zero; a prompt's size
    taken as the sum of its disjoint input and cache counts.
- **Memory:** fixed scenarios measure one total peak each, including
  ownership handed to the parent until it releases it, under a ceiling
  with headroom (programming-model.md, 6.3). The recorded turn's `Turn`
  and `Kind` cells have one exact type-size contract check,
  `TURN_KIND_SIZE`, 192 bytes together on x86_64, for every recorded
  turn. It uses neither an allocator peak nor a scenario ceiling;
  a type's size never moves a memory ceiling.
- **Its referee:**
  - every call answered once, in call order;
  - turns told in order, each after its calls settled;
  - budgets exceeded by at most one completion;
  - every prompt of a conversation carrying its affinity, and distinct
    conversations distinct ones;
  - within a window, each prompt extending the one before; one window
    boundary per compaction; every compaction asked with `None`;
  - every job's end told once, and no job running once its session has
    closed;
  - no prompt whose bound exceeds its window, and no prompt, a
    compaction's included, whose size as the provider reports it does;
  - no ordinary completion made whose check, as `session-context.md`, 1.1 counts it, passes
    the threshold, and none sent whose reported prompt passes the line;
  - no failure detail in a prompt.
- **The provider counts tokens from bytes.** Where a story or the fuzzy
  suite reports usage, the world's LLM effect gives each block a
  rendered size, drawn at least its counted bytes, and counts its tokens
  at a ratio it draws between one byte of that size a token and
  `TOKEN_BYTES`, so that the appended count's error reaches its bounds
  both ways. It applies the guard as the protocol layer does, from that
  rendered size (`session-context.md`, 1.1).
- **The fuzzy suite** draws usage, appended sizes, rendered sizes and
  those ratios, and checks that no request's bound or reported size
  exceeds its window, that none sent passes the line, and that no
  session ends for its history's length.

## 11. Open questions

- **Transcripts across providers:** a window is carried over by
  compaction (`session-context.md`, 1.2), losing its opaque blocks; whether a later version
  keeps a provider-neutral form of them, so that less is lost.
- **A window's contents:** the task, the summary preamble, the summary
  and the held-back messages, as `session-context.md`, section 1.2 has them; whether the last
  results should also be kept is for the history probe to measure.
- **Which text makes the summary:** every text block of the compaction's
  answer, as `session-context.md`, section 1.2 has it, or only those a dialect marks as the
  answer rather than as working notes, where it marks them.
- **A job's end between completions:** the model learns a job's end only
  from a `job` call (section 5); whether the session appends a note of
  it, by the path messages mid-work take (run-messages.md, section 1).
- **An unreported input or output count:** section 6 charges the
  reservation in its place. To confirm against each dialect's reporting.
- **Calls past the completion's bound:** failing the completion, as
  section 3 has it, or skein skipping them so that the session answers
  them as not run.
- **Routing state per turn:** if a probe shows that a provider's
  per-turn routing token is worth adopting, the session carries it
  within a turn and never keeps it in a transcript.
- **Provider-native compaction:** a dialect's own compaction items,
  replayed verbatim, as a later optimisation of `session-context.md`, section 1.2.
