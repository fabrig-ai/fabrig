# skein's network protocols

Provisional, 2026-11-11.

The network documents cover TLS, the socket and optional TLS held by a
connection, HTTP and server-sent events, and framed application channels.
Read the transport pieces before the protocols stacked on streams:

- [`tls.md`](tls.md), `skein-tls`: a TLS client stream machine over rustls's unbuffered connection, with ciphertext below and plaintext above.
- [`link.md`](link.md), `skein-link`: one socket and its optional TLS, the handshake, body upload and bounded close.
- [`http/README.md`](http/README.md), `skein-http`: HTTP/1.1 client and server machines, and server-sent event readers and writers.
- [`channel.md`](channel.md), `skein-channel`: framed application channels over streams, with a version-agreeing opening, bounds and flow control.
