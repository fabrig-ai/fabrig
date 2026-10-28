#[global_allocator]
static HEAP: skein_world::domain::heap::Counting = skein_world::domain::heap::Counting;

#[path = "../support/fresh_windows.rs"]
mod fresh_windows;

mod qa_closed_windows;
mod qa_settings_proposals;
