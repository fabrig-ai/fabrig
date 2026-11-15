# browser: status

2026-11-12. Where jig's code lags or departs from [README.md](README.md).

Section 11 has no known gap in taking a malformed or wrong-length
`secret` as an empty secret, removing it from the address and sending the
link exchange. Without `secret`, or at a different route, no link is taken.
The shell's table covers these cases; cookie fallback remains deferred in
section 16.

- **Not yet:** 9: the reference shell takes a kept page's return as the
  page resumed, with no step of its presentation's [in flight: change 3636]

