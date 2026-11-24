# The worker's figures

Provisional, 2026-11-16. What an engine sends each of its workers at the
opening, so that a worker keeps no copy of its engine's figures: the
places it's admitted at, the limits by name, and its agents' profile and
endpoints. It carries temper's `draft/config.md`, 6.3, for
jig's channel. The record and the two openings are jig's. What an
application puts in the record, and how its worker builds from it, is
the application's (temper's `protocol/configuration/figures.md`).

## 1. In one page

- **The engine's answer carries them.** First of what it sends once
  the channel is ready, the engine sends `figures` (2): the places it
  admits that worker at, every limit by its name, and its agents' profile
  and endpoints, converted to the worker's layout as any record
  (`layouts.md`, 4.1). A worker builds its host from them, not from its
  own document.
- **A worker with no figures opens for them alone** (4). Its credential
  says `opening: figures`. The engine answers with `figures` and closes
  the channel. Nothing is admitted, and the fleet
  hears nothing. The worker then builds, and opens its link as today,
  saying `opening: link`.
- **At every link opening, and when they change.** The engine sends
  `figures` first at each link opening, and to every linked
  worker whenever its owner gives new ones (3). A running worker tells
  its owner each one it hears. The owner applies the live ones at once,
  through `retime` for the link's timers (4), and the rest at its next
  start (temper's `protocol/configuration/figures.md`, 3.2).
- **Live timers on both sides.** The link's handshake, stall, backoff,
  ceiling, proved and busy timers, and the engine host's handshake,
  hello and stall, are read at each use, so each side takes new ones
  while it runs (3, 4). Its sizes, room, chunk, ping and silence are
  fixed for a link's life.
- **The owner gives them, as data.** jig's engine host sends what its
  owner last gave it. Until it's been given figures, it refuses a figures
  opening as *starting*.
- **The opening's own records** are laid out alike in every build (5).
- **A worker part's `agent` and `limits` may be absent**
  (`../configuration.md`, section 4). jig's reference binary still
  requires them (6).

## 2. The record

`jig-worker-wire` holds:

```
enum Opening { link, figures }
record Credential { name: text 256 secret: secret 1024 wires: list 16 Wire output: u32 opening: Opening }

record Limit { name: text 64 value: u64 }
record Header { name: text H value: text V }
record Tls { server_name: text S trust: option text T }
enum Transport { tls: Tls, plaintext }
enum Oversized { fail, drop }
record EndpointLimits { window: u32 output: u32 reasoning_item: u32 head: duration idle: duration oversized: Oversized }
record AgentEndpoint {
    name: text N number: u32 dialect: u32 provider: text N program: option text N
    host: text A port: u16 transport: Transport headers: list 16 Header
    connect: duration handshake: duration limits: EndpointLimits
}
record Agents { profile: list 128 Limit endpoints: list 64 AgentEndpoint }
record Figures { places: u32 limits: list 1024 Limit agents: Agents }
```

`Credential` gains `opening`, its last member. `FromEngine` gains
`figures: Figures` after `restarting`, so its kind is `0x0180` plus that tag.
Each text bound written as a letter above is at least the bound jig's
configuration reader gives the member it carries
(`jig/crates/jig-protocol-config/src/llm.rs`). The landing that adds
them picks each one, and the schema then fixes it.

- **`places`** is the most runs the engine admits this worker to at
  once (3).
- **`limits`** is every limit the engine holds, each by its dotted name
  as a configuration's overrides name it (`../limits.md`), jig's and the
  application's alike, at its figure in force: the value the engine
  started with, or one its owner took at once since (temper's
  `protocol/configuration/limit-settings.md`, 4). jig gives the names no
  order. A worker applies the names it knows over its
  stock: jig's through jig's override reader, the application's through
  its own. It skips a name it doesn't know, and logs it once. A limit
  the list doesn't name stays at the worker's stock.
- **`agents.profile`** is smith's profile figures, each by its dotted
  name over smith's `standard` profile: `declared.NAME` or
  `policy.NAME`, as smith's agent configuration's `profile` member
  names them (smith's `protocol/agent.md`, section 4).
- **`agents.endpoints`** is the engine's endpoints, in the engine's
  order (`../configuration-engine.md`, section 1, `llm`). Each holds what jig's
  `Endpoint` holds, and the limits the worker's agents take on it,
  smith's endpoint `limits`. `host` and `port` are the endpoint's
  address. `connect`, `handshake`, `head` and `idle` are durations.

## 3. The engine's side

- **A figures opening** is checked as any opening, in order
  (`README.md`, 4.1): the name and the secret, then the wires. Its
  `io.output` isn't compared with the engine's stream room: a worker
  that fetches its figures states its stock room, before it knows the
  engine's limits, and what the engine sends it is bounded by the
  schema's ceilings, not by `io.output`. It takes no channel's place,
  so it's never *busy* and replaces nothing. Its terms are judged for
  `figures` alone, the only kind it sends. The engine sends
  `figures` with that worker's places, then finishes the channel and
  closes it once drained. No hello is awaited, and the fleet hears
  nothing. The engine's log says `worker NAME took its figures`.
- ***starting*** is jig's reason 262, after *stopping*: the host has no
  figures from its owner yet. The worker redials after its pause, as
  after any refusal.
- **A link opening** is as today, with `figures` sent first once the
  channel is ready, when the host has figures. Then the hello is awaited
  as before (`README.md`, 4.3).
- **New figures from the owner** go to every channel that is ready, and
  every later opening sends them.
- **Places** are what the fleet admits a worker at whatever its hello's
  slots: today the fleet's per-worker `slots` limit, one figure for
  every worker (`../../domain/engine/fleet.md`). Workers made settings
  give each worker its own (temper's `draft/config.md`,
  6.7). The hello's admission is unchanged: the smaller of the hello's
  slots and that figure.
- **The owner gives the figures whole,** as data, each time they change:
  an input of the workers host carrying the generated `Figures` record,
  whose `places` the host sets for each worker as it sends. There's no
  callback. A host given none sends none at a link opening, and refuses
  a figures opening as *starting*.
- **The host's own timers, live.** Its owner may give it new handshake,
  hello and stall timers while it runs:
  `host::Workers::retime(&mut self, timers: Timers) -> bool`, with
  `Timers { handshake: Duration, hello: Duration, stall: Duration }`,
  an input of its component as `restarting` is. A deadline already set
  keeps its time; each one set after it takes the new figure: a
  connection's handshake, an opening's wait for its hello, a stall. It
  is false, and the host as it was, when the host's limits with them
  over them fail its limits' check (`valid_with_schema`), such as a zero
  figure. Its sizes, room, chunk, ping and silence stay as it was built.

## 4. The worker's side

- **The link's figures form.** `jig_protocol_worker_link`'s component
  can be made to open for figures alone: its credential says
  `opening: figures`, and its table and terms come from the link's own
  stock limits. Since `Figures`' bounds are the schema's literals, any
  table takes it. Once it hears `figures`, it reports them to its owner
  and is done: it closes and doesn't redial. A refusal, a loss or a
  deadline before then is reported as a link's is, and it redials after
  its pause, as a link does.
- **The link form** reports each `figures` it hears to its owner, at
  each opening and whenever they come. It goes on as today; the owner
  compares them with the figures in force on it.
- **Its timers, live.** Its owner may give a running link new timers:
  `Component::retime(&mut self, timers: Timers) -> bool`, with `Timers {
  handshake: Duration, stall: Duration, backoff: Duration, ceiling:
  Duration, proved: Duration, busy: Duration }`. A deadline already set
  keeps its time; each one set after it takes the new figure: the next
  dial's handshake, the next stall, the next proving. Its next redial
  wait is the new backoff's, within the new ceiling, and its next span
  of *busy* refusals the new busy's. It is false, and the link as it
  was, when its limits with them over them fail its limits' check
  (`valid_with_schema`): a zero figure, or a ceiling below the backoff.
  Its sizes, room, chunk, ping, silence and grace stay as it was built:
  they make its table and the stop bound its hello states.
- **The hello is unchanged.** It still states the worker's slots,
  stop bound and bounds, which the worker took from its figures.

## 5. Its layout

- **`Opening` is the credential's,** laid out alike in every build
  (`wires.md`, 2), so an engine reads what a worker opens for before it
  knows the worker's layouts. `Figures` is a record as any other,
  converted to the worker's layout (`layouts.md`, 4.1).

## 6. jig's reference binary

- **Its engine gives figures:** its places, and its configuration's
  limits by name, with no agents. Its worker opens as a link alone and
  ignores the figures it hears. It keeps reading its document's `agent`
  and `limits`, and refuses a worker part without them, naming the
  member.

## 7. Testing

Focused, in the link's and the host's tests and jig's channel world:

- **A figures opening:** a worker opening with `opening: figures` hears
  `figures` with its places and the owner's limits and
  agents, and the channel's end. The fleet hears no channel, and a live
  link under that worker's name stays.
- **Before figures:** a figures opening is refused as *starting*; the
  worker redials, and takes them once the owner gives them.
- **A link opening** hears `figures` first, then sends its hello. New
  figures given to the host reach every linked worker.
- **Refusals:** a wrong secret is refused *unauthorized*.
- **Timers, live:** a running link given a longer handshake by `retime`
  keeps the deadline of the dial in flight and gives its next dial the
  new one; a longer backoff and ceiling make its next redial wait the
  new backoff's; a stall and a busy span take the new figures at their
  next use; a zero figure, or a ceiling below the backoff, is false and
  changes nothing. On the engine's side, a host given a longer hello by
  `retime` waits that long for the next opening's hello, and one already
  waiting keeps its deadline; a zero figure is false.
- **The record:** generated records' drift, as
  skein's `data/codec/golden.md`, section 1 has it; patch `0006`'s
  breaking declaration, held by the wire's patch tests and
  `fabrig-boundary`'s `patches` test.

## 8. Open questions

- **Ping and silence live.** A running link keeps the ping and silence
  it was built with, since the stop bound its hello states follows them
  and the engine checks it against its grace. Default: they stay, until
  a worker can state a new stop bound without a new opening.
