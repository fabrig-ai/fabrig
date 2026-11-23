# jig's protocol layer: decisions

## 1. Decisions this design takes

- **jig ships components, not references to copy,** for its boundaries
  whose vocabulary is jig's: the channel's two sides, accounts, host
  tools, configuration, the client's sides. What is generic is skein's,
  what is the application's is the application's; the hard parts of
  jig's own boundaries (authentication, liveness, keeping secrets before
  use, decoding the LLM's calls) are written once. The store too: jig's
  component takes the application's records as bytes (`store/README.md`,
  section 7).
- **Records from schemas,** with skein's generator, for jig's channel and
  its other codecs, as smith's are.
- **The channel on skein's channel,** one per worker, dialled by the
  worker, TLS between hosts and plaintext only over loopback or in one
  process, a name and a secret authenticating the worker, and the
  wires both speak checked at the opening (`channel/wires.md`, 3).
- **The hub's vocabulary on both ends of the channel,** so the engine's
  root routes a worker's link as it routes its own hub (`channel/README.md`,
  section 1).
- **The worker's link policy is its component's:** dialling, redialling
  with backoff, liveness and the grace's clock; its stop bound counted
  from the engine's view of the link (`channel/worker-link.md`, 1.1).
- **Host tools' calls decoded in the protocol layer,** after the fleet has
  fenced them, by `jig-protocol-tools` for the core's tools and by the
  application's code for its connectors' (`channel/owners.md`, 1.3). It is a
  deliberate exception to decoding before the domain
  (skein/docs/foundation/programming-model.md, section 4): smith's domain carries a host tool's
  input as the LLM wrote it, so a call from a run in the engine is inside
  the domain before anything could decode it, and only the fleet can tell
  a live attempt's call from a stale one. One path serves both shapes:
  fenced, then handed out to be decoded, and handed back typed. A run's
  result takes the same path (`channel/owners.md`, 1.1).
- **A run's text keeps one contract,** by turn, retry and offset,
  whatever its source (`../domain/hosts/hub.md`, 5): a host that drains
  `smith-inline-agent`'s content, or agent processes on a smith channel
  that carries it at the host's ask (`needs.md`, section 2; `channel/owners.md`, 1.4).
- **Agents are smith's,** both capabilities speaking smith's parent face
  (`README.md`, section 4); jig ships no agent crate.
- **smith's payloads as bytes in jig, typed only in smith.** Siblings
  share no domain types (skein/docs/foundation/programming-model.md, 4.5): the hub, the core
  and the channel's components carry smith's payloads as bytes and link
  no smith crate. The root that composes an agent capability hands each
  payload out, and its protocol layer's routing decodes or encodes it,
  and decodes smith's facts into jig's activity, by the path of host
  tools' calls (`../domain/hosts/hub.md`, 8; `channel/owners.md`, 1.1).
- **smith's LLM component for runs in a host's own process,** with each
  slot's result contract kept beside it (`llm.md`, section 3).
- **A connector's webhooks where the application routes them:** on a
  listener of the connector's own, owned by its protocol component, or
  on the application's server beside its clients, routed by path, so a
  deployment may expose one origin. jig owns no listener for clients,
  so the choice is the application's.
- **A reference binary,** `jig-test`, which an application copies for
  its `main`, and the binary jig's end-to-end and live tests run
  (`service.md`, section 7, and `service-testing.md`, section 1).
- **LLM accounts on skein's accounts driver and the secrets' store:**
  sign-in, refresh, sign-out and tokens lent by generation are skein's;
  records and keys are kept in the store's secrets' records before
  lent; program logins stay the program's, on each host; the accounts
  child keeps which sources are usable and who holds which generation
  (`llm.md`, section 5).
- **What is generic leaves jig,** within reason: a mechanism that is not
  about jig's vocabularies goes to skein, or to smith when it is about
  agents, generalised, before jig uses it: joined sockets (`needs.md`, section 1),
  the accounts driver, the agent in a host's process, the UI kit (the
  tree, the text shell, the browser parts, the bundler and the fake
  person: `needs.md`, section 1), and a run's text on smith's channel (`needs.md`, section 2
  ). jig keeps what is entwined with its domain: the worker's link
  policy, whose grace and stop bound are the fleet's, and the
  translations.
- **Crates named by layer, and every client crate under `jig-client`:**
  `jig-domain-*` and `jig-protocol-*` for what runs in the engine and on
  workers; `jig-client-{domain,protocol,<kind>}-*` for the client's,
  wherever it runs, with `jig-client` their base, as `skein-ui` is
  `skein-ui-*`'s (`../crate-names.md`, section 1).
- **On one person's machine, a browser on loopback** signs in by the
  printed link: the clients' component gives a one-time secret at
  start, and the application's startup prints it in a URL of its own,
  `<prefix>/v1/sign-in/link?secret=…` under the default mapping
  (`client/sign-in.md` and `client/default-http.md`, section 1).
- **Presentation, the server and the transport are the application's.**
  jig's clients' component takes jig's messages with an opaque
  credential, and knows no HTTP, path, cookie, host, CORS or server-sent
  event; jig reserves no path. The application's service owns the
  listener and its URL space, mounts the component where it chooses,
  and owns every HTTP and browser guard, since the guards that protect
  a browser's credential belong to the transport, not to jig's model.
  jig ships what an application may take or leave: its default HTTP
  mapping, both ends, with example guards, and attack stories in its
  test kit, checked and never mandated (`client/default-http.md`;
  `browser/serving.md`, 1.2 and `browser/testing.md`); and its reference UI, on the
  public client alone, which proves the API is enough
  (`../client/domain/`).
- **jig owns credentials, the application owns sign-in flows.** jig
  mints, verifies, expires and revokes credentials, and keeps their
  digests; its built-in ways of signing in are the printed link,
  services' credentials and a sign-in approved by a signed-in browser.
  Every other way runs on the application's
  routes and ends in a `Mint`, which only its server code sends, and
  which names an identity of a provider of kind application
  (`configuration-engine.md`, section 1). jig adds no "acting for": a bot
  relaying a person's decision holds that person's own credential
  (`../domain/people/README.md`, section 3; `../client/domain/contract/README.md`,
  section 2).
- **Versions per owner, refused by type:** a message in a version the
  engine does not serve is answered *version not served*, naming the
  owner and the ranges served; `jig-client` and the engine come from
  one jig pin (`README.md`, section 7).
- **Templates are configuration, served and never expanded by the
  engine** (`configuration-templates.md`, 1), and lists are task queries: live
  tasks scanned in memory, ended ones through one index of the store,
  chosen by the query's keys (`store/loads.md`, 3 and 4).
- **Configuration as strict JSON,** read once before the loop, composed
  per child, secrets in private files, limits by profile
  (`configuration.md`).
- **The store on skein-kv in the engine:** one transcript file per task,
  the current window's start kept in the map, records from schemas,
  another version refused (`store/README.md`).
- **Workers are alike:** configured the same but for their name and
  secret, keeping nothing between runs that the engine does not hand them
  again. Every credential a run needs, an LLM account's or an
  application's for its workspace's systems, is the engine's, granted
  with the assignment (`channel/README.md`, section 7), but a program's login,
  which is the program's own on its host, and is why a host keeping a
  person's logins takes only runs that person's accounts pay for, until
  a run's process tree is contained from them (`llm.md`, 5.5).
- **No upgrade in place, and no rolling upgrades** (`README.md`, section 7).
- **One process** (`README.md`, section 4): the workers joined to the engine by
  in-process addresses, their agents child processes, a browser on
  loopback, with no TLS.
