# tui-remote: status

2026-11-18. Where the design lags or departs from [tui-remote.md](tui-remote.md).

- **Design lags:**
  - 3: `../client/tui/README.md`, 1 and 2, take an `http://` address on loopback alone, and keep the address in 64 bytes; `../client/tui/pages/README.md`, 3, and `../client/tui/pages/chat.md`, 1, name no deployment by name.
  - 4: jig's `protocol/client/README.md`, 9, has the native shell speak plain HTTP to a socket address.
  - 5.1: `../client/README.md`, 4.1, and `../client/tui/pages/sign-in.md`, 1, open the tab with its code and keep the paste, and their lines name a tunnel.
  - 5.2: jig's `protocol/client/sign-in.md`, *A sign-in approved by a browser*, leaves approval between hosts for later, and keeps no source; jig's `protocol/client/default-http.md`, 3, gives the component no source address; `../protocol/sign-in.md`, 2.1, doesn't say the proxy sets `X-Forwarded-For`.
  - 5.3: `../client/pages/README.md`, 2, *The approval page*, and `../client/web/README.md`, 2.3, read the code from the address on every origin, and the ask waiting says no source.
  - 6: `../client/tui/README.md`, 2, keeps one token, store and note, and offers the token at another address.
  - 7: `../client/tui/README.md` and `../client/tui/pages/README.md` have no words for a version not served.
- **Design departs:** none.
- **Open:**
  - 5.2: approval by the public name enabled by jig's one `approval` switch, with its guards, rather than a second switch. Pick: one switch. To reverse: a switch of its own, off by default.
  - 10, *Many sources asking*: a stranger with many addresses holding the four approvals, accepted for this pass. Pick: accepted. To reverse: a cap for asks by the public name apart from loopback's.
