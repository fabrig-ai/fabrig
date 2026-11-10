# A sign-in in a new tab

Draft, 2026-11-06, from a session with the user that day. Today a plan's
sign-in sends temper's own tab to the provider and back. The user asked
for a new tab instead: "opening a new tab let's the old one stay there in
case of errors I guess? also I see that something gets changed in the
client right after clicking on the signin link, and it feels almost like
a glitch." Offered the behaviour below, the user took it ("I'll go with
your recs"). The look request `sign-in-new-tab` draws the page
(`../client/journeys/sign-in-new-tab/`). This draft carries what the look
cannot. It covers a model account's sign-in alone: a forge's departures
stay as they are (section 6).

## 1. In one page

- **The press opens a tab,** at once, and the sign-in's address arrives in
  it (section 2).
- **temper's page stays and waits,** with the address as a link and the
  paste folded under it, and moves on by itself (section 3).
- **The new tab ends on one line,** "Sign-in received. You can close this
  tab." The outcome is the page's to say (section 4).
- **The departure goes,** and with it the glitch, the "come back" and the
  separate entry for a browser on another computer (section 5).

## 2. The press

- **The tab opens in the press's own turn,** before the sign-in is sent:
  a browser blocks as a pop-up a tab opened once an answer has come back.
  It says "Opening ChatGPT's sign-in…" until the address comes.
- **The answer points it.** On `AccountSignInStarted` the page sends the
  tab to the address, with its opener cut first, so that the provider's
  page has no hold on temper's.
- **A sign-in refused or lost at the press** closes the tab. The page says
  why, as today.
- **A browser that refuses the tab** leaves the page with the address as
  the person's own click: "Open ChatGPT's sign-in ›", the primary button.
  A browser that opens the tab and hides it is served by "Open it again".
- **A mechanism of skein's:** the web platform opens a tab with a line of
  words, points it at an address under the navigation's own target rule
  (`skein/docs/design/ui/web/README.md`, section 8), closes it, and reports a
  tab refused.
- **The terminal** prints the address, as today.

## 3. The page while it waits

- **Nothing leaves:** no `Return`, no `Request::Leave`. The page holds the
  sign-in from its answer, as a sign-in from another computer does today.
- **What it draws:** the look request's note, "Open it again ›" (the
  address as an external link), and the disclosure holding the paste
  (`../client/pages/README.md`).
- **It moves on by itself,** on the `accounts` watch: to Ready, on the
  welcome as on `ConnectModel`, or to the ending's words and acts, as a
  page come back does today.
- **A hidden tab lets its stream go** after a minute (the same section of
  `skein/docs/design/ui/web/README.md`), so a slow sign-in moves on when the
  person returns, its stream opened again. The page says nothing of this.
- **A reload while it waits** draws the same page: the page's address
  names the sign-in's source once it is answered (`../client/web/README.md`,
  2.3), replacing its history entry.

## 4. The new tab's end

- **The listener's fixed page,** with no return given, says "Sign-in
  received. You can close this tab."
- **Not "Signed in":** the listener answers before the exchange ends, and
  answers a provider's refusal just the same (`skein/docs/design/oauth/accounts.md`,
  5). The page in temper's tab says how it ended.
- **A browser on another computer** shows the provider's address that
  can't be reached in the new tab, which stays there for the copying, and
  temper's disclosure asks for it.

## 5. What goes

- **The departure, for a sign-in:** `Return { source, path }` where
  nothing else sends it, the root's `departing` and `returning`, and the
  page's "come back" as a separate story.
- **The entry for another computer:** "This browser isn't on temper's
  computer ›" and its way into the paste. Every sign-in now stays on the
  page with the paste offered, so that person takes the same path.
- **`PasteHistory`:** "Go forward to that page ...". The page stays in its
  tab.
- **A recording's end at a sign-in:** with no departure, a recording goes
  on through the sign-in and its end (`../client/web/journeys.md`, 3.1). What
  the shell did with the tab is among what the recording keeps and the
  replay checks (`../client/web/journeys.md`, 4.2).

## 6. Open questions

- **A forge's departures:** GitHub's App and install still leave in this
  tab ("In this tab, on GitHub"), and may want the same.
- **The new tab closing itself:** temper's page keeps a handle on the tab,
  but a provider's opener policy may sever it. Left out for now, so the
  person closes it.
