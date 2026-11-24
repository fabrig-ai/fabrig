#[test]
fn a_cold_ctl_read_keeps_the_reply_and_waiting_across_a_provider_rebind() {
    temper_one_process_world::fixtures::ctl_chat::cold_read();
}
