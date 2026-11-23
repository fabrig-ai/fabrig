mod sweep;

#[test]
fn a_response_cut_at_each_byte_ends_its_call_once() {
    sweep::seed(0);
}
#[test]
fn a_response_cut_at_each_byte_ends_its_call_once_seed_1() {
    sweep::seed(1);
}
#[test]
fn a_response_cut_at_each_byte_ends_its_call_once_seed_2() {
    sweep::seed(2);
}
#[test]
fn a_response_cut_at_each_byte_ends_its_call_once_seed_3() {
    sweep::seed(3);
}
