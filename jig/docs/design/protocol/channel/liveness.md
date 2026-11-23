# The channel: liveness and flow control

## 1. Liveness and flow control

- **Pings.** A side that has sent nothing for the ping interval sends
  skein's `Ping`. A link that has received nothing for the silence
  deadline, at least three intervals, is lost. Pings never reach a domain.
- **Silence counts bytes.** Each read from the stream in the open phase
  restarts the silence deadline, a piece of a frame included, on both
  sides: a large frame arriving slowly over a busy link keeps it.
- **Reading never waits for the root.** Each side's protocol layer
  reads its link and fires its component's deadlines on every pass,
  whatever its root has yet to take, so pings go out and come in under
  any load. Records its root is not ready for wait above the component,
  in the protocol layer's own entrance, never unread in the socket: that
  entrance holds what the peer's output cap lets it send (`sizes`'
  engine cap on a worker), so it never fills while the peer keeps its
  cap, and a record past it breaks the peer's contract and loses the
  link. A socket left unread starves both sides' pings, and the peer
  loses a healthy link at its silence deadline.
- **A lost link says why.** Each side's component tells its protocol
  layer, once per connection, why it ended, from a closed set: silence,
  stall, the output cap filled, a deadline of the opening, the stream
  ending or failing, a framing error, a refusal sent or received with its
  reason; on the engine's side also no room for the connection and
  stopping, on the worker's a dial that failed, a fault its root
  raised, and a turn busy past the busy turns it can keep to send again
  (`worker-link.md`, 1.1, 1.2). The application's protocol layer writes it as a
  line of its log naming the worker and the reason, so a link that drops
  is never silent on either side.
- **A stall deadline.** A link whose output has stayed queued, with no
  frame drained below, for the stall deadline is lost: its peer stopped
  reading (skein's `net/channel.md`, section 7).
- **Output caps from the domains' limits.** Each side's cap is the most
  its domain can have queued at once, so a record the domain sends is
  never refused at the channel's entrance:
  - engine to worker, per worker: for each slot, an assignment at its
    largest, its start's messages included, the messages its task's inbox
    may hold, an answer for each call a run may have in flight, a kept or
    busy notice for each turn it may have unacknowledged, an
    acknowledgement of its answer, a cancel, and a grant for each LLM
    account's source and each of the application's credentials; and a
    sources notice, and a check and a sign out per program source;
  - worker to engine: a hello at its largest; for each slot, the turns
    and the answer it may keep, each with the most refusals it may carry,
    and its calls in flight; a limited, a refused and an unauthorized
    notice per LLM account's source a run may hold; a logins notice, and
    a signed out per program source; and a reserve for facts.
- **Every frame fits the stream below.** skein's channel asks its stream
  for room for a whole frame (skein's `net/channel.md`, section 7), and a
  stream grants no room past its own output cap: io leaves a larger ask
  unanswered (skein's `runtime/lib-streams.md`, 1.1). Such a frame would wait below
  forever, and with it every frame behind it and the pings, which a side
  sends only while nothing is queued, until the peer's silence deadline
  lost the link with nothing said. So each side's component is given its
  stream's room, the most bytes it grants at once (io's output cap,
  `largest_room`), and no frame past it reaches the channel. Every kind
  a side sends has its largest frame within the room, checked when the
  component is made, with the channel's own `Open` and `Refuse`; the
  engine's assignment and job alone are exempt, since the domains'
  limits admit a start larger than any room (a whole charter, a resumed
  window, the calls committed since its last turn, its inbox and its
  grants, `README.md`, section 10).
- **A start past the room fails its run, never the link.** The engine's
  component measures each assignment and job before the channel takes
  it. One whose frame is past the room, or that the channel refuses at
  its entrance for anything but a full queue, is never sent: the
  component answers it itself, as it answers one that cannot encode
  (`records.md`, 2), refused as an invalid start whose words name the record, its
  frame's bytes and the bound it passed (`assignment of 1300000 bytes
  past the link's room of 1048576 bytes`). It also tells its protocol
  layer once, as `Unsent`, with the worker, the channel, the record's
  kind, its bytes and the bound, which the application writes as a line
  of its log. The link, its pings and the worker's other runs go on. A
  record of another kind that the channel refuses is told the same way
  and dropped; with every other kind within the room and the peer's
  terms checked at the opening (`README.md`, 4.1), only a broken contract reaches
  that.
- **A resumed start fits its room first.** The calls committed since a
  resumed run's last turn (`owners.md`, 1.1) are the one part of a start that grows
  with the run's work rather than with a bound of its own: a long run
  whose turn was never told, its program working an hour between two
  turns, carries as many as the core retains, each answer up to its
  bound, far past any room. Before an assignment is measured against the
  room, the engine's component fits them to the engine's own stream room,
  its `io.output`. A worker declaring a different room is refused at the
  authenticated opening (`README.md`, 4.1); its declaration never enlarges the fit's
  bound. It keeps every committed call,
  by its name and tool, in its order; each delivery's answer whole,
  since its evidence must decode and agree on the worker (`owners.md`, 1.1); and
  each host answer no longer than the words below, which a cut would not
  shorten. From the newest back, it keeps each host answer whole while the frame with it
  fits; that first one that does not fit and every older host answer are
  cut: the body becomes the fixed ASCII words `[this call ran; its answer
  of BYTES bytes was left out of this start]`, `BYTES` the cut body's
  length in plain decimal, and the answer keeps its error flag. So the
  run is told of every call it made, the newest answers as they were,
  and is asked none again: smith tells the LLM of each, as text in its
  waking prompt (smith's `domain/run.md`, section 6), and an older read
  it still needs it reads again. A program that resumes its own session
  is given the same words as user messages (smith's
  `protocol/programs-claude-code.md`, 1.8), so its session goes on from where its
  work stood. An assignment that fits whole, a job, and a start in the
  engine's own hub, which crosses no link, are never cut. Only an
  assignment still past the room with every host answer cut is refused
  as above, its words naming the frame so cut. The cut is the link's:
  the core keeps every call as committed, and retires them at the run's
  next turn as any.
- **Inventories coalesce while output waits.** The worker's component
  keeps one latest whole snapshot outside the channel queue and sends it
  when that queue is empty. A replacement snapshot overwrites an unsent
  predecessor: fleet has already received that predecessor's first inventory
  before it sends the replacement list. The held snapshot is priced.
- **Facts take only the reserve.** The worker's side sends a fact only
  while the queue leaves the rest of the cap free; a long operation's
  beginning and end are facts too.
- **A link that fills its cap anyway** is lost: with caps sized so, only a
  peer that stopped reading can fill one, and the stall deadline closes
  it in any case.
- **Both directions at once:** skein's channel writes while it reads
  (skein's `net/channel.md`, section 7), on one socket.
