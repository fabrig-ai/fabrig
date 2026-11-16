//! Upgrade power losses at the four named cuts and across drawn moments.
#[test]
fn power_lost_at_the_copys_first_write_keeps_a_whole_generation() {
    temper_one_process_world::upgrade::story::power_loss(0);
}
#[test]
fn power_lost_at_the_patch_commit_keeps_a_whole_generation() {
    temper_one_process_world::upgrade::story::power_loss(1);
}
#[test]
fn power_lost_at_the_switch_rename_keeps_a_whole_generation() {
    temper_one_process_world::upgrade::story::power_loss(2);
}
#[test]
fn power_lost_at_the_wires_commit_keeps_a_whole_generation() {
    temper_one_process_world::upgrade::story::power_loss(3);
}

#[test]
fn drawn_power_losses_during_an_upgrade_keep_the_history_and_continue_the_story() {
    for seed in 1..=4 {
        temper_one_process_world::upgrade::story::drawn_power_loss(seed);
    }
}
