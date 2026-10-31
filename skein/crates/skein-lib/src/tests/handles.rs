//! Handles and slabs (lib.md, 3): a handle's round trip through a token, a
//! full slab, a retired entity reachable until it is reclaimed, its handle
//! stale after, and the slab walked in slot order.

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

/// The slab's entities as its iterator visits them, with their handles.
fn walked(slab: &Slab<char>) -> Box<[(Id<char>, char)]> {
    let mut seen = crate::List::uncapped();
    for (id, value) in slab {
        seen.push((id, *value)).expect("an uncapped list");
    }
    assert_eq!(slab.iter().size_hint(), (seen.as_slice().len(), Some(seen.as_slice().len())), "an exact hint");
    seen.into_boxed()
}

#[test]
fn an_empty_slab_walks_nothing() {
    assert_eq!(*walked(&Slab::with_capacity(4)), []);
    assert_eq!(*walked(&Slab::with_capacity(0)), []);
}

#[test]
fn a_slab_walks_its_entities_in_slot_order_retired_ones_until_reclaimed() {
    let mut slab = Slab::with_capacity(4);
    let first = slab.insert('a').expect("room");
    let second = slab.insert('b').expect("room");
    let third = slab.insert('c').expect("room");
    assert_eq!(*walked(&slab), [(first, 'a'), (second, 'b'), (third, 'c')]);
    slab.retire(second);
    assert_eq!(*walked(&slab), [(first, 'a'), (second, 'b'), (third, 'c')], "a retired entity is still reachable");
    slab.reclaim();
    assert_eq!(*walked(&slab), [(first, 'a'), (third, 'c')]);
    let reused = slab.insert('d').expect("the reclaimed slot");
    let fourth = slab.insert('e').expect("a fresh slot");
    assert_eq!(*walked(&slab), [(first, 'a'), (reused, 'd'), (third, 'c'), (fourth, 'e')], "slot order");
    assert_ne!(reused, second, "the reused slot names its new entity");
    for (id, value) in &slab {
        assert_eq!(slab.get(id), Some(value), "each handle reaches what it was walked with");
    }
    slab.retire(first);
    slab.retire(fourth);
    slab.reclaim();
    assert_eq!(*walked(&slab), [(reused, 'd'), (third, 'c')], "vacant slots at either end are skipped");
}
