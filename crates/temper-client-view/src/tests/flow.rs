#![allow(clippy::disallowed_types, clippy::disallowed_methods, reason = "test assertions inspect progress states")]
use super::fixtures as f;
use crate::flow::{self, Mark};
use skein_ui::{Builder, Role, State};
#[test]
fn flow_steps_mark_the_current_one_and_those_before_it_done() {
    for current in 0_usize..4 {
        let mut b = Builder::new(&f::tree_limits());
        flow::steps(&mut b, &f::words(), &[b"Choose", b"Details", b"Setting up", b"Ready"], current);
        let tree = b.finish();
        assert!(f::find(&tree, Role::List, b"Steps").is_some());
        let mut index = 0_usize;
        for item in tree.nodes() {
            if item.role != Role::Item {
                continue;
            }
            assert_eq!(item.states.as_slice().contains(&State::Current), index == current);
            assert_eq!(item.styles.as_slice().contains(&crate::look::FLOW_DONE), index < current);
            index += 1;
        }
        assert_eq!(index, 4_usize);
    }
}
#[test]
fn a_progress_says_each_lines_state() {
    let mut b = Builder::new(&f::tree_limits());
    flow::progress(
        &mut b,
        &f::words(),
        b"Set up",
        &[(b"Fetch", Mark::Done), (b"Prepare", Mark::UnderWay), (b"Run", Mark::Failed), (b"Connect", Mark::ToCome)],
    );
    let tree = b.finish();
    assert_eq!(f::text(&tree).as_ref(), b"FetchdonePrepareunder wayRunfailedConnect");
    let mut statuses = 0_u32;
    for node in tree.nodes() {
        if node.role == Role::Status {
            statuses += 1;
        }
    }
    assert_eq!(statuses, 3_u32);
    let items: Vec<_> = tree.nodes().iter().filter(|n| n.role == Role::Item).collect();
    assert!(items[0].styles.as_slice().contains(&crate::look::FLOW_DONE));
    assert!(items[1].states.as_slice().contains(&State::Current));
    assert!(items[2].styles.as_slice().contains(&crate::look::FLOW_FAILED));
    assert!(items[3].styles.is_empty());
    assert!(items[3].states.is_empty());
}

#[test]
fn only_primary_controls_carry_primary_style() {
    use crate::flow::Control;
    let mut b = Builder::new(&f::tree_limits());
    b.open(Role::Slot);
    flow::control(&mut b, Control::Disabled { name: b"Later" });
    flow::control(&mut b, Control::PrimaryDisabled { name: b"Connect" });
    flow::control(&mut b, Control::QuietDisabled { name: b"All ways" });
    b.close();
    let tree = b.finish();
    let nodes: Vec<_> = tree.nodes().iter().filter(|node| node.role == Role::Button).collect();
    assert!(nodes[0].styles.as_slice().contains(&crate::look::FLOW_SECONDARY));
    assert!(nodes[1].styles.as_slice().contains(&crate::look::FLOW_PRIMARY));
    assert!(nodes[2].styles.as_slice().contains(&crate::look::FLOW_QUIET));
    assert!(nodes.iter().all(|node| node.states.as_slice().contains(&State::Disabled)));
}
