# Keyed requests

- **Records precede their sends across visits.**
  `request_records_dispatched` requires the request table's pending
  records to be dispatched before eligible sends, even when its cursor
  yields only one item per visit. The client world holds this through
  restoration, retirement and retry in `jig-client-world`,
  `orders::request_records_precede_their_sends_across_visits`.

- **What a request is:**
  - an operation of the API (../contract/requests.md, 2): `Create`, `Propose`,
    `Withdraw`, `Message`, `Amend`, `Cancel`, `Release`, `Stop`, `Move`,
    `Decide`, `Answer`, `Take`, `HandBack`, `Seen`, or one of the
    project's own;
  - a compound request, `Compound { project, operations }`: operations
    admitted in order and committed in one decision, all or none, under
    one key and one answer (../contract/requests.md, 4);
  - a family's request: its owner's number, its scope, its version and
    its record's bytes (../contract/requests.md, 3).

  The client reads a family's request only by its owner, version and size
  (`README.md`, 4.3); its domain recognizes top-level jig `Seen` to decide whether to
  keep it (below).
- **The key is the application's name for a request:** its asker, and a
  number the asker draws from its seed, fresh for each request it means.
  A second `Send` under a key the client holds is joined to the first
  when it carries the same request, so a double press or a retry makes
  one request; with another request, it is a key conflict, a mistake of
  the application's, and loud (`../../../README.md`, 6.5). The engine knows
  the request by its engine key, scoped to the party (../contract/requests.md,
  1): the key the table draws, or a name (next bullet). The client
  keeps it beside the application's key in the request's record, so a
  kept request comes back under its key (`README.md`, section 8).
- **A `Send` may carry a name:** an engine key the application gives,
  which the table admits in place of a drawn one (skein's `lib.md`,
  section 12). The request's record keeps it as the engine's key beside
  the application's key. Any ask may carry a name, a family's included.
  - *Held.* A named `Send` whose engine key the table already holds (the
    same ask under another application key, another ask, or a restored
    record of another asker) is answered held at once: nothing is kept
    and nothing is sent. The key stays held until its holder's record is
    erased; the application asks again then. Within the engine's
    retention an ordinary deciding request gets its original answer
    again. A `Seen` retains no engine answer after completion and
    is processed against current state on retry (../contract/requests.md, 1).
  - *Never loud.* A name comes from outside the application (an agent's
    key), so a name held is never the application's mistake. The same
    application key with the same request is still joined, and with
    another request is still a key conflict, loud.
  - *Not joined.* The table keeps one asker per named entry, `Seen`
    included.
- **Through skein-lib's request table** (skein's `lib.md`, section 12),
  which the client holds. It is the client's only way out for a keyed
  request without a carried value, as the journal is the engine's root's
  (`../../../README.md`, 6.2); a request carrying a value (below) is the
  one request that does not go through it. It:
  - *keeps ordinary requests before sending:* the request's record leaves
    in a `Save`, and its first `Exchange` only once the transport says
    `Stored` (`README.md`, section 8);
  - *sends it again under its key,* after a backoff drawn from the seed,
    while its answer is busy, not ready, or lost with the link live; and
    *parks it* while the link is down or the party signed out;
  - *sends only the confirmed party's,* the party `me` names (`watches.md`, 2);
  - *retires it past the retention* `me` states (../contract/requests.md, 1): a
    request first sent longer ago than the retention, less a margin, is
    never sent again, and is answered unknown;
  - *retires it not kept* when the last write that carried its record
    is not stored, whatever its state, parked included (`README.md`, section 8);
  - *is bounded:* requests held, and their bytes, a family's included. A
    `Send` past them is answered at once, refused by the client's limit,
    and never queued.
- **`Seen` stays in memory.** The client domain takes the table's `Save`
  for a top-level `Ask::Jig(Seen)` without emitting `Down::Save`. It
  creates no small-store record and uses no value-carrying path. The
  generic table and its other outputs remain the same: application and
  engine keys, joins and conflicts, optional caller names and held names,
  progress, retry after busy, not ready or lost, parking, confirmed-party
  checks, retention's unknown outcome, and one answer while the client
  lives. It counts against the same request and byte limits. The engine
  retains no completed `Seen` answer on success or routed refusal;
  after completion a retry checks current subject and authority and may
  answer `Unknown` after forget or `Role` after authority loss
  (../contract/requests.md, 1 and 2).
- **A request carrying a value,** such as an LLM account's key added or
  replaced, or a moved file moved in (../contract/requests.md, 2), or an
  application's request's credential, is sent once and never kept, so
  that no value is ever in the small store
  (`../../../protocol/client/README.md`, section 3); and so is a request whose
  answer carries one, an account moved out, whose record comes once:
  - *beside the table:* it carries a name, the engine key its caller
    gives and never makes from the value (../contract/requests.md, 1), and goes
    as one exchange: never in a `Save`, never parked, never sent again
    by the client;
  - *its one answer* is the engine's, done or refused; or the client's
    own *unsent*, when the engine answered busy or not ready, the link
    is down, or the party is signed out or not yet confirmed: the
    engine did not make it; or the client's own *lost*, when the
    exchange ended with no whole answer: the engine may have made it;
  - *its caller asks again* under the same name, with the value; within
    its retention the engine answers a name it has answered with its
    first answer (../contract/requests.md, 1);
  - *a moved-out answer with no record,* the engine's answer to the
    name sent again, is answered *lost*: the record went with the first
    answer, and the move is cancelled at its deadline. Its caller asks
    again under a new name once it is (`../../../domain/accounts.md`,
    section 6).
- **An account's move, in `jig-client`'s API:** `move_out` sends the
  request once and hands its caller the moved record and the move's
  number in its answer; the caller writes the file, then sends
  `written` with the number, an ordinary request, kept and sent again
  under its key; `move_in` carries the file's bytes as a key's value
  is carried. `jig-client` never writes or reads a file, and never
  keeps the record: the file is the application's shell's, written
  new, private and synced before *written*, and removed once a move
  in is answered, or once a *written* is refused *cancelled*.
- **Progress, for showing.** `Pending { key }` says the request's
  progress changed: in flight, retrying, parked (offline, signed out, or
  another party's), or held for its owner's version (`watches.md`, 2). Retrying
  says why: busy, not ready, lost, or a final answer that did not
  decode, which is how the client reports such an answer, once per
  answer (`../../../protocol/client/README.md`, section 3).
  `client.pending(key)` reads it. Whether and how it shows is the
  presentation's.
- **One answer.** `Answered { key, outcome }` comes once per request:
  - *done,* with its outcome: what the engine made or decided, or, where
    another party decided first, who, how and when; for a family's
    request, its owner's outcome, as bytes;
  - *refused,* with its facts (../contract/README.md, section 3): jig's, typed, for
    every request the people child refuses at admission, a family's
    included; the connector's own outcome, as bytes, for a family's
    request it refused;
  - *unknown,* the client's own: past the retention, whether the engine
    made it is unknown, and the presentation, which knows the request by
    its key, says where to look;
  - *not kept,* the client's own: its record could not be written, so the
    client never sent it (`README.md`, section 8).
  - *unconfirmed,* the client's own: sent before `me` named the party,
    so no record could say whose it is. It is answered at once, and
    nothing of it is kept or sent (`README.md`, section 8).
  - *held,* the client's own: the name the `Send` carries is held by
    another request. It is answered at once, and nothing of it is kept
    or sent (above).
  - *unsent* and *lost,* the client's own, for a request carrying a
    value, or whose answer carries one, only: the engine did not make it, or may have; its caller asks
    again under its name (above).

  Done and refused come only once the engine's commit is durable
  (`../../../domain/model.md`, section 11), so the client never says done
  before it is. Nothing a presentation shows as done may come before
  this answer.
- **Proposing is explicit.** A `Create` or an `Amend` beyond the party's
  authority is refused, naming what it lacked and whether a proposal
  could be accepted; `Propose` asks (../contract/requests.md, 5). Each document's
  offers say, for each act, made, proposable (and to whom) or refused,
  with bounds (../contract/documents.md, 3), so a presentation can say which before
  it sends. The client reads none of it: authority is the engine's,
  judged again when the request comes.
- **Decisions name what they saw.** An operation that decides about
  something that changes names its revision, from the document the
  application showed (../contract/documents.md, 2); the engine refuses one whose
  revision has moved, saying what it is. The client carries the
  revision and checks nothing.
- **One request per decision.** What must commit together is one
  request: an operation, or a compound request (closing a conversation
  with each live delegate cancelled or moved to the party; amending a
  held task and releasing it; reordering goals). The client never chains
  two requests, so nothing is left to undo when the second is refused.
- **A version not served** keeps the request: while its owner's version
  is not served (`watches.md`, 2), the request is kept and unsent, held for its
  owner's version, and it is sent under its key once a hello serves that
  version again. A client updated to another version of the owner drops
  it at start (`README.md`, section 8).
