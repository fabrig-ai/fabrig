# The run: messages, waiting, parking, resuming

## 1. Messages, waiting, parking, resuming

- **Messages in.** A host relays messages to a live run as they arrive,
  each named, in order, with a sender's label and its text, which the
  host rendered; a start may carry the first (`run-start.md`, 1.2). smith interprets none
  of them: a person's words, a delegate's result and a decision are all
  text to the LLM. The run admits each at its entrance or refuses it
  there, saying why: too large (its label and text as they will be
  rendered, against the run's message limits), the inbox full, a name
  still in use, or the run ending, its answer decided. Each refusal goes
  to its host, typed, with the message's name and the reason, and is
  that message's terminal (host.md, section 2). Host messages reach
  main only; sub-agents never receive them.
- **Host-requested finish.** Each message carries `finish_requested`, false
  for ordinary words. Only admission of a true message puts Open into the
  existing wind-down (`budget.md`, 1), waking Listening or Waiting. Start-carried and
  preparation-time requests retain this effect until main can act. The next
  successful text reply publishes its fence and parks; unread words retain
  their existing terminal. Refusals change no policy, and repeats never renew
  the allowance. Finite ignores the bit and processes an ordinary amendment,
  retaining its terminal tools. Cancel and all hard limits remain unchanged.
- **Offered in order.** Admitted messages queue in arrival order. The run
  offers them at main's opening (`run-start.md`, 1.4), with a `wait`'s result, and
  whenever main yields: every message queued, in order, at most one
  offer's worth, a number of messages and of their rendered bytes that
  the run's limits give, sized so that a window's opening can hold an
  offer (../protocol/limits.md). What an offer leaves stays queued, in
  order, for the next. A compaction carries the messages offered and not
  yet read into its new window (session-context.md, section 1).
- **The fence.** With each turn, and when it waits, the run names the last
  message read: its fence. Messages go in order, so one name covers every
  message before it. A message is read when a told turn holds it, so the
  transcript the host keeps has what was read.
- **One terminal each.** Every message the run accepts ends exactly once:
  - **read:** a told turn's fence names it or a later message;
  - **refused:** at the entrance, as above;
  - **unread:** the run answered without reading it, whether it was still
    queued or offered to a turn that was never told.

  The answer carries the final fence (`run.md`, section 9), so a host in any
  placement knows which went unread; what it does with them is its policy
  (host.md, section 5). No message is dropped in silence.
- **A finish crossing a message.** In a run whose charter grants `wait`,
  a `finish` while messages are queued is not accepted: the run answers
  the call saying that messages came, and gives the model the messages
  in the same request, so it decides with them in view. In a run without
  `wait`, a finish is judged as it comes, and what was queued goes
  unread.
- **A cancel is not a message.** It is the host's, and ends the run
  (`run.md`, section 9).
- **Waiting.** A run whose work depends on what comes next (a chat on its
  person, a coordinator on its delegates) calls `wait`. With messages
  queued, the call is answered at once, with them. Otherwise its result
  is withheld until a message arrives, and the result and the messages
  then go to the model in one request, so an exchange costs one
  completion. Meanwhile the run tells its host it is waiting, naming its
  fence, and its host holds it, the watchdog paused; the run is idle only
  when that fence covers every message its host sent (host.md, section
  4). Time spent waiting counts against wall time (`budget.md`).
- **Open text replies.** Main ending a turn with nonempty assistant text
  under an EndTurn stop has replied. Reasoning, metadata and refusal blocks
  do not count as assistant text. After the real turn is told, queued
  messages take precedence; otherwise the run tells Waiting with its read
  fence and listens for the charter's waiting interval. A new message
  continues main as a user message, once, without a tool result. No tool
  call or result is invented, and a successful reply spends no nudge.
  A zero interval parks immediately. Explicit `wait` can park silently;
  empty, truncated, refused and no-calls ends keep their unsuccessful-end
  policy (`run.md`, 7.2). In the wind-down a successful text reply tells its fence
  and parks at once; queued unread words belong to a later activation,
  never to work beyond this budget.
- **Awaiting** a first message is waiting without a session (`run-start.md`, 1.4): told,
  timed and parked the same way. It has read nothing, so its waiting
  names no fence.
- **Parking in Finite wind-down.** A granted main `wait` parks immediately,
  including on the last reserved completion. Queued words stay unread for the
  next activation; no further completion spends this activation's reserve.
  This does not grant wait to a child or bypass hard monetary or wall limits.
- **Parking.** Past its charter's waiting time it parks: every turn it
  took is sent, and it answers parked. A run whose ordinary time runs out
  while it waits parks too: it has nothing to finish, so it does not
  wind down (`budget.md`, 1). Its state is its transcript, which the host already
  has, so it hands over nothing else. Parking closes main's session,
  which ends its jobs before the run answers (session.md, section 5): a
  parked run has nothing of its own still running, so a host that knows
  of nothing else to come may say so in the message that wakes it. A run that parks while its `wait`
  is withheld tells its last turn without that call's result. Resumed, it
  supplies that result in its first request, with the messages that wake
  it, as a live `wait` is answered, just as it supplies the answers its
  transcript does not hold (below; session.md, section 3).
- **Resuming.** A run whose charter resumes opens its main session from
  the transcript in its start (its current window, session-context.md, section
  1), its turns as they were, provider blocks
  included; tells the LLM, as text in its waking prompt, of the calls its
  host answered that the transcript does not hold as answered (one whose
  turn was never told, the agent having stopped between the two, or one
  the run withdrew and its host decided later), with their answers, so the
  LLM does not ask them again; and goes on with the messages that woke
  it, which its start carries (`run-start.md`, 1.2). Each activation starts with fresh bounded
  turns and wall time; its waking instruction explicitly supersedes exhaustion
  and finish-now instructions retained from earlier activations. The host
  supplies only the task funding still available, so monetary spend stays
  cumulative and bounded across activations. Its waking prompt follows `run-start.md`, section 1.4
  's resume form; it does not add a fresh begin instruction. A transcript
  it cannot use (another version, malformed, unresolved, too large) fails
  the run as transient, saying which (session.md, section 3), and the host decides
  what the next run starts from. Whether a run resumes at all is its
  host's decision, made before the start, since a run that resumes is
  given no brief to start fresh from. A host that moves a task to
  another endpoint or backend, program to direct or back, starts its
  run fresh rather than offer it the window, as jig's engine does
  (jig's `domain/engine/executors-transcripts.md`, 1, *Resumed only where it ran*).
  A window a host offers anyway, or one that ran on another dialect or
  account, is not refused: it is carried over (session-context.md, 1.2). A run
  that does not resume starts fresh from its brief, which carries what
  the host thinks it needs.
- **Messages mid-work.** Between
  completions, not only when main yields, the run offers its queued
  messages to the working session, which appends them after the results
  of the calls it just ran, in its next request; the turn that reads them
  is told with them, its fence naming the last. Nothing in flight is
  interrupted and nothing already sent changes, so the prefix holds and
  every rule above stands. The run's threshold notes on its budget take
  the same path, as text the run wrote, with no name (`budget.md`, 2).
