# The engine's and the worker's service parts

## 1. The service parts

What every application's protocol layer writes alike, jig ships as
product code, which the reference service and an application's service
both compose. A fix in a part reaches every application, and one ordering
of token refreshes holds everywhere (`jig/AGENTS.md`). An application keeps
its routing, `iterate` and `main`, its connectors and its own settings.

- **A part is a host,** step code a service owns as a field and drives
  from `iterate`. It holds no root type, and no trait, generic item or
  callback (skein/docs/foundation/programming-model.md, 10.3). `jig_protocol_store::host::Drivers`
  is the form.
- **Values in, typed outputs out.** A part takes values at `new`. What it
  can't decide it hands its owner as a typed output, which the owner pulls
  in its own pass and routes: tagging tokens from its own table, pushing
  onto its own root queue, calling its own store. A pass hands out in the
  order it holds, so the owner's root sees events in the part's order.
- **A difference is the owner's.** Where two applications act differently
  on one fact, the part reports the fact and each owner acts; it takes no
  setting that names an application. A difference of amount is a value.
- **What two services have in common is each one's composition.** An
  application's pass, adapters, tokens and sizes are what this section
  leaves it, so the lines two services share there are no copy for a part
  to take.
- **A part that holds credentials shows none in `Debug`:** it implements
  no `Debug`, or a hand-written one that counts them.

| Part | Crate, module | It holds | The application keeps |
|---|---|---|---|
| the accounts' host | `jig-protocol-accounts`, `host` | the accounts' component and its provider templates, the pass over the store's answers and the component's effects, the table's grants | its tokens, its store's calls, its root's event, what an invalid restored secret means |
| the store's host | `jig-protocol-store`, `host` | the component and the file drivers behind one owner's calls: requests, the secrets' answers in the same pass, closing, submissions by lane | its routes, its route's table and answers, its sizes and wires, its tokens |
| the program sources | `jig-protocol-worker-agents`, `sources` | the source's command, one at a time, and the program's environment, hidden from `Debug` | how settings name a program |
| the worker route | `jig-protocol-worker-agents`, `route` | smith's host half's records to and from the channel's values | its application's decoding of parts, what a delivery's evidence must say (`channel/README.md`) |
| the workers' side | `jig-protocol-engine-workers`, `host` | the channel component, its listeners and sockets, closing them | its assignment's encoding, its parts' decoding, what a lost listener means |
| the server | `jig-protocol-engine-clients-http`, `server`, `bundle` | one listener's connections, their deadlines and page policy, each call handed out as routed; what serving a bundle has in common | its routes, its bundle, its pages' extra fields, its answers' policy |
| the clients' side | `jig-protocol-engine-clients`, `host` | the bridge between the component's secrets and the store's and sign-in endings; beside them, the printed link in `jig-protocol-engine-clients-http` and the settings every application fills alike in `jig_protocol_config::clients` | its pass, its calls, its families |

`jig-protocol-worker-agents` links smith, so an application whose workers
run no agents leaves it out, as `components.md`, 1 says. The kernel's hand-off
between a process's owners of records is skein's, `kernel::Routes`
(skein's `runtime/kernel.md`, 5).

### 1.1 The accounts' host

`jig_protocol_accounts::host::Accounts` is the accounts' component with
its effects and its io, and one pass over them. The engine's accounts'
component alone refreshes a record (`../domain/accounts.md`, section 2),
so this order is jig's: the owner's `iterate` calls the host, and holds
no step of the order but its own store's and root's. The host takes no
store: it hands out its store requests, and the owner calls its own store
and feeds the answer back.

- **At `new`:** the application's name, jig's part of the configuration,
  the crypto seed, the drawn installation identifier, the secrets' store's
  limits and a `Runtime`, which holds each provider's endpoint and
  origins, prepared by the shell, and `trust`, the bounds of the trust
  roots it prepared. With bounds, a provider on TLS is refused if either
  is zero; with `None` the shell answers for its roots and the host checks
  nothing. `jig-protocol-accounts` links `jig-protocol-config` for this,
  which links nothing back. The files a shell reads for a configuration
  are `jig_protocol_config::Files`; this host reads none.
- **A pass.** `begin` takes the iteration's time, whether the store's
  start mark is durable, the installation, and the secrets' answers the
  owner holds for the host. In this order, the component hears `Opened`,
  once; it steps with each answer; it fires. `begin` returns the pass's
  count: the effects waiting then, and the held event if there is one.
  The owner calls `output` that many times, each taking one effect in the
  component's order, and ends the pass with the table's `expire`.
- **What stays in.** `output` takes the owner's table, and does itself
  what needs no owner: an io request joins the host's `io`, a grant or a
  removal changes the table, a source's release ends there. It returns
  `None` for those, so a grant is in the table before the event that
  announces it leaves. Three things come out as `Output`:
  - `Account`, an event for the root: the owner wraps it and pushes it;
  - `Store`, a request on the host's exchange token: the owner tags the
    token and calls its store;
  - `InvalidSecret`, a restored secret of the wrong kind: what that means
    is the owner's.
- **What the owner keeps.** It routes each output before it asks for the
  next, so its root, its store and the table see the component's order.
  A store that answers within the pass (a read does) goes to `answer` at
  once. An effect that any step makes after `begin` has counted waits for
  the next pass.
- **Tokens.** The host's exchange tokens and io owners sit below the top
  four bits (skein's `runtime/lib.md`, 4). The owner tags `io`'s requests and
  `Store`'s exchange in its own namespace, and hands io's events back
  untagged. `answer` and `begin` take a store answer's namespace off.
- **Room.** The queue to a root is uncapped (`service.md`, section 2). An owner whose
  queue may be full gives an `Account` back with `hold` and ends the
  loop. The host keeps it in one slot: it leaves first in the next pass,
  and the host counts as work until it has. The rest of the count waits
  with it, and the next `begin` counts them again.
- **Aborting.** After `abort` the store is closing: the host opens
  nothing and drops the store requests it holds, and an open it will
  never make is no work.
- **Work.** `work_pending` takes `marked`, whether the store's start mark
  is durable, since the open waits for it. An owner whose own gate
  holds the host back until the mark passes `true`.
- **Reads for an owner's diagnostics and tests:** whether it has opened,
  the effects waiting, whether it holds an event, and the component, for
  what the component reports of itself. None of them formats a value.

### 1.2 The worker's agents

`jig-protocol-worker-agents` holds two parts of a worker's protocol
layer: `sources`, a component, and `route`, functions that keep no
state. Both speak the hub's values and smith's, and neither names a
root.

- **One answer per call.** The program sources run one command at a
  time, so each of `start`, `from_file`, `from_io` and `fire` returns the owner and
  the answer it settled, if any, and the service pushes it as its root's
  event in the same pass.
- **A name smith refuses is an answer.** `start` checks a source's name
  against smith's grammar; a name it refuses answers `Unconfigured`, and
  no command runs.
- **How settings name a program is the service's.** `Resources::new`
  takes smith's `Programs`, the environment, the bound on a path,
  the effective user and an optional shell-opened programs parent.
  A missing parent makes a check unconfigured. Each service keeps one
  small function from its settings to `Resources`.
- **Inspection has file I/O below it.** Both worker services keep a small
  file stage with fixed limits for one descriptor and one request, with
  no configuration member. It routes completions in the worker's token
  range and file events by owner to smith's inspection, and participates
  in deadlines, settled and emptiness checks. The programs parent closes
  through the same stage when the worker closes.
- **The route takes smith's values and the hub's,** and gives them back,
  or the hub's `Undecoded`, what of a start did not decode, or `route`'s
  own `Processed`, an agent process's event as a live piece or as smith's
  process event. A worker's root hands its payloads out in its own
  enums, in a domain crate, and `jig-domain-host` links no smith, so
  each service keeps a few lines matching those enums onto the route.
- **A delivery's evidence is a value.** The application keeps it and
  says what it must say (`channel/owners.md`, 1.2), and `decode` takes that as
  `Evidence`: `Text`, where the bytes are a receipt's or a refusal's
  words, or `Reply`, where they are smith's `DeliveryReply`, which must
  decode and agree with the outcome the core kept. `evidence` makes
  that check alone. `decode` reports the first thing that fails in the
  start's order: the charter, the transcript, then each answered call,
  so a delivery's evidence is checked in its place.

### 1.3 The workers' side

`jig_protocol_engine_workers::host::Workers` holds the channel
component, the workers' listeners and their accepted sockets, behind
one owner's calls. The owner makes it from the engine's configuration,
its private files, a payload version and `Tokens`, the namespaces of
the listeners' and the connections' tokens, and drains `io` as it
drains every component's. This `host` is the one module of the channel's
crate that reads configuration.

- **The root's events are pulled.** After every call that moves the
  component (`down`, `up`, `iterate`, `close`, `abort`) the owner reads
  `pending` and calls `output` that many times, before anything else in
  its pass. Each call takes one of the component's requests, in order.
  `output` does itself what needs no owner: an io request joins `io`, a
  socket is bound or forgotten, the component's close is noted, and it
  returns `None` for those. `Hello`, `Up` and `Lost` come out as
  `Output`, and the owner routes each before it asks for the next, so
  its root sees the component's order. An `Up` keeps its parts' bytes:
  the owner decodes them or drops them.
- **A grant's value comes in.** The table of values is the application's.
  `grants` names the grants a `LinkDown` carries; the owner looks up
  each, in order, and hands `down` the finished `Event::Down`, one slot
  per grant. An assignment is the owner's whole: it encodes one the same
  way.
- **What a lost listener means is the owner's.** `up` returns `Seen`:
  `Taken`, `Unknown` for an event the host has no use for, or
  `ListenerClosed`, which says whether the host was already ending. An
  owner may stop its engine on a listener it did not close, or carry on.
- **Closing.** `close` closes the listeners and drains the component;
  `abort` aborts them. A listener that opens after either is closed or
  aborted at once. `closed` holds once the component has closed, no
  socket or listener is left and `io` is empty.
- **The bound address.** The host keeps each listener's configured
  address, for the owner's report of a lost one, and the address last
  bound, in `listening`, until the owner takes it.
- **Its `Debug` counts.** A queued write holds an encoded grant, and a
  read what a worker sent, a credential among them. So `Workers`,
  `Component`, `Request` and `Event` show counts and variants, and no
  bytes.

### 1.4 The server

`jig_protocol_engine_clients_http::server::Server` is one listener and
its connections, each with skein's HTTP server over it, their deadlines
and the page policy. It knows no route: what a call means is its owner's.

- **Every call goes to the owner.** A head comes out as `Routed::Call`,
  and each later event of that call as `Routed::Http`. The owner hands
  the call to the mapping, or answers it from its own routes through
  `answer`, and ends it with `done`. Its pages and its bundle are its
  own (`browser/serving.md`, section 1).
- **The page policy is on every response:** `policy::FIELDS`, the
  reference's (`browser/serving.md`, 1.1), but for a field the response already
  holds.
- **Amounts are `Limits`:** the connections, skein's HTTP limits and the
  stall. `valid` says whether limits can run at all. A full listener closes an
  idle connection, with no call or request begun, and admits the next once that
  close settles; it rejects only when none is idle.
- **A peer's end is a value.** `Limits`' `peer_end` says what a peer's
  end does while a call is unfinished. `PeerEnd::Connection` ends the
  connection and tells the owner the call's connection closed, as
  `client/default-http.md`, 4 asks. `PeerEnd::Call` leaves the connection to
  skein's server, which lets a response go on once the request is all
  read and says nothing of that end above (`skein/docs/design/net/http/server.md`,
  5): the owner hears the call end with its response.
- **A call the owner holds.** `hold` keeps a call's connection to a
  deadline of the owner's, past the stall, and `release` gives the call
  back to the stall.
- **Closing.** `stop` closes the listener. `close` also ends each
  connection as its call ends; once the owner says `finish`, that no
  response is left to come, it ends those with a call still open, after
  what they have queued. `abort` ends everything at once, and `abandon`
  one call's connection.
- **Its `Debug` counts.** A head holds its cookies and a response its
  `Set-Cookie`, so `Server` shows its listener, its phase and how many
  connections it holds, and `Routed` its variant and its call's token.
- **What serving a bundle has in common** is `bundle`, beside the server:
  `File` and `kind`, a file's type by skein's closed list (`skein/docs/design/ui/web/bundle.md`,
  4), a `File` holding its compressed form where the bundle has one;
  `compressed_of`, the file a `NAME.gz` is the form of; `target`, a
  head's path; `file`, the file a path names under its digest; `field`,
  one header; `demand` and `chunk`, a body's way out in chunks of the
  server's `send`. They hold no state and answer nothing.
  The owner keeps its `Bundle`, its route, its answers, what it caches
  and how a call ends (`browser/serving.md`, section 1).

### 1.5 The store's host

`jig_protocol_store::host::Hosted` is the store's component and
`Drivers`' three file drivers (`store/README.md`, section 8) behind one owner's
calls, with the queues its answers wait in. The owner makes it from the
store's limits, the application's sizes, the models and a seed. `Drivers`
stands alone for a world that drives the drivers itself.

- **Answers are pulled.** `above` holds the store's terminals in order,
  and the owner's dispatch takes them. `secrets` and `sources` hold the
  terminals that dispatch hands its clients' side and its accounts'
  host, until their passes take them (`store/secrets.md`). These three
  and `component` are public fields: the owner's dispatch reads them and
  a test's fake terminal goes in through them, and a method for each
  would hide nothing. `mark_durable` says whether the start mark is
  durable, as every owner's pass asks.
- **Requests.** `ask` takes any request. `secrets` takes a secrets'
  request, and returns the answer to a `Get` or a `Load`, which stays in
  the pass; a write's terminal comes through `above`.
- **A get the store can't answer yet is a value.** With
  `NotReady::InPass`, `secrets` returns `SecretsNotReady` as it does any
  other answer to a get. With `NotReady::Above`, it stays in `above` for
  the owner's dispatch.
- **Submissions by lane.** `pop_submission` hands out each kernel record
  with its lane, and the owner tags it from its own table; it takes a
  completion's tag off and gives `complete` the lane (`Lane`). The
  namespaces are the owner's.
- **Closing.** `close` closes the drivers and drops what the three queues
  hold. `idle` and `closed` answer for the drivers and the queues; the
  owner adds what it holds.
- **Its `Debug` counts** its queues: a terminal can hold a kept digest.
- **What the owner keeps:** its routes and what they hand up, its sizes,
  the wires it gives the component after `new`, the staging queue its
  tagged submissions wait in, and whether the store has opened and with
  which installation, which its dispatch learns.

### 1.6 The clients' side

`jig_protocol_engine_clients::host` holds what stands between the
clients' component and the rest of a service. The component, the mount
and the server stay fields of each service's own `Clients`, whose pass is
the application's: its calls and families have their places in that
order, and no part cuts across it.

- **`Bridge` carries secrets.** `ask` takes one of the component's
  secrets' requests, names it with a token in the owner's namespace,
  keeps what it was for, and returns the token and the store's request;
  the owner calls its own store with them. `answer` takes the store's
  terminal for such a token and returns the component's `SecretsAnswer`.
  When a request goes is the owner's: it may hold requests back until its
  store's start mark is durable.
- **`Endings` holds a sign-in's ending until its own commit is durable**
  (`../domain/root.md`, section 5). `ended` records a sign-in a commit
  ends, `committed` the durable number, and `next` hands out the next
  ending whose commit is durable, in commit order; the owner calls the
  component with each. The most commits it holds is a value at `new`,
  and an owner with a bound asks `room` before it commits. `ending` reads
  the sign-in from a core key: which of an application's writes end one
  is its own to read.
- **The printed link.** The mapping's `address_text` is a bound address
  as text, and its `printed_link` joins that to the path its `link_path`
  gave. An owner with a public origin writes its own.
- **The component's settings are a value,** `Settings`. This crate can't
  read configuration, which names it, so what every application fills
  alike is `jig_protocol_config::clients::settings`, beside `models`.
  `clients::Application` carries what differs: the printed link's
  identity, the families served, the application's own public kinds and
  the moved file's bound. The two seeds come in as they are. An owner
  with its own issuer sets `provider` and `addresses` on the result.
- **What the owner keeps:** its pass, its calls and their replies, how a
  root's event becomes a component's, which starting requests it parks,
  and when it builds the component.
