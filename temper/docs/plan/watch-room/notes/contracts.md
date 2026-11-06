# watch-room: the shapes that cross tasks

These are the run's binding calls. Each side builds to them, and neither needs the other's task unless its `needs` says so.

## jig's client: a watch refused busy (`link-told`)

`jig/docs/design/client/domain/client.md`, 8.2.

```rust
// jig-client-domain, src/link.rs
impl Domain {
    /// A watch held was last answered busy, with no answer since (client.md, 8.2).
    #[must_use]
    pub fn watches_full(&self) -> bool;
}
// jig-client, src/client.rs
pub struct LinkView<'a> {
    // ...beside `streams_full`:
    pub watches_full: bool,
    // ...
}
```

It is a `bool` because a refusal names no count, and it tells no `Told::Link`. A closing watch doesn't count.

## temper's engine: the rooms (`size-budget`)

`temper/docs/design/protocol/configuration.md`, 7; `temper/docs/design/domain/root.md`, 10.

```rust
// temper-engine-domain, src/limits.rs
pub struct Limits {
    // ...
    /// The clients' streams the deployment admits, `clients.streams`: the watch rooms follow them.
    pub streams: u32,
    // ...
}
pub const WATCHES_PER_STREAM: u32 = 17;
/// The views' limits for one admitted stream. `views_for` scales its rooms.
pub const VIEWS: views::Limits; // watchers and watchers_per_party are WATCHES_PER_STREAM
/// `VIEWS` with both rooms at `streams` times `WATCHES_PER_STREAM`, saturating: `valid` refuses what doesn't fit.
pub const fn views_for(streams: u32) -> views::Limits;
```

- `core_limits` sets `views: views_for(limits.streams)`.
- `temper-service`'s `engine_candidate` sets `streams: jig.clients.streams`.
- Worlds whose root limits are written out (`temper/tests/engine/domain/src/walking.rs`, `temper-engine-domain/src/tests/resume_fixture.rs`) set `streams: 1`, so their room stays 17 and their stories and ceilings keep their meaning. A story that means the shipped figure sets `streams = 8`.
- `VIEWS` keeps its name, so `temper-engine-domain/src/route/forge_route.rs`, which `update-recovers/update-hold-words` is changing, isn't touched.

## The stories (`story`, then `size-budget`)

All in `temper/tests/client/tests/suite/watch_room.rs`, on these helpers:

```rust
// temper-client-world, src/fixtures.rs
pub fn watch_room_limits() -> Limits;       // domain_limits() at the shipped shells' figures that decide a page's watches
pub fn watch_room_world(seed: u64) -> World<Temper, Joint>;
pub fn watch_room_chat(front: &mut Joint) -> u64;   // a long chat whose page holds its delegates' watches
pub fn watch_room_client(world: &mut World<Temper, Joint>, address: Address) -> ClientId;
// temper-client-world, src/front.rs
impl Joint {
    pub fn with_watch_room(seed: u64, limits: &client::Limits) -> Self;
    pub fn open_watches(&self) -> usize;   // the engine's watches held, closing ones aside
}
```

`story` writes `two_open_tabs_leave_a_fresh_tools_client_room_for_its_read` under `#[should_panic(expected = ...)]`. `size-budget` removes that attribute, and sets the world's `streams = 8` in `with_watch_room`.

## `temper ctl`'s line (`ctl-told`)

Exactly, at the deadline, after the busy sign-in's and the full streams' lines (`temper/docs/design/client/tools.md`, 6.3):

```text
temper ctl: all the engine's watches are in use, by open pages and waits; close one, or wait and call again
```
