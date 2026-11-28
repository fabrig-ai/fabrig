//! Hold the stock wakes' paragraphs and room when a charter file moves
//! (temper/docs/design/domain/agent/room.md, 4.2 and 4.3).
#![expect(
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    reason = "the test embeds the overlay files and sorts paragraphs of composed Markdown"
)]
use jig_domain_core::settings::{Overlay, overlaid};
use jig_domain_core::{Charter, CharterPart};
use jig_domain_tasks::Reasons;
use skein_lib::Writer;
use temper_charter_set::{Set, goal_agent, implementer};

const OVERLAY: Set<'static> = Set {
    common: Some(include_str!("fixtures/wake_overlay/common.md")),
    goal: Some(include_str!("fixtures/wake_overlay/goal.md")),
    design: None,
    designer: Some(include_str!("fixtures/wake_overlay/designer.md")),
    coordinator: Some(include_str!("fixtures/wake_overlay/coordinator.md")),
    implementer: Some(include_str!("fixtures/wake_overlay/implementer.md")),
};

#[derive(Debug)]
struct Case {
    charter: u32,
    overlay: bool,
    room: u64,
    // Each wake: sorted paragraph digest, fixed owned bytes, fixed encoded bytes.
    wakes: [(u64, u64, u64); 3],
}

// From the five-file stock, before moving the Status paragraph.
const PINS: [Case; 4] = [
    Case {
        charter: 5,
        overlay: false,
        room: 313_009,
        wakes: [
            (3_986_125_627_215_283_144, 46_215, 44_980),
            (11_887_380_670_001_651_786, 34_403, 33_168),
            (14_362_533_416_734_165_073, 37_066, 35_831),
        ],
    },
    Case {
        charter: 5,
        overlay: true,
        room: 312_599,
        wakes: [
            (2_464_413_717_050_845_429, 46_580, 45_345),
            (455_444_910_648_753_004, 34_709, 33_474),
            (9_608_735_005_527_681_013, 37_417, 36_182),
        ],
    },
    Case { charter: 2, overlay: false, room: 343_288, wakes: [(10_487_615_259_973_974_260, 15_731, 15_378); 3] },
    Case { charter: 2, overlay: true, room: 343_113, wakes: [(3_409_087_444_601_038_589, 15_906, 15_553); 3] },
];

#[test]
fn stock_wakes_keep_their_paragraphs_and_room() {
    let engine = super::super::stock::initial_engine();
    let host = temper_engine_smith::room::host_part(
        Some(&super::agent().run_limits),
        65_536,
        &crate::settings::charter_endpoints(engine),
        &engine.limits.core.brief_core_budgets,
    );
    let question = Reasons::named(b"question").expect("question wake");
    let result = Reasons::named(b"result").expect("result wake");
    let observed = PINS.each_ref().map(|pin| {
        let default = engine.charters.iter().find(|charter| charter.number == pin.charter).expect("stock charter");
        let with_overlay;
        let charter = if pin.overlay {
            with_overlay = overlaid(default, &overlay(pin.charter));
            &with_overlay
        } else {
            default
        };
        let wakes = [Reasons::FIRST, question, question.union(result)].map(|wake| {
            let cost = host.on_wake(charter, wake);
            (paragraph_digest(charter, wake), cost.owned, cost.encoded)
        });
        Case { charter: pin.charter, overlay: pin.overlay, room: host.room(charter), wakes }
    });
    let fits = observed.iter().zip(&PINS).all(|(found, pin)| {
        found.room >= pin.room
            && found.wakes.iter().zip(pin.wakes).all(
                |(&(digest, owned, encoded), (expected_digest, max_owned, max_encoded))| {
                    digest == expected_digest && owned <= max_owned && encoded <= max_encoded
                },
            )
    });
    assert!(fits, "wakes first, question, question + result; (digest, owned, encoded):\n{observed:#?}");
}

fn overlay(number: u32) -> Overlay {
    let mut overlay = Overlay::empty();
    if number == 5 {
        let goal = goal_agent(&OVERLAY, |name| Reasons::named(name.as_bytes())).expect("overlay's goal files");
        overlay.instructions = goal.instructions;
        overlay.parts = goal
            .parts
            .into_iter()
            .map(|part| {
                let mut on = Reasons::EMPTY;
                for reason in part.reasons {
                    on = on.union(reason);
                }
                CharterPart { on, instructions: part.text }
            })
            .collect();
    } else {
        assert_eq!(number, 2);
        overlay.instructions = implementer(&OVERLAY);
    }
    overlay
}

fn paragraph_digest(charter: &Charter, wake: Reasons) -> u64 {
    // jig's run_instructions with no task amendment and not a standing session.
    let parts = charter.parts.iter().filter(|part| !charter.policy.resume && part.on.meets(wake));
    let length = parts.clone().fold(charter.policy.instructions.len(), |length, part| {
        length.checked_add(2).and_then(|n| n.checked_add(part.instructions.len())).expect("test text fits")
    });
    let mut text = Writer::new(length);
    text.put(&charter.policy.instructions).expect("instructions counted");
    for part in parts {
        text.put(b"\n\n").expect("separator counted");
        text.put(&part.instructions).expect("part counted");
    }
    let text = text.finish();
    let mut paragraphs: Box<[_]> = core::str::from_utf8(&text).expect("charter Markdown").split("\n\n").collect();
    paragraphs.sort_unstable();
    // FNV-1a over the sorted paragraphs, including their delimiters.
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for paragraph in paragraphs {
        for byte in paragraph.bytes().chain(*b"\n\n") {
            hash = (hash ^ u64::from(byte)).wrapping_mul(0x0100_0000_01b3);
        }
    }
    hash
}
