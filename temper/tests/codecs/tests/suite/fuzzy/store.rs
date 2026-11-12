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
    let mut versioned = vec![0_u8, 11];
    versioned.extend_from_slice(bytes);
    record(&versioned, bounds);
}
/// A drawn row, its key, and one mutation of each.
fn row_and_key(row: &forge::Stored, rng: &mut Rng, bounds: &Bounds) {
    let bytes = records::encode(row);
    assert_eq!(records::decode(&bytes, bounds), Ok(row.clone()), "drawn row");
    record(&bytes, bounds);
    record(&temper_codecs_world::mutated(rng, &bytes), bounds);
    let bytes = keys::encode(&forge::stored_key(row), u32::MAX).expect("drawn key");
    key(&bytes);
    key(&temper_codecs_world::mutated(rng, &bytes));
}
fn making_row(rng: &mut Rng) -> forge::Stored {
    forge::Stored::Making(forge::RepositoryMaking {
        key: forge::MakingKey {
            forge: u16::try_from(rng.below(65_536)).expect("u16"),
            org: temper_codecs_world::drawn(rng, LIMITS.name_bytes).into_boxed_slice(),
            name: temper_codecs_world::drawn(rng, LIMITS.name_bytes).into_boxed_slice(),
        },
        from: forge::MakeFrom::Copy { address: temper_codecs_world::drawn(rng, 512).into_boxed_slice() },
        into: Some(forge::MakeInto { project: 1, home: true, host: Box::from(b"forge.invalid".as_slice()) }),
        asked: skein_lib::Wall::from_nanos(rng.next_u64()),
        copying: rng.below(2) == 0,
        state: forge::MakingState::Failed(forge::MakingFailed {
            step: forge::MakeStep::Repository,
            reason: forge::MakeFailure::Refused { status: 409 },
            words: temper_codecs_world::drawn(rng, 256).into_boxed_slice(),
        }),
    })
}
fn person_fact(rng: &mut Rng) -> forge::PersonFact {
    if rng.below(3) == 0 {
        let subject = if rng.below(2) == 0 {
            temper_protocol_reaches::Subject::Set(u8::try_from(rng.below(256)).expect("u8"))
        } else {
            temper_protocol_reaches::Subject::Kind(
                temper_protocol_reaches::Kind::ALL[usize::try_from(rng.below(10)).expect("kind index")],
            )
        };
        forge::PersonFact::KeepAsking {
            project: u32::try_from(rng.below(4_294_967_296)).expect("u32"),
            subject,
            at: rng.next_u64(),
        }
    } else if rng.below(2) == 0 {
        forge::PersonFact::ToBegin { project: u32::try_from(rng.below(4_294_967_296)).expect("u32") }
    } else {
        forge::PersonFact::GoalFrom {
            project: u32::try_from(rng.below(4_294_967_296)).expect("u32"),
            chat: rng.next_u64(),
            goal: rng.next_u64(),
            at: rng.next_u64(),
        }
    }
}
#[test]
#[expect(clippy::too_many_lines, reason = "one seed sweeps the durable families and their keys")]
fn store_decoders_never_panic_and_reencode_the_same() {
    let bounds = temper_protocol_store::bounds(&LIMITS, LIMITS.client.op_bytes).expect("fixture limits fit");
    let mut rng = Rng::new(0x7e3d_0405);
    let goldens = temper_codecs_world::goldens("crates/temper-protocol-store/golden/v12");
    for _ in 0..2_000 {
        let address = format!("member{}@example.com", rng.next_u64()).into_bytes().into_boxed_slice();
        let binding =
            forge::Stored::Binding { address, subject: temper_codecs_world::drawn(&mut rng, 255).into_boxed_slice() };
        row_and_key(&binding, &mut rng, &bounds);
        let making = making_row(&mut rng);
        row_and_key(&making, &mut rng, &bounds);
        let dismissed = forge::Stored::Dismissed { party: rng.next_u64(), what: person_fact(&mut rng) };
        row_and_key(&dismissed, &mut rng, &bounds);
        record(&temper_codecs_world::drawn(&mut rng, 512), &bounds);
        key(&temper_codecs_world::drawn(&mut rng, 512));
        forge_rows(&mut rng, &bounds);
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
        forge::CallPayload::Decided { took: None, reach: None },
        forge::CallPayload::Decided {
            took: Some(Box::from(b"Small".as_slice())),
            reach: Some(temper_protocol_reaches::Reach {
                kind: temper_protocol_reaches::Kind::Wording,
                class: temper_protocol_reaches::Class::Decide,
                reason: temper_protocol_reaches::Reason::Kind,
                under: 0,
                sets: 0,
                matched: 0,
                away: false,
            }),
        },
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

fn forge_rows(rng: &mut Rng, bounds: &Bounds) {
    status_rows(rng, bounds);
    away_rows(rng, bounds);
    draft_row(rng, bounds);
    setting_row(rng, bounds);
    let number = u16::try_from(rng.below(65_536)).expect("u16");
    let kind = if rng.below(2) == 0 { forge::Kind::Forgejo } else { forge::Kind::GitHub };
    let mode = match rng.below(4) {
        0 => forge::Mode::Connected,
        1 => forge::Mode::Managed,
        2 => forge::Mode::Owned,
        _ => forge::Mode::Fake,
    };
    let state = if rng.below(2) == 0 {
        forge::ForgeState::Kept
    } else {
        forge::ForgeState::Ended { secrets_erased: rng.below(2) == 0, box_removed: rng.below(2) == 0 }
    };
    let place = forge::Place {
        api: forge::Address {
            host: drawn_field(rng),
            port: u16::try_from(rng.below(65_536)).expect("u16"),
            tls: optional_field(rng),
        },
        git: drawn_field(rng),
        web: optional_field(rng),
    };
    let bot = forge::Bot { login: drawn_field(rng), name: drawn_field(rng), email: drawn_field(rng) };
    let first = match rng.below(4) {
        0 => None,
        1 => Some(forge::FirstIntent::Nothing),
        2 => Some(forge::FirstIntent::Copies { addresses: Box::new([drawn_field(rng), drawn_field(rng)]) }),
        _ => Some(forge::FirstIntent::Empty(forge::EmptyIntent {
            playground: rng.below(2) == 0,
            org: drawn_field(rng),
            repository: drawn_field(rng),
            label: drawn_field(rng),
        })),
    };
    let row = forge::Stored::Forge(forge::ForgeRecord {
        person: if rng.below(2) == 0 {
            None
        } else {
            Some(forge::Link { user: rng.next_u64().max(1), person: rng.next_u64().max(1) })
        },
        git_program: if rng.below(2) == 0 { None } else { Some(drawn_field(rng)) },
        number,
        kind,
        mode,
        state,
        place,
        read_only: rng.below(2) == 0,
        bot,
        asked: rng.below(2) == 0,
        first,
    });
    row_and_key(&row, rng, bounds);
    row_and_key(&forge::Stored::ForgeNumber { last: number }, rng, bounds);
    let state = match rng.below(5) {
        0 => forge::PlaygroundState::Starting,
        1 => forge::PlaygroundState::Running,
        2 => forge::PlaygroundState::Resetting(forge::ResetStep::Project),
        3 => forge::PlaygroundState::Resetting(forge::ResetStep::Forge),
        _ => forge::PlaygroundState::Resetting(forge::ResetStep::Account),
    };
    let row = forge::Stored::Playground(forge::Playground {
        state,
        forge: number,
        project: optional_number(rng),
        account: optional_number(rng),
    });
    row_and_key(&row, rng, bounds);
    tier_rows(rng, bounds);
}

fn tier_rows(rng: &mut Rng, bounds: &Bounds) {
    use temper_engine_domain_forge_tiers as p;
    let repository = skein_forge::api::Repository {
        forge: u16::try_from(rng.below(65_536)).expect("u16"),
        repository: u32::try_from(rng.below(4_294_967_296)).expect("u32"),
    };
    let failures = drawn_failures(rng);
    let blame = match rng.below(3) {
        0 => p::Blame::Cleared,
        1 => p::Blame::Owns(failures.clone()),
        _ => p::Blame::Suspect(failures.clone()),
    };
    let tests = Box::new([Box::from(b"a".as_slice()), Box::from(b"z".as_slice())]);
    let part = match rng.below(3) {
        0 => p::Part::Whole,
        1 => p::Part::Rerun { failed: tests.clone() },
        _ => p::Part::Probe {
            landing: 0,
            tests: if rng.below(2) == 0 { p::Selection::All } else { p::Selection::Tests(tests.clone()) },
            finding: p::Finding { failing: failures.clone(), flakes: tests, blamed: Box::new([blame.clone()]) },
        },
    };
    let run = if rng.below(2) == 0 {
        None
    } else {
        Some(p::Run {
            number: rng.next_u64(),
            tip: [3; 32],
            key: [4; 32],
            faults: u32::try_from(rng.below(4_294_967_296)).expect("u32"),
            landings: Box::new([p::Landing { commit: [1; 32], key: [2; 32] }]),
            part,
        })
    };
    let row = forge::Stored::Tier(forge::TierRow {
        observations: temper_codecs_world::drawn(rng, 512).into_boxed_slice(),
        requested: false,
        cancelled: false,
        purpose: None,
        pending: Box::new([]),
        baseline: None,
        branch: forge::TierBranch::Absent,
        probe: None,
        flight: None,
        held: None,
        result: None,
        received: false,
        goals: Box::new([forge::TierGoal {
            goal: 7,
            branch: Box::from(b"temper/1/g7".as_slice()),
            tip: Some([11; 32]),
            ran: Some([12; 32]),
            failing: failures.clone(),
            started: Some(skein_lib::Time::from_nanos(rng.next_u64())),
        }]),
        goal: Some(7),
        started: Some(skein_lib::Time::from_nanos(rng.next_u64())),
        repository,
        tier: Box::from(b"tier".as_slice()),
        task: if rng.below(2) == 0 { None } else { Some(rng.next_u64()) },
        record: p::Record {
            sequence: rng.next_u64(),
            judged: if rng.below(2) == 0 { None } else { Some(p::Judged { tip: [5; 32], key: [6; 32] }) },
            failing: failures.clone(),
            flakes: Box::new([p::Flake { test: Box::from(b"flake".as_slice()), count: 2 }]),
            run,
        },
    });
    row_and_key(&row, rng, bounds);
    verification_row(rng, bounds, repository, blame);
    let row = forge::Stored::Receipt {
        key: forge::ReceiptKey {
            project: 5,
            repository,
            check: forge::Check::Tier(Box::from(b"tier".as_slice())),
            key: [9; 32],
        },
        receipt: forge::Receipt {
            failures: if rng.below(2) == 0 { None } else { Some(failures) },
            passed: false,
            run: forge::ReceiptRun { id: rng.next_u64(), number: rng.next_u64(), url: Box::new([]) },
            candidate: [10; 32],
            ended: skein_lib::Wall::EPOCH,
        },
    };
    row_and_key(&row, rng, bounds);
}
fn drawn_failures(rng: &mut Rng) -> temper_engine_domain_forge_tiers::Failures {
    use temper_engine_domain_forge_tiers::Failures;
    match rng.below(3) {
        0 => Failures::Many,
        1 => Failures::Tests(Box::new([])),
        _ => Failures::Tests(Box::new([Box::from(b"a".as_slice()), Box::from(b"z".as_slice())])),
    }
}

fn drawn_field(rng: &mut Rng) -> Box<[u8]> {
    temper_codecs_world::drawn(rng, 64.min(LIMITS.name_bytes)).into_boxed_slice()
}

fn optional_field(rng: &mut Rng) -> Option<Box<[u8]>> {
    if rng.below(2) == 0 { None } else { Some(drawn_field(rng)) }
}

fn optional_number(rng: &mut Rng) -> Option<u32> {
    if rng.below(2) == 0 { None } else { Some(u32::try_from(rng.below(4_294_967_296)).expect("u32")) }
}

fn setting_row(rng: &mut Rng, bounds: &Bounds) {
    use forge::settings as s;
    let slot = match rng.below(3) {
        0 => s::SettingSlot::Default,
        1 => s::SettingSlot::Change,
        _ => s::SettingSlot::Proposal(rng.next_u64()),
    };
    let last = if rng.below(2) == 0 {
        forge::GateLast::Exact(drawn_field(rng))
    } else {
        forge::GateLast::Open(drawn_field(rng))
    };
    let value = if rng.below(2) == 0 {
        s::Value::Charter(u32::try_from(rng.below(4_294_967_296)).expect("u32"))
    } else {
        let mut sets = skein_lib::List::with_capacity(6);
        for _ in 0..rng.below(7) {
            sets.push(u8::try_from(rng.below(256)).expect("u8 draw")).expect("bounded sets");
        }
        s::Value::Rule(s::LandingRule {
            sets: sets.into_boxed(),
            pattern: forge::GatePattern { segments: Box::new([drawn_field(rng)]), last },
            enforce: rng.below(2) == 0,
            ci: rng.below(2) == 0,
            up_to_date: rng.below(2) == 0,
            gates: Box::new([s::RuleGate { number: 1, blocking: true, freshness: forge::JudgeFreshness::Exact }]),
            approvals: Box::new([s::RuleApproval { role: 1, people: 2, freshness: forge::JudgeFreshness::Clean }]),
        })
    };
    let edit = match rng.below(3) {
        0 => s::Edit::Set(s::SizedValue { value, size: u32::try_from(rng.below(4_294_967_296)).expect("u32") }),
        1 => s::Edit::Remove,
        _ => s::Edit::Reset,
    };
    let row = forge::Stored::Setting(s::SettingRow {
        key: s::SettingKey { kind: u32::try_from(rng.below(3) + 1).expect("kind"), scope: 0, number: 1, slot },
        edit,
    });
    row_and_key(&row, rng, bounds);
}

fn verification_row(
    rng: &mut Rng,
    bounds: &Bounds,
    repository: skein_forge::api::Repository,
    blame: temper_engine_domain_forge_tiers::Blame,
) {
    let row = forge::Stored::Verification(forge::Verification {
        repository,
        commit: [7; 32],
        changes: Box::new([rng.next_u64(), rng.next_u64()]),
        tiers: Box::new([forge::TierVerification {
            tier: Box::from(b"tier".as_slice()),
            verdict: if rng.below(2) == 0 { None } else { Some(forge::TierVerdict { run: rng.next_u64(), blame }) },
            repaired: if rng.below(2) == 0 {
                None
            } else {
                Some(temper_engine_domain_forge::TierRepair { run: 2, tip: [8; 32], by: Some([9; 32]) })
            },
        }]),
    });
    row_and_key(&row, rng, bounds);
    let row = forge::Stored::LandingLog(forge::LandingLog {
        repository,
        landings: Box::new([forge::LoggedLanding {
            commit: [7; 32],
            branch: temper_codecs_world::drawn(rng, LIMITS.name_bytes).into_boxed_slice(),
            at: skein_lib::Wall::from_nanos(rng.next_u64()),
            changes: Box::new([forge::LoggedChange {
                signed: Box::new([]),
                task: rng.next_u64(),
                pull: if rng.below(2) == 0 { None } else { Some(rng.next_u64()) },
                title: temper_codecs_world::drawn(rng, 256).into_boxed_slice(),
            }]),
        }]),
    });
    row_and_key(&row, rng, bounds);
}

fn status_rows(rng: &mut Rng, bounds: &Bounds) {
    use skein_lib::Wall;
    let project = u32::try_from(rng.below(4_294_967_296)).expect("u32");
    let brief = if rng.below(2) == 0 {
        None
    } else {
        Some(forge::Brief {
            sentence: temper_codecs_world::drawn(rng, 200).into_boxed_slice(),
            lede: temper_codecs_world::drawn(rng, 600).into_boxed_slice(),
            at: Wall::from_nanos(rng.next_u64()),
            landed: rng.next_u64(),
        })
    };
    let running = if rng.below(2) == 0 {
        None
    } else {
        Some(forge::StatusRunning {
            task: if rng.below(2) == 0 { None } else { Some(rng.next_u64()) },
            landed: rng.next_u64(),
            since: Wall::from_nanos(rng.next_u64()),
        })
    };
    let facts = forge::Stored::ProjectFacts(forge::ProjectFacts {
        project,
        landed: rng.next_u64(),
        last_landed: Wall::from_nanos(rng.next_u64()),
        brief,
        running,
    });
    row_and_key(&facts, rng, bounds);
    let looked = forge::Stored::Looked(forge::Looked {
        party: rng.next_u64(),
        project,
        previous: if rng.below(2) == 0 { None } else { Some(Wall::from_nanos(rng.next_u64())) },
        latest: Wall::from_nanos(rng.next_u64()),
        seen: rng.next_u64(),
        next_seen: rng.next_u64(),
    });
    row_and_key(&looked, rng, bounds);
}

fn away_rows(rng: &mut Rng, bounds: &Bounds) {
    use skein_lib::Wall;
    let project = u32::try_from(rng.below(4_294_967_296)).expect("u32");
    let row = forge::Stored::Away(forge::Away {
        party: rng.next_u64(),
        project,
        since: Wall::from_nanos(rng.next_u64()),
        until: if rng.below(2) == 0 { None } else { Some(Wall::from_nanos(rng.next_u64())) },
        landed: rng.next_u64(),
        off: if rng.below(2) == 0 { None } else { Some(Wall::from_nanos(rng.next_u64())) },
    });
    row_and_key(&row, rng, bounds);
    let count = rng.below(5) + 2;
    let mut options = Vec::new();
    for _ in 0..count {
        options.push(temper_codecs_world::drawn(rng, 120).into_boxed_slice());
    }
    let row = forge::Stored::Decided(forge::Decided {
        project,
        number: u32::try_from(rng.below(4_294_967_296)).expect("u32"),
        party: rng.next_u64(),
        task: rng.next_u64(),
        at: Wall::from_nanos(rng.next_u64()),
        question: temper_codecs_world::drawn(rng, 400).into_boxed_slice(),
        options: options.into_boxed_slice(),
        took: Some(u8::try_from(rng.below(count)).expect("option index")),
        why: temper_codecs_world::drawn(rng, 400).into_boxed_slice(),
        overturned: rng.below(2) == 0,
        reach: temper_protocol_reaches::Reach {
            kind: *temper_protocol_reaches::Kind::ALL
                .get(usize::try_from(rng.below(10)).expect("kind index fits"))
                .expect("drawn kind exists"),
            class: temper_protocol_reaches::Class::Decide,
            reason: if rng.below(2) == 0 {
                temper_protocol_reaches::Reason::Kind
            } else {
                temper_protocol_reaches::Reason::Always
            },
            under: u8::try_from(rng.below(21)).expect("item number fits"),
            sets: u8::try_from(rng.below(64)).expect("six bits"),
            matched: u8::try_from(rng.below(64)).expect("six bits"),
            away: rng.below(2) == 0,
        },
    });
    row_and_key(&row, rng, bounds);
    if let forge::Stored::Decided(mut row) = row {
        row.took = None;
        row.options = Box::new([]);
        row.why = Box::new([]);
        row.overturned = false;
        row.reach.class = temper_protocol_reaches::Class::AskAway;
        row.reach.away = rng.below(2) == 0;
        row.reach.reason =
            temper_protocol_reaches::Reason::Set(u8::try_from(rng.below(6) + 1).expect("set number fits"));
        row.reach.sets = u8::try_from(rng.below(64)).expect("six bits");
        row.reach.matched = u8::try_from(rng.below(64)).expect("six bits");
        row.reach.under = u8::try_from(rng.below(21)).expect("item number fits");
        row_and_key(&forge::Stored::Decided(row), rng, bounds);
    }
}

fn draft_row(rng: &mut Rng, bounds: &Bounds) {
    let row = forge::Stored::LandedDraft(forge::LandedDraft {
        repository: skein_forge::api::Repository {
            forge: u16::try_from(rng.below(65_536)).expect("u16"),
            repository: u32::try_from(rng.below(4_294_967_296)).expect("u32"),
        },
        path: format!("docs/{}.md", rng.next_u64()).into_bytes().into(),
        project: u32::try_from(rng.below(4_294_967_296)).expect("u32"),
        branch: temper_codecs_world::drawn(rng, 1024).into(),
        commit: std::array::from_fn(|_| u8::try_from(rng.below(256)).expect("one byte")),
        title: temper_codecs_world::drawn(rng, 200).into(),
        writer: rng.next_u64(),
        at: skein_lib::Wall::from_nanos(rng.next_u64()),
        chat: if rng.below(2) == 0 { None } else { Some(rng.next_u64()) },
        goal: if rng.below(2) == 0 { None } else { Some(rng.next_u64()) },
        ready: rng.below(2) == 0,
    });
    row_and_key(&row, rng, bounds);
}
