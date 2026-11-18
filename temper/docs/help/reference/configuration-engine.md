# Engine configuration

Kind: reference

The engine and worker configuration members, their bounds and what temper-box init --box writes.

Configuration is read at startup; changing a permitted member needs a restart.

Objects and lists of objects name their shapes below; init values show objects and list lengths, with scalar values written as JSON.

## engine

Shape: `engine`. Paths: `engine`. The engine's configuration, read before startup.

- `limits`: limits; required; init writes object. Selects a limits profile and its named overrides.
- `llm`: refused: moved: LLM providers are settings; see them with temper ctl settings show provider
- `hosts`: hosts; required; init writes object. Names the workers' listener addresses.
- `clients`: clients; optional; init writes object. Configures the optional OpenID provider.
- `deployment`: refused: moved: temper keeps the store in the data directory's state
- `owners`: list of owner, people.initial_owners; required; init writes list (1 entry). Names the deployment's first owners.
- `providers`: refused: moved: temper fixes its identity providers: 1 link, 2 deployment, 3 Google's
- `charters`: refused: settings live in the store
- `templates`: refused: settings live in the store
- `policies`: refused: settings live in the store
- `rules`: refused: settings live in the store
- `projects`: refused: moved: projects are made at runtime; make one from the client or temper ctl
- `connectors`: refused: moved: temper fixes its connector, the forge's, 1
- `acts`: refused: moved: temper fixes the forge connector's acts
- `memory_bytes`: integer, 0 to 18446744073709551615 bytes; required; init writes 1073741824. Gives the application's byte budget for the engine.

## engine.limits

Shape: `limits`. Paths: `engine.limits`. A limits profile and named overrides, shared by engines and workers.

- `profile`: text, small or standard; optional; init writes "standard". Reads the base when absent or standard, or selects small's limits.
- `overrides`: object, known limit names; optional; init writes {"hub.accounts":4,"hub.event_bytes":10660,"hub.run_calls":48,"authority.executors":22,"clients.exchanges":12,"clients.streams":8,"clients_http.calls":12,"clients_http.streams":8,"core.resume_bytes":16803814,"documents.rows":2,"documents.text_bytes":512,"fleet.configured_workers":8,"fleet.workers":2,"hub.transcript_bytes":67215250,"hub.turn_bytes":16803810,"hub.turn_queue_bytes":134430480,"io.output":16804602,"tasks.charters":16,"tasks.executor_kinds":22,"tasks.inbox_bytes":874120,"tasks.message_bytes":10660,"tasks.result_bytes":10660}. Overrides the profile's limits by their dotted names.

## engine.owners[]

Shape: `owner`. Paths: `engine.owners[]`. An identity seeded as one of the deployment's first owners.

- `provider`: integer, 0 to 65535; required; init writes 1. Names the identity's configured provider.
- `subject`: text, people.identity_bytes; at most 4096 bytes; required; init writes "person@localhost". Names the owner's subject under that provider.

## engine.hosts.listener[]

Shape: `address`. Paths: `engine.hosts.listener[]` `workers[].engine` `application.clients.listener` `application.webhooks.listener`. One socket address, host and port, or in-process name.

- `socket`: text, 128 bytes; exclusive with host and in_process; optional; init writes "127.0.0.1:8367" | "127.0.0.1:8371". Gives an IPv4 or IPv6 socket address.
- `host`: text, 253 bytes; exclusive with socket and in_process; optional; init leaves it out. Names a host the shell resolves at startup.
- `port`: integer, 0 to 65535; required with host; optional; init leaves it out. Gives the named host's port.
- `in_process`: text, 32 bytes; exclusive with socket and host; optional; init writes "workers". Names an in-process endpoint within the process.

## engine.hosts

Shape: `hosts`. Paths: `engine.hosts`. The workers' listener addresses.

- `workers`: refused: workers are settings in the store
- `listener`: list of address, loopback or in-process; required; init writes list (1 entry). Gives the workers' plaintext listener addresses.

## engine.clients

Shape: `clients`. Paths: `engine.clients`. The clients' OpenID provider and fixed members.

- `heartbeat_ms`: refused: moved: temper fixes the clients' heartbeat, printed link and approval
- `link`: refused: moved: temper fixes the clients' heartbeat, printed link and approval
- `approval`: refused: moved: temper fixes the clients' heartbeat, printed link and approval
- `provider`: client-provider or null; optional; init writes object. Configures the deployment's OpenID provider, with none when absent or null.

## engine.clients.provider

Shape: `client-provider`. Paths: `engine.clients.provider`. The deployment's OpenID provider.

- `key`: text, 4096 bytes; data path; required; init writes "provider.key". Names the private PKCS#8 PEM file holding the provider's 3072-bit RSA signing key.
- `clients`: list of provider-client, at least one; required; init writes []. Lists clients allowed to use the provider, with unique identifiers.
- `addresses`: list of integer, 0 to 65535; application or link providers; required; init writes [1]. Names identity providers whose subjects are verified addresses.

## engine.clients.provider.clients[]

Shape: `provider-client`. Paths: `engine.clients.provider.clients[]`. A client allowed to use the deployment provider.

- `id`: text, 1 to 255 visible ASCII bytes; required; init leaves it out. Gives the client's unique identifier.
- `secret`: text or null, 4096 bytes; data path; required; init leaves it out. Names the client's secret file, or null for a public client.
- `redirect_uri`: text, 2048 bytes; HTTPS or loopback HTTP with a port; required; init leaves it out. Gives the client's one redirect URI.

## workers[]

Shape: `worker`. Paths: `workers[]`. A worker's connection, execution and stop bounds.

- `engine`: address, loopback or in-process; required; init writes {"in_process":"workers"}. Gives the engine address the worker dials.
- `name`: text, link.name; at most 4096 bytes; required; init writes "local". Gives the worker's configured name.
- `secret`: text, 4096 bytes; data path; required; init writes "worker.secret". Names the private file holding the worker's secret.
- `slots`: integer, hub.slots; required; init writes 4. Bounds how many runs the worker hosts at once.
- `graces`: refused: moved: temper fixes a worker's graces (temper help server)
- `programs`: refused: moved: temper fixes the login checks' interval
- `agent`: refused: moved: temper writes the agent's configuration from its engine's figures; remove it
- `limits`: refused: moved: a worker takes its limits from its engine; remove it
- `memory_bytes`: integer, 0 to 18446744073709551615 bytes; optional; init leaves it out. States the worker's available memory when configured.
