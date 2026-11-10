# Public reads

The messages of public reading (`../../domain/people/public-reading.md`):

- **`PublicRead { owner, version, kind, ids }`,** an exchange with no
  credential and no stream: an owner, jig or an application's family;
  the version of the owner's public records, which are versioned apart
  (`../../client/domain/contract/README.md`, section 10); a kind of that owner's
  public documents; and at most two public identifiers, which the kind
  names. jig's kinds: *projects*, with none; *board*, a project; *card*,
  a project and a task. An application's kinds take a project, then at
  most a task: never a number of the application's own.
- **Answered once:** the document, as the owner's record at that
  version; unknown; busy; or the version not served. Nothing else, and
  never why: a private project, one never made, a task that is not
  public, an identifier that names nothing, a kind no owner declared, a
  connector's failure and a fault decoding or encoding are all unknown,
  each with the same answer.
- **Public identifiers** are the component's (`../../domain/people/public-reading.md`,
  8): a keyed permutation of a project's or a task's number, under a
  key kept in the secrets' store under an owner of its own, made from
  the component's random source at its first start and read at each
  start before it answers a public read (until then, busy). Every
  well-formed read is mapped and handed on alike, whatever its
  identifiers name; the core's public index decides.
- **The component admits it** within its public room (in flight), its
  reads' bucket and its bytes' bucket, reserving the kind's declared
  largest answer before it hands the read on and giving back what the
  answer did not use, or all of it on any other end
  (`../../domain/people/public-reading.md`, 5), each apart from the sign-ins' room;
  past any, it answers busy at once and hands the engine nothing. It
  hands the engine's root what it admits as the public party's read,
  and a read the root refuses is answered busy, never offered again.
- **An application declares its public kinds** to the component at
  start: owner, kind, the versions it serves, whether it takes a task,
  and its answer's bound in bytes, within the deployment's answer
  ceiling; a kind past it refuses the start. A kind no owner declared
  is unknown before the engine.
- **The engine's side** admits it on the public index
  (`../../domain/engine/views.md`): the project public, then the
  task. It makes jig's kinds from the index, and asks the connector
  that declared an application's kind for its document, handing it the
  project's public tasks; the connector answers with the document and
  the list of tasks it names, which the core checks again before it
  answers (`../../domain/people/public-reading.md`, 10).
- **An application's document crosses in jig's envelope:**
  `PublicApplication { tasks, record }`, the tasks' public identifiers
  in the order the connector listed them, and its record in its own
  wire, which names a task only by its place in that list. The
  component maps the list; a connector never holds the key.
- **Its freshness:** each answer carries when the decision that made it
  was taken, by the engine's monotonic clock, from which the clients'
  end gives the rest of its `max-age` (`default-http.md`, 5).
- **Every other exchange without a credential** is answered signed out
  (`sign-in.md`) by the component from the exchange alone: a read, a
  watch, a stream, a keyed request. The public party opens no stream
  and no watch; a page reads again by its own timer.
- **Testing:** the component's world plays public reads with no
  credential and with a signed-in person's cookie, answered alike; past
  the room, the rate and the bytes, busy while signed-in exchanges are
  answered; reservations given back on every end, two reads completing
  at once and one whose exchange is gone among them, the bytes' bucket
  whole after; an unknown kind, a private project, an identifier that
  names nothing and one of a task that is not public, unknown alike,
  with the same messages to the engine; identifiers mapped both ways,
  distinct for projects and tasks, and the same after a restart. The
  mapping's world: each answer's status and exact headers, a late
  answer's shortened `max-age`, a credential on the route never read,
  malformed paths (an overflowing number, a leading zero, upper-case or
  short identifiers, an encoded or dot segment, a query, a trailing
  slash) `404`, and `OPTIONS`, `HEAD` and `POST` `404` with no
  `Access-Control-` header. Its fuzzy tests: the public path's parser
  and the public records' decoders over seeded bytes.
