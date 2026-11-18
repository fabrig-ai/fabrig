# tui-remote: status

2026-11-18. Where the design lags or departs from [tui-remote.md](tui-remote.md).

- **Design lags:**
  - 5.1: jig's `protocol/client/sign-in.md`, *A sign-in approved by a browser*, has no loopback return or grant; `../client/README.md`, 4.1, and `../client/tui/pages/sign-in.md`, 3, take by polling alone, with no listener on loopback, no one-click ask and no Enter for a typed code instead.
  - 5.3: `../client/pages/README.md`, 2, *The approval page*, and `../client/web/README.md`, 2.3, read the code from the address for every ask, have no loopback return, and say no source.
  - 5.4: jig's `protocol/client/sign-in.md`, *A sign-in approved by a browser*, jig's `protocol/configuration.md`, 3 and 8, and jig's `protocol/client/README.md`, 12, leave approval between hosts for later and keep no source or kind; jig's `protocol/client/default-http.md`, 3, gives the component no source address; `../protocol/sign-in.md`, 2.1, doesn't say the proxy sets `X-Forwarded-For`, and 2.3 calls the printed link the terminal's only way.
- **Design departs:** none.
- **Open:**
  - 5.4: approval by the public name enabled by jig's one `approval` switch, with its guards, rather than a second switch. Pick: one switch. To reverse: a switch of its own, off by default.
  - 10, *Many sources asking*: a stranger with many addresses holding the four approvals, and one sharing a person's address holding its two, accepted for this pass. Pick: accepted. To reverse: room for asks by the public name apart from loopback's.
