# Watches and the link

## 1. Watches

- **A watch** follows one subject (../contract/watches.md, 1.1): `me`, a task, a
  tree, chips, the inbox, a task query, a run's text, the deployment, or
  a family's subject, naming its owner, its scope and its record's bytes
  (../contract/watches.md, 1.5). Its deliveries are documents, each the subject's
  whole current state within the window.
- **Level-triggered, as facts are** (`../../../domain/model.md`, section 4).
  A watch never carries a change to apply to what came before, so it
  cannot miss one: what changes while a delivery is in flight is in the
  next. A slow client sees fewer deliveries, never a gap. The same holds
  toward the application: `Changed { handle }` says the latest document
  moved, and the application reads it with `client.document(handle)`
  when it likes; one that reads late reads the latest.
- **Handles share the engine's watches.** Several handles naming one
  subject and one window share one watch on the engine and one document.
  It opens with the first handle and closes with the last.
- **A watch waits for a sign-in.** A `Watch` while no sign-in is live
  (the link signed out, 2) is admitted, its handle opening, and
  nothing leaves for the transport: a stream opened with no credential
  ends at once (`../../../protocol/client/wire.md`, 2.2), and the client sends
  none unless the transport carries one (`sign-in.md`, section 1). Once signed in,
  the client opens each such watch under a fresh name; one on `me` is
  the client's own, which its stream brings (2), and opens nothing.
- **Each opening is named afresh.** The client names each watch it opens
  on the engine with a name of its own, never reused, so a delivery for
  a watch it closed is dropped, even when the application has given its
  handle to another subject.
- **Counted until the engine has ended it.** Each of the engine's
  watches counts toward the client's limit (`README.md`, section 11) from the opening
  it sends until its terminal: refused, ended on the stream, its closing
  answered, or its stream dropped. The client closes a watch only once
  its opening is answered: admitted, it sends the closing; refused, the
  watch is over. A watch's opening or closing that gets no answer drops
  the stream (2), whose end ends every watch on it
  (`../../../protocol/client/README.md`, section 5). So, but for a dropped
  stream's watches until the engine has ended them, the engine holds no
  watch of the client's that the client does not count, and a client
  within its limit takes no more of the party's room (../contract/watches.md, 1.2).
- **A `Watch` on an open handle** changes its subject or its window: the
  client opens the new one, keeps the handle's last document, marked
  behind, until the new one's first delivery, and closes the old once no
  handle names it.
- **A handle's states,** which `client.document(handle)` gives beside the
  document: *opening,* no document yet; *live;* *behind,* its watch
  dropped and opening again, or changed, so the document held may be
  old; *ended.* A handle that falls behind, or is live again, is told
  `Changed { handle }`, as for a newer document: level-triggered, the
  application reads both with `client.document(handle)`.
- **Dropped is not ended.** A watch the stream lost, one the engine let
  go as it stopped, or one it refused as busy, is opened again after a
  backoff, under a fresh name; its handle stays, behind, and is live
  again at its next delivery. Busy is the party's room full for now
  (../contract/watches.md, 1.2): the party's other clients, or a stream this client
  dropped, or its predecessor's (`../../../protocol/client/wire.md`, section 2),
  whose watches the engine has not yet ended, hold the rest, and it
  empties as their watches end.
- **`Ended { handle, why }`** is a handle's one terminal, for what opening
  again would not mend: the engine refused it (unknown, not allowed);
  its subject ended and left the engine; the party lost the right to see
  it; signed out; or its owner's version not served. `Unwatch`
  ends a handle with no `Ended`.
- **Bounded:** handles, the engine's watches, and the bytes of each
  document held (`README.md`, section 11). A `Watch` past them ends at once, refused
  by the client's limit. One of the engine's watches is always the
  stream's `me` (2), so the application's take at most one fewer.

## 2. The link

- **`me` comes with the stream.** The client holds `me` while a sign-in
  is live, under a handle of its own, which an application's handle on
  `me` shares (1), and opens no watch of it: its stream's opening names
  jig's version, the hello brings `me`'s first value, and its later
  values are deliveries of the stream's own watch of it, named `0`
  (`../../../protocol/client/wire.md`, section 2). `me` names the party,
  which the client confirms to the request table, so that only that
  party's requests are sent; and the retention of answers
  (../contract/requests.md, 1), from which the table cuts its retries
  (`requests.md`). `me` ends only with the sign-in, or when jig's version
  is not served (below). Its watch refused or ended on the engine ends
  the stream, which the client takes as dropped and opens again after a
  backoff, the link offline until the next hello. A hello alone whose
  ranges serve jig's version is a record out of its order: it drops the
  stream, to open again after its backoff.
- **Its states,** told by `Link(state)` and read with `client.link()`:
  - *signed out:* no sign-in is live (`sign-in.md`, section 1);
  - *starting:* signed in, and no hello has come on the stream;
  - *live:* the stream is live, its hello and `me` with it come;
  - *offline:* the stream dropped, or went silent past twice its
    heartbeat (`../../../protocol/client/wire.md`, section 2), or the
    engine ended it, its `me` refused or ended (above). Requests wait,
    parked, and the stream and its watches open again after
    backoffs drawn from the seed: a first span, doubling to a most,
    jittered.
    The next hello, with `me`, brings the link back, and with it the
    confirmed party's parked requests;
  - *out of date:* the engine serves no version of jig's that the client
    speaks, or another layout of jig's wire than the client's
    (below); or it sent a record of jig's this build can't read
    (`unreadable`, below). It sends nothing, and keeps every request.

  Beside its state, the link names the party confirmed, each declared
  owner whose version the last hello does not serve (below), and a
  sign-out that did not reach the engine (`sign-in.md`, section 1). It names too the
  expiry of the sign-in it holds, `expires`, a wall time, as the last
  sign-in's answer said it (`../../../protocol/client/sign-in.md`): a
  sign-in made or kept by a link, a service's credential or an
  approval's take. It is none for a credential given `Held` until a
  sign-in's answer names one, and none once signed out, the credential
  dropped with it; so an application that keeps a sign-in across its own
  restart keeps its expiry beside its credential, as it likes. A stream refused
  full adds `streams_full { bound, size }`, naming the component or
  transport bound (`../../../protocol/client/wire.md`, 2, and `../../../protocol/client/default-http.md`, 2). `Link(state)`
  tells the caller when this fact arrives even if the state is unchanged;
  it stays through retries until `me` returns or the sign-in ends.
  `watches_full` says that a watch the client opened for an application
  was last answered busy (1) and has had no answer since; `me`'s, which
  the engine opens, ends its stream instead (above).
  It clears when that watch is answered, or is closed or ended. It carries
  no count, since a refusal names none, and tells no `Link(state)`: an
  application reads it when it has cause to, as at a deadline.
  `unreadable` says the link is out of date because the engine sent a
  record of jig's that this build can't read
  (`../../../protocol/client/versions.md`, section 1): a record on the
  stream (the hello, a delivery, a run's text, a watch's end) whose
  bytes, or whose translation, the link refuses as malformed or of
  another version, or an answer to a watch's opening or closing whose
  bytes don't decode. A record refused for a bound or for its order on
  the stream is not one: it drops the stream, which opens again after
  its backoff, offline meanwhile. The link drops the stream
  as for any refused record, then the client is out of date and opens
  nothing again: no stream, no watch, `me`'s included, and no request
  sent, each kept. It holds for the client's life, since no later hello
  comes to lift it; a reload or a restart is a new client. A link out of
  date by the hello's ranges or layouts has it false. So an application can say which it is: a client of another
  version or layout, refused by name, or one that can't read what it was sent.
  Each tells its person to reload or update; neither leaves the person
  offline with no word.
- **Judged from the stream alone,** whatever the application watches. A
  request that got no answer while the link is live is sent again after
  a backoff, and does not by itself make the client offline.
- **Behind is per handle** (1): a presentation that says when what it
  shows may be old reads it from its handles.
- **Versions and layouts, per owner.** The stream's `Opened` states the
  layout of each owner the engine serves, jig's and each application
  owner's; the hello's ranges keep their bytes and no client reads them
  (`../../../protocol/client/versions.md`, section 1). The client speaks the one
  version and layout of each owner its build holds: jig's, its wire's own; and each
  family owner's, which the application declares as it makes the
  client, with its limits (`README.md`, section 11): each owner whose families it
  sends, by its number, with its wire's version and layout (../root.md, section 15).
  Each family's request, read and subject names a declared owner, in
  its declared version; any other is the application's mistake, and
  loud, as a key conflict is (`requests.md`).
  - *Judged at each hello, whole.* A declared owner is not served when
    its `Opened` names another layout of it, or none, as a hello alone
    names none. jig's layout
    judged so makes the link out of date. The judgement holds until the
    next hello, so `client.link()` names
    an owner not served from the hello on, before the application has
    asked anything of it. The client keeps, for each declared owner, its
    judgement; of an owner it does not declare, nothing.
  - *A read waits for the judgement:* none leaves, alone or in a batch,
    until a hello since the stream last opened has judged its owner
    served, as a watch's opening waits for the stream's name; so a read
    asked while the link is starting or offline leaves with the next
    hello, or is answered as its owner was judged.
  - *An answer version not served,* naming the owner, says the same of
    one message.
  - *Not served, a family's owner's* requests stay
    kept and unsent, held for its version (`requests.md`); its reads are
    answered at once, unsent (`README.md`, section 6); its watches end, none opened
    again at that hello; each as version not served. Outside jig's, the
    link is out of date, and its reads are answered so too.
  - *Served again* by a later hello, the client sends what it kept,
    under its keys. Updated, it sends what it kept at a layout its new
    build speaks, and drops what it kept at another (`small-store.md`, section 1).
    `jig-client` and the engine come from one pin of jig in an
    application; how jig is released and pinned is `../../../README.md`'s,
    section 14.
- **A restart of the engine** is a moment offline, then answers by key.
  Provisional text streamed before it, never committed, is dropped when
  the next delivery comes (3).

## 3. A run's text

- **The one stream.** A watch of a run follows a task's live run: its
  attempt; the turn being written, named by its attempt, its number
  within it and its completion's retry; that retry's text so far; and
  the call it is making and the call being written (`writing`); all
  provisional, and marked so (../contract/watches.md, 1.3).
- **Read from an offset, one turn at a time.** The client keeps the
  text it holds of the turn being written, within the bound on a
  provisional turn's bytes, takes each delivery's text past its offset,
  and says `Text { handle }`. A delivery that begins within what it holds
  extends it, marked cut or not, since nothing between them is missing;
  one past it, marked cut, takes its place. One delivery's text is within
  `long_text` (`README.md`, section 11). It holds that one turn alone, the newest
  delivered: a delivery naming a later turn, or a later retry of the
  same turn, takes the place of what it holds, from its own offset; one
  naming an earlier turn, or an earlier retry, changes nothing, and
  nothing is told. Names are ordered by attempt, then number, then
  retry.
- **Text of a later turn never resets an earlier one's.** A later turn's
  text starts under its own name: it is never appended to the earlier
  turn's, nor taken as that text cut back to its start, and what the
  application read under the earlier turn's name stays that turn's.
  Keeping each turn's text until its committed turn takes its place is
  the thread's (`conversations.md`), fed the run's text at each `Text`.
- **Opened again,** the watch starts from the beginning of the turn being
  written: the client keeps what it holds, and takes what lies past it.
- **Committed turns are not in it.** A turn reaches the application
  committed only through its task's activity, read once the task's
  document says the activity has moved (../contract/watches.md, 1.3).
  `jig_client::thread` puts the two together (`conversations.md`).
- **The call being written.** `RunText.writing` is the whole current
  provider id, tool and UTF-8 input suffix, its byte offsets and cut mark.
  Each accepted delivery replaces it whole, including a delivery with no
  new text. Earlier turns or retries change nothing. `call` still names
  the call being made; reasoning has no surface.
- **Only for a run asked for it.** A run whose task is not a conversation,
  or whose host names no call, has no text or no call respectively, and
  its words appear as each turn commits.
