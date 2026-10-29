#[global_allocator]
static HEAP: skein_world::domain::heap::Counting = skein_world::domain::heap::Counting;

mod job_answer;
mod locate;
mod memory;
mod queue;
mod recovery;
mod simulation;

mod fuzzy {
    mod simulation;
}

#[path = "../fixtures/mod.rs"]
mod fixtures;
