# The channel's records

What each record carries, by its fields; the schema gives the bytes. In
the tables, a **call** is answered once, an **answer** is that one
answer, and a **notice** is not answered.

## 1. Up, worker to engine

| Record | Fields | Shape |
|---|---|---|
| hello | slots; stop bound, a duration; the agent's bounds: for each endpoint it serves, its number and the largest window and output a charter may name on it; the endpoint numbers that serve programs; the most turns, spend, time and waiting a run may have; its checks' timeout; the largest message a run takes and how many its inbox holds; the turns it keeps unacknowledged (`README.md`, 4.3); the longest host answer its agent takes, in bytes, if it names one (`../../domain/hosts/README.md`, section 2); the job kinds it runs, ascending and each once, each with the most bytes a spec and a result of it may hold and its longest deadline (4); workstreams held; runs hosted, jobs among them, each with its task, attempt and phase; runs whose turns, parts or answer wait for acknowledgement; `fresh`, whether no hello of the worker's process has yet been admitted (`../../domain/engine/executors.md`, 6.5), with the schema's default `false` for a layout without the field (`layouts.md`, 2.2) | first, once |
| turn | task, attempt; its number within the attempt; its cumulative spend; the last message it read, by name, if any; the messages refused since the run's previous turn, each by name with smith's reason: too large as the run reads it, the run's inbox full, a name in use, or the run ending; whether it opens a window; its bytes, smith's (jig/docs/design/protocol/channel/owners.md, 1.1) | notice, kept by the hub until acknowledged |
| part | task, attempt; its number within the attempt, apart from turns; the turn it belongs to, as a turn's number in the attempt; its thread and its call, numbers smith gives; its kind, `answer`, `result` or `window`; its bytes, smith's turn (smith's `domain/turn-parts.md`, section 2), (5) | notice, kept by the hub until acknowledged; sent only where the engine's terms list it (`layouts.md`, 4.2) |
| call | task, attempt; the run's name for the call, as bytes (`README.md`, section 6); the host tool's name; whether it writes; its input; the time left on its deadline; `contract`, the required `u16` version of the host's tools contract its run's charter names (`../../domain/hosts/hub.md`, 8), with no default: no Read plan derives from a layout without the field (`layouts.md`, sections 2.2 and 3) | call |
| answer | task, attempt; the turns it took; what it spent; the last message it read, by name, if any: the unread fence; the messages refused since the run's last turn, each by name with smith's reason, as a turn carries them; how it ended: refused (busy, or invalid and what), ended with its result, smith's bytes (`owners.md`, 1.1), parked, or failed with its typed failure and bounded detail, and, for a failure of its model, the model call where its application names one: endpoint and model, each at most 40 bytes, its kind, its evidence, its HTTP status, if one came, and the name of its refusal's cause, if it was refused before it was sent (`../../domain/hosts/hub.md`, 3); what it left on resources and what it saved, each the application's bytes (`owners.md`, 1.2) | answer to an assignment, kept by the hub until acknowledged; a refusal, kept by nothing (below) |
| job answer | task, attempt; how it ended: refused (busy, or invalid and what), finished with its result, the application's bytes, or failed with its typed failure and bounded detail (4); what it left on resources, the application's bytes (`owners.md`, 1.2) | answer to a job, kept by the hub until acknowledged; a refusal, kept by nothing |
| fact | task, attempt; its value, activity or a piece (the schema's `FactValue`, so that "activity" stays the name of what carries no content): activity: the run's phase, a long operation begun (checks, an LLM completion or a command, with its span) or ended, or a count from smith's channel facts (its kind, its time, its count), all jig's typed activity, never smith's bytes; or a piece of live content: its turn, retry, kind (`text`, `reasoning` or `input`) and offset, and its bytes, when the assignment asked; or, from a host that gives it, a tool begun with its call's name, its tool and its input (`owners.md`, 1.4) | notice, best effort |
| limited | task, attempt; the LLM grant's source number (`account`); how long until it may be used again, only where its provider said | notice |
| refused | task, attempt; the LLM grant's source number, as limited's; its provider's code and reason, as text within the detail's bound and their respective wire ceilings of 128 and 512 bytes | notice |
| unauthorized | task, attempt; the LLM grant's source number, as limited's; the generation its provider refused, or whose program login was lost | notice |
| usage | task, attempt; the LLM grant's source number, as limited's; one usage window (below) | notice |
| logins | for each program source the engine named (2), its name and its login's state on this host: usable, not signed in, signed in otherwise and how, or failed, with bounded detail (smith's `protocol/host.md`, section 2); and, for a usable one, the email its status names, of at most 255 bytes, where it names one, which the engine keeps as a fact of the account (`../../domain/accounts-lists.md`, 1) | notice: once checked after each `sources`, then on each change |
| signed out | a program source's name; whether the program's sign-out ran, or failed, with bounded detail | answer to a sign out |
| list read | a read's number; at most 256 rows, each a name of at most 128 bytes, an optional shown name of at most 64, an optional window, an optional output and at most 8 efforts of at most 16 bytes, or a typed failure: refused with a status, unreachable, unreadable, deadline or the agent's fault (`../../domain/hosts/README.md`, 5.1) | answer to a read list, best effort: a read's number the engine no longer holds is dropped |
| usage read | a read's number; at most 8 usage windows (below), or not signed in, spent, or failed with the source's failure and bounded detail, as `logins`' (`../../domain/hosts/README.md`, 5.1) | answer to a read usage, best effort as a list read's |
| draining | none: the worker has started to shut down, and the fleet places nothing more on this channel (`../../domain/engine/fleet.md`) | notice: once as its shutdown begins, before anything its cancelled runs send, and again after each hello while it shuts down (`../../domain/hosts/hub.md`, 7) |
| ask | task, attempt; the ask's number, the hub's for that attempt; the application's bytes, within the application's largest ask (`../../domain/hosts/hub.md`, 10) | call, answered by `asked` |
| paused | the worker's state: taking, paused, draining or drained (`pause.md`) | notice: at each change of its state, after every pause request while not shutting down even if the state stays the same, and again after each hello while it is not taking, where the engine's terms list it (`../../domain/hosts/hub.md`, 7) |

- **A usage window** is a program's report of one window
  (`../../domain/accounts-lists.md`, 1): its label, 1 to 32 printable ASCII
  bytes; its length, an optional duration; its share used, optional basis
  points to 10,000; its reset, an optional wall time, the provider's own;
  and its state, the registry's number. The hub takes it from its agent's
  usage notice (smith's `protocol/channel.md`, section 4), the root mapping
  smith's state to the registry's, and relays it as the limited notice is.
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
has `transcript`, and `RunFailure` has a unit `transcript` variant.
The failure's bounded detail carries the diagnostic. Plans map variants
by name across layouts (`layouts.md`, 2.1).

A program that started without its tools keeps its typed reason too:
`RunFailure` has a unit `untooled` variant
(`../../domain/hosts/hub.md`, 3), its detail the agent's as for any
failed run.

Usage windows (`../../domain/accounts-lists.md`, 1) are `usage` and
`usage read` up and `read usage` down; the engine sends `read usage`
only where the worker's layout has it (`layouts.md`, 4.1).
In jig's link vocabulary, `jig-domain-link`, the lowest that the hub, the
fleet and the accounts child all name:

```rust
/// A window's value, moved from `jig-domain-accounts`, which re-exports
/// it and the window states under their names (../../domain/accounts-lists.md, 1).
pub struct FactWindow { pub length: Option<Duration>, pub used: Option<u16>, pub resets: Option<Wall>, pub state: u16 }
/// One usage window as a program reports it.
pub struct UsageWindow { pub label: Box<[u8]>, pub window: FactWindow }
pub const USAGE_WINDOWS: u32 = 8;
/// A usage read's answer.
pub enum UsageOutcome { Read(Box<[UsageWindow]>), NotSignedIn, Spent, Failed { reason: SourceFailure, detail: Box<[u8]> } }
// jig-domain-host's link: LinkUp::Usage { run, attempt, account, window: UsageWindow },
// LinkUp::UsageRead { read: u64, outcome: UsageOutcome },
// LinkDown::ReadUsage { read: u64, source: SourceName }.
// Hello::reads_usage: bool, no field of the wire's: the engine's component sets
// it where the worker's layout has `read usage`, for the fleet's placing.
```

A call refused before it was sent keeps its cause too: `ModelFailure`
has `refusal: option text 24 = none`
(`../../domain/hosts/hub.md`, 3).

- **A refused start is not kept.** An answer that refuses its start
  (busy, a duplicate among them, or invalid, at the entrance or as a
  start the agent refuses; `../../domain/hosts/hub.md`, 1 and 3) crosses
  once. No acknowledge follows a matched refusal; the fleet acknowledges
  and drops one for an unknown, crossed, fenced or already acknowledged
  attempt
  (`../../domain/hosts/hub.md`, 2; `../../domain/engine/fleet.md`).
- **A call the run withdraws** ends on the worker: the hub answers the run
  that it is withdrawn, or unsent where the link never carried it, and
  drops the engine's answer if one comes (`../../domain/hosts/hub.md`, 2
  and 6). Nothing crosses for it. A call asked again
  by the same name crosses again: the engine answers it busy while it is
  still deciding the first, so the run asks again, and from the record it
  kept once the first is decided, never deciding it twice
  (`../../domain/engine/executors-tools.md`, 1).
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
  (smith's `protocol/channel.md`, section 4): limited, refused,
  unauthorized and usage, each carrying the grant's source number in
  `account`. They are LLM accounts' only (`README.md`, section 7), and the
  core decides what each changes (`../../domain/accounts-sources.md`, 2). Unsupported is the
  run's failure, never a notice. A limited notice preserves an absent
  reset as absent; an explicit zero span is a supplied reset. The core
  converts a supplied span to engine time once on receipt, and the accounts
  child keeps its wall reset across restarts (`../../domain/accounts-sources.md`,
  2).

## 2. Down, engine to worker

| Record | Fields | Shape |
|---|---|---|
| assignment | task, attempt; the run's identity, 16 bytes (`README.md`, section 6); an optional workspace: its workstream key and items, the application's bytes; whether to save unfinished work; the ask for live content, a boolean for each kind (`../../domain/hosts/README.md`, section 2); the charter, smith's bytes; the transcript, when the run resumes: its current window's turns, smith's bytes, from the last that opens a window (`owners.md`, 1.1); the calls committed since its last turn, each with its name, its tool and its answer; the parts kept since its last turn, each with the attempt that told it, its turn, thread, call, kind and bytes, as a part has them (5); the start's messages: those waiting in the task's inbox, oldest first, as many as the run's inbox takes, each with its name, its sender's label, its words and `finish_requested`, whether it asks the run to finish (`README.md`, 4.1), the rest relayed as message records as the run reads (`../../domain/tasks/messages.md`, 2); its credential grants, each naming its form, which selects the table its value comes from, with their values: a credential with its scheme, the application's secret, or a program source's name (`README.md`, section 7) | call |
| job | task, attempt; its kind; its spec, the application's bytes; its deadline; an optional workspace: its workstream key and items, the application's bytes; its credential grants, the application's only (`README.md`, section 7) (4) | call |
| message | task, attempt; the message's name; its sender's label; its words; `finish_requested`, whether it asks the run to finish (`README.md`, 4.1) | notice |
| relayed | task, attempt; the call's name; the engine's answer: its text for the LLM and whether it is an error, or busy, to ask again after a backoff | answer to a call |
| turn kept | task, attempt; the turn's number | notice |
| turn busy | task, attempt; the turn's number: send it again after a backoff | notice |
| part kept | task, attempt; the part's number: the worker may forget it and every earlier part of the attempt. The engine has durably settled the prefix (a moot body need not be stored), or wants none of it for an unknown or fenced attempt (5) | notice, where the worker's layout has it |
| part busy | task, attempt; the part's number: send it again after a backoff, as a turn busy | notice, where the worker's layout has it |
| acknowledge | task, attempt: the answer is durable, or not wanted | notice |
| cancel | task, attempt | notice, at most one per attempt on each connection; sent again on a redial that lists the still-unanswered attempt to be cancelled |
| grant | task, attempt; its form, which selects the table its value comes from, jig's accounts' or the application's; its number there, generation, how long it is valid, if it lapses; its value: a credential with its scheme, a program source's name, or the application's secret (`README.md`, section 7) | notice |
| sources | the program sources this worker may be given, each by its name, the whole list: a source not in it is none of the worker's | notice, after each hello and on each change |
| check | a program source's name: check its login now, at its holder's asking | notice; `logins` follows |
| sign out | a program source's name: run the program's own sign-out in the source's configuration directory (smith's `protocol/host.md`, section 2) | call, answered by `signed out` |
| read list | a read's number; an endpoint; a grant, as `grant`'s form and value, a credential alone (`README.md`, section 7): read this account's list once (`../../domain/hosts/README.md`, 5.1) | call, answered by `list read` |
| read usage | a read's number; a program source's name: run the program's usage command on it once (`../../domain/hosts/README.md`, 5.1) | call, answered by `usage read`; where the worker's layout has it |
| asked | task, attempt; the ask's number; the application's answer, its bytes within the application's largest answer, or refused: the attempt is not its task's live one, or the application answers none (`../../domain/hosts/hub.md`, 10) | answer to an ask, best effort: a number the hub no longer holds is dropped |
| restarting | the restart wait, in milliseconds: the engine restarts, and asks the worker to keep its runs that long after the link drops (`restart.md`) | notice: at the engine's settled stop, where the worker's layout has it |
| figures | the worker's places, the engine's limits and its agent settings (`figures.md`, 2) | notice: for a figures opening, or on a link where the worker's layout has it (`figures.md`, 3) |
| pause | the mode: pause, take no new run or job; drain, pause and drain; resume, take work again (`pause.md`) | notice: at an owner's ask (`../../domain/engine/workers.md`, 5), where the worker's layout has it |

- **The fleet's refusals travel as the records above:** a turn answered
  busy, a relayed call answered busy or as an error, and an answer
  acknowledged again when the engine has finished with its attempt
  (`../../domain/engine/fleet.md`).
- **An assignment that cannot encode ends at the engine's component.**
  It preserves the task, attempt and channel, sends no assignment to the
  worker, and returns one upward answer: invalid start, cause other,
  zero turns and spend, no read fence, refused messages or application
  parts. Its diagnostic is its reason's words (below), shortened to the
  configured detail bound, empty at zero. The fleet settles this
  refusal through its usual answer route, releasing the reservation;
  the worker has no start to cancel. A `read list` that cannot encode is
  counted and dropped, and answered upward as `ListFailure::Agent` for
  that read. Except for such requests answered upward, records that
  cannot encode are counted and discarded.
- **A record that cannot encode says why.** `jig-protocol-engine-workers`'
  `translate::down` and the records it builds return
  `Result<_, Unencoded>`, never a bare none, and its component renders
  the reason's words where it answers the record:

  ```rust
  /// Why a record cannot encode: what didn't fit, and the figures known.
  #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
  pub struct Unencoded {
      /// A codec path's name (`Assignment.charter`, skein's codec, 4), or
      /// the encoder's own field in lower-case dotted form (`grant.value`).
      pub field: &'static str,
      /// The codec's reason, where a codec refused it.
      pub reason: Option<skein_codec::Reason>,
      /// The field's bytes or count, where the encoder measured it.
      pub figure: Option<u64>,
      /// The configured limit it was held to.
      pub bound: Option<u64>,
      /// The family's ceiling for it.
      pub ceiling: Option<u64>,
  }

  impl Unencoded {
      /// `WHAT cannot encode: FIELD`, then what is known.
      pub fn words(&self, what: &[u8]) -> Box<[u8]>;
  }
  ```

  - **Its words,** in step code, figures with thousands' commas: `WHAT
    cannot encode: FIELD`, then ` REASON` where there is one, ` FIGURE`
    where there is one, `, bound BOUND` and `, wire ceiling CEILING`
    where they are known. `WHAT` is `assignment`, `job` or `record`. A
    reason reads `too short`, `too large`, `bad tag`, `bad boolean`,
    `not UTF-8`, `unknown version`, `trailing bytes` or `malformed`.
    So `assignment cannot encode: Assignment.transcript 5,000,000,
    bound 4,194,304, wire ceiling 134,217,728`, and `assignment cannot
    encode: Message.words not UTF-8`.
  - **Its figures.** Where the translation checks a value before encoding
    (an assignment's charter and its framed transcript, a list's count),
    it gives the figure, the configured limit and the family's ceiling,
    and no reason, which they say; a codec's refusal gives its path and
    reason. Records have public fields, checked where bytes are made or
    read (skein's `data/codec/README.md`, section 4). A refusal with no codec path names the encoder's own field:
    a missing part, a grant whose value doesn't match its form, a
    program's sources that repeat a name.
  - **One record for temper's too.** temper's translation fits a charter
    before it builds the assignment and refuses with this record, naming
    its fitting's fields (temper's `domain/agent/room.md`, 3).
- **A start the link cannot carry ends there too** (`liveness.md`, section 1, *A start
  past the room fails its run, never the link*): an assignment or a job
  whose frame is past the room the engine's stream grants at once, or
  that skein's channel refuses at its entrance for anything but a full
  queue, is answered the same way, with no record sent. An assignment's
  committed calls are fitted to the room first (`liveness.md`, section 1, *A resumed
  start fits its room first*), so an assignment is past the room only
  with every host answer cut. Its diagnostic
  names the record, its frame's bytes and the bound: `assignment of
  BYTES bytes past the link's room of ROOM bytes`, or `assignment of
  BYTES bytes refused by the channel: REASON`, `job` for a job. A job
  that cannot encode is answered too, as its job's invalid start, its
  reason's words (`job cannot encode: FIELD`), rather than discarded.
- **Busy, sent again.** A turn answered busy is held by the worker's
  link component, which sends it again after a jittered backoff, as it
  keeps the grace's clock (`worker-link.md`, 1.1); the hub arms no timers
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
- **After a redial, kept records go again,** in order, after the
  hello: the hub's unacknowledged turns, parts and answers, with the
  refusals the turns and answers carry, and its relays still waiting.
  A retained part is settled locally instead where the engine's terms
  lack `part` (5). The engine sends its program sources, whole, and its
  current credential grants for every attempt it keeps (`README.md`,
  section 12 says what the fleet owes for them). A sign out lost with its
  channel is not sent again: its answer names the host as not signed out.
  The domains drop what they have seen
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
  engine, for lost contact or for shutdown. A job's invalid start has its
  own reasons: a kind the worker does not run, a spec or deadline past its
  kind's bounds, invalid grants, or a start refused with bounded detail.
- **Hosted, it is listed** in the hello as a run is, its phase preparing,
  starting, active, ending or answered, never waiting, and its answer
  goes again after a redial until acknowledged (3).
- **No turns, calls, facts of live content or account notices** cross
  for a job.

## 5. Parts

A run whose agent tells its turns' parts (smith's `domain/turn-parts.md`)
sends them up, and a resumed run is given back those kept since its last
turn. `Assignment.parts` has an empty default; plans judge the two
layouts, not the field's place (`layouts.md`, 2.2; skein's
`data/codec/README.md`, sections 3 and 5).

```
enum PartKind { answer, result, window }
record Part { task: u64 attempt: u64 number: u32 turn: u32 thread: u32 call: u32 kind: PartKind body: bytes 33554432 }
record PartKept { task: u64 attempt: u64 number: u32 }
record PartBusy { task: u64 attempt: u64 number: u32 }
record KeptPart { attempt: u64 turn: u32 thread: u32 call: u32 kind: PartKind body: bytes 33554432 }
# Assignment's field: parts: bytes 134217728 = 0x
```

- **An assignment's parts travel inside one byte envelope.** It holds at
  most 4,096 `KeptPart` records, each preceded by its four-byte length.
- **The engine uses the part's numbers and size.** It treats thread zero
  as the main thread and groups sub-agent parts by call when selecting
  and fitting them. The core keeps the body's turn bytes opaque; the
  engine's protocol converts the body as a `Turn` across layouts
  (`owners.md`, 1.1; `layouts.md`, 2.1).
- **Kept until acknowledged, as a turn:** where the engine's terms take
  it, a part goes up at once, again after each hello, and after a backoff
  when the engine answers it busy. It counts with the run's turns against
  what the hub keeps unacknowledged (`../../domain/hosts/hub.md`, 4).
  Parts and turns go up in the order the agent told them.
- **Numbered apart from turns,** from one in each attempt. Its `turn`
  is the number the turn it belongs to will have in that attempt, the
  last turn told plus one, so a part of turn `n` is moot once turn `n`
  or a later one of its attempt, or any turn of a later attempt, is
  committed (`../../domain/engine/executors-transcripts.md`, 1).
- **An assignment's parts** are those the engine kept for the task
  since its last committed turn, by attempt, then number, each with the
  attempt that told it: none when the run starts fresh.
- **Parts go first to fit a start's room** (`liveness.md`, section 1): an
  assignment carrying parts that does not fit whole is measured again
  without its sub-agents' parts, each sub-agent's all or none from the
  last call back, then without its parts at all, before any host answer
  is cut; a run so started resumes from its window alone, or opens those
  sub-agents afresh (smith's `domain/turn-parts.md`, section 6). The
  core keeps the parts as it keeps the calls.
- **Only where the other side has them.** A worker whose engine's terms
  lack `part` asks its agents for no parts, so it sends none
  (`layouts.md`, 4.2). After a redial whose terms lack `part`, the
  worker's link answers a retained part with `PartKept` locally instead
  of sending it. `part kept` and `part busy` go only to a worker that
  sent the part they answer. For an assignment, the engine drops the kept
  parts the worker's layout refuses and sends the rest; where the worker's
  layout has no `parts` field, an assignment with kept parts is refused
  (`layouts.md`, 4.1).
