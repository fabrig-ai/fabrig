# The forge component

Provisional, 2026-10-20. `skein-forge-connection` runs a service's calls
to a forge end to end for the protocol layer that owns it. For each call
it:

- connects through io, over TLS or in plaintext to loopback;
- runs the dialect's plan (dialects.md, section 1.2) and decodes each
  answer as it streams;
- counts the requests the call cost, and reuses idle connections.

It also takes each forge's access when the owner hands it, checks it,
holds and renews the credential, resolves repositories and users, and
turns verified deliveries into hints: a webhook's, or a GitHub App's
delivery log's. When its owner closes it, it closes everything it keeps
and says so. A service that works on a forge writes no connection code.

## 1. In one page

- **A component of a protocol layer:** a step machine with its own
  state, limits, deadlines and most outputs per entry point,
  which a service's protocol layer owns and routes to
  (../../foundation/programming-model.md, sections 4 and 4.2).
- **Calls in, answers out.** A call goes out once; its one terminal is
  an `Answer` or an `Error`, with the requests it cost.
- **A forge starts with no access,** and its calls are answered at once,
  not connected. The owner hands access at runtime, and the same request
  replaces it (3.5).
- **Each forge's start checks its access:** that the credential stands
  for the user the access names. A wrong user, or another `4xx`, stops
  that forge alone, never the component (3.3, 3.4).
- **Credentials have a kind** (README.md, section 7): a token is never
  renewed. A GitHub App's key signs the JWTs that mint each
  installation's tokens, and the component renews those tokens before
  they are due, so a call waits for one within its deadline (section 5).
  None leaves the component but git's, which goes to the owner alone
  (5.3).
- **An App's installations** are chosen by account: a call takes the
  token of the installation for its repository's owner (`connection-calls.md`, 1.4).
- **Hints for latency, polling for liveness:** a listener of its own
  verifies each webhook delivery as it streams, answers it first, then
  offers its hint (`connection-webhooks.md`, section 1). For a GitHub App, a puller reads the
  App's delivery log instead (`connection-webhooks.md`, 1.5).
- **It reads no file,** and its memory is bounded: the owner hands it
  every value (3.1), and nothing grows with a forge's history (`connection-machines.md`, 1.5).

## 2. In its owner's protocol layer

- **`skein-forge-connection`** depends on lib, io's vocabulary,
  skein-http (client, and server for the listener), skein-tls's client,
  `skein-link`, skein-json and `skein-forge`. It knows no service's
  configuration, files or domain. One component serves every forge its
  owner hands it, at start or while it runs (3.1).
- **The owner's protocol layer:**
  - gives io its own tokens for the component's entities, keeping the
    component's token in each, so io's events reach the component that
    asked. The component's tokens are below 2^60, so 60 bits keep one:
    its API connections from 1, its listener 2^59, and its webhook
    connections above it;
  - reports the component's earliest deadline, and fires its deadlines
    where it fires its own;
  - routes what passes between the component, io and the owner's
    domain, and decides nothing;
  - closes the component as part of its own end, and counts it settled
    only on the component's `closed` (../../foundation/programming-model.md, section
    5.2).

### 2.1 What crosses

Down, from the owner: a **call** (token, forge, repository, `Op`);
**refused** and **refused hint**, an answer or a hint its domain did not
take; **returned**, an answer handed back in the domain's drain; an
**offer**, when the domain has room; a forge **added**, with its values
(3.1), and a forge **removed**, while the component runs (3.1); a
forge's **connect**, with its access (3.5); a GitHub forge's
**installations**, the ones the owner accepted (`connection-calls.md`, 1.4), and one more
**log origin** (3.1); and the **close**, its last act.

Up, to the owner: **answered**, a call's one terminal with its token,
cost and result; a **hint**, after its delivery's response is queued
(`connection-webhooks.md`, 1.2) or as its log's row is read (`connection-webhooks.md`, 1.5); **listening**, the webhooks
listener's bound address (`connection-webhooks.md`, 1.4); **listener lost** (`connection-webhooks.md`, 1.4); **connected**,
a forge's start done (3.3); **stopped**, a forge's access refused, at
most once per connect; an **installation**'s change (`connection-calls.md`, 1.4); a
**git token**, an installation's for git (5.3); a **fact**, a
diagnostic for the owner's log; and **closed**, the last event (`connection-machines.md`, 2.5).

| Down | Up |
|---|---|
| `Request::Add(forge)` | `Event::Connected { forge }` |
| `Request::Remove { forge }` | `Event::Stopped { forge, why }` |
| `Request::Connect { forge, access }` | `Event::Installation { forge, installation, account, change }` |
| `Request::Installations { forge, accepted }` | `Event::GitToken { forge, installation, account, token, expires }` |
| | `Event::Fact { forge, fact }` |

An installation event's `account` is `Some` with its current id and
login, or its last known value on `Deleted`. `None` occurs only on
`Deleted` of an id never seen; a missing accepted id keeps its known
account across later reads. `change` is `Created`, `Deleted`,
`Repositories`, `Suspended`, `Unsuspended` or `Renamed`, and `expires` is
the token's expiry in wall seconds, as the forge said it. A `fact` is
`Origin { origin }`, a job log's origin not configured (`connection-calls.md`, 1.5);
`Unkept { installation }`, an installation the table has no room for
(`connection-calls.md`, 1.4); or `Gap`, a delivery log read short of its cursor (`connection-webhooks.md`, 1.5). A fact
is a diagnostic: the owner logs it, and no decision, the component's or
the owner's, rests on it.

Events up and requests to io enter uncapped queues. The owner reserves
room only where io submits to the kernel or a stream sends to its peer.

### 2.2 Room, and the drain

(../../foundation/programming-model.md, sections 5.2 and 7.) An answer or a hint the
owner cannot take is held and offered at its next offer: at most one per
call, and one per webhook connection, which reads nothing more while its
hint is held. One the domain returns during its drain is never offered
again: the next process reads afresh.

### 2.3 The rules it keeps

API only, with `page` and `limit` always and within the forge's cap
(`connection-machines.md`, 1.3). Paths and users are cached by their change token (`connection-calls.md`, section 1).
Webhooks are hints, answered first (`connection-webhooks.md`, section 1). The rate is honoured: a
`429` is `RateLimited { after }`, and nothing retries. GitHub's pauses
are the same error, with `after` as github.md gives it (`connection-machines.md`, section 1
), and each answer carries the rate's window the forge reported (`connection-calls.md`, 1.1).
A request is a call's step, a start read, a renewal (5.1), a read of the
installations (`connection-calls.md`, 1.4), a redirect followed (`connection-calls.md`, 1.5) or the puller's (`connection-webhooks.md`, 1.5),
with no heartbeat. The rest are the client's and the plans'
(client.md, sections 4 and 6; forgejo-operations.md, section 1).

## 3. Start

### 3.1 What the owner hands it

The values of README.md, section 2.2, from no file:

- **for each forge:** its number, unique among the forges; its dialect,
  `Kind::Forgejo` or `Kind::GitHub`; its API's socket address; its
  transport, TLS with a server name and what to trust, or plaintext to
  loopback with the name the forge was configured by, when it was one;
  its access, when the owner has one at start (3.5); and, for GitHub,
  its job logs' origins (`connection-calls.md`, 1.5);
- **the listener's address,** when webhooks are used;
- **the word** (README.md, section 5) and **the user agent** of every
  request: temper's are `temper` and `temper/0.0.0`;
- **the limits** (`connection-machines.md`, 1.2) and **the shared values** (`connection-machines.md`, 1.3).

A forge's values are handed at start, or while the component runs, and
a forge is dropped while it runs, so that a service serves the forges
it makes and removes without a restart:

- **`Add`** hands one forge's values, as at start. It is checked as
  start checks each forge (`connection.md`, 3.2, `connection-machines.md`, 1.4), beside the forges held: a number
  the component holds, a forge past `forges`, or one a check refuses is
  ignored, since the owner checks its values first as it does for its
  start. A forge added with its access starts (3.3); one without waits,
  not connected, for its connect (3.5).
- **`Remove`** drops one forge whole. Each of its calls still open is
  answered `Unavailable` at once; its connections are aborted, an
  exchange in flight among them unheard; its start, its access, its
  installations, their tokens and its puller go with it; and each of
  its events the owner has not yet taken, but a call's answer, is
  dropped. From then on a call to it is answered `Unavailable` at once,
  a delivery on its path `404` (`connection-webhooks.md`, 1.3), and a hint verified for it is
  never offered. Its rows in the tables go as any do, the one used
  longest ago first (`connection-calls.md`, 1.2, 1.3). A number the component does not hold
  is ignored, and a number removed may be added again, as a new forge.
- **`LogOrigin`** appends a resolved origin to a GitHub forge's list
  (`connection-calls.md`, 1.5), checked as an `Add`'s; one listed already, past
  `log_origins` or refused is ignored.

The component makes each request's `Host` from the address and the
transport's name (forgejo.md, section 1.1); the owner hands no `Host`. It
reads a TLS name back (`Name::server`, ../net/tls.md, 3.4): a DNS name goes to
`host` as the name, and an IP name in place of the address's IP, with
the address's port, so that it is written as an address is. An IP name
that differs from the address's IP is the `Host`, as a DNS name is: the
name names the forge, and the address only reaches it. Start's check and
each request make it by the same function, from the same values, so they
agree.

### 3.2 What start checks

A refusal names the value, or both limits where two disagree (`connection-machines.md`, 1.4).
Nothing starts and no io is requested. It refuses: more forges than
`forges`, or two with one number; an in-process name for an address, and
plaintext off loopback; a plaintext name that is empty or holds a byte
at or below a space, at or above DEL, or one of `/:[]`; an access that
3.5 refuses; a user agent that is empty or holds a byte below a space
other than tab, or DEL, which skein-http would refuse in every request's
head; more than `log_origins` origins, an origin on a Forgejo forge, and
an origin whose host is not a DNS name or whose address 3.1's rules
refuse; an `https` origin on a plaintext forge, which has no trust to
check it with; a listener that is not a loopback IP socket; a word that is not
1 to 16 bytes of lowercase ASCII letters, digits and `-`; and the limits
`connection-machines.md`, 1.4 checks. A TLS name is refused by none of them: `Name` holds only a
DNS name or an IP address, and either makes a `Host`. The check makes
each forge's `Host` as its requests will (3.1), for `connection-machines.md`, 1.4's head.

JSON response bodies pass through a connection-local intake. It reads
what has arrived from HTTP, at most the remaining intake room, and meets
the tokenizer's finer demands locally. There is at most one body read
outstanding below. HTTP retains responsibility for framing and the body
bound, so read-ahead cannot cross a response. Header lines and raw bodies
keep their own demands; a raw body of unknown length (chunked, or ending
at close) reads what has arrived, at most `chunk`, never a byte a demand.

A pump routes at most 64 child-machine movements. At that boundary,
local work remains ready or its answer remains queued. Held bytes meet
the tokenizer before its end is delivered; queued HTTP terminal events
are consumed before another refill is requested. Closing, discarding or
reusing an exchange clears its local bytes and demand state.

### 3.3 Each forge's start

A forge's start begins at each connect (3.5). Calls for a forge wait
until it is ready, within their deadlines. Its start runs two of the
dialect's plans (dialects.md, section 1.1): the identity read, which says
who the credential stands for, and the page cap read, which gives the
forge's page cap. Forgejo's are `GET /api/v1/user`, for the `id`, and
`GET /api/v1/settings/api`, for `max_response_items`. GitHub's are 3.4's.
Each plan runs on a connection of the forge's own within `deadline`, and
is counted in no call's cost. The component decides from what a plan
heard (`connection-machines.md`, 2.3), and says `connected` when the forge is ready.

- **Stopped:** a user other than the access's, or any `4xx` but `429`.
  A wrong credential would make the owner act as someone else, and
  another `4xx` says the address names no such API; asking again would
  mend neither. The component says `stopped`, naming the forge and the
  id it heard, or the read and the status. The forge then holds no
  access, and its calls are answered as 3.5 says, until the next
  connect. The other forges run on.
- **Asked again** for anything else (`connection-machines.md`, 2.3), after `start_backoff`, then
  at double the wait up to `start_backoff_max`, or a `429`'s
  `Retry-After` when longer, however long: the rate is honoured (2.3),
  the sum saturates, and the forge's calls meanwhile end at their
  deadlines (`Unavailable`, forgejo.md, section 1.4).

### 3.4 A GitHub App's start

A GitHub forge's credential is an App's (5.2), and its start runs these
of github.md's plans (`github-app.md`, section 1), each on the App's JWT unless it
says otherwise:

1. **The installations,** `GET /app/installations`, paged, which fill
   the installations' table (`connection-calls.md`, 1.4). A `401` says the key is refused:
   stopped, `KeyRefused`.
2. **Each accepted installation's tokens,** minted (5.1), the API's
   token and git's narrower one (5.3).
3. **The identity read,** GraphQL's `viewer { databaseId login }` on one
   installation's token: the bot's user id, which must be the access's
   user. With no installation accepted, there is no token to read it on.
   The forge is ready without it, and the read runs at the first
   accepted installation's mint, its user checked as here.
4. **No page cap read:** GitHub's cap is the dialect's constant, 100.

Each read is asked again, or stops the forge, as 3.3 says. A mint's
refusals are 5.1's.

### 3.5 Access, at runtime

- **What it is:** `Access { credential, user, webhook_secret }`. The
  credential is `Credential::Token(Secret)` for Forgejo and
  `Credential::App { id, key }` for GitHub (README.md, section 7). `user`
  is the forge user id the credential must stand for, never 0, which
  setup's connect read and checked before the owner kept it
  (setup/forgejo.md, section 1, and setup/github.md, section 1). `webhook_secret` is the hook's,
  when the forge has one.
- **None at start is allowed.** A forge with no access runs no start,
  and each of its calls is answered `NotConnected` at once, cost 0. So is
  each call to a stopped forge, and to a forge whose replacement is
  refused, until its next connect.
- **`connect` hands it,** as the first access, a replacement or a
  reconnect after a stop alike. The component checks it at once, then
  starts the forge again (3.3). A connect it refuses says `stopped`,
  `Invalid`, naming the value: a token past `token_bytes`, empty, or with
  a byte skein-http would refuse in a head; a user of 0; a credential of
  the other dialect's kind; an App key that is not a 2048-bit RSA key's
  PKCS#1 DER, which `skein_tls::rs256_public_pkcs1` reads without signing
  (../net/tls.md, section 4), or that is past `key_bytes`; a secret on a GitHub forge
  (`connection-webhooks.md`, 1.1) or on a forge with no listener, or one empty or past
  `hook_secret_bytes`.
- **A replacement ends what the old access began, as a revocation
  would.** An exchange already written is heard to its end, so a write
  that reached the forge is answered. A call's next step is not sent on
  the old access: the call ends `Forbidden`, as the forge would answer a
  revoked credential. Calls waiting for the forge's start wait for the
  new one, within their deadlines. The old access is dropped, not kept.
- **At most one of each answer per connect generation:** `connected`
  when the start is done, and `stopped` if access is refused, including
  a later refusal after Ready (deferred Identity or a key refused at a
  mint). Configured access starts a generation too. A replacement ends
  the old generation unheard: only the newest generation may answer.
- **Another user is refused.** A connect whose user differs from the
  access the forge holds says `stopped`, `Invalid`, naming the user, and
  the forge holds no access after it. A credential that stands for
  someone other than its access's user stops at the start's check
  (3.3). So the owner's domain never acts as a second user on one forge,
  and its client refuses a second author too (client-effects.md, 1.1). The
  component never holds an admin's token (setup/README.md, section 3).

## 4. Calls, repositories and users

Moved to [`connection-calls.md`](connection-calls.md), section 1.

## 5. Credentials and their renewal

- **A kind** says how a credential is made and how long it lasts
  (README.md, section 7): a token lasts until revoked (Forgejo's), and a
  GitHub App's key lasts until the App's owner replaces it, and signs
  the JWTs that mint each installation's tokens, which expire within
  the hour (github-app.md, sections 1.1 and 1.2).
- **One per forge,** in the component's own table: a token, a `Secret`
  of at most `token_bytes`, or an App's id and key, at most `key_bytes`,
  with the installations' tokens (`connection-calls.md`, 1.4). The dialect says how each
  renders on a request: Forgejo's is `Authorization: token <t>`, and
  GitHub's `Authorization: Bearer <t>`, a JWT's or an installation
  token's as the plan's step says (github.md, section 3).
- **It reaches no one:** no record, request line, fact, hint or event
  carries a credential, nothing logs it, and no entry point returns it.
  The one exception is git's token, handed to the owner alone (5.3).

### 5.1 Renewal

A token is never renewed: Forgejo's is held until it is replaced (3.5).
An installation's tokens are, by the component itself:

- **Minted** by github.md's plan (`github-app.md`, section 1.2), `POST
  /app/installations/<id>/access_tokens` on the JWT, for each accepted
  installation that is not suspended: at the start (3.4), when the owner
  accepts it, and before each falls due. A mint answers the token and
  its `expires_at`, wall seconds.
- **Due `renew_before` its expiry.** The expiry is turned into the
  component's own time once, when the mint is heard: the life left is
  `expires_at` less `env.wall`, and the due time is `env.now` plus it,
  less `renew_before` (../../foundation/programming-model.md, section 9). A wall that
  jumps later moves no due time.
  - A mint already due when it is heard is used, and renewed after the
    private backoff, as a failed renewal is: a short `expires_at` or a
    skewed wall makes no loop.
- **Run on the forge's own connections,** within `deadline`, each
  request counted in no call's cost, as a start's reads are.
- **A call that needs a token while its mint runs waits,** within its
  own deadline, and is sent with the new one. Until the old one
  expires, calls go on with it. The new value replaces the old, which is
  not kept.
- **A refused mint:** `401` says the JWT is refused, so the forge stops,
  `KeyRefused` (3.3). `404` says the installation is gone, and `403`
  that it is suspended: the installations are read again (`connection-calls.md`, 1.4) and the
  change goes up. Another `4xx` but `429` stops the forge,
  `TokenRefused`, naming the installation and the status: the App's
  permissions do not cover the mint (github-app.md, section 1.2), and
  asking again would not mend that. Anything else is asked again as a
  start's read is (3.3).
- **A token refused in a call,** `401` on an installation's token, ends
  that call `Forbidden` (forgejo.md, section 1.4), and the component mints
  that installation's tokens anew, once until a mint is heard.

### 5.2 The App's key, and its JWT

- **The key** is the App's RSA private key, 2048 bits, as PKCS#1
  `RSAPrivateKey` DER, at most `key_bytes`. Setup turns the PEM GitHub
  gives into it, at the conversion and at a paste (setup/github.md,
  section 1.2). The component holds it as a `Secret` and checks it at
  connect (3.5).
- **The JWT** is github.md's (`github-app.md`, section 1.1): the App's id as its
  issuer, issued at `env.wall` less 60 seconds, and expiring 600 seconds
  after that, so 540 seconds from its signing. It is signed RS256 by
  `skein_tls::sign_rs256` (../net/tls.md, section 4). Signing is local: no
  request.
- **Reused** until less than `deadline` of its life is left, then signed
  anew, so no request goes out on a JWT that ends before its deadline.
  `connection-machines.md`, 1.4 keeps `deadline` within half its life, so each JWT serves for at
  least half of it.
- **A key that will not sign** stops the forge, `Invalid`, naming the
  key, before any request goes out on it. 3.5's check parses a key and
  does not sign with it, so a key ring refuses only when it signs
  (../net/tls.md, section 4) passes that check and is stopped here.

### 5.3 Git's tokens

Git pushes with a token narrower than the API's: `contents` alone, for
the whole installation (github-app.md, section 1.2). The component
mints it beside the API's token, renews it the same way (5.1), and hands
each one up as a `git token`, with its installation, account and expiry,
at each mint.

- **It keeps only the expiry,** to renew it: the value goes up once, and
  the owner holds it for git, in one git identity per forge whose login
  is `x-access-token`.
- **The owner drops it** at the installation's `Deleted` or `Suspended`,
  and at an `installations` that no longer accepts it. The component
  mints none for those.
- **GitHub doesn't fully honour the narrowing:** a contents-only git
  token can still open issues as the bot for its hour, though it is
  refused an edit (observed 2026-10-20, `token-narrowed-creates-not-edits`,
  github-app.md, section 1.2). The narrowing stays; a second App would
  close the gap, and that is the user's call.
- **Forgejo has none:** its git token is the owner's, set up beside the
  API's (setup/forgejo.md, section 1.3).

## 6. The webhooks' listener

Moved to [`connection-webhooks.md`](connection-webhooks.md), section 1.

## 7. Entities and limits

Moved to [`connection-machines.md`](connection-machines.md), section 1.

## 8. State machines

Moved to [`connection-machines.md`](connection-machines.md), section 2.

## 9. What its owner owes it

The owner's protocol layer owes the routing of section 2, **the values**
of `connection.md`, 3.1, and `connection-machines.md`, 1.3 with limits that pass `connection-machines.md`, 1.4, the log origins' addresses
resolved, **a decision on `stopped`**, **a decision on `listener lost`**
and **the close after its domain's drain** (`connection-machines.md`, 2.5). It owes too:

- **an add** for each forge it makes while the component runs, before
  that forge's connect, and **a remove** for each forge it drops, after
  which it calls that forge no more (3.1);
- **a connect** for each forge it holds access for, at start or when
  setup hands it, with access that setup checked and the owner keeps
  (setup/forgejo.md, section 1, and setup/github.md, section 1); a stopped forge waits, not
  connected, for the next one;
- **the accepted installations** of a GitHub forge, after its connect
  and at each change of the owner's set (`connection-calls.md`, 1.4);
- **git's tokens** kept for git alone, in memory, never logged, and
  dropped as 5.3 says.

The owner's domain owes:

- **at most `calls` calls outstanding,** one live call per token;
- **a locate,** `Read::Locate { owner, name }`, answered
  `Answer::Located { repository }`, addressed with repository 0, its
  answer checked as every other's, a located id never 0;
- **costs,** as the boundary carries them: an answer's `cost` is the
  requests the call wrote, cache fills included, 0 for none, which the
  owner's budget charges (client.md, section 6).

## 10. Verification

- **Step tests:** the start's decision for every state and everything
  heard; the tables' fills, drops and eviction; each limit check, one
  below and at its bound; each listing's page size; the signature check
  against known HMACs, with a header of 63, 64 and 65 digits and one of 64
  uppercase; `Host` for a TLS forge and a named plaintext one, and for an
  IPv4 and an IPv6 address, at the scheme's default port and off it. The
  plans' own are `skein-forge`'s (README.md, section 9).
- **The machine world:** the component against a scripted HTTP peer over
  in-memory streams, for a plaintext and a TLS forge: every state of
  `connection-machines.md`, section 2, with faults at every byte and each deadline passing; a
  start stopped, and asked again with its backoff doubling; a path
  dropped and resolved again; a body past `body_bytes`; an idle
  connection closed for a call to another forge; a held answer offered
  again, and a returned one never; the owner's close in every state,
  crossed with a terminal racing it, `closed` once and last, with no
  spin; access handed at runtime, replaced while calls run and wait,
  and one answering as someone else; a forge added while the component
  runs, a Forgejo and a GitHub one, with their access, and one past
  `forges` ignored; a forge removed while its calls wait and fly and
  its events and hints wait for the owner: its calls answered
  `Unavailable` once, nothing of it said after, the other forges'
  starts kept, and a connection whose socket io names later aborted;
  for a GitHub forge, on the fake's
  GitHub layer and clock (../testing/fakes/forge.md): a renewal due while a call
  waits on it, a mint refused each way of 5.1, a call's token chosen by
  its repository's account, a repository with no row found by trying
  the installations in turn, the puller's catch-up, cursor, gap and
  `304`s, and a job log's redirect followed to its configured origin
  alone, another origin said once per connect as a fact.
- **By hand.** The machine world's `hand` steers the same component one exchange at a time: a connect completes when the test says, an answer is written when it says, in the literal bytes the test holds, whatever fault they carry. What plays the fake goes through `skein_fake_forge_github::inline` (../testing/fakes/github.md, 1.6); no second HTTP face stands in for it.
- **The seam.** A machine-world test reads or seeds the component's installation rows, jobs or connections through the public module `seam` alone: its re-exports of the state's types, and its traits, implemented for `Component` and a connection, whose methods are the `test_` accesses. It is built always, since nothing else reaches the state from outside (../../foundation/cli.md, 3); no tracked source outside the crate and `skein/tests/forge/machine/` names `skein_forge_connection::seam`, as `tests/boundary/tests/skein_worlds.rs` holds. The crate's step tests read its `#[cfg(test)]` `fixtures`, and `hand` reads their copy, `skein-forge-connection-fixtures` (../testing/layout.md, 1).
- **Over TLS, boundary and every-byte sweeps in fuzzy.** Each fault runs
  its own world and a real handshake drawing the kernel's entropy
  (../../foundation/testing-strategy.md, 4.4); its flight belongs to its hello. Both sweeps
  live in the machine world's `suite::fuzzy::tls`, with the same
  assertions: each write's cost equals its bytes, and one terminal and
  one close follow every fault. The boundary sweep cuts at each TLS
  record's start, header's end and flight's end, and at HTTP line,
  head and body ends, one byte either side, with EOF and reset. The
  every-byte sweep applies EOF and reset at every byte of both handshake
  flights and the start's and first call's requests and responses.
  Both run at the gate under ../../foundation/testing-strategy.md, section 8.
- **The listener,** against scripted deliveries: a good delivery hinted,
  its `204` queued before its hint goes up; each answer of `connection-webhooks.md`, 1.3; an event
  the dialect does not hint; a hint held while the owner has no room,
  and dropped at its deadline or the owner's close; `hooks` connections
  full; faults at every byte.
- **The protocol world,** in skein's `tests/forge/protocol`: the client and
  the component against the fake forge's protocol layer and domain
  (../testing/fakes/forgejo.md, section 1), joined by in-memory streams cut at random and
  driven by a scripted durable owner that restarts (client.md, section
  10). Its stories: a creation whose answer was lost is found after a
  restart and not made twice; listings keep up with another party's changes;
  deliveries, signed or read from the log, reach the owner. Its invariants:
  each call is answered once, however its streams are cut; the requests of a
  forge's own aside, the costs the answers reported equal the requests whose
  first byte the component wrote, which the streams count apart from it
  (`connection-calls.md`, 1.1), and the fake served those but the ones whose stream was cut before
  their head was whole, which are charged and not served; no credential
  reaches a record, a request line or a fact, the handed, replaced and
  renewed ones too. The fuzzy runs take seeds and stay within 10 seconds (5
  seconds before the user's doubling of 10-22, provisional) (../testing/README.md,
  section 7).
- **Memory,** against the counting allocator (skein-heap): all `calls`
  outstanding, tables full and each webhook holding a hint, with runtime
  access, App keys and JWTs, installation tokens and pullers' hints held
  together. Scenarios check each kind and both kinds together under
  named ceilings with headroom.
- **The conformance** drives the component against a real Forgejo
  (../testing/fakes/throwaway.md, section 2).

## 11. Open questions

- **A clock off by more than a minute.** A JWT is issued a minute in
  the past, so a wall more than 60 seconds ahead or behind GitHub's
  makes it refused, which reads as `KeyRefused`, and a skewed wall
  shortens or lengthens an installation token's life as 5.1 measures
  it. Default: the host keeps its clock, and the diagnostic says to
  check it. Measuring the life from the mint's `Date` answer instead
  would take skew out of the tokens, not the JWT.
- **Runtime DNS** for a job log's origin. Its address is resolved once
  (`connection-calls.md`, 1.5), so a host whose address moves is refused until the owner
  restarts. Default: until io resolves names.
- **A listener over TLS,** for a forge on another host, waits for
  skein-tls's server side (../net/tls.md). Default: that forge polls alone
  until then.
- **What a connection's states hold past their tables** (`connection-machines.md`, 2.1, 2.4): what
  outlives an exchange in Closed and Idle, the deadline Idle keeps, and a
  refused answer's wait in Answering. Whether the tables take them or the
  states lose them, which moves events and times.
- **An installation's lifecycle** (`connection.md`, 3.4, `connection-machines.md`, 2.6): `GitHub`'s mint, read and
  identity flags take lifecycle states (../../foundation/programming-model.md, section
  5.4). Default: as they stand, until a debt item names them.
- **A call's side beside its dialect** (`connection-machines.md`, 1.1): a step matches the call's variant
  with its entry's dialect, since a GitHub call takes its installations from
  the entry. Whether the variant can reach them with no second match, so that
  the dialect is read once. Default: as it stands, until a debt item names it.
- **Who sets a connection's deadline** (`connection-machines.md`, 2.1): a state holds its `until`, a holder
  sets its own above `exchange.start`, and six more `until` calls across the
  component set it otherwise, where `connection-machines.md`, 2.1 has one place after each transition.
  Whether a transition can take it from its holder. Default: as they stand, until
  a debt item names them.
- **A JSON body's state** (`connection-machines.md`, 2.1): `Exchange` keeps the body and its next request
  beside `receive`, not in `Receiving::Json` (../../foundation/programming-model.md, section
  5.4), since a demand can reach them after `discard`. Whether `discard` clears
  that demand, so that they move. Default: as they stand, until a debt item
  names them.
- **The forge kit's fixtures beside the crate's** (10):
  `skein_forge_kit::fixtures` keeps its own `limits`, `shared` and `settings`,
  near-copies of `skein-forge-connection-fixtures`', because temper's and other
  worlds read the kit, and its numbers are its own. Whether the kit takes the
  fixtures crate's instead. Default: the copy stays.
