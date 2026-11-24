# jig's design: the client stack

## 1. The client

```
presentation   the application's: routes and URLs, screens, words and look, confirmations, drafts, home and navigation
helpers        jig's, optional: shapes, facts and models, jig's words, the reference UI
the client     jig-client: the link, watches by handle, the request table, kept work, families as bytes
transport      the application's: HTTP, a socket or in-process; jig's default HTTP mapping and its example guards, to use or replace
the API        jig's messages: plumbing operations, subjects and reads, documents with revisions and offers
the engine     the application's root around jig's core, and jig's clients' component
```

- **Nothing below the presentation knows what a page is,** and nothing
  of jig's knows HTTP but its default mapping onto it.
- **Presentation is the application's:** its routes and URLs, its
  screens, words and look, its confirmations and drafts, its home and
  navigation. jig's client knows no page, place or arrangement. An
  application presents with skein's UI kit (skein's `ui/README.md`), with
  anything else, or with nothing at all.
- **So are the server and its URL space.** The application's service
  owns the clients' listener, its routes and every HTTP and browser
  guard. It serves its own pages and bundle, through skein's serving
  contract (skein's `ui/web/bundle.md`), its own API, and whatever
  else it wants. It mounts jig's clients' component,
  `jig-protocol-engine-clients`, under a prefix of its choosing, and
  carries jig's messages to it and back. jig reserves no path
  (`protocol/service.md`, section 2).
- **The clients' component is transport-agnostic.** It takes jig's
  messages from the transport, each with the credential it came with as
  opaque bytes, or none, and the stream's opening and end; it answers
  with jig's messages and the stream's events. It takes a `Mint` from the
  application's server code, never from a client. It knows no HTTP,
  path, cookie, host or CORS (`protocol/client/README.md`, section 2).
- **jig's default HTTP mapping** puts the component's messages onto
  HTTP, at both ends: `jig-client-protocol-http` at the client's,
  `jig-protocol-engine-clients-http` at the engine's. Its routes sit
  under the prefix the application mounts it at; its stream is
  server-sent events; the credential travels as a cookie in a browser,
  and as a bearer token elsewhere. Its engine end carries example guards:
  a Host allow-list, a required header that forces a preflight with no
  CORS, the cookie's attributes, and a referrer policy on the printed
  link's route. The page's policy (its content security policy, frames)
  is the application's page's, of which jig's test application's server
  sends an example (`protocol/browser/serving.md`, 1.1). An application uses
  the mapping as it is, wraps it, or writes its own; replacing it means
  taking over what it held. jig's test kit carries attack stories (a
  cross-site request, a rebound Host, a framed page) an application runs
  against its own server, checked and never mandated
  (`protocol/browser/README.md`).
- **jig owns credentials; the application owns sign-in flows and
  guards.** jig mints a credential for an identity, verifies a presented
  one to its party, expires and revokes them, signing out included, and
  keeps their digests in the store's secrets (`protocol/store/secrets.md`
  ). Its own ways of signing in are two: the printed link, a
  one-time secret on loopback whose URL the application prints, and a
  service's credential (`domain/people/README.md`, section 3). Any other way is
  the application's sign-in: its own routes, redirects included, then a
  `Mint` of the clients' component for the identity it verified. The
  application decides how the credential travels. jig adds no "acting
  for": a bot relays a person's decision with that person's own
  credential, naming the revision it showed them.
- **One client, with no view.** `jig-client` is a step machine the
  application holds as a component, `jig_client::Client`, with `step`,
  `fire` and `next_deadline` (`client/domain/client/README.md`). It holds:
  - the link;
  - watches and reads by handle, and the latest document of each;
  - keyed requests, through skein-lib's request table;
  - the work kept across a restart, in a small store its host keeps;
  - the application's own families, as bytes under their owner's
    number, on the same connection.

  It emits jig's messages, never HTTP, and is built for the host and for
  wasm32. The application calls it and reads its documents; it never
  calls the application, and runs no loop of its own. Turns are decoded
  below its surface, by `jig-protocol-transcript`, so neither the
  application nor jig's helpers link smith. It serves three modes:
  long-lived, in a browser or a terminal whose host drives its loop; a
  service, the same with a service's credential and no presentation;
  and one-shot, which signs in, reads or sends, waits for the durable
  answer and exits, with its small store in a file
  (`client/domain/client/README.md`, section 16).
- **Plumbing and sugar.** jig's API is its primitives, as plumbing
  (`client/domain/contract/README.md`):
  - operations: create a batch of tasks, propose, withdraw, message a
    task, amend, cancel with a grace, release, stop, move a task to
    oneself, decide, answer, take, hand back, mark what it has read, and
    the project's own (notes, policy, roles, pools, services, adoption);
    several of them in one compound request, committed in one decision,
    all or none;
  - subjects and reads: `me`, a task, a tree, a run's text, a task's
    activity, a proposal, the inbox, task queries, chips, the deployment,
    and the application's own;
  - documents with their revisions, the acts the party may make (made,
    proposable or refused), and a decision's effect, as data.

  Sugar is built only from plumbing, in three forms: templates, data the
  client fills in; helpers, functions composing plumbing requests; facts
  and models, pure functions over documents. Templates are the
  application's configuration's, jig's chat and goal among its defaults,
  which a project's policy may override; the engine keeps them and
  serves them merged in `me`, and never expands or checks one
  (`protocol/configuration-templates.md`, 1). Whatever must be atomic, whatever
  only the engine can compute, and whatever the engine's own behaviour
  turns on, is plumbing, in general form and never shaped: so a goal, a
  tracked task with a priority, is plumbing, and a chat, a template, is
  sugar (`domain/model.md`, section 9).
- **jig's helpers are optional,** modules of `jig-client` on its public
  surface alone, as the application's code is: `shapes`, which fills
  jig's templates into requests and composes a chat's closing; `facts`,
  such as a chat's status; `thread`, a conversation's turns, words and
  provisional text in order; and `words`, jig's English for its enums, a
  phrasebook an application overrides phrase by phrase. A helper the
  application ignores costs it nothing. jig adopts sugar an application
  wrote when a second consumer uses it, or early when its interpretation
  is hard to keep consistent, and once it has settled
  (`client/domain/client/helpers.md`). Correctness (revisions, once,
  atomicity) is never sugar.
- **The reference UI is optional:** jig's pages, cards, frame,
  confirmation, drafts and view, in `jig-client-view`, on the public
  client alone and skein's UI kit, presented over the native shell,
  `jig-client-shell-native`, or jig's browser parts,
  `jig-client-shell-web` (`client/domain/pages.md`;
  `client/domain/view.md`). `ops` and jig's test application use it
  (examples.md, section 8). It proves the API is enough: it reads
  nothing an application cannot. An application takes it whole, takes a
  part of it, or leaves it.
- **Every consumer is the same client:** a browser, through wasm32; a
  terminal, on skein's text shell (skein's `ui/text.md`); a phone or a
  desktop application on Linux, macOS or Windows; a chat bot; a one-shot
  command; a service with no presentation at all.
  - **Only the shell and the transport are the platform's.** The client,
    its wire and link, the default mapping's client end and the
    reference UI are step crates, built for the host and for wasm32.
  - **A shell can sit on any HTTP stack.** The default mapping's client
    end gives the shell a method, a path, headers and a body, and cuts
    the stream's server-sent events into events itself. The browser's
    fetch, the operating system's own HTTP stack, or skein's HTTP client,
    whose io is Linux's, each make a small shell.
- **Clients that are not people.** A service acting through a client is
  a party of its own kind, with an identity and a role
  (`domain/people/README.md`, section 3).
- **The application's own traffic** rides the same client, the same
  request table and the same connection: its families' requests, reads
  and watches, each under its owner's number (`client/domain/root.md`,
  section 15), encoded and decoded by its own wire crate. Its scopes are
  a project, the deployment, and the party's own, which is admitted on
  the credential alone; its watches and reads carry windows; each
  owner's records are versioned apart, and a version not served is
  refused, naming the ranges served (`client/domain/contract/watches.md`, section 1, and `client/domain/contract/README.md`, section 6
  ). `jig-client` and the engine come from one pin of
  jig. Data its people follow as it comes is a stream of the
  application's (`client/domain/contract/watches.md`, 1.4).
- **The same rails as the engine's,** made for a client:
  - **the request table is the barrier:** every keyed request, jig's and
    the application's alike, leaves through skein-lib's request table,
    which has its host keep ordinary requests in the small store before
    sending, and retries under the same key until answered. Top-level
    `Seen` uses that table in memory only, is silently lost on closing,
    reload or crash, and is checked and sent again on the next opening
    (`client/domain/client/requests.md`);
  - **views are level-triggered:** a watch delivers its subject's current
    document whenever it changed, one delivery in flight, so nothing is
    queued and nothing missed;
  - **the owner keeps the value:** jig's documents carry jig's values,
    and name an application's by reference, as parts, which their owner
    shows (`domain/model.md`, section 5);
  - **the promises stay below the API,** whatever presents them: a
    decision names the revision it saw, and a moved revision is refused;
    an ordinary deciding request is made once, and every request is
    answered once durable; `Seen` advances a read position only forward
    without retaining an engine answer (`client/domain/contract/requests.md`,
    1); a document reflects only committed state, and leaves after the
    commit it
    reflects; admission and authority are the engine's; a credential is
    its party's.
- **What is the application's:** confirming a deciding act with its
  facts shown, from what jig gives as data in its documents (the
  reference UI keeps jig's confirmation); and keeping a credential from
  use across sites, its server's work, with jig's example guards and
  attack stories. jig's check, in its test kit and never mandated, finds
  in an application's worlds a decision naming a revision no document
  delivered to it carried, or a deciding act made on a tree that did not
  show its facts (`client/domain/testing.md`, 6.2).
- **The client's generic half** (keyed requests until durable,
  level-triggered watches, the link) is jig's while it has one user
  (`protocol/needs.md`, section 1).
