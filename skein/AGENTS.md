# skein

The io and protocol kit, and the rules every crate in fabrig follows.

- **`skein/clippy.toml`:** `include_bytes!`, `copy_from_slice` and `Vec` only under a scoped `#[expect]` with its reason.
- **skein-tls** is also checked alone (`cargo check -p skein-tls`), since its features differ standalone.
- **rustls's debug assertions are off** in every dev build (`[profile.dev.package.rustls]`; `skein/docs/design/net/tls.md, 6`).
