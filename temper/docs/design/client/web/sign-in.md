# Signing in through the browser

Signing in through the browser is split from [the browser overview](README.md).

## 1. Signing in

- **The printed link,** at its page,
  `http://HOST:PORT/sign-in?secret=SECRET`, or the same link at the
  route, `http://HOST:PORT/jig/v1/sign-in/link?secret=SECRET`, as a
  build before printed it, opened in a
  browser on the engine's machine (`../sign-in.md`, section 1): temper's
  server answers either's `GET` with the page, which spends nothing (`serving.md`, 1); jig's sign-in part,
  given the link's route and its page, takes
  the secret at either and replaces the address with `/welcome`, temper's home for
  a sign-in, before anything is sent; the client signs in with it
  (`SignIn { with: Link { secret, held: None } }`), and the mapping's
  answer sets its cookie, which the client never sees (jig's
  `protocol/browser/sign-in.md`, section 1). The browser sends its cookie with
  the link's sign-in, as with any exchange: with a live link, a person
  still signed in keeps their sign-in, its number and expiry, and the answer sets no
  cookie, so the browser's stays as it was (jig's `domain/people/README.md`,
  section 3); a cookie the engine does not know, or one ended or
  expired, signs in afresh, with a new cookie. The welcome follows,
  which moves on to the chats page once the first run is over, and to
  the inbox, drawing nothing of itself, on a deployment with no welcome
  (`../pages/README.md`, The welcome).
- **A reload, or another tab,** signs in with `Transport`, by the cookie
  (jig's `client/domain/client/sign-in.md`, section 1).
- **Signed out,** by "Sign out" (`../line.md`, 6; jig's sign-out clears
  the cookie) or with no link: the main is the view's
  signed-out column (`../view.md`, 1), its heading "Signed out" and,
  in the browser's words, "To sign in, open a sign-in link temper
  printed." The view takes its shell's words at construction; the
  terminal's heading stands alone, with its own line (`../line.md`, 6).
  With a way of signing in configured, the main is the sign-in page
  instead: "Sign in" and "Continue with Google", the ways
  read from the page's `temper-sign-in` meta at start
  (`../../protocol/sign-in.md`, section 7); the link leaves for Google's
  sign-in and comes back to the page's address. Where the deployment
  has public projects, the signed-out main and the sign-in page add a
  link, "Read the public projects", to `/public` (`public.md`).
- **A link's sign-in not taken,** answered full, busy or not ready: the
  link is not spent, and the main says why and that the link stays
  usable until it expires (`../view.md`, 1), full with the deployment's
  room, its earliest end and the link's own. The retry is the person's: opening the
  printed link again sends its secret again, as the first open did, and
  signs in once there is room; opened past its expiry, it answers why
  (expired while remembered), and temper prints a fresh one if no link is
  live or reserved. The page never retries by itself, and keeps no secret:
  the address already says `/welcome`.
- **A used, expired or unknown link** draws the reason's paragraph in
  place of the sign-in page (`../view.md`, 1), with no automatic retry.
  A link forgotten by the engine, after a restart included, is unknown.
  A cookie still live does not override that paragraph; a reload uses
  the cookie instead (`README.md`, section 3).
- **A terminal's approval** (`../pages/README.md`, *The approval page*):
  the page at `/terminal` signs in with `Transport`, by the cookie, as a
  reload does. jig's cookie is `SameSite=Strict` under jig's prefix, so
  the tab's navigation, started by the terminal, carries none; the
  page's own exchanges, same-origin and started by temper's page,
  carry it, whatever opened the tab, as the `Authorize` place's do
  (`../../protocol/openid-provider.md`, 3). Its read and its decision go under
  jig's browser guards (jig's `protocol/browser/serving.md`, 1.2), and the
  mapping tells the engine they carried the cookie, so the engine
  takes them as a browser's (jig's `protocol/client/default-http.md`, 3); `temper
  ctl`'s bearer is refused (`../ctl/README.md`, section 7). Signed out,
  the page says "Sign in first". With Google's way given, as the
  sign-in page reads it, it offers "Continue with Google", whose `then`
  is the page's address with its code, so the sign-in comes back to the
  approval (the look request `outside-pages`). Without it, the printed
  link is the way in: the page says to open it first, then the page's
  address again, since the link's sign-in lands on `/welcome`, not back
  on the page (`../pages/README.md`, *The approval page*).
- **One person, on loopback,** as `../sign-in.md`, section 1 says. A browser
  on another machine is open (`../README.md`, section 5).

## 2. Open questions

- **A person's list of sign-ins,** each ended from there
  (`../../draft/tui.md`, section 7).

- A failed link with a live cookie: `../sign-in.md`, section 2.
