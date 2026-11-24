# Serving the browser

Serving the browser's page and bundle is split from [the browser overview](README.md).

## 1. Serving

- **The routes of the clients' listener,** temper's, in this order,
  over jig's mapping (jig's `protocol/browser/serving.md`, section 1) and
  skein's serving of a bundle (skein's `ui/web/bundle.md`, 4):
  1. what jig's default mapping claims, under `/jig`, to jig's clients'
     component: every request there but a `GET` of the printed link's
     route, which the mapping leaves to the page, answered with the
     page's policy and `Referrer-Policy: no-referrer` (jig's
     `protocol/browser/serving.md`, 1.1 and 1.2);
  2. with no bundle, the configuration naming none (`null`) and the
     binary built without its own (`bundle.md`, 1), a `GET` of any other path but
     under `/bundle/` and `/robots.txt`, the printed link's route and
     its page, `/sign-in`, among them, answers `200` temper's page for no bundle, every other
     request not found, as `../engine.md` says. The page is
     fixed HTML in the binary, under the page's policy, with no script,
     style or picture: the level-1 heading "temper", then "This temper
     was built without its browser client.", "Open the link in a
     terminal with temper-box open DIRECTORY LINK." and "To serve the
     browser, build temper with the wasm32-unknown-unknown target, or
     name a bundle's directory in application.clients.bundle, and
     restart temper." (`../../protocol/setup/init.md`, 2);
  3. a request other than `GET`: `405`;
  4. `/bundle/<digest>/…`: the bundle's files, immutable, each in the
     form the request takes (skein's `ui/web/bundle.md`, 4); any other path
     under `/bundle/`: not found;
  5. `/robots.txt`, the bundle's one root file (skein's `ui/web/README.md`,
     3.3, step 5; jig's `protocol/browser/serving.md`, section 1), disallowing
     every path;
  6. `/sw.js` and `/manifest.webmanifest`, temper's two root files,
     fixed in its binary, `no-cache` (`push.md`, 1): the service worker,
     `Content-Type: text/javascript`, and the web app's manifest,
     `application/manifest+json`;
  7. any other `GET`, the printed link's route and its page,
     `/sign-in` (`../sign-in.md`, section 1), among them: the page,
     with its ETag and `no-cache`, and `304` on a matching
     `If-None-Match` (skein's `ui/web/bundle.md`, 4).
- **The page's metas,** made into the bundle's page at start, before
  its `</head>` and before its ETag is taken: `temper-sign-in`
  (`../../protocol/sign-in.md`, 7); then, when the command that serves
  was given `--record` (`../../protocol/binaries.md`), `<meta
  name="temper-record" content="on">`, so that every page the
  deployment serves records from its load (`shell.md`, 1), in each person's own
  browser, signed in or not; and the forges' `temper-forge-sign-in`
  metas, made once their records load
  (`../../protocol/openid-provider.md`, 7). Without `--record` the page
  has no `temper-record` meta. Nothing of a recording reaches the
  deployment.
- **The page's policy,** on every answer temper gives: jig's reference
  policy as it stands (jig's `protocol/browser/serving.md`, 1.1): `default-src 'none';
  script-src 'self' 'wasm-unsafe-eval'; style-src 'self'; img-src
  'self'; font-src 'self'; connect-src 'self'; base-uri 'none';
  form-action 'none'; frame-ancestors 'none'`, and temper's two
  sources beside it, `worker-src 'self'; manifest-src 'self'`, for its
  root files (`push.md`, 1), with `nosniff`,
  `Cross-Origin-Opener-Policy: same-origin`,
  `Cross-Origin-Resource-Policy: same-origin` and `Referrer-Policy:
  no-referrer`. skein-http's own refusals, `400`, `413` and `431`, given
  before temper sees the request, carry none: they hold no page to frame
  or sniff. temper adds no origin: no font, picture or script from
  elsewhere. The mockups' optional web font is not used; the look takes
  the system's monospaced font.
- **jig's example guards, as they are** (`../engine.md`): the
  Host allow-list holding the listener's own address; a cookie without
  jig's browser header refused.
- **Ordinary calls wait for jig's mapping's room** (jig's
  `protocol/client/default-http.md`, 4). Each waiting head stays under the listener's
  connection bound and stall deadline, with no body read until admitted;
  a closed connection drops its head. The small profile's six calls,
  four ordinary exchanges beside its two streams, therefore serve one
  page's concurrent requests in turn. Public reads retain their separate busy response (jig's
  `protocol/client/default-http.md`, 5); stopping refuses waiting ordinary calls.
- **The bundle** is skein's bundler's output (skein's `ui/web/bundle.md`
  ): the shell's module, skein's stylesheet and temper's
  (`look.md`). temper's build embeds it in the binary (`bundle.md`, 1);
  `scripts/bundle.sh` builds one apart, from the module built for
  wasm32, into a directory it prints. `application.clients.bundle` is
  `null` for the binary's own, or names a directory, whose bundle is
  served in place of the binary's (`../../protocol/configuration/engine.md`
  ). `temper-box serve` makes it, at start, whole, into memory,
  within the limits `clients.bundle_files` and `clients.bundle_bytes`
  (`../../protocol/configuration/limits.md`), and refuses the start
  past them, or when it is no bundle (its page and its digest's
  directory), by one code for both: a directory's files read, or the
  binary's, already in memory, copied (`bundle.md`, 1). **Both forms stay whole in
  memory,** a file and its `NAME.gz`: the service holds no inflater, and
  a client not taking gzip is rare but is answered without one, a
  `curl` among them. A `NAME.gz` with no `NAME` beside it refuses the
  start, as no bundle does; a directory without its forms (built with
  `--gzip off`) serves `NAME` alone. A new bundle is served
  after a restart; a tab on the old one is answered not found for its
  files, and a reload loads the new (skein's `ui/web/bundle.md`, 4). A page
  from the binary's own bundle always speaks its engine's records, so a
  client out of date (`../view.md`, 1) comes only from a binary
  replaced under an open tab, or from a directory configured, and its
  page asks the person to reload, which loads the engine's bundle.

## 2. Open questions

- **A switch to serve no browser** from a binary that has one, such as
  `clients.bundle` `false`. Default: none until a deployment asks: the
  browser is reached only by a printed link, as the terminal is.
