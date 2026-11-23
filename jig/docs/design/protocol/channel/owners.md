# The channel: bytes of other owners

## 1. Bytes of other owners

Calls are refused at the worker component before encoding when their input
exceeds its configured call-input bound. The hub receives one terminal
answer under the original run, attempt and opaque call name: an error with
ASCII `call too large`, shortened to the answer bound (empty at zero).
Other call encoding failures return `call refused` under the same rule.
The component never copies or truncates the refused input, sends no call
frame and retains no replay entry. The agent can then continue its run.

### 1.1 smith's charter, turns and results

smith's payloads are bytes in the hub and typed in the agent; the root
that composes the two hands each out to its protocol layer's routing, to
be decoded or encoded with smith's codecs
(`../../domain/hosts/hub.md`, 8). jig's channel carries the bytes the hub
holds, and neither of its components reads them.

- **The charter** is put into smith's form by jig's one translation,
  `jig-protocol-charter` (`../../domain/hosts/charters.md`), in the
  engine's protocol layer, and crosses as smith's charter bytes (smith's
  `protocol/charter.md`). The worker's routing decodes it as the agent
  starts, against the endpoints of the worker's agent configuration,
  which resolve the names it uses (`../configuration.md`, section 4;
  smith's `protocol/charter.md`, section 7). The run's provider bindings supply its account/source numbers; configured
  endpoints carry no account preference or default.
- **Turns** cross as smith's turn bytes (smith's
  `protocol/transcript.md`), encoded on the worker as the agent hands
  each up, before the hub keeps it. The record carries beside them what
  the core reads: the turn's number, size, spend, the last message it
  read and whether it opens a window; the core keeps the bytes as they
  came. Down, a resumed run's transcript is its current window's turns,
  in order, within the resume limit, decoded on the worker as the agent
  starts; earlier windows' turns are never sent (`../../domain/engine/executors-transcripts.md`,
  1).
- **A window crosses as one piece,** in smith's form: each turn's bytes
  after their length, a `u32`, as smith's channel carries a start's
  transcript (smith's `protocol/channel.md`, section 3; skein's
  `data/codec/README.md`, section 3, "A sum crosses as one piece"). The resume limit
  bounds the piece, each turn counted with the four bytes of its length,
  so one field's bound holds any window the domain may resume, many
  small turns or a few large; the domain bounds a window's bytes, not
  its turns, and smith's resume limit holds any window one of its
  sessions makes. `jig-protocol-engine-workers` joins the core's turns
  into the piece as it encodes an assignment; the worker's link splits
  it back into the hub's turns as it decodes, and a piece whose lengths
  do not end exactly at its end is a transcript that does not decode
  (below).
- **A run's result** is encoded on the worker as smith's result, in its
  charter's version, before the hub keeps the answer.
  `jig-protocol-charter`'s reader decodes it in the engine's protocol
  layer, for runs on workers and in the engine alike, by the path of
  host tools' calls (1.3): the core hands an answer's result out once
  the fleet has fenced it, and the typed result comes back. A result
  that does not decode fails its attempt as invalid.
- **An assignment whose charter, transcript or a committed call's name
  does not decode** is refused as invalid by the worker's root, naming
  which, before the agent is started (`../../domain/hosts/hub.md`, 8). One
  refused for its transcript is the engine's to start afresh
  (`../../domain/engine/executors-transcripts.md`, 1).
- **Calls committed since a resumed run's last turn** cross with their
  answers as the engine kept them, so the run is told of them, not asked
  them again (`../../domain/engine/executors-tools.md`, 1).
- **In the engine's own hub,** nothing crosses this channel: the engine's
  root hands its hub's payloads out the same way, so the core holds the
  same bytes from either kind of host.

### 1.2 The application's parts

- **Workspace items, what a run left, what it saved, and a delivery's
  evidence** are the application's: its connectors name them and its
  workspace makes them (`../../domain/hosts/README.md`, section 6). Each is a field
  of bytes in jig's records, encoded and decoded by the application's own
  codecs in its protocol layers (`README.md`, section 3).
- **Versioned by the application.** Its encodings are its wires, which
  the opening states and checks beside jig's (`wires.md`, 2 and 3);
  another layout is refused before any part crosses.
- **A delivery's evidence is the application's to keep:** the core
  keeps a settled delivery's outcome, never its evidence; the
  application keeps the evidence in its own records, by the call's key,
  and its routing puts it in the assignment's answered call as it sends
  one (`../store/records.md`, 1).

### 1.3 Host tools' calls

- **A call crosses as the run wrote it:** the call's name, as the bytes
  `README.md`, section 6 gives it, the tool's name as the charter declared it, whether
  it writes, and its input, the JSON the LLM wrote, checked by smith only
  to be an object within its bound (smith's `domain/run.md`, 5.2).
- **The fleet fences it first,** as any call: one from an attempt that is
  not its task's live one is dropped. The core then hands the admitted
  call out for decoding, and the engine's protocol layer decodes its input
  with the decoder written beside the tool's schema: `jig-protocol-tools`'
  for the core's own tools, the application's for its connectors' reads
  and effects (`../../domain/hosts/charters.md`). The typed request goes
  back to the root; an input that does not decode is answered at once as
  an error naming what is wrong, and decides nothing.
- **Calls from runs in the engine** take the same path: the engine's own
  hub relays them to the fleet, and the protocol layer decodes them, so no
  domain parses a call's input.
- **An answer crosses as the engine's text** for the LLM, with whether it
  is an error, rendered in the engine's protocol layer, by the same crates,
  from the core's part and the connector's (`../../domain/root.md`,
  section 7).
- **Claude Code's built-in tools run on the worker,** as smith's own
  workspace tools do, and never cross this channel as calls. On a
  program source, the run's agent starts Claude Code with an ask rule
  for each built-in tool of the families the charter grants,
  `--setting-sources user` with the source's configuration directory,
  and the agent's own MCP server as its permission prompt tool. So every
  built-in call asks and reaches smith's policy before it runs. The
  prompt tool answers allow, with the call's input unchanged, or deny,
  with a message, within the run's families, as a call of smith's own
  tools is allowed or denied; the agent never changes a call's input, so
  the host sees the call that runs, in the run's turns and activity, as
  it sees a direct backend's (smith's `protocol/programs-claude-code.md`, 1.3 and
  1.4; `../../domain/authority-checks.md`, 1.5). What stays outside the agent's
  reach, the host's managed settings among it, is smith's
  `protocol/programs.md`, section 7's.
- **Host tools on a program source** are offered to the program by the
  agent's own MCP server, under the names the charter declares, and the
  agent relays each call to the host. Such a call crosses as this
  section says, the same record whichever backend the run has, and its
  answer goes back the same way.

### 1.4 Facts, and a run's text

- **Facts are best effort** (`../../domain/hosts/hub.md`, 5): the worker's side
  sends one only while its output queue leaves room for everything else
  (`liveness.md`, section 1), and drops it, counted, otherwise.
- **Activity** is the fleet's typed report, which every host gives, and
  which carries no content: a run's phase; its long operations (checks,
  an LLM completion or a command), each begun with its span and ended
  (smith's `domain/host.md`, section 2); and counts from smith's channel
  facts, each its kind, its time since the activation's start and its
  count (smith's `protocol/channel.md`, section 8), decoded from smith's
  facts by the worker's hand-out (`../../domain/hosts/hub.md`, 8). The fact
  record carries jig's activity, never smith's bytes, and the views show
  a run by it.
- **A run's text comes from any worker, when the assignment asks.** The
  fact record carries a piece of live content with its kind, from a
  worker whose agents are processes as from one whose hub composes
  `smith-inline-agent`: in a process, smith's channel carries it, decoded
  by the worker's hand-out into jig's typed fact (`../../domain/hosts/hub.md`,
  8; smith's `protocol/channel.md`, section 4). The wire carries all
  three kinds, so that reasoning and a tool's input need no new version;
  smith produces `text` and `input` from agent processes. A worker never sends a piece of a kind
  its agents do not build. One that arrives crosses the engine's side as
  it is, every field translated and nothing decided, and the engine's
  root routes text and input to the core, dropping reasoning and offsets
  past `u32`, counted
  (`../../domain/engine/views.md`).
- **A tool begun with its call comes only from a host that gives it:** a
  tool begun with its call's name, its tool and its input within the
  views' bound. smith drains it only in the host's process, so a hub
  whose agent capability is `smith-inline-agent` gives it (`../llm.md`,
  section 3), and this channel carries it from such a worker in the fact
  record. A worker whose agents are processes gives none; the views
  show its runs by their activity and their text.
- **A piece** crosses with its turn, retry, kind and offset as
  `../../domain/hosts/hub.md`, 5 names them, its offset smith's 64 bits, and
  its bytes within the sizes' bound on a piece (`README.md`, section 10); the views
  keep it (`../../domain/engine/views.md`), and the engine's root
  drops, counted, one whose offset is past what they count. It keeps the
  order that section gives: pieces of a turn before its turn record,
  none after.
- **The ask is not a flow-control matter.** A worker whose assignment
  asks for nothing sends no piece; one that asks is still best effort,
  by the rule of this section's first item.
