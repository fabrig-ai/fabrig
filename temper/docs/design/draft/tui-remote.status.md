# tui-remote: status

2026-11-18. Where the design lags or departs from [tui-remote.md](tui-remote.md).

- **Design lags:**
  - 3: `../client/tui/README.md`, 1 and 2, take an `http://` address on loopback alone, and keep the address in 64 bytes; `../client/tui/pages/README.md`, 3, and `../client/tui/pages/chat.md`, 1, name no deployment by name.
  - 4: jig's `protocol/client/README.md`, 9, has the native shell speak plain HTTP to a socket address.
  - 5.1 and 5.2: `../client/README.md`, 4.1, opens the tab with its code for every ask, takes by polling alone, and names a tunnel.
  - 5.5: `../client/tui/pages/sign-in.md`, 1, keeps the paste and binds Enter to its field.
  - 6: `../client/tui/README.md`, 2, keeps one token, store and note, and offers the token at another address.
  - 7: `../client/tui/README.md` and `../client/tui/pages/README.md` have no words for a version not served.
- **Design departs:**
  - 5.3: "Sign in first", Google's way given, keeps the address's code in its `then` whatever the ask (`../client/pages/README.md`, 2, *The approval page*): signed out, the page can't read the ask's kind. A typed code's ask still needs its code typed, since the page drops the address's code when its read, after the sign-in, answers that kind. Decided by goal 8650's designer, 2026-11-18.
  - 5.3: the waiting ask's age is `../client/view.md`, 6's *ago*, as every page's, so an ask under a minute old says "Asked just now from 203.0.113.7" rather than the draft's "Asked 12 seconds ago" (`../client/pages/README.md`, 2, *The approval page*): the age is said, in the one form the pages share, with no wake each second. Decided by goal 8650's coordinator, 2026-11-19.
- **Open:**
  - 5.4: approval by the public name enabled by jig's one `approval` switch, with its guards, rather than a second switch. Pick: one switch. To reverse: a switch of its own, off by default.
  - 10, *Many sources asking*: a stranger with many addresses holding the four approvals, and one sharing a person's address holding its two, accepted for this pass. Pick: accepted. To reverse: room for asks by the public name apart from loopback's.
