#[global_allocator]
static HEAP: skein_heap::Counting = skein_heap::Counting;

mod frames;
mod fs;
mod hash;
mod memory;
mod trees;
mod walk;

mod fuzzy {
    mod tree;
}
