#[global_allocator]
static HEAP: skein_heap::Counting = skein_heap::Counting;

mod direct_opening;
mod opening;

mod coding;
mod content;
mod dump;
mod fact_drain;
mod grants;
mod memory;
mod open_opening;
mod orders;
mod program;
mod program_budget;
mod program_children;
mod program_configuration;
mod program_events;
mod program_open;
mod program_resume;
mod program_tools;
mod program_turns;
mod stories;

mod fuzzy {
    mod fact_drain;
    mod open_opening;
    mod process;
    mod profile;
}
