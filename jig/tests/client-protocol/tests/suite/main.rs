#[global_allocator]
static HEAP: skein_world::domain::heap::Counting = skein_world::domain::heap::Counting;

mod client;
mod deadline_spans;
mod native;
mod orders;
mod protocol;
mod referee;
mod settings;
mod turn_read;

mod fuzzy {
    mod client;
    mod protocol;
}
