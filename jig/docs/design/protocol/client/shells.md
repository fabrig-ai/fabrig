# The client's protocol: shells

## 1. Shells

- **A shell is the application's:** its loop around `jig-client` and its
  presentation, its transport, its storage, its clock and its entropy,
  on its platform. `jig-client` runs no loop of its own, and never calls
  the application (`../../client/domain/client/README.md`, section 1).
- **jig's native shell,** `jig-client-shell-native`: skein's HTTP client
  on skein's io, with the default mapping's client end,
  `jig-client-protocol-http`, which cuts the stream's body into
  messages and a batch's into its parts, each piece handed on as it
  arrives (`default-http.md`, section 6); a file for the small store, and a private file for the
  credential, which it writes when its host gives it one to keep, or
  none, and reads back at start for its host to give as held (`sign-in.md`
  ); for Linux and for the worlds that make bytes real. With no
  presentation, it is a service's shell, and a one-shot command's.
  Its public `Transport` owns the socket loop and admission, including
  sockets still closing, for both this shell and a caller's shared io,
  with a caller-chosen owner tag.
- **Its engine, plain or over TLS:** the shell's settings name the
  engine one of two ways, `Settings.engine`:

  ```rust
  pub enum Engine {
      Plain(skein_io::Addr),                    // HTTP to a socket address, `Host` that address printed
      Tls(Tls),                                 // HTTPS to an engine by name
  }
  pub struct Tls {
      pub name: skein_tls::Name,                // the server's name: SNI and the certificate's check
      pub host: Box<[u8]>,                      // `Host`: NAME, or NAME:PORT where PORT isn't 443
      pub addresses: Box<[core::net::SocketAddr]>, // resolved by the host before the loop, in order, at least one
      pub config: skein_tls::Config,            // the roots the host gives, `http/1.1` offered by ALPN
  }
  impl Shell {
      pub fn refused(&self) -> Option<skein_tls::client::Error>;
  }
  ```

  - *Over TLS,* each connection is skein-io's TCP connection carried by
    skein's TLS client, `skein-tls`: the server's name `name`, `http/1.1`
    offered by ALPN, the certificate checked against `config`'s roots at
    the wall time the handshake starts. The host reads the roots, the
    machine's as skein-shell's `read_trust_roots` finds them
    (`SSL_CERT_FILE`, else the distribution's bundle), and resolves the
    name once, before the loop, as skein's shells resolve their
    configured peers (skein's `runtime/io/README.md`, section 3); the
    shell names no resolver and reads no file for it.
  - *The addresses* are tried in their order at each connection: one
    not reached, refused or reset before its handshake ends gives way to
    the next, and the connection fails, as any connection's failure,
    once none is left. Resolving while running is later.
  - *What runs above it* is as over plain HTTP: the exchanges, the
    stream, their limits, deadlines and backoff, the credential as
    `Authorization: Bearer` and no browser header. The TLS client's
    limits are the shell's `Limits`' own, checked by `valid` against
    io's and HTTP's.
  - *A handshake that fails,* a certificate refused, a name it doesn't
    name, or the protocol's own failure, fails the connection as any
    failure does, and the client tries again after its backoff, so a
    renewal under way heals by itself. The shell keeps that TLS error,
    `Shell::refused`, from the newest handshake that failed until one
    succeeds, for its host's words; a connection that fails below TLS
    leaves it as it was, and a `Plain` engine has none.
  - *Its tests,* the shell's own, over a TLS listener on skein's
    simulated io with skein's test certificates (`skein-tls-kit`) and no
    network: a certificate trusted and the stream through it, one that
    chains to no root given, one for another name, a first address not
    reached and the next taken, and `refused` cleared by a handshake
    that succeeds.
- **jig's browser parts,** `jig-client-shell-web`: the client's
  adapter, which carries the default mapping's calls over skein's
  fetch, the printed link's sign-in, and jig's look, over skein's
  `skein-ui-web` (`../browser/loop.md`, 1.1), for `ops` and jig's test
  application, whose browser shell is `jig-client-test-shell-web`
  (`../browser/application.md`, 1.1). An application's own browser shell uses them,
  skein's parts alone, or neither.
- **Its pages are served by the application's server,** through skein's
  serving contract (skein's `ui/web/bundle.md`), under its own
  policy: jig serves no page, no bundle, and reserves no path
  (`../service.md`, section 2; `../browser/serving.md`, section 1).
