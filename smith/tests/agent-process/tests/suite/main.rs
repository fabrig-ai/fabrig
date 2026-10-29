#[global_allocator]
static HEAP: skein_heap::Counting = skein_heap::Counting;

#[path = "../support/direct_opening.rs"]
mod direct_opening;
#[path = "../support/opening.rs"]
mod opening;

mod content;
mod fact_drain;
mod grants;
mod memory;
mod open_opening;
mod program;
mod program_budget;
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
