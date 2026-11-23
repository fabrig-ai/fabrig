# The browser's layer: the application's server

## 1. Served by the application's server

jig serves nothing from an origin: the application's server serves it
all, from the clients' listener it owns (`../service.md`, section 2).

- **Its page and its bundle,** as skein's `ui/web/bundle.md`, 4 says a bundle
  is served: under its bundle's prefix, the digest directory immutable;
  its page at every GET of a path it does not reserve, revalidated by
  its `ETag`; its root files; each file's type from the bundle's closed
  list; `X-Content-Type-Options: nosniff` on every answer. skein's
  example's server keeps that contract in code an application may copy
  (skein's `examples.md`, 9.3), as jig's test application's does.
- **jig's mapping, mounted at a prefix of its own,** every path under it
  the mapping's (`../client/default-http.md`, 1), but a GET of the link's route,
  which the mapping leaves to the page (`sign-in.md`, section 1). The mapping's
  answers carry `Cache-Control: no-store` and
  `X-Content-Type-Options: nosniff`; under its prefix it answers no
  `OPTIONS` and sends no `Access-Control-` header (1.2).
- **Or a transport of its own,** in place of the mapping, carrying jig's
  messages to the clients' component and back (`application.md`, 1.5, step 10).
- **Its own routes:** its API, its sign-ins (`sign-in.md`, section 1), anything else.
- **jig's test application's server,** in `jig-test-service`, is the
  example (`../service.md`, section 7).
- **One application per origin** is the deployment's to keep (`README.md`, section 18
  ).

### 1.1 The policy

The page's policy is the application's page's, sent by its server on
every answer.

- **Its floor is skein's strict policy,** what skein's parts and the
  bundle need (skein's `ui/web/bundle.md`, 4): scripts and styles from the
  origin only, nothing inline, WebAssembly compiled only by the
  origin's code (`'wasm-unsafe-eval'`); `script-src` and `style-src`
  take nothing more (skein's `ui/web/README.md`, section 14).
- **The reference's,** which jig's test application's server sends, is
  skein's starting policy (skein's `ui/web/bundle.md`, 4), with
  `X-Content-Type-Options: nosniff`, `Cross-Origin-Opener-Policy:
  same-origin`, `Cross-Origin-Resource-Policy: same-origin` and
  `Referrer-Policy: no-referrer`. An application copies it, and adds
  what its own data needs: named `https` origins for images, fonts,
  media, connections and other sites' pages, never a wildcard, a scheme
  alone or `'unsafe-*'`.
- **Frames:** `frame-ancestors 'none'` on every answer of the origin's,
  in the reference, so no other site frames a page that decides; an
  application that frames its own pages names its own origin there.
- **The referrer:** at the printed link's route and at the link's page
  (`sign-in.md`, section 1), the page's address holds
  the secret until the sign-in part rewrites it, so its answer sends a
  policy that never sends a path to another origin (1.2); the page's
  own `referrer` meta or attributes set no wider one, which the header
  cannot overrule.

### 1.2 The example guards

The guards that protect a browser's credential belong to the transport,
not to jig's model, so they are the application's server's. jig ships
them as example code in its mapping's engine end,
`jig-protocol-engine-clients-http`, on every request under the prefix,
before any reaches the clients' component. What each stops is `README.md`, section 18
's.

| Guard | What the example does |
|---|---|
| a Host allow-list | any `Host` off the list answered `421`, with nothing done; the list is the listener's addresses, which the application gives once it listens, and the public name it gives when it mounts the mapping, if any (`../client/default-http.md`, 3); the reference's is the address its printed link names on loopback, and gives no public name |
| the required header, with no CORS | a request the cookie authenticates without `Jig-Client: browser` answered `403`, with nothing done; a POST whose `Content-Type` is not `application/octet-stream` answered `415`, with nothing done; no `OPTIONS` answered and no `Access-Control-` header sent |
| the cookie's attributes | `../client/default-http.md`, 3's |
| a referrer policy on the printed link's route | `Referrer-Policy: no-referrer` added to the page the application answers at the link's route, and at its link page where it gives one (`sign-in.md`, section 1) |

- **The page's policy** (1.1) is the fifth guard, against injected
  scripts acting as the person and a decision tricked by a frame; it is
  the application's page's, and the reference shell's example of it.
- **No switch:** the mapping takes the allow-list and nothing that turns
  a guard off. An application uses the mapping as it is, wraps it (a
  check of its own before it, a header added to its answers), or writes
  its own transport; writing its own means taking over what the guards
  held, as replacing one of skein's browser parts does (skein's
  `ui/web/README.md`, 3.3), which jig's attack stories check, if it runs them
  (`testing.md`).
