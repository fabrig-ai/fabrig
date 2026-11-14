//! The bounded containers (runtime/lib.md, 5): each refuses past its capacity and
//! keeps its order, a B-tree is priced in its nodes, and a full buffer
//! grows by one rule.

use alloc::boxed::Box;

use crate::btree::price;
use crate::growth::grown;
use crate::{Bisection, Buffer, Intake, Joined, List, Lru, Map, Queue, Rng, Set, Stack, state_digest};

#[test]
fn a_full_buffer_doubles_from_the_floor_to_what_it_needs_under_its_cap() {
    assert_eq!(grown(0, 1, 100), 4, "the floor");
    assert_eq!(grown(4, 5, 100), 8, "twice");
    assert_eq!(grown(8, 30, 100), 30, "what it needs, past twice");
    assert_eq!(grown(64, 65, 100), 100, "never past the cap");
    assert_eq!(grown(0, 1, 2), 2, "a cap below the floor");
    assert_eq!(grown(usize::MAX / 2 + 1, 1, usize::MAX), usize::MAX, "doubling saturates");
}

#[test]
fn a_list_refuses_past_its_capacity() {
    let mut list = List::with_capacity(2);
    assert_eq!(list.push(1_u8), Ok(()));
    assert_eq!(list.push(2), Ok(()));
    assert_eq!(list.push(3), Err(3));
    assert_eq!(list.get(1), Some(&2));
    assert_eq!(list.get(2), None);
    assert_eq!(&*list.to_boxed(), &[1, 2]);
    assert_eq!(&*list.into_boxed(), &[1, 2]);
}

#[test]
fn a_grown_list_refuses_at_its_logical_cap_after_refilling() {
    // Zero, below the growth floor, and caps reached after one or two doublings.
    const CAPS: [u32; 4] = [0, 3, 5, 9];
    for cap in CAPS {
        let mut list = List::with_capacity(cap);
        for item in 0..cap {
            list.push(item).expect("within the logical cap");
        }
        list.clear();
        for item in 0..cap {
            list.push(item).expect("refill within the logical cap");
        }
        assert_eq!(list.push(cap), Err(cap), "cap {cap}");
        assert_eq!(list.len(), cap, "cap {cap}");
    }
}

#[test]
fn a_list_from_owned_items_is_full_and_keeps_their_order() {
    let items: Box<[Box<u8>]> = Box::new([Box::new(3), Box::new(7)]);
    let address = items.as_ptr();
    let mut list = List::from_boxed(items);
    assert_eq!(list.as_slice().as_ptr(), address, "ownership transfers without copying items");
    assert_eq!(list.len(), 2);
    assert_eq!(list.capacity(), 2);
    assert_eq!(list.room(), 0);
    assert_eq!(list.push(Box::new(9)), Err(Box::new(9)));
    assert_eq!(list.as_slice(), &[Box::new(3), Box::new(7)]);
    list.clear();
    assert_eq!(list.room(), 2);
    assert_eq!(list.push(Box::new(11)), Ok(()));
    assert_eq!(&*list.into_boxed(), &[Box::new(11)]);
}

#[test]
fn a_list_from_no_items_has_no_capacity() {
    let mut list = List::<u8>::from_boxed(Box::new([]));
    assert!(list.is_empty());
    assert_eq!(list.capacity(), 0);
    assert_eq!(list.room(), 0);
    assert_eq!(list.push(1), Err(1));
    assert!(list.into_boxed().is_empty());
}

#[test]
fn list_value_derives_compare_and_hash_items() {
    use core::hash::BuildHasher;

    let mut first = List::with_capacity(2);
    first.push(7_u8).expect("room");
    let cloned = first.clone();
    assert_eq!(cloned.capacity(), 2);
    assert_eq!(cloned, first);

    let mut same_items_with_more_room = List::with_capacity(3);
    same_items_with_more_room.push(7_u8).expect("room");
    assert_eq!(first, same_items_with_more_room);

    let state = std::collections::hash_map::RandomState::new();
    assert_eq!(state.hash_one(&first), state.hash_one(&same_items_with_more_room));
}

#[test]
fn slots_pushed_in_order_are_filled_in_any_order() {
    let mut results = List::with_capacity(3);
    for _ in 0_u32..3 {
        results.push(None).expect("room");
    }
    for (index, result) in [(2, 'c'), (0, 'a'), (1, 'b')] {
        let slot = results.get_mut(index).expect("a slot per call");
        *slot = Some(result);
    }
    assert_eq!(results.get_mut(3), None);
    assert_eq!(&*results.into_boxed(), &[Some('a'), Some('b'), Some('c')]);
}

#[test]
fn a_list_cleared_is_refilled_to_its_capacity() {
    let mut list = List::with_capacity(2);
    assert_eq!(list.push(1_u8), Ok(()));
    assert_eq!(list.push(2), Ok(()));
    list.clear();
    assert!(list.is_empty());
    assert_eq!(list.room(), 2, "its capacity kept");
    assert_eq!(list.push(3), Ok(()));
    assert_eq!(&*list.into_boxed(), &[3]);
}

#[test]
fn a_full_map_refuses_new_keys_but_replaces_values() {
    let mut map = Map::with_capacity(2);
    assert_eq!(map.insert('b', 2_u8), Ok(None));
    assert_eq!(map.insert('a', 1), Ok(None));
    assert_eq!(map.insert('c', 3), Err(('c', 3)));
    assert_eq!(map.insert('a', 10), Ok(Some(1)));
    assert_eq!(map.len(), 2);
    assert_eq!(map.get(&'a'), Some(&10));
    assert!(!map.contains_key(&'c'));
    assert_eq!(map.remove(&'b'), Some(2));
    assert_eq!(map.remove(&'b'), None);
    assert_eq!(map.insert('c', 3), Ok(None), "removing makes room");
}

#[test]
fn entries_come_out_in_key_order() {
    let mut map = Map::with_capacity(4);
    for key in [3_u8, 1, 4, 2] {
        map.insert(key, u32::from(key) * 10).expect("room");
    }
    for (expected, (key, value)) in (1_u8..).zip(&map) {
        assert_eq!((*key, *value), (expected, u32::from(expected) * 10));
    }
    assert_eq!(map.first(), Some((&1, &10)));
    assert_eq!(map.last(), Some((&4, &40)));
    if let Some(value) = map.get_mut(&4) {
        *value = 0;
    }
    assert_eq!(map.iter().last(), Some((&4, &0)));
}

#[test]
fn a_map_keyed_by_bytes_is_queried_by_a_slice() {
    let mut map: Map<Box<[u8]>, u8> = Map::with_capacity(1);
    map.insert(Box::from(&b"src/lib.rs"[..]), 7).expect("room");
    assert_eq!(map.get(&b"src/lib.rs"[..]), Some(&7));
    assert!(map.contains_key(&b"src/lib.rs"[..]));
    assert_eq!(map.get(&b"src"[..]), None);
    assert_eq!(map.remove(&b"src/lib.rs"[..]), Some(7));
}

#[test]
fn a_zero_capacity_map_refuses_everything() {
    let mut map = Map::with_capacity(0);
    assert_eq!(map.insert(1_u8, ()), Err((1, ())));
    assert!(map.is_empty());
    assert_eq!(map.first(), None);
}

#[test]
fn a_full_set_refuses_new_keys_only() {
    let mut set = Set::with_capacity(2);
    assert_eq!(set.insert(2_u8), Ok(true));
    assert_eq!(set.insert(1), Ok(true));
    assert_eq!(set.insert(3), Err(3));
    assert_eq!(set.insert(1), Ok(false), "a key already present takes no new room");
    assert!(set.contains(&2));
    assert_eq!((set.first(), set.last()), (Some(&1), Some(&2)));
    for (expected, key) in (1_u8..).zip(&set) {
        assert_eq!(*key, expected);
    }
    assert!(set.remove(&1));
    assert!(!set.remove(&1));
    assert_eq!(set.insert(3), Ok(true), "removing makes room");
    assert_eq!(set.len(), 2);
}

#[test]
fn the_least_key_is_taken_first_and_makes_room() {
    let mut set = Set::with_capacity(2);
    assert_eq!(set.pop_first(), None);
    assert_eq!(set.insert(5_u8), Ok(true));
    assert_eq!(set.insert(4), Ok(true));
    assert_eq!(set.pop_first(), Some(4));
    assert_eq!(set.insert(6), Ok(true), "taking a key makes room");
    assert_eq!(set.pop_first(), Some(5));
    assert_eq!(set.pop_first(), Some(6));
    assert!(set.is_empty());
}

#[test]
fn a_zero_capacity_set_refuses_everything() {
    let mut set = Set::with_capacity(0);
    assert_eq!(set.insert('a'), Err('a'));
    assert!(set.is_empty());
}

#[test]
fn a_queue_is_fifo_and_bounded() {
    let mut queue = Queue::with_capacity(2);
    queue.push(1_u8);
    assert_eq!(queue.try_push(2), Ok(()));
    assert_eq!(queue.try_push(3), Err(3));
    assert_eq!(queue.room(), 0);
    assert_eq!(queue.pop(), Some(1));
    assert_eq!(queue.pop(), Some(2));
    assert_eq!(queue.pop(), None);
}

#[test]
fn a_queue_cleared_is_refilled_to_its_capacity() {
    let mut queue = Queue::with_capacity(3);
    queue.push(1_u8);
    queue.push(2);
    assert_eq!(queue.pop(), Some(1));
    queue.push(3);
    queue.push(4);
    queue.clear();
    assert!(queue.is_empty());
    assert_eq!((queue.room(), queue.capacity()), (3, 3), "its capacity kept");
    for item in [5, 6, 7] {
        assert_eq!(queue.try_push(item), Ok(()));
    }
    assert_eq!(queue.try_push(8), Err(8));
    for item in [Some(5), Some(6), Some(7), None] {
        assert_eq!(queue.pop(), item);
    }
}

#[test]
fn clearing_an_empty_queue_changes_nothing() {
    let mut queue = Queue::<u8>::uncapped();
    queue.clear();
    assert!(queue.is_empty());
    assert_eq!(queue.room(), u32::MAX);
}

#[test]
#[should_panic(expected = "the loop reserves room for every output")]
fn pushing_past_the_reservation_is_a_bug() {
    let mut queue = Queue::with_capacity(0);
    queue.push(());
}

#[test]
fn a_stack_is_lifo_and_bounded() {
    let mut stack = Stack::with_capacity(2);
    assert_eq!(stack.push('a'), Ok(()));
    assert_eq!(stack.push('b'), Ok(()));
    assert_eq!(stack.push('c'), Err('c'));
    assert_eq!((stack.len(), stack.capacity()), (2, 2));
    assert_eq!(stack.top(), Some(&'b'));
    assert_eq!(stack.pop(), Some('b'));
    assert_eq!(stack.push('c'), Ok(()));
    assert_eq!(stack.pop(), Some('c'));
    assert_eq!(stack.pop(), Some('a'));
    assert_eq!(stack.pop(), None);
    assert!(stack.is_empty());
}

#[test]
fn a_stack_cleared_is_refilled_to_its_capacity() {
    let mut stack = Stack::with_capacity(2);
    assert_eq!(stack.push('a'), Ok(()));
    assert_eq!(stack.push('b'), Ok(()));
    stack.clear();
    assert!(stack.is_empty());
    assert_eq!(stack.top(), None);
    assert_eq!(stack.capacity(), 2, "its capacity kept");
    assert_eq!(stack.push('c'), Ok(()));
    assert_eq!(stack.push('d'), Ok(()));
    assert_eq!(stack.push('e'), Err('e'));
    assert_eq!(stack.pop(), Some('d'));
    assert_eq!(stack.pop(), Some('c'));
    assert_eq!(stack.pop(), None);
}

#[test]
fn the_top_changes_in_place() {
    let mut stack = Stack::with_capacity(3);
    assert_eq!(stack.top_mut(), None);
    assert_eq!(stack.push(1_u32), Ok(()));
    assert_eq!(stack.push(10), Ok(()));
    *stack.top_mut().expect("two items") += 1;
    assert_eq!(stack.pop(), Some(11));
    assert_eq!(stack.top(), Some(&1));
}

#[test]
fn a_zero_capacity_stack_refuses_everything() {
    let mut stack = Stack::with_capacity(0);
    assert_eq!(stack.push(()), Err(()));
    assert_eq!(Stack::<u64>::price(4), Some(32));
    assert_eq!(Stack::<u64>::price(u32::MAX), Some(8 * u64::from(u32::MAX)));
}

#[test]
fn the_price_grows_with_the_entries_and_their_size() {
    let empty = price(0, 8, 8, 8).expect("fits");
    assert!(empty > 0, "the root is priced even before it is allocated");
    assert!(price(5, 8, 8, 8).expect("fits") > empty);
    assert!(price(5, 8, 16, 8).expect("fits") > price(5, 8, 8, 8).expect("fits"));
    assert_eq!(price(u32::MAX, usize::MAX, 0, 8), None);
}

// Twice 2^32 bytes times u32::MAX exceeds u64::MAX, without allocating an item.
// Such a type is representable only on a 64-bit target.
#[cfg(target_pointer_width = "64")]
type OversizedItem = [u8; 8_589_934_592];

#[test]
fn a_list_price_that_overflows_is_none() {
    #[cfg(target_pointer_width = "64")]
    {
        assert_eq!(List::<OversizedItem>::price(u32::MAX), None);
    }
}

#[test]
fn a_queue_price_that_overflows_is_none() {
    #[cfg(target_pointer_width = "64")]
    {
        assert_eq!(Queue::<OversizedItem>::price(u32::MAX), None);
    }
}

#[test]
fn an_intake_price_fits_the_largest_capacity() {
    // A byte buffer's u32 capacity cannot overflow its u64 price.
    assert_eq!(Intake::price(u32::MAX), Some(u64::from(u32::MAX)));
}

#[test]
fn a_map_price_that_overflows_is_none() {
    #[cfg(target_pointer_width = "64")]
    {
        assert_eq!(Map::<u8, OversizedItem>::price(u32::MAX), None);
    }
}

#[test]
fn a_set_price_that_overflows_is_none() {
    #[cfg(target_pointer_width = "64")]
    {
        assert_eq!(Set::<OversizedItem>::price(u32::MAX), None);
    }
}

#[test]
fn a_stack_price_that_overflows_is_none() {
    #[cfg(target_pointer_width = "64")]
    {
        assert_eq!(Stack::<OversizedItem>::price(u32::MAX), None);
    }
}

#[test]
fn a_buffer_price_fits_the_largest_capacity() {
    // A byte buffer's u32 capacity cannot overflow its u64 price.
    assert_eq!(Buffer::price(u32::MAX), Some(u64::from(u32::MAX)));
}

#[test]
fn removing_a_list_item_moves_the_last_and_keeps_capacity() {
    // Three items distinguish first, middle, last and the first absent position.
    const ITEMS: [u8; 3] = [3, 7, 11];
    const CAPACITY: u32 = 3;
    for (index, removed, remaining) in [
        (0, Some(3), &[11, 7][..]),
        (1, Some(7), &[3, 11][..]),
        (2, Some(11), &[3, 7][..]),
        (3, None, &[3, 7, 11][..]),
        (u32::MAX, None, &[3, 7, 11][..]),
    ] {
        let mut list = List::from_boxed(Box::new(ITEMS));
        let buffer = list.as_slice().as_ptr();
        assert_eq!(list.swap_remove(index), removed, "position {index}");
        assert_eq!(list.as_slice(), remaining, "position {index}");
        assert_eq!(list.capacity(), CAPACITY, "position {index}");
        assert_eq!(list.as_slice().as_ptr(), buffer, "the grown buffer stays");
        if removed.is_some() {
            assert_eq!(list.push(13), Ok(()), "removing makes room");
            assert_eq!(list.as_slice().as_ptr(), buffer, "refilling allocates nothing");
        }
        assert_eq!(list.push(17), Err(17), "the cap holds");
    }
    assert_eq!(List::<u8>::with_capacity(CAPACITY).swap_remove(0), None);
}

#[test]
fn list_items_change_in_place_in_push_order() {
    let mut list = List::from_boxed(Box::new([3_u8, 7, 11]));
    for (item, value) in list.iter_mut().zip([5, 9, 13]) {
        *item = value;
    }
    assert_eq!(list.as_slice(), &[5, 9, 13]);
    list.clear();
    assert_eq!(list.iter_mut().next(), None);
}

// Two rows let a hit save the older entry; a miss must leave its order alone.
fn lru_pair(capacity: u32) -> Lru<u8, u8> {
    let mut table = Lru::with_capacity(capacity);
    for (key, value) in [(2, 20), (1, 10)] {
        assert_eq!(table.insert(key, value), None);
    }
    table
}

#[test]
fn a_lru_evicts_by_use_and_a_get_saves_an_entry() {
    for (get, found, evicted) in [(9, None, (2, 20)), (2, Some(20), (1, 10)), (1, Some(10), (2, 20))] {
        let mut table = lru_pair(2);
        assert_eq!(table.get(get), found.as_ref());
        assert_eq!(table.insert(3, 30), Some(evicted));
        assert_eq!(table.get(evicted.0), None);
        assert_eq!(table.len(), 2);
        assert!(!table.is_empty());
    }
}

#[test]
fn a_lru_replaces_without_evicting_another_entry() {
    for (key, old, next) in [(2, 20, (1, 10)), (1, 10, (2, 20))] {
        let mut table = lru_pair(2);
        assert_eq!(table.insert(key, 99), Some((key, old)));
        assert_eq!(table.len(), 2);
        assert_eq!(table.insert(3, 30), Some(next), "replacement refreshes its use");
        assert_eq!(table.get(key), Some(&99));
    }
}

#[test]
fn a_lru_remove_makes_room_and_an_absent_key_changes_nothing() {
    for (key, removed, length, evicted) in [(2, Some(20), 1, None), (1, Some(10), 1, None), (9, None, 2, Some((2, 20)))]
    {
        let mut table = lru_pair(2);
        assert_eq!(table.remove(key), removed);
        assert_eq!(table.remove(key), None);
        assert_eq!(table.len(), length);
        assert_eq!(table.insert(3, 30), evicted);
        for key in [1, 2, 3] {
            let _removed = table.remove(key);
        }
        assert!(table.is_empty());
    }
}

#[test]
fn a_zero_capacity_lru_hands_every_pair_back() {
    let mut table = Lru::<u8, u8>::with_capacity(0);
    for (key, value) in [(1, 10), (1, 20), (2, 30)] {
        assert_eq!(table.insert(key, value), Some((key, value)));
        assert_eq!(table.get(key), None);
        assert_eq!(table.remove(key), None);
        assert_eq!(table.len(), 0);
        assert!(table.is_empty());
    }
}

#[test]
fn a_lru_hash_covers_values_and_use_order_without_its_history() {
    let original = state_digest(&lru_pair(2));
    for (capacity, refresh, value, alike) in [(2, 1, 10, true), (9, 1, 10, true), (2, 2, 20, false), (2, 1, 99, false)]
    {
        let mut table = lru_pair(capacity);
        assert_eq!(table.insert(refresh, value), Some((refresh, if refresh == 1 { 10 } else { 20 })));
        assert_eq!(state_digest(&table) == original, alike);
    }
    assert_ne!(state_digest(&Lru::<u8, u8>::with_capacity(2)), original);
}

#[test]
fn a_join_gives_each_key_once_with_values_in_map_order() {
    struct NotClone;
    fn needs_copy<T: Copy>() {}

    let mut first = Map::with_capacity(3);
    let mut second = Map::with_capacity(3);
    for (key, value) in [(1_u8, 1_u8), (3, 3), (4, 4)] {
        first.insert(key, value).expect("room");
    }
    for (key, value) in [(2, 2), (3, 30), (5, 5)] {
        second.insert(key, value).expect("room");
    }
    let mut join = first.join(&second);
    assert_eq!(join.next(), Some((&1, Joined::First(&1))));
    assert_eq!(join.next(), Some((&2, Joined::Second(&2))));
    let (key, values) = join.next().expect("shared key");
    assert_eq!((key, values), (&3, Joined::Both(&3, &30)));
    let (first_key, _) = first.iter().nth(1).expect("shared key");
    assert!(core::ptr::eq(key, first_key), "the first map supplies a shared key");
    assert_eq!(join.next(), Some((&4, Joined::First(&4))));
    assert_eq!(join.next(), Some((&5, Joined::Second(&5))));
    assert_eq!(join.next(), None);
    assert_eq!(join.next(), None);

    needs_copy::<Joined<'_, NotClone>>();
}

#[test]
fn joins_cover_empty_sides_and_all_shared_keys() {
    let empty = Map::<u8, u8>::with_capacity(0);
    let mut first = Map::with_capacity(2);
    let mut second = Map::with_capacity(2);
    for (key, value) in [(1_u8, 10_u8), (3, 30)] {
        first.insert(key, value).expect("room");
        second.insert(key, value.checked_add(1).expect("small value")).expect("room");
    }
    assert_eq!(empty.join(&empty).next(), None);
    let mut second_only = empty.join(&first);
    let mut first_only = first.join(&empty);
    for (key, value) in &first {
        assert_eq!(second_only.next(), Some((key, Joined::Second(value))));
        assert_eq!(first_only.next(), Some((key, Joined::First(value))));
    }
    assert_eq!(second_only.next(), None);
    assert_eq!(first_only.next(), None);
    let mut shared = first.join(&second);
    assert_eq!(shared.next(), Some((&1, Joined::Both(&10, &11))));
    assert_eq!(shared.next(), Some((&3, Joined::Both(&30, &31))));
    assert_eq!(shared.next(), None);
}

// These generators and references also run long in tests/lib/src/containers.rs.
fn random_maps(rng: &mut Rng, round: u32) -> (Map<u8, u8>, Map<u8, u8>) {
    let mut first = Map::with_capacity(32);
    let mut second = Map::with_capacity(32);
    for key in 0_u8..32 {
        let in_first = rng.chance(500);
        let in_second = match round % 3 {
            0 => in_first,        // All shared.
            1 => !in_first,       // Disjoint and interleaved.
            _ => rng.chance(500), // A mixture of all three variants.
        };
        let value = u8::try_from(rng.below(8)).expect("small value");
        let other = if rng.chance(500) { value } else { value.checked_add(1).expect("small value") };
        if in_first {
            first.insert(key, value).expect("room for every key");
        }
        if in_second {
            second.insert(key, other).expect("room for every key");
        }
    }
    (first, second)
}

fn naive_join<'a>(first: &'a Map<u8, u8>, second: &'a Map<u8, u8>) -> alloc::collections::BTreeMap<u8, Joined<'a, u8>> {
    let mut keys = alloc::collections::BTreeSet::new();
    for (key, _) in first.iter().chain(second) {
        let _: bool = keys.insert(*key);
    }
    let mut entries = alloc::collections::BTreeMap::new();
    for key in keys {
        let joined = match (first.get(&key), second.get(&key)) {
            (Some(a), Some(b)) => Joined::Both(a, b),
            (Some(a), None) => Joined::First(a),
            (None, Some(b)) => Joined::Second(b),
            (None, None) => unreachable!("key from either map"),
        };
        assert!(entries.insert(key, joined).is_none());
    }
    entries
}

#[test]
fn seeded_joins_agree_with_key_union_and_lookups() {
    let mut rng = Rng::new(0xA01A_5EED);
    for round in 0..256 {
        let (first, second) = random_maps(&mut rng, round);
        let expected = naive_join(&first, &second);
        let mut joined = first.join(&second);
        for (key, values) in &expected {
            assert_eq!(joined.next(), Some((key, *values)), "round {round}");
        }
        assert_eq!(joined.next(), None, "round {round}");
    }
}

fn bisect(values: &[i16], key: i16) -> usize {
    let mut search = Bisection::over(values.len());
    for _ in 0..Bisection::PROBES {
        let Some(probe) = search.probe() else { break };
        let value = values.get(probe).expect("probe within the slice");
        search.narrow(value.cmp(&key));
    }
    assert_eq!(search.probe(), None, "within PROBES, length {}, key {key}", values.len());
    search.end()
}

#[test]
fn bisection_finds_the_first_equal_or_insertion_index() {
    for (key, expected) in [(3, 1), (4, 3), (0, 0), (6, 4)] {
        assert_eq!(bisect(&[1, 3, 3, 5], key), expected, "key {key}");
    }
}

fn linear_bound(values: &[i16], key: i16) -> usize {
    for (index, value) in values.iter().enumerate() {
        if *value >= key {
            return index;
        }
    }
    values.len()
}

fn check_bisections(values: &[i16]) {
    // Values are even and in 0..=42: every held key, every gap and both ends.
    for key in -1..=43 {
        let expected = linear_bound(values, key);
        assert_eq!(bisect(values, key), expected, "slice {values:?}, key {key}");
    }
}

#[test]
fn bisection_matches_a_scan_for_every_short_length() {
    let mut buffer = [0_i16; 64];
    for (index, value) in buffer.iter_mut().enumerate() {
        *value = i16::try_from((index / 3).checked_mul(2).expect("small index")).expect("small key");
    }
    for len in 0..=64 {
        check_bisections(&buffer[..len]);
    }
}

fn random_sorted_slice<'a>(rng: &mut Rng, buffer: &'a mut [i16; 64]) -> &'a [i16] {
    let len = usize::try_from(rng.between(0, 64)).expect("short slice");
    let values = &mut buffer[..len];
    for value in values.iter_mut() {
        *value = i16::try_from(rng.below(22)).expect("small key").checked_mul(2).expect("small key");
    }
    values.sort_unstable();
    values
}

#[test]
fn seeded_bisections_agree_with_a_linear_scan() {
    let mut rng = Rng::new(0xB15E_5EED);
    let mut buffer = [0_i16; 64];
    for _ in 0_u32..256 {
        let values = random_sorted_slice(&mut rng, &mut buffer);
        check_bisections(values);
    }
}

#[test]
fn bisection_covers_the_full_index_range_within_its_probe_bound() {
    use core::cmp::Ordering::{Greater, Less};
    for (order, expected) in [(Less, usize::MAX), (Greater, 0)] {
        let mut search = Bisection::over(usize::MAX);
        let mut probes = 0_u32;
        for _ in 0..Bisection::PROBES {
            let Some(probe) = search.probe() else { break };
            assert!(probe < usize::MAX);
            probes = probes.checked_add(1).expect("at most PROBES");
            search.narrow(order);
        }
        assert_eq!(search.probe(), None);
        assert_eq!(search.end(), expected);
        assert_eq!(probes, Bisection::PROBES);
    }
}
