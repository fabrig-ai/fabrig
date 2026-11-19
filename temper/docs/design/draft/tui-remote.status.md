# tui-remote: status

2026-11-19. Where the design lags or departs from [tui-remote.md](tui-remote.md).

- **Design lags:**
  - 5.1: jig's `protocol/client/sign-in.md`, *A sign-in approved by a browser*, has no loopback return or grant; `../client/README.md`, 4.1, and `../client/tui/pages/sign-in.md`, 3, take by polling alone, with no listener on loopback, no one-click ask and no Enter for a typed code instead.
  - 5.3: `../client/pages/README.md`, 2, *The approval page*, and `../client/web/README.md`, 2.3, read the code from the address for every ask, have no loopback return, and say no source.
  - 5.4: jig's `protocol/client/sign-in.md`, *A sign-in approved by a browser*, jig's `protocol/configuration.md`, 3 and 8, and jig's `protocol/client/README.md`, 12, leave approval between hosts for later and keep no source or kind; jig's `protocol/client/default-http.md`, 3, gives the component no source address; `../protocol/sign-in.md`, 2.1, doesn't say the proxy sets `X-Forwarded-For`, and 2.3 calls the printed link the terminal's only way.
  - 7: signed out, a build the engine refuses shows `../client/tui/pages/refused.md`'s screen only once signed in: jig's `protocol/client/sign-in.md`, *A sign-in approved by a browser*, *Asked*, gives the approval ask's answer no case for a build not served, and `jig-client` reads the refusal as lost. Left for a follow-up once M2, which reshapes those messages, has landed (draft-4, 2026-11-18); rare while the laptop's build stays at the deployment's (7). The follow-up first checks whether `layouts-both-ways.md`'s client part, keeping the hello and the sign-in at layouts that don't move, already closes it: a client of another build then always refused by name or signed in.
- **Design departs:**
  - 5.5: "Enter is the screen's one key, with no field" is, at a name, a focused "Ask again" button under the lines that Enter or Space presses, standing once the approval has ended or wasn't asked; while an ask is in flight or a typed code's ask waits there is no button and no key does anything (`../client/tui/pages/sign-in.md`, 3). skein's UI kit has no region that acts on a key (skein's `ui/README.md`, 4.1), and its vocabulary isn't grown for this (draft-4, 2026-11-19). The one-click ask (5.1) puts "Type a code instead" in the same place while its ask waits.
- **Open:**
  - 5.4: approval by the public name enabled by jig's one `approval` switch, with its guards, rather than a second switch. Pick: one switch. To reverse: a switch of its own, off by default.
  - 10, *Many sources asking*: a stranger with many addresses holding the four approvals, and one sharing a person's address holding its two, accepted for this pass. Pick: accepted. To reverse: room for asks by the public name apart from loopback's.
