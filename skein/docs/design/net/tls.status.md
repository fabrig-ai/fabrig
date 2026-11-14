# tls: status

2026-11-14. Where skein's code lags or departs from [tls.md](tls.md).

Unlisted sections have no known gap. Section 4's App public-half reader
shares signing's key parser and refuses without signing; keys refused
only at signing retain the public half the section specifies.

- **Not yet:** 5: the fuzz target.
- **Not yet:** 5: the test of invalid limits also constructs the portable client and catches its assertion [in flight: change 2729]
- **Not yet:** 5: the machine's tests of the bugs it asserts, `#[should_panic]`, go with `temper/docs/design/draft/panics.execution.md`, P8.
