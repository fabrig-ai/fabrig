# The client: vocabulary toward the application

## 1. Its vocabulary toward the application

```
jig_client::Client: step(now, wall, Event, &mut Queue<Request>), fire(now, wall, &mut Queue<Request>), settle(now, wall, &mut Queue<Request>), next_deadline()

Event, in
  from the application:
    Start { kept }                                the small store's records, as the host loaded them
    SignIn { with } | SignOut                     with: Link { secret, held } | Service(credential) | Held(credential) | Transport
    Watch { handle, subject, window } | Unwatch { handle }
    Read { handle, read, cursor }
    Send { key, name, request }                   an operation, a compound request, or a family's request as bytes; name optional
  from the transport:
    Reply { exchange, message } | Part { exchange, message } | Done { exchange } | Lost { exchange }
    Streamed { message } | StreamGone
    Stored | NotStored
Request, out
  to the transport:   Exchange { exchange, credential, message } | OpenStream { credential, replaces } | CloseStream | Save { records } | Erase { records }
  to the application: Link(state) | Changed { handle } | Ended { handle, why } | Page { handle, page }
                      Pending { key } | Answered { key, outcome } | Kept { key, request } | Dropped { count } | Text { handle }
read by the application: client.document(handle), client.text(handle), client.pending(key), client.link()
```

- **`now` and `wall`** are the monotonic instant and the wall time, as
  every step machine takes them (skein/docs/foundation/programming-model.md, section 9:
  `env.now` and `env.wall`).
- **Handles and keys are the application's.** Each names its asker and a
  number of the asker's. The asker is the owner number of the child of
  the client root that asked (../root.md, section 15; jig's reference UI's
  is 65536): the client keeps it, hands it back with everything it says
  of that handle or key, and never reads it, as skein-ui keeps an owner
  (skein's `ui/tree.md`, 1.3). The client root routes by it, and keeps no map
  from handles or keys to its children (../root.md, section 5).

### 1.1 What it hears

- **From the application:**
  - *`Start { kept }`:* the small store's records, as the host loaded
    them, before any other event (`small-store.md`, section 1);
  - *`SignIn { with }` and `SignOut`* (`sign-in.md`, section 1);
  - *`AskApproval { port }`, `ReadApproval { code }` and `DecideApproval { code,
    approve }`:* an approval's ask, read and decision (`sign-in.md`, section 1);
  - *`AskLink`:* an asked link's ask (`sign-in.md`, section 1);
  - *`Watch { handle, subject, window }` and `Unwatch { handle }`:* a
    subject of jig's (../contract/watches.md, 1.1), or a family's as bytes, and the
    window its document holds (`watches.md`);
  - *`Read { handle, read, cursor }`:* one document, or one page of a
    list from a cursor (`README.md`, section 6);
  - *`Send { key, name, request }`:* an operation, a compound request,
    or a family's request as bytes, with a name or none (`requests.md`).
- **From the transport:**
  - *`Reply { exchange, message }`:* the one record that answers an
    exchange; *`Part { exchange, message }`:* one record of a batch's
    answer, a member's reply, as it comes; *`Done { exchange }`:* a
    batch's answer ended whole after its parts
    (`../../../protocol/client/wire.md`, 2.3); *`Lost { exchange }`:*
    none came whole, or a batch's answer was cut short;
  - *`Streamed { message }`:* one event of the stream, in order: the
    hello, a delivery, a heartbeat, a watch ended; *`StreamGone`:* the
    stream ended;
  - *`Stored` and `NotStored`:* the last `Save` or `Erase` written, or
    not (`small-store.md`, section 1).
- **Time:** `fire(now, wall, ..)`, at the deadline `next_deadline()` gave
  (`README.md`, section 10).
- **The settling:** `settle(now, wall, ..)`, once the client root has
  routed every event of its iteration and what followed from it
  (../root.md, section 3). Linked, `step` and `fire` give the transport
  no `Exchange`, `OpenStream` or `CloseStream`: the link gathers them,
  in the order they were asked for, and `settle` gives them, each
  gathered read, watch's opening and watch's closing in one batch while
  more than one is gathered (`../../../protocol/client/wire.md`, 2.3).
  `Save` and `Erase` leave from `step` and `fire`, as before, so a kept
  request's record is still stored before its exchange leaves (`small-store.md`, section 1
  ). Unlinked, `settle` gives nothing.

### 1.2 What it asks for

- **Of the transport:**
  - *`Exchange { exchange, credential, message }`:* one of jig's messages
    (`../../../protocol/client/wire.md`, 2.1) with the credential held, or
    none, a batch of reads and watches among them; answered by one
    `Reply`, by a batch's `Part`s then `Done`, or by one `Lost`;
  - *`OpenStream { credential, replaces }` and `CloseStream`:* the one
    stream every watch's deliveries come on; `replaces` names the last
    stream this client knew, the one its small store kept at start and
    then its own, or none (`../../../protocol/client/wire.md`, section 2);
  - *`Save { records }` and `Erase { records }`:* the small store's,
    written before the requests they keep are sent (`small-store.md`, section 1).
- **Of the application,** told, never asked to act:
  - *`Link(state)`:* the link's state, whenever it changes, and when a
    sign-out does not reach the engine (watches.md, 2, and sign-in.md, 1): what a
    presentation shows of the link, which it reads with `client.link()`;
  - *`Changed { handle }`:* a newer document for the handle, or the
    handle behind or live again (`watches.md`, 1);
    *`Text { handle }`:* more of a run's text (`watches.md`, 3); *`Ended { handle,
    why }`:* the handle's one terminal, and why (`watches.md`, 1);
  - *`Approval(answer)`:* an approval's ask, read or decision answered,
    once each (`sign-in.md`, section 1);
  - *`Page { handle, page }`:* a read's one terminal (`README.md`, section 6);
  - *`Pending { key }`:* the request's progress changed; *`Answered {
    key, outcome }`:* its one answer; *`Kept { key, request }`:* a
    request restored at start (`requests.md` and `small-store.md`, section 1); *`Dropped { count
    }`:* how many kept records were dropped at start, unread, under no
    asker (`small-store.md`, section 1).

### 1.3 What it never asks

- **It never calls the application.** What it tells is a value in the
  queue, which the application takes when it steps it. What it holds,
  the application reads:
  - `client.document(handle)`: the handle's latest document, and its
    state (`watches.md`, 1);
  - `client.text(handle)`: the run's text it holds for the handle, the
    one turn being written (`watches.md`, 3);
  - `client.pending(key)`: the request's progress, its party and its
    owner (`requests.md`);
  - `client.link()`: the link's state, and the credential held (watches.md, 2, and sign-in.md, 1
    ).
- **It never reads an application's value.** A family's request, read,
  subject, page and document are bytes under their owner's number, which
  the application's wire crate encodes and decodes; the client reads
  their owner, their version and their size, and nothing more. A part
  that one of jig's documents names stays a reference (../root.md, section
  6).
- **It never decides for a presentation:** what to show, when to
  confirm, what to keep of a draft. It gives the facts as data: the
  documents with their revisions and offers (../contract/documents.md, 3), and each
  request's progress and answer.

### 1.4 Without a link

`Client::unlinked` makes a client that holds no link: its host carries
the domain's work in the link's place, and answers it with `Up`. Each
mode is stepped by its own pair, and each pair has its own request type:

```
Client::new:      step(now, wall, Event, &mut Queue<Request>), fire(now, wall, &mut Queue<Request>)
Client::unlinked: step_unlinked(now, wall, Event, &mut Queue<UnlinkedRequest>),
                  fire_unlinked(now, wall, &mut Queue<UnlinkedRequest>)

UnlinkedRequest, out
  Told { told }               as a linked client's
  Authorized(authorized)      a printed link's authorization, always lost: no link signs in
  ApprovalAct(act)            an approval's ask, read or decision, which the host carries (section 12)
  Approval(answer)            its answer, as the host gave it back
  AskedLink(asked)            an asked link's answer, always lost: its host carries no ask (section 9)
  Down { down }               the domain's work, which the host carries: a save, an erase, a send,
                              a watch opened, continued or closed, a read
  SignOut                     the sign-out, which the host answers
Event, in, besides the application's: StartUnlinked { kept } in place of Start, Up { up }, SignOutLost,
  ApprovalAnswer { answer }
```

- **A linked client's `Request` holds no `Down` and no `SignOut`:** what
  its domain asks goes to the link, and its sign-out is an exchange. A
  linked host matches only what a linked client asks.
- **A pair given the other mode's client is a fault** (an assertion), as
  an event of the other mode is.
- **An application root stepping either mode** keeps the two apart in its
  own requests: what the linked client asks, and what the unlinked one
  asks, each under its own variant.
