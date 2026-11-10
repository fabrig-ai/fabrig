# OAuth client

Provisional, 2026-10-09, revised 2026-10-26. `skein-oauth` is a bounded,
sans-IO OAuth client for one exchange at a time. It prepares
authorization-code sign-in, refresh and revocation requests, accepts the
resulting HTTP answers, and hands token records to its caller. It knows no
account, provider, token store, browser, or service policy, and discovers
nothing but an issuer's published keys (section 2.2). The caller supplies endpoints, client
registration, clocks, limits, and fresh random bytes. It owns the redirect
listener, HTTP/TLS connection, durability, and the decision to grant a token
after keeping it. skein's driver, `skein-oauth-accounts` (`accounts.md`), is that
caller for any service that wants one: it owns those for its owner, so a
service writes no listener, connection or token store of its own.

The client covers refresh; public sign-in with PKCE and a loopback redirect;
and confidential sign-in with a client secret and the web's redirect. Where a
registration asks for them, it also covers a client id the issuer issues at
the first sign-in (section 2.1), an OpenID ID token checked against the
issuer's published keys (section 2.2), and revocation (section 2.3). Its
records may also be access-only, never refreshed: a token obtained
elsewhere, or a sign-in's whose issuer gave no refresh token. Device authorization is outside this component. The same HTTP
records travel through `skein-http` as in `skein-llm`.

## 1. Boundary and entities

The caller supplies a `Registration` with an authorization URL, token
endpoint, client id, redirect URI, requested scope, token wire format, and
optional client secret. The authorization URL and token endpoint are `https`,
with a host and no userinfo, or `http` on loopback for an issuer on the same
machine, such as a replaying tier's fake (../../foundation/testing-strategy.md, 4.4). It admits
one `Client` machine for an
exchange. The endpoint binding is kept through the exchange; a redirect cannot
choose a new token endpoint. A public client requires an already bound
loopback listener and a redirect URI whose loopback address, port, and path
match that listener. The confidential client has a secret supplied beside its
registration. Its redirect URI is `https`, or `http` on loopback, written as a
public client's is (below: `127.0.0.1`, `[::1]` or `localhost`, an explicit
port that is not zero, and a path), for a caller that serves its sign-in on
its own loopback address; the caller's server receives it, never a listener
of the client's, and its port never varies. The caller routes the browser's redirect to the same in-progress
client and checks the request's actual URI against the bound redirect URI
before giving its query parameters to the machine. A caller using multiple
issuers keeps each sign-in bound to its issuer and redirect; the machine never
infers an issuer from the redirect or token response.

**A registration's additions.** A registration may also carry one optional
value, the issuer's terms; a registration without it is exactly the one
above. The terms are:

- **parameters added to the authorization URL,** each a name of at least
  one byte and a value, which may be empty, sent on every sign-in, or only
  on a new registration (section 2.1);
- **a resource,** sent in the authorization URL, the code exchange and
  every refresh (RFC 8707);
- **an issued client id:** the registration's client id is then only the
  entry point of a new registration, and the issuer issues the client id
  that every later request carries (section 2.1);
- **OpenID:** the issuer's identifier, and its key set's URL, configured or
  found in the issuer's metadata; the client then asks for an ID token and
  checks it (section 2.2);
- **the scopes that must be granted,** among those requested (section 2.2);
- **a revocation endpoint** (section 2.3);
- **the error names that mean sign in again,** each of at least one byte,
  beside `invalid_grant` (section 4);
- **a redirect whose port varies** (below).

Every endpoint the terms name, the metadata's and the key set's among them,
is `https`, or `http` on loopback, as the token endpoint is.

A public client's redirect URI is `http://127.0.0.1:PORT/path`,
`http://[::1]:PORT/path`, or `http://localhost:PORT/path`, the last for an
issuer that registered its public client with that name, as some do. The path
is present, and the port is explicit and not zero, except where it varies
(below). A `localhost` redirect is
bound to 127.0.0.1 and never resolved: the name is a spelling the issuer
requires, not an address to look up. It is compared exactly, byte for byte
with the registered URI, in the registration and in the browser's redirect:
the redirect's authority must be `localhost:PORT` as registered, so
`127.0.0.1:PORT`, `localhost.`, `localhost.example`, another case, another
port or another path is refused before the machine sees the query.

**A redirect whose port varies,** for an issuer that accepts any port on its
loopback redirect (RFC 8252, section 7.3). Only a `127.0.0.1` or `[::1]`
redirect may vary; a `localhost` one never does, since its issuer compares it
byte for byte. Its port is zero, meaning any free port, or a preferred port,
used when it can be bound. Each sign-in is given the port its listener bound
(`accounts.md`, section 5), and that port replaces the registration's for the sign-in: the
authorization URL, the redirect's check and the code exchange all carry the
same URI, and a redirect to any other port is refused. A redirect whose port
does not vary is bound as written, or the sign-in fails. A confidential
client's redirect never varies: its registration refuses a varying port.

`Client` owns only a sign-in or refresh in progress: its registration, private
state value and optional PKCE verifier, one authorization code or prior token
record, deadlines, retry count, and at most one pending HTTP exchange. It
does not own an account table. The caller may create several clients, under
its own admission cap. One accepted exchange emits one terminal `Tokens` or
`Failed`; `Visit` and `Http` are demands, not terminals.

Token and authorization-code bytes are opaque secrets. The saved record is a
versioned encoding of access token, refresh token if it has one, expiry,
caller's opaque record name and generation, and optional bounded claim
metadata; and, each where the exchange gave one, the issued client id, the
granted scopes, the subject, the ID token and the earliest refresh time
(sections 2 and 2.2). It never contains a client secret, state, nonce, code,
or PKCE verifier. The ID token is a secret as the tokens are: it is kept only
to be sent back as a hint (`README.md`, section 2.1), and never logged.
The caller keeps a candidate record durably before lending its access token.
On refresh, an omitted replacement refresh token preserves the previous one,
and generation increases once. A crash before keeping a rotated record may
leave the old refresh token unusable; the caller reports that according to its
own policy.

**The record's version is 3.** The client writes version 3 only, and reads
versions 2 and 3. A version-2 record reads as a version-3 record with none
of the five additions: the registration's own client id, the requested scope
as granted (as a token response without `scope` means, RFC 6749, section
5.1), no subject, no ID token and no earliest refresh time. It is lent and
refreshed as any record, and its refresh is kept as version 3. Under a
registration whose client id is issued, a record without an issued client id
is unreadable instead (`accounts.md`, section 4: a sign-in replaces it), since its refresh
would carry the entry point's client id. Any other version is unreadable.

An **access-only record** has no refresh token. It holds a token obtained
elsewhere and handed in by the caller, such as another program's sign-in that
the caller may read but must not rotate: a shared refresh token, once
rotated, would sign the other program out. A sign-in whose token answer
carries no refresh token, as an OpenID issuer's may for a scope without
offline access (RFC 6749, section 5.1, makes it optional), gives an
access-only record too, holding what the sign-in gave beside it: the issued
client id, the granted scopes, the subject and the ID token. An access-only
record has no refresh state, so no refresh can be asked of it; it is lent
until it expires, and only its source renews it (`accounts.md`, section 3): a sign-in's,
by signing in again.

## 2. Sign-in and refresh

The public flow uses a fresh transaction-specific verifier of 43–128 allowed
ASCII bytes, and `S256`: base64url without padding of SHA-256 over the
verifier. The verifier is supplied by the caller's seeded cryptographic
generator (../runtime/lib.md, section 8: `CryptoRng`); the machine never draws
entropy. It also takes an independent fresh state
value. The authorization URL carries `response_type=code`, client id,
registered redirect URI, state, requested scope, challenge, and
`code_challenge_method=S256`; and, where the registration's terms ask, its
added parameters, the resource, a nonce and the hints of section 2.1. `Visit`
gives the URL to the caller to show or open. The caller's loopback listener
remains bound while awaiting redirect.

**Each sign-in's own values** come in its `SignIn` event, so that the
machine draws nothing and keeps nothing between attempts: the state, the
verifier and, for OpenID, the nonce, each fresh from the caller's generator;
the redirect with the port its listener bound (section 1); and, when the
caller holds a record of the account, that record's issued client id, ID
token and subject, for signing in again (section 2.1). The registration
holds what every attempt shares: its added parameters and its resource. An
authorization URL that carries an ID token is a secret, as the URL's state
is (section 4).

The confidential flow carries the same state and redirect binding. It uses
its client secret in the token exchange. PKCE with `S256` is enabled when the
registration says the issuer supports it; this is recommended even for
confidential clients. A registration cannot ask for an unsupported challenge
method. The secret never enters the authorization URL. The caller presents
the `Visit` URL through its web sign-in flow.

On redirect, the client compares the returned state in constant time with
the in-progress state, refuses duplicates or missing parameters, and consumes
the code only once. OAuth `error` on a matching redirect is a typed failure.
A wrong state or redirect URI is refused without an HTTP exchange. The
machine sends a bounded POST to the configured token endpoint with
`grant_type=authorization_code`, code, client id, redirect URI, plus verifier
for PKCE and secret for a confidential client. Refresh sends
`grant_type=refresh_token`, client id, prior refresh token, and client secret
when needed. Both carry the resource where the registration has one, and the
issued client id where it is issued (section 2.1). The caller chooses JSON or form encoding for this endpoint;
both encoders measure before allocating. The response is a bounded JSON token
or error document. Unknown extension fields are ignored only within the
document, nesting, token-count, and string bounds. Recognized fields may not
occur twice. The bearer token type is checked before a record is returned.

Refresh receives a complete prior record or refresh state, which only a record
with a refresh token has. It does not start two exchanges for the same record;
the caller serializes each account's refreshes. A response yields a candidate
with an absolute expiry calculated from the injected wall clock, shortened by
any read-only JWT `exp` claim the caller requested. `remaining` is
recalculated when durability completes or when a grant is encoded. A JWT
claim read from an access token is metadata from a token obtained at the
configured endpoint, never proof of authentication. The one token this crate
authenticates is an OpenID ID token, only as section 2.2 says, and it never
uses claims to choose an issuer or account.

A token response's earliest refresh time (`earliest_refresh_at`), where it
gives one, is kept in the record, and no refresh is asked before it (`accounts.md`, section 3
). Its granted scopes are the response's `scope`, or the requested scope
where the response has none (RFC 6749, section 5.1); a refresh that names
none keeps the record's.

### 2.1 A client id the issuer issues

A registration whose client id is issued signs in in one of two forms, chosen
by whether the caller holds a record of the account with an issued client id:

- **A new registration,** with no such record: the authorization URL
  carries the registration's client id, the entry point, and the added
  parameters sent on every sign-in and those sent only on a new
  registration (such as a name hint).
- **Signing in again,** with such a record: the URL carries the issued
  client id, the added parameters sent on every sign-in and none of the
  others, and, from the record, `id_token_hint` with its ID token and
  `login_hint` with its subject.

The redirect then carries the issued `client_id` beside `code` and `state`.
The machine reads it from a redirect whose state matched, within the client
id's bound. A matching redirect without it is an incomplete registration: the
sign-in fails as incomplete, and no exchange is made. Signing in again keeps
the client id the redirect gives, which is the record's unless the issuer
issued another. The code exchange, every refresh and the revocation carry the
issued client id, never the entry point, and the record keeps it.

### 2.2 The ID token

A registration with OpenID asks for an ID token: its requested scope includes
`openid`, and its authorization URL carries the sign-in's nonce.

- **The keys come first.** Before the visit, the machine fetches the
  issuer's keys with bounded GETs: the key set at its configured URL, or
  else the issuer's metadata (`<issuer>/.well-known/openid-configuration`)
  and then the key set its `jwks_uri` names. A metadata document whose
  `issuer` differs from the registration's is refused. A sign-in whose keys
  cannot be fetched or read fails before its `Visit`, so a person is never
  sent to an issuer whose answer cannot be checked. The keys are held for
  that sign-in only: sign-ins are rare, and a key set held longer would need
  an expiry of its own.
- **The code exchange's answer must carry an ID token,** a compact JWS,
  checked before `Tokens` is emitted:
  - **its signature** is RS256 (RSASSA-PKCS1-v1_5 with SHA-256) only, by
    the key whose `kid` its header names, or by the key set's one RSA
    signing key where the header names none. Any other `alg`, `none` among
    them, is refused. A key is an RSA JWK (`kty` `RSA`, with `n` and `e`)
    whose `use` is `sig` or absent, of 2048 to 8192 bits; other keys in the
    set are ignored.
  - **its claims:** `iss` equals the registration's issuer; `aud` holds the
    client id the exchange used, and `azp`, where `aud` holds several,
    equals it; `exp` is after the injected wall clock, less the skew the
    limits allow; `nonce` equals the sign-in's, compared in constant time;
    and `sub` is present, within its bound.
  - **its subject,** `sub`, becomes the record's subject, and the token
    itself the record's ID token (section 1). An email is never a subject.
- **The profile claims its caller asks for.** A registration's OpenID terms
  name which of `email` and `name` to read; no other claim is read, and none
  unless asked. `Tokens` carries what was read, beside the record and never in
  it, and nothing reads them to choose an issuer or account: they are words to
  show.
  - **`email`** is read only where `email_verified` is `true`, or the string
    `"true"`, as some issuers write it;
  - **each is reported or absent, and never fails the check:** a claim that is
    missing, repeated, not a string, empty, past 255 bytes or holding a control
    byte is absent.
- **Granted scopes are reported, not enforced here.** A sign-in that was
  granted fewer scopes than the registration requires still returns its
  record, and `Tokens` names the missing ones; the caller decides (`accounts.md`, section 3
  ). A person who declines part of a grant has still signed in.
- **A refresh's ID token** has no signature check, and the record keeps the
  sign-in's (section 6). Where the registration asks for profile claims, they
  are read from it when its `iss` equals the registration's issuer, its `aud`
  and `azp` hold as above, its `exp` is after the wall clock less the skew, and
  its `sub` equals the record's subject; it has no nonce to compare. The answer
  came straight from the configured token endpoint, whose TLS stands in for the
  signature (OpenID Connect Core, 3.1.3.7), and what it gives is shown, never
  trusted. A refresh with no ID token, or with one that fails any of these,
  reads nothing and is not failed for it.

The check is the one cryptography this crate does. It runs in `skein-tls`,
over rustls's ring provider, which the workspace already builds for TLS
(../net/tls.md, section 4): the client depends on `skein-tls` for this check alone,
and adds no crate. RSA verification draws no randomness and reads no clock,
so the machine stays deterministic. With `skein-json` and `skein-tls`
beneath it, the client is a protocol composition of machines
(../../foundation/programming-model.md, section 4), as it already is over `skein-json`;
`skein-tls` exposes the check as one function over the key's modulus and
exponent, the signed bytes and the signature, and the client names no
ring or rustls type.

### 2.3 Revocation

A registration with a revocation endpoint can revoke a record's refresh
token (RFC 7009): a bounded form POST with
`token`, `token_type_hint=refresh_token`, the client id (the issued one where
it is issued) and the client secret where there is one. A 200 answer, empty
or not, means revoked. Any other answer, or none within the attempt budget,
leaves it unconfirmed, and the terminal, `Revoked`, says which. The caller
forgets the record either way (`accounts.md`, section 3). An access-only record has no
refresh token and is never revoked by the client.

## 3. Machine and deadlines

`step` consumes one event and writes bounded requests. `resume` emits held
work when output room exists. The owner reserves the published maximum output
before either call. Every deadline is compared with an injected monotonic
`Time`; wall time is used only to calculate saved expiry. The caller bounds
the absolute sign-in time, each HTTP attempt, retry count and backoff ceiling.
No delay extends the absolute deadline. A retry is armed only for a transient
failure and only when the entire attempt budget still fits; its backoff grows
to the configured ceiling. An HTTP outcome includes whether a request was
proved unsent, may have been sent, or received a response. Refresh after an
ambiguous send is allowed only as a caller policy decision because a rotating
refresh token may already be spent.

| State | Holds and waits for | Event → state and output |
|---|---|---|
| Idle | limits and registration | `SignIn` → AwaitRedirect, emit `Visit`, or, with OpenID, AwaitKeys, emit a keys `Http`; `Refresh` or `Revoke` → AwaitHttp, emit `Http` |
| AwaitKeys | the sign-in's values, the metadata if fetched, sign-in deadline | metadata answer → AwaitKeys, emit the key set's `Http`; key set answer → AwaitRedirect, emit `Visit`; unreadable answer, deadline or cancellation → Done, `Failed`, no `Visit`; transient failure → Backoff, as AwaitHttp |
| AwaitRedirect | state, redirect, optional verifier, sign-in deadline | matching `Redirected` → AwaitHttp, emit code-exchange `Http`; OAuth error → Done, `Failed`; wrong state, replay, cancellation, or deadline → Done, `Failed`; `admits` → unchanged, answering whether a redirect would be taken |
| AwaitHttp | exchange and HTTP deadline | successful `Http` → Done, emit `Tokens`, or, for a revocation, `Revoked`; invalid grant, malformed answer or a failed ID-token check → Done, `Failed`; transient answer or transport failure → Backoff or Done; deadline → Backoff or Done; a revocation that fails or runs out of attempts → Done, `Revoked`, unconfirmed |
| Backoff | exchange, retry time, absolute deadline | `Tick` at retry time → AwaitHttp, emit `Http`; absolute deadline or retry budget exhausted → Done, `Failed` |
| Done | terminal already emitted | late redirect, HTTP answer, or tick → ignored; explicit reset → Idle |

`admits` is a question, not an event: it answers whether a redirect, at a
time, would be taken by `Redirected`, by the checks `Redirected` makes before
it ends the sign-in or sends the exchange, in their order: not waiting (any
state but AwaitRedirect, or the deadline reached), another address (the URI
not the one the sign-in bound; a code and an error both; neither; no issued
client id beside a code, where one is issued), or another state (compared
in constant time). A redirect it admits with an error is taken, and ends the
sign-in as `Redirected` does. It changes nothing, so its caller can refuse a
redirect without ending the sign-in (`accounts.md`, 5). It takes the redirect's values as
its caller decoded them (`Redirect`: the address without its query, the state,
and the code, error and client id when given; an empty one counts as not
given), and answers why it would not take them (`Inadmissible`: `NotWaiting`,
`Address` or `State`). One check serves both, so they cannot differ. A caller
that hands `Redirected` a redirect without asking still ends the sign-in on
it, as the table says: another address or state, or the deadline, as
`InvalidRedirect`; a code and an error both, or neither, as `Malformed`; no
issued client id as `Incomplete`.

`Cancel` in any active state emits one cancellation failure and goes to Done.
An event for a different exchange cannot complete the current one. The HTTP
transport always produces a terminal outcome for each emitted request;
otherwise the injected deadline closes it. The machine does not own socket
settlement. A successful token response consumes the code or refresh attempt;
it is never replayed by the machine. A retry of a sign-in code exchange is
permitted only if the prior request was proved unsent. The caller can start
another sign-in after a failure with a new state and verifier.

A start in any state but Idle is refused in that step and changes nothing:
the exchange it found goes on, its terminal still to come, and Done still
waits for its reset. A refused `SignIn` or `Refresh` emits `Failed` with
`Limit`, a refused `Revoke` emits `Revoked`, unconfirmed. The caller
dispatches each step's output before the next, so a terminal in a start's
own output answers that start, and no caller is left waiting on a start the
client did not take.

An `Http` demand names its method: a POST, or a GET for the issuer's
metadata and keys, which carries no secret and may be retried as any
transient failure is. A `Revoke` takes the record whose refresh token it
revokes; `Revoked` is its terminal, confirmed or not, and only a cancellation
ends it as `Failed`.

## 4. Failure vocabulary and secrets

`SignInAgain` means an authorization code or refresh grant was rejected or
revoked (`invalid_grant`), or the matching redirect returned denial.
`RetryLater` holds a bounded retry delay for rate limits, server failures,
transport failure or deadline exhaustion after the machine's retry budget.
`Malformed` means a bounded peer answer failed syntax, required-field, type,
duplicate-field, or token checks. `InvalidRedirect` and `Limit` identify
local admission failures. Client authentication failure is a separate
`ClientRejected` so a wrong server registration is not presented as a
person's revoked grant. Untrusted descriptions are never placed in a domain
record or log. A status alone does not prove a grant was revoked.

- **The names a registration adds** (section 1) are `SignInAgain` as
  `invalid_grant` is, whatever the status they come with: an issuer's
  terminal refresh errors, such as `invalid_refresh_token`, `token_expired`,
  `refresh_token_expired`, `refresh_token_invalidated` and
  `refresh_token_reused`. Only the error name decides; a status with no
  name decides as above.
- **`Unverified`** names the ID-token check that failed (section 2.2): the
  keys (not fetched, unreadable, or none that fits), the signature, the
  issuer, the audience, the expiry, the nonce, or a missing ID token or
  subject. It is the issuer's fault or an attacker's, never the person's:
  nothing of the sign-in is returned, and its code is spent.
- **`Incomplete`** is a matching redirect without the issued client id
  (section 2.1), handed to `Redirected` without asking `admits`: no exchange
  is made, and the person signs in again. The accounts component asks first,
  and refuses such a redirect with the sign-in still waiting (`accounts.md`, 5).

Every type carrying a token, authorization code, verifier, state, nonce, ID
token or client secret omits its bytes from `Debug` and logs. HTTP request bodies and headers
that contain them are treated as secrets by the owner. Authorization URLs
contain state, challenge and nonce, and when signing in again an ID token,
and are handed only to the person's visit flow;
the URL is not a durable record. Error descriptions and JWT claims are
untrusted metadata. A record's `subject`, `metadata`, `granted_scopes` and
`issued_client_id`, and a redirect's address without its query, its `error`
and its `client_id`, are no credential, and print as they came. Only an
authenticated TLS token endpoint, or an
`http` one on loopback (section 1), can supply a usable bearer token.

## 5. Bounds and verification

`Limits` caps authorization URL, endpoint and redirect URI, client id and
secret, scope, state, verifier, code, request body, response document, JSON
depth and tokens, string, access and refresh tokens, error detail, record,
and optional JWT payload bytes. For the issuer's terms it also caps, in
one sum, the added parameters' names and values and the added error names;
and the resource, issued client id, nonce, subject, granted scopes,
ID token, the metadata and key-set documents, the keys a set holds, and an
RSA modulus of at most 8192 bits; and the skew allowed on an ID token's
expiry. Encoders measure the complete escaped URL or
body before allocating. Readers reject overflow before storing. A
registration whose terms hold an empty parameter name or an empty error
name is refused as `Limit`: each entry then holds at least one byte of the
terms' sum, so that sum also bounds how many entries there are. Limits
keep every input and encoded output representable; the client and its
common data have no per-process heap sum.
The maximum output per step is one demand plus one terminal in the case
where an answer completes an
exchange; exact values are exported with the implementation.

Focused tests cover both sign-in types, state and redirect mismatch, duplicate
and late redirect, PKCE challenge and form/JSON encodings, refresh rotation
with and without a new refresh token, a sign-in answered without one and
kept access-only, expiry and skew, refusals, malformed
answers, and every limit boundary. Redirect tests admit `localhost` with an
explicit port and path, and refuse `localhost.evil`, `localhost.`,
`localhost:0`, a missing port, and a redirect whose authority is `127.0.0.1`
for a `localhost` registration; a confidential registration admits each
loopback redirect as a public one does, refuses the same neighbours and
plain `http` to any other host, and keeps its `https` one. Record tests round-trip access-only records. A
separate fake issuer drives sign-in, refresh, revocation, delay, replay, and
transport faults through the real client machine. It also runs as a process on
loopback, as the fake LLM peer does (../testing/fakes/llm.md, 3). It mints no token: each
token response, to a sign-in or a refresh, is one its script queues, so its
script lists every access token it grants, each refresh's included, before
a world starts. A world that runs it beside the fake LLM peer gives the peer
that list (../testing/fakes/llm.md, 6). Before its first drive, the hosted issuer
peer may adopt a caller-held listener with `Peer::adopt_listener(fd, addr)`,
as the fake LLM peer does. Its queued listen claims that address; on refusal
the caller retains the descriptor. The peer owns an accepted descriptor
through ordinary shutdown. A simulated peer test serves an issuer document
on the adopted address and checks settled io and no open descriptors.
Randomized cases use the fuzzy profile. Memory high water is checked
under named scenario ceilings with headroom.

**The issuer's terms** are tested the same way: a new registration and
signing in again, each sending what section 2.1 says and nothing else; an
incomplete redirect; the resource in the authorization, the exchange and the
refresh; a redirect whose port varies, refusing another port; every ID-token
check failing alone (a key the set does not hold, `alg` `none` and `HS256`,
no `kid` with two keys, another issuer, another audience, an expiry at the
skew's edge, another nonce, a modulus past the bound); a metadata document
naming another issuer; fewer scopes granted than required; a version-2
record read, refreshed and kept as version 3, and refused under an issued
client id; revocation confirmed and unconfirmed; each added error name on
a refresh, with each status; an empty parameter name or error name refused,
and a parameter with an empty value sent; and terms at their most entries
under a named scenario ceiling with headroom.

**The fake issuer's additions,** each scriptable and observed:

- **a registration** that issues its client id in the first sign-in's
  redirect, or scripted to leave it out, and checks that each later request
  carries it;
- **the added parameters, `nonce` and `resource`,** read from each request
  and observed by name, so that a test sees which a request carried;
- **an ID token signed with a test key,** with its metadata and key set
  published, and scripted faults in each: another key, another `alg`, a
  wrong claim, an unreadable document;
- **the account that signs in:** its subject, and the claims an identity
  provider such as Google adds, an address (`email`, with
  `email_verified` true or false) and a name, each in the ID token where
  the account has it. A world sets the account for the authorizations
  that follow, and each code keeps the account its authorization had,
  so that one world signs in several people, or one address under two
  subjects; until one is set, the account is the OpenID terms' subject
  alone, with no other claim;
- **granted scopes,** all those asked or fewer, in the token response;
- **revocation,** answered as scripted: confirmed, refused, or never;
- **the added error names,** on a refresh or an exchange, with the status
  the script gives.

The test key is the fake's own, never a secret. The fake signs through
rustls's ring provider, as the client verifies. RSA signing in ring blinds
with randomness it draws from the kernel, outside a world's seed; the
signature's bytes do not depend on it (RSASSA-PKCS1-v1_5 is deterministic),
so neither does a world's trace (section 6).

The protocol choices follow [RFC 7636](https://www.rfc-editor.org/rfc/rfc7636.html),
[RFC 8252](https://www.rfc-editor.org/rfc/rfc8252.html), and
[RFC 9700](https://www.rfc-editor.org/rfc/rfc9700.html); the issuer's terms
follow [OpenID Connect Core 1.0](https://openid.net/specs/openid-connect-core-1_0.html)
for the ID token and its checks,
[RFC 7009](https://www.rfc-editor.org/rfc/rfc7009.html) for revocation and
[RFC 8707](https://www.rfc-editor.org/rfc/rfc8707.html) for the resource. Current
[Gitea OAuth2 documentation](https://docs.gitea.com/development/oauth2-provider/)
describes authorization-code public and confidential registrations, PKCE,
and JSON or form token POSTs. The LLM subscription endpoints are caller
configuration; provider-specific admission remains with those consumers.

## 6. Open questions

- **Long-answer figures on the owner's page** (`accounts.md`, 3, 5): the driver
  reports the address's length and bound at its terminal. Carrying them
  through an owner's domain and client wire to its sign-in page is
  separate work in that owner's contract
  (`jig/docs/design/protocol/configuration.md`, section 8).
- **The accounts world's scenarios as data** (`accounts.md`, 8): fixture, private-file
  fault, owner-script and expectation rows wait while higher-priority work
  holds the round. `PasteThenListener` waits for `Fact::Taken` in the
  browser and for `pasted` one observe turn earlier on the wire; making
  them one predicate changes the story. It needs a step script interpreted
  in each tier, as do the restart, listener-close and browser-stage paths,
  or an agreed departure from ../../foundation/testing-strategy.md, section 10. Default:
  keep their ordering and the remaining story branches; the open item is
  `skein/docs/development/debt/D048-oauth-accounts-world-copies.md`.
- **A private file's record of an older version.** Default: unreadable,
  and its account signs in again (`accounts.md`, 4); skein-oauth could patch it as it
  loads, with its own patch set, as jig patches transcripts as read.

- **A `localhost` redirect and IPv6.** The listener binds 127.0.0.1 only. A
  browser that resolves `localhost` to `::1` and does not fall back would
  find nothing there; whether to bind `[::1]` beside it, for the same
  redirect.
- **Device authorization,** for a person whose browser is on another
  machine, beside the pasted redirect (`accounts.md`, 5): whether it joins the client.
- **A confidential redirect's authority.** Its `https` URI is read by its
  scheme alone, so `https://` and `https://a@b` are admitted where an
  endpoint's are refused (section 1), and the issuer matches it against its
  registration. Default: so; whether it takes an endpoint's reading.
- **A credential helper,** a program that prints a token, as another source
  of access-only records beside handing them in.
- **The fake issuer's signing in a simulated world.** ring's RSA signing
  blinds with randomness from the kernel, which no world seeds, and ring
  takes no generator from its caller. The signature's bytes do not depend
  on it. Default: the fake signs as it answers; if a world must own every
  draw, the fake signs its ID tokens once, before the world starts.
- **A refresh's ID token.** Default: no signature check, its profile claims
  read as 2.2 says, and the record keeps the sign-in's. Checking it would fetch
  the issuer's keys for refreshes too.
- **What `login_hint` carries.** Default: the record's subject. An issuer
  that expects another claim would need the registration to say which.
- **How `earliest_refresh_at` is written.** Default: seconds since the
  epoch, as a JWT's `exp`; a value the client cannot read is ignored.
- **A wanted account's interval.** Default: its refresh lead, as for a held
  grant, so an issuer whose access tokens last an hour sees a refresh an
  hour; whether a longer interval keeps a refresh token alive as well.
- **An exchange's lifecycle** (3): `skein-oauth-accounts`'s `Exchange` keeps
  `pending_grant`, `aborted` and `file_active` beside its stage, and neither
  crate's machine has one handler per cell (../../foundation/programming-model.md, section
  5.4). Default: as they stand, until a debt item names them.
- **A return no application gives** (`accounts.md`, 5): temper's sign-in runs in a tab of its
  own and holds its page (`temper/docs/design/client/pages/README.md`,
  `ConnectModel`, The sign-in's tab), so neither `Request::Return` nor
  `jig-protocol-accounts`'s `return_to`, over it, has a shipping caller: their own
  tests alone reach them. Whether they go, or wait for an application that sends
  its own tab to `visit`. Default: they stay.
