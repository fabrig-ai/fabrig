# Components and crates

**jig's,** composed as they are (jig's `protocol/README.md`, section 5):

| Crate | In | What temper uses it for |
|---|---|---|
| `jig-protocol-config` | engine, worker | jig's parts of the configuration (configuration/README.md, section 2) |
| `jig-protocol-store` | engine | the store, temper's families beside jig's in the same commits (store.md) |
| `jig-protocol-worker-wire` | engine, worker | the channel's records; temper's parts ride in them as bytes (channel.md) |
| `jig-protocol-engine-workers` | engine | the worker's link, the engine's side: opening, the payload version, grants' values filled in |
| `jig-protocol-worker-link` | worker | the worker's link, the worker's side: dialling, grants' values taken out |
| `jig-protocol-worker-agents` | worker | the program sources and the worker route, between smith's host half and the link (jig's `protocol/service.md`, 11.2) |
| `jig-protocol-accounts` | engine | LLM accounts on skein's accounts driver: signed-in records and keys from jig's secrets' store, a loopback listener for each sign-in's redirect, the access tokens and keys lent (credentials.md, section 3) |
| `jig-client-protocol-wire` | engine, client | jig's records for clients; the forge's client family's records ride in them as bytes (`../client/README.md`) |
| `jig-protocol-engine-clients` | engine | the clients' component: jig's messages with their credentials, the printed link's secret, sign-ins, credentials (`../client/engine.md`) |
| `jig-protocol-engine-clients-http` | engine | jig's default HTTP mapping, its engine end, with its example guards, mounted on temper's clients' listener under `/jig`, and the server part that listener stands on (jig's `protocol/service.md`, 11.4) |
| `jig-client` | client | the client temper's client root holds: the link, watches by handle, the request table, kept work, families; jig's helpers (`../client/README.md`, section 3) |
| `jig-client-protocol-link` | client | `jig-client`'s half toward its transport: jig's messages and the small store's records as bytes (jig's `protocol/client/wire.md`, 2.3) |
| `jig-client-protocol-http` | client | jig's default HTTP mapping, its client end, under `jig-client-shell-native` and `jig-client-shell-web` |
| `jig-client-shell-web` | client | the browser shell's adapter over skein's fetch, and the printed link's sign-in part (`../client/web/README.md`, 2.1) |
| `jig-protocol-transcript` | client | turns' bytes, decoded below `jig-client` |
| `jig-client-shell-native` | client | `temper-box open`'s and `temper ctl`'s transport: HTTP on io, the credential `jig-client` holds sent as a bearer, the small store's file in its directory |

temper does not use `jig-protocol-charter` or `jig-protocol-tools`: its
root takes its own `Tool` vocabulary, translated by `temper-engine-smith`
(llm.md, sections 4 and 5). Nor does it use jig's reference UI
(`jig-client-view`, `jig-client-test-shell-web`, nor the look of
`jig-client-shell-web`): temper's pages are its own,
on skein's UI kit (`../client/view.md`).

**smith's:** `smith-host-domain`, the worker root's agent child;
`smith-host-protocol`, smith's host half, one channel per agent process
over its pipes (smith's `protocol/host.md`, sections 2 and 3);
`smith-protocol-channel`, `smith-channel`, `smith-charter` and
`smith-transcript`, smith's codecs, which the routing calls to decode and
encode smith's payloads; `smith-agent-shell`, the agent's modes in
`temper-box` and in `temper-agent` (`binaries.md`, 3); `smith-agent-document`, the agent's
document and programs' directories read without smith's agent, for the
launcher's links (setup/owned.md, 6) and for the worker's half of a start,
which reads the agent's host limits from its profile at start, as jig's
reference binary does.

**skein's:** `skein-lib` and `skein-io` everywhere, lib's HMAC-SHA256
checking the deliveries' signatures (skein's `runtime/lib.md`, 6.2), io's
processes under the box; `skein-shell`, the binary's startup kit
(`read_file`, `read_private_file`, seeds, `Host`, `drive`);
`skein-json`, configuration and the forges' documents; `skein-http`, the
forge component's client and its webhooks' listener's server, the setup
host's client, and the fake forge's server; the forge kit,
`skein-forge` (Forgejo's and GitHub's documents and call plans), `skein-forge-client`,
`skein-forge-connection` (the forge component) and `skein-forge-setup`
(the setup host and its plans), with the fake forge and the throwaway Forgejo in its worlds
(skein's `forge/README.md`, section 2);
`skein-tls`, a forge's TLS; `skein-codec` and its generator, every codec
crate; `skein-kv`, under jig's store and temper's key encoders;
`skein-channel`, under jig's channel; `skein-oauth-accounts`, under
jig's accounts; `skein-ui`, under temper's view, and
`skein-ui-text`, the terminal's text shell. In the worlds: skein's
simulator, fake machine, fake LLM peer, fake issuer and fake person.

**temper's own,** each step code under the subset but the binaries,
`temper-shell`, the terminal's host, `temper ctl`'s host,
`temper-forge-setup-shell`, `temper-fetch-shell`, `temper-launcher`, the box's host and the
fake git program's core:

| Crate | Kind | Holds | Depends on |
|---|---|---|---|
| `crates/temper-service-core` | services | what the engine's and the worker's services share and that names neither half (`services.md`, 3): the process's io, kernel queues and signals; `tokens`, `long_pass`, the stages and `Refusal`; the shared limits (the checkout's and the git component's envelopes, the tooling's bounds, a grant's sizing, the peer envelope); `Files`, `Program`, `Adapter`; the agent's declaration; the document's shape, `Shape` and `nearest`; the startup problems' container and lines | `skein-io`, `skein-lib`, `skein-json`, `jig-protocol-config`, the codec and wire crates both halves speak; none of the engine's or the worker's crates (`services.md`, 2) |
| `crates/temper-engine-service` | services | the engine's half, as jig's reference service's engine module is: its component (the root, its protocol children and the routing) and the engine alone, `Service`, each `iterate`, the clients' listener on jig's server part, with jig's clients' component and its default HTTP mapping mounted, temper's forge routes, and the browser's page and bundle served (`../client/web/README.md`, 2.2), an engine's start from configuration's bytes, its end, its notes and stops; temper's profile `small`; the engine's limits of the worker's checkout and git component are the core's shared envelopes (`services.md`, 4) | `temper-service-core`, the roots, `temper-engine-smith`, `temper-protocol-brief`, jig's engine components, smith's codecs and `smith-protocol-translate`, temper's codec crates and components, `temper-protocol-records-forge`, `skein-forge-connection`, `skein-io`, `skein-json`; no worker crate |
| `crates/temper-worker-service` | services | the worker's half: its component (the worker's root, its link, program sources, git component and jobs) and the worker alone, `Service`, its start from a worker alone's part, its end, its notes and stops; the worker's limits on the core's shared parts and the peer envelope; `Agent`, smith's host profile, which gives the agent's declaration (`services.md`, 5) | `temper-service-core`, the worker's root, checkout and git component, `jig-protocol-worker-link`, `jig-protocol-worker-agents`, smith's host half, `temper-protocol-brief`; no engine crate |
| `crates/temper-service` | services | the one process, as jig's reference service's system module is: the engine's and the worker's components on one process, joined by the in-process name `workers`, each turn in today's order, the standalone start with its paired checks, the dispatch of every shape's start, and the supervision that starts, drains and stops them together; its `End`, `Stop` and `Note` join the halves' (`services.md`, 6; `README.md`, section 6). For the box, the playground and the worlds | `temper-service-core`, `temper-engine-service`, `temper-worker-service` |
| `bin/temper-box` | binary | the box's binary, `temper-box`: its `main` alone, calling `temper-box-shell`'s with `temper-version`'s identity (`../cli/README.md`, 2) | `temper-box-shell`, `skein-fail-stop`, `temper-version` |
| `crates/temper-box-shell` | binary's crate | the box's table and its reach (`../cli/README.md`, 6), and the commands of `binaries.md`, copied from jig's reference binary; for each that reads the document, `temper-service`'s startup, with `machine` and the setup's and the box's limits checked by their own crates (configuration/README.md, section 2); the forge's commands are `temper-engine-shell`'s; `serve`'s standalone start is this crate's, composed from `temper-engine-shell`'s engine parts and `temper-worker-shell`'s worker parts, with its service, `temper-service`'s, and its lines (`services.md`, 7); `program-login` is `temper-worker-shell`'s | `temper-service`, `temper-engine-shell`, `temper-service-shell`, `temper-worker-shell`, `temper-agent-shell` (the tree's tables, `../cli/README.md`, 9), `temper-shell`, `skein-shell`, `smith-agent-shell`, `skein-oauth-accounts`, `jig-client-shell-native`, the client crates, `temper-forge-setup-shell`, `temper-launcher`, `temper-box-machine`, `temper-client-bundle` |
| `bin/temper` | binary | the deployment's client's binary, `temper`: its `main` alone, calling `temper-client-shell`'s with `temper-version`'s line (`../cli/README.md`, 2) | `temper-client-shell`, `skein-fail-stop`, `temper-version` |
| `crates/temper-client-shell` | binary's crate | `temper`, the deployment's client (`binaries.md`): its table (../cli/README.md, 7.4), capture, reach, dispatch and help, and `--version`, the line its binary hands it; the TUI, `temper box` and `ctl` it runs are its shells' (`../client/README.md`, section 2) | `temper-shell`, `temper-client-shell-tui`, `temper-client-shell-cli`, `temper-fetch`, `temper-fetch-shell`, `skein-shell`, `skein-io`, `skein-lib` |
| `bin/temper-agent` | binary | the agent's binary, `temper-agent`: its `main` alone, calling `temper-agent-shell`'s with `temper-version`'s identity (`../cli/README.md`, 2; `binaries.md`, 3) | `temper-agent-shell`, `skein-fail-stop`, `temper-version` |
| `crates/temper-agent-shell` | binary's crate | `temper-agent`'s table, `AGENT`, its capture, reach and `main` (`../cli/README.md`, 7.3), its help and `--version`, with the identity its binary hands it; the run, the list and the program's modes it runs are smith's agent shell's, and the launcher `temper-launcher`'s | `temper-shell`, `smith-agent-shell`, `temper-launcher`, `skein-shell`, `skein-io` (a run's exit) |
| `crates/temper-forge-setup` | step code | the forge's setup, but for reconcile, which is skein's (`skein-forge-setup`): its types and checks, and the setup mapped onto reconcile's given values, with the kit's check of them; the preparation's machine; `app.ini` and the runner's files; the processes' specs; the configuration's steps, `steps` and `Step`. Names no host | setup/README.md, section 7 |
| `crates/temper-forge-setup-shell` | hosts | ordinary Rust: the hosts of `temper-box forge prepare` and `run`: commands, files, the forge's locks and the exec (setup/owned.md, 4); HTTP on skein-http over skein-io | setup/README.md, section 7 |
| `crates/temper-fetch` | step code | a pinned binary's fetch: its machine, its pins' fields and their origin, Forgejo's and Claude Code's alike; Claude Code's pin and the word `fetched` (setup/owned.md, 6). Names no host | setup/README.md, section 7 |
| `crates/temper-fetch-shell` | host | ordinary Rust: the fetch's host over TLS on skein-io; Claude Code's fetch under its lock (setup/owned.md, 6); no smith crate | setup/README.md, section 7 |
| `crates/temper-launcher` | host | ordinary Rust: Claude Code's launcher, the placing of its links in a private directory, and the substitution of `"fetched"` in an agent's document (setup/owned.md, 6), on `temper-fetch-shell`'s fetch | setup/README.md, section 7 |
| `crates/temper-box-machine` | machine, host | the box's machine, step code, and its host, ordinary Rust, on skein-io's processes; nothing of the setup but its steps (setup/box.md, 3) | setup/README.md, section 7 |
| `crates/temper-fake-git` | program | the fake git program's core, ordinary Rust, which the binary runs as `temper-box fake git`: its checks, answers, step core, replay and two remotes (`../testing/fakes.md`, 3) | setup/README.md, section 7 |
| `crates/temper-fake` | step code | the fake backends' documents and seeds, for `temper-box fake forge`, `temper-box fake llm` and the box's fake backends (setup/box.md, 5 and setup/playground.md; configuration/directories.md, 1); no io | skein's fake forge and fake LLM, `skein-forge-setup`, `temper-forge-setup`, `temper-protocol-store`, `temper-engine-domain-forge-vocabulary` |
| `crates/temper-init` | step code | the files `temper-box init` writes, the default document among them (setup/init.md, 2 and 3) | `skein-lib`, `skein-json`, `skein-forge-setup` |
| `crates/temper-worker-protocol-git` | component | the git component: each `git::Op` of `temper-worker-domain-checkout` as a fixed sequence of git invocations, their arguments, environment, output and failures; the operations in flight, their processes, pipes and deadlines; `Make` as file operations (git.md) | `skein-lib`, `skein-io`, the checkout's vocabulary |
| `crates/temper-protocol-worker-wire` | codecs | temper's parts on jig's channel, and a git identity's value, from a schema; `PAYLOAD_VERSION` (channel.md) | `skein-lib`, `skein-codec` |
| `crates/temper-protocol-store` | codecs, keys | temper's families: key encoders, the schema, its kept texts, patches and golden bytes, the translations to and from `temper-protocol-store-codec`'s records, `STORE_VERSION`, the restart's interval (store.md) | `skein-lib`, `skein-codec`, `skein-kv`, `jig-protocol-store`, `temper-protocol-store-codec`, the forge's records |
| `crates/temper-protocol-store-codec` | codecs | the store's generated record code, its current version alone (store.md, section 4) | `skein-lib`, `skein-codec` |
| `crates/temper-protocol-records` | codecs | the forges' records in jig's secrets' store and a pasted bundle, from `schema/secrets-v1.schema`, the family `temper-secrets` (`README.md`, section 7; credentials.md, 2.1): the schema's codecs, the records' numbers and the pasted bundle's, of no side, so that the client encodes a pasted bundle with the engine's codec | `skein-lib`, `skein-codec` |
| `crates/temper-protocol-records-forge` | protocol | the forges' records read at start, on `temper-protocol-records`' codecs: each forge's rows (`Rows`, `ForgeRows`), its origin and its check (`Origin`, `origin`), the access they give (`access`, `Start`) and why a forge is not connected (`NotConnected`) (credentials.md, 2.1); the engine's service's and the box's, never the client's (`../draft/cli.md`, section 6) | `temper-protocol-records`, the forge kit's vocabularies, `jig-protocol-config` |
| `crates/temper-client-protocol-wire` | codecs | the forge's client family's records: the adoption request and its outcome, the small store's record of a pending one; `FORGE`, the forge connector's number, 1; `Owner`, temper's owners; `CLIENT_VERSION` (`../client/forge/records.md`) | `skein-lib`, `skein-codec` |
| `crates/temper-engine-smith` | translation | smith's start from an assignment, host calls decoded into the root's `Tool` vocabulary, answers rendered, results and turns read; called by the engine's routing, keeping no state (llm.md); the brief fitted by `temper-protocol-brief` | the root's and its forge children's vocabularies, jig's and smith's domain vocabularies, `temper-protocol-brief`, `skein-lib`, `skein-json` |
| `crates/temper-client-domain` | client root | `jig-client` held, temper's pages, the forge's client side (`../client/README.md`, section 3) | `../client/README.md`, section 2 |
| `crates/temper-client-view` | view | temper's pages as skein-ui's tree, with the forge's view (`../client/view.md`) | `../client/README.md`, section 2 |
| `crates/temper-client-shell-terminal` | shell | the terminal: its host ordinary Rust, over skein's text shell and jig's native transport, and how an argument names an engine's address or its link (`../client/line.md`) | `../client/README.md`, section 2 |
| `crates/temper-client-shell-web` | shell | the browser: a library for wasm32 and every target, ordinary Rust, over skein's browser parts and jig's browser adapter, its start reading the page's internal settings (`../client/web/README.md`, 2.1) | `../client/README.md`, section 2 |
| `crates/temper-client-web-page` | shell | the shipped page's entry, a `cdylib` for wasm32 whose start calls the shell's, from which the bundle is built (`../client/web/README.md`, 2.1 and 2.6) | `temper-client-shell-web`, `wasm-bindgen` |
| `crates/temper-client-redaction` | journeys | the journeys' shared originals, their redaction and the last scan's shapes, which the browser's and the TUI's recorders share, of no shell (`../client/web/journeys.md`, 3.6 and 3.7; `../client/tui/journeys.md`, 2) | `skein-lib` |
| `crates/temper-client-bundle` | data | the browser's bundle in the binary: its build script builds the shipped page, `temper-client-web-page`, for wasm32 and skein's bundler in a nested cargo, runs the bundler and lists the bundle's files; its library includes them, or nothing under `TEMPER_INTERNAL_BUNDLE=none` (`../client/web/README.md`, 2.6) | `skein-shell` for its build variable's declaration; its script, `std` alone |
| `crates/temper-client-tools` | client root, rendering | the agents' tools: the table, the runner, marks and conditions, the rendering, the tools' root (`../client/ctl/README.md`, section 4) | `../client/ctl/README.md`, section 4 |
| `crates/temper-client-shell-cli` | shell | `temper ctl`: its host ordinary Rust, one call per process over jig's native transport, and its start, which `temper ctl` and `temper-box ctl` run (`../client/ctl/README.md`, section 6) | `../client/ctl/README.md`, section 4 |
| `crates/temper-settings-form` | codecs | the settings file's form and words and `FIELDS`, shared by the client and the engine (`../client/ctl/tools.md`, 3; documentation.md, section 8) | `skein-lib`, `skein-json`, `skein-codec`, `jig-client-protocol-wire` |
| `crates/temper-docs` | data | the help pages compiled in, with their paging and search, of no side (documentation.md, sections 2 and 3) | `skein-lib` |
| `bin/temper-engine` | binary | the engine's binary: its `main` alone, calling `temper-engine-shell`'s with `temper-version`'s identity, whose table and reach are `../cli/README.md`'s, 7 (`binaries.md`, 2) | `temper-engine-shell`, `skein-fail-stop`, `temper-version` |
| `bin/temper-worker` | binary | the worker's binary, its `main` alone, calling `temper-worker-shell`'s with `temper-version`'s identity (`binaries.md`, 5) | `temper-worker-shell`, `skein-fail-stop`, `temper-version` |
| `crates/temper-engine-shell` | binary's crate, hosts | ordinary Rust: `temper-engine`'s table, `ENGINE`, its reach and its `main` (`../cli/README.md`, 7), with `versions`; an engine alone's start, which `temper-service-shell`'s run runs with the engine's service and its lines, and the engine's parts of a start as functions, which the box's standalone start composes (`services.md`, 7); the forge's commands, beside `temper-forge-setup-shell`'s hosts; the bundle read or embedded; the GitHub, Google and accounts runtimes; the rows both tables hold (`binaries.md`, 1); `setup`, the document's check that `serve` and the forge's commands make, the box's limits and the `demo` member included; `printed`, the words of a command for this machine (`../client/forge/records.md`) | `temper-engine-service`, `temper-service-shell`, `temper-shell`, `jig-protocol-config`, `temper-forge-setup`, `temper-forge-setup-shell`, `temper-box-machine` (the box's limits), `temper-fake` (the `demo` member), `temper-client-bundle`, `skein-shell` |
| `crates/temper-service-shell` | hosts | ordinary Rust, of no side: serve's run for every binary that serves, given its start and its service, an engine alone's, a worker alone's or the one process: the inherited listeners, the termination signals, the kernel and `drive`; its host, `Host`, driving the service through what the binary's shell gives it, which formats the lines and endings the host prints (`cluster.md`, 3; `services.md`, 7); the engine's link file; `read_files`, the files a document names; `jig_directories`, jig's form of `temper-shell`'s three directories | `temper-service-core`, `temper-shell`, `jig-protocol-config`, `skein-shell`, `skein-io`, `skein-lib`; no half's crate |
| `crates/temper-worker-shell` | hosts | ordinary Rust: the worker's half of a start, a worker alone's whole with its service's lines, and the root, program sources' resources and launcher's links a standalone start's worker takes, given to the box's start (`services.md`, 7); the agent's document read and preflighted on the worker's naming (`agent`); `program-login` and its command, `PROGRAM_LOGIN`, which the box's table holds too; `temper-worker`'s table, `WORKER`, its capture, reach and `main` (`../cli/README.md`, 7.2; `binaries.md`, 5) | `temper-worker-service`, `temper-service-shell`, `temper-shell`, `temper-launcher`, `smith-agent-document`, smith's host half, `jig-protocol-config`, `skein-shell`, `skein-io` |
| `crates/temper-shell` | shell | ordinary Rust, of no side: the commands' table, `Row` and its renderings under a binary's name (`binaries.md`, 1), and the shared `help`, `[ADDRESS]` and `ctl` rows; the directories and their settings; `help` over `temper-docs`; the engine's link file, written by serve and read by the TUI (`../client/tui/README.md`, 2); the variables several tables declare, `TEMPER_INTERNAL_COMMIT`'s among them (`../cli/README.md`, 8) | `skein-shell`, `skein-io` (a kernel error's number, in a file's refusal), `temper-docs`, `skein-lib`; no jig crate |
| `crates/temper-version` | data | `VERSION` and `COMMIT` (`../release.md`, section 3): its build script reads git, its library holds both and `line`; linked by the binaries' packages alone, each handing them to its crate (`../cli/README.md`, 2) | nothing; its script, `std` alone |
| `crates/temper-protocol-brief` | translation | a charter's brief fitted to the room its run leaves, cut from its last section with each cut's line (`../domain/agent/tools.md`), for the engine's translation and the worker's service alike | `smith-domain-run`, `skein-lib` |

**temper's test crates, worlds and tiers** are `../testing/README.md`'s
(`../testing/README.md`, section 2 and `../testing/fakes.md` and `../testing/README.md`, section 5): the fake git program, the worlds below the
domain, the process and live tiers, and the tool that records git's
transcripts (git.md, section 9). The fake forge's protocol layer, the
throwaway Forgejo and the tool that records Forgejo's transcripts are
skein's (`testing/fakes/forge.md`, sections 4, 6 and 7).

**What temper's components share** is what every component of jig's
does (jig's `protocol/README.md`, section 1; skein/docs/foundation/programming-model.md,
sections 4 and 9): each a child of the protocol layer with its own
limits, deadlines and kernel submission edges; its
connections or invocations its own entities, bound to calls by echoed
tokens; one exchange in flight per connection, with an output cap and a
room check before the next is parsed; a call with no room refused before
anything is set aside for it; each call's deadline the domain's, and
each connection's or invocation's its own, armed from its limits. Two
rules every temper component keeps:

- **No retries below the domain.** A call goes out once and ends once,
  with the failure the domain's vocabulary defines: for the forge,
  unavailable only when the call provably never left, and timed out
  otherwise (skein's `forge/forgejo.md`, 1.4). A retry below the domain hides its cost from
  the budget, and adds load just when the peer is struggling.
- **Limits at the entrances.** The worker's `temper-worker-domain::valid`
  checks entrance constraints and count representability at start
  (configuration/limits.md). No heap sum admits a component, and
  nothing is summed with jig's for admission.
