# Engine configuration

Kind: reference

The engine and worker configuration members, their bounds and what temper-box init --box writes.

Configuration is read at startup; changing a permitted member needs a restart.

Objects and lists of objects name their shapes below; init values show objects and list lengths, with scalar values written as JSON.

## engine

Shape: `engine`. Paths: `engine`. The engine's configuration, read before startup.

- `limits`: limits; optional; init writes object. Selects a limits profile and named overrides; absent, reads the application's base.
- `llm`: llm; required; init writes object. Configures the LLM endpoints, models and providers.
- `hosts`: hosts; required; init writes object. Names the workers and their listener addresses.
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

Shape: `limits`. Paths: `engine.limits` `workers[].limits`. A limits profile and named overrides, shared by engines and workers.

- `profile`: text, small or standard; optional; init writes "standard". Reads the base when absent or standard, or selects small's limits.
- `overrides`: object, known limit names; optional; init writes {"authority.executors":22,"clients.exchanges":12,"clients.streams":8,"clients_http.calls":12,"clients_http.streams":8,"core.resume_bytes":16803814,"documents.rows":2,"documents.text_bytes":512,"fleet.configured_workers":8,"fleet.workers":2,"hub.transcript_bytes":67215250,"hub.turn_bytes":16803810,"hub.turn_queue_bytes":134430480,"io.output":16804602,"tasks.charters":16,"tasks.executor_kinds":22,"tasks.inbox_bytes":874120,"tasks.message_bytes":10660,"tasks.result_bytes":10660} | {"hub.accounts":4,"hub.event_bytes":10660,"hub.run_calls":48,"hub.transcript_bytes":67215250,"hub.turn_bytes":16803810,"hub.turn_queue_bytes":134430480,"io.output":16804602}. Overrides the profile's limits by their dotted names.

## engine.owners[]

Shape: `owner`. Paths: `engine.owners[]`. An identity seeded as one of the deployment's first owners.

- `provider`: integer, 0 to 65535; required; init writes 1. Names the identity's configured provider.
- `subject`: text, people.identity_bytes; at most 4096 bytes; required; init writes "person@localhost". Names the owner's subject under that provider.

## engine.llm

Shape: `llm`. Paths: `engine.llm`. The endpoints and providers, with document models when defaults are not passed.

- `endpoints`: list of endpoint; required; init writes list (4 entries). Declares the endpoints serving the configured models.
- `providers`: list of llm-provider; optional; init writes list (3 entries). Assigns each configured model to exactly one LLM provider.

## engine.llm.endpoints[]

Shape: `endpoint`. Paths: `engine.llm.endpoints[]`. A model endpoint and its transport.

- `name`: text, 4096 bytes; required; init writes "openai" | "anthropic" | "anthropic-api" | "demo". Names the endpoint for configured models to refer to.
- `number`: integer, 0 to 4294967295; required; init writes 1 | 2 | 4 | 3. Gives the endpoint's number, unique among endpoints.
- `dialect`: integer, 2 or 3; required; init writes 3 | 2. Chooses the endpoint's model protocol dialect.
- `provider`: text, 4096 bytes; required; init writes "openai" | "anthropic" | "demo". Names the endpoint's provider for the model protocol.
- `program`: text, 32 bytes; optional; init writes "claude_code". Names a program offered by every provider whose models use this endpoint.
- `address`: address, network address; required; init writes {"host":"api.openai.com","port":443} | {"host":"api.anthropic.com","port":443} | {"socket":"127.0.0.1:8370"}. Gives the endpoint's socket address or host and port.
- `transport`: text, tls or plaintext; required; init writes "tls" | "plaintext". Selects TLS or plaintext, with plaintext restricted to loopback.
- `server_name`: text, 4096 bytes; required for tls; optional; init writes "api.openai.com" | "api.anthropic.com". Names the server authenticated by TLS.
- `trust`: text, 4096 bytes; configuration path; tls only; optional; init leaves it out. Names trusted roots for TLS, with the system roots used when absent.
- `headers`: list of header; required; init writes []. Adds the configured headers to endpoint requests.
- `connect_ms`: integer, 0 to 18446744073709 milliseconds; required; init writes 10000. Bounds the wait to connect to the endpoint.
- `handshake_ms`: integer, 0 to 18446744073709 milliseconds; required; init writes 10000. Bounds the endpoint's TLS handshake.

## engine.llm.endpoints[].address

Shape: `address`. Paths: `engine.llm.endpoints[].address` `engine.hosts.listener[]` `workers[].engine` `application.clients.listener` `application.webhooks.listener` `application.demo.llm.listen`. One socket address, host and port, or in-process name.

- `socket`: text, 128 bytes; exclusive with host and in_process; optional; init writes "127.0.0.1:8370" | "127.0.0.1:8367" | "127.0.0.1:8371". Gives an IPv4 or IPv6 socket address.
- `host`: text, 253 bytes; exclusive with socket and in_process; optional; init writes "api.openai.com" | "api.anthropic.com". Names a host the shell resolves at startup.
- `port`: integer, 0 to 65535; required with host; optional; init writes 443. Gives the named host's port.
- `in_process`: text, 32 bytes; exclusive with socket and host; optional; init writes "workers". Names an in-process endpoint within the process.

## engine.llm.endpoints[].headers[]

Shape: `header`. Paths: `engine.llm.endpoints[].headers[]`. One header added to an endpoint request.

- `name`: text, 4096 bytes; required; init leaves it out. Names the request header.
- `value`: text, 4096 bytes; required; init leaves it out. Gives the request header's value.

## engine.llm.providers[]

Shape: `llm-provider`. Paths: `engine.llm.providers[]`. An LLM provider, its added models' endpoint, the document models it serves and the sources it offers.

- `name`: text, 1 to 256 bytes; public text; required; init writes "openai" | "anthropic" | "demo". Gives the provider's name for showing.
- `number`: integer, 0 to 4294967295; required; init writes 1 | 2 | 3. Gives the provider's number, unique among LLM providers.
- `endpoint`: text, 4096 bytes; a configured endpoint name; required; init writes "openai" | "anthropic" | "demo". Names the endpoint its added models take.
- `direct`: text, 4096 bytes; program-free endpoint of the same dialect; requires program endpoint and sign-in or key; optional; init writes "anthropic-api". Names the direct endpoint its sign-ins, keys and list reads take.
- `keep`: list of text, 16 patterns; 1 to 64 model-name bytes or * and ?; requires endpoint; optional; init writes ["gpt-*"] | ["claude-*"]. Keeps list names matching any whole-name pattern; absent disables reading the list.
- `drop`: list of text, 16 patterns; 1 to 64 model-name bytes or * and ?; requires keep; optional; init writes ["*audio*","*image*","*realtime*","*search*","*transcribe*","*tts*","*-20??-??-??"] | ["*-20??????"]. Excludes list names matching any whole-name pattern, even when kept.
- `sources`: sources; required; init writes object. Declares the sign-in, key and program sources the provider offers.
- `signed_in_plan`: true or false, required with sign_in; optional; init writes true | false. Says whether the provider's signed-in source is a plan.
- `manage_usage`: text, 2048 ASCII bytes; public HTTPS address; required for plans and programs; optional; init writes "https://chatgpt.com/settings/usage" | "https://claude.ai/settings/usage". Gives the public address where a holder manages usage.

## engine.llm.providers[].sources

Shape: `sources`. Paths: `engine.llm.providers[].sources`. The sources an LLM provider offers, each optional.

- `sign_in`: registration; optional; init writes object. Offers a sign-in with its public registration.
- `keys`: true or false; optional; init writes true. Offers an API key source when true.
- `programs`: list of text, 32 bytes per name; optional; init writes ["claude_code"]. Names offered program sources, such as claude_code.

## engine.llm.providers[].sources.sign_in

Shape: `registration`. Paths: `engine.llm.providers[].sources.sign_in`. A sign-in's public registration, with no client secret.

- `issuer`: text, 1 to 4096 bytes; required; init writes "https://auth.openai.com". Names the issuer expected in the sign-in's tokens.
- `client`: text, 1 to 4096 bytes; required; init writes "dynamic_agent_client". Gives the entry client identifier of the public registration.
- `scopes`: list of text, nonempty scope tokens; required; init writes ["openid","profile","email","offline_access","resource.invoke","chatgpt.tokens.use.direct"]. Lists the scopes requested by the sign-in.
- `name_hint`: text, 4096 bytes; optional; init writes "temper". Gives the registration's optional name hint.
- `authorization`: text, 1 to 4096 bytes; required; init writes "https://auth.openai.com/api/accounts/authorize". Gives the authorization endpoint's address.
- `token`: text, 1 to 4096 bytes; required; init writes "https://auth.openai.com/api/accounts/oauth/token". Gives the token endpoint's address.
- `revocation`: text, 4096 bytes; optional; init writes "https://auth.openai.com/api/accounts/oauth/revoke". Gives the revocation endpoint's address when available.
- `keys`: text, 1 to 4096 bytes; required; init writes "https://auth.openai.com/.well-known/jwks.json". Gives the address of the issuer's published keys.
- `transport`: text, tls or plaintext; required; init writes "tls". Chooses the transport used for registration endpoints.
- `server_name`: text, 4096 bytes; required for tls; optional; init writes "auth.openai.com". Names the server authenticated by the endpoints' TLS.
- `trust`: text, 4096 bytes; configuration path; tls only; optional; init leaves it out. Names trusted roots for registration endpoints.
- `must_grant`: list of text, a subset of scopes; required; init writes ["chatgpt.tokens.use.direct"]. Lists scopes that must be granted for a successful sign-in.
- `resource`: text, 4096 bytes; optional; init writes "https://api.openai.com/v1". Names the registration's resource when required.
- `redirect_path`: text, 4096 bytes; no leading slash, query or fragment; required; init writes "auth/callback". Names the redirect's path within the loopback listener.
- `redirect_port`: integer, 1 to 65535; optional; init writes 1455. Fixes the loopback redirect port, with an ephemeral port used when absent.
- `secret`: refused; optional; init leaves it out. Refuses a client secret because the registration must be public.

## engine.hosts

Shape: `hosts`. Paths: `engine.hosts`. Workers the engine accepts and their listener addresses.

- `workers`: list of worker-name, link.names and fleet.configured_workers; required; init writes list (1 entry). Names configured workers and their secret digests.
- `listener`: list of address, loopback or in-process; required; init writes list (1 entry). Gives the workers' plaintext listener addresses.

## engine.hosts.workers[]

Shape: `worker-name`. Paths: `engine.hosts.workers[]`. One worker accepted by the engine.

- `name`: text, link.name and fleet.worker_name_bytes; at most 4096 bytes; required; init writes "worker". Gives the worker's unique name.
- `digest`: text, 4096 bytes; data path; required; init writes "worker.digest". Names the file holding the worker secret's digest.

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
- `name`: text, link.name; at most 4096 bytes; required; init writes "worker". Gives the worker's configured name.
- `secret`: text, 4096 bytes; data path; required; init writes "worker.secret". Names the private file holding the worker's secret.
- `slots`: integer, hub.slots; required; init writes 4. Bounds how many runs the worker hosts at once.
- `graces`: refused: moved: temper fixes a worker's graces (temper help server)
- `programs`: refused: moved: temper fixes the login checks' interval
- `agent`: agent; required; init writes object. Names the agent program, its arguments and its configuration document.
- `limits`: limits; optional; init writes object. Selects the worker's limits profile and overrides; absent, reads the application's base.
- `memory_bytes`: integer, 0 to 18446744073709551615 bytes; optional; init leaves it out. States the worker's available memory when configured.

## workers[].agent

Shape: `agent`. Paths: `workers[].agent`. The program a worker spawns for each run.

- `program`: text, 4096 bytes; required; init writes "/opt/temper/bin/temper-box". Names the agent executable, passed as written.
- `arguments`: list of text, 4096 bytes per argument; required; init writes ["internal","agent","/etc/temper/agent.json"]. Gives the agent's arguments, passed as written.
- `configuration`: text, 4096 bytes; configuration path; required; init writes "/etc/temper/agent.json". Names smith's agent configuration document, which jig does not read.
