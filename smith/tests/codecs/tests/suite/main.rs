#[global_allocator]
static HEAP: skein_heap::Counting = skein_heap::Counting;

mod support {
    pub(crate) mod events;
    pub(crate) mod goldens;
    pub(crate) mod patches;
    pub(crate) mod wire;
}

mod channel;
mod charter;
mod drift;
mod events;
mod memory;
mod patches;
mod transcript;

mod fuzzy {
    mod decode;
    mod events;
    mod patches;
}

#[path = "../../../../../skein/testing/codec-memory.rs"]
mod codec_memory;
mod largest_memory;
