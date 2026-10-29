#[global_allocator]
static HEAP: skein_heap::Counting = skein_heap::Counting;

mod append;
mod cuts;
mod directory;
mod files;
mod joined;
mod limits;
mod memory;
mod output;
mod process_memory;
mod processes;
mod remove;
mod ring;
mod scenarios;

mod fuzzy {
    mod append;
    mod cuts;
    mod files;
    mod joined;
    mod limits;
    mod output;
    mod processes;
    mod scenarios;
}
