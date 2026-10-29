#[global_allocator]
static HEAP: skein_heap::Counting = skein_heap::Counting;

mod drift;
mod memory;
mod placed;
mod profiles;
mod questions;
mod secrets;

mod fuzzy {
    mod decode;
}
