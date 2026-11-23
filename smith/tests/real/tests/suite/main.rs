#[global_allocator]
static HEAP: skein_world::domain::heap::Counting = skein_world::domain::heap::Counting;

mod dump;
mod program_configuration;
mod stories;
mod tests;
