#[global_allocator]
static HEAP: skein_world::domain::heap::Counting = skein_world::domain::heap::Counting;

mod both_ways;
mod channel;
mod list_read;
mod program_runs;
mod program_sources;
mod referee;
mod refused_calls;
mod source_carry_over;
mod worker_orders;

mod fuzzy {
    mod channel;
}
