#[global_allocator]
static HEAP: skein_heap::Counting = skein_heap::Counting;

#[path = "../support"]
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
