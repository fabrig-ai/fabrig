#[global_allocator]
static HEAP: skein_heap::Counting = skein_heap::Counting;

mod declarations;
mod memory;
mod world;

mod fuzzy {
    mod declarations;
    mod documents;
    mod goldens;
}
