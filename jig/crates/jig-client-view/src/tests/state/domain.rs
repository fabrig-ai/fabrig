//! Vocabulary invariants; behaviour step tests land with the spine.
use jig_client::contract::{Text, VersionRange};
use skein_lib::List;
#[test]
fn text_refuses_bytes_past_the_owner_bound() {
    assert_eq!(Text::new(Box::from(*b"abcd"), 3).unwrap_err().as_ref(), b"abcd");
    assert_eq!(Text::new(Box::from(*b"abc"), 3).unwrap().bytes(), b"abc");
    assert_eq!(Text::new(Box::from([]), 0).unwrap().bytes(), b"");
}
#[test]
fn lists_refuse_past_their_capacity() {
    let mut rows = List::with_capacity(1);
    assert_eq!(rows.push(7_u64), Ok(()));
    assert_eq!(rows.push(8_u64), Err(8));
    assert_eq!(rows.as_slice(), &[7]);
}
#[test]
fn version_ranges_are_inclusive_and_invalid_ranges_match_nothing() {
    let range = VersionRange { first: 2, last: 4 };
    assert!(!range.contains(1));
    assert!(range.contains(2));
    assert!(range.contains(4));
    assert!(!range.contains(5));
    assert!(!VersionRange { first: 4, last: 2 }.contains(3));
}

pub(crate) mod spine;

pub(crate) mod storage;

pub(crate) mod lifecycle;

pub(crate) mod time;

pub(crate) mod objects;

pub(crate) mod confirmation;
pub(crate) mod pages;

pub(crate) mod answers;
pub(crate) mod conversations;
pub(crate) mod drafts;

pub(crate) mod policy;

pub(crate) mod frame_facts;

pub(crate) mod chip_facts;

pub(crate) mod history_facts;

pub(crate) mod owner_inbox;
pub(crate) mod question_sender;
pub(crate) mod task_ending;

pub(crate) mod owner_tree;
pub(crate) mod task_resources;

pub(crate) mod owner_person_task;
pub(crate) mod owner_proposal;
pub(crate) mod task_context;

pub(crate) mod ended_pages;
pub(crate) mod owner_deployment;
pub(crate) mod owner_project;

pub(crate) mod ended_objects;
pub(crate) mod owner_effect_parts;
pub(crate) mod owner_hold_parts;
pub(crate) mod task_revisit;

pub(crate) mod native;

pub(crate) mod pending;
