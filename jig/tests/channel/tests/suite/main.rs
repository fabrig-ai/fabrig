#[global_allocator]
static HEAP: skein_world::domain::heap::Counting = skein_world::domain::heap::Counting;

mod channel;
mod program_runs;
mod program_sources;
mod referee;
mod source_carry_over;
mod worker_orders;

mod fuzzy {
    mod channel;
}
