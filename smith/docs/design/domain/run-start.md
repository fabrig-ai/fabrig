# The run: starting a run

## 1. Starting a run

### 1.1 The charter

What the host gives a run to set it up:

- **Mode:** `Finite` or `Open`, stated by every creator. Open requires
  waiting authority and no delivery grant, otherwise admission refuses
  it as a malformed charter. Its bounded result contract remains present
  and is still validated, but is never offered or priced as a finish schema.
- **Instructions:** the role, as text, the first part of main's system
  text (1.3): a chat, a researcher, a coordinator, a reviewer with a
  lens, an operator, a producer of a change. A host gives its own;
  smith's shell gives the task role, one text for chats and for tasks,
  when its settings give none (`../shell/settings.md`, sections 1.2 and
  2.6).
- **Brief:** titled sections of text, in order, that the host wrote for
  why the run runs, already fitted to the host's budget for them. A host
  that keeps typed context renders it; smith reads only titles and text.
- **Tools:** the families the LLM may call (`run.md`, section 5): inspect, modify
  and shell, on the workspace, if it has one; sub-agents; `deliver`;
  `save`, only with `deliver` (`run-delivery.md`, 1.5); `wait`; and each host tool, declared
  (`run.md`, 5.2). Data, never derived from a
  role's name. A run may have no tools but `finish`.
- **Result contract:** what counts as done (`run.md`, section 7).
- **Conventions:** where a workspace directory keeps its guide and its
  checks (`run-delivery.md`, 1.1); smith's defaults when absent.
- **Budget:** what the whole run may spend across all its sessions, in
  the host's unit, with each model's prices; its wall time; its turns, a
  loop guard; and the reserve it keeps for finishing (`budget.md`).
- **LLMs:** the endpoint and model the main session runs on, and the
  others sub-agents may use, each with its prices, the window of input it
  may use and the most output a completion may have, as the host chose
  them within the agent's configuration (session-context.md, section 1). The
  agent is configured with endpoints; a charter only names them.
- **Waiting:** how long the run may wait for a message before it parks
  (`run-messages.md`, section 1), its first message included (1.4); zero parks as soon as it
  waits. A run is *wait-granted* when its charter's tools include `wait`,
  whatever its waiting time: a headless host grants `wait` with a waiting
  time of zero, so a run that would wait parks at once and its host
  reports that it needs input (host.md, section 8).
- **Resuming:** whether the main session opens from the transcript in
  the start, when there is one.

### 1.2 The start

What comes with the charter:

- **The workspace,** if any (tools.md, section 2): its directories, each
  with the name the LLM calls it (one safe path component), where it
  sits, whether it may be written and whether it is a git repository; and
  for a repository that starts from a merge in progress, the files the
  merge left in conflict (`run-delivery.md`, 1.3).
- **Messages,** if any: those that triggered the run, such as a person's
  first line or a headless run's prompt, each named and labelled like any
  message (`run-messages.md`, section 1). They enter the inbox when the start is admitted,
  before any message relayed after it, and a start refused at the
  entrance refuses them with it.
- **The run's identity:** opaque and of the fixed size the vocabulary
  seals (../protocol/limits.md, section 2.4), the same for every
  activation and resume of the run, which its host keeps with the run.
  A start without one takes the identity its transcript records (the key
  of main's affinity), so a minted identity is also the run's for its
  whole life. Only a start with neither, a run's first activation without
  a host identity, gets one the run draws from its seeded randomness as
  it admits the start. Every conversation of the run carries it, with an
  ordinal of its own (main's is zero), as its affinity (session.md,
  section 2), so a provider can route the run's requests to where their
  prefixes are cached; a main session resumed from a transcript keeps the
  affinity its transcript records. It is never derived from a credential
  or a path.
- **The transcript,** when the run resumes, and the calls its host
  answered after the last of its turns, with their answers (`run-messages.md`, section 1).
- **Credential grants** for the endpoints that need them (host.md,
  section 7).
- **An activation number,** new for every start of the run and never
  reused; every call's name carries it (`run.md`, 5.2).

More messages may follow at once, before the run has read its start.

### 1.3 The prompt

The run composes its main session's system text: the instructions, the
role (1.1), first; the brief's sections; then the sections about its
own mechanics, because those are what it enforces: the workspace, if
any, and what it may write; the tools; each directory's guide; the
checks; the budget (`## Budget`, `budget.md`, 2); sub-agents, when granted (`run.md`, 5.3);
how to finish in Finite (the contract, as the LLM must meet it), or how to
reply in Open; and how waiting works, when `wait` is granted. The domain decides what each says and in which order, and
writes it as text, as it writes the results and problems of the run's
own tools and the host's, and the notes it appends (`budget.md`, 2), so a
transcript keeps exactly what the LLM was told. The workspace tools'
outcomes stay typed (tools.md, section 4).

- **The working directory, named.** The workspace's section says
  concretely where paths start (tools.md, section 2). With one directory,
  it is the working directory: paths are relative to it, and its name
  appears in no path the prompt shows. With several, the first is the
  working directory, and the prompt says how each other is reached, as
  `/name/...`.
- **No workspace, no word of one.** A run with no workspace has neither
  the workspace's section nor the section on its tools that act on it:
  nothing it could read, write or run is named, so that its system text
  is its instructions, its brief and the mechanics it has (its budget,
  how it finishes or replies, and waiting), as a run that only talks or
  reads through host tools needs (the user, 2026-11-07).
- **Fixed for the conversation's life.** The system text is written once,
  when the run opens a session, and does not change while that
  conversation lives, across its windows (session.md, section 3, and session-context.md, section 1):
  it holds no clock and nothing else that moves. What moves is appended
  to the conversation as text the run wrote (a nudge, a sub-agent's usage
  line, the wind-down's note, a message), never placed in the system text
  or a tool's description, so each request extends the one before and a
  provider's cache keeps it. A session that resumes the conversation in a
  later activation has it written afresh, from the start that resumes it.
- **The summary instruction and its preamble.** With each session it
  opens, the run gives the instruction the session sends when it
  compacts its conversation into a new window, and the preamble that
  opens the summary in that window (session-context.md, section 1.2). The
  session decides when; the run owns the words, as it owns every
  prompt. The instruction must say:
  - that the work is not finished, and that a new window continues it,
    opening with the task's own words;
  - to answer with text and to call no tool;
  - to write what the next window needs that the task does not say: what
    was done and found, with exact values, names and paths as they were
    read; what remains; and the decisions taken;
  - not to restate the task or its required answer;
  - to keep within 1,000 words, since a summary is generated at the
    model's output rate.

  The preamble says that what follows is the model's own summary of an
  earlier window, and that the work goes on. Both are the same whatever
  the charter says, so they count in smith's own part of a window
  (../protocol/limits-derivation.md, section 1.9).

A guide is read from each workspace directory at the path the
conventions name, as text, UTF-8: whole within its share of the host's
part, at most the guide limit, which the product's presets size so that
a guide is normally whole (../protocol/limits-derivation.md, 1.9), and past it cut at
a character boundary, the prompt saying that the file goes on; a file
that is not text counts as no guide.

A sub-agent's system text is its brief, then the same mechanics for the
tools it has, with the same guides: `## Your share` in place of
`## Budget`, and how to answer, within its answer cap, in place of how
to finish (`run.md`, 5.3).

### 1.4 The opening

A run opens its main session once it has prepared and has work: a brief
that is not empty, or a message.

- **The opening** of a fresh conversation is main's first user message:
  the instruction to begin the work the brief describes, when the brief
  is not empty, then every message queued, in arrival order, at most one
  offer's worth (`run-messages.md`, section 1); the rest stay queued for main's next yield.
  An Open run's brief is its person's first words, not work handed to
  it, so its opening carries the brief itself, whole, as that first
  user message, before the messages queued, and its system text keeps
  its instructions alone, with no instruction to begin: a model that
  reads a person's request as its own system text plans the work
  itself where its instructions would have it delegate. A Finite run's
  opening, a sub-agent's and a resumed window's are as above.
  When concrete history is selected for resume, the waking prompt contains
  an instruction that starts a fresh bounded activation and supersedes prior
  exhaustion, followed by the offered messages without a new instruction to
  begin. With no messages, that activation instruction stands alone. It first includes
  any host answers absent from the transcript (`run-messages.md`, section 1). The session
  supplies this prompt as the result of a withheld `wait`, or as user text
  otherwise (`run-messages.md`, section 1). The first turn told names the last offered message
  as read (`run-messages.md`, section 1).
- **Awaiting.** A run with an empty brief, no message and `wait` granted
  opens nothing: no session, and no request to a provider. It tells its
  host it is waiting, and the first message opens main, as its opening.
  Its time awaiting counts against its wall time (`budget.md`), and it
  parks at its charter's waiting time like any waiting run, having taken
  no turn; its host starts it again with the message that wakes it
  (1.2). A run whose ordinary time runs out while it awaits parks then,
  as any waiting run does, having nothing to finish (`budget.md`, 1).
- **Instructions only.** A run with an empty brief, no message and no
  `wait` has only its instructions to go on, and opens on them, with the
  instruction to begin the work they describe.
