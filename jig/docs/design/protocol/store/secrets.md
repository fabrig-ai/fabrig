# Secrets

- **A second skein-kv store,** in `secrets/`, opened as a private
  directory (skein's `runtime/io/files.md`, 3): the directory `0o700`, its files
  `0o600`, owned by the engine's user. The store, its snapshots and its
  logs hold no secret.
- **What it keeps,** each under its owner's tag and the number the domain
  gives it:
  - minted credentials' digests, by their sign-in's number, with their
    expiry: each credential the clients' component mints, for a printed
    link's sign-in, a service's, or an identity the application's server
    verified (`../client/sign-in.md`);
  - services' credentials' digests, by the service's number;
  - each signed-in source's record, by the source's number: its tokens,
    the client id the provider issued, the scopes granted and the ID
    token, as skein's accounts driver hands it to keep (skein's
    `oauth/accounts.md`, 4; `../llm.md`, 5.1), or as a moved record brought it
    (`../llm.md`, 5.6);
  - each LLM key, by its source's number, adopted from the value its
    request carried (`../client/README.md`, section 3);
  - an application's own records, under its tag: its connectors'
    credentials, which it decodes itself, by numbers below 2^63;
  - each value a request carried (`../client/README.md`, section 3), an LLM key
    or an application's value, under the application's tag, by a number
    jig draws itself from 2^63 up, never one the request names: the next
    number is a row of its own beside the values (key `[2, 1]`), written
    in the commit that keeps the value, so a number is never drawn
    twice, an erased one included, and a number already holding a value
    is never written over: the carry is refused and nothing is kept;
  - the public identifiers' key (`../../domain/people/public-reading.md`, 8), one row
    under a tag of its own: 32 bytes the clients' component draws from
    its random source at its first start and never changes, so a public
    address stays the same across restarts.

  A program source's login is not here: it is the program's own, on
  each host (`../llm.md`, 5.5).
- **Each row from a schema,** jig's family `secrets`
  (`schema/secrets-v1.schema`),
  versioned records whose key says which:
  - `Digest { expiry: u64, digest: fixed 32 }`, a sign-in's (`[1, 0]`)
    and a service's (`[1, 1]`), an expiry of `u64::MAX` for a service's;
  - `Source`, a source's row (`[3, 0]`): a versioned record whose one
    field is an enum of records, `Record { record: secret N holds
    skein-oauth SavedToken }` (skein's `oauth/accounts.md`, 4) or `Key { key:
    secret N }`, so a row says which it holds, never guessed from its
    bytes; written and restored by the accounts component, which knows
    which it keeps;
  - `Carried`, a carried value (`[2, 0]` from 2^63), likewise: `Key`,
    `Value` (an application's) or `Moved { moved: secret N holds
    jig-accounts Moved }` (`../llm.md`, 5.6), its kind passed by the
    clients' component through `carry`, from the request it decoded; a
    pasted address stays held in memory, never kept;
  - `NextCarried { next: u64 }`, the carried values' next number
    (`[2, 1]`).

  The store encodes its family's rows, so its secrets' requests carry
  typed values and no service keeps a hand codec of a row:
  `SecretValue { Digest { expiry, digest }, Record(Secret), Key(Secret),
  Carried { kind, value }, Application(Secret) }`, `CarriedKind { Key,
  Value, Moved }`. `Keep`, `Replace` and `AdoptValue` take one, `Carry`
  takes its kind, `Get` and `Load` answer them, and a value that
  disagrees with its key's owner is refused; `Adopt` unwraps a carried
  key into a source's `Key`, any other kind answering stale. A row whose
  envelope does not decode fails the read and stops the engine, as a
  failed commit does (an older build's store was refused at open by its
  layout); a held record's digest is taken before it is decoded, so one
  that does not decode can still be replaced.

  Each `N` is its value's bound; the version, the tags and the options
  add bytes, which each row's bound and the secrets' store's worst case
  count. A held record is decoded and checked by its own code (skein's
  `decode_record` and its checks, `Moved`'s), the words changing no
  decode. Adopting a carried value unwraps its `Carried` and writes a
  `Source`; every digest a conditional write states (`adopt`, `keep`'s
  replaced record, a key's) is the digest of the value held, never of
  its envelope. The application's rows
  are its own family's, by its own schema (temper's
  `protocol/credentials.md`, 2.1). jig's set names the family, `map:
  Secrets`, and its keys' ranges, the application's left out (`opening.md`, 3), so
  the copy's secrets are patched as the records are. Its layout is one
  of jig's families' in the main store's layouts row (`records.md`, 3), so a store
  of a build before it is refused at open, `Older::Layout`, before a
  secret is read, and made again.
- **Written by components alone,** through the protocol layer's routing.
  The domain names a secret by number, never holds one, and never sees
  the secrets' store's commits, which have numbers of their own.
- **Kept before used.** A sign-in is committed first, which gives it its
  number; its credential's digest is then kept, and durable; only then
  does the clients' component answer the credential, to the client that
  signed in or to the application's server that asked it to mint
  (`../README.md`, section 8). A crash between leaves a sign-in no client
  holds a credential for, which expires with its time.
- **A record kept before lent.** skein's driver takes this store as its
  keeper (skein's `oauth/accounts.md`, 4): each new record goes out as a keep,
  which the routing writes here under its source's number, stating the
  digest of the record it replaces, and answers `kept` once it is
  durable; the driver lends the record's token only then. A crash
  between leaves at worst a record its driver never lent, which the
  next start lends from.
- **A carried value kept before the core sees it.** The clients'
  component takes the value out of the request carrying it as it
  decodes it, and keeps it here before the core gets the request, which
  names it by its number and digest only (`../client/README.md`, section 3).
  Where the core leaves the request unused (refused, a key sent again
  under a key answered or in flight, or the engine stopping) it says
  so, naming the number, and the component erases that value, if its
  own decode kept it; replacing or erasing a kept value goes through
  the core's decisions alone.
- **A pasted address held, never kept.** A sign-in's pasted address
  (`../llm.md`, 5.4) is carried as a key is, with one difference: the
  store holds it in memory and writes nothing, under a number of a
  range of its own, drawn in memory, which names a value of this run
  alone. The core names it by number and digest as it names a carried
  value, and leaves its request unused whatever the answer, so the
  clients' component erases it once the request is answered, its one
  eraser; the accounts component only reads it, and a value already
  gone answers its finish not waiting. The store has no clock and
  drops a value by no time of its own: the accounts child answers
  every finish by its sign-in's end at the latest, and the clients'
  component then erases it. The store drops what it holds at its
  close; a crash or a restart loses it with the sign-in it was for,
  which never survives one. Held values are bounded by the clients'
  component's carried values at once, each by the driver's redirect
  bound, in the worst case; past them the store holds nothing and the
  request is answered busy.
- **An LLM key adopted before granted.** The accounts component checks
  the carried value against its byte bound, then asks the store to
  adopt it into the source's row, naming the digest it replaces, or
  none for a new source. Adoption writes the source's row and erases
  the carried row in one secrets commit; a missing carried value or a
  changed destination digest answers stale and changes neither row. The
  component grants the key only once adoption is durable, a replaced
  key as its next generation (`../llm.md`, 5.1). A later erase of the
  carried number is a no-op. A source's record or LLM key occupies one
  secret row under a fixed-size store key.
- **A moved record adopted the same way,** in one secrets commit
  (`AdoptValue`, beside the key's `Adopt`) that writes the source's row
  and erases the carried row: the row holds
  the record the carried file holds, in skein's form, not the file's
  bytes as they came (`../llm.md`, 5.6), and the adoption states the
  carried value's digest, answering stale when it changed. The record
  is added to the driver only once adoption is durable, and lent only
  after its first refresh is kept.
- **A moved-out record is read, never copied in the store:** the
  routing reads the source's row once the driver has removed it, and
  the file's bytes made from it live only in the protocol layer until
  the client takes them (`../llm.md`, 5.6). The row is erased when the
  move is written.
- **A presented credential names its sign-in:** a credential is the
  sign-in's number and its random bytes, opaque to the transport that
  carries it. The component gets the digest by number, and the clients'
  component compares in constant time.
- **Erased** when the core says a sign-in ended (signed out or revoked)
  or expired, or a service was removed; and when the accounts child
  says a source was signed out or removed, a key replaced, a record
  revoked, or a move written. A crash between that commit and the erase leaves a value
  nobody names, which the restart sweeps: once people's families are
  restored, the component erases each of jig's digests whose sign-in or
  service it did not hand up. For sources, the accounts child names
  each party source to restore before wanting it, then says the list
  is complete. The accounts component re-adds signed-in sources and
  loads keys by number; it then pages the source records once and
  erases each neither configured nor named, including orphaned keys.
  Configured sources are spared even without an accounts-family row.
  Every application row is spared: a carried value an application used
  may still live at its drawn number. Every link is made before a
  sweep runs (`../../domain/engine/restart.md`), so a link's refusal
  erases nothing. A value orphaned by a crash
  before the core says used or unused remains there, including an LLM
  key not yet adopted; collecting those values is later work.
- **A failure stops the engine,** as the store's does (`README.md`, section 5): what a
  failed commit of the secrets' store was keeping is never handed out.
- **Read at start,** within its own limits and worst case: named sources
  by number, the sources' sweep in bounded pages.
- **Patched with the store** (`opening.md`, 2), in the copy's own secrets' store
  and its own commits, by the patch sets of the families it holds:
  jig's digests and records, the application's records under its tag,
  and skein's OAuth records by skein's set (temper's
  `draft/upgrades.md`, 6.3). Its numbers are kept as they are: the
  carried values' next number (`[2, 1]`, its own row) and every number
  a row is kept under, the sources' and accounts' drawn by the accounts'
  counter in the store (`[1, 4, 0]`), so that the start's sweep erases
  nothing it should keep and no number is drawn twice. Its last commit
  is named in the manifest (`README.md`, section 2). A kept family names
  its map (`KeptFamily::map`, `opening.md`, 3): a secrets family is walked in the
  copy's secrets' store, each page one commit of that store's own,
  within that map's limits and record room, and the seal stays the
  store's (the secrets' store has no format row or header). A fixture
  family of the application's records proves it (`jig-test-patches`'
  `fixture-token`), both numbers unchanged across the switch.
