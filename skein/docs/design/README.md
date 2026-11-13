# skein's design

Provisional, 2026-10-19. The index of skein's parts: what each holds,
its crates and its design document, how the crates depend on one
another, and what skein leaves to the services built on it. The rules
every part follows are skein's foundation documents: `programming-model.md`,
`testing-strategy.md` and `notes.md`. How a service is built on skein is
`examples.md`, section 10. `STATUS.md` says how far the code is from
this design.

## 1. In one page

- **A kit, not a framework.** skein has no service trait, no generic loop,
  no scheduler and no callbacks. A service calls the parts by name, in
  its own loop of about ten lines. Nothing in skein calls into a service,
  not even the simulator.
- **io_uring is the kernel interface.** io talks to the kernel in owned
  records: an operation goes down carrying its memory, and its completion
  comes back carrying the same memory. The ring backend maps each record
  onto one submission; the simulator implements the same records; the
  readiness backend (epoll, kqueue), for hosts without io_uring and
  macOS, implements them a third time. Nothing above the records changes.
- **Limits at the edges, dynamic memory inside.** Parts check admission
  limits and derived capacities; containers grow with admitted work.
  Worlds hold measured process peaks under named scenario ceilings
  (programming-model.md, 6.3).
- **Built when pulled.** A part is built when its first user needs it,
  and that user is its first test.
- **Laid out by role** (testing/README.md, section 6): step crates, and the
  fakes and the simulated world a playground ships, in
  `crates/`; shared test harnesses and fakes in `testing/`, worlds in
  `tests/`, the example services in `examples/`.

## 2. The parts

| Part | Crate | What | Design |
|---|---|---|---|
| lib | `skein-lib` | handles, slabs, bounded containers, cursors, streams, deadlines, time | [runtime/lib.md](runtime/lib.md) |
| the kernel boundary | `skein-io` (`kernel`) | the records io submits to a backend and the completions it gets back | [runtime/kernel.md](runtime/kernel.md) |
| the readiness backend | `skein-shell` | completions emulated over epoll or kqueue on hosts without io_uring | [runtime/readiness.md](runtime/readiness.md) |
| io | `skein-io` | sockets, pipes, files, processes, signals to the service | [runtime/io/README.md](runtime/io/README.md) |
| sockets and pipes | `skein-io` | stream entities, transfers, close and in-process sockets | [runtime/io/sockets.md](runtime/io/sockets.md) |
| files | `skein-io` | the file machine, its requests, events and operation ownership | [runtime/io/files.md](runtime/io/files.md) |
| processes | `skein-io` | spawning children, their pipes, signals and containment | [runtime/io/processes.md](runtime/io/processes.md) |
| the shell kit | `skein-shell`, `skein-fail-stop` | the io_uring backend, the clock, the seed, startup; the panic hook every binary's `main` installs first | [runtime/shell.md](runtime/shell.md) |
| the simulator | `skein-sim` | the simulated kernel and its faults; beside it, the conformance suite (`skein-conformance`) | [testing/simulator.md](testing/simulator.md) |
| the fake LLM | `skein-fake-llm-domain`, `skein-fake-llm-protocol` | shared bounded scripts and independent byte peers; application worlds supply tools and policy | [testing/fakes/llm.md](testing/fakes/llm.md) |
| the key-value store | `skein-kv` | ordered byte keys and values in memory, made durable by a commit log and snapshots on io's files | [data/kv.md](data/kv.md) |
| the fake push service | `skein-fake-push` | for tests: a push service that checks a Web Push request's form, token and seal, records it and answers by a script; its peer in `skein-fake-peers` | [testing/fakes/push.md](testing/fakes/push.md) |
| the fake checkout | `skein-fake-checkout` | for tests: deterministic files, scripted commands and local git mechanics; worlds own policy and delivery | [testing/fakes/checkout.md](testing/fakes/checkout.md) |
| the counting allocator | `skein-heap` | for tests: the heap counted, and each scenario's peak checked under its ceiling | [testing/README.md](testing/README.md) |
| the simulated world | `skein-sim-world`, `skein-fake-machine` | a service's processes hosted over the simulator and run a step at a time, as the harness and a playground run them; the minimal fake machine | [testing/world.md](testing/world.md) |
| the world harness | `skein-world` | for tests: the simulated world's run with a referee and every world's checks, and the same processes over the ring | [examples.md](examples.md) |
| HTTP | `skein-http` | HTTP/1.1 client and server, server-sent events | [net/http/README.md](net/http/README.md) |
| the HTTP client | `skein-http` | the client's requests and events for an exchange | [net/http/client.md](net/http/client.md) |
| the HTTP server | `skein-http` | the server's requests and events for an exchange | [net/http/server.md](net/http/server.md) |
| server-sent events | `skein-http` | reader and writer machines over body streams | [net/http/sse.md](net/http/sse.md) |
| testing HTTP | `skein-http` | step tests and worlds for HTTP and server-sent events | [net/http/testing.md](net/http/testing.md) |
| JSON | `skein-json` | a bounded tokenizer, a sized writer | [data/json.md](data/json.md) |
| framed channels | `skein-channel` | a framed channel between two peers over a stream: an opening that agrees a version, bounds, flow control | [net/channel.md](net/channel.md) |
| codecs | `skein-codec`, `skein-codegen` | records described by schemas, and their codecs, bounds and golden bytes generated from them | [data/codec/README.md](data/codec/README.md) |
| schema diffs | `skein-codegen` | matching two family versions and listing their changes | [data/codec/diff.md](data/codec/diff.md) |
| record patches | `skein-codec` | versioned patch files and upgrades of stored records | [data/codec/patches.md](data/codec/patches.md) |
| the patch command | `skein-codegen` | keeping a crate's patches, generated files and drift check | [data/codec/command.md](data/codec/command.md) |
| TLS | `skein-tls` | the TLS client, a stream over rustls's unbuffered connection, with ring | [net/tls.md](net/tls.md) |
| Web Push | `skein-push` | a push message sealed for a browser's subscription and signed with the service's VAPID key, and its answer read | [net/push.md](net/push.md) |
| the link | `skein-link` | one socket and its optional TLS, under a protocol layer's HTTP clients: the handshake, a body's upload, a bounded close that sends `close_notify` | [net/link.md](net/link.md) |
| LLM | `skein-llm` | provider-neutral calls, streaming deltas, concrete replay and completions; subscription wire dialects | [llm/README.md](llm/README.md) |
| LLM connections | `skein-llm-connection` | a protocol layer's component that runs LLM calls end to end: connecting, TLS, deadlines, reuse | [llm/connection.md](llm/connection.md) |
| OAuth | `skein-oauth`, `skein-oauth-codec`, `skein-oauth-accounts`, `skein-fake-oauth` | bounded sign-in and refresh client, with its saved records' generated code in a codec crate below it; the driver that runs it end to end for its owner, listener, connection and keeping included; an independent rotating issuer for worlds | [oauth/README.md](oauth/README.md), [data/codec/README.md](data/codec/README.md) |
| the OAuth issuer | `skein-oauth-issuer` | OpenID Connect's issuer, stateless: an authorization and a token request checked, an ID token signed, the discovery document, key set and userinfo answer | [oauth/issuer.md](oauth/issuer.md) |
| the forge's vocabulary and dialects | `skein-forge` | a code forge's calls and documents, deliveries and credentials, neutral and rendered by a dialect, Forgejo's or GitHub's | [forge/README.md](forge/README.md), [forge/github.md](forge/github.md) |
| the forge component | `skein-forge-connection` | a protocol layer's component that runs a forge's calls end to end: connecting, TLS, credentials, deliveries | [forge/connection.md](forge/connection.md) |
| the forge client | `skein-forge-client` | a bounded view of a forge for its owner's domain, and each write made at most once | [forge/client.md](forge/client.md) |
| the forge setup | `skein-forge-setup` | makes a forge ready for a service: the identity, credentials, repositories and hooks it acts with | [forge/setup/README.md](forge/setup/README.md) |
| the fake forge | `skein-fake-forge-domain`, `skein-fake-forge-http`, `skein-fake-forge-forgejo`, `skein-fake-forge-github`, `skein-fake-peers`, `skein-throwaway-forgejo` | for tests: a forge's state and git, Forgejo's API over it, GitHub's API and blob layer, a forge peer on loopback beside the LLM's and OAuth's, the pinned Forgejo run in a scratch, and the conformance that holds the dialect to it | [testing/fakes/forge.md](testing/fakes/forge.md) |
| the fake GitHub | `skein-fake-forge-github`, `skein-fake-forge-http`, `skein-fake-peers` | GitHub's API and browser faces, signed job-log blobs and hosted peer scenarios | [testing/fakes/github.md](testing/fakes/github.md) |
| the browser kit | `skein-browser` | for tests: a headless Chromium driven over the DevTools protocol on a pipe, by role and name, from the loop | [testing/browser.md](testing/browser.md) |
| the UI kit | `skein-ui` | a semantic tree of an application's state, by accessibility roles, built, diffed into patches, its acts bound to their owners; words, Markdown, scenes | [ui/README.md](ui/README.md) |
| the text shell | `skein-ui-text` | the tree printed as lines, a typed line read as an act, for a terminal or a pipe | [ui/text.md](ui/text.md) |
| the full-screen shell | `skein-ui-screen`, `skein-ui-screen-terminal` | the tree laid out on a terminal's grid of cells in an application's theme, keys read as acts | [ui/screen.md](ui/screen.md) |
| the browser parts | `skein-ui-web`, `skein-ui-bundle` | parts an application's browser shell wires into its own loop, compiled to WebAssembly; the bundler | [ui/web/README.md](ui/web/README.md) |
| a world in a browser | `skein-playground` | an application's simulated world in a Web Worker, stepped in slices paced by the browser; the visitor speaking HTTP to its listener; the bridge; the session log and its replay | [playground.md](playground.md) |
| the fake person | `skein-fake-person`, `skein-fake-person-browser` | for tests: a person who finds nodes by role, name and text and acts on them, on the tree or in Chromium | [ui/README.md](ui/README.md), [ui/web/README.md](ui/web/README.md) |

Directory reading guides:

- [Runtime](runtime/README.md): lib, the kernel boundary and backends, the shell and io.
- [Data](data/README.md): JSON, codecs and the key-value store.
- [Network protocols](net/README.md): TLS, links, HTTP and framed channels.
- [Fakes](testing/fakes/README.md): LLM peers, forges and checkout mechanics.

## 3. The crate graph

```
crate          depends on
skein-lib      nothing
skein-fail-stop nothing
skein-io       lib
skein-http     lib
skein-json     lib
skein-channel  lib
skein-codec    lib
skein-codegen  codec, fail-stop (a generator, in consumers' tests)
skein-tls      lib, rustls (and ring beneath it)
skein-llm      lib, http, json
skein-llm-connection lib, io, tls, llm
skein-oauth    lib, json, codec, tls
skein-oauth-accounts lib, io, http, tls, oauth
skein-fake-oauth lib, json, oauth, tls
skein-oauth-issuer lib, json, tls
skein-forge    lib, json
skein-forge-client lib, forge
skein-forge-connection lib, io, http, tls, json, forge
skein-forge-setup lib, io, http, tls, json, forge
skein-fake-forge-domain lib
skein-fake-forge-http lib, http, json, forge, fake-forge-domain
skein-fake-forge-forgejo lib, http, json, forge, fake-forge-domain, fake-forge-http
skein-fake-forge-github lib, http, tls, json, forge, fake-forge-domain, fake-forge-http
skein-fake-llm-domain lib
skein-fake-llm-protocol lib, http, json, llm, fake-llm-domain
skein-push     lib, tls
skein-fake-push lib, json, tls, http
skein-fake-peers lib, io, http, llm, oauth, shell, fake-llm-*, fake-oauth, fake-forge-*, fake-push; rustls
skein-fake-checkout nothing
skein-throwaway-forgejo lib, io, http, json, shell, forge, fail-stop, world
skein-browser  lib, json
skein-ui       lib
skein-ui-text  lib, io, ui
skein-ui-screen lib, ui
skein-ui-screen-terminal lib, io, ui, ui-screen
skein-ui-web   lib, ui; on wasm32, wasm-bindgen, js-sys, web-sys
skein-ui-bundle lib, fail-stop (a tool), wasm-bindgen-cli-support
skein-fake-person lib, ui
skein-fake-person-browser lib, io, http, shell, ui, ui-web, browser, fake-person, fake-peers, scratch, world
skein-shell    lib, io; natively, io-uring (Linux) and libc (runtime/shell.md, 13)
skein-sim      lib, io
skein-sim-world lib, io, sim, shell (its Host)
skein-fake-machine lib, io, sim, sim-world
skein-kv       lib, io
skein-playground lib, io, http, sim-world, ui-web; on wasm32, wasm-bindgen, js-sys, web-sys
```

The crates from outside skein are few, and each is an exception the
programming model names (its 2.1 and 3): `io-uring` and `libc`, the
shell's, for the ring adapter; and `rustls` and its dependencies, `ring`
among them for its cryptography, the TLS client's; and the browser
parts' glue, `wasm-bindgen`, `js-sys` and `web-sys`, with the bundler's
`wasm-bindgen-cli-support`, pinned to exact versions (ui/web/bundle.md, 1).

## 4. Not in skein

- **Domains, and protocols only one application speaks:** jig's
  protocol between engine and workers. They are built on skein's machines.
- **What a service does with an LLM:** its tools and their execution,
  when to sign in or refresh a credential, and whether to retry. The
  calls, their connections and OAuth's documents are skein's.
- **What a service does on a forge:** what it watches, its effects and
  their purposes, its policy, and what people see. A forge's calls, its
  component, client and setup, and the fakes of a forge are skein's.
- **A service's wiring:** its `iterate`, its admission checks, its
  `main`.
- **What a simulated program does.** The simulator plays the kernel; the
  files and programs a scenario needs come from the service's own fake
  machine, which plugs into the simulator.
- **Schedulers, async and threads.** There is one loop per process. A
  service that needs more cores runs more processes.
