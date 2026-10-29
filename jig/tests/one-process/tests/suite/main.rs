#[global_allocator]
static HEAP: skein_world::domain::heap::Counting = skein_world::domain::heap::Counting;

mod accounts;
mod one_process;

mod fuzzy {
    mod accounts;
    mod one_process;
}
