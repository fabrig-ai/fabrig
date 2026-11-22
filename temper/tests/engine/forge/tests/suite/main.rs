#[global_allocator]
static HEAP: skein_world::domain::heap::Counting = skein_world::domain::heap::Counting;

mod blame;
mod blame_open;
mod brief;
mod catching_up_outbox;
mod change_history;
mod changes;
mod check_record_open;
mod delivered;
mod follow_check;
mod follows;
mod goal_branch_restart;
mod green;
mod history_world;
mod hooks;
mod lagging;
mod landing;
mod landing_batches;
mod landing_cancelled;
mod landing_history;
mod landing_judgement;
mod landing_leaving;
mod landing_merge_readback;
mod landing_proofs;
mod landing_red_four;
mod landing_reports;
mod landing_rounds;
mod landing_search_tests;
mod ledger;
mod member_message;
mod memory;
mod opened;
mod protection;
mod public;
mod readiness;
mod replay;
mod rules_by_path;
mod runner_wait;
mod settle;
mod simulation;
mod taking;
mod tiers;
mod train_drills;
mod trains;
mod update_lost;
mod workflow_flights;
mod writer;

mod fuzzy {
    mod bounds;
    mod branch_patterns;
    mod busy;
    mod ci;
    mod gate_keys;
    mod history;
    mod lagging;
    mod landing_batches;
    mod member_message;
    mod opened;
    mod pull_cutoff;
    mod queue_history;
    mod refusing;
    mod rules_by_path;
    mod simulation;
    mod stale;
}

mod refusing;

mod kept_change;

mod history;
mod history_sweep;
