#[global_allocator]
static HEAP: skein_heap::Counting = skein_heap::Counting;

mod memory;
mod placed;
mod profiles;
mod questions;
mod runs;
mod secrets;
mod values;

mod fuzzy {
    mod decode;
}

mod codec_memory {
    include!("../../../../../skein/testing/codec-memory.rs");
}
mod largest_memory;
