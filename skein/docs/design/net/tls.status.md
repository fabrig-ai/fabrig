# tls: status

2026-11-07. Where skein's code lags or departs from [tls.md](tls.md).

Unlisted sections have no known gap. Section 4's App public-half reader
shares signing's key parser and refuses without signing; keys refused
only at signing retain the public half the section specifies.

- **Not yet:** 4: skein-tls names `rustls`, without `std`, on every target, and six native crates name `rustls-std` ungated [in flight: change 2117].
- **Not yet:** 5: the fuzz target.
