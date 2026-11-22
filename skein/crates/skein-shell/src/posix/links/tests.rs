//! The walk's owner argument stands in for root on either system.

use crate::testing::links::{configure, walk};
use skein_scratch::Scratch;
use std::os::unix::{ffi::OsStrExt, fs::symlink};

fn layout(mode: u32) -> Scratch {
    let scratch = Scratch::new("system-links");
    configure(scratch.path(), mode, false);
    scratch
}

fn parent_link(mode: u32) -> Scratch {
    let scratch = Scratch::new("system-links");
    configure(scratch.path(), mode, true);
    scratch
}

#[test]
fn dotdot_stops_before_a_link() {
    let scratch = parent_link(0o755);
    assert_eq!(walk(&scratch.path().join("real/../sys/f"), crate::effective_user()), None, "dotdot ends the walk");
}

#[test]
fn absolute_link_targets_restart_at_root() {
    let scratch = layout(0o755);
    let real = scratch.path().join("real");
    symlink(&real, scratch.path().join("abs")).expect("absolute target");
    let answer = walk(&scratch.path().join("abs/f"), crate::effective_user()).expect("absolute link passed");
    assert_eq!(&*answer, real.join("f").as_os_str().as_bytes(), "an absolute target replaces the whole prefix");
    assert!(answer.starts_with(b"/"), "the answer starts again at root");
}

#[test]
fn eight_links_are_passed_and_the_ninth_is_left() {
    let scratch = layout(0o755);
    for number in 1_u8..=9 {
        let target =
            if number == 9 { "real".into() } else { format!("l{}", number.checked_add(1).expect("small chain")) };
        symlink(target, scratch.path().join(format!("l{number}"))).expect("chain link");
    }
    let answer = walk(&scratch.path().join("l1/f"), crate::effective_user()).expect("eight links passed");
    assert_eq!(&*answer, scratch.path().join("l9/f").as_os_str().as_bytes(), "the ninth link is left as written");
}
