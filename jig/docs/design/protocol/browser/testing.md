# Testing the browser layer

Each within jig's suites' budgets. skein's parts, its bundler, its face
and its harness are
tested in skein, and in a browser on skein's UI example (skein's
`ui/web/testing.md`); jig's tests take what is jig's.

- **On the host,** in jig's default suites:
  - the client's adapter's pure half: the host's requests as HTTP to
    the transport's plain data (the small store's writes first, the
    browser header on each exchange and on the stream's request, the
    bound on an answer, the stream's piece), fetch's and the small
    store's reports to the host's events, an answer past the bound
    failed;
  - the sign-in part's pure half: the secret read from the address, the
    address rewritten first, the credential's dialog's words;
  - the mapping's engine end, in `jig-protocol-engine-clients-http`'s
    step tests: each guard of `README.md`, 12.2 refusing with nothing reaching the
    component (a `Host` off the list, the header missing beside the
    cookie, a POST of another content type, an `OPTIONS`); the public
    name admitted beside the listener's address; the cookie's
    attributes by the listener's address and by the public name, set at
    a browser's sign-in, cleared at its sign-out, and set by the
    application's function after a `Minted`; a GET of the link's route left to the page
    with the referrer policy, spending nothing; an exchange told to the
    component as a browser's for a credential from the cookie alone, a
    bearer's never, beside the header or not (`../client/default-http.md`, 3);
  - the reference UI's codec: every place printed and parsed back, at
    the root, under a prefix, with other segment names; any bytes parsed
    to a place, never a panic.
- **The wasm32 builds in jig's gate:** `jig-client-shell-web`,
  `jig-client-test-shell-web` and every client crate, built for
  `wasm32-unknown-unknown` with clippy's lints as errors, since the
  parts' platform halves are compiled only there; skein's gate builds
  skein's own (`README.md`, section 17).
- **The browser tier:** a suite of its own, `--profile browser-jig`, run by
  `python3 -I -B tools/tiers/run jig browser` from the repository root.
  The runner builds the test application's bundle with skein's bundler
  and its native server. It stays outside jig's gate while the gate cannot
  build the wasm bundle and run Chromium; its CPU budget and wall hard
  limit are in `tools/tiers/budgets.toml`.
  Each test starts one server (the application's binary; for jig, the
  reference `jig-test`, serving its bundle and the mapping at `/jig`)
  and one Chromium, and plays its stories with a person each: a browser
  context, its own cookies and storage. A missing Chromium or bundle
  fails the suite, naming the command that provides it; it is never
  skipped. The bundle's directory reaches the tests in
  `JIG_TEST_BUNDLE`.
- **The harness and the face** are skein's, in
  `skein-fake-person-browser` (skein's `ui/web/testing.md`, 2; skein's
  `ui/README.md`, 14.1). jig's tier gives the harness the prefix of the line
  `jig-test` prints its link on, so the address it opens first is the
  printed link, which signs the person in (`README.md`, section 11); the face opens
  the paths the application's codec prints.
- **skein's stories for a browser shell** (skein's `ui/web/testing.md`, 2):
  jig's tier calls them with jig's binary, bundle and codec.
- **jig's stories for a browser shell,** in `jig-client-fake-person-browser`,
  on skein's face and harness: functions an application's tier calls
  with its binary, bundle and codec, to check its wiring of
  `jig-client`, the client's adapter and the printed link, since its
  loop is its own. Each signs its person in by the printed link:
  - a reload and engine restart keep a pending request, which the engine
    makes once;
  - a reload with nothing pending meets no close warning.
  Spent-link admission and the link's offline/live state are checked in
  the lower worlds.
- **jig's attack stories,** in the same kit: functions an application's
  tier may call with its binary, its page's address and its mapping's
  prefix, against its own server, whether it uses jig's mapping or its
  own transport. Checked, never mandated, as jig's check is
  (`../../client/domain/testing.md`, 6.2). Another origin's page is
  served by jig's kit, in the story's own loop, as the browser kit's
  facts are served (skein's `ui/web/testing.md`, 1); skein's harness opens it
  as an address the test gives, and the story names the policy
  violation it causes as trouble it expects (skein's `ui/web/testing.md`, 2):
  - *a cross-site request:* such a page, on another port of the same
    host and on another host name, makes the signed-in person's browser
    send a request under the prefix; it is refused, and nothing is done,
    which the person's page then shows;
  - *a rebound Host:* a request under the prefix naming a `Host` off
    the allow-list is answered `421`, and nothing is done;
  - *a framed page:* such a page frames the application's page; the
    browser shows nothing of it, under the page's policy.
- **jig's tier** calls skein's panic-at-start story, jig's two reload
  stories and three attack stories: the cross-site request from another
  port and from another host, and the framed page; Host admission is
  covered by the HTTP mapping's tests. It adds: a person opens the
  printed link and sees their inbox, under the reference policy, with
  no trouble.
- **jig's test application takes skein's bundler's page** (`README.md`, 4.3), and
  two of `README.md`, 3.5's steps once each: a root file (step 5); a prefix on the
  reference codec, in one story (step 3). An element override, a
  component and a JavaScript snippet are skein's example's (skein's
  `ui/web/testing.md`, 3); jig's test application carries no snippet by
  default (`README.md`, section 16).
- **What an application adds:** its tier, over skein's harness and face:
  skein's stories and jig's called, jig's attack stories if it runs
  them, and its own pages, drawings, components and sign-ins.
