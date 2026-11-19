# tui-remote: status

2026-11-18. Where the design lags or departs from [tui-remote.md](tui-remote.md).

- **Design lags:**
  - 3: `../client/tui/README.md`, 1 and 2, take an `http://` address on loopback alone, and keep the address in 64 bytes; `../client/tui/pages/README.md`, 3, and `../client/tui/pages/chat.md`, 1, name no deployment by name.
  - 4: jig's `protocol/client/README.md`, 9, has the native shell speak plain HTTP to a socket address.
  - 5.1 and 5.2: jig's `protocol/client/sign-in.md`, *A sign-in approved by a browser*, has no loopback return or grant; `../client/README.md`, 4.1, opens the tab with its code for every ask, takes by polling alone, and names a tunnel.
  - 5.3: `../client/pages/README.md`, 2, *The approval page*, and `../client/web/README.md`, 2.3, read the code from the address for every ask, have no loopback return, and say no source.
  - 5.4: jig's `protocol/client/sign-in.md`, *A sign-in approved by a browser*, jig's `protocol/configuration.md`, 3 and 8, and jig's `protocol/client/README.md`, 12, leave approval between hosts for later and keep no source or kind; jig's `protocol/client/default-http.md`, 3, gives the component no source address; `../protocol/sign-in.md`, 2.1, doesn't say the proxy sets `X-Forwarded-For`, and 2.3 calls the printed link the terminal's only way.
  - 5.5: `../client/tui/pages/sign-in.md`, 1, keeps the paste and binds Enter to its field.
  - 6: `../client/tui/README.md`, 2, keeps one token, store and note, and offers the token at another address.
  - 7: `../client/tui/README.md` and `../client/tui/pages/README.md` have no words for a version not served.
- **Design departs:** none.
- **Open:**
  - 5.4: approval by the public name enabled by jig's one `approval` switch, with its guards, rather than a second switch. Pick: one switch. To reverse: a switch of its own, off by default.
  - 10, *Many sources asking*: a stranger with many addresses holding the four approvals, and one sharing a person's address holding its two, accepted for this pass. Pick: accepted. To reverse: room for asks by the public name apart from loopback's.
