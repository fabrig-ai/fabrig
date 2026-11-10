# The default HTTP mapping

jig's option for the transport: the component's messages and the
link's, onto HTTP, both ends. An application uses it, wraps it, or
writes its own transport; neither end decides anything of jig's model,
and the component below knows none of it.

## 1. Routes

Under the prefix the application mounts it at, the same paths at every
prefix; jig's test application mounts it at `/jig`.

| Route | Message |
|---|---|
| `GET <prefix>/v1/stream` | opening the stream, answered with server-sent events (2) |
| `POST <prefix>/v1/request` | a keyed request |
| `POST <prefix>/v1/read` | a read |
| `POST <prefix>/v1/watch` | opening a watch |
| `POST <prefix>/v1/unwatch` | closing a watch |
| `POST <prefix>/v1/stream-gone` | a stream gone |
| `POST <prefix>/v1/sign-in/link` | signing in with the printed link's secret |
| `POST <prefix>/v1/sign-in/deployment` | signing in as a service, its credential as the bearer |
| `POST <prefix>/v1/sign-in/approval` | taking an approval with its secret (`sign-in.md`) |
| `POST <prefix>/v1/approval/ask` | asking an approval |
| `POST <prefix>/v1/approval/read` | reading an approval by its code |
| `POST <prefix>/v1/approval/decide` | approving or refusing an approval by its code |
| `POST <prefix>/v1/sign-out` | signing out |
| `POST <prefix>/v1/authorize` | an OpenID authorization (`openid.md`) |
| `GET <prefix>/v1/public/<owner>/<version>/<kind>[/<id>[/<id>]]` | a public read (5, `public-reads.md`) |

- **A POST's body** is its message's record, `application/octet-stream`.
  Its answer is `200` with the reply's record and `Cache-Control:
  no-store`, whatever jig's answer says: a refusal, signed out and a
  version not served are records. A POST whose `Content-Type` is not
  `application/octet-stream` is `415`, a body that is not a record of
  its route `400`, and one past the route's bound `413`, each with
  nothing done; any other method or path under the prefix is `404`, but
  a `GET` of the link's route (below) and the public reads' (5). The guards' answers are the
  example guards' (`../browser/README.md`, 12.2).
- **The printed link's URL** is `<prefix>/v1/sign-in/link?secret=…` on
  the loopback address the application's server listens on, which the
  application's startup prints. A `GET` of it, a browser's opening the
  link, spends nothing and draws nothing, since any other site can make
  a person's browser send a `GET`: the engine end leaves it to the
  application's page, which the server answers there, with
  `Referrer-Policy: no-referrer`; the shell takes the secret out of the
  address and `POST`s it, with the browser header, which signs in
  (`../browser/README.md`, section 11). A native client sends the secret by the
  same `POST`.
- **No provider route.** The application's sign-in runs on its own
  routes; after a `Minted`, its server sets the credential's cookie on
  its own answer with the mapping's function for it (3). So do the
  OpenID provider's (`openid.md`): its discovery document, key set, token
  and userinfo routes are the application's, and only `Authorize`, a
  client's message, is the mapping's.

## 2. The stream

- **Server-sent events,** `text/event-stream`, `Cache-Control: no-store`:
  the hello first; each streamed message one event, whose data is its
  record's bytes in base64; a heartbeat a comment line. No event names
  and no ids: a record says what it is, and a client reopens by its own
  backoffs, never by the event stream's retry.
- **Paced by its connection.** The engine end says `StreamTaken` once an
  event's bytes are in the connection's send buffer, bounded by the
  mapping's limits; a client that reads slowly fills it, and nothing
  more is handed out until it drains.
- **The client end** reads the body as it comes, a piece at a time;
  skein's server-sent events reader cuts it into events, each decoded
  from base64 into a `Streamed` message. A `401` to its `GET` is the
  stream refused signed out, the engine end's `StreamEnded` signed out
  before the stream opened (`wire.md`, 2.2); a `409` whose body names a bound is
  the stream refused full (`wire.md`, section 2), told to its caller; any other
  status but `200`, the body's end, or a failure, is `StreamGone`.
  The full body's record is `StreamsFull { bound, size }`, with bound
  `component` or `transport` and size a `u32`; it is read within its
  fixed encoded bound before being told. A missing, malformed or trailing
  body is gone. The full fact stays beside the client's link state while
  it retries, until `me` returns or the sign-in ends.

## 3. The credential

- **As a cookie,** for a browser: set by the engine end on a sign-in's
  answer, which then carries no credential in its body, or by the
  application's server after a `Minted`, through the engine end's
  function that sets the same cookie: `jig-sign-in=<the credential in
  base64url>; Path=<the prefix>; HttpOnly; SameSite=Strict;
  Max-Age=<the sign-in's remaining life>`, and `Secure` on an exchange
  that came by the public name (below), never on one that came by the
  listener's own address (why each: `../browser/README.md`, section 18). Its life is
  the sign-in's, from the sign-in's expiry, which a `Minted` carries
  (`wire.md`, 2.2); signing out clears it, with `Max-Age=0`. The engine end reads
  it as each exchange's credential, the link's sign-in's among them: a
  sign-in kept by the link answers with no credential, so it sets no
  cookie, and the browser's stays as it was (`sign-in.md`); nor does a
  sign-in refused, not ready, busy or full.
- **The public name,** for an application behind a TLS proxy: its
  HTTPS origin's host, with its port where it is not 443, such as
  `app.example.com`, as a browser sends it in `Host`. The
  application gives it the mapping when it mounts it, beside the
  prefix, or gives none; it is no member of jig's configuration, as
  the listener is not (`../configuration.md`, section 3). The `Host`
  allow-list holds it beside the listener's own addresses
  (`../browser/README.md`, 12.2), and each exchange is decided by its `Host`
  alone, byte for byte as the request sent it, with no case folded and
  no port normalised: one that came by the public name came through the
  proxy, over TLS, and its cookie is `Secure`; one that came by the
  listener's address came in plaintext, and its cookie is not. An
  exchange that names the public name to the loopback listener
  directly, with no proxy, gets `Secure` too, which is harmless: the
  browser keeps no such cookie over plaintext, or never sends it back
  there. Every answer that
  sets or clears the cookie takes that decision: the engine end's at a
  sign-in and a sign-out, and the application's after a `Minted`,
  whose function takes the decision the engine end reads from the
  application's own exchange.
- **As `Authorization: Bearer`,** everywhere else: the credential in
  base64url, and a service's credential on its sign-in route the same
  way.
- **The browser header,** `Jig-Client: browser`, on every request a
  browser makes under the prefix, the stream's `GET` among them, added
  by the browser's transport: in jig's browser parts, the client's
  adapter (`../browser/README.md`, 5.1 and section 11). The engine end checks it
  (`../browser/README.md`, 12.2): an exchange carrying it is a browser's, its
  sign-in answered with the cookie; any other, with the credential in
  its answer's body.
- **The credential's carrier, told to the component:** the engine end
  says an exchange is a browser's (`wire.md`, 2.2) when its credential came by the
  cookie, which the guard lets through only beside the browser header;
  never when it came as a bearer, with the header or without, since a
  valid bearer is read first. That is what reading and deciding an
  approval admit (`sign-in.md`); a take's `signed_in` is answered to a
  browser as a link's is, the cookie set and no credential in the body.

## 4. Its two ends

- **`jig-client-protocol-http`,** the client end, step code for the host
  and for wasm32: an `Exchange` as a method, a path under the prefix,
  headers (its content type, and the bearer where the client holds a
  credential) and a body, with no browser header, which a browser's
  transport adds (3); a response's status and body back as a
  `Reply`, or `Lost`; `OpenStream` as the stream's `GET`, its body cut
  into messages (2). The native shell and the reference's browser
  client host use it (`../browser/README.md`, 3.2); an application's own shell
  may.
- **`jig-protocol-engine-clients-http`,** the engine end, a component on
  skein's HTTP server: each call under its prefix routed to one of the
  component's messages, but a `GET` of the link's route, left to the
  application's page with the referrer policy (1); its credential
  read from the cookie or the bearer; each `Reply` written as `200` with
  its record; the stream written as events, `StreamTaken` said as their
  bytes are written. The application's server mounts it at a prefix of
  its listener and hands it the calls under that prefix.
- **The engine end's room** is its calls in flight, streams among them
  (`README.md`, section 10), and it owns that bound alone. It holds a call from its
  head until it has handled the call's end, even once the server has
  let the call's connection go: until then the call takes room, and
  nothing tells the server it has let the call go. The application asks
  for its room before it hands the engine end a call. It may hold an
  ordinary call's head under its existing connection bound and stall
  deadline until room is free, without asking for its body; the peer's
  end forgets that head. This lets one page's concurrent calls wait for
  each other. A server choosing to refuse instead answers the call
  itself: `503`, with no body, closing the connection, with
  nothing of the engine end's set aside. That is skein's server's busy
  status (skein's `http.md`, section 5: "at its entrance it refuses by
  not asking, or by answering a busy status"). The client takes it as a
  lost exchange or a dropped stream. Keyed requests and watches are
  sent again after their backoffs. A read ends unreachable and is not
  resent (`../../client/domain/client/README.md`, 6). A lost sign-out keeps the
  party signed in and tells the link it did not reach the engine; another
  `SignOut` sends it again (`../../client/domain/client/README.md`, 9). The
  application's
  server bounds its own connections, never by the engine end's room,
  and a call handed with no room is the application's bug, asserted.
- **A peer's end is the call's end.** skein's server lets a response go
  on after its peer has ended, once the request is all read ("the
  response still goes, room may still come": skein's `http.md`, 5.5),
  and says nothing of that end above. No client of the mapping ends its
  side to read on: jig's client end and a browser end a connection only
  to leave it, and a process's kernel ends it when the process exits.
  So the application's server, which reads each connection's events
  below skein's server, ends a connection whose peer ends while a call
  it handed the engine end is unfinished, and tells the engine end the
  call's connection closed, as it does when a connection fails. The
  engine end takes that as the call's end and tells the component: a
  stream's as its connection's end, `StreamGone`, and an exchange's or a
  public read's whose reply it has not had, `ExchangeGone` (`wire.md`, 2.2).
  `wire.md`, section 2's "until the engine sees the stream's connection end" is
  then the moment the peer's end is read, not a heartbeat's failed write
  later.
- **Example guards,** in the engine end, which an application uses,
  wraps, or replaces with its own: a `Host` allow-list; the browser's
  required header and the POSTs' content type, which force a preflight,
  with no CORS; the cookie's attributes; a referrer policy on the
  printed link's route. What each stops is `../browser/README.md`'s (`../browser/README.md`, section 18
  ), and so are jig's attack stories an application may run against
  its own server (`../browser/testing.md`). A page's policy
  (its content security policy, its frames) is the application's
  page's.

## 5. Public reads

- **A `GET`, its read in its path:**
  `<prefix>/v1/public/<owner>/<version>/<kind>[/<id>[/<id>]]`: the
  owner's number (0 for jig, and one more than its family's number for
  an application's), its public records' version and the public kind, each
  a decimal number with no sign and no leading zero, within its field's
  type; then the kind's public identifiers (`public-reads.md`), each exactly 16
  lowercase hexadecimal digits. No query, no trailing slash, no empty,
  dot or percent-encoded segment. One document has one address, so a
  proxy keeps one copy of it. Any other path under
  `<prefix>/v1/public/`, and any method there but `GET`, a preflight's
  `OPTIONS` among them, is `404` with `Cache-Control: no-store`.
- **No credential is read.** The engine end reads no cookie and no
  bearer on these routes, and hands the component no credential, so the
  answer is the same for every visitor, signed in or not, and varies on
  nothing the request carries but its path. The guards that stand are
  the `Host` allow-list (`../browser/README.md`, 12.2), checked first; a public
  read changes nothing, as the printed link's `GET` does not
  (`../browser/README.md`, section 18), so it needs neither the browser header nor
  a content type. No guard of another route is relaxed.
- **Its answers,** each with a fixed set of headers and no others:
  - *the document:* `200`, `application/octet-stream`, its record, and
    `Cache-Control: public, max-age=<rest>, must-revalidate`, where the
    rest is the deployment's `max-age` less the whole seconds, rounded
    up, since the decision that made the answer (`../../domain/people/public-reading.md`,
    10), and never less than zero;
  - *unknown:* `404`, no body, with the same `Cache-Control`, so a
    private project and one never made cost a proxy alike;
  - *busy:* `503`, no body, `Cache-Control: no-store` and `Retry-After`
    of one second; nothing waits;
  - *a version not served:* `200` with its record, as every version
    reads it (`README.md`, section 6), and `Cache-Control: no-store`.

  Every answer, a malformed path's and a guard's among them, carries
  `X-Content-Type-Options: nosniff`, and none sets a cookie or carries
  `Vary`, `ETag`, `Last-Modified`, `Age` or any `Access-Control-`
  header. No answer allows stale serving: no `stale-while-revalidate`,
  no `stale-if-error`. The engine end gives these and no others; the
  application's server adds its page's policy to every answer of its
  origin's, these among them (`../browser/README.md`, 12.1), as it would any
  other: the same fields on every answer, varying on nothing the
  request carries, and its `Cross-Origin-Resource-Policy: same-origin`
  keeps another origin's page from loading the document at all.
- **A room of its own in the engine end:** public calls in flight,
  apart from its other calls (4) and within the component's public
  room; a public call that finds none is `503` as busy, the call's
  head read and nothing more. The application still answers it, as
  4 says, but with the response the engine end gives for its head,
  a pure answer that sets nothing aside: the `Host` allow-list first,
  then the busy answer above, so a refused `Host` is refused alike
  whether the room is full or not. A call to another route with no
  credential is answered signed out by the component with no engine
  work (`public-reads.md`). The application's server bounds its connections,
  as 4 says; a proxy in front holds few of them.
- **No CORS:** a page of another origin may send it, as any site may
  send a `GET`, and cannot read the answer; neither end grants any
  origin, on a document, an error or a preflight.
- **The `max-age`** is the deployment's, `clients.public.max_age`, a
  few seconds, at most jig's bound of 60. With `must-revalidate` and no
  stale serving, a cache that keeps the age it adds (`Age`) serves a
  copy no longer than the rest it was given, so a project made private
  stops being served everywhere within the `max-age` of the decision
  that made the last answer (`../../domain/people/public-reading.md`, 2); what was
  already downloaded is not withdrawn.
- **The client end** sends it as a `Call` with the method `GET`, no
  bearer and no body, and takes back the status, the body and the
  freshness left: the `Cache-Control`'s `max-age` less the `Age` a
  cache added. A `503` is busy, a `404` unknown.
