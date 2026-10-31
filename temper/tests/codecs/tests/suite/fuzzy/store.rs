//! Arbitrary and mutated durable rows and keys (protocol/store.md, section 6).
use skein_lib::Rng;
use temper_engine_domain_forge as forge;
use temper_protocol_store::{Bounds, keys, records};

use crate::fixture;
use crate::protection;
use fixture::LIMITS;

fn record(bytes: &[u8], bounds: &Bounds) {
    if let Ok(row) = records::decode(bytes, bounds) {
        assert_eq!(records::encode(&row).as_ref(), bytes, "durable row");
        let key = forge::stored_key(&row);
        let encoded = keys::encode(&key, u32::MAX).expect("a decoded row's key fits");
        assert_eq!(keys::decode(&encoded, &LIMITS), Some(key));
        assert_eq!(records::row(&encoded, bytes, bounds), Ok(row));
    }
}
fn key(bytes: &[u8]) {
    if let Some(key) = keys::decode(bytes, &LIMITS) {
        assert_eq!(keys::encode(&key, u32::MAX).expect("decoded key fits").as_ref(), bytes, "durable key");
    }
}
fn golden(bytes: &[u8], bounds: &Bounds) {
    record(bytes, bounds);
    // Value's goldens hold each family; prepend Row's version to reach its decoder.
    let mut versioned = vec![0_u8, 10];
    versioned.extend_from_slice(bytes);
    record(&versioned, bounds);
}
/// A drawn row, its key, and one mutation of each.
fn row_and_key(row: &forge::Stored, rng: &mut Rng, bounds: &Bounds) {
    let bytes = records::encode(row);
    record(&bytes, bounds);
    record(&temper_codecs_world::mutated(rng, &bytes), bounds);
    let bytes = keys::encode(&forge::stored_key(row), u32::MAX).expect("drawn key");
    key(&bytes);
    key(&temper_codecs_world::mutated(rng, &bytes));
}
fn end_act(rng: &mut Rng, bounds: &Bounds) {
    let act = if rng.below(2) == 0 {
        forge::DemoActKind::Close {
            project: u32::try_from(rng.below(u64::from(u32::MAX))).expect("u32"),
            task: rng.next_u64(),
        }
    } else {
        forge::DemoActKind::RemoveAccount { account: u32::try_from(rng.below(u64::from(u32::MAX))).expect("u32") }
    };
    let row = forge::Stored::DemoAct(forge::DemoAct { key: [u8::try_from(rng.below(256)).expect("byte"); 16], act });
    let bytes = records::encode(&row);
    record(&bytes, bounds);
    record(&temper_codecs_world::mutated(rng, &bytes), bounds);
    let bytes = keys::encode(&forge::stored_key(&row), u32::MAX).expect("act key");
    key(&bytes);
    key(&temper_codecs_world::mutated(rng, &bytes));
}
#[test]
fn store_decoders_never_panic_and_reencode_the_same() {
    let bounds = temper_protocol_store::bounds(&LIMITS, LIMITS.client.op_bytes).expect("fixture limits fit");
    let mut rng = Rng::new(0x7e3d_0405);
    let goldens = temper_codecs_world::goldens("crates/temper-protocol-store/golden");
    for _ in 0..2_000 {
        let address = format!("member{}@example.com", rng.next_u64()).into_bytes().into_boxed_slice();
        let binding =
            forge::Stored::Binding { address, subject: temper_codecs_world::drawn(&mut rng, 255).into_boxed_slice() };
        row_and_key(&binding, &mut rng, &bounds);
        record(&temper_codecs_world::drawn(&mut rng, 512), &bounds);
        key(&temper_codecs_world::drawn(&mut rng, 512));
        let demo = forge::Stored::Demo(forge::Demo {
            forge: u16::try_from(rng.below(65_536)).expect("u16"),
            state: if rng.below(2) == 0 { forge::DemoState::Running } else { forge::DemoState::Ended },
        });
        row_and_key(&demo, &mut rng, &bounds);
        end_act(&mut rng, &bounds);
        kept_families(&mut rng, &bounds);
        let writer = forge::Stored::Writer(skein_forge_client::Writer {
            forge: u16::try_from(rng.below(65_536)).expect("u16"),
            author: rng.next_u64().max(1),
        });
        row_and_key(&writer, &mut rng, &bounds);
    }
    for _ in 0..128 {
        let flags = u8::try_from(rng.below(16)).expect("four flags");
        let bypass = match rng.below(3) {
            0 => skein_forge::api::Bypass::Never,
            1 => skein_forge::api::Bypass::PullsOnly,
            _ => skein_forge::api::Bypass::Always,
        };
        for row in protection::rows(protection::rule(flags, bypass)) {
            let bytes = records::encode(&row);
            assert_eq!(records::decode(&bytes, &bounds), Ok(row), "all protection facts survive");
            record(&bytes, &bounds);
            for _ in 0..8 {
                record(&temper_codecs_world::mutated(&mut rng, &bytes), &bounds);
            }
        }
    }
    // Queue-bearing records are also fuzz seeds, with both queue flags.
    for queued in [false, true] {
        let row = forge::Stored::Entry {
            entry: skein_forge_client::Entry {
                number: 1,
                task: 2,
                repository: skein_forge::api::Repository { forge: 1, repository: 3 },
                effect: skein_forge_client::Effect {
                    write: skein_forge::api::Write::Merge {
                        number: 4,
                        head: [5; 32],
                        how: skein_forge::api::MergeHow::Style,
                    },
                    condition: skein_forge_client::Condition::None,
                },
                start: None,
                attempt: None,
                failures: 0,
                queued,
            },
            held: false,
        };
        let bytes = records::encode(&row);
        record(&bytes, &bounds);
        for _ in 0..8 {
            record(&temper_codecs_world::mutated(&mut rng, &bytes), &bounds);
        }
    }
    for answer in [
        forge::CallPayload::Effect {
            entry: 1,
            deadline: skein_lib::Wall::EPOCH,
            outcome: Some(skein_forge_client::Outcome::Queued),
        },
        forge::CallPayload::Read(Box::new(Ok(skein_forge::api::Answer::Queued))),
        forge::CallPayload::Read(Box::new(Err(skein_forge::api::Error::NotInstalled))),
    ] {
        let row =
            forge::Stored::Call { key: forge::NamedCall { task: 1, attempt: 2, completion: 3, position: 4 }, answer };
        let bytes = records::encode(&row);
        record(&bytes, &bounds);
        for _ in 0..8 {
            record(&temper_codecs_world::mutated(&mut rng, &bytes), &bounds);
        }
    }
    let row = forge::Stored::Application {
        token: 1,
        until: skein_lib::Wall::EPOCH,
        outcome: forge::ApplicationOutcome::ForgeRefused(forge::AdoptionRefusal::NotInstalled),
    };
    let bytes = records::encode(&row);
    record(&bytes, &bounds);
    for _ in 0..8 {
        record(&temper_codecs_world::mutated(&mut rng, &bytes), &bounds);
    }
    for bytes in &goldens {
        golden(bytes, &bounds);
        for _ in 0..8 {
            golden(&temper_codecs_world::mutated(&mut rng, bytes), &bounds);
        }
    }
}

fn kept_families(rng: &mut Rng, bounds: &Bounds) {
    let forge = u16::try_from(rng.below(u64::from(u16::MAX - 1)) + 1).expect("forge number");
    let project = u32::try_from(rng.below(u64::from(u32::MAX))).expect("u32");
    let first = if rng.below(2) == 0 {
        None
    } else {
        Some(forge::FirstIntent {
            playground: rng.below(2) == 0,
            org: temper_codecs_world::drawn(rng, 128).into_boxed_slice(),
            repository: temper_codecs_world::drawn(rng, 128).into_boxed_slice(),
            project: if rng.below(2) == 0 {
                forge::SetUpProject::Existing(project)
            } else {
                forge::SetUpProject::New(temper_codecs_world::drawn(rng, 128).into_boxed_slice())
            },
            label: temper_codecs_world::drawn(rng, 128).into_boxed_slice(),
        })
    };
    let mode = match rng.below(4) {
        0 => forge::Mode::Connected,
        1 => forge::Mode::Managed,
        2 => forge::Mode::Owned,
        _ => forge::Mode::Fake,
    };
    let state = match rng.below(4) {
        0 => forge::PlaygroundState::Running,
        1 => forge::PlaygroundState::Resetting(forge::ResetStep::Project),
        2 => forge::PlaygroundState::Resetting(forge::ResetStep::Forge),
        _ => forge::PlaygroundState::Resetting(forge::ResetStep::Account),
    };
    let act = match rng.below(3) {
        0 => forge::KeptActKind::Cancel { project, task: rng.next_u64() },
        1 => forge::KeptActKind::RemoveGroup,
        _ => forge::KeptActKind::EraseSecrets,
    };
    for row in [
        forge::Stored::Forge(forge::ForgeRecord {
            number: forge,
            kind: if rng.below(2) == 0 { forge::Kind::Forgejo } else { forge::Kind::GitHub },
            mode,
            removing: rng.below(2) == 0,
            definition: temper_codecs_world::drawn(rng, 512).into_boxed_slice(),
            first,
        }),
        forge::Stored::ForgeNumbers { next: forge },
        forge::Stored::Playground(forge::Playground {
            state,
            forge: if rng.below(2) == 0 { None } else { Some(forge) },
            project: if rng.below(2) == 0 { None } else { Some(project) },
            account: if rng.below(2) == 0 { None } else { Some(project) },
        }),
        forge::Stored::KeptAct(forge::KeptAct { key: [u8::try_from(rng.below(256)).expect("byte"); 16], forge, act }),
    ] {
        row_and_key(&row, rng, bounds);
    }
}
