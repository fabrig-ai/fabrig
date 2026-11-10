# Signing in

- **jig owns credentials; the flows that ask for one are the
  application's,** but for jig's three built-ins
  (`../../client/domain/contract/README.md`, section 2): the printed link, for one
  person's machine; a service's credential; and a sign-in approved by a
  browser, for another client of a person signed in there (below).
- **A credential** is a sign-in's number beside 32 random bytes, so the
  engine finds its digest by number (`../README.md`, section 8; `../store/secrets.md`
  ). Credentials come from a cryptographic generator of the
  component's, seeded from the kernel at start.
- **A sign-in, made:** the core hears the identity and commits the
  sign-in, which gives it its number; the component keeps the digest of
  a fresh credential under that number, in the store's secrets' records;
  and only then hands the credential out. The core says each sign-in
  that ends or expires, and the component erases its digest then. A
  sign-in kept by the printed link (below) is no sign-in made: nothing
  is committed, drawn or kept.
- **The application's sign-in.** An application whose people sign in
  another way (an OAuth or OpenID server, a company's directory, a chat
  platform's identity) runs that flow on its own routes, redirects
  included, in its own server, verifies the identity, and gives the
  component `Mint { exchange, identity }`: a provider's number, as its
  configuration numbers it (`../configuration.md`, section 3, providers),
  the subject there, and a login and a name for display. The component
  makes a sign-in for it, as above, and answers `Minted` with its
  credential and the sign-in's expiry; or `MintRefused`, saying why: a
  provider the configuration does not give as one of the application's
  (of kind application), the printed link's and the deployment's being
  jig's own; a login or a name not UTF-8 or past its bound; not ready,
  while the engine stops; busy, at the people child's limits.
  - *From the server only.* `Mint` is a message of the application's
    server code; no message a client sends becomes one, and a transport
    never makes one from a client's bytes.
  - *How the credential travels* is the application's: a cookie its
    server sets, as the default mapping's (`default-http.md`, 3), or a token its client
    holds.
  - *jig keeps nothing of the flow:* what it returned is the
    application's, kept or dropped as its own (`../../domain/people/README.md`,
    section 3).
- **The printed link,** on one person's machine: at start the component
  draws a one-time secret, valid for the profile's `link_span`, and gives it
  to the application's startup, `Link { secret }`, which prints a URL carrying
  it. The URL is the application's; the default mapping's is its link
  route (`default-http.md`, 1). Signing in with the secret signs that person in as the
  deployment's first owner it names, and spends it. Two links are live
  at most, the printed link and the local link (below), each one at a
  time: a sign-in with a secret spent, expired or unknown is answered
  why (below), and the component draws a fresh printed link, which the
  startup prints again, unless the printed link is live or reserved
  (below), or the secret is remembered as a local link's: a native
  client that tries the local link never makes the startup print.
  - *Signed in by it, said:* each time people answers signed in to a
    sign-in whose reserved secret was the printed link's, a sign-in made
    or a held one kept (below), the component says `LinkSignedIn` to the
    application's startup, carrying nothing, so that an application can
    tell its people that the link was used, as a box's front end shows
    (temper's `client/forge/records.md`). The local link, a `Mint`, a
    service's sign-in, an approval's take and every refusal say nothing.
    It is memory alone: what an application keeps of it is gone at a
    restart, as the links are.
  - *The local link,* for the person's native clients on that machine:
    at start, beside the printed link, the component draws a second
    one-time secret from its generator and gives it to the application's
    startup, `LocalLink { secret }`. The startup keeps its URL, made as
    the printed link's, where only the person's own clients on that
    machine read it, and never prints or logs it; an application that
    keeps none drops it. It is the printed link's kind: it signs in the
    deployment's first owner by the same route and spends itself, lives
    `link_span` from its drawing, and all this section says of the
    printed link holds of each of the two apart, so that signing in by
    one, reserving it or its expiring leaves the other as it was.
    - *Drawn again at once* when it is spent, by a sign-in or a refusal
      (below), and given again, so that the next native client signs in
      the same way; while the engine closes or aborts, none is drawn.
    - *Expired,* found by its deadline or by a sign-in, it is remembered
      expired, and the next sign-in whose secret isn't live, whatever
      secret it tried, draws a fresh local link, unless one is live or
      reserved.
    - *Its first, given or drawn:* the startup may give the first local
      link's secret, `local_secret`, as it gives the printed link's
      (*A secret given*, below); every later one is drawn. Where the
      printed link isn't enabled there is no local link either. A restart
      forgets both links, and the startup keeps the local link the new
      component gives.
    - *Its exposure* is where the startup keeps it: whoever can read
      that can sign in as the owner, once, within its span, as whoever
      can read the printed link's output can, so it is kept as privately
      as the machine's other secrets. It adds nothing to the printed
      link's exposure, which stays as it is.
  - *Why a link can't sign in.* The component keeps, in memory, the digests
    of its last 8 links no longer live, printed or local, each marked used
    or expired and with which of the two it was; a restart forgets them. A link's sign-in whose secret isn't live is
    answered by what it finds: `link_used`, a link a sign-in spent, signed
    in or refused; `link_expired`, one past its expiry; `link_unknown`, any
    other, whether another deployment's link, one past those 8, or bytes
    that were never a secret. The three differ only for whoever holds a
    secret this component drew, so they tell a stranger nothing. They leave
    the live link as it is, and none is sent again. Where the printed link
    isn't enabled, no link is any of these, and the answer stays refused.
  - *How long it lives.* The small profile's `link_span` is 2 hours, because
    a link may be opened late on a person's own machine: they start a box,
    leave it, and come back to its terminal. It is one value, so a server's
    printed link takes it too, and there it is a secret live for longer in
    what that server wrote, its terminal or the log it goes to. What holds it
    is that it is a one-time link, spent by its first sign-in, on the machine
    that printed it: whoever can read that output on that machine within the
    2 hours can sign in as the owner, if the owner hasn't. So that output is
    kept as privately as the machine's other secrets, and a link in doubt is
    spent by opening it.
  - *A secret given.* An application's startup may draw the first link's
    secret itself, 32 bytes from the platform's randomness, and print its
    URL before the engine starts. It gives the secret in the component's
    settings (`link_secret`), and the component takes it at start in the
    draw's place: its `link_span` runs from that taking, it says no
    `Link`, since the startup holds it, and all below holds of it as of a
    drawn one. It is the first link's alone: every later link is drawn,
    and a component started with none given draws its first. A secret of
    any other length is refused at start, as unusable settings; one given
    where the printed link is not enabled is dropped, no link being live.
    The first local link's secret may be given alike, `local_secret`, so
    that the startup can hand it to a native client it starts itself, as a
    box hands its front end one (temper's `protocol/setup/box.md`, 6): taken
    in the first local link's draw's place, its `link_span` from that
    taking, no `LocalLink` said for it, spent and drawn again as any local
    link; refused at start for any length but 32 bytes, and dropped where
    the printed link is not enabled. The two given are two links: one
    equal to the other is refused at start too.
  - *The secret is the link's, or its exchange's.* A sign-in that finds
    the secret live, the printed link's or the local link's, before its
    expiry, reserves that link for its exchange until people answers.
    While it is reserved, a sign-in with the same secret is answered
    *busy* and changes nothing, and one with a secret that isn't live is
    answered why (above) and draws no fresh link of the reserved one's
    kind: an attempt that crosses the exchange never replaces its secret.
    The other link stays live meanwhile, and a sign-in by it goes on. A sign-in the component has
    no room for among its exchanges is answered busy, and the secret
    stays the link's.
  - *Not ready, busy and full spend nothing.* The reserved secret is the
    link's again, with the expiry it was drawn with, so the same link
    signs in when sent again, until that expiry, checked again then.
    Signed in, refused and signed out spend it, as do the core's refusal
    before people answers and the engine's close or abort, which end the
    exchange; a spent secret is the link's no more: the next sign-in by
    any other draws a fresh printed link, and a spent local link is drawn
    again at once (above).
  - *Full* (`../../domain/people/README.md`, section 3) carries the deployment's
    room for sign-ins, the earliest saved expiry among them, and the
    reserved link's own expiry, both wall times: the person reads when the next
    room comes, and until when this link serves, past which a fresh one
    is drawn when it is tried. The link's expiry is kept on the
    monotonic clock, and full says it at the answer's clocks. Full is the
    printed link's answer alone: a `Mint` and a service's sign-in are
    answered busy for it. On the wire, a sign-in's answer is signed in,
    refused, not ready, busy, full with those three, `link_used`,
    `link_expired` or `link_unknown`.
  - *A sign-in held is kept* (`../../domain/people/README.md`, section 3). The
    link's exchange carries a credential as every exchange may, a
    browser's cookie or a native client's bearer (`default-http.md`, 3). Once the secret
    is found live and reserved, the component checks that credential as
    any exchange's, its digest under its number and its saved expiry,
    and names its sign-in to the core as held; one that fails the check
    is no refusal, and the link signs in afresh. When people answers
    with the held sign-in, the component draws no credential and keeps
    no digest: its answer is signed in, with that sign-in's party and
    saved expiry, carrying no credential, and sets no cookie, since the
    client already holds it. The secret is spent, as by any sign-in.
  - *Reuse extends nothing:* the kept sign-in's expiry, the cookie's
    `Max-Age` and the bearer's file stay as they were.

  It is enabled by configuration (`../configuration.md`, section 3,
  clients), and only where the application's clients' listener is on
  loopback, which its startup checks (`../service.md`, section 3).
- **A sign-in approved by a browser,** for that browser's own party:
  a client that can't open the printed link, a program in a terminal
  on the person's machine, asks; the person's browser, signed in by
  its cookie, approves; and the client takes the sign-in with the
  secret its ask was answered with. Approving makes nothing: only the
  take makes the sign-in and draws its credential. Until then the
  approval is the component's, in memory, as the printed link's state
  is: nothing is committed, kept or written, and a restart forgets
  every approval.
  - *Asked:* `ApprovalAsk`, with no credential. The component draws
    32 bytes from its generator, the approval's secret, and a code of
    8 letters of `BCDFGHJKLMNPQRSTVWXZ`, twenty consonants, none like
    another or like a digit and no vowel, so no word, each drawn
    uniformly, the code none of the live approvals has. It answers
    `asked` with the secret, the code and the approval's expiry, its
    wall time, 10 minutes on; and keeps the secret's digest, never the
    secret, the code, the wall time it was asked and its expiry on the
    monotonic clock. The secret is the client's alone, never shown;
    the code is for the person to compare, on the client and in the
    browser. At most 4 approvals are live at once in the deployment,
    whoever asked, and an ask past them is answered busy until one
    ends. Where approval isn't enabled, an ask is refused; while the
    engine stops, not ready.
  - *Read and decided by a browser alone:* `ApprovalRead { code }` and
    `ApprovalDecide { code, approve }`, each sent with a credential the
    transport carried itself, a browser's cookie (`default-http.md`, 3). The transport
    says that of each exchange it hands the component (`wire.md`, 2.2); any other
    exchange, a bearer's among them even beside the browser header, is
    answered `not_a_browser` before anything is read, so that a
    program holding a person's credential, an agent driving the
    application's command line among them, never approves. The
    credential is checked as any exchange's (above), and one that
    fails is answered `signed_out`. A read answers what the approval
    is: waiting, with the wall time it was asked; approved, taken,
    refused or expired; or `unknown`, for a code no approval live or
    remembered (below) has. It reads by code alone, and nothing lists
    the approvals that wait. A decision spends the approval: approving
    marks it approved, its approver the deciding sign-in, and makes
    nothing; refusing ends it, refused. Any decision after the first is
    answered `used`, one past its expiry `expired`.
  - *Taken:* `ApprovalSignIn { secret }`, with no credential. An
    approval still waiting is answered `waiting`, and the client asks
    again; refused, expired, taken already (`used`) and unknown are
    answered so, the secret's digest compared in constant time. An
    approved one is reserved for the take's exchange, and a take with
    the same secret meanwhile is answered busy. The component asks
    people whose the approver's sign-in is (`Whose`,
    `../../domain/people/README.md`, section 3) and signs that identity in, held
    none, as it signs in the printed link's owner: a sign-in made, its
    credential drawn and its digest kept (above), and handed out in
    that answer alone, never again. Its party is the approver's, and
    no other party's sign-in comes of it. The approval is then taken.
    An approver whose sign-in has ended by then signs no one in: the
    take is refused, and the approval spent. Not ready, busy and full
    spend nothing: the approval stays approved until its expiry, and
    full carries the deployment's room, the earliest saved expiry among
    its sign-ins and the approval's own expiry, as the link's full does.
  - *Never taken, nothing made:* an approval approved and not taken
    before its expiry, its client gone, ends expired with no sign-in
    made, and nothing past its expiry is taken. The sign-in a take makes
    counts against the deployment's sign-ins as any does (*Full*,
    above), only once it is made.
  - *Remembered:* the component keeps, in memory, its last 8 approvals
    ended, each by its secret's digest and its code, marked taken,
    refused or expired, so that a take, a read or a decision of one is
    answered why; past those 8, and after a restart, an approval is
    unknown, as a link is.
  - *Its risk:* a client may carry the code in the browser's address,
    so that one press approves, and the person's check is that the
    code their own client shows is the code their browser shows. On
    loopback, whoever could ask an approval and send its address to
    the person is on the engine's machine already, as whoever can read
    the printed link is.

  It is enabled by configuration apart from the printed link
  (`../configuration.md`, section 3, clients, `approval`), and only where
  the application's clients' listener is on loopback, which its startup
  checks as for the link (`../service.md`, section 3). Over TLS, between
  hosts, it is later (`README.md`, section 12).
- **A service's credential** is issued when an owner makes the service:
  the component draws it, keeps its digest in the secrets' store under
  the service's number (`../store/secrets.md`), and answers the owner's
  request with it once, never again.
  - *Its shape* is a sign-in's credential's: the service's number beside
    32 random bytes.
  - *To its owner:* the answer carries it beside the service's party; the
    client hands it to the application once, as a secret, and never
    keeps it in its small store; the application shows it once.
  - *Signing in with it:* the service's client signs in as a service,
    the service's credential as the message's credential, and takes
    back a sign-in's credential, with which it makes every exchange. The
    core commits that sign-in as any other, the provider the deployment
    and the subject the service (`../../domain/people/README.md`, section 3); it
    expires as any, and the client signs in again. So exchanges are
    checked one way, by sign-in, and the service's credential only at
    sign-in.
  - *Once:* a request made again by its key is answered as it was, its
    party, and no credential, since the engine keeps only its digest. An
    owner who lost it reissues it (`../../domain/people/README.md`, section 3).
  - *Replaced* when an owner reissues it: a new credential is drawn, its
    digest replaces the old, durable before the answer, which carries it
    once; the sign-ins made with the old end.
  - *Ended* when an owner removes the service: its digest is erased and
    its sign-ins end.
- **Where a sign-in's credential is held:** by the client, as a secret,
  when its transport needs it in hand; the application keeps it across a
  restart, where its platform keeps secrets, and gives it back as held
  (`../../client/domain/client/README.md`, section 9). A transport may keep it
  itself, as a browser's cookie the client never sees: a sign-in's
  answer then carries none, and every exchange leaves with none, the
  transport adding it (`default-http.md`, 3). A link's answer that carries none to a
  client that sent the sign-in with a credential in hand names the
  sign-in kept: the client goes on holding that credential.
- **Signed out:** a credential the engine does not know, or a sign-in
  ended or expired, is answered signed out on every exchange and ends
  the stream, and the client reports signed out to the application
  (`../../client/domain/client/README.md`, section 9). Signing out is a message
  that ends the sign-in whose credential it carries, its digest erased;
  a transport that keeps the credential clears it then. Sent with a
  credential whose sign-in has ended, it is answered signed out like
  any exchange, so a client may send it again after it ended lost; a
  lost one clears nothing on the client's side. Its answer, signed out
  or lost, is the sign-in's whose credential it carried: a client that
  has since authenticated a newer sign-in drops it, and keeps the newer
  credential (`../../client/domain/client/README.md`, section 9).
- **Installing a credential** ends every pending read visibly in the
  client domain (`README.md`, section 4), including `Held` or `Transport` before
  the first `me`. A `Link` or `Service` exchange that fails, is refused,
  is lost, is not ready, busy or full and installs no credential leaves old reads
  pending; reads made during it still belong to the old credential.
  A read made under that credential cannot answer under its replacement.
- **While the engine stops** (`../../domain/engine/restart.md`, 1), a sign-in, a
  sign-out and a `Mint` are answered not ready, as a keyed request is:
  nothing is committed, kept or erased, and a link's secret is not
  spent. Not ready is *again* (`README.md`, section 3): a service's client signs in
  again after its backoff, reaching the next engine; a person's sign-out
  shows as one that did not reach the engine
  (`../../client/domain/client/README.md`, section 9). A refusal would tell the
  person their sign-in failed, and an answer with no record would be
  *lost*, which cannot tell a sign-in not taken from one taken whose
  answer was lost.
