#[global_allocator]
static HEAP: skein_world::domain::heap::Counting = skein_world::domain::heap::Counting;

mod brief;
mod changes;
mod follow_check;
mod follows;
mod green;
mod hooks;
mod landing;
mod memory;
mod protection;
mod public;
mod replay;
mod rules_by_path;
mod settle;
mod simulation;
mod tiers;
mod update_lost;
mod writer;

mod fuzzy {
    mod branch_patterns;
    mod pull_cutoff;
    mod rules_by_path;
    mod simulation;
}

mod kept_change;
