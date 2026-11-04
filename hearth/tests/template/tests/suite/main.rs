#[global_allocator]
static HEAP: skein_heap::Counting = skein_heap::Counting;

mod expressions;
mod memory;
mod templates;

mod fuzzy {
    mod expressions;
    mod goldens;
    mod templates;
}
