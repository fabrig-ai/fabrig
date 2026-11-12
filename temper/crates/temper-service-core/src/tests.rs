#[global_allocator]
static HEAP: skein_world::domain::heap::Counting = skein_world::domain::heap::Counting;

mod limits;
mod process;
mod long_pass;
mod startup;
mod tokens;
mod values;
