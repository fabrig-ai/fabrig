# The protocol layer

Provisional, 2026-10-03, revised 2026-11-02. temper's protocol layer:
how temper composes jig's protocol layer with components of its own (its
forge's, its checkout's, its client's) into the processes of one binary.
jig owns the layer's shape (jig's `protocol/README.md`) and the services'
(jig's `protocol/service.md`); this document says what temper adds and
the crates it is made of. The mechanics are those of skein's
`programming-model.md` (sections 2, 4 and 9); the entities it carries are
those of `../domain/`. Each boundary of temper's own is designed in a
document of its own (section 2). What is open across them is listed in
section 11. The cluster, temper as several processes of one machine for
its tests, is `cluster.md`.

## 1. In one page

- **Translation, and nothing else.** The protocol layer turns bytes into
  the domains' entities and back, and decides nothing about the domain
  (programming-model.md, section 4). The domain decides deadlines,
  retries and what a failure means; the protocol layer runs the call, its
  timer and its one attempt.
- **jig's protocol layer, composed.** jig ships the components for its
  own vocabularies: the store, the workers' channel, LLM accounts,
  clients, configuration (jig's `protocol/components.md`, section 1).
  temper's services own them as children beside smith's, skein's and
  temper's own components, and route between them, io and the roots, as
  jig's reference service does (jig's `protocol/service.md`, section 2).
- **What temper adds:** the forge's protocol (skein's forge component,
  with its webhooks' listener, handed temper's values and routed to the
  root and the forge connector); the checkout's io
  (the git component); its parts' codecs on jig's channel; its families
  in jig's store; its git identities; its host tools' translation; its
  client (its wire, its client root and view on `jig-client`, its
  terminal); the clients' listener, on which it mounts jig's clients'
  component and serves its forge routes; the setup host, skein's, in the
  engine (forge/engine.md, 6); and the forge's setup's commands outside the
  services (setup/README.md). Each is a crate of its own (`components.md`).
- **Foreign protocols are stacks of skein's machines.** Forgejo's REST
  API and GitHub's are skein's forge kit, each a dialect of it, over
  skein's HTTP client and JSON (skein's `forge/README.md`); git's arguments in and text out are temper's decoders
  over io's processes. temper's records on jig's channel, in jig's store
  and on the client's wire are generated from schemas by skein's
  generator.
- **Standalone first.** `temper-box serve` runs the engine and one worker in
  one process, on one loop, joined by jig's in-process address; the
  worker's agents are child processes of temper's own binary (`temper-box
  internal agent`); a person's client reaches the engine on loopback;
  the forge is polled, and hinted by its webhooks on loopback when they
  are configured (`service-process.md`, section 1; forge/README.md, section 4). Hosts apart,
  webhooks from another host, the forge as a sign-in provider and the
  forge's client documents come after (`README.md`, section 11). The browser is a
  person's client on loopback too (`../client/web/README.md`).
- **The cluster, for tests.** The same `temper-box serve`, on a
  document of another shape, is an engine alone or a worker alone. One
  engine alone and its workers alone, each a process of its own on one
  machine, joined over loopback in plaintext, with the fakes as
  processes beside them, are the cluster that temper's process tier
  runs from one description (`cluster.md`). It is no
  deployment's shape: a box keeps its one worker inside `temper-box serve`.
- **The box's binary, `temper-box`:** `serve`, `internal agent`, `open`,
  `program-login`, the forge's `forge prepare`, `forge run`, the bare form, and the
  agents' `ctl`, which `../client/ctl/README.md` designs (`binaries.md`).
- **No retries below the domain.** A call goes out once and ends once,
  with the failure the domain's vocabulary defines (`components.md`).
- **The forge's load follows its changes.** The calls a pass makes grow
  with what changed on the forge, never with its history, its labels or
  the items temper holds, and the worlds count them by HTTP request
  (forge/README.md, section 5).
- **Secrets never reach a domain.** A domain names a credential by number
  and generation; values live in the protocol layer's tables and in jig's
  secrets' store: LLM accounts' as jig keeps them, and each forge's
  records under temper's tag, from which its access and its git identity
  are made (section 8; credentials.md).
- **Tested at every tier:** codecs with records drift and
  built-value round trips, and parsers against transcripts of real peers;
  the forge component and the git component in worlds of their own; temper's client joined to the engine's root; the
  one-process world on skein's simulator; the real binary with an owned
  Forgejo, its runner and real git; a live tier with a real model
  (section 10; `../testing/README.md`).

## 2. Reading order

1. **jig's `protocol/README.md` and `protocol/service.md`:** the layer
   temper composes, the service parts jig ships for it, and the routing
   temper copies.
2. **README.md:** this overview.
   - **service-routing.md:** the engine and worker services' routing.
   - **service-process.md:** the services in one process, its waits and stops.
   - **binaries.md:** the deployment's binaries and their closures.
   - **binaries-box.md:** the box's commands and standalone processes.
   - **binaries-client.md:** the client binary's interface.
3. **configuration/README.md:** what `temper-box serve` and its other commands are
   told at start.
4. **credentials.md:** the secrets temper's processes hold: LLM accounts
   as jig's, the forges' records in jig's secrets' store, the forges'
   access, git identities, and the values clients' requests carry.
   - **credentials-llm-accounts.md:** LLM accounts and program sign-in.
   - **credentials-git-identities.md:** git identity credentials.
5. **channel.md:** temper's parts on jig's channel, and a git identity's
   value.
6. **store.md:** temper's families in jig's store.
   - **store-versions.md:** records and store version changes.
   - **store-restart.md:** the store's restart and restore protocol.
7. **llm.md:** where LLM calls are made, host tools, the charter and a
   run's result.
8. **git.md:** the checkout's io: the git component; its sweep of
   `trash` in `git-sweep.md`.
   - **git-operations.md:** the checkout's git operations.
9. **forge/README.md:** the forge's protocol layer on skein's forge kit: what
   temper hands the forge component, the setup host, its marks, its
   webhooks' address, and what it relies on of Forgejo and GitHub.
10. **setup/README.md:** the forge's setup: connected, managed and owned, the
    setup host in the engine, and `temper box`.
11. **sign-in.md:** people's sign-in with Google on a deployment's
    public address: its members, routes, endings and binding.
12. **`../client/README.md`:** temper's client: its client root on `jig-client`,
    its pages, the forge's client family, its view, the terminal, and the
    clients' listener of temper's server; then **`../client/ctl/README.md`**, `temper
    ctl`, the agents' tools.
13. **`../testing/README.md`:** the tiers below the domain.
14. **`../client/web/README.md`:** the browser, the second shell of
    temper's client.
15. **documentation.md:** the user documentation compiled into the
    binary, `temper help`, and agents' `read_docs`.
16. **tooling.md:** the contract with a project's own `prepare` and
    `key` commands, run as jobs on a worker for a landing by turn.
    - **hooks.md:** a repository's hooks, `.temper/hooks/deliver`,
      `finish`, `merge` and `close`: how long each may run, how each is
      found, and the rule by repository while the older paths and
      settings stay.
17. **floor.md:** the oldest store this build opens: its families and
    versions, the owner sets temper hands jig's store, the golden
    deployment, named refusals and the policy on stored changes.
18. **wires.md:** the wires between the engine, its workers, their
    agents and its clients: their frozen versions and their layouts,
    what the opening states, the engine converting, a mismatch, the host
    tools' contract, and `versions`.
19. **checkpoints.md:** a deployment's state at one commit, taken
    while the engine serves by an owner's act, or over a stopped
    deployment by the binaries' `checkpoint`, on jig's store.

## 3. Names

jig's names keep their meaning here (jig's `protocol/README.md`, section
3; jig's `domain/README.md`, section 4): the hub, the core, the fleet,
the worker's link, credential grant, an application's credential,
wire, profile, the clients' component, the default HTTP
mapping, the reference service (`jig-test-service`), the reference
binary (`jig-test`).

| Name | What it is |
|---|---|
| standalone | one process on one person's machine: `temper-box serve` with the engine and one worker joined in process (`service-process.md`, section 1) |
| engine alone | `temper-box serve` on a document with an engine and no worker, its workers' listener on loopback (`cluster.md`, 1) |
| worker alone | `temper-box serve` on a document with one worker and no engine, dialling an engine alone on loopback (`cluster.md`, 1) |
| the cluster | one engine alone and its workers alone on one machine, with the fakes as processes, as temper's process tier runs it (`cluster.md`) |
| the cluster's description | `bin/temper-box/tests/fixtures/cluster/`: the cluster's processes and their documents, read by the process tier (`cluster.md`, 5) |
| the engine's service, the worker's service | each `iterate` with its root and protocol layer, in `temper-engine-service` and `temper-worker-service`, composed by `temper-service` (`service-routing.md`, sections 1 to 2, and `service-process.md`, section 1; `services.md`) |
| the routing | the protocol layer's code between io, components and the root (jig's `protocol/service.md`, section 2) |
| the services' routing | the engine and worker services' composition (`service-routing.md`) |
| the shared process | the services in one process, its waits and stops (`service-process.md`) |
| the binaries | the deployment's processes and closures (`binaries.md`) |
| the box binary | the box's commands and standalone processes (`binaries-box.md`) |
| the client binary | the client's interface (`binaries-client.md`) |
| the forge component | skein's `skein-forge-connection` (forge/README.md; skein's `forge/connection.md`) |
| the git component | `temper-worker-protocol-git` (git.md) |
| git operations | the checkout's git operations (`git-operations.md`) |
| temper's parts | the workspace items, the saved-work branch, the work, the preparation (channel.md) |
| temper's wires, store version, client version | temper's own versions: its wires' (`temper-worker-wire`, `temper-host-tools`: wires.md), `STORE_VERSION`, `CLIENT_VERSION` (section 7) |
| temper's families | the forge connector's records in jig's main store (store.md) |
| the store's versions | records and store version changes (`store-versions.md`) |
| the store's restart | the restore protocol (`store-restart.md`) |
| a forge's records | its credentials, ids and accepted installations: rows of jig's secrets' store under temper's tag (credentials.md, 2.1) |
| LLM accounts | LLM accounts and program sign-in (`credentials-llm-accounts.md`) |
| git identity | one of temper's application credentials, one per forge: Forgejo's git token, or a GitHub App's installations' tokens, with a login and an author (credentials-git-identities.md, section 1) |
| git identity credentials | the identities' values, grants and generations (`credentials-git-identities.md`) |
| the identities' table | the protocol layer's table of git identities' values, on the engine and on each worker |
| a forge's access | what the forge component authenticates with: Forgejo's API token, or a GitHub App's id and key and its installations' tokens; the component's only (credentials.md, section 5) |
| the setup host | skein's in-process host for the setup's plans, a child of the engine's protocol layer (forge/engine.md, 6) |
| the forge's client family | temper's client side of the forge connector: its words, and the adoption form (`../client/forge/family.md`) |
| the terminal shell | `temper-client-shell-terminal` (`../client/line.md`) |
| the browser shell | `temper-client-shell-web` (`../client/web/shell.md`, 1) |
| the bundle | the browser shell's module, glue and stylesheets, skein's bundler's output, which the binary holds and the clients' listener serves (`../client/web/serving.md`, 1, and `../client/web/bundle.md`, 1) |
| the tools | the agents' interface to temper, `temper ctl`: `temper-client-tools` and `temper-client-shell-cli` (`../client/ctl/README.md`, section 4) |
| the clients' listener | `temper-engine-service`'s listener for clients, on which jig's clients' component is mounted with jig's default HTTP mapping, under `/jig` (`../client/engine.md`) |
| the fake git program | git as the worker runs it, faked: its core `crates/temper-fake-git`, `temper-box fake git`, and its hosted half `crates/temper-fake-git-hosted` (`../testing/fakes.md`, 3) |
| the throwaway Forgejo | a Forgejo started by a test script on loopback, with SQLite in a scratch directory, removed after (`../testing/process-tier.md`) |
| locate | the forge connector's read of a repository's id by owner and name (skein's `forge/forgejo-operations.md`, 1) |
| the webhooks' listener | the forge component's own listener for Forgejo's signed deliveries, plaintext on loopback, one path per forge (skein's `forge/connection-webhooks.md`, section 1; forge/README.md, section 4) |

The forge's setup's names are setup/README.md's: connected, managed and owned
(setup/README.md, section 2), the forge's directory (setup/README.md, section 3), reconcile, a plan of the
setup host's (setup/reconcile.md),
the pin (setup/owned.md, 1), the preparation (setup/owned.md, 3), the box and its steps (setup/box.md).

## 4. The engine's service

Moved to [`service-routing.md`](service-routing.md), section 1.

## 5. The worker's service

Moved to [`service-routing.md`](service-routing.md), section 2.

## 6. One process

Moved to [`service-process.md`](service-process.md), section 1.

## 7. Versions

jig's rules hold (jig's `protocol/README.md`, section 7): one version,
frozen, a layout named by its fingerprint, and a reader that converts
another layout by a plan or refuses it by name. The wires between
temper's parts, and what their opening states,
are `wires.md`'s. jig's versions
(its channel's, its store's format and families', its clients' per
owner) and smith's (its charter's, transcript's and result's) are as
jig's `protocol/README.md`, section 7 and smith's `protocol/README.md`,
section 8 give them. temper's sit beside them:

| Version | Owner | Covers | Checked |
|---|---|---|---|
| `temper-worker-wire` | `temper-protocol-worker-wire` | temper's parts: the workspace items, the saved-work branch, the work, the preparation, a job's spec; a git identity's value | the worker's credential at the link's opening: another layout is refused as *wires* (channel.md, section 3; wires.md, section 4) |
| `temper-host-tools` | `temper-engine-smith` | the host tools a charter offers: names, effects, inputs | its kept text, by the gate (wires.md, section 7) |
| `STORE_VERSION` | `temper-protocol-store` | temper's families | at the store's opening: a store written by another refuses to open (store-versions.md, section 1) |
| `CLIENT_VERSION` | `temper-client-protocol-wire` | the forge's client family's records | per owner, as jig's: the engine serves exactly its own, and `temper-box open`, from the same binary, speaks it (`../client/forge/records.md`); frozen, its layout compared at the hello (wires.md, section 9) |
| keys' markers | the forge connector | the keys temper writes into forge bodies | the key's version, its first byte; the marker has none of its own (forge/README.md, section 3) |
| configuration | none | — | unknown members refuse the start (configuration/README.md, section 2) |

- **The store's floor is dev.fabrig.ai's settled store** (`floor.md`, 1
  and 2). Registered families convert at read through the registry;
  defaults, former names and written jobs change them without a patch,
  version move or map walk. Formats outside the registry are fixed at
  their versions (`floor.md`, 6). The wires' and clients' versions are
  frozen too: a change of layout moves none (wires.md, sections 3 and 9).
- **Both channel components are given temper's wire,** its `Spoken`
  statement and the roots its parts hold, when they are made; neither
  knows what it covers, and the engine's converts each part by its
  plans (wires.md, sections 4 and 6).
- **The store keeps the wires** the last start spoke (jig's
  `protocol/store/records.md`, 3, `Wires`): temper's shell hands jig's store
  every wire of `temper_engine_service::wires()`, jig's and smith's beside
  temper's, so the next binary's rehearsal can say whether the workers
  may stay (wires.md, section 10).
- **What a deployment keeps.** The floor (`floor.md`) bounds the stores
  a build opens. Registered layouts come from the store's registry;
  fixed forms and small files have their own readers and tests:

  - **The forges' secrets from a schema:** temper's family
    `temper-secrets` (`temper-protocol-records`, `schema/secrets-v1.schema`),
    a versioned record per row of jig's secrets' store under temper's tag
    (credentials.md, 2.1): an enum of records, one per item (the three
    tokens and the webhook's secret each a `secret N`; the user and the
    app's id `u64`s, big-endian as every schema's; the app's slug a
    `text N`; its key a `secret N` holding its DER; the accepted
    installations a list; the origin its kind and address). The
    owner set names the family, `map: Secrets`, its keys temper's
    numbers below 2^63 (jig's `protocol/store/secrets.md`). A pasted
    bundle is a carried value, jig's from 2^63 up, kept only until the
    request that carries it is used (credentials.md, 2.2): its form is
    `temper-secrets`' `Pasted`, versioned, and one of another version is
    refused by name and pasted again, never patched.
  - **temper's store says what it holds** (store-versions.md, section 1): a
    retained refusal `holds jig-store-people Refusal`; each list of
    answers' items, comments, reviews, statuses, remarks and files its
    `pieces`; each list of raw names (a branch's segments, protection
    contexts, labels, branch names) a declared item type and its
    `pieces`, converted at read through the registry.
  - **An effect's key is an identity, never patched:** kept as written,
    in the store and in the forge's markers, compared byte for byte.
    Its first byte is its version (skein's `forge/client-effects.md`, 1.1);
    the connector reads the versions the floor's stores hold,
    and a purpose's tag keeps its meaning for good, a new meaning taking
    a new tag (forge/README.md, section 3). The marker's wrapper and the bot's
    marker are fixed forms.
  - **Small files** (`../draft/upgrades.md`, section 10): the forge's setup
    `state`, versioned JSON, is read in its supported versions
    and written in the current at its next write; the
    preparation writes it before the engine starts, so it follows the
    forge's own upgrade, whose going back is its dump (setup/owned.md, 3).
    The worker's secret and digest (hex digits and a newline), the
    scripted model's key, the forge's sign-in client secret (setup/owned-sign-in.md,
    1), the box's group files and started markers (setup/box.md, 1) and the admin token's
    handover are fixed forms, never versioned: a change is a new file
    name, read beside the old one for as long as a release may find it.
    The engine writes no small file before its start's wires are durable
    (jig's `protocol/store/records.md`, 3), so going back finds the engine's
    as they were; the preparation's `state` and an owned Forgejo's data
    are the forge's, and going back past a preparation that migrated it
    needs its dump (`../draft/upgrades.md`, 3.2 and 11).
  - **The bytes' pin is skein's `every`** (`skein/docs/design/data/codec/golden.md`,
    1). Each schema's drift compares its records with the committed code;
    round trips, fuzzing and memory tests build their values,
    without family golden files or generated tests. Fixed forms and the
    key's frame and marker keep their own byte tests.

  Outside it: what an external program reads (Forgejo's and the
  runner's files, which their own upgrade carries), the fake backends'
  sandbox, logs, caches and worker roots.
- **Owner sets, the golden store and the gate** are `floor.md`'s:
  the four owners' current families, value types, jobs, frame histories
  and explicit levels (3); a once-kept floor deployment that opens and
  serves, and its floorless refusal (5); registered-family changes
  without patches, and the gate on wires and fixed formats (6). Still
  open: a delivery's evidence with its kept form (jig's
  `protocol/store/README.md`, section 11).

## 8. Secrets

The secrets temper's processes hold, where each comes from, who holds it
and what it reaches are credentials.md's (section 2); the files they are
read from are configuration/README.md's (`configuration/README.md`, section 5). Across the layer:

- **Names in the domain, values beside it** (jig's `protocol/README.md`,
  section 8): no domain state, fact, record of the main store or log
  holds a value. Values move between components through the routing,
  never through a root.
- **temper's forge secrets are in jig's secrets' store:** each forge's
  records (its credentials, ids and accepted installations) under
  temper's tag, read whole at start and written only by the engine's
  protocol layer (credentials.md, 2.1), and the values clients' requests
  carry, kept by jig until their run ends (credentials.md, 2.2). The
  files read at start are configuration/README.md's (`configuration/README.md`, section 5).
- **Nothing crosses a host** in standalone: the link is in process, the
  client and the webhooks on loopback, and a forge off the machine is
  reached over TLS only (configuration/engine.md).
- **The forge's setup's secrets:** the engine's setup host holds an
  admin's token or a GitHub App's key only while a plan runs (setup/README.md,
  the setup host in the engine; credentials.md, section 5). The one setup
  file `temper-box serve` reads is an owned forge's handover `admin_token`,
  read once and removed after its `TakeOver` is done (forge/engine.md, 6;
  configuration/README.md, section 5).

## 9. What temper uses from jig, smith and skein

temper's protocol layer uses jig, smith and skein as they are, at the
revisions temper's `Cargo.lock` pins.

- **jig:** the components of `components.md`, with the service parts jig ships
  (its `jig/docs/design/protocol/service-parts.md`, section 1), composed; the reference
  service's routing and the reference binary, copied; `jig-client` and
  its native transport; jig's client test kit,
  its check, faults and attack stories, for temper's client tiers; the
  conformance kit, as the domain's worlds use it.
- **smith:** the agent shell, the host domain and host half, the codecs;
  the agent process over skein's LLM client, and its program backend,
  Claude Code (smith's `protocol/programs.md`); smith's fake `claude`, in
  the worlds.
- **skein:** io with its in-process names, processes and process groups;
  the startup kit and its private reads; lib's SHA-256 and its HMAC;
  JSON; HTTP's client and server; TLS's client; the codec generator; the
  key-value store; the framed channel; the accounts driver, under jig's
  accounts; the UI kit and its text shell; the simulator, the fake
  machine, the fake LLM peer, the fake issuer and the fake person.
- **What is temper's own:** its presentation, since presentation is the
  application's (jig's `client/domain/client/README.md`), on skein's UI kit and
  text shell (`../client/view.md` and `../client/line.md`); and, where something
  would be better upstream, as jig's design allows, the fake git
  program, on skein's fake checkout and fake forge; and `temper-box program-login`, the
  application's command a program source's holder runs on each host
  (jig's `domain/accounts.md`, section 5).
- **Not used:** `jig-protocol-charter` and `jig-protocol-tools` (llm.md);
  agents in a host's own process (`smith-inline-agent`; section 11);
  jig's reference UI (`components.md`).

## 10. Testing

`../testing/README.md` applies skein's strategy (testing-strategy.md) to
temper: its tiers, fakes, layout and budgets. What checks this layer:

- **Step tests:** every codec with records drift and
  built-value round trips, at its bounds and by fuzz targets; each translation both ways; git's argument and
  environment builders, and its output parsers against transcripts of
  real git (`temper-git-transcripts`); `skein-forge`'s decoders against
  transcripts of the throwaway Forgejo (skein's conformance,
  `skein/docs/design/testing/fakes/throwaway.md`, section 2);
  temper's pages as skein's text shell prints them (`../client/testing.md`
  ).
- **Machine worlds:** the forge component against a scripted HTTP peer
  (skein's `forge/connection.md`, section 10); the git component against
  scripted process ends and outputs; the terminal's host against a
  scripted stdin and stdout.
- **The forge's protocol world,** `tests/engine/protocol-forge`: the
  forge connector world's stories through bytes, over in-memory streams
  cut at random, with the cost checks by HTTP request (`../testing/tiers.md`,
  1.2).
- **The client's joined world,** `tests/client`: temper's client root
  and view, with skein's fake person and jig's check, joined to temper's
  engine root (`../testing/tiers.md`, 1.1, and `../testing/checks.md`, 3).
- **The one-process world,** `tests/one-process`: `temper-service` on
  skein's simulator, killed and restarted at drawn moments, drained by a
  shutdown. Its referee: nothing a party, the worker or the forge saw
  depended on a lost commit; a drained shutdown ends with nothing
  waiting for a commit; no secret's value in a domain's state, a store
  record or an agent process's environment (`../testing/tiers.md`, 1.5).
- **Reconcile's world** is skein's: reconcile against the fake forge's
  protocol layer, serving the setup's routes (skein's `forge/setup/README.md`,
  section 5; setup/testing.md, 2).
- **The process tier and the live tier:** the real binary end to end,
  outside the two suites: a chat answered, a change landed on an owned
  Forgejo with CI and webhooks, the binary killed and restarted, `temper
  box`, `temper-box program-login` over smith's fake `claude`, the cluster
  with a task on each of two workers alone and a worker lost (`cluster.md`, 6);
  then a real model (`../testing/process-tier.md` and `../testing/live-tier.md`; setup/testing.md,
  4).

## 11. Open questions

Each document keeps its own; these cross them:

- **The TUI's later passes** (`binaries.md`): the browser's approval,
  recording and post-landing terminal walk stay with the client's
  design (`../client/README.md`, section 5).
- **Names while running.** io connects to addresses, resolved at
  startup, while a forge's address may move behind its CDN, and a name
  that does not resolve at the start leaves its forge unreachable for
  that run, not the start refused (setup/init.md, 6). GitHub's names
  alone are resolved once serving, when a forge first needs them, and
  each holds its pass for one lookup (`forge/README.md`, 2.6). Default:
  a restart resolves names again; skein's DNS client, or
  skein-forge-connection resolving at its connect, would mend it and
  take the lookups off the loop (a skein addition).
- **Other names resolved before serving.** A sign-in's hosts
  (configuration/README.md) and Google's sign-in endpoints
  (`sign-in.md`, 3.1) are still resolved by the shell at start, and a
  kept forge's own name before the start serves. Default: so; the
  deferral GitHub's names take would serve them too.
- **HTTP/1.1 only.** Forgejo serves it; a forge that required HTTP/2
  would need skein to grow it. Default: HTTP/1.1.
- **After standalone,** each designed where it is named, and needed only
  beyond one person's machine: webhooks from another host, over TLS
  (forge/README.md, section 4); hosts apart: TLS on the engine's listeners,
  containment for shared hosts, a worker alone on another host (jig's
  `protocol/needs.md`, section 1; on one machine it is the cluster's,
  `cluster.md`); the forge as people's sign-in
  provider (credentials.md, section 8); the forge's client documents and
  panels (`../client/forge/README.md`); and what the forge's setup leaves for later
  (setup/README.md, section 9). Default: standalone.
- **`temper-box init` for the cluster's shapes.** Default: none; `temper-box
  init` writes standalone, and the cluster's documents are its
  description's templates (`cluster.md`, 5). Hosts apart would want it.
- **Limits across processes.** Each process checks its own document
  (configuration/limits.md); what holds only between an engine's
  limits and a worker's, the cluster's one template holds, writing the
  same `application.limits` into each of its documents, and jig's link
  holds the rest at its opening, by its terms and the hello's bounds
  (`cluster.md`, 3). Default: so; a digest of the limits at the
  link's opening, beside the payload's version, comes with hosts apart.
- **The description's Rust reader's home.** Default: temper's process
  tier's support (`cluster.md`, 5); skein-world's, once a second product runs a
  cluster of its own.
- **The box's members in the engine's closure** (`binaries.md`, 2): `temper-engine`
  checks a document's `box.` limits and `demo` member as the box's
  binary does, so it links the box's machine and the fakes' crates,
  `temper-box-machine` and `temper-fake`, with none of their commands (`components.md`
  ). Default: so; a check of those members in a crate of no side would
  cut both from its closure.
- **The box's `upgrade`** (`binaries.md`): the draft gives it to
  `temper-engine` (`binaries.md`, 2) and has the box keep the commands
  it has (`../draft/deployment-binaries.md`, sections 3 and 9); the box
  holds `temper-engine`'s `versions` row and code, and neither binary
  has `upgrade` yet. Default: the box's table takes `upgrade` as one
  constant with the engine's, once jig's rehearsal and going back are
  built.
