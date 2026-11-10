# The agent process

Provisional, 2026-10-07, revised 2026-10-20. What an agent process is:
the `smith-agent` binary, glue over `smith-agent-shell`, which a host
spawns to run one run. It covers:

- its machine component, `smith-protocol-machine`, which carries the
  workspace's files and processes, and which the product's one process
  owns too;
- its configuration;
- its facts and its event stream;
- the process itself: its streams, its startup and how it ends.

Its channel and its LLM calls are channel.md's and llm.md's; its limits
are limits.md's, and its event vocabulary events.md's.

## 1. In one page

Live and Start-carried messages preserve `finish_requested` through decoding
and pre-admission buffering to the run entrance (domain/run.md, section 6).

- **The machine component translates the tools' and the checks'
  operations** into skein's io (domain/tools.md, section 6):
  - files loaded and stored;
  - directories scanned and trees listed;
  - searches and commands run, each in its own process group;
  - commands left running as jobs, held, read and stopped;
  - checks run and guides read.
- **Everything runs in a view.** Each command, search and check is a
  contained tree (skein's `runtime/io/processes.md`, 2–4) that sees the workspace's
  directories side by side, writable as the start says, with every git
  directory read-only. Without contained trees they are plain children in
  process groups, and the agent says so (`agent.md`, section 3).
- **Configuration is declared quantities.** It gives:
  - the endpoints and their declared limits for charter models;
  - the programs it may run, and their configuration directories'
    parent;
  - a profile: a named bundle of the rest;
  - the commands' environment;
  - where the event stream goes.

  Every limit is derived from it and every relationship checked at
  startup (limits.md). It never holds a credential, nor a raw byte
  limit.
- **One drain, two sinks.** The domain's facts are taken once, losslessly,
  and go to the host on the channel, which drops and counts, and to the
  event stream, which is complete by default and written through io
  (section 5).
- **One process, one run.** The binary reads its channel on its standard
  input and output, runs the run it is started with, sends its answer,
  closes everything it owns and exits (domain/host.md, section 1).
- **Two backends.** The agent runs a run on its own loop, over skein's
  LLM client, or on a program it spawns as its child, Claude Code first,
  as the run's grant says: a credential, or a program source's name
  (programs.md).
- **Its end is one phase:** `Live → Answered → Closing → Closed`. The LLM
  pool closes at the answer, and a termination signal after the answer
  aborts what is still closing (section 6).

## 2. The machine

| Operation | How |
|---|---|
| load a file | whole, within its bound, beneath its directory's root, with a digest of its content as its version |
| store a file | to a temporary file beside it, then renamed over it, once the file's content still has the version read; never through a symbolic link. A new file's directory is first loaded without following a link, and when nothing is there, the directories its path names that do not exist are made from the top, an existing one counting as made |
| scan a directory | its entries, within their bound |
| list a tree | the entries beneath a directory, to a depth, leaving out what the ignore files and git exclude, within their bound |
| search | `rg` in a process group of its own: no configuration, an empty environment, the pattern and glob after `--` |
| run a command | `sh -c` in a process group of its own, in the working directory, with the configured environment and the domain's deadline, its output's head and tail kept; asked with a yield, answered at the yield with its output so far while it goes on as a job |
| hold a job | its group kept running under its command's deadline, and what it writes kept as the head and the tail of the bytes not yet read, with the count dropped between them, within the job's kept output (limits.md, 3.5) |
| read a job | the job's kept output, as much of its head and its tail as the domain asks for, the rest counted as dropped, and the read empties it; then its end, once it has ended. It answers at the job's end, or when the wait the domain asks for ends |
| stop a job | its group stopped in the three steps below, then its end answered |
| run the checks | the convention's executable, in its directory, the same way |
| read a guide | as text, UTF-8, cut at a character's boundary within its bound |

- **Paths** stay beneath the workspace's roots, and io refuses one that
  leaves them. A path with a `.git` part, in any case, is refused before io
  is asked (domain/tools.md, section 4). The directory itself is `.` at io
  for every operation.
- **A version** is a digest of a file's content. Comparing it again before
  a store catches a change made meanwhile, by another session or anything
  else.
- **Output** is captured as the domain asks: how much of the head and of
  the tail, and the count of bytes dropped between them.
- **Deadlines** are the domain's. When one passes, the machine stops the
  command's process group in three steps, each given `group_stop` before
  the next:
  1. terminate: the termination signal to the group;
  2. kill: the kill signal to the group;
  3. close: io's close of the child, which kills what is left of its
     group and reaps it (skein's runtime/io/processes.md).

  It answers once the group's leader is reaped and its pipes are closed.
- **Jobs** are commands the domain left running past their call's
  answer (domain/tools.md, section 5). The machine holds each job's
  process group and its unread output until the domain has taken its
  end. A job's deadline is its command's, counted from its start: when
  it passes, its group is stopped in the three steps, and its end is
  timed out. A stop or a cancel stops its group in the same three steps.
- **Its close stops the jobs it holds.** At the answer it holds none,
  since a session's close cancels its jobs and waits for them to settle
  (domain/session.md, section 5), and the run answers only once its
  sessions have closed (domain/run.md, section 4). One it still holds,
  after a run dropped without an answer, it stops with the third step
  alone, io's close of the job's child, which kills what is left of the
  group and reaps it within one `close`, so jobs add nothing to the
  teardown bound (limits.md, section 7).
- **Its limits are derived from the tools'** (limits.md, section 3.5),
  so a tool's limit never meets a smaller one here: its processes hold
  `jobs` for each of `conversations` beside the listings, searches,
  commands and checks the run's calls may have running at once.

## 3. The view

- **The directories side by side,** each at the name the LLM calls it,
  under one parent. Relative paths and commands start in the working
  directory, the first (domain/tools.md, section 2).
- **Writable as the start says.** Every git directory is read-only, a
  repository's included.
- **The rest of the machine** is read-only, with a private temporary
  directory, so a build's tools and caches work.
- **The commands' environment is the configuration's, whole.** Every
  command and every check sees exactly the configured variables, and
  nothing else: what a build needs (a `PATH`, a home, a locale), never a
  credential. A search sees none. The domain carries no environment, and
  no call sets a variable (domain/tools.md, section 5). Startup refuses
  an environment whose names are not valid or not distinct (limits.md,
  section 4).
- **Process groups.** Each command, check and search leads a process
  group of its own, and a deadline or a stop signals the whole group, so
  what a command started ends with it, not only its shell.
- **Without contained trees,** none of this view is the kernel's.
  Commands, checks and searches run as plain children of the agent, in
  their process groups, with the agent's user's permissions and its file
  system as it is: nothing makes a git directory read-only, or the rest
  of the machine, or keeps a command within the writable directories.
  The tools' own rules still hold for `write` and `edit` (domain/tools.md,
  section 4); a command bypasses them. A process that leaves its group
  outlives its call. The agent says so: in its first line on standard
  error, and in its event stream's header (events.md, section 3.2).
  Contained trees bring the view, and resource limits per tree with it.

## 4. Configuration

What the agent is given at startup, before the loop. Charters,
transcripts and the channel's terms refer to what it holds, so that is
this document's concern. It is a JSON document, generated by whoever
spawns the agent, which `smith-agent-shell` reads whole and strictly; the
product's own settings are `../shell/README.md`'s.

- **Endpoints:** at least one. Every run needs main's LLM, and the limits
  are derived from the endpoints' limits and their smallest room (limits.md,
  3.9), so a configuration with no endpoint is refused as it is read, naming
  the field. It is never given neutral bounds, since that would invent a
  declaration. Each endpoint has:
  - its name, which charters and transcripts use;
  - **its dialect,** by name and number, each fixed for good: `openai`,
    3, OpenAI's public Responses dialect (skein's `llm/README.md`, 4.8);
    `anthropic`, 2, Anthropic's Messages dialect (skein's `llm/README.md`,
    4.7). A transcript's turns and opaque blocks carry the number
    (transcript.md, section 2), so a number is never given to another
    dialect, and 1 is given to none;
    a name and number that do not match are refused, naming the field;
  - its address, as a name resolved at startup (skein's `runtime/io/README.md`,
    section 3);
  - its transport: TLS, with its server name and what to trust it by, the
    machine's roots by default; or plaintext, only to an address on
    loopback (skein's `llm/connection.md`, section 3);
  - the dialect's path and headers, each the dialect's own by default
    (skein's `llm/README.md`). The affinity's header names are reserved and
    refused here (llm.md, 2.1);
  - **its program,** optional: the program, `claude_code`, that runs a
    run whose grant names a program source (programs.md, section 3). An
    endpoint without one takes credentials only. The account a run's
    grants are named by is its charter's LLM's (charter.md, section 2),
    never an endpoint's. Reasoning effort is a run's choice, in its
    charter's LLM too, and cache affinity is the run's, never an
    endpoint's (llm.md, 2.1);
  - its connect and handshake deadlines;
  - **its limits,** one declaration for every model served on it, with
    its quantities: `window`, `output`, `reasoning_item`, `head` and
    `idle` (limits.md, section 2.1), and whether an oversized reasoning
    item fails or is dropped. The agent holds no list of models. A charter
    takes a model from the charter's own LLM entry: it may name any on a
    configured endpoint, within that endpoint's window and output, and any
    effort within its bound (charter.md, section 2); only the provider
    knows whether it serves that name.
- **Programs:** the programs directory, which holds each program
  source's configuration directory by the source's name (programs.md,
  4.2), and each program's path, by the program's name: the path of
  `claude`, for `claude_code` (programs.md, section 3). A configuration
  whose endpoint names a program it gives no path for is refused,
  naming the field. The agent runs a program as programs.md, section 5
  says, in the commands' environment as programs.md, 4.2 changes it.
- **A profile:** its name, and the deployment's declared quantities,
  `charter_models` among them, and policy values (limits.md, sections 2.2
  and 2.3). A profile is a named
  bundle, kept as step code in the service crate, which the configuration
  names and may override value by value. Every byte limit is derived
  from it (limits.md, section 3); none is configuration. Tests may
  override a derived limit; nothing else may.
- **The commands' environment,** whole, and the graces of a group's three
  steps.
- **The event stream:** a file to append to, if any; its capture policy;
  its sink's policy, complete by default; and its `write_deadline`, the
  most one write may stay in flight (events.md, section 5; limits.md,
  2.3).
- **Checked at startup.** Every relationship between the limits is
  checked, by name, before the loop starts; a configuration that breaks
  one is refused, naming it (limits.md, section 4). The derived limits
  are also the agent's terms on the channel (channel.md, section 2).

Under a host, the host provides it, as temper's worker would with its
machines. The local host writes its agent's from its own settings when it
spawns one (host.md, section 5.2).

## 5. Facts and events

- **One drain.** Each pass, the service takes the domain's facts, up to
  its work quantum, whatever the channel's phase or room. Inside the
  process, facts are lossless: the domain's fact queue is a step's
  output in an uncapped queue (domain/run.md, section 10;
  programming-model.md, section 3). The next domain entrance waits until
  the previous entrance's requests and facts have been dispatched
  (limits.md, 3.8).
- **Two sinks, each with its own policy and its own count of loss:**
  - **the channel** projects facts, content-free, only while the run is
    admitted and has not answered, and only with room to spare. The rest
    it drops and counts as not projected; it never holds the run back
    (channel.md, sections 7 and 8);
  - **the event stream** turns every fact, with the protocol layer's and
    the shell's measurements, into records (events.md, section 4). It is
    complete by default: when its file is behind, the run waits for room.
    Under best effort it drops and counts instead. `session.started`,
    `run.completed` and `session.ended` have room reserved, and are never
    dropped (events.md, section 5).

  A fact taken by one sink is never taken from the other.
- **Written through io, as an append stream.** The shell opens the trace
  file for appending, beneath its root, and io adopts it as a write
  stream with the configured `write_deadline` (skein's `runtime/io/files.md`, 1). A
  destination that is a pipe or a redirected standard output is adopted
  the same way. Each record is encoded at its final length into the
  stream's output, within its cap. A write that outstays the deadline
  fails the stream once: the sink gives that destination up, counts
  every later record as lost, and says so on standard error (events.md,
  section 5.2). The stream is closed in the teardown, after
  `session.ended` (`agent.md`, section 6), under io's close deadline in place of the
  write deadline. There is no thread and no grace of its own.
- **The capture policy** decides what content the stream holds
  (events.md, section 5.3):
  - **none:** names, counts, sizes, times and classifications only;
  - **calls:** with each tool call's input, and a provider's failure
    text;
  - **everything:** with prompts, completions, results and messages.

  Credentials never appear in it.
- **Prompts are recorded as per-turn deltas.** Under `everything`, each
  request's record holds the messages the conversation appended since
  its previous request, and the system text and tools only when a
  conversation or a window begins (events.md, section 3.11). Every
  request is recorded, however many start in one pass, and a trace grows
  with the conversation, not with its square.
- **Loss is told.** `session.ended` gives each sink's count, so a reader
  knows whether its stream is complete without reading standard error.

## 6. The process

- **Streams:**
  - standard input and output carry the channel;
  - standard error is the agent's log for operators: a line when it
    starts, saying whether commands are contained (section 3); a line
    when it gives up a stream's destination (events.md, section 5.2);
    and one bounded line when it ends, saying why it ended without an
    answer, or, for a failed answer, its typed failure in words (the
    class, and what it names: the provider's class and the limit that
    fired, the budget that ran out) and the last failure's detail,
    clipped as an operator's line is (limits.md, 2.4; domain/run.md,
    section 9; llm.md, section 6). The words are rendered from the
    typed failure, matched exhaustively, never from a debug rendering.

  Program failures also drain to standard error as they arrive, even when
  capture is disabled and the main conversation later succeeds. Each line
  names `claude_code`, its process-local conversation owner token, and the
  exit code when available (`unknown` otherwise), followed by at most 512
  bytes of the already-redacted diagnostic tail, with controls escaped.
  The pending lines have the service queue's entry bound; a full queue
  holds further terminals until the shell drains it. These lines never
  enter canonical model feedback (domain/session.md, section 4).

  Its host keeps the tail of standard error and never parses it
  (domain/host.md, section 4).
- **Startup:**
  - read the configuration;
  - derive the limits and check every relationship (limits.md);
  - resolve the endpoints' names;
  - check whether the machine allows contained trees;
  - open the event stream's file, if any, and write `session.started`;
  - then start the loop, as the channel's responder.

  A startup that fails writes its reason to standard error, a refused
  relationship by name, and exits with failure without opening the
  channel. Its host reports that the agent could not be started.
- **The service:** `smith-agent-service`'s `iterate` composes `smith-domain`
  with the three components and the event sink (README.md, section 5).
- **The end is one phase,** matched exhaustively in one place, from which
  what work is pending and whether the process is done both follow:
  - **`Live`:** the run runs. A termination signal is the domain's
    cancel: the run winds down and answers, as it would on the host's
    cancel.
  - **`Answered`:** the answer is sent, and `run.completed` written. The
    service closes everything it owns at once: the LLM pool, whose idle
    connections close at once and whose busy ones close when their calls
    end; the machine's roots; and the channel, which it finishes.
  - **`Closing`:** every close is requested, and the service waits for
    each component to report it closed (programming-model.md, section
    5.2). When all have, it writes `session.ended` and closes the event
    stream.
  - **`Closed`:** everything it owns has reported closed and io is
    empty. The process exits.

  Only io's close deadlines bound `Closing`; no keep or idle time decides
  when the process ends. The components close together; then the shell
  reads the kernel's resource usage of the process and its reaped
  children through io, after the last child has closed; then the event
  stream writes `session.ended` and closes. Two of io's close deadlines in
  turn, the stream's write deadline for its last records, and the moment
  its cancels take to settle, are its **teardown bound** (limits.md,
  section 7; skein's `runtime/shell.md`, section 12).
- **A termination signal after the answer** turns every close still in
  progress into an abort: `Answered` and `Closing` end as soon as io has
  settled what it aborted. `session.ended` says the teardown aborted.
- **Exit evidence.** The process exits with success once it has answered,
  whatever the answer, and with failure only when it could not answer:
  startup, or a broken channel. Its host learns its `exit`, the code or
  the signal, and `forced`, whether the host had to terminate or kill it
  (domain/host.md, section 4). An agent that keeps this section's
  contract is never forced: a host's exit grace exceeds the agent's
  teardown bound (limits.md, section 4). A forced exit after an accepted
  answer is a warning for operators, not the run's failure.

### 6.1 Reading a list

The agent also reads an endpoint's list of models, when a host starts it to
(`host.md`, section 2). This is no run, and no record of the channel's: it has
no charter, turn, budget or event stream, and starts none of the components a
run has.

- **Its flag:** `--list`, in place of a run, with the same configuration. It
  reads the configuration and resolves the endpoints' names, as a startup does,
  and refuses as one does, on standard error, with failure.
- **Standard input** carries `ListAsk`, one record of `smith-channel`'s codec,
  and then ends: the endpoint, by the number a charter's LLM names it by; its
  credential, scheme and bytes, or none. This is the channel's grant value, kept
  from argv and the environment as a run's is, and goes nowhere but the request.
- **The request** is skein's (`skein/docs/design/llm/connection.md`, 4.3), under
  the endpoint's connect and handshake deadlines and a whole deadline the host
  bounds by stopping it.
- **Standard output** carries `ListAnswer`, one record, then ends: the rows of
  `skein_llm::list::Listed`, at most 256, or its failure, as
  `skein_llm_connection::list::ListAnswer` says it, with a status alone.
- **An endpoint** the configuration doesn't hold, a scheme its dialect doesn't
  take or an ask that doesn't read answers `Unreadable`, with no request. An
  endpoint that names a program is read as any other, with a key.
- **It exits** with success once it has answered, whatever the answer.

## 7. What the domain is owed, and what skein owes

- **The domain is owed** its operations carried out as asked, each answered
  once, with outcomes in its terms: a file's content and version, a
  command's exit and output, a job's output since its last read and its
  end, a group ended after a deadline.
- **skein owes:**
  - io's files and roots;
  - process groups, signalled whole (skein's `runtime/io/processes.md`), and
    contained trees (skein's `runtime/io/processes.md`, 2–4);
  - append streams on files, with write and close deadlines (skein's
    `runtime/io/files.md`, 1);
  - owner close, with a closed terminal, for every component the agent
    owns, the LLM connection pool's included (programming-model.md,
    section 5.2; skein's `llm/connection.md`);
  - the shell's startup, its loop and its end (skein's `runtime/shell.md`,
    sections 6, 11 and 12).

## 8. The world

- **The machine component,** over skein's simulator and the fake machine:
  - files changed behind a session's back;
  - symbolic links and `.git` parts on paths;
  - commands that hang, fail and flood their output;
  - commands whose children hold their pipes, ended with their group at
    the deadline;
  - a command past its yield held as a job: read while it runs, its
    output past its kept output's bound kept as head and tail with the
    count dropped, then read to its end; a job stopped; a job past its
    deadline; a job still held at the machine's close, stopped within
    one `close`;
  - the configured environment reaching commands and checks, and none
    reaching a search;
  - checks that pass, fail and time out.

  The referee requires that every group is ended before its call is
  answered, a job's before its end is answered or told, and every group
  before the machine reports closed; and, with contained trees, that
  nothing is written outside the writable directories.
- **The process,** as a simulated world: the agent's `iterate` with a
  scripted host on its channel and a fake LLM peer. Each world inherits
  skein's teardown invariant: a world that ends by itself fires no
  deadline after the last word but io's close and retry deadlines
  (testing-strategy.md, section 6). It covers:
  - startup refusals, each relationship by name;
  - a run from start to answer, then an exit with no signal from the
    host, at shipped keep and idle times;
  - a termination signal mid-turn, as a cancel;
  - a termination signal after the answer, with a peer that ignores the
    close, ending by abort before io's close deadline;
  - a host that hangs up mid-call: one terminal per call;
  - standard error's last line, for each way of ending without an answer
    and for a failed answer;
  - a host slow to acknowledge during a run full of facts: the channel's
    loss counted, the event stream's none, the answer unchanged;
  - an event stream slowed under `complete`, losing nothing, and under
    best effort, counting what it dropped;
  - three sessions starting completions in one pass, each recorded;
  - a run on a program backend, against the fake program (programs.md,
    section 8).
- **Configuration:** every refusal of a malformed one, and of each
  relationship of limits.md, section 4; an endpoint naming a program
  with no path; each field an endpoint does not take, `account` and
  `identity`, refused, naming it.
- **Memory:** fixed process scenarios against skein's counting
  allocator, one measured total peak under a ceiling with headroom per
  scenario (limits.md, section 11).

## 10. Open questions

- **A daemon:** listening for connections and deciding on a credential
  waits on the domain (README.md, section 11).
- **The network for commands:** whether a command's tree has a network,
  which a build may need and a review may not.
- **Traces kept:** their format is events.md's. Their rotation, and how
  long they are kept, remain open (events.md, section 10).
- **An environment per run:** a charter field that adds to the
  configured environment, merged in the machine under a stated
  precedence and refused at the run's entrance. Not until a host needs
  it.
- **A build profile:** cache directories writable in the view, tool roots
  read-only, and resource limits per tree, once contained trees exist.
