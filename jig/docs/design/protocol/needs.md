# jig's protocol layer: what jig needs from skein and smith

## 1. What jig needs from skein

Each is generic, and says nothing about jig.

- **Joined sockets in one process:** a listener on an in-process name
  accepting sockets joined in memory to those that connect to it, with
  the same requests, events, backpressure and ends as TCP, and never
  reaching the simulator, for the one-process shape (`README.md`, section 4; skein's
  `runtime/io/sockets.md`, 5, and `skein/docs/design/runtime/io/README.md`, 3).
- **TLS's server side with independent output,** for the engine's
  workers' listener between hosts, and for clients' HTTPS on an
  application's server built on skein's HTTP, as the reference's is
  (skein's `net/tls.md`, section 8; skein's `net/channel.md`, section 12). Not
  needed in one process.
- **Contained process trees,** for workers' agents: spawned within a
  deadline, with an environment given whole and only named directories
  writable, stopped politely, then terminated, then killed, and proved
  empty (`../domain/hosts/README.md`, section 8; skein's `runtime/io/processes.md`, 2–4;
  smith's `protocol/README.md`, section 9 asks the same). Without them,
  each agent process, command and check runs in a process group of its
  own, which a stop signals whole (skein's `runtime/io/processes.md`).
- **Private files read at startup:** the startup kit's
  `read_private_file`, refusing a file that is not private to the
  process's user, or reached through a link, for configuration's secrets
  (skein's `runtime/shell-startup.md`, 1.1; skein's `runtime/io/files.md`, 11).
- **A cryptographic generator:** skein-lib's `CryptoRng`, keyed by the
  seed the shell draws at start (skein's `runtime/lib.md`, section 8; skein's
  `runtime/shell.md`, section 5), for minted credentials, the printed link's
  secret, services' credentials, the accounts driver's states and
  verifiers, and the installation's identifier (`store/keys.md`, 1.4).
- **The loop's host:** skein's `Host` and `drive`, with
  `next_policy_deadline` (skein's `runtime/shell.md`, section 11; `service.md`,
  section 4).
- **For `jig-client` on WebAssembly, and its records:** skein-lib and
  skein-codec built for `wasm32-unknown-unknown`, with skein-http's
  machines for the default mapping's client end, and `skein-ui` for the
  reference UI, checked by skein's gate (skein's `runtime/lib.md`, section 2;
  skein's `testing/layout.md`, section 1); skein-lib's bounded `base64`, for
  records in the default mapping's server-sent events (`client/default-http.md`,
  2);
  and skein-lib's `Secret`, the generator's `secret` type, whose `Debug`
  shows its length only, for credentials' values (`README.md`, section 8; skein's
  `data/codec/README.md`, section 3).
- **For the reference UI and its shells, skein's UI kit** (skein's
  `ui/README.md`, `skein/docs/design/ui/text.md` and `skein/docs/design/ui/web/README.md`), which an application's own
  presentation may use or leave:
  - `skein-ui`: the tree and its vocabulary, owners as opaque numbers,
    to which jig maps its own (`../client/domain/view.md`, section 3;
    skein's `ui/tree.md`, 1.3), the builder with slots and splicing, bindings
    and acts, the diff and the view, style tokens and the look's names,
    the phrasebook's machinery with its reference hole, Markdown,
    scenes, the outline's rules, and limits with their worst case;
  - `skein-ui-text`, the text shell, which jig's terminal client
    composes with jig's settings (skein's `ui/text.md`);
  - `skein-ui-web`, the browser parts each usable alone and their
    default mapping of roles to elements, beside which jig's own parts
    sit (`browser/loop.md`, 1.1): their platform halves call through the glue
    crates, `wasm-bindgen`, `js-sys` and `web-sys`, pinned in the
    workspace, which with their dependencies are the only `unsafe` such
    a client runs (`skein/docs/foundation/programming-model.md`, section 2.1);
    `jig-client-shell-web` forbids it in its own code;
  - `skein-ui-bundle`, the bundler, which writes the reference UI's
    bundle, under the prefix the application's server serves it at
    (skein's `ui/web/bundle.md`, 2 to 4; `browser/README.md`, 4.2);
  - `skein-fake-person`, the person over the tree, and
    `skein-fake-person-browser`, its face over skein's browser kit (a
    name matched by words as well as exactly, a press at a point within
    a node's box: skein's `testing/browser-vocabulary.md`, section 1, and `testing/browser-operations.md`, section 1) and the
    harness, which stops the application's binary and starts it again
    within a story, with skein's stories for a browser shell, which
    jig's tier calls beside its own (skein's `ui/web/testing.md`, 2;
    `../client/domain/testing.md`, section 3; `browser/testing.md`
    ).
- **LLM accounts' mechanism:** the accounts driver, with sources added
  and removed while it runs, sign-out with revocation, the client id a
  registration is issued kept with its record, signed in with the
  subject and scopes granted, and records kept through its owner's
  store (skein's `oauth/accounts.md`, 2 to 4); a credential with its scheme,
  and the failures' classes (skein's `llm/vocabulary.md`, sections 1 and 1.5).
- **What it has, as it is:** the framed channel, codecs and their
  generator, the JSON reader and writer, the key-value store, the LLM
  client and connection component and their fake peer, OAuth's client and
  fake issuer, HTTP's server and client, server-sent events.
- **The store** needs skein-kv as it is, and io's `OpenWrite` and `Cut`
  (skein's `runtime/io/files.md`; `store/README.md`, section 10).
- **Nothing new for the client's API.** Its generic half (keyed
  requests until durable, level-triggered watches, the link, generic
  guards) is jig's while it has one user.

## 2. What jig needs from smith

- **What smith gives:** its charter, with each model's window, output and
  prices with cache writes, and the budget's reserve (smith's
  `protocol/charter.md`, section 2); its transcript, whose turns say
  whether they open a window (smith's `protocol/transcript.md`, section
  3); its channel, with a start's
  messages and the run's identity, messages refused at the kit's
  entrance, the answer's final fence, and content-free facts (smith's
  `protocol/channel.md`, sections 3 to 8); the host domain and its host
  half, `smith-host-protocol`, with the agent process and its shell as a
  library, `smith-agent-shell` (smith's `protocol/host.md`, section 3);
  the parent face (smith's `domain/host.md`, 9.4); `smith-inline-agent`,
  with its LLM component, serving many runs' completions with one table
  of grants, and its machine component; and smith's codecs between bytes
  and smith's values: `smith-protocol-channel`'s functions, and
  `smith-channel`'s records for a call's name and a fact (`decisions.md`, section 1).
- **`smith-transcript` built for `wasm32-unknown-unknown`,** as it is:
  `jig-client` decodes turns with it below its surface, through
  `jig-protocol-transcript`, whose build for that target jig's gate
  checks (`../client/domain/client/conversations.md`).
- **LLM accounts' grants:** a grant carrying a credential with its
  scheme, or a program source's name; the limited, refused and
  unauthorized notices; the program backend, which runs Claude Code
  with its own login and its built-in tools asking smith's policy; and
  the host half's sources, login checks and sign-outs (smith's
  `protocol/channel.md`, section 6; `smith/docs/design/protocol/host.md`, section 2;
  `smith/docs/design/protocol/programs.md`, sections 3 to 4, and `smith/docs/design/protocol/programs-claude-code.md`, section 1).
- **Not asked: a completion's result contract.** A protocol layer that
  runs `smith-inline-agent` keeps each slot's beside smith's LLM
  component, as smith's own local host does (`llm.md`, section 3).
- **What jig asks of smith:**
  - **a run's live content on smith's channel:** the host asks at a run's
    start, one boolean per kind (`text`, `reasoning`, `input`), and
    jig's assignment carries the ask, set for a run whose task is a
    conversation. smith sends, best effort and counted when dropped, the
    main conversation's pieces of the kinds it builds (`text` and
    `input`), each
    naming the turn it will be told as, its completion's retry, its kind
    and its offset, all before that turn's record. An ask for a kind
    smith does not build is not refused, and gets nothing. Without the
    ask, a run in an agent process shows its activity, and its words as
    each turn commits (`channel/owners.md`, 1.4; `../domain/hosts/hub.md`, 5).
    A conversation's presentation wants it. A tool begun with its call
    stays `smith-inline-agent`'s;
  - **closing smith's LLM component as an owner closes it, later:** a
    completion asked while it closes refused as closed, never as a local
    limit. Its abort, which ends every completion in flight for a
    process that stops without draining (`service.md`, section 5), is
    smith's as it stands.
