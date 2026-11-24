# jig's design: crates' names

## 1. Crates' names

- **Named by layer.** A crate's name says the layer it is in
  (skein/docs/foundation/programming-model.md, section 2):
  - `jig-domain-<name>`: a domain's step code: the core and its children,
    the hub;
  - `jig-protocol-<name>`: the protocol layer's: codecs, components,
    translations, parsing;
  - test kits keep their kind as their prefix (`jig-test-`, `jig-fake-`,
    `jig-scripted-`, `jig-conformance`), and worlds end in `-world`.
- **Client crates live under one prefix.** Every client crate, wherever
  it lives, is named `<project>-client-{domain,protocol,<kind>}-<...>`,
  so that everything client-related lives under `<project>-client`. The
  base crate of that prefix is `<project>-client`, as skein's `skein-ui`
  is the base of `skein-ui-*`. After the prefix comes the layer, as for
  the engine's crates (`-client-domain`, `-client-protocol-<name>`), or
  the crate's kind: `-client-view`, `-client-shell-<platform>`, a test
  kit's `-client-test-<name>`, `-client-fake-<name>` or
  `-client-scripted-<name>`, and a world's name ending in `-world`. The
  project is jig, or an application under its own name: `jig-ops` for
  `ops`. An application's name, `ops` in `jig-ops-client`, is one word,
  and never one of jig's layer or kind words (`domain`, `protocol`,
  `engine`, `test`, `fake`, `scripted`, `conformance`, `shell`, `view`),
  so that a crate named `jig-<layer>-client`, such as
  `jig-domain-client`, is no application's client.
- **What a client crate is:** a crate that exists for the client: its
  domain, its protocol (the records it speaks, even where the engine
  reads them too), its presentation, its shells, its test kits and its
  worlds. A crate that runs in the engine's process keeps the engine's
  name, even when it serves clients; a crate shared by both ends for
  another purpose keeps its own.
- **An application follows the same rules** for its own crates, under
  its own name: its engine's `jig-ops-domain-*` and
  `jig-ops-protocol-*`; its client's `jig-ops-client-domain`,
  `jig-ops-client-protocol-wire` and `jig-ops-client-view`, and its
  browser shell, `jig-ops-client-shell-web`, as every application's is
  `<application>-client-shell-web`;
  `jig-ops-domain-connectors`, its connectors' numbers, which its
  engine's root and its client share, keeps the engine's name. Its
  service (its `iterate`, its protocol layer's routing and its server)
  and its binary carry its name alone: `jig-ops-service`, `jig-ops`.
- **jig's engine crates,** by layer, and where each is designed:

| Layer | Crates |
|---|---|
| a domain's step code | the core, `jig-domain-core`, and its children `jig-domain-accounts`, `-authority`, `-brief`, `-fleet`, `-notes`, `-people`, `-tasks`, `-views` (`domain/engine/README.md`, section 3); the hub, `jig-domain-host` (`domain/hosts/hub.md`); the engine's link between them, `jig-domain-hub-link` (`domain/hosts/hub.md`, 1); their shared link values, `jig-domain-link` (`domain/root.md`, section 2) |
| the protocol layer's | `jig-protocol-charter`, `jig-protocol-transcript`, and the codecs, components and parsing `protocol/components.md`, section 1 lists; among them the clients' component, `jig-protocol-engine-clients` (`protocol/client/README.md`, section 2), and the default HTTP mapping's engine end, with its example guards, `jig-protocol-engine-clients-http` (`protocol/client/default-http.md`, 4; `protocol/browser/serving.md`, 1.2) |

- **jig's client crates,** with `ops`'s and an application's browser
  shell, and where each is designed:

| Crate | What it is | Designed in |
|---|---|---|
| `jig-client` | the client the application holds, `jig_client::Client`, composing the client's domain and its link; jig's helpers as modules: `shapes`, `facts`, `thread`, `words` | `client/domain/client/README.md` |
| `jig-client-domain` | the client's domain: the link's state, watches by handle and their latest documents, the request table's use, kept work, families as bytes | `client/domain/client/README.md` |
| `jig-client-protocol-wire` | jig's wire records, for both ends, from schemas | `protocol/client/wire.md`, section 1 |
| `jig-client-protocol-link` | jig's messages encoded and decoded, the stream's events, the small store's records; no HTTP | `protocol/client/README.md`, section 2 |
| `jig-client-protocol-http` | the default HTTP mapping's client end: a message as a method, a path, headers and a body; the stream's server-sent events cut into events; for the host and wasm32 | `protocol/client/default-http.md`, 4 |
| `jig-client-view` | the reference UI: pages, places, navigation, the confirmation, drafts, notices, cards and the view, on `jig-client` and skein's `skein-ui` | `client/domain/pages.md`; `client/domain/view.md` |
| `jig-client-view-words` | the reference UI's words, below `jig-client-view`, which re-exports them under its paths: the phrasebook, each part's vocabulary, jig's owner, region names and style tokens | `client/domain/view-crates.md` |
| `jig-client-view-codec` | the generated record code of the reference UI's records, below `jig-client-view`, which keeps their schema and translates them | `tools/crates/design.md`, 3.2 |
| `jig-client-shell-native` | the native shell, over skein's HTTP client, with `jig-client-protocol-http` | `protocol/client/shells.md`, section 1 |
| `jig-client-shell-web` | jig's browser parts, over skein's `skein-ui-web`, each usable alone: the client's adapter over skein's fetch, the printed link's sign-in, jig's look | `protocol/browser/README.md`, section 2, and `protocol/browser/loop.md`, section 1.1 |
| `jig-client-test-shell-web` | jig's test application's browser shell, written to be copied | `protocol/browser/application.md`, 1.1 |
| `jig-client-test-domain`, `jig-client-test-view`, `jig-client-test-kit`, `jig-client-test-records`, `jig-client-test-protocol-wire` | the client's test kits | `client/domain/testing.md` |
| `jig-client-test-protocol-wire-codec` | the generated record code of the test application's client records, below `jig-client-test-protocol-wire`, which keeps their schema and translates them | `tools/crates/design.md`, 3.2 |
| `jig-client-kit`, `jig-client-protocol-kit` | what the client's worlds share: the harness, fronts and stories; the protocol client | `domain/testing.md`, section 7 |
| `jig-client-scripted-engine` | the engine as the client's worlds play it | `client/domain/testing.md`, section 4 |
| `jig-client-fake-person-browser` | jig's stories for a browser shell and its attack stories, on skein's face and harness | `protocol/browser/testing.md` |
| `jig-client-world`, `jig-client-protocol-world`, `jig-client-wire-world`, `jig-client-browser-world` | the client's worlds: through its surface; through bytes; its wire's records; the browser tier | `client/domain/testing.md`, section 4; `protocol/client/testing.md`, section 1; `protocol/browser/testing.md` |
| `jig-ops-client-domain`, `jig-ops-client-protocol-wire`, `jig-ops-client-view`, `jig-ops-client-shell-web` | `ops`'s client crates, written to be copied: its client's domain, its families' wire, its presentation on the reference UI, its browser shell | `examples.md`, section 8; `protocol/client/README.md`, section 8 |
| `<application>-client-shell-web` | an application's browser shell: its start and its loop, copied from `jig-client-test-shell-web` | `protocol/browser/application.md`, 1.1 |

- **Not client crates,** since they run in the engine's process: the
  clients' component, `jig-protocol-engine-clients`, and its world,
  `jig-engine-clients-world`; the default mapping's engine end,
  `jig-protocol-engine-clients-http`; and `jig-fake-parties`, the parties
  of the engine's worlds. `jig-protocol-transcript`, turns decoded for
  the engine and the client alike, keeps its name too.
- **Outside the layers,** a crate is named by its kind: the engine's test
  kits (`jig-test-domain`, and the rest), its worlds, the reference
  root's test application, `jig-test` and `jig-test-service`
  (`protocol/service.md`, section 7), with the rig its stories share,
  `jig-test-kit` (`domain/testing.md`, section 7), and `jig-boundary`,
  the check of the crate graph.
- **The UI kit's crates are skein's** (skein's `ui/README.md`, section 2): the
  tree, `skein-ui`, which `jig-client-view` builds on; the text shell,
  `skein-ui-text`; the browser parts, `skein-ui-web`; the bundler,
  `skein-ui-bundle`, a tool that makes the bundle a browser loads
  (skein's `ui/web/bundle.md`, 2); the person, `skein-fake-person`, and its
  browser face and harness, `skein-fake-person-browser`. jig has no
  crate of its own for them: a terminal presents the client with
  `skein-ui-text`, and the reference UI with jig's settings
  (`client/domain/view.md`, section 15).
- **How `jig-boundary` finds client crates,** in the crate graph of the
  project it checks: a crate is a client crate when its own name has a
  client crate's shape, when it depends on a client crate, or when every
  crate that uses it is one. Each it finds outside the project's prefix,
  taken by the project's whole name (`jig-ops-client` for `jig-ops`), is a
  breach, unless the check's caller exempts it by name. A dependency is a
  client crate by its name in two cases only:
  - under a prefix this section gives, `<project>-client` or
    `<project>-<application>-client`, where `<project>` is jig or the
    checked project, by its whole name, and `<application>` is never a
    layer's or a kind's word; but not the records both ends read, whose
    names end in `-client-protocol-wire`, `-client-test-protocol-wire` or
    `-client-test-records`;
  - one of the UI kit's crates a client is built and tested on, by its
    name: `skein-ui`, `skein-ui-text`, `skein-ui-web`, `skein-fake-person`
    and `skein-fake-person-browser`; not the bundler, a tool.

  A name in any other namespace, skein's among them, follows its own
  repository's rules, and says nothing here: a `<kit>-<service>-client`
  there is a client of a service, which a crate in the engine's process
  may use, and makes no crate that depends on it a client crate. Another
  UI kit counts only once this section names its crates, as it names
  skein's; never from a `-client` or a `-ui` in a name. A crate that
  presents an application's client is found without that, by its use of
  `jig-client` or of the project's own client crates, and a helper that
  only client crates use, by its users.
