# The channel's records

What each record carries, by its fields; the schema gives the bytes. In
the tables, a **call** is answered once, an **answer** is that one
answer, and a **notice** is not answered.

## 1. Up, worker to engine

| Record | Fields | Shape |
|---|---|---|
| hello | slots; stop bound, a duration; the agent's bounds: for each endpoint it serves, its number and the largest window and output a charter may name on it; the most turns, spend, time and waiting a run may have; its checks' timeout; the largest message a run takes and how many its inbox holds; the turns it keeps unacknowledged (`README.md`, 4.3); the longest host answer its agent takes, in bytes, if it names one (`../../domain/hosts/README.md`, section 2); the job kinds it runs, ascending and each once, each with the most bytes a spec and a result of it may hold and its longest deadline (4); workstreams held; runs hosted, jobs among them, each with its task, attempt and phase; runs whose turns or answer wait for acknowledgement | first, once |
| turn | task, attempt; its number within the attempt; its cumulative spend; the last message it read, by name, if any; the messages refused since the run's previous turn, each by name with smith's reason: too large as the run reads it, the run's inbox full, a name in use, or the run ending; whether it opens a window; its bytes, smith's (jig/docs/design/protocol/channel/README.md, 5.1) | notice, kept by the hub until acknowledged |
| call | task, attempt; the run's name for the call, as bytes (`README.md`, section 6); the host tool's name; whether it writes; its input; the time left on its deadline | call |
| answer | task, attempt; the turns it took; what it spent; the last message it read, by name, if any: the unread fence; the messages refused since the run's last turn, each by name with smith's reason, as a turn carries them; how it ended: refused (busy, or invalid and what), ended with its result, smith's bytes (`README.md`, 5.1), parked, or failed with its typed failure and bounded detail, and, for a failure of its model, the model call where its application names one: endpoint and model, each at most 40 bytes, its kind, its evidence and its HTTP status, if one came (`../../domain/hosts/hub.md`, 3); what it left on resources and what it saved, each the application's bytes (`README.md`, 5.2) | answer to an assignment, kept by the hub until acknowledged; a refusal, kept by nothing (below) |
| job answer | task, attempt; how it ended: refused (busy, or invalid and what), finished with its result, the application's bytes, or failed with its typed failure and bounded detail (4); what it left on resources, the application's bytes (`README.md`, 5.2) | answer to a job, kept by the hub until acknowledged; a refusal, kept by nothing |
| fact | task, attempt; its value, activity or a piece (the schema's `FactValue`, so that "activity" stays the name of what carries no content): activity: the run's phase, a long operation begun (checks, an LLM completion or a command, with its span) or ended, or a count from smith's channel facts (its kind, its time, its count), all jig's typed activity, never smith's bytes; or a piece of live content: its turn, retry, kind (`text`, `reasoning` or `input`) and offset, and its bytes, when the assignment asked; or, from a host that gives it, a tool begun with its call's name, its tool and its input (`README.md`, 5.4) | notice, best effort |
| limited | task, attempt; an LLM account and its source; how long until it may be used again, only where its provider said | notice |
| refused | task, attempt; an LLM account and its source; its provider's code and reason, as text within the detail's bound | notice |
| unauthorized | task, attempt; an LLM account and its source; the generation its provider refused, or whose program login was lost | notice |
| logins | for each program source the engine named (2), its name and its login's state on this host: usable, not signed in, signed in otherwise and how, or failed, with bounded detail (smith's `protocol/host.md`, section 2); and, for a usable one, the email its status names, of at most 255 bytes, where it names one, which the engine keeps as a fact of the account (`../../domain/accounts.md`, 3.7) | notice: once checked after each `sources`, then on each change |
| signed out | a program source's name; whether the program's sign-out ran, or failed, with bounded detail | answer to a sign out |
| list read | a read's number; at most 256 rows, each a name of at most 128 bytes, a shown name of at most 64, a window, an output and at most 8 efforts of at most 16 bytes, or a typed failure: refused with a status, unreachable, unreadable, deadline or the agent's fault (`../../domain/hosts/README.md`, 5.1) | answer to a read list, best effort: a read's number the engine no longer holds is dropped |
| draining | none: the worker has started to shut down, and the fleet places nothing more on this channel (`../../domain/engine/fleet.md`) | notice: once as its shutdown begins, before anything its cancelled runs send, and again after each hello while it shuts down (`../../domain/hosts/hub.md`, 7) |

- **Every login notice is whole:** the current `sources` list, each name
  once, including an empty list. An explicit check publishes it even when
  unchanged. The fleet keeps the sent list and at most one desired
  replacement, and sends the replacement only after the sent list's first
  inventory. While awaiting that inventory, a snapshot naming exactly the
  previous sent list is stale and changes nothing; one naming exactly the
  sent list answers it. Any other list is a link fault. After that first
  inventory, only the sent list is valid. Names, rather than request IDs,
  distinguish these ordered replacements.

Transcript refusal keeps its typed reason on the channel: `StartCause`
appends `transcript`, and `RunFailure` appends a unit `transcript` variant.
The failure's bounded detail carries the diagnostic. Existing variant
tags stay fixed (`../README.md`, 7).

- **A refused start is not kept.** An answer that refuses its start
  (busy, a duplicate among them, or invalid, at the entrance or as a
  start the agent refuses; `../../domain/hosts/hub.md`, 1 and 3) crosses
  once, and no acknowledge follows it (`../../domain/hosts/hub.md`, 2;
  `../../domain/engine/fleet.md`).
- **A call the run withdraws** ends on the worker: the hub answers the run
  that it is withdrawn and drops the engine's answer if one comes
  (`../../domain/hosts/hub.md`, 2). Nothing crosses for it. A call asked again
  by the same name crosses again: the engine answers it busy while it is
  still deciding the first, so the run asks again, and from the record it
  kept once the first is decided, never deciding it twice
  (`../../domain/engine/executors.md`, 3).
- **Live call capacity is retryable.** After checking the current hosting
  channel and bounded opaque name, the fleet answers a call beyond its
  capacity as busy at that original channel, run, attempt and name. It
  allocates no call flight and starts no tool effect. Only the name travels
  up: the fleet drops the call's input. A stale or foreign
  call, or one whose name exceeds the bound, is still discarded before
  capacity handling. The application only forwards the addressed answer.
- **Refusals ride with turns** (`../../domain/hosts/hub.md`, 4): a turn or an
  answer carries those the hub kept since the run's previous turn, at
  most as many as the run's inbox takes, and carries them again after a
  hello.
- **The accounts' notices are relayed as the run's agent sends them**
  (smith's `protocol/channel.md`, section 4): limited, refused and
  unauthorized, each naming the account and source by the grant's
  number. They are LLM accounts' only (`README.md`, section 7), and the core decides
  what each changes (`../../domain/accounts.md`, 3.5). Unsupported is the
  run's failure, never a notice. A limited notice preserves an absent
  reset as absent; an explicit zero span is a supplied reset. The core
  converts a supplied span to engine time once on receipt, and the accounts
  child keeps its wall reset across restarts (`../../domain/accounts.md`,
  3.5).

## 2. Down, engine to worker

| Record | Fields | Shape |
|---|---|---|
| assignment | task, attempt; the run's identity, 16 bytes (`README.md`, section 6); the workstream key and the workspace's items, the application's bytes; whether to save unfinished work; the ask for live content, a boolean for each kind (`../../domain/hosts/README.md`, section 2); the charter, smith's bytes; the transcript, when the run resumes: its current window's turns, smith's bytes, from the last that opens a window (`README.md`, 5.1); the calls committed since its last turn, each with its name, its tool and its answer; the start's messages: those waiting in the task's inbox, oldest first, as many as the run's inbox takes, each with its name, its sender's label and its words, the rest relayed as message records as the run reads (`../../domain/tasks/messages.md`, 2); its credential grants, each naming its form, which selects the table its value comes from, with their values: a credential with its scheme, or a program source's name (`README.md`, section 7) | call |
| job | task, attempt; its kind; its spec, the application's bytes; its deadline; the workstream key and the workspace's items, the application's bytes; its credential grants, the application's only (`README.md`, section 7) (4) | call |
| message | task, attempt; the message's name; its sender's label; its words | notice |
| relayed | task, attempt; the call's name; the engine's answer: its text for the LLM and whether it is an error, or busy, to ask again after a backoff | answer to a call |
| turn kept | task, attempt; the turn's number | notice |
| turn busy | task, attempt; the turn's number: send it again after a backoff | notice |
| acknowledge | task, attempt: the answer is durable, or not wanted | notice |
| cancel | task, attempt | notice, at most one per attempt |
| grant | task, attempt; its form, which selects the table its value comes from, jig's accounts' or the application's; its number there, generation, how long it is valid, if it lapses; its value: a credential with its scheme, a program source's name, or the application's secret (`README.md`, section 7) | notice |
| sources | the program sources this worker may be given, each by its name, the whole list: a source not in it is none of the worker's | notice, after each hello and on each change |
| check | a program source's name: check its login now, at its holder's asking | notice; `logins` follows |
| sign out | a program source's name: run the program's own sign-out in the source's configuration directory (smith's `protocol/host.md`, section 2) | call, answered by `signed out` |
| read list | a read's number; an endpoint; a grant, as `grant`'s form and value, a credential alone (`README.md`, section 7): read this account's list once (`../../domain/hosts/README.md`, 5.1) | call, answered by `list read` |

- **The fleet's refusals travel as the records above:** a turn answered
  busy, a relayed call answered busy or as an error, and an answer
  acknowledged again when the engine has finished with its attempt
  (`../../domain/engine/fleet.md`).
- **An assignment that cannot encode ends at the engine's component.**
  It preserves the task, attempt and channel, sends no assignment to the
  worker, and returns one upward answer: invalid start, cause other,
  zero turns and spend, no read fence, refused messages or application
  parts. Its diagnostic is ASCII `assignment cannot encode`, shortened
  to the configured detail bound, empty at zero. The fleet settles this
  refusal through its usual answer route, releasing the reservation;
  the worker has no start to cancel. Other records that cannot encode
  are counted and discarded.
- **A start the link cannot carry ends there too** (`README.md`, section 8, *A start
  past the room fails its run, never the link*): an assignment or a job
  whose frame is past the room the engine's stream grants at once, or
  that skein's channel refuses at its entrance for anything but a full
  queue, is answered the same way, with no record sent. An assignment's
  committed calls are fitted to the room first (`README.md`, section 8, *A resumed
  start fits its room first*), so an assignment is past the room only
  with every host answer cut. Its diagnostic
  names the record, its frame's bytes and the bound: `assignment of
  BYTES bytes past the link's room of ROOM bytes`, or `assignment of
  BYTES bytes refused by the channel: REASON`, `job` for a job. A job
  that cannot encode is answered too, as its job's invalid start, `job
  cannot encode`, rather than discarded.
- **Busy, sent again.** A turn answered busy is held by the worker's
  link component, which sends it again after a jittered backoff, as it
  keeps the grace's clock (`README.md`, 9.1); the hub arms no timers
  (`../../domain/hosts/hub.md`, 2). Turn busy is not in the hub's link
  vocabulary (`../../domain/hosts/hub.md`, 1): `jig-protocol-engine-workers`
  encodes it from the fleet's request, beside the hub's records; the
  worker's link takes it, and at its deadline asks its root for the
  hub's kept copy of the turn, which goes up again as any turn. For the
  engine's own hub, its root keeps the backoff the same way, on its
  protocol layer's timers. A call answered busy goes back to its run,
  which asks it again after a backoff (smith's `domain/run.md`, 5.2). An
  answer the engine leaves unacknowledged is sent again after the next
  hello.
- **A refused worker** is refused by skein's channel, with jig's reasons;
  the domains have no record of their own for it.

## 3. Order

- **A component's channel cascade settles before the next entrance.**
  `channel_events_dispatched` requires every channel event generated by
  the entrance to be handled; `lower_requests_dispatched` requires its
  lower requests to be handed out before the component takes another
  entrance. A refusal's send, finish and terminal therefore precede a
  later entrance's output. Both component worlds hold this with buffered
  records and delayed write credit in `jig-engine-workers-world` and
  `jig-worker-link-world`, each in
  `cascades::a_channel_cascade_settles_before_the_next_entrance`.
- **Each side sends in its domain's order,** and skein's channel keeps it.
  The fleet and the hub rely on it: an answer goes after the turns it
  counts (`../../domain/hosts/hub.md`, 4), messages go in their order, a
  start's before those sent after it (`../../domain/tasks/messages.md`, 2), a
  worker's draining before the answers of the runs its shutdown cancels
  (`../../domain/hosts/hub.md`, 7), and acknowledgements in the order of the
  commits they follow.
- **After a redial, everything kept goes again,** in order, after the
  hello: the hub's unacknowledged turns and answers, with the refusals
  they carry, and its relays still waiting; the engine's program
  sources, whole, and its current credential grants for every attempt it
  keeps (`README.md`, section 12 says what the fleet owes for them). A sign out lost
  with its channel is not sent again: its answer names the host as not
  signed out. The domains drop what they have seen
  (`../../domain/engine/fleet.md`; `../../domain/hosts/hub.md`, 6).

## 4. Jobs

A job (`../../domain/hosts/hub.md`, 9) crosses as a run does, by records of
its own where a run's carry an agent's parts:

- **Its kinds, in the hello:** each kind a number the application
  gives, with the most bytes a spec and a result of it may hold and the
  longest deadline the worker runs one to, from its job capability; none
  for a worker without one. The fleet places a job only where its kind
  is declared and those bounds cover the job's (`../../domain/engine/executors.md`,
  5).
- **Down, a `job`** in place of an assignment: no identity, charter,
  transcript, calls, messages, ask for live content, LLM account or
  program source, and no save. Its grants are the application's
  credentials its workspace needs, which the workspace keeps and the job
  is never given. A `cancel` and an `acknowledge` are a run's, by task
  and attempt.
- **Up, a `job answer`** in place of an answer: no turns, spend, unread
  fence or refusals, since a job takes no messages. Its typed failures
  are: its workspace not prepared, transient or permanent as a run's;
  failed, as the job said; its deadline passed; its result past its
  kind's bound; not started, or ended without its capability's word;
  undelivered, its delivery stale, refused or failed; cancelled, by the
  engine, for lost contact or for shutdown. Invalid, at the entrance,
  adds to a run's: a kind the worker does not run, and a spec or a
  deadline past its kind's bounds.
- **Hosted, it is listed** in the hello as a run is, its phase preparing,
  starting, active, ending or answered, never waiting, and its answer
  goes again after a redial until acknowledged (3).
- **No turns, calls, facts of live content or account notices** cross
  for a job.
