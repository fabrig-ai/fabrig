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
fn slab_iteration_skips_retired_and_vacant_slots_and_uses_current_handles() {
    let mut slab = Slab::with_capacity(5);
    let first = slab.insert(10_u32).expect("fresh slot");
    let second = slab.insert(20).expect("fresh slot");
    let third = slab.insert(30).expect("fresh slot");
    let fourth = slab.insert(40).expect("fresh slot");
    let fifth = slab.insert(50).expect("fresh slot");
    slab.retire(first);
    slab.retire(third);
    assert_eq!(slab.get(first), Some(&10), "retirement still permits lookup");
    assert_eq!(slab.iter().count(), 3, "retired entities are absent before reclaim");
    slab.reclaim();
    assert_eq!(slab.get(first), None);
    assert_eq!(slab.get(third), None);
    let reused = slab.insert(60).expect("reuse first slot, leaving third vacant");
    assert_eq!(reused.slot(), first.slot());
    assert_ne!(reused, first);
    slab.retire(fifth);

    let mut iter: crate::slab::Iter<'_, u32> = slab.iter();
    for (id, expected) in [(reused, 60), (second, 20), (fourth, 40)] {
        let (visited, value) = iter.next().expect("one item per live slot");
        assert_eq!((visited, *value), (id, expected));
        assert_eq!(slab.get(visited), Some(value), "the yielded handle names this entity");
    }
    assert_eq!(iter.next(), None);
    assert_eq!(iter.next(), None);
}

#[test]
fn mutable_slab_iteration_changes_only_live_entities_in_slot_order() {
    let mut slab = Slab::with_capacity(5);
    let first = slab.insert(10_u32).expect("fresh slot");
    let second = slab.insert(20).expect("fresh slot");
    let third = slab.insert(30).expect("fresh slot");
    let fourth = slab.insert(40).expect("fresh slot");
    let fifth = slab.insert(50).expect("fresh slot");
    slab.retire(first);
    slab.retire(third);
    slab.reclaim();
    let reused = slab.insert(60).expect("reuse first slot, leaving third vacant");
    slab.retire(fifth);

    let mut iter: crate::slab::IterMut<'_, u32> = slab.iter_mut();
    let (a_id, a) = iter.next().expect("reused first slot");
    let (b_id, b) = iter.next().expect("second slot");
    let (c_id, c) = iter.next().expect("fourth slot after the vacant third");
    assert_eq!((a_id, b_id, c_id), (reused, second, fourth));
    assert_eq!((*a, *b, *c), (60, 20, 40));
    assert_eq!(iter.next(), None, "retired fifth slot is skipped");
    assert_eq!(iter.next(), None);
    // Items from different next calls remain usable together.
    *a = 61;
    *b = 21;
    *c = 41;

    assert_eq!(slab.get(reused), Some(&61));
    assert_eq!(slab.get(second), Some(&21));
    assert_eq!(slab.get(fourth), Some(&41));
    assert_eq!(slab.get(first), None);
    assert_eq!(slab.get(third), None);
    assert_eq!(slab.get(fifth), Some(&50), "retired storage was untouched");
}

#[test]
fn slab_walks_are_empty_without_live_entities() {
    for capacity in [0_u32, 3] {
        let mut slab = Slab::<u32>::with_capacity(capacity);
        assert_eq!(slab.iter().next(), None);
        assert_eq!(slab.iter_mut().next(), None);
        if capacity > 0 {
            let id = slab.insert(7).expect("one slot");
            slab.retire(id);
            assert_eq!(slab.iter().next(), None);
            assert_eq!(slab.iter_mut().next(), None);
            slab.reclaim();
            assert_eq!(slab.iter().next(), None);
            assert_eq!(slab.iter_mut().next(), None);
        }
    }
}
