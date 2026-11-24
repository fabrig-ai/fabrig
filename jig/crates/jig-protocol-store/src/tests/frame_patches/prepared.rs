//! Prepared schemas for the frame machines and their admission check (protocol/store/transcripts.md, 6).
use super::{
    BUDGET, PATH, REPLAY, SETS, answer, expected, frames, jig, payload, request, scene_with_schemas, smith, turn,
};
use crate::{Frames, KeptFamily, Owner, OwnerSet, OwnerSets, Store, transcripts::patches, upgrade::admission};
use skein_codec::schema::{Schema, parse};

const BODY: &[&str] = jig::kept::fixture_frame::TEXTS;
const TURNS: &[&str] = smith::kept::fixture_turn::TEXTS;
const FIRST: &str = TURNS[0];
const SECOND: &str = TURNS[1];

fn prepare(texts: &[&str]) -> &'static [Schema] {
    Box::leak(texts.iter().map(|text| parse(text.as_bytes()).expect("fixture schema")).collect())
}

fn read_behind(prepared: &'static [Schema]) -> Store {
    let bytes = frames::frame(5, &turn(1, REPLAY));
    let (mut disk, mut store) = scene_with_schemas(&bytes, SETS, prepared);
    let events = disk.request(&mut store, request(1, BUDGET));
    let events = answer(&mut disk, &mut store, events, false);
    assert_eq!(payload(&events), &*expected(1));
    assert_eq!(&*disk.bytes(PATH), &*bytes);
    store
}

#[test]
fn prepared_schemas_admit_and_make_the_same_serving_machines() {
    let cold = read_behind(&[]);
    let prepared = read_behind(prepare(&[BODY[0], BODY[1], FIRST, SECOND]));
    for (cold, prepared) in [
        (&cold.transcripts.machines.body, &prepared.transcripts.machines.body),
        (&cold.transcripts.machines.turn, &prepared.transcripts.machines.turn),
    ] {
        let cold = cold.as_ref().expect("cold machine");
        let prepared = prepared.as_ref().expect("prepared machine");
        assert_eq!((prepared.last, prepared.capacity, prepared.root), (cold.last, cold.capacity, cold.root));
    }
}

#[test]
fn an_altered_prepared_text_does_not_replace_a_kept_text() {
    // The same family and version as FIRST, but Burn is not the Turn the patch names.
    const ALTERED: &str = "family fixture-turn 1\nversioned record Burn { said: text 64 replay: bytes 64 }\n";
    let store = read_behind(prepare(&[BODY[0], BODY[1], ALTERED, SECOND]));
    let machine = store.transcripts.machines.turn.expect("turn machine").patcher.expect("resident patcher");
    assert_eq!(machine.schema(1).expect("first turn").text(), FIRST.as_bytes());
}

#[test]
fn prepared_schemas_do_not_bypass_frame_admission_checks() {
    const THIRD: &str = "family fixture-turn 3\nversioned record Turn { said: text 64 tokens: u32 }\n";
    const TO_SECOND: &str = smith::PATCHES[0];
    const TO_THIRD: &str = "patch 1\nowner fixture-smith 2\nfamily fixture-turn 2 to 3\n";
    const CASES: &[(&str, &[&str], &[&str])] = &[
        ("out of order", &[SECOND, FIRST, THIRD], &[TO_SECOND, TO_THIRD]),
        ("invalid patch", &[FIRST, SECOND, THIRD], &[TO_SECOND, TO_THIRD, "not a patch"]),
        ("missing plan", &[FIRST, SECOND, THIRD], &[TO_SECOND]),
    ];
    let prepared = prepare(&[FIRST, SECOND, THIRD]);
    for &(case, texts, patches) in CASES {
        let smith = SETS.sets[1];
        let families = Box::leak(Box::new([KeptFamily {
            current: THIRD,
            frames: Some(Frames { root: smith.families[0].frames.expect("turn frames").root, texts, patches }),
            ..smith.families[0]
        }]));
        let sets = OwnerSets { sets: Box::leak(Box::new([OwnerSet { families, ..smith }])) };
        for schemas in [&[], prepared] {
            assert_eq!(admission::frames(&sets, schemas), Err(Owner::Smith), "{case}");
        }
    }
}

#[test]
fn kept_text_room_is_the_same_with_prepared_schemas() {
    const FRAME: Frames = Frames { root: "Turn", texts: &[SECOND], patches: &[] };
    let prepared = prepare(FRAME.texts);
    let at = patches::frame_capacity(&FRAME, patches::Room { bytes: 0, nodes: 0 }).expect("frame capacity");
    let short = skein_codec::patch::Capacity { text: at.text.checked_sub(1).expect("nonempty text"), ..at };
    for schemas in [&[], prepared] {
        assert!(patches::kept_patcher(&FRAME, at, schemas).is_some(), "at the text limit");
        assert!(patches::kept_patcher(&FRAME, short, schemas).is_none(), "one past the text limit");
    }
}
