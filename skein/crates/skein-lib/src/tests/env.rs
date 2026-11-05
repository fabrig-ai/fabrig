//! A child environment shares the iteration's clocks (skein/docs/design/lib.md, 9).

use alloc::boxed::Box;

use crate::{Env, Time, Wall};

#[test]
fn a_child_environment_keeps_the_clocks_and_owns_its_limits() {
    let parent = Env { now: Time::from_nanos(17), wall: Wall::from_nanos(29), limits: Box::new(3_u8) };
    let child = parent.with(Box::new([5_u32, 7]));
    assert_eq!(child.now, parent.now);
    assert_eq!(child.wall, parent.wall);
    assert_eq!(*child.limits, [5, 7]);
    assert_eq!(*parent.limits, 3);
}
