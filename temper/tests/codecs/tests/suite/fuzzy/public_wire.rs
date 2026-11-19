//! Seeded visitor record decoding (temper/docs/design/client/forge/README.md, 9.4).
use temper_client_protocol_wire::v1::Record;
use skein_lib::{List, Reader, Rng, Writer};
use temper_client_protocol_wire::public_v1 as w;

// Match client_wire.rs's 2,000 draws, with each draw independently replayable.
const SEEDS: u64 = 2_000;
// Arbitrary inputs span the same 512-byte range as client_wire.rs.
const ARBITRARY_BYTES: u32 = 512;

// Lengths precede gates (with full optional step data), landings, and members in public-v1.schema.
const CHANGE_LENGTH: usize = 2 * size_of::<u16>()
    + 2 * (size_of::<u8>() + 32)
    + size_of::<u8>()
    + 2 * size_of::<u8>()
    + size_of::<u64>()
    + size_of::<u32>()
    + 2 * size_of::<u8>()
    + size_of::<u8>();
const LANDINGS_LENGTH: usize = size_of::<u16>() + size_of::<u8>() + size_of::<u16>();
const LANDING_LENGTH: usize = size_of::<u16>() + size_of::<u8>() + 2 * size_of::<u8>() + size_of::<u32>();

#[derive(Clone, Copy, Debug)]
enum Kind {
    Change,
    Landings,
    Landing,
}

macro_rules! decode {
    ($record:ident, $reader:ident, $refuse:ident, $path:ident) => {{
        let result = w::$record::decode(&w::CEILINGS, &mut $reader);
        if $refuse {
            assert_eq!(result.as_ref().expect_err("length past ceiling must be refused").path, w::paths::$path);
        }
        if let Ok(record) = result {
            if !$reader.is_empty() {
                return None;
            }
            let mut writer = Writer::new(usize::try_from(record.measure(&temper_client_protocol_wire::public_v1::CEILINGS).expect("within ceilings")).expect("measured length"));
            record.write(&temper_client_protocol_wire::public_v1::CEILINGS, &mut writer).expect("measured room");
            Some(writer.finish())
        } else {
            None
        }
    }};
}

fn decode(kind: Kind, bytes: &[u8], refuse: bool) -> Option<Box<[u8]>> {
    let mut reader = Reader::new(bytes);
    match kind {
        Kind::Change => decode!(PublicChange, reader, refuse, PUBLIC_CHANGE_GATES),
        Kind::Landings => decode!(PublicLandings, reader, refuse, PUBLIC_LANDINGS_LANDINGS),
        Kind::Landing => decode!(PublicLanding, reader, refuse, PUBLIC_LANDING_MEMBERS),
    }
}

fn check(kind: Kind, bytes: &[u8], context: &str, case: &str, valid: bool, refuse: bool) {
    // Seed, kind and case replay the input without dumping the full bound fixture.
    eprintln!("{context}{kind:?}, {case}, input length {}", bytes.len());
    let encoded = decode(kind, bytes, refuse);
    if valid {
        assert!(encoded.is_some(), "{context}{kind:?}, {case}: bound fixture must decode");
    }
    if let Some(encoded) = encoded {
        assert_eq!(encoded.as_ref(), bytes, "{context}{kind:?}, {case}");
    }
}

#[test]
fn public_records_decoders_never_panic_and_reencode_the_same() {
    eprintln!("seed 1: build bound records");
    let fixtures = bound_records();
    for (kind, golden, length_at, ceiling) in &fixtures {
        fixed(*kind, golden, *length_at, *ceiling);
    }
    for seed in 1..=SEEDS {
        eprintln!("seed {seed}: public record sweep");
        sweep(seed, &fixtures);
    }
}

fn sweep(seed: u64, fixtures: &[(Kind, Box<[u8]>, usize, u32); 3]) {
    let context = format!("seed {seed}, ");
    let mut rng = Rng::new(seed);
    let arbitrary = temper_codecs_world::drawn(&mut rng, ARBITRARY_BYTES);
    for (kind, golden, _, _) in fixtures {
        check(*kind, &arbitrary, &context, "arbitrary", false, false);
        let length = skein_lib::len_u64(golden.len());
        let at = usize::try_from(rng.below(length)).expect("seed: byte position fits usize");
        let mut bytes = golden.to_vec();
        *bytes.get_mut(at).expect("seed: chosen byte") ^= 1 << rng.below(u64::from(u8::BITS));
        check(*kind, &bytes, &context, "flipped", false, false);
        bytes.clear();
        bytes.extend_from_slice(golden);
        let inserted_at = usize::try_from(rng.between(0, length)).expect("seed: insertion position fits");
        bytes.insert(inserted_at, u8::try_from(rng.below(256)).expect("seed: one byte"));
        check(*kind, &bytes, &context, "inserted", false, false);
        bytes.clear();
        bytes.extend_from_slice(golden);
        bytes.remove(at);
        check(*kind, &bytes, &context, "removed", false, false);
    }
}

fn fixed(kind: Kind, golden: &[u8], length_at: usize, ceiling: u32) {
    check(kind, golden, "", "at every bound", true, false);
    let mut bytes = golden.to_vec();
    let end = length_at.checked_add(size_of::<u32>()).expect("length prefix fits");
    bytes
        .get_mut(length_at..end)
        .expect("length prefix")
        .copy_from_slice(&ceiling.checked_add(1).expect("ceiling has successor").to_be_bytes());
    check(kind, &bytes, "", "length past ceiling", false, true);
}

fn bound_records() -> [(Kind, Box<[u8]>, usize, u32); 3] {
    let limits = &w::CEILINGS;
    let change = change(limits);
    let mut writer = Writer::new(usize::try_from(change.measure(&temper_client_protocol_wire::public_v1::CEILINGS).expect("within ceilings")).expect("seed 1: measured length"));
    change.write(&temper_client_protocol_wire::public_v1::CEILINGS, &mut writer).expect("seed 1: measured room");
    let change = writer.finish();
    let landings = landings(limits);
    let mut writer = Writer::new(usize::try_from(landings.measure(&temper_client_protocol_wire::public_v1::CEILINGS).expect("within ceilings")).expect("seed 1: measured length"));
    landings.write(&temper_client_protocol_wire::public_v1::CEILINGS, &mut writer).expect("seed 1: measured room");
    let landings = writer.finish();
    let landing = landing(limits);
    let mut writer = Writer::new(usize::try_from(landing.measure(&temper_client_protocol_wire::public_v1::CEILINGS).expect("within ceilings")).expect("seed 1: measured length"));
    landing.write(&temper_client_protocol_wire::public_v1::CEILINGS, &mut writer).expect("seed 1: measured room");
    let landing = writer.finish();
    [
        (Kind::Change, change, CHANGE_LENGTH, limits.public_change_gates),
        (Kind::Landings, landings, LANDINGS_LENGTH, limits.public_landings_landings),
        (Kind::Landing, landing, LANDING_LENGTH, limits.public_landing_members),
    ]
}

fn gate() -> w::PublicGate {
    w::PublicGate {
            number: u64::MAX,
            kind: w::PublicGateKind::Check,
            blocks: true,
            verdict: w::PublicVerdict::Changes,
        }
}

fn used() -> w::PublicUsed {
    w::PublicUsed { used: u32::MAX, limit: u32::MAX }
}

fn change(limits: &w::Limits) -> w::PublicChange {
    let mut gates = List::with_capacity(limits.public_change_gates);
    for _ in 0..limits.public_change_gates {
        gates.push(gate()).expect("seed 1: fixture within ceilings");
    }
    w::PublicChange {
            task: u16::MAX,
            head: Some([255; 32]),
            step: w::PublicStep::Repairing,
            repair: Some(w::PublicRepair::Gate(
                w::PublicGateNumber { gate: u64::MAX },
            )),
            merge: Some([255; 32]),
            tiers: u32::MAX,
            hold: Some(w::PublicHold::Authority),
            ci: w::PublicCi::Failed,
            gates,
            repairs: used(),
            resolutions: used(),
            updates: used(),
            way: w::PublicWay::Turn,
            in_landing: true,
        }
}

fn line(limits: &w::Limits) -> w::PublicLandingLine {
    let mut members = List::with_capacity(limits.public_landing_line_members);
    for _ in 0..limits.public_landing_line_members {
        members.push(u16::MAX).expect("seed 1: fixture within ceilings");
    }
    let mut landed = List::with_capacity(limits.public_landing_line_landed);
    for _ in 0..limits.public_landing_line_landed {
        landed.push([255; 32]).expect("seed 1: fixture within ceilings");
    }
    w::PublicLandingLine { members, phase: w::PublicLandingPhase::Landed, landed }
}

fn queued() -> w::PublicQueued {
    w::PublicQueued { task: u16::MAX, goal: u16::MAX, ready_since: u64::MAX }
}

fn queue(limits: &w::Limits) -> w::PublicQueue {
    let mut changes = List::with_capacity(limits.public_queue_changes);
    for _ in 0..limits.public_queue_changes {
        changes.push(queued()).expect("seed 1: fixture within ceilings");
    }
    w::PublicQueue { changes }
}

fn landings(limits: &w::Limits) -> w::PublicLandings {
    let mut landings = List::with_capacity(limits.public_landings_landings);
    for _ in 0..limits.public_landings_landings {
        landings.push(line(limits)).expect("seed 1: fixture within ceilings");
    }
    let mut queues = List::with_capacity(limits.public_landings_queues);
    for _ in 0..limits.public_landings_queues {
        queues.push(queue(limits)).expect("seed 1: fixture within ceilings");
    }
    w::PublicLandings { turn: Some(u16::MAX), landings, queues }
}

fn proof() -> w::PublicGateProof {
    w::PublicGateProof {
            faults: 0,
            number: u64::MAX,
            kind: w::PublicGateKind::Check,
            blocks: true,
            proof: w::PublicProof::Failed,
        }
}

fn member(limits: &w::Limits) -> w::PublicMember {
    let mut gates = List::with_capacity(limits.public_member_gates);
    for _ in 0..limits.public_member_gates {
        gates.push(proof()).expect("seed 1: fixture within ceilings");
    }
    w::PublicMember {
            head: None,
            left: None,
            judgement: None,
            task: u16::MAX,
            base: Some([255; 32]),
            candidate: Some([255; 32]),
            landed: Some([255; 32]),
            gates,
        }
}

fn landing(limits: &w::Limits) -> w::PublicLanding {
    let mut members = List::with_capacity(limits.public_landing_members);
    for _ in 0..limits.public_landing_members {
        members.push(member(limits)).expect("seed 1: fixture within ceilings");
    }
    w::PublicLanding {
            search: None,
            combined: None,
            step: None,
            phase: w::PublicLandingPhase::Held,
            hold: Some(w::PublicLandingHold::Faults),
            preparation: u32::MAX,
            members,
            stale_proofs: u32::MAX,
            holds_turn: true,
        }
}
