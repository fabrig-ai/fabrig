#[global_allocator]
static HEAP: skein_heap::Counting = skein_heap::Counting;

mod access_only;
mod accounts;
mod memory;
mod private;
mod real;
mod referee;
mod shared;
mod sign_in;
mod sign_out;
mod simulated;
mod world;

mod fuzzy {
    mod private;
    mod shared;
    mod sign_in;
    mod world;
}
