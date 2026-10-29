#[global_allocator]
static HEAP: skein_world::domain::heap::Counting = skein_world::domain::heap::Counting;

mod one_process;

mod fuzzy {
    mod one_process;
}
