#[global_allocator]
static HEAP: skein_world::domain::heap::Counting = skein_world::domain::heap::Counting;

mod brief;
mod changes;
mod follow_check;
mod follows;
mod green;
mod history_world;
mod hooks;
mod landing;
mod landing_batches;
mod landing_judgement;
mod landing_leaving;
mod landing_rounds;
mod member_message;
mod memory;
mod protection;
mod public;
mod replay;
mod rules_by_path;
mod settle;
mod simulation;
mod tiers;
mod trains;
mod update_lost;
mod writer;

mod fuzzy {
    mod branch_patterns;
    mod gate_keys;
    mod landing_batches;
    mod member_message;
    mod pull_cutoff;
    mod rules_by_path;
    mod simulation;
}

mod kept_change;
