use temper_one_process_world::fixtures::ctl_chat::receipts_across_drained_services;

#[test]
fn a_service_after_a_drained_one_numbers_its_first_two_tasks_from_one() {
    let [first, second] = receipts_across_drained_services();
    let expected = [(1, 1), (2, 2)];
    assert_eq!(first, expected, "the first service's creation receipts");
    assert_eq!(second, expected, "the next service's creation receipts");
}
