#[test]
fn a_private_file_key_authenticates_the_claimed_chat() {
    temper_one_process_world::fixtures::ctl_chat::private_file_key();
}
