#[global_allocator]
static HEAP: skein_heap::Counting = skein_heap::Counting;

mod close;
mod failures;
mod fake_protocol;
mod memory;
mod protocol;
mod simulated;
mod waiting;

mod fuzzy {
    mod close;
    mod simulated;
}
