# debt-8: the shapes this plan binds

Binding for `core-prepare-agent`. The design is `jig/docs/design/domain/root.md`, 3.

## Why the route is a field on `Event::Activate`

`Now::PrepareAgent` did two things: it asked for a read slot, and it let the root answer `busy` while the task
was still `Due`. Once the core goes straight on, a task is `Preparing` before `Now::StartPreparation` reaches the
root, so a full table found there has no way back: the task must not leave `Due` while it waits, or a client sees
it. The answer has to reach the core before it moves the task, and so it comes with the activation.

The core's check sits in `route_activate`'s `Allow` arm, after admission, where `Now::PrepareAgent` was, so a
held task is still held and a job, a procedure or an agent with no window to load never waits on the table.

## The shapes

```rust
// jig-domain-core/src/vocabulary.rs
pub enum Now {
    /// Load the current transcript window in key order from its first turn.
    StartPreparation { task: u64, from: (u64, u32) },
    // PrepareAgent goes; Activate keeps `context`.
}
pub enum Event {
    /// An activation with the root's readiness; `busy` when it has no room to read a transcript.
    Activate { context: Box<tasks::RunContext>, ready: bool, busy: bool },
    // PreparedAgent goes.
}

// jig-domain-core/src/routing/run/preparation.rs
/// The window a preparation loads: the proof's, unless a fresh retry starts an open one over.
fn transcript_from(core: &Core, context: &tasks::RunContext) -> Option<(u64, u32)>;
fn route_prepared_agent(core, work, context, from: Option<(u64, u32)>) -> Queue<Request>;
```

`route_activate`'s `Allow` arm:

```rust
let from = transcript_from(core, &context);
if busy && from.is_some() {
    remember_due(core, context);
} else {
    return route_prepared_agent(core, work, context, from);
}
```

`transcript_from` is the one place this is decided. `route_prepared_agent` sets `transcript.window = from` after
`begin_transcript`, in place of its `fresh && !transcript.closed` reset. By reading, at this plan's base,
`load_transcript` and `from.is_some()` agree: `begin_transcript` takes `window` and
`closed = crossed || window.closed` from the proof, so both read
`window.is_some() && (transient == 0 || crossed || window.closed)`. The task's core tests pin it.

ops and the test root answer `busy: false`: they have no read table.

## temper reads the room when the activation runs

```rust
// temper-engine-domain/src/boundary.rs, `Work`, beside `Activate`
Due { context: Box<tasks::RunContext>, ready: bool },

// route/now.rs
core::Now::Activate { context } => {
    let ready = domain.ready() && crate::domain::within_budget(domain, &env.limits);
    domain.work.push(Work::Due { context, ready });
}

// route.rs, `route_into`
Work::Due { context, ready } => activate(domain, env, decision, context, ready),
Work::Activate(context) => {
    if domain.settling.active() || domain.core.stopping() { continue; }
    let ready = domain.ready();
    activate(domain, env, decision, context, ready);
}

/// One activation's step: `busy` is read here, so the load it allows finds its read slot.
fn activate(domain: &mut Domain, env: &Env<Limits>, decision: &mut Decision, context: Box<tasks::RunContext>, ready: bool) {
    let busy = domain.archive_reads.is_full();
    let routed = core::step(&mut domain.core, &environment_core(env), core::Event::Activate { context, ready, busy });
    route_core_requests(domain, env, decision, routed);
}

// route/now.rs
fn start_preparation(domain, env, decision, task, from: (u64, u32)) {
    let waiter = domain.archive_reads.insert(Some(Read::Transcript { task }))
        .expect("the activation's step found read room")
        .token();
    let range = Range::Core(crate::CoreRange::TaskTranscript { task, from });
    emit(decision, &env.limits, Delivery::Load { waiter, range, after: None });
}
```

`busy` is read when the work runs, never in the `Now::Activate` arm. Several activations can share a decision, and
any read can take a slot between that arm and its work; reading there would make the `expect` a crash. Between
`activate`'s read and `start_preparation` nothing takes a slot: `Now::StartPreparation` comes out of the same step,
and with a window to load the brief has not begun. `ready` stays where it is, so it means what it means today.

`Resume` does not change: no gate on `archive_reads` is added. While the table is full an agent with a window to
load is taken, answered `busy` and put back, as `Now::PrepareAgent`'s `busy` does at this plan's base, and a job or
an agent with no window goes on.
