# jig's protocol layer: components

## 1. Components

| Crate | Kind | In | What it does |
|---|---|---|---|
| `jig-protocol-worker-wire` | codecs | engine, worker | the channel's records, a worker's credential, and why a channel or a dial ended (`channel/README.md`) |
| `jig-protocol-engine-workers` | component | engine | workers' channels: opening, authentication, liveness, translation to and from the hub's vocabulary (`channel/README.md`) |
| `jig-protocol-worker-link` | component | worker | the link: dialling, liveness, the grace's clock, relays across channels, translation to and from the hub's vocabulary, smith's payloads carried as bytes (`channel/README.md`) |
| `jig-protocol-charter` | translation | engine | the core's charter put into smith's, as smith's bytes, and a run's result read back (`../domain/hosts/charters.md`) |
| `jig-protocol-worker-agents` | component | worker | the program sources and the worker route, between smith's host half and the channel (`service-parts.md`, section 1) |
| `jig-protocol-tools` | translation | engine | the core's host tools: their schemas and descriptions, their calls decoded into the core's typed requests, their answers rendered as text (`channel/owners.md`, 1.3) |
| `jig-protocol-accounts` | translation, table | engine | the accounts child's vocabulary to skein's accounts driver's and back, and the values lent, tokens and keys with their scheme (`llm.md`, section 5) |
| `jig-protocol-accounts-codec` | codecs | engine | the generated record code of `jig-protocol-accounts`' family (`schema/moved-v1.schema`), below it, which keeps its schema and translates it (`tools/crates/design.md`, 3.2) |
| `jig-protocol-config` | parsing | engine, worker | jig's part of a configuration, parsed and checked (`configuration.md`) |
| `jig-protocol-store` | component | engine | jig's records and the application's on skein-kv, transcripts' files, the secrets' store; the owner of its families' current schemas (`store/README.md`) |
| `jig-protocol-store-records` | translation | engine | jig's records to the store's generated families and back, below `jig-protocol-store`, which re-exports it as its `codec` (`store/crates.md`) |
| `jig-protocol-store-people`, `jig-protocol-store-tasks`, `jig-protocol-store-core`, `jig-protocol-store-settings`, `jig-protocol-store-families` | codecs | engine | the store's families' generated record code, below `jig-protocol-store-records`, which translates them, and `jig-protocol-store`, which keeps their current schemas (below) |
| `jig-client-protocol-wire` | codecs | engine, client | jig's wire records, both ends, from schemas: jig's messages, their operations and compound requests, subjects and reads, documents, the stream's events, the small store's records (`client/wire.md`, section 1) |
| `jig-protocol-engine-clients` | component | engine | the clients' component: jig's messages from the application's transport, each with its credential as opaque bytes, and to it; a `Mint` from the application's server; streams and their pacing; credentials minted, verified, expired and revoked, the printed link's secret; the OpenID provider's codes and access tokens (`client/openid.md`); translation to and from the core's vocabulary. No HTTP, path, cookie or listener (`client/wire.md`, 2.2) |
| `jig-protocol-engine-clients-codec` | codecs | engine | the generated record code of the clients' component's cursor family, below it, which keeps its schema and translates it (`tools/crates/design.md`, 3.2) |
| `jig-protocol-engine-clients-http` | mapping | engine | the default HTTP mapping's engine end: its routes under the prefix the application mounts it at, the stream as server-sent events, the credential as a cookie or a bearer, and the example guards (`client/default-http.md`; `browser/serving.md`, 1.2), and the server part an application builds its listener on (`service-parts.md`, 1.4) |
| `jig-client-protocol-link` | component | client | jig's messages encoded and decoded, the stream's events, the small store's records; no HTTP (`client/wire.md`, 2.3) |
| `jig-client-protocol-http` | mapping | client | the default HTTP mapping's client end: a message as a method, a path, headers and a body; the stream's server-sent events cut into events; for the host and wasm32 (`client/default-http.md`, 4) |
| `jig-protocol-transcript` | decoding | engine, client | turns' bytes into the client's vocabulary (`../client/domain/client/conversations.md`) |
| `jig-client-shell-web`, `jig-client-shell-native` | shells' parts, a shell | client | jig's browser parts over skein's `skein-ui-web`: the client's adapter over skein's fetch, the printed link's sign-in, jig's look (`browser/loop.md`, 1.1); and the native shell over skein's HTTP client, with `jig-client-protocol-http` (`client/shells.md`, section 1) |

**The store's codec crates.** `jig-protocol-store`'s families' generated
record code sits in codec crates below it, which depend on skein-lib and
skein-codec alone (skein's `data/codec/README.md`, section 2), so that
they compile beside the core's crates rather than after them:

| Codec crate | Families |
|---|---|
| `jig-protocol-store-people` | `people` |
| `jig-protocol-store-tasks` | `tasks` |
| `jig-protocol-store-core` | `core` |
| `jig-protocol-store-settings` | `settings` |
| `jig-protocol-store-families` | `accounts`, `component`, `notes`, `openings`, `registry`, `secrets` |

- **`people`, `tasks`, `core` and `settings`** each take their own crate,
  so that a change to one recompiles it alone. The six small families
  share one.
- **Each module has its versioned name,** `people_v17`, at its codec
  crate's root, and `jig-protocol-store` re-exports it at its own as
  `jig_protocol_store::people_v17`. A caller may name either path.
- **The store keeps the rest:** the current schemas, the registered
  families' declarations (`store/layouts.md`, 2), the families' bounds
  from the domain's limits (`store/README.md`, section 8) and the
  translations in `jig-protocol-store-records` (`src/generated/` and
  the hand-written `codec` module), with openings in the store itself
  (`src/generated/openings.rs`). The drift test, `jig/tests/codec-families.rs`, compares
  each family's generated records with the committed module (skein's
  `data/codec/golden.md`, section 1). Its inventory names the codec crates'
  generated records and the maps' generated translations, not golden
  bytes or generated tests.

Beside them, smith's components, by the agent capability:
`smith-protocol-llm`, over skein's `skein-llm-connection`, in a process
that runs `smith-inline-agent`, with `smith-protocol-machine` on a
worker (`llm.md`); `smith-host-protocol` on a worker with agent
processes (smith's `protocol/host.md`, sections 2 and 3).

- **The client itself is not a component of these:** `jig-client`, the
  client the application holds, with its domain, `jig-client-domain`,
  and the reference UI, `jig-client-view`, are `../client/domain/`'s.
  Every client crate is named under `jig-client`, wherever it runs
  (`../crate-names.md`, section 1).
- **Each crate is step code** under the subset (skein/docs/foundation/programming-model.md,
  section 10), as every crate of jig's is but its test kits and the
  client's shells and their parts. The reference UI's bundle is made at
  build by skein's bundler, `skein-ui-bundle`, a tool, not a component,
  and served by the application's server (skein's `ui/web/bundle.md`, 2;
  `browser/README.md`, 4.2, and `browser/serving.md`, 1).
- **Each depends** on skein-lib, io's vocabulary, the skein components it
  drives, its codec crate, and the domain vocabulary it translates: the
  hub's for the channel's sides, the core's (or a child's) for the
  engine's others. None depends on an application's crate, and none names
  a system. A host of `service-parts.md`, section 1, reads jig's configuration
  too.
- **Which crates link smith.** Of jig's, only `jig-protocol-charter`,
  `jig-protocol-worker-agents`, which an application whose workers run no
  agents leaves out, and the client's decoding of turns;
  `jig-protocol-tools` gives its schemas as bytes the charter declares.
  The application links the rest where it
  composes them: its root, the agent capability; its protocol layer,
  smith's components and smith's codecs.
