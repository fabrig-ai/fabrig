# tls: status

2026-11-07. Where skein's code lags or departs from [tls.md](tls.md).

Unlisted sections have no known gap. Section 4's App public-half reader
shares signing's key parser and refuses without signing; keys refused
only at signing retain the public half the section specifies.

- **Not yet:** 5: the fuzz target.
- **Not yet:** 4: `sign_es256`, `es256_public`, `verify_es256`,
  `seal_aes128gcm` and `seal_aes128gcm_from` (a push's signature and
  seal); 5: their tests.
