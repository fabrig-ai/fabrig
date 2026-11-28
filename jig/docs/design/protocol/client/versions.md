# The client's protocol: versions

## 1. Versions and layouts

- **Per owner:** jig's wire and each application's carry their own
  version and layout; every message names its owner and the version of
  its record, and a streamed message's record is in the version its
  watch was opened in.
- **Versions are frozen** (`temper/docs/design/draft/archive/layouts-both-ways.md`,
  3). Each client family keeps the version it has, and a layout change
  moves none: jig's `jig-client-wire` and `jig-public-wire`, in
  `jig-client-protocol-wire`; `jig-engine-cursor`, in
  `jig-protocol-engine-clients`; the reference UI's
  `jig-client-view-records`, in `jig-client-view`; temper's
  `temper-client-wire` and `temper-public-wire`, in
  `temper-client-protocol-wire`. No list of layouts is kept. One check
  in `fabrig-boundary`, `client_layouts`, holds the freeze: a listed
  family whose version differs from the base's is refused, naming the
  family and both versions, "client versions are frozen". The base is
  the gate's (`.config/gate/README.md`), else git's merge base of the
  branch with `origin/main`, each family found by its header among its crate's
  `schema/*.schema`; a family the base lacks passes. Each crate's own
  tests hold its version constants equal to its schema's `family` line:
  jig's `VERSION` and `PUBLIC_VERSION`, temper's `CLIENT_VERSION` and
  `PUBLIC_VERSION`. jig's example and test wires, which no deployment
  serves, are not listed.
- **A layout is the family's fingerprint** (skein's
  `data/codec/fingerprints.md`, section 1), which the family's generated code
  names `LAYOUT` (skein's `data/codec/README.md`, section 4). The family's, not its
  declarations': a declaration added, dropped or moved changes it, so a
  client may be refused for a change no message's bytes show, which
  costs it a reload and never a record misread.
- **The envelope is fixed.** A message's owner and version, and the
  answer *version not served*, are encoded the same in every version
  and layout of jig's wire, so any engine reads them from any client,
  and any client reads that answer from any engine. *Version not served*
  is the first variant, tag 0, of each answer that carries it (a keyed
  request's, a read's, a watch's opening's, and a watch's end), and its
  bytes never change: a later layout adds its variants after it. So is
  the hello alone, the stream's first variant, tag 0: the stream's
  name, the ranges served and the heartbeat, encoded as version 27
  encodes them, which an engine sends a stream whose opening names no
  version it serves, so that a client of any version reads it and is
  refused by name (`wire.md`, section 2). So is `Opened`'s head,
  the stream's variant `opened`, tag 4: its `hello`, then its `layouts`,
  each `Layout { owner, layout }`, an owner and its 32-byte layout, at
  most the hello's bound of ranges, ahead of `me`; so a client of any
  layout reads the engine's layouts before anything that may differ.
- **The engine serves one layout of each owner,** at its one frozen
  version, which its stream states: the layout in `Opened`'s `layouts`,
  for jig and for each owner the application declares, with its version
  and its layout. To serve a layout is to hold its generated codecs and
  their translations. The hello's `served` keeps its bytes: each owner's
  one version written as both ends of its range
  (`../../client/domain/contract/README.md`, section 6), which no client
  reads.
- **A message of another version** than the engine's one of its owner
  is answered so, typed, before the record is read: *version not
  served*, naming the owner, its range written as the hello's is. A
  keyed request, a read and a watch's opening are each answered so;
  nothing is done. No client of a frozen family sends one.
- **Layouts alone are judged at the hello.** The client compares each
  layout `Opened` names with its own. Another layout of jig's wire, or
  none named, makes the link out of date, refused by name; another
  layout of a declared owner's, or none, makes that owner not served
  (`../../client/domain/client/watches.md`, 2). A hello alone names no
  layout, so it is judged as one naming none: the engine sends it only
  to a stream whose opening names a version not its own. No client
  reads a range, from a hello or from an answer.
- **Nothing of an owner leaves unjudged.** A keyed request waits for
  `me` and a watch's opening for the stream's name, as they do; a read,
  alone or in a batch, waits too, until a hello since the stream last
  opened has judged its owner served, so the engine decodes no record of
  a client at a layout it does not share. Signing in and out, and the
  approvals and link asks a client makes whether or not its stream is
  live, wait for no hello, and no layout judges them (section 2).
- **What a client keeps or gives back names its layout:** a kept record
  its wire's layout and a kept request its owner's (`small-store.md`,
  section 1); a cursor the engine's cursor layout (`README.md`, section 4); the
  reference UI's records their family's
  (`../../client/domain/view-state.md`, 8); a public read the reader's layout of
  its owner's public records (`public-reads.md`). One at another layout is
  dropped, or answered as one of another version is.
- **The client's versions and layouts:** jig's are its wire's; each
  family owner's, the application declares as it makes the client, with
  its limits. Its messages name the version; the client judges each
  hello's layouts against its own, whole
  (`../../client/domain/client/watches.md`, 2). A hello names at most its wire's
  bound of ranges, jig's among them: an engine serves no more owners
  than that, and a client declares no more family owners than that bound
  less jig's.
- **A client at another layout,** from the hello, or answered *version
  not served*, sends nothing more of that owner: its requests
  stay kept and unsent, its reads are answered at once, unsent, and its
  watches end, as version not served. Outside jig's, the link is out of
  date, and the client sends nothing at all
  (`../../client/domain/client/watches.md`, 2).
- **A record the client can't read** is the same news, found later: a
  stream's record of jig's, or a watch's opening's or closing's answer,
  that the link cannot decode or translate means the engine speaks a
  layout this build does not, or is at fault. The stream is dropped and
  not opened again: the link is out of date, naming it `unreadable`
  (`../../client/domain/client/watches.md`, 2). Offline, it would open
  the stream again after each backoff, meet the same record each time,
  and tell no one. A record refused for a bound (past the client's
  limits) or for its order on the stream (a delivery, a run's text or
  an end before the hello names the stream) says nothing of a layout:
  an engine of the same build can send it, and a new stream may not
  meet it again. Such a record drops the stream, which opens again after
  its backoff, as before.
  The application says how to update; updated, the client sends what it
  kept at a layout it speaks, under its keys (`small-store.md`, section 1).
- **One jig pin:** an application's `jig-client` and its engine come
  from one jig pin, so a client the application's server serves is at
  the engine's layouts unless the engine was updated under a client
  still running; reloading or restarting the client updates it. A
  client that does not come from the engine's application (a phone's, a
  command line's) may lag it.
- **How jig is released and pinned** is an open question
  (`../../README.md`, section 14).

## 2. Later

- **Clients bridged:** the engine converting for a client at another
  layout, so that a client outlives a redeploy without reloading
  (`temper/docs/design/draft/archive/layouts-both-ways.md`, 10).
- **The exchanges no hello judges:** signing in and out, approvals and
  the link's asks may reach the engine before any hello, so a client at
  another layout may meet a refusal there that names no layout, where a
  record whose own declarations changed fails to decode. Naming the
  client's layouts in each such exchange, as bridging would need them,
  would judge these too.
