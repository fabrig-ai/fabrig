//! The person answers the work's question before its CSV change starts.
use temper_real_loop_world::green_light;

#[test]
fn a_green_light_in_the_inbox_lets_the_change_land() {
    let ran = green_light::run();
    green_light::check(&ran);
}
