//! Worker domain worlds (temper/docs/design/testing/README.md, 2).

#[global_allocator]
static HEAP: skein_world::domain::heap::Counting = skein_world::domain::heap::Counting;

mod close;
mod jobs;
mod merge;
mod prepare;
mod orders;
mod runtime;
mod runtime_memory;
mod whole;

mod fuzzy {
    mod runtime;
    mod whole;
}
