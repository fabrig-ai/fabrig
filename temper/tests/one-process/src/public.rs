//! Visitors through the service and a caching proxy (temper/docs/design/protocol/sign-in.md, 2.5).
mod checks;
mod deployment;
mod http;
mod programs;
mod proxy;
mod visitor;
mod wire;

use std::cell::Cell;
thread_local! { static ACTIVE: Cell<bool> = const { Cell::new(false) }; }
pub(crate) fn configure(value: &mut serde_json::Value) {
    if !ACTIVE.with(Cell::get) {
        return;
    }
    for (name, count) in [
        ("clients.public.reads", 4),
        ("clients_http.public_calls", 4),
        ("clients.public.rate", 8),
        ("clients.public.rate_burst", 8),
        ("clients.public.bytes", 524_288),
        ("clients.public.bytes_burst", 524_288),
        ("clients.public.answer_bytes", 131_072),
        ("clients.public.max_age", 2),
        ("core.public.goals", 4),
        ("core.public.ended", 4),
        ("core.public.lines", 8),
        ("core.public.depth", 4),
    ] {
        value["engine"]["limits"]["overrides"][name] = count.into();
    }
}

/// Replay a visitor's negative walk with failures naming its seed.
pub fn walk(seed: u64, name: &str) {
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        if name == "each_broken_relation_refuses_the_start" {
            deployment::relations();
        } else {
            checks::run(seed, name);
        }
    }));
    ACTIVE.with(|active| active.set(false));
    if let Err(error) = result {
        let detail = error
            .downcast_ref::<String>()
            .map(String::as_str)
            .or_else(|| error.downcast_ref::<&str>().copied())
            .unwrap_or("non-text panic");
        panic!("public walk {name}, seed {seed}: {detail}");
    }
}
