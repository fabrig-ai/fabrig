#[global_allocator]
static HEAP: skein_heap::Counting = skein_heap::Counting;

#[path = "../support"]
mod support {
    pub(crate) mod events;
    pub(crate) mod events_max;
    pub(crate) mod patches;
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
