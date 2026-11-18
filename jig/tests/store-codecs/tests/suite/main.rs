#[global_allocator]
static HEAP: skein_heap::Counting = skein_heap::Counting;

mod memory;
mod placed;
mod profiles;
mod questions;
mod secrets;
mod task_bounds;
mod values;

mod fuzzy {
    mod decode;
}
