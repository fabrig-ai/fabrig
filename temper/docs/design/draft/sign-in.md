# Signing in with Google, and Forgejo through temper

**Superseded** by `../protocol/sign-in.md`, which keeps these sections' numbers and adds 9, temper as Forgejo's provider. Kept until the mockups that cite it move (`../client/mockups/`).

Draft, 2026-10-23. A temper deployment that people reach at a name of
its own, over HTTPS, where they sign in with their Google account, and
whose Forgejo signs them in through temper: once signed in to temper, a
person opens Forgejo signed in, with nothing more to do. The names here
are examples: temper at `https://temper.example.com`, Forgejo at
`https://git.example.com`. It records a session with the user on
2026-10-23. It crosses skein, jig and temper on purpose; an owner
splits it.

The mockups draw its pages ahead of this draft: `sign-in.html`, in the
world Google sign-in (`../client/mockups/README.md`).

## 1. In one page

- **One host, two sites, a TLS proxy in front** (section 2). `temper
  serve` runs the engine and its worker, its clients' listener on
  loopback, and a proxy (nginx, here) terminates TLS for temper's name
  in front of it. Forgejo runs beside it, managed
  (`../protocol/setup.md`, section 5), behind the proxy at a name of its
  own. Not `temper box`: Forgejo is the owner's, run as a service of its
  own.
- **Two hops of OpenID Connect:**

  ```
  Google ──▶ temper.example.com ──▶ Forgejo, at git.example.com
     temper signs people in     temper signs Forgejo's users in
  ```

- **Google signs people in to temper** (section 4) through jig's
  application sign-in: temper's routes run Google's flow with skein's
  OAuth client, and jig's `Mint` makes the sign-in (jig's
  `protocol/client.md`, section 8). Both are built; temper has never
  used them for people.
- **The sign-in page is stock** (4.1): the same for every deployment,
  "Sign in" and a button for each way of signing in, with nothing of
  the deployment's own. A deployment's own name and words wait
  (section 9).
- **Only members sign in** (section 3): the configuration names each by
  their Google address. Google signs anyone in; temper mints for a
  member alone, and answers anyone else with a page that says so.
- **temper is Forgejo's OpenID provider** (section 5): Forgejo has an
  authentication source that names temper, so Forgejo's sign-in
  bounces through temper, which answers at once for a person signed
  in. temper's "Open Forgejo ↗" starts that sign-in itself, so a person
  signed in to temper is signed in to Forgejo with no click.
- **The provider is new, the client side is not.** skein has the OAuth
  client, its ID token's checks and an RS256 signer; jig has `Mint`.
  What no repository has yet is the issuer's side: the discovery
  document, the authorization, the token, the user's claims and the
  key set (section 7).
- **Two stages** (section 7.4): first Google into temper, with Forgejo
  signing in its own way; then temper as Forgejo's provider.
- **Signing out of temper does not sign out of Forgejo** (section 6),
  and the signed-out page says so.

## 2. The deployment

### 2.1 The host

- **`temper serve`,** standalone (`../protocol/configuration.md`,
  section 2): the engine and one worker in one process, the agents on
  the same host. Its clients' listener on loopback at a fixed port,
  `127.0.0.1:8367`; its webhooks' listener on loopback, which Forgejo
  reaches on the same host.
- **Forgejo, managed** (`../protocol/setup.md`, section 5): run by its
  owner as a service, its runner beside it; temper sets itself up on
  it with a site admin's token and makes its bot. Forgejo's `ROOT_URL`
  is `https://git.example.com/`.
- **The proxy terminates TLS for both names,** a certificate each, and
  passes each to its loopback listener. For temper:
  - the `Host` passed through as the person's browser sent it, which
    the Host allow-list then checks (2.2);
  - jig's stream, `GET /jig/v1/stream`, unbuffered (nginx's
    `proxy_buffering off`) and with a read timeout longer than the
    stream's heartbeat (jig's `protocol/client.md`, 17.2);
  - the printed link's route, `/jig/v1/sign-in/link`, denied from
    outside (2.3);
  - plain HTTP answered with a redirect to HTTPS, and nothing else.

### 2.2 The public address

Adopted: `../protocol/sign-in.md`, 2.2, and
`../protocol/configuration.md`, section 3, `clients.public`.

### 2.3 The printed link

Adopted: `../protocol/sign-in.md`, 2.3.

## 3. Who may sign in

Adopted: `../protocol/sign-in.md`, section 3, and the binding, its
section 5.

## 4. Signing in with Google

Adopted: `../protocol/sign-in.md`, sections 4 and 7: the routes, the
flow, its endings' pages and the browser's sign-in page. Stage 2 adds
to the page's facts the forge's name, for 5.1's "to continue to", as a
further `temper-` meta (its section 7).

## 5. temper as Forgejo's sign-in provider

### 5.1 What a person sees

- **From temper:** "Open Forgejo ↗" goes to Forgejo's own route that
  starts a sign-in with the temper source, `/user/oauth2/temper` on
  Forgejo's origin; Forgejo sends the browser to temper, temper answers
  at once, and Forgejo opens signed in. No click.
- **At Forgejo first:** Forgejo's sign-in page offers "Sign in with
  temper". Signed in to temper, one click; signed out, temper's sign-in
  page says "to continue to" the forge, Google follows, then Forgejo,
  signed in.
- **The first time,** Forgejo makes the person's account, named by
  their login (5.4); an account already there is linked once, by its
  password (5.4).

### 5.2 The endpoints

On temper's clients' listener, its issuer the public origin:

| Route | What it is |
|---|---|
| `GET /.well-known/openid-configuration` | the discovery document: the issuer, the endpoints below, the code flow alone, the scopes, the claims, the signing algorithm |
| `GET /oauth/authorize` | the authorization, a page of the client's (5.3) |
| `POST /oauth/token` | the code's exchange, from Forgejo's server, with its client secret: the ID token and an access token |
| `GET /oauth/userinfo` | the claims, for the access token |
| `GET /oauth/keys` | the key set |

- **One client, configured:** Forgejo's, `forgejo`, for the forge
  temper knows it as, its redirect URI exactly Forgejo's origin's
  `/user/oauth2/temper/callback`, its secret a secrets' file. Several
  clients are a list; one is all a deployment with one Forgejo needs.
- **Codes and access tokens are the engine's memory:** a code lives a
  minute and is spent once, bound to its client, its redirect URI, its
  sign-in, its nonce and, when Forgejo sends one, its PKCE challenge;
  an access token lives five minutes. A restart drops them: a sign-in
  to Forgejo under way fails, and the person signs in again.
- **The signing key** is drawn at the first start and kept in jig's
  secrets' store. Its algorithm is RS256 (section 9).
- **What Forgejo 17's client does,** and the endpoints meet (its goth
  v1.82.0): it always sends a PKCE challenge, S256, and its verifier at
  the exchange; it authenticates at the token endpoint by Basic first,
  and by its secret in the body when that fails, so the token endpoint
  takes both; it calls the userinfo endpoint only when the discovery
  document advertises one, by `GET` with the access token as Bearer,
  and requires its `sub` to be the ID token's. The ID token is required
  in the token's answer either way.

### 5.3 The authorization

jig's cookie is `SameSite=Strict` and its `Path` is jig's prefix
(jig's `protocol/client.md`, 17.3), so a request to `/oauth/authorize`
never carries it, and a navigation that started on another site (a
link to Forgejo in a mail) would not carry it anywhere. So the
authorization is decided by the client, not by the route:

1. `GET /oauth/authorize?…` is answered with temper's page, as every
   other `GET` is. The client root's routing reads it as a place of its
   own, `Authorize`, its parameters in the address.
2. The client signs in by `Transport`: its exchanges are same-origin,
   started by temper's page, so the cookie goes with them whatever site
   the navigation came from.
3. **Signed in:** the client sends jig's new `Authorize` with the
   request's parameters. jig checks the client, the redirect URI and
   the scopes, draws the code for the signer's sign-in, and answers the
   address to go to, Forgejo's callback with the code and Forgejo's
   state. The page replaces its address with it, so the back button
   never returns to the authorization. The person sees "Continuing to
   Forgejo…" for as long as that takes.
4. **Signed out:** the sign-in page, under its "Sign in", "to continue
   to" the forge as the client names it, "Forgejo at HOST"
   (`../client/forge.md`, 5.4), its `then` the authorization's address,
   so Google's return lands back at step 1.
5. **Refused** (an unknown client, a redirect URI not registered): the
   page says so and goes nowhere: a redirect URI that does not match is
   never followed.

No consent page: Forgejo is the deployment's own, configured by its
owners.

### 5.4 The claims, and Forgejo's account

- **`sub`:** the identity, Google's provider number and the member's
  address. It outlives the store: a deployment started afresh gives
  Forgejo the same `sub` for the same member, never a party's number,
  which a fresh store would give to someone else.
- **`preferred_username`:** the member's login. **`email`,**
  verified: the member's address. **`name`:** as Google gave it. No
  `nickname`: Forgejo's client takes a user's nickname from `nickname`,
  then `preferred_username`, so without one `USERNAME = nickname` and
  `USERNAME = preferred_username` (below) both name the account by the
  login.
- **Forgejo's settings** (its configuration cheat sheet):
  - `[oauth2_client] ENABLE_AUTO_REGISTRATION = true` and `USERNAME =
    preferred_username`: a member's first sign-in makes their account,
    named by their login;
  - `ACCOUNT_LINKING = login`, Forgejo's default: an account that
    already exists with that name or address is linked once, by its
    password. Not `auto`, which links by a matching name or address
    alone;
  - `[service] ALLOW_ONLY_EXTERNAL_REGISTRATION = true`; and, once the
    source works, `ENABLE_INTERNAL_SIGNIN = false`, so the temper
    source is Forgejo's only sign-in on the web. Its admin keeps
    Forgejo's command line, and temper's bot its tokens.
- **The source is added on Forgejo's host,** since Forgejo's API has no
  route for authentication sources (Forgejo 17):
  `forgejo admin auth add-oauth --name temper --provider openidConnect
  --key forgejo --secret … --auto-discover-url
  https://temper.example.com/.well-known/openid-configuration`. temper's
  setup for a managed Forgejo prints it with its values; for an owned
  Forgejo, temper's preparation runs it (`../protocol/setup.md`, 6.3).
  The name `temper` is the callback's path segment, so it is fixed.

## 6. Signing out

- **Signing out of temper ends temper's sign-in only.** Forgejo keeps
  its own session, and Google its own. While a forge signs its people
  in through temper, the signed-out page says so: "You signed out.
  Forgejo at HOST keeps its own sign-in: sign out there too."
- **Signing out of Forgejo** ends Forgejo's session; its next sign-in
  bounces through temper, and a person still signed in to temper is
  signed in again at once.
- **Single sign-out is open** (section 9).

## 7. What each repository gains

### 7.1 skein

- **The issuer's half of OpenID Connect,** sans-IO, beside `skein-oauth`:
  an authorization request checked against a client's registration;
  the token request, with a client's secret by basic or in the body,
  and its PKCE; the ID token's claims and signature, by
  `skein_tls::sign_rs256` or the algorithm chosen (section 9); the
  discovery document and the key set. A crate of its own, as the
  client's is.
- **A world for it against `skein-oauth`:** the client and the issuer
  each other's peer, with `skein-fake-oauth` as the independent issuer
  it already is.

### 7.2 jig

- **The public address in the default mapping** (2.2): the Host
  allow-list's extra name, and `Secure` decided by the exchange's
  `Host`.
- **The provider** in the clients' component, since jig owns sign-ins
  and credentials (jig's `domain/people.md`, section 3): the clients'
  registrations in its configuration; `Authorize` from a signed-in
  client, on the default mapping as `POST <prefix>/v1/authorize`; the
  code's exchange and the claims as messages of the application's
  server, as `Mint` is; the signing key in its secrets' store. The
  routes and their addresses stay the application's (jig's
  `protocol/client.md`, 17.1: no provider route).
- **Nothing for Google:** jig's sign-in helper for OAuth stays where
  jig's `protocol/client.md`, section 15 puts it, sugar adopted once a
  second application needs it. temper's flow is its own.

### 7.3 temper

- **Stage 1's,** adopted into `../protocol/sign-in.md`: the public
  address, the members, Google's routes, their endings' pages, the
  binding and the browser's stock sign-in page.
- **The provider's client for Forgejo** in the configuration (5.2).
- **The provider's routes** (5.2), handing each to jig.
- **The client:** the browser's stock sign-in page with the configured
  ways (4.1), the `Authorize` place (5.3), and "Open Forgejo ↗" to
  Forgejo's route for the temper source when the forge signs in through
  temper.
- **The setup:** the `add-oauth` command printed for a managed Forgejo,
  run for an owned one (5.4).

### 7.4 The stages

1. **Google into temper:** 2.2, 2.3, section 3, section 4, and jig's
   public address; adopted into `../protocol/sign-in.md` (jig's part
   built, jig 936c9f1a2). The deployment is usable by its members; Forgejo
   signs them in by its own passwords meanwhile.
2. **temper as Forgejo's provider:** section 5, skein's issuer and
   jig's provider. Forgejo's passwords are then retired by
   `ENABLE_INTERNAL_SIGNIN = false`.

## 8. Testing

- **The Google flow:** adopted, `../protocol/sign-in.md`, section 8.
- **The provider against a real Forgejo** in the process tier, on
  skein's throwaway Forgejo with the temper source added by its command
  line: a scripted client with a cookie jar follows Forgejo's sign-in
  through temper and back, and Forgejo's API then names the member's
  account. It proves the claims and Forgejo's settings, which no fake
  can.
- **The browser tier, for what only a browser proves:** jig's strict
  cookie absent from a navigation that started on another site and
  present on the page's own exchanges (5.3); the binding cookie's
  return from Google's cross-site redirect is stage 1's
  (`../protocol/sign-in.md`, section 8).

## 9. Open questions

- **A deployment's own sign-in page.** Default: stock (4.1). Later,
  perhaps a name and a line in the configuration, or a template of the
  owner's, as Forgejo takes its `APP_NAME` and `APP_SLOGAN` and
  templates under its custom directory.
- **The signing algorithm: RS256.** Forgejo 17's OpenID Connect client
  (goth v1.82.0) reads the ID token's payload and checks its `aud`,
  `iss` and `exp`, with no fetch of the key set: it verifies no
  signature, so it accepts ES256, and any algorithm, alike. That is
  acceptable here, since Forgejo takes the token directly from temper's
  token endpoint, over TLS, authenticated by its secret by Basic
  (OpenID Connect Core, 3.1.3.7). So ES256 buys Forgejo nothing, and
  RS256 stays, for its own reasons: `skein_tls::sign_rs256` signs it;
  its signatures (PKCS#1 v1.5) are deterministic, so the worlds replay
  byte for byte, where `ring`'s ECDSA draws from a system randomness of
  its own and signs differently on every run; and skein-oauth's client,
  the issuer's peer in its world, verifies RS256 alone. `ring` signs
  RS256 with a key it is given but cannot make one: the key, 2048 bits,
  is made at the first start by the setup (`openssl genrsa`) and kept
  in jig's secrets' store. Rotation waits.
- **Single sign-out: none,** said on the signed-out page. Forgejo 17
  takes no signal from its provider: no back-channel, front-channel or
  session-management logout, and no RP-initiated one (it parses the
  discovery document's `end_session_endpoint` and never uses it); its
  own sign-out is local. A logout Forgejo would take from its provider
  waits for Forgejo to take one.
- **Members at runtime.** jig seeds roles on a first start only. A
  member added later signs in with no project; an owner's invitation,
  and roles granted from the client, wait for a design of their own.
  Default: the configuration's members, roles by the seed.
- **Owners as Forgejo's admins.** Forgejo's source can make admins of a
  group claim (`--admin-group`) and map groups onto teams
  (`--group-team-map`). Default: none; Forgejo's admins are made on
  Forgejo.
- **Google's Workspace domain** as a member rule (`hd`), for a
  deployment of one organization. Default: addresses alone.
- **The person's forge identity.** With temper as Forgejo's provider,
  temper knows each member's Forgejo account by construction: the way
  to a person's approval posted as the person (`../README.md`, section
  8) and to "the forge as people's sign-in provider", turned around
  (`../client/README.md`, section 11).
- **The terminal for a member who is not the owner:** a device code's
  flow. Default: the browser alone.

## 10. The mockups

`../client/mockups/sign-in.html` in the world Google sign-in
(`world-google-sign-in.js`): the stock sign-in page signed out;
arriving from Forgejo signed out; each of 4.3's endings; signed out.
Behind it, the chats page and the Forges page, whose Forgejo at
git.example.com opens signed in through temper. Each is marked "ahead"
while this draft is one.
