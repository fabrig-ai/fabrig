# The client: signing in

## 1. Signing in

- **jig owns credentials, not sign-in flows** (../contract/README.md, section 2).
  Its own three ways of signing in are the printed link's one-time
  secret, on loopback, a service's credential, and a sign-in approved
  by a signed-in browser (below). Every other way is
  the application's sign-in, on its own routes, ending in a `Mint` its
  server gives the clients' component (`../../../protocol/client/sign-in.md`
  ).
- **`SignIn { with }`:**
  - *`Link { secret, held }`:* the printed link's secret, beside the
    credential the application holds from an earlier sign-in, if any.
    The client sends the component's sign-in with the secret, carrying
    `held` as its credential, and holds the credential the answer gives;
    an answer that gives none keeps `held`, the sign-in the engine kept
    (`../../../protocol/client/sign-in.md`), unless the transport keeps
    it: a browser's `held` is none, its cookie carried by the transport.
    An answer refused, not ready, busy, full, why a link can't sign in
    or lost installs nothing, and `held` stays the application's to give
    again. Not ready, busy and full leave the link's secret usable until
    its expiry (`../../../protocol/client/sign-in.md`), and the link's
    view says which, as `not_signed_in`: not ready, busy, or full with the
    deployment's room for sign-ins, the earliest saved expiry among
    them and the link's own expiry, for the application's words. It says
    too why a link can't sign in, `link_used`, `link_expired` or
    `link_unknown`, as the answer did, and these are not sent again. The client keeps no secret and
    signs in again by itself never: the person's retry is the
    application's, the same `Link { secret, held }` given again, as a
    native shell does, or the printed link opened again, as a browser
    does. The next `SignIn`, and signed in, clear `not_signed_in`;
  - *a service's busy* ends its signing as a refusal does: a service
    waits for no room, and is never answered full;
  - *`Service(credential)`:* a service's credential, sent the same way,
    and answered with a sign-in's credential, held the same way;
  - *`Approval { secret, grant }`:* an approval's take, its secret the
    one `Asked` gave (below), and `grant` the grant its loopback return
    brought, or none, sent with no credential. Answered signed in,
    the client holds the credential the answer gives, as a link's. While
    it is answered waiting, the client sends the same take again by
    itself after a second, `APPROVAL_AGAIN`, and tells `not_signed_in`
    `Approval(Waiting)`; answered `not_granted`, a grant that doesn't
    match, it tells `not_signed_in` `Approval(NotGranted)` and sends the
    take again after a second without the grant, as while it waits, the
    approval left as it was or ended by then; not ready, busy and lost take the backoff a
    link's sign-in takes, and `not_signed_in` says not ready for not
    ready and busy for busy and for lost, so its application tells a
    lost take from one not yet answered. Any other answer ends it,
    installing nothing:
    `not_signed_in` says `Approval` with why (refused, expired, used or
    unknown), or full with the deployment's room, the earliest saved
    expiry and the approval's own expiry in place of the link's. A next
    `SignIn` or a `SignOut` ends it too. The client keeps the secret in
    memory while it takes, never in the small store, and drops it at
    the end;
  - *`Held(credential)`:* a credential the application obtained by its own
    sign-in, which the client holds as it is;
  - *`Transport`:* the transport carries a credential the client never
    sees, as a browser carries a cookie its server set.

  The `credential` on `Exchange` and `OpenStream` is the one held, or
  none; a `Link` sign-in's is its `held`. Signed in, the client opens
  the stream, whose hello brings `me` (`watches.md`, 2), and opens the watches asked before
  (`watches.md`, 1); under `Transport` it is signed in from its `SignIn`. So a host
  may sign in after `Start { kept }`, its children watching from their
  start.
- **An approval, asked and decided** (`../../../protocol/client/sign-in.md`
  ), each one exchange whose one answer the client tells the
  application, `Approval`, and keeps nothing of:
  - *`AskApproval { port }`,* sent with no credential, by a client
    signed out, `port` its loopback return, `Option<u16>`, or none:
    `Asked { secret, code, expires }`, the secret the application's to
    give back in `Approval { secret, grant }` and never show, the code
    to show; or `NotAsked`, refused, not ready, busy or lost;
  - *`ReadApproval { code }` and `DecideApproval { code, approve }`,*
    sent with the credential held, or none under `Transport`, by a
    browser's client: `Read { code, state }`, the approval waiting,
    `Waiting { asked, kind, source, reader }`, with when it was asked,
    its kind, the wire's `ApprovalKind`, its source and the reader's,
    each an `Option<IpAddr>`, an IPv4-mapped address read as its IPv4
    one; approved, taken, refused, expired or unknown; `Decided { code,
    decided }`, approved, `Approved { loopback }`, its loopback return's
    `Loopback { port, grant }` where the ask named one, the grant a
    `Secret` the application hands to the deciding browser alone and
    never shows or logs, refused, used, expired or unknown; either not
    a browser, signed out, not ready, busy or lost. Answered signed
    out, the client is signed out, as by any exchange (below);
  - *the code* is `ApprovalCode`, the wire's (`../../../protocol/client/sign-in.md`
    ): eight letters of its alphabet, read from what a person
    types or an address carries, either case, hyphens and spaces
    ignored, and shown as four letters, a hyphen and four.
- **An asked link** (`../../../protocol/client/sign-in.md`, the asked
  link), one exchange whose one answer the client tells the
  application, `AskedLink`, and keeps nothing of: *`AskLink`,* sent with
  the credential held, by a client signed in, or with none under
  `Transport`, is answered `Asked { secret, expires }`, the secret the
  application's to give the person's browser in the printed link's URL
  (`../../../protocol/client/default-http.md`, 1) and never show or log,
  `expires` the link's wall time; `NotAsked`, refused, not ready, busy
  or lost, as an approval's ask; or `SignedOut`, the client then signed
  out, as by any exchange (below). A client not signed in sends it with
  no credential, and the engine refuses it. A client with no link
  answers it at once, `NotAsked` lost (`vocabulary.md`, 1.4). At most one ask is in
  flight: another meanwhile is answered `NotAsked` busy, unsent.
- **The credential is never in the small store.** It is the link's, and
  `client.link()` gives it: a host that keeps a sign-in across its own
  restart keeps the credential where its platform keeps secrets, and
  gives it back with `Held`.
- **Signed out:** an answer, a read's terminal, a watch's end or the
  stream saying so drops the credential held, ends every watch handle
  (`Ended`, signed out), visibly ends every pending read (`README.md`, section 6),
  parks every request, and tells `Link` signed out. The application
  signs in again as it likes, and the parked requests are sent once the
  same party is confirmed, within the
  retention (`../../../domain/people/acts.md`, 1).
- **`SignOut`** sends the component's sign-out with the credential held,
  or none under `Transport`, to end the sign-in on the engine. Answered
  signed out, the client drops the credential and is signed out, as
  above.
- **A sign-out that does not reach the engine** (lost, or answered not
  ready while the engine stops) leaves the party signed in, as they
  still are on the engine: the client keeps the credential and its
  handles, and the link says the sign-out did not reach the engine.
  `SignOut` again sends it again, which is safe: a sign-out whose
  sign-in has already ended is answered signed out, the end asked for.
  Signed out meanwhile, whatever ended the sign-in, the client is signed
  out, as above, and the link no longer names the sign-out.
- **A sign-out's answer is its sign-in's.** The client knows which of
  its sign-ins each exchange belongs to. Once a newer sign-in is
  live, an earlier one's sign-out answer is dropped, and the newer
  stands. Nothing else the earlier sign-in owes reaches the newer one:
  going signed out parks every request, visibly ends every pending read
  and ends every watch handle. Successful sign-in also ends all pending
  reads before opening its stream, even before the first sign-in's `me`.
  Thus an earlier attempt's answer is stale to the request table, a
  completed or cancelled read's late terminal fails its token match,
  and an ended watch's deliveries are dropped by their names (`requests.md` and `README.md`, section 6 and `watches.md`
  ). The link erases old exchange names; the domain owns read
  cancellation, with no new link output, public `Readout` variant or
  wire record.
- **A service's credential, when an owner makes one,** reaches the
  application once, in that request's `Answered` outcome; the client
  keeps no copy of it, and the small store holds no answer
  (`../../../protocol/client/sign-in.md`).
- **A bot relaying a person's decision** holds that person's own
  credential, from the application's sign-in (`Held`), and the decision
  is the person's request, naming the revision the bot showed them: jig
  adds no acting for another (`../../../domain/people/README.md`, section 3).
- **Keys are the party's across sign-ins,** so the parked requests are
  sent again once the same party is confirmed, within the engine's
  retention.
