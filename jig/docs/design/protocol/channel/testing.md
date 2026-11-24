# The channel: testing

## 1. Testing

Following skein/docs/foundation/testing-strategy.md, and skein's `net/channel.md`, section 11, each
within jig's suites' budgets: focused tests in the default suite, randomized
ones in the fuzzy suite.

- **Records' drift** in `jig-worker-wire-world`: the generated records
  match the committed module,
  as skein's `data/codec/golden.md`, section 1 has it. No generated tests
  or records' golden bytes are kept. Step tests in `jig-protocol-worker-wire`
  check the sizes against bodies measured by hand; patch goldens stay.
- **A machine world per component:** the component between the hub's
  scripted vocabulary and a scripted peer of skein's channel, with every
  refusal of the opening, a worker replaced, a hello not admitted,
  silence, stall, the hello late, grants filled and dropped, of both
  owners and of each kind of value (a bearer, a key, a program source's
  name), sources, checks, logins and sign-outs, relays across channels,
  the ask and a piece of each kind, every field both ways, a piece
  dropped for room counted with the facts, and the grace passing and
  not. smith's
  payloads are opaque bytes there, as they are to the components.
  The engine's world also loses one worker fifty times, each way a link
  is lost, and takes it back each time, with its connections never past
  their room; aborts a closing connection whose channel never ends at
  its deadline; and names each loss's `Loss`. The worker's world keeps a
  link whose frame arrives in pieces past the silence deadline, and
  names each loss's `Loss`; joined to the real engine's component, a
  link busy both ways for minutes, its queue never empty, is never
  lost. The worker link's incompatible-terms and malformed-body fixtures
  assert the literal wire reasons 2 and 3, exactly one loss, and no
  forwarded body. Its trace fixture admits a malformed `TurnBusy` to
  the real decoder, which never decodes it as a valid busy before the
  component sees it.
- **What the machine worlds' fixtures prepare once.** A value the two
  worlds' fixtures and harness derive from constants alone is built the
  first time a test reads it, kept in a `std::sync::OnceLock`, and
  copied to each reader: the channel's schema and limits, jig's wires
  as `jig_protocol_worker_wire::spoken` states them, the wires a
  worker's statement carries, and the derived layouts a world's peer at
  another layout states (`layouts.md`, 8). Such a value is a constant
  (`skein/docs/foundation/testing-strategy.md`, 8): nothing a test sets
  reaches it, and each reader changes only its own copy. Each peer
  still takes its own wire and its credential's encoding, and
  every connection, loss and assertion runs as before: the fifty-loss
  row makes each of its peers, but parses the texts once, not for
  each. One focused test in each world checks every prepared value
  against one built afresh.
- **A frame past the room, in the machine worlds,** their scripted
  streams granting room up to a cap and leaving a larger ask unanswered,
  as io does. The engine's world, one table: an assignment and a job at
  the room are sent; one byte past it, each is answered refused with its
  bytes and the room, told once as `Unsent`, and a ping still goes out
  at each interval for two minutes, with the next record sent after it.
  The engine's component's step tests: limits whose largest message,
  relayed answer or grant frame passes the room are not made, a job that
  cannot encode is answered, and a record the channel refuses at its
  entrance is answered once. The worker's world: limits whose largest turn,
  call, answer or hello frame passes the room are not made. Joined to
  the real engine's component, an assignment past the engine's room
  leaves the link up for two minutes, pings crossing both ways, its run
  answered refused, and a run assigned after it is hosted.
- **A resumed start fitted to its room.** In `jig-protocol-worker-wire`,
  one step table: an assignment that fits is left whole; one past the
  room by its committed calls has its newest answers whole and the
  older cut, its frame within the room, each cut answer's words naming
  its bytes, its error flag and every call's name and tool kept; a
  delivery among them whole; the room met exactly; one still past the
  room with every host answer cut, left so; and the smallest resumed
  start against an assignment built at it, measured. The engine's
  world: limits whose smallest resumed start passes the room are not
  made, and an assignment past the room by its committed calls is sent
  cut, with no `Unsent`. Joined to the real engine's component, on each
  stock profile's limits: a resumed start carrying more committed calls
  at the largest answer than one start can carry crosses, the worker's
  root is handed every call, the newest answers whole and the rest cut,
  its run's turn and answer go up and are kept, and the link lives. The
  core's choice of the calls a start carries, every one committed since
  the last turn in commit order, is the core world's (`../../domain/engine/executors-transcripts.md`,
  1).
- **The channel's protocol world,** `tests/channel`: the engine's side
  with the core under jig's test root, and the worker's side with the hub
  under the test worker's root, both as the reference service composes
  them (`../service.md`, section 7), with `testing/jig-test-worker`'s smith's agent
  processes over skein's fake LLM and scripted workspace beneath, joined
  by in-memory
  streams cut and rejoined at random; the scripted agents tell real
  turns and results, so smith's payloads cross the hand-out and
  `jig-protocol-charter`. It runs the hub's world's stories
  (`../../domain/hosts/README.md`, section 9) through bytes, and adds:
  - connections cut mid-frame, and the worker redialling;
  - a worker cut fifty times, each redial taken back, no run hosted;
  - a peer that stops reading, and one that goes quiet;
  - a partition each side notices at a different time;
  - garbage frames, a wrong secret, another version, another application
    version, terms that do not fit;
  - a worker that dials again while its old channel looks alive;
  - the engine restarting while workers keep runs;
  - an assignment carrying its task's inbox, a message refused for each
    of smith's reasons and committed with the next turn or the answer,
    the engine restarting between, and an answer leaving messages unread;
  - a run resumed after a compaction, sent only its current window, with
    its conversation's identity;
  - an assignment whose charter or transcript does not decode;
  - smith's facts reaching the views as activity;
  - a conversation's reply, the fake LLM's text in several held pieces,
    reaching the views from its agent process as the run's text before
    its turn: each piece's prefix seen by the run's watcher before the
    next hold is released, then the turn committed. A task the
    conversation delegates is not asked and gives no text; no story
    here runs a delegate, so the lower worlds check it: the core world
    its ask, the host world the hub's fence, the reference worker's
    step tests that an unasked run hands up no piece;
  - a run on a program source, its agent process running smith's fake
    `claude` (smith's `protocol/programs.md`, section 8) with the
    source's configuration directory: the login reported usable, not
    signed in, then signed in again after a check; a usage limit
    relayed as limited, with its reset and without; a login lost
    mid-run relayed as unauthorized; and the source signed out, one
    worker lost before it answers;
  - runs on a bearer and on a key, a key replaced mid-run, and each
    notice reaching the core once per run that sent it.

  Its referee is the hub's and the fleet's: every turn, and every answer
  of a run the host started, reaches the engine once and is acknowledged
  once; a refused start's answer reaches it at most once and is never
  acknowledged, and the start ends once, refused or lost (`records.md`, 1;
  `../../domain/engine/fleet.md`); every relayed call is answered
  once, every message sent ends once (read, refused or unread), no
  attempt is placed while a worker may still run the last, no run on a
  program source is placed on a host whose login was not last reported
  usable, no run is placed on a worker before its first `logins` after
  its hello, no run is placed on a host that keeps a person's logins
  unless that person's accounts pay for it, and no credential grant's
  value appears in any domain's state. Once a grant exists, the focused
  suite looks for its value when the wire's grant count changes and at
  each host's next root step. It also checks before the referee replaces
  a drained engine's service and checks every surviving host at the story's
  end. The fuzzy stories check at each root step: drawn `Chat`, `Progress`,
  `Call`, `HeldCommit`, `MidFrame` and `Restart`, and the replaced-key and
  application-generation stories. Other stories keep the focused checks.
- **The application's worlds** keep running its hosts without bytes
  (`../../domain/testing.md`), on the domain's vocabulary.
- **Fuzzing,** in the fuzzy suite: each record's decoder and the
  credential's.
