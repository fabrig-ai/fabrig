# browser: status

2026-11-18. Where jig's code lags or departs from [README.md](README.md).

- **Not yet:** 11: the sign-in part is given the link's route alone, and takes no secret at the application's link page [in flight: change 8721].

Section 11 has no known gap in taking a malformed or wrong-length
`secret` as an empty secret, removing it from the address and sending the
link exchange. Without `secret`, or at a different route, no link is taken.
The shell's table covers these cases; cookie fallback remains deferred in
section 16.

