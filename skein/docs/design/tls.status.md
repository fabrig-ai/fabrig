# tls: status

2026-10-29. Where skein's code lags or departs from [tls.md](tls.md).

- **Built:** 2, 3.5, 3.6: client edge checks and scenario ceilings without a heap sum. 4: PKCS#8 signing and `rs256_public`. 4: on the portable target, ring's JS entropy and TLS refused.
- **Not yet:** 1: server and client certificates. 3.6: native-face consumer. 5: real-loop TLS and fuzz target. 6: rustls close-notify debug assertion unresolved. 7: kernel TLS.
