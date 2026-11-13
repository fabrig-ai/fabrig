use super::fixtures;
use skein_lib::Queue;

const HOUR: u64 = 3600;

// Thursday 22 October 2026, midnight UTC.
const MIDNIGHT: u64 = 1_792_627_200;

use crate::words_tests::words::wall;

#[test]
fn a_build_reads_the_shells_zone_from_the_domain() {
    let (mut root, _, _) = fixtures::root_at(
        temper_client_domain::Address::Home { project: fixtures::PROJECT },
        &fixtures::domain_limits(),
    );
    assert_eq!(root.zone_minutes(), 0_i32);
    root.set_zone_minutes(120);
    assert_eq!(root.zone_minutes(), 120_i32);
    let mut view = fixtures::view();
    let mut patches = Queue::with_capacity(view.limits().tree.patches);
    crate::build(&mut view, &root, wall(MIDNIGHT + 12 * HOUR), &mut patches);
    assert!(skein_lib::bytes::find(&fixtures::text(view.tree()), b"14:00").is_some());
}
