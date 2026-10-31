//! Counted loss at a capped queue (skein/docs/design/lib.md, 5).

use crate::Lossy;

#[test]
fn a_full_lossy_queue_keeps_the_first_items_and_counts_each_drop() {
    let mut queue = Lossy::with_capacity(2);
    assert!(queue.is_empty());
    assert_eq!(queue.len(), 0);
    assert_eq!(queue.lost(), 0);
    for item in 1_u8..=2 {
        queue.push(item);
    }
    assert_eq!(queue.lost(), 0);
    for item in 3_u8..=5 {
        queue.push(item);
        assert_eq!(queue.lost(), u64::from(item - 2));
        assert_eq!(queue.len(), 2);
    }
    assert_eq!(queue.pop(), Some(1));
    assert_eq!(queue.pop(), Some(2));
    assert_eq!(queue.pop(), None);
    assert!(queue.is_empty());
    assert_eq!(queue.len(), 0);
    assert_eq!(queue.lost(), 3);
}

#[test]
fn popping_makes_room_without_resetting_loss() {
    let mut queue = Lossy::with_capacity(2);
    queue.push(1_u8);
    queue.push(2);
    queue.push(3);
    assert_eq!(queue.pop(), Some(1));
    queue.push(4);
    assert_eq!(queue.lost(), 1);
    assert_eq!(queue.len(), 2);
    assert_eq!(queue.pop(), Some(2));
    assert_eq!(queue.pop(), Some(4));
}

#[test]
fn a_zero_capacity_lossy_queue_counts_every_item() {
    let mut queue = Lossy::with_capacity(0);
    queue.push(1_u8);
    queue.push(2);
    assert_eq!(queue.lost(), 2);
    assert!(queue.is_empty());
    assert_eq!(queue.len(), 0);
    assert_eq!(queue.pop(), None);
}

#[test]
fn loss_saturates_instead_of_wrapping() {
    let mut queue = Lossy::with_capacity(0);
    queue.lost = u64::MAX - 1;
    queue.push(1_u8);
    assert_eq!(queue.lost(), u64::MAX);
    queue.push(2);
    assert_eq!(queue.lost(), u64::MAX);
}
