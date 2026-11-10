# Signing people in

Provisional, 2026-10-23, revised 2026-11-02. How people sign in to a deployment that people
reach at a name of its own, over HTTPS behind a TLS proxy: its public
address, its members, and Google's sign-in on the clients' listener,
which ends in jig's `Mint` (jig's `protocol/client/sign-in.md`). The
printed link stays for recovery. And temper as the OpenID provider of
the deployment's Forgejo, through jig's provider (jig's
`protocol/client/openid.md`), so that a member signed in to temper
opens Forgejo signed in (`sign-in.md`, section 9). temper knows nothing of one
deployment: every name here is an example, temper at
`https://temper.example.com`, Forgejo at `https://git.example.com`.

## 1. In one page

- **A public address,** `application.clients.public`: the origin
  people's browsers use (section 2). jig's mapping admits its host beside
  the listener's, and sets `Secure` on an exchange that came by it; every
  address temper gives out is the public origin's.
- **Members are configured** (section 3): `application.sign_in.members`,
  each a Google address and a login. Google signs anyone in; temper
  mints for a member alone. An identity is Google's provider number and
  the member's address as configured.
- **Bound at the first sign-in:** the Google subject that first signed in
  with a member's address is kept in temper's main store, the forge
  connector's `Binding` family (section 5); another subject with that
  address is refused.
- **Two routes on the clients' listener** (section 4):
  `GET /sign-in/google` starts the sign-in and binds its state to the
  browser by a cookie; `GET /sign-in/google/return` checks Google's
  answer, the member and the binding, sends `Mint`, and on `Minted`
  sets jig's cookie and sends the browser to where it was going.
- **temper's own sans-IO sign-in** over skein-oauth's client: each
  sign-in under way is a `skein_oauth::Client`, its HTTP requests
  carried by the clients' fetcher (section 6), so the flow joins the fake
  issuer as records in its world.
- **Every ending has a page of its own,** server-owned and stock, never
  the client's (4.3).
- **The browser's signed-out page is the sign-in page,** temper's mark,
  "Sign in" and "Continue with Google", told the ways by the page itself
  (section 7). The terminal is unchanged.
- **The printed link signs in the first owner's Google identity** (2.3),
  so the link and Google sign in the same person.
- **No real Google in any test:** `skein-fake-oauth` stands in, its
  addresses given by the configuration's `endpoints` (3.1).
- **temper is Forgejo's OpenID provider** (section 9): Forgejo has an
  authentication source named `temper`, so its sign-in bounces through
  temper, which answers at once for a member signed in. "Open Forgejo ↗"
  starts that sign-in, so a member signed in to temper opens Forgejo
  signed in, with no click. jig holds the codes and tokens and skein's
  issuer writes the documents; temper serves the routes and the page.
- **The authorization is the client's** (9.3): jig's cookie is strict,
  so `/oauth/authorize` is temper's page, which sends jig's `Authorize`
  by `Transport` and replaces its address with Forgejo's callback.
- **Forgejo's account is named by the login:** `preferred_username`,
  with `sub` the identity, `<provider>:<address>`, which outlives the
  store (9.4).
- **The source is added on Forgejo's host by a printed command** (9.5;
  `setup.md`, 6.8), for a managed Forgejo and `temper serve`'s owned one
  alike. A box's owned Forgejo sets all of this up by itself (9.9).
- **No single sign-out** (9.6): the signed-out page says that Forgejo
  keeps its own sign-in.

## 2. The deployment

### 2.1 The host

`temper serve`, standalone, its clients' listener on loopback at a fixed
port; a TLS proxy terminates TLS for temper's name and passes it on, the
`Host` as the browser sent it; plain HTTP is answered with a redirect to
HTTPS. The proxy denies the printed link's route,
`/jig/v1/sign-in/link`, from outside (2.3). jig's stream, `GET
/jig/v1/stream`, is passed unbuffered, with a read timeout longer than
its heartbeat (jig's `protocol/client/default-http.md`, 2).

### 2.2 The public address

`application.clients.public`: `null`, or the HTTPS origin the people's
browsers use (`configuration.md`, section 3). With one:

- **The Host allow-list** (jig's `protocol/browser.md`, 12.2) holds its
  host, with its port where it is not 443, beside the listener's own
  address: jig's `Settings { prefix: "/jig", public: Some(host) }`. With
  none, `public: None`, the listener's address alone.
- **`Secure` by the exchange's `Host`** (jig's `protocol/client/default-http.md`,
  3): jig's cookie is `Secure` on an exchange that came by the public
  name, never on one by the listener's address; temper's sign-in cookie
  (4.2) takes the same decision, by jig's `via`.
- **Every address temper gives out** is the public origin's: Google's
  return (4.2), and a GitHub App's redirect and setup URLs, when a
  GitHub forge is connected (`../client/forge.md`, 9.1). With none, they are the
  listener's, `http://<listener>/`, as before.
- **The bundle is the same,** served at every other `GET`
  (`../client/browser.md`, 2.2).

### 2.3 The printed link

- **It stays, for recovery:** the owner's way in when Google or the
  configuration is wrong, and the terminal's only way.
- **On the host alone:** jig enables it only on a loopback listener; the
  proxy denies its route from outside; the owner reaches it through an
  SSH tunnel to the listener.
- **Whom it signs in:** with `sign_in` configured, the first of jig's
  `owners` whose identity is Google's provider, with that member's login
  as login and name; there must be one, or the start is refused, naming
  `engine.owners`. With `sign_in` `null`, the first owner of the link's
  provider, `owner` and `Owner`, as before; on a box whose
  `machine.owned.sign_in` is set, `machine.owned.person.login` and `Owner`, so
  that the login is the person's on Forgejo too (9.9).

### 2.4 Public reading

A deployment that opens its public projects to anyone
(`../draft/first-release.md`, 5.3; jig's `domain/people/public-reading.md`
; `../client/browser.md`, section 8):

- **Off until figured:** jig's public limits are zero in every profile,
  so a deployment opens by setting `clients.public.*`,
  `clients_http.public_calls`, the root's public figures and the core's
  public index's in its `limits` (jig's `protocol/limits.md`
  ), which the start checks against one another, and its
  owners make a project public (`project amend --visibility public`,
  `../client/tools.md`, 5.3). Each figure is the deployment's;
  dev.fabrig.ai's are the user's (`../draft/first-release.status.md`).
- **The proxy caches the public reads alone:** `GET /jig/v1/public/…`,
  a `200` and a `404` as their `Cache-Control` says, never a `503`, and
  nothing else temper serves, whose answers say `no-store` or
  `no-cache`. It serves a cached answer to every visitor, so the engine
  sees one read per document per `max-age`.
- **Its key:** the deployment, as the upstream the proxy sends to, and
  the `Host` it validated, then the path exactly as received: never the
  path alone, so two deployments behind one proxy never share a copy.
  The `Host` is checked against the deployment's names before the
  cache is looked in, so a refused `Host` is never answered from it.
  The path is neither decoded nor normalized; the engine answers any
  path that is not canonical `404` with `no-store`, which is never
  kept. Cookies and `Authorization` are no part of the key, and the
  proxy drops both from a public read before it goes on, as the engine
  reads neither; a client's own `Forwarded` and `X-Forwarded-` headers
  are dropped too.
- **No stale copy:** it obeys `must-revalidate`, serves nothing past
  its freshness, on an upstream's failure or while it refreshes, and
  refreshes nothing in the background; it keeps the age a copy has and
  sends it as `Age`, and never gives a copy freshness of its own. A
  copy is fresh for the rest the engine gave it from the decision that
  made it (jig's `protocol/client/default-http.md`, 5), so a project made private
  is served nowhere past the `max-age`; what a visitor already holds is
  theirs.
- **Public reads before the engine's room:** per client address, public
  reads per second and in flight, and public reads in flight in all,
  answering past them itself, before it takes a connection to the
  engine; its connections to the engine for public reads are a pool
  apart from the signed-in routes', so a crowd of visitors never holds
  a signed-in person's. Requests with no credential to the other routes
  are limited per address likewise. jig counts no address (jig's
  `domain/people/public-reading.md`, 5).
- **No CORS:** the proxy adds no `Access-Control-` header to any
  answer, an error's and a preflight's among them, as the engine adds
  none.
- **Nothing else opens:** the printed link's route stays denied from
  outside (2.3); every other route answers as without public reading,
  a visitor with no cookie signed out.
- **Checked against a proxy of the world:** the negative walk (2.5)
  runs the public reads through a caching proxy that follows these
  rules; a real proxy's configuration is the operator's, checked
  against them when a deployment opens.

### 2.5 The negative walk

What a visitor can try against a deployment that has opened, played in
the one-process world (`../domain/testing.md`, 2.5) through temper's real
service and a caching proxy of the world that follows 2.4, each answer
checked against what 2.4, jig's `protocol/client/default-http.md`, 5 and `protocol/client/public-reads.md`
, and jig's `domain/people/public-reading.md` allow:

- **Every owner, version and kind:** jig's and the forge's, each
  version served and one not, a kind no owner declared; identifiers of
  another project's tasks, of a project that is private, being deleted
  or never made, of a conversation's delegate and of a change outside
  every goal's tree, all one unknown, byte for byte.
- **Malformed paths:** numbers past their type, leading zeros, short,
  long and upper-case identifiers, encoded, empty and dot segments, a
  query, a trailing slash: `404`, `no-store`, never kept by the proxy.
- **No credential anywhere else:** every keyed request, read, watch,
  stream and sign-out with no credential answered signed out, and the
  same with a visitor's cookie of another deployment; a signed-in
  person's cookie and a bearer on a public read changing no byte of it.
- **No widening:** a project's member's `SetVisibility` refused; the
  limits' relations broken one at a time, each refusing the start.
- **A store written before public reading** opened with every project
  private.
- **Revocation:** a project made private and one deleted while reads
  of it are under way and cached, each answered unknown by the engine
  at once and by the proxy within the `max-age`, never later; the
  proxy's upstream stopped, its copies served until their freshness
  ends and never after.
- **The proxy's key:** two deployments behind one proxy with the same
  path, each answered its own; a refused `Host` with a copy in the
  cache, refused.
- **Load:** public reads past the proxy's and the engine's limits,
  busy, while a signed-in person's reads and requests at the same
  moment are all answered within their usual time; reads abandoned
  midway, the budgets whole after.
- **No CORS:** no `Access-Control-` header on any answer, a document's,
  an error's, a refusal's or a preflight's.

## 3. Who may sign in

- **An identity is Google's provider and the member's address.**
  `application.sign_in.google.provider` names jig's provider of kind
  application that numbers Google's sign-in (jig's
  `protocol/configuration.md`, section 3, `providers`). The subject
  temper gives jig is the member's address exactly as configured, not
  Google's `sub`, so that a member can be named before their first
  sign-in: jig's `owners` and a project seed's roles name them.
- **Members are configured:** `application.sign_in.members`, each
  `{"address", "login"}` (3.1). Each of jig's owners of Google's provider
  is a member: its subject is a member's address, checked at start.
- **Google's word on the address:** the ID token's `email`, with
  `email_verified` true, or the sign-in is refused as not verified.
- **A member is the configured address equal to Google's,** compared with
  ASCII letters folded to lower case; the identity's subject is the
  configured spelling.
- **Bound to Google's account at the first sign-in:** temper keeps the
  `sub` that first signed in with a member's address (section 5). A
  later sign-in with that address and another `sub` is refused: an
  address Google gave to a new account is not the member.
- **The check is temper's, before `Mint`:** a member, bound to this
  `sub` or not yet bound, is minted; anyone else is answered with a page
  that says why, and nothing is minted.
- **Adding a member** is a change to the configuration and a restart. A
  removed member's binding stays, unread, until the address is a
  member's again.

### 3.1 The configuration

```
"application": {
  "clients": {"listener": "127.0.0.1:8367", "bundle": null,
              "public": "https://temper.example.com"},
  "sign_in": {
    "google": {
      "provider": 2,
      "client": "1234-abc.apps.googleusercontent.com",
      "secret": "google-client.secret",
      "endpoints": null
    },
    "members": [{"address": "sam@example.com", "login": "sam"}]
  },
  ...
}
```

`configuration.md`, section 3 holds each member's rule. In short:

- `clients.public`: `null`, or `https://HOST[:PORT]`: a lower-case DNS
  name, no user, path, query, fragment or final `/`, the port not 443;
  `http://` to a loopback host alone, for a test.
- `sign_in`: `null` for none, never missing: the browser's signed-out
  page keeps its printed link's line, and both routes answer not found.
- `google.provider`: a provider of jig's `engine.providers` of kind
  application.
- `google.client`: Google's client id, 1 to 256 bytes of visible ASCII.
- `google.secret`: a data path, a secrets' file holding the client
  secret (`configuration.md`, section 6).
- `google.endpoints`: `null` for Google's own, as in every deployment;
  otherwise the addresses of an issuer that is not Google, for a test
  or a fake, naming no deployment, as a GitHub forge's addresses are: `{"issuer", "authorization", "token", "keys",
  "transport"}`, each address an `https` URL, or `http` to a loopback
  host; `transport` as a forge's (`"plaintext"` only to loopback, or
  `{"tls": {"server_name", "trust"}}`). Google's own: issuer
  `https://accounts.google.com`, authorization
  `https://accounts.google.com/o/oauth2/v2/auth`, token
  `https://oauth2.googleapis.com/token`, keys
  `https://www.googleapis.com/oauth2/v3/certs`, TLS with the machine's
  roots. Each host is resolved at start; one that does not resolve
  refuses the start, naming it.
- `members`: 1 to `sign_in.members` entries (`configuration.md`, section
  7). `address`: 3 to 254 bytes of visible ASCII, one `@` with bytes on
  both sides; unique, folded. `login`: Forgejo's shape, 1 to 40 bytes of
  ASCII letters, digits, `-`, `_` and `.`, its first and last a letter
  or a digit; unique, folded.

Google's registration is a web application's OAuth client in Google's
console, its redirect URI the public origin's `/sign-in/google/return`,
with the non-sensitive scopes `openid email profile` alone.

## 4. Signing in with Google

### 4.1 The routes

On the clients' listener, after what jig's mapping claims and beside the
GitHub routes (`../client/browser.md`, 2.2): `GET /sign-in/google` and
`GET /sign-in/google/return`, any other method `405`, any other path
under `/sign-in/` the page as today. With `sign_in` `null`, both answer
not found. Every answer of theirs carries the page's policy,
`Cache-Control: no-store` and `Referrer-Policy: no-referrer`.

### 4.2 The flow

1. **Start:** `GET /sign-in/google?then=PATH`, the address of the page
   the person was on, and `&account=choose` for "Use another Google
   account". `then` is a path on the origin: one leading `/`, not `//`,
   no `\`, visible ASCII once decoded, at most 1024 bytes; absent or
   anything else is `/`.
   - Past `sign_in.pending` sign-ins under way, the busy page (4.3).
   - Otherwise temper draws the sign-in's tag, 16 bytes, from the
     clients' generator, and starts a `skein_oauth::Client` (6.1). Its
     key set is fetched first; then it gives its authorization URL, with
     `prompt=select_account` for `account=choose`.
   - The answer, once the URL is there: `303`, `Location` the URL, and
     `Set-Cookie: temper-sign-in=<tag, base64url>; Path=/sign-in/google;
     HttpOnly; SameSite=Lax; Max-Age=<sign_in.deadline in seconds>`,
     with `Secure` by the exchange's `Host` (2.2). A key set not fetched
     is the unreachable page (4.3).
2. **Google** signs the person in and sends the browser to `GET
   /sign-in/google/return?code=…&state=…`, or `?error=…&state=…`. Lax
   lets the sign-in cookie come back with this cross-site navigation;
   jig's, strict, does not, and is not needed.
3. **The return,** whose every answer clears the sign-in cookie
   (`Max-Age=0`, the same `Path` and `Secure`):
   - no cookie, or one naming no sign-in under way: the expired page;
   - `error`: the sign-in is cancelled; the cancelled page, saying
     Google's word;
   - otherwise the redirect, the return address and the request's query,
     goes to the sign-in's client, which checks the state in constant
     time, exchanges the code (the client secret in the body), and
     checks the ID token: its issuer, audience, expiry, nonce and RS256
     signature by the key set (skein's `oauth/README.md`, 2.2). A state that
     does not match, a deadline past or an exchange that fails: the
     expired page, or the unreachable page when Google could not be
     reached.
4. **The claims:** temper reads `email`, `email_verified` and `name` from
   the checked token's payload (6.2). No `email`, or `email_verified`
   not true: the unverified page. An address no member's: the
   not-a-member page.
5. **The binding:** temper asks the engine's root to bind the member's
   address to the token's `sub` (section 5). Another `sub` bound: the
   other-account page. The root not running yet, or stopping: the busy
   page.
6. **`Mint`** with the identity (Google's provider, the member's address
   as configured), the member's login, and Google's `name` where it is
   UTF-8 within jig's bound, the login otherwise. On `Minted`: `303` to
   `then`, jig's cookie set by the mapping's `cookie(&settings, via,
   &credential, remaining)`, `remaining` the sign-in's expiry less the
   wall clock. On `MintRefused`, busy or not ready: the busy page; its
   other reasons are configuration the start checks, and answer the busy
   page too.
7. **Google's tokens are dropped:** the client's record is never kept,
   and the sign-in under way ends with its answer. The page at `then`
   loads, and its own exchanges carry jig's cookie: the client signs in
   by `Transport` (jig's `client/domain/client/README.md`, section 9).

- **The return's call is held** while the exchange, the binding and
  `Mint` run, within `sign_in.deadline`; past it, the expired page.
- **Sign-ins under way are memory alone:** a restart drops them, and a
  return after it is answered the expired page.

### 4.3 Its endings

Each a page of the server's own, HTML in temper-service as the GitHub
routes' are (`../client/forge.md`, 9.1): the level-1 heading, a
paragraph, and its links; no script; stock, as the sign-in page is.
`ADDRESS` is the token's address, escaped. "Use another Google account"
is `/sign-in/google?then=THEN&account=choose`, "Continue with Google"
`/sign-in/google?then=THEN`, "Back to sign in" `THEN`. Their words take
the client's typographic apostrophe, ’ (U+2019), as the client's own
words do: "didn’t", never "didn't".

- **In the client's look outside a project** (`../client/README.md`,
  5.1; the look request `outside-pages`): `<html lang="en">` with the
  viewport's meta; a `header` holding temper's mark, a link "temper" to
  `/`; then a `main` column: the heading, the paragraph, the first link
  filled (`class="primary"`: "Continue with Google" or "Use another
  Google account"), and "Back to sign in" quiet, alone in a paragraph
  (`class="quiet"`).
- **Its stylesheet is the bundle's** `files/pages.css`, linked as
  `/bundle/DIGEST/files/pages.css`, DIGEST the served bundle's, as the
  client's page links the bundle's `loading.css`: a file of the page's
  own origin, so `style-src 'self'` admits it. It draws the column of
  `../client/browser.md`, 2.5 (*Outside a project*) in the day's
  colours alone, with the bundle's IBM Plex Sans. With no bundle served,
  no stylesheet is linked and the page reads plain.

| Ending | Status | Heading | Paragraph | Links |
|---|---|---|---|---|
| a member, minted | `303` to `then` | | | |
| not a member | 403 | "ADDRESS can’t sign in here" | "Google signed you in, but no one here signs in with this address. Ask an owner to add it, or use the Google account they added." | Use another Google account; Back to sign in |
| bound to another subject | 403 | "ADDRESS signs in here with another Google account" | "This isn’t the Google account that first signed in with this address. Use that one, or ask an owner." | Use another Google account; Back to sign in |
| not verified | 403 | "Google hasn’t verified ADDRESS" ("this account’s address" with none) | "temper signs in only an address Google has verified. Verify it with Google, or use another account." | Use another Google account; Back to sign in |
| cancelled, or Google's `error` | 400 | "The sign-in didn’t finish" | `access_denied`: "You cancelled at Google. Nothing was kept."; another: "Google answered ERROR. Nothing was kept.", ERROR its code, escaped, at most 64 bytes | Continue with Google; Back to sign in |
| expired: no cookie, a tag or state unknown, used or past its deadline, a restart since | 400 | "The sign-in didn’t finish" | "It took too long, or temper restarted while you were at Google. Nothing was kept." | Continue with Google; Back to sign in |
| unreachable: the key set or the token endpoint not reached | 502 | "The sign-in didn’t finish" | "temper couldn’t reach Google. Nothing was kept: try again in a moment." | Continue with Google; Back to sign in |
| busy: pending full, the root not running, `MintRefused`, bindings full | 503 | "temper couldn’t sign you in just now" | "It is starting or stopping, or signing many people in at once. Nothing was kept: try again in a moment." | Continue with Google |

## 5. The binding

- **The forge connector's family `Binding`,** byte 18 of temper's
  families (`store.md`, section 2): the forge connector is temper's one
  connector, whose `Stored` holds every family of temper's, so the
  binding is restored in the forge's restart step with the rest, and
  written in the root's decisions, with no range of its own.
- **Its key, `Binding(address)`:** the member's address folded to lower
  case. **Its row,** `{ address, subject }`: that address and Google's
  `sub`, at most 255 bytes, as skein-oauth bounds a subject.
- **Asked by the clients' composition:** `forge::Event::Bind { request,
  address, subject }`, through the root as the forge's other events
  come. The connector answers `forge::Request::Bound { request, answer }`
  once its decision's commit is durable: `Bound` (bound before to this
  `sub`, or bound now, its row saved), `Another` (bound to another
  `sub`, nothing written) or `Full` (a new address with every row taken,
  nothing written: the busy page). The forge holds its bindings in
  memory, at most `forge.bindings`, at least the configured members'
  count; since rows outlive their members, a new member can find the
  table full. A row past it is refused at restart, as each family's is.
- **Never erased:** a binding outlives its member's removal; a store made
  again binds afresh (`store.md`, 4).

## 6. The sign-in's exchanges

### 6.1 The sign-ins

`temper-service`'s `clients::google`: a table of at most
`sign_in.pending` sign-ins under way, each its tag, its `then`, its
deadline, the call held, if any, and a `skein_oauth::Client` made with
Google's `Registration` (skein's `oauth/README.md`, section 1): the configured
endpoints, the client id, the client secret from its file, the redirect
URI the public origin's `/sign-in/google/return`, the scopes `openid
email profile`, the form wire, `pkce_for_confidential`, and terms with
OpenID (the issuer and the key set's URL, so that no discovery document
is fetched) and, for `account=choose`, the parameter
`prompt=select_account`. The state, nonce and verifier come from the
clients' generator. Its requests go out as the client asks: `Http` to
the fetcher, `Visit` to the held start's answer; its terminals end the
sign-in: `Tokens` with the checked record, or `Failed`.

- **Printed links wait for the binary in order.** `links.room() > 0`
  is the service's predicate for passing a printed link to the binary.
  That channel is an edge, bounded by `sign_in.pending`; a stalled
  consumer holds the producer, drops no link and preserves FIFO order.
  The service world holds this with the binary's consumer stalled in
  `temper-service`,
  `tests::clients::a_stalled_link_consumer_drops_nothing_and_keeps_order`.

### 6.2 The claims

A small reader in `clients::google` over the record's ID token, already
checked by the client: its payload's `email` (a string within the
member's bound), `email_verified` (`true`, or the string `"true"`, as
Google has sent) and `name` (a string), each optional; nothing else is
read, and the token is dropped with the record.

### 6.3 The fetcher

`clients::fetch`: one HTTP/1.1 exchange per connection, skein-http's
client over skein-tls's client with the configured trust, or plaintext
to a loopback address alone; the origin of each request's URL must be
one of the configured endpoints', resolved at start, or the request
fails before any socket. At most `sign_in.fetches` at once, each within
the client's deadline and `sign_in.body_bytes`; its answer is the
client's `HttpResponse`. A step machine of the clients' composition,
its sockets tagged as the server's are.

## 7. The browser's sign-in page

- **The page tells the ways.** When temper-service makes the bundle's
  page at start (`../client/browser.md`, 2.2), it inserts `<meta
  name="temper-sign-in" content="WAYS">` before the page's `</head>`,
  `WAYS` the configured ways, space-separated (`google`, or nothing),
  and the `ETag` is the page's so made; a page without `</head>` is
  refused at start, naming the bundle. The web shell reads the meta and
  the page's origin (`location.origin`) at start and gives the view
  both at construction; a page whose origin is not `http` or `https`
  offers no way. Further facts of a
  signed-out page come as further `temper-` metas: the forges that sign
  in through temper, for "to continue to" and the signed-out note (9.3),
  made once the forge's records load (9.7).
- **The signed-out main** (`../client/README.md`, 5.1), in the browser
  with a way: temper's mark, the level-1 heading "Sign in", and a link
  for each way, "Continue with Google" to
  `ORIGIN/sign-in/google?then=PATH`, absolute as skein-ui's links are,
  `PATH` the printed address of the page, percent-encoded. With no way,
  the printed link's line, as before. The terminal's "Signed out." is
  unchanged.
- **A navigation, not an exchange:** the link is the page's own `href`,
  so the policy's `form-action 'none'` holds.

## 8. Testing

- **The sign-in world,** `temper-service`'s own tests
  (`crates/temper-service/src/tests/sign_in.rs`), as its GitHub routes'
  are (`../domain/testing.md`, 2.1): the clients' composition with its
  Google sign-ins, jig's clients' component and the engine's root on its
  store, people's requests as calls through `fixtures::http::Peer`, and
  `skein-fake-oauth`'s `Issuer` as Google, answering the sign-ins' HTTP
  requests as records where the fetcher stands, its accounts by
  `sign_in_as`. Stories: a member minted, `then` kept, its binding kept
  and the same `sub` minted again; a second `sub` with that address
  refused; each other ending of 4.3; a restart between start and return
  answered expired. Each story within the world's budget
  (`../domain/testing.md`, 6.1).
- **Step tests:** the configuration's members and their checks; the
  `Binding` family's codec, goldens and fuzz seed; the forge connector's
  bind in its world (`tests/engine/forge`); the claims' reader; the
  fetcher against scripted io; the endings' pages' bytes.
- **The view's tests:** the signed-out main with and without a way, and
  its link's `then`.
- **The browser tier,** one story (`../client/browser.md`, section 4):
  from the page's "Continue with Google", the sign-in cookie set on
  temper's origin comes back from the issuer's cross-site redirect, a
  fake issuer on another site (`localhost` against `127.0.0.1`), and the
  return answers a member's check, not the expired page.
- **No real-loop story.**

## 9. temper as Forgejo's OpenID provider

A Forgejo of the deployment's signs its people in through temper: its
authentication source `temper` is an OpenID Connect client of jig's
provider (jig's `protocol/client/openid.md`), whose codes and
access tokens jig holds and whose documents and tokens skein's issuer
writes (skein's `oauth/issuer.md`). temper serves the routes, builds
the issuer at start, and draws the authorization's page.

### 9.1 What a person sees

- **From temper:** on the Forges page, a Forgejo that signs in through
  temper has "Open Forgejo ↗" (`../client/forge.md`, 5.4), to
  Forgejo's own route that starts a sign-in with the temper source,
  `ROOT/user/oauth2/temper`, in a new tab. ROOT is the forge's browser
  origin: its owned setup's `public` when present, else its record's
  `web` when it has one, else its `git` URL
  (`forge.md`, 2.1; `setup.md`, 6.2). Forgejo sends the
  browser to temper's `/oauth/authorize`, temper answers at once, and
  Forgejo opens signed in. No click. A repository's link does the same with
  Forgejo's `redirect_to`, so it opens on the repository's page
  (`../client/forge.md`, 5.4, *A repository's link*).
- **At Forgejo first:** Forgejo's sign-in page offers "Sign in with
  temper". Signed in to temper, one click; signed out, temper's sign-in
  page says "to continue to" the forge, Google follows, then Forgejo,
  signed in.
- **The first time,** Forgejo makes the member's account, named by
  their login (9.4); an account already there by that name or address
  is linked once, by its password. On a box, the person's account is linked
  with no password (9.9).

### 9.2 The routes

On the clients' listener, after what jig's mapping claims, beside
Google's (4.1). The issuer is the public origin (2.2), or
`http://<listener>` with none; each URL below is the issuer's.

| Route | What it is | Answered by |
|---|---|---|
| `GET /.well-known/openid-configuration` | the discovery document: the issuer, the endpoints below, the code flow alone, PKCE S256, RS256, the scopes and the claims | the issuer's bytes, taken once at start |
| `GET /oauth/keys` | the key set, its `kid` the key's | the issuer's bytes, taken once at start |
| `GET /oauth/authorize` | the authorization: temper's page, as every other `GET` (9.3) | the bundle |
| `POST /oauth/token` | the code's exchange, from Forgejo's server: the client id, a confidential client's secret by Basic or in the body, and the verifier; a public client's password is ignored (9.9) | jig's `Redeem` with the request's `Authorization`, `Content-Type` and body, answered `Redeemed` |
| `GET /oauth/userinfo` | the claims, for the access token as Bearer | jig's `Claims` with the request's `Authorization`, answered `Claimed` |

- **Each answer jig gives is sent as it is:** its status, its body,
  `Content-Type: application/json`, `Cache-Control: no-store`, and
  `WWW-Authenticate` from its challenge, when there is one. The discovery
  document and the key set are `application/json`, cacheable.
- **The token's body** is read within the issuer's request bound (jig's
  `provider_limits`); past it, `413` and nothing sent to jig.
- **Any other method** on the four is `405`. **With no provider**
  (`engine.clients.provider` `null`), the four answer not found, and
  `/oauth/authorize` is the page, whose `Authorize` jig refuses
  `no_provider`.
- **The issuer is built at start** by jig's
  `jig_protocol_config::clients::provider_issuer`, from jig's
  `engine.clients.provider`, its key's and its clients' secrets' files
  read privately by temper's startup (`configuration.md`, section 6),
  and the URLs above; it goes to jig's clients' component in its
  settings. A key or a client the helper refuses refuses the start at
  its path.
- **A moved port moves the issuer,** at the next start, and Forgejo keeps
  the address its source was given. A box's preparation gives it the
  current one (9.9); on any other arrangement the source is its owner's
  to change, as its owner added it.
- **What Forgejo 17's client does,** and the routes meet (its goth
  v1.82.0): it always sends a PKCE challenge, S256, and its verifier at
  the exchange; it authenticates at the token endpoint by Basic first,
  and by its secret in the body when that fails; it calls the userinfo
  endpoint by `GET` with the access token as Bearer, and requires its
  `sub` to be the ID token's. It verifies no signature of the ID token:
  it takes the token straight from the token endpoint. A confidential
  client authenticates with its secret; the box's public client has none,
  and relies on S256 and the exact redirect (9.9).
- **The signing algorithm is RS256 alone,** by the key jig's
  configuration names (9.7): `skein_tls::sign_rs256` signs it, its
  signatures replay byte for byte in the worlds, and skein's client, the
  issuer's peer in its world, verifies RS256.

### 9.3 The authorization

jig's cookie is `SameSite=Strict` and its `Path` is jig's prefix (jig's
`protocol/client/default-http.md`, 3), so a request to `/oauth/authorize` never
carries it, and a navigation that started on another site (Forgejo's
redirect) would not carry it anywhere. So the authorization is decided
by the client, not by a route:

1. `GET /oauth/authorize?QUERY` is answered with temper's page. The
   client root's routing reads the address as a place of its own,
   `Authorize { query }`, `QUERY` as the address carried it
   (`../client/README.md`, 5.1; `../client/browser.md`, 2.3).
2. The client signs in by `Transport`: its exchanges are same-origin,
   started by temper's page, so the cookie goes with them whatever site
   the navigation came from.
3. Once the link is live and local transport credit is available, it
   sends jig's `Authorize { query }` (`POST /jig/v1/authorize`), and
   draws the answer. Cancelled watch exchanges and cleanup calls retain
   credit until their HTTP reply or failure (jig's `protocol/client/README.md`,
   section 10); the first authorization waits for that credit:

| jig's answer | The page |
|---|---|
| *go*, an address | "Continuing to Forgejo…", with "Signed in as PERSON · ADDRESS. FORGE opens in a moment, signed in as LOGIN."; the page replaces its address with jig's, so the back button never returns to the authorization |
| *signed out* | the sign-in page (section 7), under its "Sign in" the line "to continue to **FORGE**", its `then` the authorization's address, so Google's return lands back at step 1 |
| refused *unknown client* or *unknown redirect* | "This sign-in can’t continue": "The site that sent you here asked temper to send you back to an address temper doesn’t know. Nothing was sent anywhere.", and "Back to sign in"; no address is followed |
| refused *malformed*, *no provider* or *not a person* | the same heading, its paragraph open (section 10) |
| *not ready*, *busy* or a locally *lost* exchange | "temper couldn’t sign you in just now": "It is starting or stopping, or signing many people in at once. Nothing was kept: try again in a moment." |

- **FORGE** is the forge as the client names it (`../client/forge.md`,
  5.4): "Forgejo at HOST", or "Forgejo on this machine" for an owned
  one, of the forge whose client is the query's `client_id`; a
  `client_id` that names no forge's is "Forgejo". PERSON, ADDRESS and
  LOGIN are the signer's, as the client knows them.
- **The page's facts:** temper-service inserts, beside the
  `temper-sign-in` meta (section 7), one `<meta
  name="temper-forge-sign-in" content="NUMBER CLIENT ROOT MODE">` for
  each entry of `sign_in.forges` that serves (9.7): its forge's number,
  its client's id, its
  browser origin (9.1) and `owned` or `other`, space-separated,
  attribute-escaped.
  The web shell gives them to the view with the ways; the signed-out
  note and "to continue to" read them, as "Open Forgejo ↗" reads the
  forge's row.
- **The address's bound:** signed out, an authorization address
  (`/oauth/authorize?` and its query) past the Google start's `then`
  bound, 1,024 bytes (4.2), cannot ride the sign-in, and is refused
  *malformed* before anything is sent. Signed in, the query goes to jig
  as it came, within jig's own bound on it.
- **No ADDRESS yet:** jig's party gives the client its name and login,
  not its identity's subject, so the signed-in line drops " · ADDRESS"
  until it does (section 10).
- **No consent page:** the clients are the deployment's, configured by
  its owners.

### 9.4 The claims, and Forgejo's account

- **`sub`:** the identity, Google's provider number, `:`, and the
  member's address as configured, `2:sam@example.com`. It outlives the
  store: a deployment started afresh gives Forgejo the same `sub` for the
  same member, never a party's number, which a fresh store would give to
  someone else.
- **`preferred_username`:** the member's login. **`email`:** the
  member's address, verified, for Google's provider is one of jig's
  `addresses`. **`name`:** the member's name as minted. No `nickname`.
- **Forgejo's settings,** in its `app.ini` (`setup.md`, 6.2 and 6.8):
  - `[oauth2_client] ENABLE_AUTO_REGISTRATION = true` and `USERNAME =
    preferred_username`, which Forgejo 17 takes (a value it does not
    take is logged and read as `nickname`): a member's first sign-in
    makes their account, named by their login;
  - `ACCOUNT_LINKING = login`, Forgejo's default: an account that
    already exists with that name or address is linked once, by its
    password. Not `auto`, which links by a matching name or address
    alone, but on a box's owned Forgejo (9.9);
  - `[service] ALLOW_ONLY_EXTERNAL_REGISTRATION = true`, beside
    `DISABLE_REGISTRATION`, so that no one registers on Forgejo's own
    form and the temper source registers its members. `ENABLE_INTERNAL_SIGNIN
    = false`, once the source works, makes it Forgejo's only sign-in on
    the web; its admin keeps Forgejo's command line, and temper's bot its
    tokens.

### 9.5 The source on Forgejo

Forgejo's API has no route for authentication sources (Forgejo 17), so
the source is added on Forgejo's host, by its command line:

```
forgejo admin auth add-oauth --name temper --provider openidConnect \
  --key forgejo --secret "$(cat CLIENT_SECRET_FILE)" \
  --auto-discover-url https://temper.example.com/.well-known/openid-configuration
```

`temper forge sign-in` prints it, for a managed Forgejo and `temper serve`'s
owned one alike (`setup.md`, 6.8); a box runs its own (9.9), with no secret.
The name `temper` is the callback's path
segment, so it is fixed. `add-oauth` fetches the discovery document at
once and adds no source when it does not answer, so temper serves
before it runs.

### 9.6 Signing out

- **Signing out of temper ends temper's sign-in only.** Forgejo keeps
  its own session, and Google its own. While a forge signs its people in
  through temper, the signed-out page says so, under its "Sign in": "You
  signed out. FORGE keeps its own sign-in: sign out there too.", one
  line for each such forge.
- **Signing out of Forgejo** ends Forgejo's session; its next sign-in
  bounces through temper, and a member still signed in to temper is
  signed in again at once.
- **No single sign-out:** Forgejo 17 takes no signal from its provider:
  no back-channel, front-channel or session-management logout, and no
  RP-initiated one (it parses the discovery document's
  `end_session_endpoint` and never uses it); its own sign-out is local.

### 9.7 The configuration

jig's `engine.clients.provider` (jig's `protocol/configuration.md`,
section 3), and temper's `application.sign_in.forges`:

```
"engine": {"clients": {"provider": {
  "key": "provider.key",
  "clients": [{"id": "forgejo", "secret": "forgejo-client.secret",
               "redirect_uri": "https://git.example.com/user/oauth2/temper/callback"}],
  "addresses": [2]}}},
"application": {"sign_in": {..., "forges": [{"forge": 1, "client": "forgejo"}]}}
```

- **The key:** an RSA key of 3072 bits, PKCS#8 PEM, mode 0600, the
  service's own user, made on its host by `temper forge sign-in`
  (`setup.md`, 6.8) with `openssl genpkey`, never copied, printed or
  logged; jig's start refuses any other. A key replaced is a file
  replaced; its key set's `kid` names it anew.
- **`sign_in.forges`:** a list, absent for none (`configuration.md`,
  section 3): each `{"forge", "client"}`, `forge` a forge's number, as
  the Forges page and `temper forge` name it, `client` one of the
  provider's clients. Each forge and each client at most once, and each
  client one of the provider's, or the start is refused at its path. A
  member whose login, folded, is the bot's (`temper`) refuses the start at
  its path, whatever the forges: temper never signs a person in as the bot.
- **Checked once the forge is known.** A forge is a record in temper's
  store, made at runtime (`forge.md`, 2.1), so the start cannot check
  it, and an entry refuses no start, whichever forges are kept. Once
  the forge's records load, and when a forge is made or removed, an
  entry serves only when its forge is a kept Forgejo whose browser
  origin's `/user/oauth2/temper/callback` is exactly its client's
  `redirect_uri`: its owned setup's `public` when present, else its
  record's `web` when it has one, else its `git` URL (9.1); and when
  its meta has room within `clients.bundle_bytes`. An entry that does
  not serve is named once in the log, for that reason, and its forge
  has no meta, no "Open Forgejo ↗" and no line on the signed-out page;
  every other entry serves. Server-to-server API calls keep the
  record's `api` and transport; git uses `git`.
- **A browser origin apart from `git`,** by `web`: a Forgejo connected
  as its admin (`setup.md`, 5.5) gives it where temper and its workers
  reach it at another address than people do, as a Forgejo a host's own
  units run beside temper is on loopback to temper and on its public
  name to a browser (`setup.md`, 7.4).
- **The page follows.** The metas are made anew when an entry's serving
  changes, and the `ETag` is the page's so made, so a browser's next
  revalidation takes them. A reply on its way ends with the page it
  began.
  With a provider and no `sign_in.forges`, nothing of temper's names
  the provider's clients.

### 9.8 Testing

- **An attacker review of the box's sign-in** (9.9): another process on
  the machine and another browser tab, account linking by name and address,
  excluded identities, public-client redemption and PKCE, the printed link,
  cookies, key files, source failure and restored source facts. Every finding
  is fixed or recorded with its remaining exposure.
- **A box's owned Forgejo** (9.9): the step tests of its claims, its refused
  logins, its address's encoding and the issuer's refusals; the box world, its
  source step failing with the box running and the set-up done; the process
  tier, its one click on a real Forgejo against temper's own issuer.
- **The sign-in world** (section 8) gains the provider: Google's
  sign-in, then the page's `Authorize`, the token by Basic and by the
  body, and the userinfo, with the same `sub` and `preferred_username`;
  the discovery document and the key set at their routes; `405`; not
  found with no provider; a token's body past its bound. jig's own
  refusals are its world's, not repeated.
- **The view's tests:** the `Authorize` place's pages for each answer,
  "to continue to", the signed-out note, and "Open Forgejo ↗" with its
  line. **The client's domain:** the address read as `Authorize`, each
  of jig's answers.
- **Step tests:** `sign_in.forges`' checks, at start and once the
  forge's records load, an entry for a forge not kept, not a Forgejo,
  of another origin or past the bundle's room named and left out; the
  metas' bytes;
  `temper forge sign-in`'s printed bytes and the files it makes.
- **One process-tier story** (`../domain/testing.md`, 2.6): an owned
  Forgejo, the temper source added by the printed command, a scripted
  client with a cookie jar that signs in to temper by the printed link
  and then to Forgejo through temper, and Forgejo's API naming the
  member's account by their login, their address and the `sub` it
  keeps. It proves the claims and
  Forgejo's settings, which no fake can. The printed link, not the fake
  Google: with `sign_in` configured it signs in the member of Google's
  provider (2.3), so Forgejo gets the same claims; Google's leg is the
  service world's; and skein's fake issuer peer cannot yet give its
  account a verified address.
- **The browser tier, one story** (`../client/browser.md`, section 4):
  jig's strict cookie absent from a navigation that started on another
  site and present on the page's own exchanges, so the `Authorize` place
  answers *go*.

### 9.9 A box's owned Forgejo

A box's person presses a repository's link and lands on its page, signed in
(`../client/forge.md`, 5.4, *A repository's link*). A box has no provider
without this, and nobody to run a printed command, so its owned Forgejo sets
itself up, with no command for the person. A managed Forgejo and `temper
serve`'s owned one stay as 9.4 to 9.7 say. This is an authentication change, so
its reasons are kept here.

**What the box's configuration holds,** as `temper init --box` writes it
(`setup.md`, 8.2):

```
"engine": {"owners": [{"provider": 1, "subject": "ada@localhost"}],
           "providers": [{"number": 1, "kind": "link"}, ...],
           "clients": {"provider": {
             "key": "provider.key",
             "clients": [{"id": "forgejo", "secret": null,
                          "redirect_uri": "http://127.0.0.1:8370/user/oauth2/temper/callback"}],
             "addresses": [1]}}},
"application": {"sign_in": null,
                "machine": {"owned": {"listen": ..., "person": {"login": "ada", "email": "ada@localhost"},
                                      "sign_in": "forgejo", ...}}}
```

- **`application.sign_in` stays `null`:** it holds Google's sign-in and its
  members, and a box has neither. `sign_in.forges` names a forge by its number,
  which is known only once the forge is made (9.7), so a box's entry names a
  kind instead: **`machine.owned.sign_in`,** a client of the provider, or `null`.
  It is the entry for the kept forge of mode owned, under 9.7's check, whatever
  its number. A client it names that the provider lacks refuses the start at its
  path.
- **The redirect is known at init,** since the owned `listen` socket is written
  there: 9.7's check holds it against the forge's browser origin as ever.

**The claims** (9.4), which Forgejo's `auto` needs both of:

- **`preferred_username` is `machine.owned.person.login`.** The printed link's
  login on such a box is that login (2.3), so the party's login is the Forgejo
  account's. Its name stays `Owner`. With `machine.owned.sign_in` `null`, nothing
  of this applies and the login stays `owner`.
- **`email` is `machine.owned.person.email`:** the owner's subject is that
  address and provider 1 is in `addresses`. `sub` is `1:ada@localhost`, which
  outlives the store as 9.4 wants.

**`ACCOUNT_LINKING = auto`, on a box's owned Forgejo alone.** Measured on a
throwaway Forgejo 17.0.1, with a stand-in provider, against a site-admin account
with a password, as the set-up makes:

| Setting and claims | Where it lands |
|---|---|
| `login` | Link account, asking for the password: what breaks one click |
| `auto`, the name matching, the address differing | the repository, signed in, three redirects, no second account |
| `auto`, no `email` claim | Link account, asking for the password |
| a second visit in one browser | through the provider again, every click passing `/oauth/authorize` |

So every press passes "Continuing to Forgejo…" (9.3), which replaces its address
as soon as jig answers, with nothing to hold it longer. `auto` links by the name,
then by the address, with no password. That is safe here because temper made every
account on this Forgejo and the printed link signs in the box's one person (below).
A box trusts every program listening on this machine's loopback, since browser
cookies ignore ports. So Forgejo's session lasts an hour (`setup.md`, 6.2): every
press signs in again, and one click needs no more. `login` stays for a
managed Forgejo, whose accounts are not temper's, and for `temper serve`'s owned
one, until the same reasoning has been checked for several members.

**What a name match could take,** and how a box rules each out:

| Forgejo holds | Its name | How a box rules it out |
|---|---|---|
| the person's own account, a site admin | `machine.owned.person.login` | the one it is meant to take: the same person, who holds the machine and may read its password |
| temper's bot | `temper` | a person so named refuses the start (below); **temper never signs a person in as the bot** |
| the set-up's admin | `temper-admin`, `temper-admin@example.invalid` | refused likewise, the name and the address |
| the first repository's organization: `auto` looks in the table that holds users and organizations alike | `work` | refused likewise; any other organization's name can't be the person's, since the person's account is made first and a name is taken once |
| an account by the person's address, if the name finds none | `ada@localhost` | the bot's address (`temper@temper.invalid`) as the person's refuses the start; the address is otherwise the person's alone |

- **Refused, at the start, at its path:** `machine.owned.person.login`, folded,
  being `temper`, `temper-admin` or `work`, or its `email`, folded, being the bot's
  or the set-up's admin's, when `machine.owned.sign_in` is not `null`. `temper init --box` already falls back
  from these logins; a hand-written document is refused. A login reaches jig only
  from these or from `sign_in.members` (9.7), so a start-time refusal is whole.
- **Refused too, at the start, at `machine.owned.sign_in`'s path:** that member
  beside `machine.owned.public`. Behind a proxy the forge's browser origin is
  `public`, where the redirect init writes is the owned `listen`'s, so 9.7's check
  would serve nothing, with the source added and no off fact. No walk has proved a
  box behind a proxy (section 10).
- **Refused too, at the start, at `machine.owned.sign_in`'s path:** that member
  beside an `application.sign_in` that isn't `null`, in the words "a box's sign-in
  takes application.sign_in null". A box is its one person's: a member Google signs
  in could land on the person's Forgejo account by name, and temper's link to that
  account (`../draft/forge-identity.md`, 9) would count them as the person.
- **Nobody else signs in:** a box has no member and no Google; the printed link
  alone signs anyone in, always its owner, so the link provider's one owner is the
  one person who can reach `/oauth/authorize` signed in. An agent or a coding run
  is refused *not a person* (jig's `protocol/client/openid.md`).

**The provider's key:** an RSA key of 3072 bits, as 9.7 says, made at the first run
by `temper init --box`, before the document is written, in `provider.key` in the data
directory, 0600 and the service's user's, under `temper forge sign-in`'s rule
(`setup.md`, 6.8): `openssl genpkey`, since skein has no key generation (ring signs
and does not generate); a key already there is left as it is. With no `openssl`, or if it fails, the document has
`clients.provider` and `machine.owned.sign_in` `null`, the box goes on and the links
stay plain, and `box.log` says so.

**The client is public** (skein's `oauth/issuer.md`, section 2): `secret` `null`.
`add-oauth` takes `--secret` only as an argument, which a command line shows to other
local users, and a fixed value is no secret. So the box's client has none, and
`add-oauth` is given none. S256 and the exact redirect stand in for it. The clients'
other rules are skein's, per client, so no other client is weakened.

**The source's step:** the last of the owned group's, after Forgejo answers
(`setup.md`, 7.1): `temper forge source`, which makes the source serve. It reads
temper's discovery document first, `ISSUER/.well-known/openid-configuration`, within
the set-up's deadline; with no answer it ends failed, `temper's discovery did not
answer`. Then, unless Forgejo lists the source, it adds it, by `add-oauth --name temper
--provider openidConnect --key forgejo --auto-discover-url
ISSUER/.well-known/openid-configuration` with `--work-path` and `--config`, which works
at once while Forgejo runs. With the source listed, it asks Forgejo for the source's
sign-in, `GET /user/oauth2/temper` at the owned `listen`, following no redirect: a
redirect to temper's `/oauth/authorize` says Forgejo serves it, and the step ends
added. Forgejo loads a source's discovery when it starts, and a source whose discovery
did not answer then is listed but not served; on that sign-in Forgejo, finding no
provider for it, loads its sources again, so the same request makes it serve. Any
other answer ends the step failed, `Forgejo does not serve the source`. A step that
ends failed runs again while its group stands (`setup.md`, 7.2), so the source serves
once the engine answers, whichever started first, and Forgejo is never restarted for
it. temper is serving by the step's first run, since it asked for the group. On a restart, serve
starts before this group too: a later preparation's source commands and Forgejo, loading its
source, fetch discovery. So serve's read at its start comes before preparation's new handover,
which it reads again once its first reconcile has waited for Forgejo, taking it before the
connect (`setup.md`, 6.5).
A restored group reports started only after its final source outcome, as on its first start.
Its source fact is accepted for the kept box-owned Forgejo even with no setup ask in flight;
the restored forge needs no ask. An unknown or non-owned forge cannot set this fact.

**A moved port.** The discovery address names temper's own listener, and a port a box
finds taken is one its person may edit (`setup.md`, 7.1). Forgejo reads a source's
address when it loads the source, at its start, and a source already listed is not
added again. So a later preparation, before Forgejo runs, lists the sources and, with
`temper` among them, runs `forgejo admin auth update-oauth --id ID --use-custom-urls false
--auto-discover-url ISSUER/.well-known/openid-configuration`, with `--work-path`
and `--config` (`setup.md`, 6.3). Each of the two commands has the set-up's deadline, since Forgejo may fetch the
address it is given and serve answers it only once it is up.

- **It never fails the set-up.** Any ending of the step leaves the set-up succeeding:
  one line in `box.log`, the forge's page fact saying so, and the links plain until a
  later run of the step makes the source serve, which turns the fact and the links back.
  Never a welcome failure (`setup.md`, 7.2). Any ending of the preparation's two commands
  leaves the preparation going, with one line in its own log and the source as it was:
  the entry still serves, so a press then ends at Forgejo's own error, on the old
  address, until a later start succeeds. Forgejo's own login still works.
- **Until the step says the source is there, the entry does not serve:** 9.7's
  "does not serve" with the reason, so no address leads to a route Forgejo lacks.
  Then the metas are made anew as 9.7 says, and "Open Forgejo ↗" and the repository's
  link appear on a box too.
- **The page's other fact:** while a box's owned forge is kept and its entry can't serve
  for want of a key or a source, one `<meta name="temper-forge-sign-in-off"
  content="NUMBER">`; the Forges row says so (`../client/forge.md`, 5.4).

Forgejo's own login with the password keeps working: `ENABLE_INTERNAL_SIGNIN` stays
as it is. `temper ctl` and a coding run's credentials are untouched: they are the
bot's and the person's tokens, and neither passes this provider.

## 10. Open questions

- **Startup readiness beyond local admission** (9.3): waiting for people
  during authorization, Google's return mint and approval sign-in is
  separate from waiting for transport credit. Its policy and bound are
  jig's (`jig/docs/design/protocol/client/README.md`, section 12); no startup
  retry is chosen here.
- **Generating the provider's key in process** (9.9): it needs a key
  generator in skein; its signing library does not generate RSA keys.
  Default: `openssl` at init, with the provider off and password links
  when the key cannot be made. No new cryptography dependency for this alone.
- **Upgrading an existing box to repository sign-in** (9.9): an older
  document has no provider or `owned.sign_in` and keeps its password links
  and printed-link login `owner`. Default: no migration; init's new
  document sets up the provider on a fresh box.
- **The terminal's sign-in fallback status** (9.9): the page's off fact
  reaches the browser's Forges row alone. Default: `box.log` for the
  terminal, as the `ui-repo-link` cut decided; a wire fact waits for a
  terminal status design.
- **A handover after Forgejo restarts with serve still running** (9.9):
  discovery can be answered, but taking preparation's new handover
  needs a notification and a takeover while the forge is connected
  (`setup.md`, section 15). Default: it waits for serve's next start;
  9.9's second read is once per serve start, not per Forgejo restart.

- **Links after a failed source-address update** (9.9): whether a box
  records the failure between preparation and the source step, so the
  entry and its links stay off until the current discovery address is
  stored. Default: the preparation logs the failure and succeeds, the
  entry still serves, and Forgejo may send the press to the old address.

- **`temper serve`'s owned Forgejo signing in with no command** (9.9). Default: its
  printed command, until `auto`'s reasoning has been checked for several members.
- **A box's owned Forgejo behind `machine.owned.public`** (9.9). Default: the pair is
  refused, until a walk proves its redirect, its source and its off fact.
- **A deployment's own sign-in page.** Default: stock. Later, perhaps a
  name and a line in the configuration, or a template of the owner's.
- **Members at runtime:** an owner's invitation and roles from the
  client wait for a design of their own. Default: the configuration's
  members, roles by jig's seed.
- **Google's Workspace domain** as a member rule (`hd`). Default:
  addresses alone.
- **A terminal sign-in for a member who is not the owner:** a device
  code's flow. Default: the browser alone.
- **The signed-in line's address.** Default: dropped, since jig's
  party carries no subject (9.3); otherwise jig's party carries it.
- **The authorization's refusals without words:** *malformed*, *no
  provider* and *not a person* (9.3). Default, until words are chosen:
  the heading "This sign-in can’t continue" with no paragraph.
- **Owners as Forgejo's admins.** Forgejo's source can make admins of a
  group claim (`--admin-group`) and map groups onto teams
  (`--group-team-map`). Default: none; Forgejo's admins are made on
  Forgejo.
- **Several members' forge identities.** A box's sole-owner link names
  its person on an owned Forgejo or a manifest-made personal GitHub App
  (`../domain/forge.md`, 4.6). Extending that to members needs the sign-in
  reasoning checked for several people and a manifest's connecting owner,
  with a family of links (`../draft/forge-identity.md`, section 12).
  Default: adoption's seeds outside a box's one-person scope. Posting an
  approval as the person in the client remains separate (`../README.md`,
  section 8).
- **The provider's key's rotation.** Default: a key replaced is a file
  replaced and a restart; Forgejo verifies no signature, so nothing of
  Forgejo's changes.
- **Single sign-out** waits for Forgejo to take a logout from its
  provider (9.6).
