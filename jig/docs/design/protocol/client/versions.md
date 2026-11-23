# The client's protocol: versions

## 1. Versions

- **Per owner:** jig's wire and each application's carry their own
  version; every message names its owner and the version of its record,
  and a streamed message's record is in the version its watch was opened
  in.
- **The envelope is fixed.** A message's owner and version, and the
  answer *version not served*, are encoded the same in every version of
  jig's wire, so any engine reads them from any client, and any client
  reads that answer from any engine. *Version not served* is the first
  variant, tag 0, of each answer that carries it (a keyed request's, a
  read's, a watch's opening's, and a watch's end), and its bytes never
  change: a later version adds its variants after it. So is the hello
  alone, the stream's first variant, tag 0: the stream's name, the
  ranges served and the heartbeat, encoded as version 27 encodes them,
  which an engine sends a stream whose opening names no version it
  serves, so that a client of any version reads the ranges and is
  refused by name (`wire.md`, section 2).
- **The engine serves a range** of each owner's versions, which its
  stream's hello states (`../../client/domain/contract/README.md`, section 6). To
  serve a version is to keep its generated codecs and their translations
  beside the current ones. Pre-release, each range is one version
  (`../README.md`, section 7).
- **A version not served** is answered so, typed, before the record is
  read: *version not served*, naming the owner and the ranges the engine
  serves. A keyed request, a read and a watch's opening are each
  answered so; nothing is done.
- **Versions move with a wire's layout.** Each changed record moves its
  wire's version and its schema, codec and translations. Tests build
  fixed-rule values from that schema or use public fields, keeping no golden files
  (skein's `data/codec/golden.md`, 1).
  The engine serves one version; a client of another version is refused
  before its record is read. Client bridges are deferred
  (`temper/docs/design/draft/redeploys.md`, 8).
- **A version is its schema's `family` line.** A wire's version is the
  number its schema's `family` line names (skein's `data/codec/README.md`,
  section 3), and its crate's version constant equals it: jig's
  `VERSION` and `PUBLIC_VERSION`, an application's own (temper's
  `CLIENT_VERSION` and `PUBLIC_VERSION`). The small store's versioned
  records carry that number too (`small-store.md`, section 1).
- **The layouts' list holds versions to layouts.** Each client wire's
  crate keeps, beside its schema, its family's list of layouts,
  `schema/<family>.layouts`: one line per version since the list began,
  oldest first, the version in decimal, one space, and the family's
  fingerprint at that version (skein's `data/codec/fingerprints.md`, section 1)
  as 64 lowercase hexadecimal digits; a line starting `#` is a comment.
  The family's fingerprint, not its declarations': a declaration added,
  dropped or moved moves it, so a version may move for a change no
  message's bytes show, which costs an open client a reload and never a
  record misread. Two checks hold each list, in `fabrig-boundary`, and
  delivery runs them for any change to a listed wire's crate; the
  crate's own tests check that its version constant is its schema's
  version, so that `fabrig-boundary` builds no wire crate:
  - *Now:* the list's versions rise by one, line to line, and its last
    line is the schema's version and the schema's fingerprint now.
  - *Landed:* the list as main has it (git's merge base of the branch
    with `origin/main`, as the patch check takes it, `.config/gate/README.md`)
    stands unchanged at the head of the branch's list, its lines in
    order. A list main lacks passes.

  So a layout changed with its version unmoved fails one check or the
  other: *now* while its line is left, *landed* once a landed line is
  edited. A layout changed again before its version reaches main edits
  that version's own line. The lists: jig's `jig-client-wire` and
  `jig-public-wire`, in `jig-client-protocol-wire`; temper's
  `temper-client-wire` and `temper-public-wire`, in
  `temper-client-protocol-wire`. An application whose clients a
  deployment serves lists its wires there too; jig's example and test
  wires, which no deployment serves, are not listed.
- **The client's versions:** jig's is its wire's; each family owner's,
  the application declares as it makes the client, and the client
  judges each hello against them, whole (`../../client/domain/client/watches.md`,
  2). A hello names at most its wire's bound of ranges, jig's among
  them: an engine serves no more owners than that, and a client
  declares no more family owners than that bound less jig's.
- **A client outside a range,** from the hello or from such an answer,
  sends nothing more of that owner: its requests stay kept and unsent,
  its reads are answered at once, unsent, and its watches end, as
  version not served. Outside jig's range, the link is out of date, and
  the client sends nothing at all (`../../client/domain/client/watches.md`, 2).
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
  kept in a version it speaks, under its keys (`small-store.md`, section 1).
- **One jig pin:** an application's `jig-client` and its engine come from
  one jig pin, so a client the application's server serves is in range
  unless the engine was updated under a client still running; reloading
  or restarting the client updates it. A client that does not come from
  the engine's application (a phone's, a command line's) may lag it.
- **How jig is released and pinned** is an open question
  (`../../README.md`, section 14).
