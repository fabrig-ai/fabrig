//! Handles and slabs (lib.md, 3): a handle's round trip through a token, a
//! full slab, and a retired entity reachable until it is reclaimed, its
//! handle stale after.

use crate::{Id, Slab};

#[test]
fn a_handle_survives_the_round_trip_through_a_token() {
    let id: Id<u8> = Id::new(0xDEAD_BEEF, 7);
    assert_eq!(Id::<u8>::from_token(id.token()), id);
    assert_ne!(Id::<u8>::new(1, 2).token(), Id::<u8>::new(2, 1).token());
}

#[test]
fn a_full_slab_hands_the_value_back() {
    let mut slab = Slab::with_capacity(1);
    let id = slab.insert('a').expect("room for one");
    assert_eq!(slab.insert('b'), Err('b'));
    assert_eq!(slab.get(id), Some(&'a'));
    assert!(slab.is_full());
}

#[test]
fn a_retired_entity_is_reachable_until_reclaimed_and_its_handle_goes_stale() {
    let mut slab = Slab::with_capacity(1);
    let old = slab.insert('a').expect("room for one");
    slab.retire(old);
    assert_eq!(slab.get(old), Some(&'a'));
    assert_eq!(slab.len(), 1);
    slab.reclaim();
    assert_eq!(slab.get(old), None);
    assert!(slab.is_empty());
    let new = slab.insert('b').expect("the slot was freed");
    assert_ne!(new, old);
    assert_eq!(slab.get(old), None);
    assert_eq!(slab.get_mut(new), Some(&mut 'b'));
}

#[test]
#[should_panic(expected = "an entity is retired once")]
fn retiring_twice_is_a_bug() {
    let mut slab = Slab::with_capacity(1);
    let id = slab.insert(()).expect("room for one");
    slab.retire(id);
    slab.retire(id);
}

#[test]
fn a_zero_capacity_slab_refuses_everything() {
    let mut slab = Slab::with_capacity(0);
    assert_eq!(slab.insert(1_u8), Err(1));
}

#[test]
fn a_slab_hands_out_slots_in_the_order_it_did() {
    let mut slab = Slab::with_capacity(5);
    let first = slab.insert('a').expect("fresh slot");
    let second = slab.insert('b').expect("fresh slot");
    let third = slab.insert('c').expect("fresh slot");
    slab.retire(second);
    slab.reclaim();
    let reused_second = slab.insert('d').expect("reclaimed slot before a fresh one");
    let fourth = slab.insert('e').expect("fresh slot");
    slab.retire(first);
    slab.retire(third);
    slab.reclaim();
    let reused_first = slab.insert('f').expect("last freed slot first");
    let reused_third = slab.insert('g').expect("next freed slot");
    let fifth = slab.insert('h').expect("fresh slot");
    let expected = [(0, 0), (1, 0), (2, 0), (1, 1), (3, 0), (0, 1), (2, 1), (4, 0)];
    for (id, parts) in
        [first, second, third, reused_second, fourth, reused_first, reused_third, fifth].into_iter().zip(expected)
    {
        assert_eq!((id.slot(), id.generation()), parts);
    }
    assert_eq!(slab.get(reused_second), Some(&'d'), "a reused slot keeps its generation across growth");
}

#[test]
fn slab_walks_skip_retired_and_vacant_slots_in_slot_order() {
    let mut slab = Slab::with_capacity(4);
    assert_eq!(slab.iter().next(), None);
    assert_eq!(slab.next_after(None), None);
    let first = slab.insert('a').expect("room");
    let second = slab.insert('b').expect("room");
    let third = slab.insert('c').expect("room");
    slab.retire(second);
    assert_eq!(slab.get(second), Some(&'b'), "retired remains reachable by handle");
    let mut iter = slab.iter();
    assert_eq!(iter.next(), Some((first, &'a')));
    assert_eq!(iter.next(), Some((third, &'c')));
    assert_eq!(iter.next(), None);
    assert_eq!(slab.next_after(Some(first)), Some(third));
    slab.reclaim();
    assert_eq!(slab.next_after(Some(first)), Some(third), "vacant slot is skipped");
    let reused = slab.insert('d').expect("freed slot");
    assert_ne!(reused, second);
    for (from, expected) in [
        (None, Some(first)),
        (Some(first), Some(reused)),
        (Some(second), Some(third)),
        (Some(reused), Some(third)),
        (Some(third), None),
    ] {
        assert_eq!(slab.next_after(from), expected);
    }
    let mut iter = slab.iter();
    for expected in [(first, &'a'), (reused, &'d'), (third, &'c')] {
        assert_eq!(iter.next(), Some(expected));
    }
    assert_eq!(iter.next(), None);
}

#[test]
fn a_slab_can_change_between_walk_steps() {
    let mut slab = Slab::with_capacity(3);
    let first = slab.insert(1_u8).expect("room");
    let second = slab.insert(2).expect("room");
    assert_eq!(slab.next_after(None), Some(first));
    *slab.get_mut(first).expect("live") = 10;
    slab.retire(first);
    slab.reclaim();
    let replacement = slab.insert(3).expect("reclaimed slot");
    let third = slab.insert(4).expect("room");
    assert_eq!(slab.next_after(Some(first)), Some(second));
    slab.retire(second);
    assert_eq!(slab.next_after(Some(first)), Some(third));
    slab.reclaim();
    assert_eq!(slab.next_after(Some(second)), Some(third));
    assert_eq!(slab.next_after(None), Some(replacement));
}
