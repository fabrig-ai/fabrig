# browser: status

2026-11-17. Where jig's code lags or departs from [README.md](README.md).

Section 11 has no known gap in taking a malformed or wrong-length
`secret` as an empty secret, removing it from the address and sending the
link exchange. Without `secret`, or at a different route, no link is taken.
The shell's table covers these cases; cookie fallback remains deferred in
section 16.

- **Not yet:** 5.2: the client host settling its root once per report before the requests are carried [in flight: change 6345].
- **Not yet:** 5.2: a batch's answer read on after a piece [in flight: change 6354].
