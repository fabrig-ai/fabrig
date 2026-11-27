//! Each kept frame family opens and decodes independently of the maps.
use crate::frame_reads;
use frame_reads::{SETS, closed, decode, open, turn};
use jig_protocol_store::OwnerSets;
use jig_test_layouts::frames;
const TURN_ONLY: OwnerSets = OwnerSets { sets: &[SETS.sets[1]] };

#[test]
fn frame_plans_decode_with_both_families_or_only_the_turn() {
    let limits = jig_store_world::upgrade_fixtures::limits();
    for (label, sets, body) in [("body and turn", SETS, 5), ("turn", TURN_ONLY, 6)] {
        let raw = frames::frame(body, &turn(1, b"hello", b"r", 0));
        let (mut component, env) = closed(sets, &limits);
        open(&mut component, &env);
        let result = decode(&mut component, &env, &raw);
        assert!(result.is_some(), "{label}");
    }
}

#[test]
fn a_warm_frame_conversion_reuses_its_buffer_and_does_not_index_nodes() {
    use jig_protocol_store::{KeptFamily, OwnerSet};
    // Many nodes occupy only their four-byte list count. A generic value
    // index would be proportional to the count, not the bytes on the edge.
    const OLD: &str = "family fixture-turn 1\nrecord Empty {}\nversioned record Turn { nodes: list 512 Empty }\n";
    const NEW: &str =
        "family fixture-turn 2\nrecord Empty {}\nversioned record Turn { nodes: list 512 Empty added: u8 = 7 }\n";
    const SMITH: OwnerSet =
        OwnerSet { families: &[KeptFamily { current: NEW, ..SETS.sets[1].families[0] }], ..SETS.sets[1] };
    const OWNERS: OwnerSets = OwnerSets { sets: &[SETS.sets[0], SMITH] };
    let (mut component, env) =
        frame_reads::closed_with_old_turn(OWNERS, &jig_store_world::upgrade_fixtures::limits(), OLD);
    open(&mut component, &env);
    let raw = frames::frame(5, &[0, 1, 0, 0, 2, 0]);
    let expected = [0, 2, 0, 0, 2, 0, 7];
    assert_eq!(decode(&mut component, &env, &raw).as_deref(), Some(expected.as_slice()));
    let native = frames::frame(6, &expected);
    let baseline = skein_heap::Span::start();
    assert_eq!(decode(&mut component, &env, &native).as_deref(), Some(expected.as_slice()));
    let made = baseline.made();
    assert_eq!(baseline.end().net, 0);
    let converted = skein_heap::Span::start();
    assert_eq!(decode(&mut component, &env, &raw).as_deref(), Some(expected.as_slice()));
    assert_eq!(converted.made(), made, "a warm pass allocates no more than a current read");
    let growth = converted.end();
    // Allow the file response and event/turn cells, but not a second 4 KiB
    // conversion buffer or a generic index over the 512 nodes.
    assert!(growth.peak < 4096, "{growth:?}");
    assert_eq!(growth.net, 0);
}
