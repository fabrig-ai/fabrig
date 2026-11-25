#[global_allocator]
static HEAP: skein_heap::Counting = skein_heap::Counting;

mod support {
    pub(crate) mod events;
    pub(crate) mod fixed_values;
    pub(crate) mod wire;
}

mod channel;
mod charter;
mod drift;
mod events;
mod memory;
mod transcript;

mod fuzzy {
    mod decode;
    mod events;
}

mod codec_memory {
    include!("../../../../../skein/testing/codec-memory.rs");
}
mod largest_memory;
