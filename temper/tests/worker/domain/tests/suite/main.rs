//! Worker domain worlds (temper/docs/design/testing/tiers.md, 1).

#[global_allocator]
static HEAP: skein_world::domain::heap::Counting = skein_world::domain::heap::Counting;

mod close;
mod faults;
mod inspection;
mod jobs;
mod merge;
mod orders;
mod prepare;
mod restarts;
mod runtime;
mod runtime_memory;
mod whole;
mod workload;

mod fuzzy {
    mod restarts;
    mod runtime;
    mod whole;
    mod workload;
}
