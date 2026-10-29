//! Worker domain worlds (temper/docs/design/domain/testing.md, 2).

#[global_allocator]
static HEAP: skein_world::domain::heap::Counting = skein_world::domain::heap::Counting;

mod orders;
mod runtime;
mod runtime_memory;
mod whole;

mod fuzzy {
    mod runtime;
    mod whole;
}
