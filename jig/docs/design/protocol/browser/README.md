# The browser's layer

Provisional, 2026-10-14, revised 2026-10-22. What jig gives a browser
client, and where it stops: the application's server serves its pages
and its bundle by skein's serving contract (skein's `ui/web/bundle.md`, 4),
and jig serves no page, no bundle and reserves no path. jig gives the
default HTTP mapping's browser end over skein's browser parts, in
`jig-client-shell-web`, with the cookie that carries a browser's
credential; example guards in the mapping's engine end,
`jig-protocol-engine-clients-http`, which an application uses, wraps,
or writes its own in place of; attack stories in its test kit, checked,
never mandated; and jig's test application's browser shell,
`jig-client-test-shell-web`,
with its server and its page's policy, as the example of an
application's. `README.md`, section 18 says what jig promises and where its promise
stops. jig's messages, the clients' component and the mapping's routes
and framing are `../client/README.md`'s; the tree the parts map is skein's
(skein's `ui/README.md`); the reference UI is `../../client/domain/view.md`'s and
`../../client/domain/pages.md`'s. What is still open is listed in `README.md`, section 16
, and what is not the layer's, or comes later, in `README.md`, section 15.

- [`application.md`](application.md): what an application writes for its browser client.
- [`loop.md`](loop.md): the application's loop, elements, patches and scenes.
- [`sign-in.md`](sign-in.md): signing in and out in a browser.
- [`serving.md`](serving.md): the application's server, page policy and example guards.
- [`testing.md`](testing.md): the browser's trials and worlds.

## 1. In one page

- **The server and its URL space are the application's.** Its server
  serves its page and its bundle as skein's `ui/web/bundle.md`, 4 says, its
  own routes and its page's policy, and mounts jig's clients' component
  wherever it chooses (`../service.md`, section 2). jig serves no page and
  no bundle, and reserves no path (`README.md`, section 9, and `serving.md`, section 1).
- **The default HTTP mapping's browser end** is jig's, optional:
  `jig-client-shell-web`'s client adapter carries `jig-client`'s
  exchanges, its stream and its small store's records over skein's
  fetch, transport and small store, as `jig-client-protocol-http` maps
  them onto HTTP, adding the browser header to each request (`loop.md`, 1.1;
  section 10; `../client/default-http.md`).
- **A browser's credential is a cookie** the mapping's engine end sets,
  HTTP-only, so the client and the module never see it (`sign-in.md`, section 1; its
  attributes, `../client/default-http.md`, 3).
- **Example guards,** in `jig-protocol-engine-clients-http`: a Host
  allow-list; a required header that forces a preflight, with no CORS;
  the cookie's attributes; a referrer policy on the printed link's route.
  An application uses them, wraps them, or writes its own and takes over
  what they held (`serving.md`, 1.2).
- **The page's policy is the application's page's** (`serving.md`, 1.1): skein's
  strict policy is what its parts need; jig's test application's server
  sends a stricter one, as an example.
- **Attack stories** in jig's test kit (a cross-site request, a rebound
  Host, a framed page), which an application's browser tier may run
  against its own server: checked, never mandated (`testing.md`).
- **The printed link, opened in a browser,** is answered by the
  application's page at the mapping's link route; the shell reads its
  secret, takes it out of the address, and signs in (`sign-in.md`, section 1).
- **The reference:** jig's test application's browser shell,
  `jig-client-test-shell-web`, over skein's parts, jig's and the
  reference UI, with its server in `jig-test-service` and its page's
  policy: the example of an application's server, page policy and shell
  (`application.md`, 1.1; `serving.md`, section 1).
- **The rest is skein's:** the loop, the tree as elements, patches,
  scenes, history, the toolchain and the bundle, with what the reference
  UI adds to each (`README.md`, section 4, and `loop.md`, sections 1 to 4, and `README.md`, section 9).
- **jig's promise stops at the credential** (section 18): a request
  presented with a valid credential is its party's, and everything after
  is checked below the API. That a credential is not used across sites,
  and that a deciding act is confirmed with its facts shown, are the
  application's.

## 2. Where it sits

```
a person, a screen reader
  ▲ elements, focus, keys, the address bar, back and forward
the browser: fetch, session storage, history, cookies, clock, date, entropy
  ▲ the glue: wasm-bindgen, js-sys, web-sys
skein-ui-web                      skein's browser parts (skein's ui/web/README.md, 4.1)
jig-client-shell-web              jig's: the client's adapter, the printed link's sign-in, jig's look (5.1)
  ▲ plain data: reports, patches, the transport's requests, paths, the look
<application>-client-shell-web    its start and its loop: the turn (5.2)
  ▲ its client host's functions, called directly
its client host                   its client root holding jig_client::Client, its presentation, the mapping's client end (3.2)
  ▲ the client's requests as HTTP: exchanges, one stream; the small store's records
════════════════════════════════ one origin ═════════════════════════════════
its server                        its page, its bundle, its routes, its page's policy (section 12)
  ▲ the requests under the prefix it mounts jig's mapping at
jig-protocol-engine-clients-http  the default mapping's engine end, with the example guards (12.2)
  ▲ jig's messages, each with its credential as opaque bytes
jig-protocol-engine-clients       the clients' component (client.md)
```

- **A person** reads the page by sight or with a screen reader, and acts
  on it with a pointer and keys. **The browser** is any current one,
  which the parts target; the browser tier drives Chromium (`testing.md`).
- **`skein-ui-web`,** skein's crate: the browser parts and the default
  mapping of roles to elements, knowing no application (skein's
  `ui/web/README.md`, section 2). **`jig-client-shell-web`,** jig's crate,
  ordinary Rust, forbidding `unsafe`: jig's two parts and jig's look,
  over skein's parts, knowing no application. Each of its parts is
  usable alone: an application with its own presentation may take the
  client's adapter and nothing else.
- **The application's browser shell,** compiled to the module, is its
  start and its loop (`application.md`, 1.1, `loop.md`, section 1), with no trait between it and the
  parts or its client host. **Its client host** is its client root,
  which holds `jig-client` as a component, its presentation and its
  transport's client end, stepped as one machine (`application.md`, 1.2).
- **Its server** is the application's: skein's HTTP server over its
  clients' listener, or any other, serving the bundle by skein's
  contract and routing every request (`serving.md`, section 1). Under the prefix it
  mounts jig's mapping at, it hands each request to
  `jig-protocol-engine-clients-http`, which speaks jig's messages to the
  clients' component, `jig-protocol-engine-clients`, which knows no HTTP
  (`../client/README.md`, section 2; `../service.md`, section 2).

## 4. The toolchain and the bundle

skein's (skein's `ui/web/bundle.md`): the toolchain, skein's
bundler, `skein-ui-bundle`, the bundle, its page, its digest directory
and the starter. What is jig's follows.

### 4.1 The toolchain

- **skein's** (skein's `ui/web/bundle.md`, 1): `wasm32-unknown-unknown`,
  on the toolchain the repository's `rust-toolchain.toml` pins
  (`skein/docs/foundation/programming-model.md`, 10.1), and the glue
  crates pinned to exact versions in the
  workspace, the only `unsafe` the module runs
  (`skein/docs/foundation/programming-model.md`, 2.1); no wasm-pack, no npm, no binaryen.
  `jig-client-shell-web` forbids `unsafe`, as `skein-ui-web` does; an
  application's browser shell writes none of its own (`application.md`, 1.5, step 7).
- **One lock:** the workspace builds skein's bundler beside jig's test
  application's module, so the glue's generator is locked with the
  macros' version in its one `Cargo.lock`; an application's workspace
  does the same in its own.
- **jig's release profile** is `web`: it inherits `release`, with
  `opt-level = "s"`, `lto = "fat"`, `codegen-units = 1`,
  `panic = "abort"`, `overflow-checks = true` and `strip = true`. An
  application's workspace defines its own the same way.
- **The browser tier's module** is built with the dev profile, as code
  under test; the bundler is a tool, built optimized.

### 4.2 The bundler and the bundle

- **skein's bundler writes the bundle whole** (skein's `ui/web/bundle.md`,
  2). An application's build gives it its bundle's prefix, its own
  (`README.md`, section 9), and, where it takes the reference UI:
  - **`jig.css`,** jig's stylesheet, which ships with
    `jig-client-shell-web`, first of the given stylesheets, then the
    application's, `<name>.css` (section 13);
  - **jig's pictures,** which ship with `jig-client-shell-web`, as the
    owner `jig`'s, beside each of the application's owners' under its
    prefix, so no two owners' clash.
- **jig's test application's bundle,** under its prefix, `/bundle/`, as
  skein's example's is: `index.html`, and under `/bundle/<digest>/`
  `boot.js`, `<name>.js`, `<name>_bg.wasm`, `snippets/`,
  `skein-ui.css`, `jig.css`, `<name>.css`, `pictures/jig/`, a
  `pictures/<prefix>/` for its test family's owner, and `files/`.
- **jig's one text that is not Rust** is `jig.css`. Nothing inline.
- **Its size is a guide:** jig's test application's module, built with
  `web`, kept with jig's budgets; past it, what grew is reported.

### 4.3 The page and the starter

skein's (skein's `ui/web/bundle.md`, 3). jig's test application's page is
skein's bundler's, which names the digest directory by absolute path,
`/bundle/<digest>/`, since its server answers it at every path it does
not reserve, and at the printed link's route (`serving.md`, section 1); it loads
`skein-ui.css`, `jig.css` and then its own stylesheet, and its title is
the application's name.

## 9. Addresses and history

- **Every path is the application's:** its codec parses and prints every
  path the page shows, below its state, as total, bounded step code,
  since a URL is a person's input (skein's `ui/web/README.md`, section 7).
- **jig reserves none.** The application's server reserves its own: the
  prefix it mounts jig's mapping at, its bundle's prefix, and its own
  routes, none of which its codec prints as a place (`serving.md`, section 1). jig's
  test application reserves `/jig`, where it mounts the mapping, and
  `/bundle/`.
- **The reference UI's codec,** `jig-client-view`'s `address` module,
  gives jig's places their paths, at the root:

  | Address | Path |
  |---|---|
  | the inbox | `/inbox`; filtered by a project, `/inbox/<project>` |
  | conversations | `/conversations`; by a project, `/conversations/<project>` |
  | a task's overview | `/tasks/<task>` |
  | a task's other section | `/tasks/<task>/<section>`: `activity`, `plan`, `history`, `budget`, `panels` |
  | a project's overview | `/projects/<project>` |
  | a project's other section | `/projects/<project>/<section>`: `goals`, `tasks`, `notes`, `settings`, `resources`, `policy`, `roles`, `spend`, `recurring` |
  | the deployment | `/deployment` |

  An application takes it as is, under a prefix of its own
  (`/work/inbox`), with its own segment names, or writes its own. The
  table changes only by a change to this design; an application keeps
  its bookmarks by keeping its codec, which says which place `/` is: the
  reference's, at the root, the home. The places are the reference UI's
  (`../../client/domain/pages.md`, section 2).
- **In the reference's codec,** numbers are decimal, with no leading
  zero, within their type; one trailing slash and a query (an
  application's codec may read one) are ignored; any other path is a
  place with no page, shown as missing (`../../client/domain/pages.md`,
  section 8). The fragment is not reported.
- **Links' `href`s and history** are skein's parts' (skein's
  `ui/web/README.md`, section 7): a link node's binding resolves, without
  acting, to the address it leads to, which the client host prints as
  a path; an address a step shows is a new entry, or takes the current
  one's place, as the step says; back and forward are a place gone to,
  as a path; a path opened directly starts the application at it.
- **A page the browser kept** returns to its entry (skein's
  `ui/web/README.md`, section 7), and the reference shell takes the
  return as the page resumed as it was: the presentation hears no step
  and the page draws again. Nothing leaves the page and its handles hold
  (`../../client/domain/view-state.md`, section 2), so a request still
  pending on it is answered on the page that asked it, as for a person
  who stayed; a page loaded anew keeps no intent (section 4 there).

## 10. Transport and the small store

The default mapping's browser end: the client host's
`jig-client-protocol-http`, the mapping's client end (`../client/default-http.md`,
4), maps `jig-client`'s requests onto HTTP, and the client's adapter
gives them to skein's fetch, small store and transport as their plain
data (skein's `ui/web/effects.md`, section 1; `loop.md`, 1.1).

- **Exchanges** by skein's fetch: each of the client's exchanges as
  `jig-client-protocol-http` maps it, a POST of the message's record to
  its route under the prefix the application gives (`../client/default-http.md`, 1),
  with `Content-Type: application/octet-stream`, and the browser header,
  which the adapter adds (`sign-in.md`, section 1); the adapter sends it with
  same-origin credentials, so the cookie rides with it, and abandons it
  by aborting. A body is read up to the client's largest record; past
  it, the exchange failed. The response's status and body go back to
  the host, whose client end makes them a reply, and a failure, an
  abandon or a body cut short a loss.
- **The credential** rides in the cookie alone: the reference signs the
  client in by the transport (`sign-in.md`, section 1), so the client sends no
  credential with an exchange. An application whose page holds a
  credential in hand sends it as a bearer, and takes on what HTTP-only
  kept from its scripts (`README.md`, section 18).
- **The stream** by skein's fetch: a GET of the stream's route with the
  browser header, which the adapter adds, and same-origin credentials,
  its body read a piece at a time, each piece cut to the client's stream
  piece and handed to the host, whose client end cuts the server-sent
  events into messages (`../client/default-http.md`, 2). The next read is asked by
  the loop after the step (`loop.md`, 1.2), so the platform paces it. Its end is
  the stream gone.
- **Each load of the page is a client,** with a stream of its own
  (`../client/wire.md`, section 2): links and back and forward within the page's
  history keep it (`README.md`, section 9); a reload, an address typed or opened,
  and skein's restart part's reload (`loop.md`, section 1) start a new one, which
  sends its predecessor's stream gone, by the name the small store
  kept, before it opens its own. The browser ends the old page's
  connection when it likes, often after the new page's stream has
  opened.
- **The transport,** skein's, takes a step's requests whole, as the
  client's adapter gives them: the small store's writes first, then the
  exchanges and the stream, then a navigation, if any (`loop.md`, 1.3).
- **A hidden tab lets its stream go** a minute after it is hidden,
  skein's default, which jig's reference keeps (the application's
  parameter): fetch closes the stream and reports its end, which the
  client takes as any drop, and an opening while hidden waits until the
  tab is shown (skein's `ui/web/effects.md`, section 1).
- **The small store** is skein's, the tab's session storage: each of the
  client's records under its key (`../client/small-store.md`, section 1), the stream's
  name among them, both as base64 text, under the prefix the application
  gives, the reference's `jig.`; read whole at start. The reference UI's
  own records (its places and drafts) are its application's, kept beside
  the client's through the same transport (`../../client/domain/view-state.md`, 8).
  A write the platform refuses, its quota full, is a failed write. The
  restart part's key is outside the prefix (`loop.md`, section 1).
- **Closing a tab with stored kept requests** asks the platform to warn
  (`beforeunload`), through skein's title and closing part, only while
  its small store holds those records, judged by their sixteen-byte
  request keys. A top-level `Seen` has no stored record, so `Seen` alone
  does not warn; ordinary or mixed kept acts still do. Closing, reload
  or crash silently loses an unanswered `Seen`; the next opening checks
  and sends the forward-only watermark again (`../client/small-store.md`, section 1).

## 13. Stylesheets

The reference UI's, for an application that takes it; skein's
otherwise (skein's `ui/web/README.md`, section 10).

- **`skein-ui.css`,** which skein's bundler writes first in every
  bundle, holds the generic rules (skein's `ui/web/README.md`, section 10). jig
  has no class of its own for acts.
- **Every style token and region name has a name** (skein's `ui/README.md`,
  section 8). jig's start with `jig-`, its prefix and a hyphen: its
  tokens' names are `../../client/domain/view.md`'s, section 13, and its
  regions' `../../client/domain/pages.md`'s, 12.4. An application's start
  with its owners' prefixes, which its shell gives the parts as the
  look (`application.md`, 1.3).
- **jig's stylesheet,** `jig.css`, ships with `jig-client-shell-web` and
  loads after `skein-ui.css`. It places jig's regions by `data-region`,
  and the landmarks of the reference UI's default arrangement: the
  banner at the top; the navigation beside the main on a wide screen
  and above it on a narrow one; asides beside the main; the composer
  pinned below its conversation. It styles jig's tokens, in jig's
  palette, light or dark as the platform prefers.
- **jig's palette:** custom properties an application's stylesheet uses:
  `--jig-text`, `--jig-background`, `--jig-muted`, `--jig-accent`,
  `--jig-warning`, `--jig-danger`, `--jig-success`, and six series
  colours for drawings.
- **The application's stylesheet** loads after jig's, styles its own
  tokens and regions, and may restyle jig's (its own page: `application.md`, 1.5, step 4).
- **No style is a fact** (skein's `ui/README.md`, section 8;
  `../../client/domain/view.md`, section 13).

## 14. Limits

- **The client's:** an exchange's answer, at most its largest record; a
  piece of the stream; exchanges in flight (`../client/README.md`, section 10),
  which the client's adapter gives fetch.
- **The mapping's engine end's:** a request's bytes; calls in flight,
  and streams among them, in all, never per sign-in (`../client/wire.md`, section 2, and `../client/README.md`, section 10
  ); its numbers in `jig-protocol-config`'s profiles
  (`../limits.md`).
- **The tree's:** `skein_ui::Limits` (skein's `ui/README.md`, section 13), whose
  numbers the reference UI's constructors derive from the client's
  limits (`../../client/domain/view.md`, section 10); a drawing's SVG has
  at most a scene's marks (skein's `ui/web/README.md`, section 11).
- **The platform's:** session storage's quota, past which a write fails.
- **The parts' own,** skein's (skein's `ui/web/README.md`, section 11): one
  timer; one read of the stream outstanding; the inbox's reports raised
  during a turn, only those the turn's own calls raise; three reloads in
  a minute, by default, which jig's reference keeps.
- **The bundle's size,** a guide (4.2).

## 15. What is not the layer's

- **Following data as it comes** (a log tailed, a metric's latest
  points) is an application's streams (`../../client/domain/contract/watches.md`,
  1.4): a contract and engine piece, carried by skein's parts and the
  client as any watch, with no change to them.
- **Graphics beyond a scene** (a canvas of hundreds of thousands of
  points, three dimensions, video) are out of jig's scope: a component
  of the application's in a browser (`loop.md`, 2.1; `application.md`, 1.5, step 6), its own shell
  elsewhere (skein's `ui/README.md`, section 10; `../../client/domain/view.md`,
  section 11).
- **Later,** each adding without changing what `application.md`, section 1, and `README.md`, section 4, and `loop.md`, sections 1 to 4, and `README.md`, sections 9 to 10, and `sign-in.md`, section 1, and `serving.md`, section 1, and `README.md`, sections 13 to 14 and `testing.md` settle:
  - *jig's own drawing kinds,* a chart every application shares: a kind
    moves into the reference UI when a second application draws the
    same;
  - *tabs of one person sharing one stream, and HTTP/2* (`../client/README.md`,
    section 12);
  - *the bundle served compressed:* skein's (skein's `ui/web/README.md`,
    section 12), and its server's;
  - *the browser tier in other browsers than Chromium,* which comes to
    skein's harness first (skein's `testing/browser.md`, section 11; skein's
    `ui/web/README.md`, section 13); *a press that copies text, downloads and
    uploads of files:* effects (skein's `ui/README.md`, section 11), their parts
    skein's (skein's `ui/web/effects.md`, section 2, and `ui/web/README.md`, section 12), for an application
    whose people need them;
  - *`ops`'s browser shell,* `jig-ops-client-shell-web`, and its server:
    part of `ops` standalone (`../../examples.md`, 9.1), which gives it an
    engine.

## 16. Open questions

- **Approvals at a public name:** the code typed rather than carried in
  the address, the ask's origin shown, and a person's list of sign-ins
  with each ended from there (`../client/README.md`, section 12). The loopback
  approval uses the application's page under the existing cookie guards.

- **A failed printed link with a live cookie** (11): the link's reason
  remains the client's sign-in outcome even when the cookie can still
  sign in on a reload. A fallback is the client's question
  (`../../client/domain/client/README.md`, section 15). Default: no fallback.
- **Right-to-left languages:** what they ask of `jig.css` and the
  reference UI's phrasebook, beyond `<html lang>`
  (`../../client/domain/view.md`, section 14); what they ask of the
  elements and scenes is skein's question (skein's `ui/README.md`, section 16).
- **Media from the origin:** the bundle's closed list (skein's
  `ui/web/bundle.md`, 4) has no audio or video type, and skein's vocabulary
  no media (skein's `ui/tree.md`, 2.2): what an application serving its own
  would need of both.
- **A JavaScript snippet in jig's test application,** the user's
  decision; by default, none. skein's example carries the snippet that
  checks that the bundler and the strict policy carry an application's
  JavaScript (skein's `ui/web/testing.md`, 3). One in jig's would check the
  same under the reference policy, at the cost of a component and a
  snippet jig's test application has no other use for.

## 17. What it owes and is owed

- **The reference UI** (`../../client/domain/view.md`,
  `../../client/domain/pages.md`) owes: jig's part of the look, the names
  of jig's style tokens and region names under jig's prefix (section
  13), and jig's owner's number as skein-ui's opaque owner (`application.md`, section 1);
  a link node's address, without acting; the parts' own phrases among
  jig's (`loop.md`, section 1); jig's confirmation's cancel button naming Escape
  (`loop.md`, section 3); and its codec, `jig-client-view`'s `address` module,
  parsing and printing jig's places, total and bounded, at the root,
  under a prefix, or with other segment names (`README.md`, section 9).
- **`jig-client`** (`../../client/domain/client/README.md`) owes its transport
  requests as plain data, in the order it gives them (an exchange with
  its credential or none, the stream opened and closed, records saved
  and erased), and takes its transport's events (a reply, a loss, a
  message streamed, the stream gone, records stored or not); a sign-in
  by link and by the transport; its pending requests and its next
  deadline, read by the client host.
- **`jig-client-protocol-http`** (`../client/default-http.md`, 4) owes: a message
  as a method, a path under the prefix the application gives, headers
  and a body; the stream's request; the stream's server-sent events cut
  into messages; the bound on an answer, from the client's limits.
- **`jig-protocol-engine-clients-http`** owes: the example guards of
  `serving.md`, 1.2, with no switch, every POST's content type among them; the
  cookie set at a browser's sign-in, an approval's take among them,
  with no credential in the body, and cleared at its sign-out; each
  exchange told to the component as a browser's when its credential
  came by the cookie, and only then (`../client/default-http.md`, 3); a function that
  sets the same cookie for a credential the application minted; a GET
  of the link's route left to the application's page, with the referrer
  policy; its answers with `no-store` and `nosniff`; no `OPTIONS` and
  no CORS under its prefix (`sign-in.md`, section 1, and `serving.md`, sections 1 and 1.2).
- **`jig-protocol-engine-clients`** owes: the printed link's secret,
  given to the application to print, and a fresh one after a refused
  link; a credential minted on the application's word (`../client/sign-in.md`
  ).
- **An application's server** owes: its page and bundle as skein's
  `ui/web/bundle.md`, 4 says; its page's policy, at least skein's strict
  policy; jig's mapping mounted with its Host allow-list, or a transport
  of its own that takes over what the guards held; the path its own
  sign-in comes back to, checked (`sign-in.md`, section 1).
- **`../configuration.md`** names no bundle for jig: jig's test
  application's startup reads its bundle's directory from a member of
  its own (`../service.md`, section 7).
- **An application's browser shell** is owed skein's parts, jig's, and
  jig's reference to copy; driving fetch and the small store without
  the transport, it owes the transport's order (`loop.md`, 1.3).
- **skein owes:**
  - `skein-ui`, which the reference UI builds on: the tree (skein's
    `ui/tree.md`, section 1), owners as opaque numbers (skein's `ui/tree.md`, 1.3),
    the look (skein's `ui/README.md`, section 8), and Escape as a dialog's
    shortcut (skein's `ui/README.md`, 6.1);
  - `skein-ui-text`, the text shell an application's terminal composes
    (skein's `ui/text.md`);
  - `skein-ui-web`: the parts and the default mapping (skein's
    `ui/web/README.md`, 4.1 and `skein/docs/design/ui/web/elements.md`); `wasm-bindgen`'s start attribute
    re-exported and the glue pinned (skein's `ui/web/README.md`, 3.1, and `skein/docs/design/ui/web/bundle.md`, 1);
    fetch taking the headers, the body and the bound on an answer an
    application gives, for exchanges and the stream; the small store's
    prefix, the restart part's limit and the hidden tab's delay as the
    application's parameters;
  - `skein-ui-bundle`: the bundle under a prefix the application gives,
    its given stylesheets after `skein-ui.css` in the order given, each
    owner's pictures under its prefix, an application's page and files,
    and the closed list of types (skein's `ui/web/bundle.md`, 2 to 4);
  - its UI example's server, keeping the serving contract in code to
    copy (skein's `examples.md`, 9.3);
  - `skein-fake-person` and its browser face and harness,
    `skein-fake-person-browser`, with skein's stories for a browser
    shell (skein's `ui/testing.md`, 1.1; skein's `ui/web/testing.md`, 2): the harness
    opening an address the binary prints, found by a prefix the test
    gives, or one the test gives; stopping the binary and starting it
    again within a story; and failing a test only on trouble its story
    did not expect, which jig's attack stories use as it stands (`testing.md`
    );
  - the wasm32 builds: lib, `skein-codec`, `skein-http`, `skein-ui` and
    `skein-ui-web` built for wasm32 and checked by its gate (skein's
    `runtime/lib.md`, section 2; skein's `testing/layout.md`, section 1); a client
    compiled to WebAssembly in its model, with no kernel and no ring,
    the glue crates its only `unsafe`, a panic a trap the restart part
    reloads from (`skein/docs/foundation/programming-model.md`, 2.1).
- **smith owes** `smith-transcript` building for wasm32, which jig's
  gate checks through `jig-protocol-transcript`.

## 18. What jig promises, and where its promise stops

- **jig's promise** is `../../client-stack.md`, section 1's: a request
  presented with a valid credential is that credential's party's, and
  everything after is checked below the API
  (`../../client/domain/contract/documents.md`, 2); jig mints, verifies, expires
  and revokes the credential (`../client/sign-in.md`).
- **Where it stops,** as `../../client-stack.md`, section 1 says what is the
  application's:
  - *a credential not used across sites* is the application's server's
    work, as keeping it out of its logs is: jig gives example guards
    (`serving.md`, 1.2) and attack stories (`testing.md`), to use if they suit;
  - *a deciding act confirmed, with its facts shown,* is the
    application's, with jig's check to run if it suits
    (`../../client/domain/testing.md`, 6.2);
  - *one application per origin* is each deployment's: session storage,
    `'self'` in the policy and the cookie are per origin, so two
    applications on one origin would share the small store's storage,
    each other's scripts and the sign-in, and either could act as the
    person in the other. An origin each, by host name or port.

What jig's parts hold whole, whatever value an application gives them,
with the reason; an application that writes its own in the part's place
(`application.md`, 1.5, steps 9 and 10) holds the rule itself. What each guard of `serving.md`, 1.2
stops is this table's. The rules skein's parts and bundle hold are
skein's, with their reasons (skein's `ui/web/README.md`, section 14): text
never read as markup; scripts and styles from the origin only, nothing
inline; the bundle's closed list of types; an element override keeping
its node's accessible role and name; the transport's and the inbox's
invariants, on which the client's "once" rests (the table's last row).

| Held | Why | Instead |
|---|---|---|
| the `Host` allow-list (`serving.md`, 1.2) | DNS rebinding: a page of another site, its name rebound to the loopback address, would reach the engine as its own origin; answered `421`, it reaches nothing. | the list it gives: its listener's addresses, and the public name of the TLS proxy in front of it |
| the required header, the content type and no CORS (`serving.md`, 1.2) | Cross-origin requests: a page of another origin can neither set the header nor send a body of the mapping's type (either would need a preflight the mapping never answers, and no form can send that type) nor, from another site, send the cookie (same-site strict), so it can neither act as the person nor sign them in by the link. Same-site is not same-origin: on one person's machine, a local server on another port of the same host is the same site, which same-site strict lets send the cookie, and only the header stops it. | its own transport, holding what they held |
| the cookie's attributes (`../client/default-http.md`, 3) | HTTP-only keeps the credential from every script, the module's too; same-site strict, from requests another site starts; the `Path`, from every request but the mapping's; `Secure`, from plaintext: set on an exchange that came by the public name, through a TLS proxy, and left off on one that came by the listener's own address, plaintext on loopback, since not every browser keeps a secure cookie set over plaintext, loopback included. | its own sign-in, which sets the same cookie through the mapping's function |
| the referrer policy at the link's route (`serving.md`, 1.2) | The page opened at the link's route holds the sign-in's secret in its address until the sign-in part rewrites it; a policy sending a path to another origin would send the secret with a first request to an origin the page's policy names. | `strict-origin`, the origin alone |
| an approval read and decided under the cookie alone (`../client/sign-in.md` and `../client/default-http.md`, section 3) | A program holding a person's credential as a bearer, an agent driving the application's command line as that person among them, could otherwise approve a client of its own, which would then hold a sign-in of the person's. A browser's cookie no script and no program of the person's reaches through the mapping. | its own transport, telling the component a browser's exchange by the same rule |
| the link's route safe to GET (`sign-in.md`, section 1) | A GET is the one request any other site can make the person's browser send (a link, an image), with no header of its own and past every guard; one that signed in, or drew a fresh link on a refusal (`../client/sign-in.md`), would let any page the person visits spend the printed link or keep drawing new ones, until the link printed never works. | its own page at its own route, as safe |
| the secret taken out of the address first (`sign-in.md`, section 1) | Left in the address, the secret stays in the tab's history and in what the page shows or shares, after it is spent and before. Taken out first, it leaves the address bar and the tab's history; the browser's record of visited addresses keeps it, spent once signed in. | its own page at its own sign-in route |
| the credential never kept (`sign-in.md`, section 1) | A service's credential acts as the service; kept in a tab, it would outlive its dialog, in storage any script of the origin reads and in the session's restore. | the person copies it from the dialog, which the application's stylesheet may restyle |
| a step's client requests through the transport (`loop.md`, 1.3) | A request that leaves before it is kept is lost or made twice after a reload. It is the client's "once" (`../../client/domain/client/requests.md`), which jig owns (`../../client-stack.md`, section 1), and which skein's transport holds for any application (skein's `ui/web/README.md`, section 14). | its own order for the rest of the turn (`loop.md`, 1.2); the transport replaced, the order taken with it and checked by skein's stories for a browser shell and jig's (`testing.md`) |
