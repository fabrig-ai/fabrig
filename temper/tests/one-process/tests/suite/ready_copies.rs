//! Ready's burst made and adopted (temper/docs/design/domain/testing.md, 2.5–2.6).
use temper_one_process_world::ready_copies;

#[test]
fn readys_two_copies_are_made_in_order_and_adopted_with_the_first_as_home() {
    let outcome = ready_copies::run();
    assert_eq!(outcome.kept, 2, "both requests were kept");
    assert!(outcome.first_making_second_waiting, "the second copy waited for the first");
    assert_eq!(outcome.adopted, 2, "both copies were adopted");
}
