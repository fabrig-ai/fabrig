# tui-native-secret: the contracts between its tasks

The run's binding calls: what `local-link` makes and `serve-file` and `world-file` build on. The design is `jig/docs/design/protocol/client.md`, 4.2 and section 8, *The local link*.

## 1. The message

In `jig/crates/jig-protocol-engine-clients/src/boundary.rs`, beside `Link`:

```rust
pub enum Request {
    // ...
    Link { secret: Secret },
    /// The local link's one-time secret, each time one is drawn: kept for the
    /// person's own clients on the machine, never printed (protocol/client.md, 8).
    LocalLink { secret: Secret },
    // ...
}
```

- **At start,** where the printed link is enabled (`settings.link` is some), the component's first requests are, in order: `Link { secret }` (none when `link_secret` was given, as today), then `LocalLink { secret }`, then the public names' `Secrets(Get { owner: PublicNames })`. The printed link is drawn before the local link, so a seeded component's printed link is the one it draws today. Where the printed link isn't enabled, neither is said.
- **Later,** `LocalLink` is said each time the component draws the next local link: at once when one is spent, and when a sign-in finds none live (section 2).
- **The URL** of a local link is made exactly as the printed link's: in temper, `printed::link_public(listener, public, &secret)`, the function `print_link` uses; in jig's kits, `fixtures::printed(&secret)`. It is signed in by the same route, `/jig/v1/sign-in/link`, with the same `LinkSignIn` record.

## 2. What the component does with two links

Each of these is acceptance in `local-link` and holds by construction for the other two tasks:

1. Two links are live at most: the printed link and the local link, each one at a time, each live `link_span` from its drawing (the given printed secret's from its taking).
2. A link's sign-in compares its secret's digest, in constant time, with each live link's. Found and before its expiry, it reserves that link alone; the other stays live, and a sign-in by it goes on meanwhile. Reserved, the same secret is answered busy.
3. Not ready, busy and full give the reserved link back to its own place with its expiry; full carries that link's expiry.
4. Spent (signed in, a held sign-in kept, refused, signed out, the core's refusal, a lost credential check, each as the printed link is today): the link is remembered used, with which of the two it was. A spent local link is drawn again at once and said as `LocalLink`, in the phases where a reserved link is given back (`mint.rs`'s `return_link`: `Active` or `Stopped`); close and abort draw nothing.
5. A sign-in whose secret isn't live is answered `link_used`, `link_expired` or `link_unknown` from the past links, as today. It then draws a fresh printed link (`Link`) unless the printed link is live or reserved, or the secret is remembered as a local link's; and a fresh local link (`LocalLink`) unless the local link is live or reserved, whatever the secret.
6. Expiry: `fire` and a sign-in remember each expired live link expired, with which it was; `next_deadline` is the earliest of both links' expiries and the rest, as today.
7. The past links: the last 8 of both kinds together (`PAST_LINKS`), each its digest, used or expired, and which it was.
8. Close and abort drop both links and both reservations. A restart is a new component: it forgets both and says a new `Link` and `LocalLink`.

## 3. Who reads it

- **jig's kits** (`local-link`): `jig/testing/jig-engine-clients-kit/src/peer.rs`'s `Peer` keeps `local_links: Vec<Box<[u8]>>`, each `fixtures::printed(&secret)`, beside `links`, with

  ```rust
  #[must_use]
  pub fn local_link_count(&self) -> usize;
  #[must_use]
  pub fn local_link(&self) -> Box<[u8]>; // the last said; panics "local link" when none
  ```

  `jig-client-protocol-kit`'s engine, `jig-client-kit`'s joint component, `jig-test-service`'s clients and `jig/tests/ops`' joint drop it; `jig/tests/engine-clients/tests/suite/provider.rs` lists it beside `Link`.
- **temper's service** (`serve-file`): `temper-service`'s `Clients` keeps the secret until its listener is up, then pushes its URL on a queue of its own, `local_links`, never on `links`; `Service::local_links()` hands that queue to the binary, as `Service::links()` hands the printed links:

  ```rust
  /// Local sign-in links, taken by the binary to keep in the engine's link file, never printed.
  pub const fn local_links(&mut self) -> &mut Queue<Box<[u8]>>;
  ```

  The binary's host prints each of `links()` and keeps none; it keeps each of `local_links()` in the link file (`Host::keep_link`) and prints none. Serve's start writes nothing to the file itself.
- **temper's client world** (`world-file`): `temper/tests/client/src/component.rs` keeps `pub(crate) local_link: Option<Secret>`, set at each `LocalLink`; `front::Joint` gains

  ```rust
  #[must_use]
  pub fn local_link(&self) -> Option<Box<[u8]>>; // its URL, as latest_link makes the printed link's
  ```

  and `tui::File::Engine` reads `local_link()`, where it read `latest_link()`. `latest_link()` stays the printed link's.
- Until `serve-file` and `world-file` land, `local-link` gives temper's two readers an arm that drops the secret (its spec), so the trunk builds and the file still holds the printed link.
