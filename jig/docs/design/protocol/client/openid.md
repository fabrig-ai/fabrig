# The OpenID provider

The clients' component may be an OpenID Connect provider for the
deployment's own web applications: a forge, a wiki, whatever signs its
people in by OpenID. A person signed in to the application is signed in
there by the code flow, with no consent page: the clients are the
deployment's, configured by its owners. skein's issuer
(`skein-oauth-issuer`, skein's `oauth/issuer.md`) checks the requests
and writes the documents and tokens; the component holds what lives,
its codes and access tokens, and decides who is signed in. The routes
and their addresses are the application's (`default-http.md`, 1: no provider route).

- **Configured** by `clients.provider` (`../configuration-engine.md`, section 1):
  its signing key (a 3072-bit RSA key, PKCS#8, a file of the
  service's own user, mode 0600, made on its host and never copied,
  printed or logged; any other form, size, mode or owner refuses the
  start, each with its reason), its clients (each an id, a secret and
  one redirect URI) and the providers whose subjects are verified
  addresses. Its tests, from the key file's reader up, use fixed
  3072-bit test keys committed beside them, made for the tests and no
  deployment's, and a fixed client secret, so that each run signs the
  same bytes and none waits on a key's generation. With
  none, the component is no provider: `Authorize` is refused
  `no_provider`, and a `Redeem` or a `Claims` is answered as an unknown
  client.
- **Built by the application's startup,** which alone knows its routes:
  it gives jig's helper, in `jig-protocol-config::clients` beside the
  key's reader, the configuration's member and its own issuer
  identifier, its public origin, and the URLs of its authorization,
  token, userinfo and key set routes; the helper builds skein's
  `Issuer`, which the startup hands the component in its settings,
  after taking from it the discovery document and the key set, which
  the application serves itself (skein's `oauth/issuer.md`, section 3).
- **A client added or removed while it runs.** An application whose
  clients come and go with what it sets up (temper's owned forge:
  temper's `protocol/setup/place.md`, 6) configures the provider with
  its key and none or some clients, and gives the component a client to
  add, or a client's id to remove, while it runs, each as a request of
  its root's. A client added is checked as a configured one is (its id,
  its secret's file read by the application, its one redirect URI), and
  refused, naming the reason, past the provider's `clients` limit, with
  an id already held, or with no provider. One removed has its codes and
  access tokens dropped, and is answered as an unknown client from then
  on; an id it doesn't hold is refused. A restart holds the configured
  clients alone, until the application adds its own again.
- **`Authorize`,** a client's message: the authorization request's
  query, as the browser's address carried it to the application's
  page, which the client hands on (the application's client decides
  that the address is an authorization). The component answers, in
  this order:
  1. the query checked by the issuer: refused *malformed*, *unknown
     client* or *unknown redirect*, which the page shows and follows
     nowhere; or an error the issuer sends back to the client's
     redirect URI, answered *go* with that address;
  2. the exchange's credential, checked as any exchange's (`sign-in.md`):
     none, unknown, ended or expired is *signed out*, and the page signs
     the person in, then sends the same `Authorize` again;
  3. the signer, read from people (`../../domain/people/README.md`, section 3): a
     service is refused *not a person*, as is a subject that is not
     UTF-8;
  4. a code drawn, kept with the authorization and the person: *go*,
     with the client's redirect URI carrying it and the client's state.
     The page replaces its address with it.

  *Not ready* while the engine stops, and *busy* when the codes are
  full (below), each changing nothing.
- **The person** the claims name, from the signer's identity:
  `sub` its provider's number in decimal, `:`, then its subject. A
  party is one identity (`../../domain/people/README.md`, section 3), and its
  identity outlives the store: a deployment started afresh gives the same
  person the same `sub`, so a client's account linked to it stays theirs,
  where a party's number would pass to whoever the fresh store numbers
  so. It is stable for as long as the configuration numbers the provider
  so; `preferred_username` its login; `name` its name, where
  not empty; and, where its provider is one of `addresses`, `email` its
  subject, verified. The same for the ID token and the userinfo answer.
- **The code's exchange and the claims** are messages of the
  application's server, as `Mint` is (`sign-in.md`), from its token and
  userinfo routes: `Redeem` with the request's `Authorization` and
  `Content-Type` headers and its body, answered `Redeemed`; `Claims`
  with its `Authorization` header, answered `Claimed`. Each answer is a
  status, an optional challenge for `WWW-Authenticate`, and a body,
  which the server sends as it is, `application/json` and
  `Cache-Control: no-store` (skein's `oauth/issuer.md`, sections 5 and
  7). A code is spent at its first presentation, whatever follows;
  `Redeemed` with tokens draws an access token, kept with the person.
- **In the component's memory,** never the store: a code lives one
  minute, an access token five; each 32 bytes of the component's
  generator (`sign-in.md`) in base64url, found by its digest. Their counts
  are limits (`../limits.md`): with every code's room
  taken, `Authorize` is busy; with every access token's, a code is not
  drawn either, so a code presented always finds a token's room. A
  restart drops them all: a sign-in under way fails, and the person
  signs in again.
- **On the default mapping,** `POST <prefix>/v1/authorize` (`default-http.md`, 1).
- **Testing:** the component's protocol world
  (`jig-engine-clients-world::protocol`) plays the application's page
  and server and a client of the provider's, on the world's own clock,
  holding back people's or the credential's read where a story needs it
  outstanding: a sign-in through `Authorize`, `Redeem` and `Claims` with
  the same `sub`, its ID token signed with the fixed key and checked by
  skein's RS256 verification, which a different signing input fails; a
  code spent once; a code past its minute and an access token past its
  five; a wrong redirect URI, at the authorization and at the exchange;
  a wrong secret; a wrong verifier; signed out, then signed in; a
  service refused; the codes full and the tokens full; a restart
  keeping no code or token; no provider. Clients added and removed
  while it runs: an added one signing in as the same person; a removal
  dropping only its own codes and tokens, which re-adding it revives
  none of; each change's refusal named, leaving the registrations as
  they were; and a removal while people or the credential is being
  read, after which the late reply draws no code. skein's issuer world
  checks the protocol against skein's client; the application's
  process tier, against a real client.
