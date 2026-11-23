# The browser's layer: signing in and out

## 1. Signing in and out

The cookie is the mapping's, set by its engine end; the browser header
is added by the client's adapter (`loop.md`, 1.1); the engine end checks both
(`serving.md`, 1.2). The sign-in part is jig's (`loop.md`, 1.1), and the client's own exchanges
sign in and out (`../../client/domain/client/sign-in.md`, section 1; `../client/sign-in.md`
).

- **The cookie,** its attributes `../client/default-http.md`, 3's (why: README.md, section 18):
  every request under the prefix carries it; the module, the client and
  the application's client host never see it, since the client is
  signed in by the transport.
- **The browser header,** `Jig-Client: browser`, on every request of a
  browser under the prefix, the stream's among them: one the cookie
  authenticates without it is answered `403`, with nothing done (`serving.md`, 1.2).
- **The printed link, opened in a browser,** through the application's
  route:
  - the clients' component gives the secret, and the application's
    startup prints the URL: under the default mapping, the link's route
    under its prefix, the secret in its query,
    `<origin><prefix>/v1/sign-in/link?secret=…`, or, where the
    application gives the mapping a link page, the same link at that
    page, `<origin><page>?secret=…` (`../client/sign-in.md` and `../client/default-http.md`, section 1
    );
  - a GET of the link's route spends nothing and draws nothing: the
    mapping's engine end leaves it to the application's page, with the
    referrer policy (`serving.md`, 1.2), and the server answers its page there (why:
    `README.md`, section 18);
  - the same link at the application's link page, where it gives the
    mapping one (`../client/default-http.md`, 1), `<origin><page>?secret=…`,
    is the application's path, outside the prefix: its server answers
    the page there too, with the same referrer policy, and its GET
    spends nothing either;
  - the sign-in part reads the secret from the address's `secret`, at the
    link's route or at the link's page, given to it beside the route, and at
    once rewrites the address to the application's home, `/` in the
    reference (why: `README.md`, section 18). An address at either with no
    `secret` is no link. A `secret` that doesn't decode to 32 bytes is
    taken as an empty one, so that a made-up link takes the same path as
    any other link not this deployment's. The client host then signs the client
    in by the link, whose exchange is the link route's POST, the one a
    native client makes, with the browser header; the engine end answers
    it signed in, the credential in the cookie and none in the body;
  - answered why it can't sign in (`../client/sign-in.md`), the application
    prints a fresh link if none is live, and the client is signed out, its
    link's view saying which, for the application's words. Refused, not
    ready or lost, the client is signed out as well, which the reference UI
    shows as its sign-in page, saying to open the link printed; a link not
    spent still works opened again.
- **An application's sign-in** runs on its own routes (an OAuth or
  OpenID server, a company's directory, a chat platform's identity), as
  the application's server's code: it verifies the identity, asks the
  clients' component to mint a credential for it (`../client/sign-in.md`
  ), and sets the mapping's cookie with it, through the engine
  end's function for that, so the cookie is the one the mapping reads.
  In a browser it is a navigation, which skein's transport makes after
  the step's writes (`loop.md`, 1.3); the path it comes back to is its own to
  check, as one path of its own, never another site's (a `//` or `/\`
  read as one) nor one of its reserved prefixes.
- **Signing out** is the client's exchange to the sign-out route, which
  the engine end answers by clearing the cookie (`Max-Age=0`). Its
  answer is its sign-in's (`../../client/domain/client/sign-in.md`, section 1): each sign-in
  loads a new page (the printed link's, before the client starts, or an
  application's sign-in by navigation), so an earlier page's sign-out
  never answers into the client of a newer sign-in.
- **A service's credential, shown once,** in the reference: in the
  sign-in part's dialog of its own, outside the tree, as selected text,
  with a button closing it, in jig's words. The part never keeps it,
  nor does the client (`../client/sign-in.md`).
