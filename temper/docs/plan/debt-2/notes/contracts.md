# debt-2: contracts

The shapes that cross this plan's tasks, as binding calls. A task builds its half to this
note; the task that creates a shape sketches it, the others read it on the trunk.

## jig D018: the client's requests by mode (`jig/docs/design/client/domain/client.md`, 4.4)

`jig-client` (`jig/crates/jig-client/src/client.rs`, exported from its `lib.rs`), created by
`jig-client-unlinked-requests`:

```rust
/// What an unlinked client asks of its host (client/domain/client.md, 4.4).
#[derive(Debug, PartialEq, Eq)]
pub enum UnlinkedRequest {
    Told { told: Told },
    Authorized(crate::Authorized),
    Down { down: Down },
    SignOut,
}

impl Client {
    pub fn step_unlinked(&mut self, now: Time, wall: Wall, event: Event, out: &mut Queue<UnlinkedRequest>);
    pub fn fire_unlinked(&mut self, now: Time, wall: Wall, out: &mut Queue<UnlinkedRequest>);
}
```

- `step_unlinked` and `fire_unlinked` given a linked client are a fault (an assertion).
- Until `jig-linked-requests` lands, `step` and `fire` keep serving both modes as today, and
  `Request` keeps `Down` and `SignOut`, so the callers move in their own tasks without a
  broken trunk between. `jig-linked-requests` then removes `Request::Down` and
  `Request::SignOut`, and makes `step` and `fire` given an unlinked client a fault.
- Every other name (`Request`'s other variants, `Event`, `Down`, `Told`, `Authorized`) is
  unchanged.

**An application root** that steps a client in either mode (jig's
`jig/testing/jig-client-test-domain` and `jig/examples/ops/client-domain`, temper's
`temper-client-domain` and `temper-client-tools`'s `Root`) keeps its `Client { request:
jig_client::Request }` for the linked mode and gains one variant for the unlinked one:

```rust
pub enum Request {
    Client { request: jig_client::Request },
    /// What the unlinked client asks, which its host carries (jig's client.md, 4.4).
    Unlinked { request: jig_client::UnlinkedRequest },
    // ... the root's other variants, unchanged
}
```

- In the unlinked mode the root steps its client with `step_unlinked`/`fire_unlinked` and
  forwards as `Unlinked` every request it forwarded as `Client` before; what it turns into
  its own requests today (a `Down::Save` or `Down::Erase` into its `Save` or `Erase`, say) it
  turns the same way, in the same order.
- In the linked mode nothing changes; until `jig-linked-requests`, the root's arm for
  `jig_client::Request::Down { .. } | jig_client::Request::SignOut` in its linked path is one
  `unreachable!("a linked client asks no Down or SignOut (jig's client.md, 4.4)")`, which
  that task deletes.
- A host that only ever runs a linked root matches `Unlinked { .. }` as it matched the
  client's `Down` and `SignOut` before: where it handed them to the native shell (which
  panicked) or panicked itself, `unreachable!("a linked root asks no unlinked request (jig's
  client.md, 4.4)")`; where it ignored them, it ignores `Unlinked`.
