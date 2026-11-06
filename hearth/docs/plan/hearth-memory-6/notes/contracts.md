# hearth-memory-6: the shapes that cross tasks

Binding. `hearth/docs/design/domain/render.md`, 8.4 and 8.5 say why.

## `hearth-service-builder/src/lib.rs` after `stage`

```rust
pub struct Service {
    io: Io,
    files: FileIo,
    file_told: Queue<file::Event>,        // uncapped
    io_submissions: Queue<Submit>,        // uncapped
    file_submissions: Queue<Submit>,      // uncapped
    io_completions: Queue<Complete>,      // uncapped, new: Io's half of `completions`, the stage's to take
    io_asked: Queue<io::Request>,         // uncapped, new: at most one request, the stage's to take
    // the rest as they stand
    completions: Queue<Complete>,         // the kernel's edge: `limits.queue`
    submissions: Queue<Submit>,           // the kernel's edge: `limits.queue`
}

fn io_up(service: &mut Service) {
    // Every completion, split on its lane. A token is the one its owner minted.
    for _ in 0..service.completions.len() {
        let Some(complete) = service.completions.pop() else { break };
        if file_layer::owns(complete.op) {
            file_layer::up(&mut service.files, complete, &mut service.file_told, &mut service.file_submissions);
        } else {
            service.io_completions.push(complete);
        }
    }
    io::stage::up(&mut service.io, &service.io_env, &mut service.io_completions, &mut service.told,
        &mut service.io_submissions, service.io_env.now);
    if service.files.is_due(service.io_env.now) {
        file_layer::expire(&mut service.files, service.io_env.now, &mut service.file_submissions);
    }
}
```

`io_down` keeps one order for all of `asked`: a request that can't be taken holds the ones behind it, as today's single loop does. An `Asked::Io` goes through `io_asked` and `io::stage::down`, and the next request isn't popped until `io_asked` is empty and `files.takes()`. `Asked::Beneath` and `Asked::File` go to `file_layer::down` as they do now.

`iterate` ends by moving `io_submissions`, then `file_submissions`, into `submissions` while it has room. That is the one room check `iterate` keeps: the kernel's edge.

`work_pending` and `is_empty` read both new queues.

## `hearth-service-builder` after `uncap`

```rust
// limits.rs
pub struct Limits {
    pub io: skein_io::Limits,
    pub files: Files,
    pub protocol: protocol::limits::Limits,
    pub domain: domain::Limits,
    /// The two queues shared with the kernel, `completions` and `submissions`.
    pub queue: u32,
    /// Where render waits, and the most a pass resumes a layer (render.md, 8.4).
    pub watermark: u32,
}
// SHIPPED: `queue: 32`, `watermark: 30`

// lib.rs
impl Service {
    /// What `told`, `file_told`, `answers` and `asked` hold (render.md, 8.4).
    #[must_use]
    pub fn backlog(&self) -> u32 { /* saturating sum of the four lengths */ }
}
/// Whether the domain's next resume may run: the backlog is under the watermark (render.md, 8.5).
fn admits(service: &Service) -> bool { service.backlog() < service.watermark }
```

`Service` keeps `watermark: u32` where it kept `queue_limit`. `Service::new` asserts `limits.queue > 0 && limits.watermark > 0`, in place of `limits.queue >= 3`: a queue of nothing carries no operation, and a watermark of nothing admits no work.

```rust
fn protocol_up(service: &mut Service) {
    protocol::fire(...);
    for _ in 0..service.watermark {
        if !service.protocol.work_pending() { break; }
        protocol::resume(...);
    }
    if service.protocol.work_pending() { return; }
    for _ in 0..service.file_told.len() { /* work_pending guard, pop, up */ }
    for _ in 0..service.told.len() { /* work_pending guard, pop, up */ }
}
fn domain_stage(service: &mut Service) {
    for _ in 0..service.watermark {
        if !service.domain.work_pending() || !admits(service) { break; }
        domain::resume(...);
    }
    if service.domain.work_pending() { return; }
    for _ in 0..service.calls.len() { /* pop, step */ }
    if service.calls.is_empty() { domain::fire(...); }
}
fn protocol_down(service: &mut Service) {
    for _ in 0..service.answers.len() {
        if !service.protocol.takes() { break; }
        /* pop, down */
    }
}
```

`hearth::Client` (`hearth/crates/hearth/src/lib.rs`) gains `pub const fn service(&self) -> &Service`, for a world to read the service after an iteration.

## `hearth-protocol-builder/src/lib.rs` after `store-step`

```rust
/// Whether a store step answered: it added to what this entry gave above, whatever
/// that queue already held (hearth/docs/design/domain/render.md, 8.5).
fn store_answered(before: u32, above: &Queue<Event>) -> bool { above.len() != before }
```

`up` reads `above.len()` before `protocol.store.up`, and `store_answered` after it, where it now reads `!above.is_empty()`.
