//! A committed answer survives loss of the caller on the simulated kernel.

#[test]
fn a_lost_ctl_answer_is_read_back_once_from_a_second_directory() {
    temper_one_process_world::lost::run();
}
