//! Upgrade power losses across drawn moments.

#[test]
fn drawn_power_losses_during_an_upgrade_keep_the_history_and_continue_the_story() {
    for seed in 1..=4 {
        temper_one_process_world::upgrade::story::drawn_power_loss(seed);
    }
}
