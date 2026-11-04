# debt-10: contracts

The shapes that cross tasks, as this run's binding calls. `jig/docs/design/protocol/service.md`, 11 is the
design; this note is its surface, in code.

## `jig_protocol_engine_clients_http::bundle` (service.md, 11.4)

A new public module, `jig/crates/jig-protocol-engine-clients-http/src/bundle.rs`, `pub mod bundle;` in `lib.rs`. Step
code, no state, no root type, no trait, generic or callback. `http` is `skein_http::server`.

```rust
/// One digest-directory file, typed by the closed list (skein's ui-web.md, 4.4).
#[derive(Debug)]
pub struct File { pub kind: &'static [u8], pub bytes: Box<[u8]> }
/// A path's type by its extension, from the closed list; `None` off it.
#[must_use] pub fn kind(path: &[u8]) -> Option<&'static [u8]>;
/// A head's path, without its query.
#[must_use] pub fn target(head: &http::Call) -> &[u8];
/// The file `path` names: `/robots.txt`, or `/bundle/<digest>/<name>` under this digest.
#[must_use] pub fn file<'a>(files: &'a Map<Box<[u8]>, File>, digest: &[u8], path: &[u8]) -> Option<&'a File>;
/// Appends one header.
pub fn field(headers: &mut List<Header>, name: &[u8], value: &[u8]);
/// Asks the call's reply for room for the next chunk: at most `send`, at most `left`.
pub fn demand(server: &mut Server, call: Token, left: usize, send: u32, out: &mut Queue<Routed>);
/// Sends `body` from `offset`, at most `send` bytes, then finishes or demands; returns the offset reached.
pub fn chunk(server: &mut Server, call: Token, body: &[u8], offset: usize, send: u32, out: &mut Queue<Routed>) -> usize;
```

The services keep `Bundle`, `Reply`, `Answer`, `route`, and how a call starts and ends. Each service's bundle module
`pub use`s `File` and `kind`, so its readers (`jig/testing/jig-test/src/bundle.rs`, temper's startup) don't change.

`kind` is not skein's `skein_ui_bundle::types::of`: that crate is a build tool, ordinary Rust that links
`wasm-bindgen-cli-support`, and step code can't link it.

## `jig_domain_host::Undecoded` (service.md, 11.2)

`jig_domain_host::Invalid` is taken: it is the hub's refusal. So the one enum is:

```rust
/// What of the hub's start did not decode.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Undecoded {
    Charter,
    Transcript,
    CallName,
    /// A resumed delivery's evidence, which must decode and agree with its outcome.
    Evidence,
}
```

`route::decode` and `route::evidence` return it. `route::Invalid`, `temper_worker_domain::Invalid` and
`jig_test_worker::Invalid` go.

## The store's host (service.md, 11.5)

- `jig_protocol_store::host::Host` is renamed `Drivers`. `Hosted` keeps its name.
- `Hosted::mark_durable(&self) -> bool`, `#[must_use]`, is `self.component.mark_durable()`.

## Finish reads

`jig_protocol_accounts::host::Accounts::is_empty` is renamed `idle`, as `Hosted::idle` and `Bridge::idle` are. The
workers' `closed` keeps its name: `service.md`, 11.3 says what it holds, and it means more than idle.
