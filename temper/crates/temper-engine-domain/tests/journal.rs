//! Root journal decisions through their public admission and release ports.
#![cfg(test)]

use jig_domain_link::Bounds;
use skein_lib::{List, Queue, Token, Wall};
use step_journal_fixture::*;
use temper_engine_domain::{
    Decision, Delivery, Deployment, Family, Journal as SkeinJournal, JournalLimits as Limits, Key, Record, Write,
};
use temper_engine_step_kit::journal::{DEPLOYMENT, LIMITS, turn};

temper_engine_step_kit::journal_fixture!(temper_engine_domain);

#[test]
fn a_first_start_commits_its_deployment_before_releasing_a_delivery() {
    let mut j = Journal::bootstrap([37; 16], &LIMITS);
    let mut out = Queue::with_capacity(1);
    let mut d = Decision::decide(&mut j.inner, &LIMITS);
    d.deliver(&LIMITS, ack(1)).unwrap();
    accept(&mut j, &LIMITS, d, &mut out).unwrap();
    let (number, writes) = commit(&mut out);
    assert_eq!(number, 1);
    assert_eq!(
        writes.as_ref(),
        &[Write::Save(Record::Core(jig_domain_core::Record::Core(jig_domain_core::CoreRecord::Deployment(
            Deployment { id: [37; 16], commits: 1, ..DEPLOYMENT }
        ))))]
    );
    resume(&mut j, &mut out);
    assert!(out.is_empty());
    committed(&mut j, number);
    resume(&mut j, &mut out);
    assert_eq!(acknowledged(&mut out), 1);
}

#[test]
fn a_store_answer_makes_outputs_ready_without_draining_them() {
    let mut j = Journal::new(DEPLOYMENT, &LIMITS);
    let mut out = Queue::with_capacity(1);
    let d = decision(&mut j, 1);
    accept(&mut j, &LIMITS, d, &mut out).unwrap();
    let (number, writes) = commit(&mut out);
    assert_eq!(number, 1);
    assert_eq!(writes.len(), 2);
    assert_eq!(
        writes[0],
        Write::Save(Record::Core(jig_domain_core::Record::Core(jig_domain_core::CoreRecord::Deployment(Deployment {
            commits: 1,
            ..DEPLOYMENT
        }))))
    );
    resume(&mut j, &mut out);
    assert!(out.is_empty());
    committed(&mut j, number);
    assert!(out.is_empty());
    resume(&mut j, &mut out);
    assert_eq!(acknowledged(&mut out), 1);
    resume(&mut j, &mut out);
    assert!(out.is_empty());
}

#[test]
fn fresh_numbers_and_unused_gaps_are_durable_in_the_decisions_header() {
    let mut j = Journal::new(DEPLOYMENT, &LIMITS);
    let mut out = Queue::with_capacity(1);
    for family in [Family::Task, Family::Person, Family::SignIn, Family::Message, Family::Run, Family::Call] {
        assert_eq!(fresh(&mut j, family), Some(1));
    }
    assert_eq!(fresh(&mut j, Family::Task), Some(2));
    let d = Decision::decide(&mut j.inner, &LIMITS);
    accept(&mut j, &LIMITS, d, &mut out).unwrap();
    let (number, writes) = commit(&mut out);
    assert_eq!(number, 1);
    let Write::Save(Record::Core(jig_domain_core::Record::Core(jig_domain_core::CoreRecord::Deployment(header)))) =
        writes[0]
    else {
        panic!("header");
    };
    assert_eq!(header.tasks, 2);
    let mut restarted = Journal::new(header, &LIMITS);
    assert_eq!(restarted.durable(), 1);
    assert_eq!(fresh(&mut restarted, Family::Task), Some(3));
}

#[test]
fn same_key_saves_and_erases_collapse_to_the_last_write_before_commit() {
    const SHORT: &[(u32, u32)] = &[(1, 2), (2, 3)];
    // The fifth write builds the index; the sixth is admitted through it.
    const INDEXED: &[(u32, u32)] = &[(1, 2), (2, 3), (3, 2), (4, 2), (5, 2), (6, 2)];
    // The indexed row has room for its six writes, the header and a new key.
    const INDEXED_LIMITS: Limits = Limits { writes: 8, ..LIMITS };
    for (limits, initial) in [(&LIMITS, SHORT), (&INDEXED_LIMITS, INDEXED)] {
        let mut j = Journal::new(DEPLOYMENT, limits);
        let mut out = Queue::with_capacity(1);
        let mut d = Decision::decide(&mut j.inner, limits);
        for &(number, bytes) in initial {
            d.write(limits, turn(number, bytes)).unwrap();
        }
        d.write(
            limits,
            Write::Erase(Key::Core(jig_domain_core::Key::Core(jig_domain_core::CoreKey::Turn {
                task: 1,
                attempt: 1,
                turn: 1,
            }))),
        )
        .unwrap();
        d.write(limits, turn(1, 4)).unwrap();
        accept(&mut j, limits, d, &mut out).unwrap();
        let (_, writes) = commit(&mut out);
        let mut expected = List::with_capacity(limits.writes);
        expected.push(turn(1, 4)).unwrap();
        for &(number, bytes) in &initial[1..] {
            expected.push(turn(number, bytes)).unwrap();
        }
        assert_eq!(&writes[1..], expected.as_slice(), "cohort of {} writes", initial.len());
    }
}

#[test]
fn a_decision_with_more_outputs_than_the_door_releases_them_all_in_order() {
    let mut journal = Journal::new(DEPLOYMENT, &LIMITS);
    let mut decision = Decision::decide(&mut journal.inner, &LIMITS);
    decision.write(&LIMITS, turn(1, 1)).unwrap();
    for number in 1..=LIMITS.deliveries + 2 {
        decision.deliver(&LIMITS, ack(number)).unwrap();
    }
    let mut out = Queue::with_capacity(1);
    accept(&mut journal, &LIMITS, decision, &mut out).unwrap();
    let (number, writes) = commit(&mut out);
    assert_eq!(number, 1);
    assert_eq!(writes.len(), 2, "one write and the deployment header");
    assert_eq!(writes[1], turn(1, 1));
    resume(&mut journal, &mut out);
    assert!(out.is_empty(), "all outputs wait for durability");
    committed(&mut journal, number);
    for number in 1..=LIMITS.deliveries + 2 {
        resume(&mut journal, &mut out);
        assert_eq!(acknowledged(&mut out), number, "every delivery releases once, in order");
    }
    resume(&mut journal, &mut out);
    assert!(out.is_empty());
    assert!(!journal.stopped());
}

#[test]
fn deep_child_rows_and_arbitrary_internal_payloads_are_refused_before_retention() {
    use jig_domain_fleet as fleet;
    use jig_domain_people as people;
    let limits =
        Limits { commits: 1, held: 2, writes: 2, deliveries: 2, transcript_bytes: 4, result_bytes: 4, run_bytes: 1 };
    let mut journal = SkeinJournal::uncapped(1, 0);
    let mut decision = Decision::decide(&mut journal, &limits);
    let row = Record::Core(jig_domain_core::Record::People(people::Stored::Person {
        number: 1,
        identity: people::Identity {
            key: people::IdentityKey { provider: 0, subject: 1_u64.to_be_bytes().into() },
            login: b"abc".as_slice().into(),
            name: b"de".as_slice().into(),
        },
    }));
    assert_eq!(temper_engine_domain::record_bytes(&row), Some(13));
    assert!(decision.write(&limits, Write::Save(row)).is_err());
    let roster_answer = Record::Core(jig_domain_core::Record::People(people::Stored::Answer {
        models: Box::new([]),
        key: people::RequestKey { person: 1, key: [1; 16] },
        ask: Box::new(people::Ask::SetRoles {
            project: 1,
            holdings: Box::new([people::Holding { person: 1, role: people::Role::Owner }]),
        }),
        outcome: people::Outcome::RolesSet { project: 1 },
        at: Wall::EPOCH,
    }));
    assert_eq!(
        temper_engine_domain::record_bytes(&roster_answer),
        Some(u64::try_from(size_of::<people::Holding>()).expect("holding slot"))
    );
    assert!(
        decision.write(&limits, Write::Save(roster_answer)).is_err(),
        "saved keyed roster has separately owned bounded bytes"
    );
    let callback = Delivery::Fleet(fleet::Event::Hello {
        worker: 0,
        channel: Token::new(1),
        hello: jig_domain_link::Hello {
            fresh: false,
            reads_usage: true,
            jobs: Box::default(),
            stop_bound: skein_lib::Duration::from_secs(0),
            slots: 1,
            workstreams: Box::new([]),
            hosting: Box::new([]),
            bounds: Bounds {
                program_endpoints: Box::new([]),
                endpoints: Box::new([]),
                turns: 1,
                spend: 1,
                time: skein_lib::Duration::from_secs(1),
                waiting: skein_lib::Duration::from_secs(1),
                checks: skein_lib::Duration::from_secs(1),
                message_bytes: 1,
                inbox: 1,
                unacknowledged: 1,
                reply_bytes: None,
            },
        },
    });
    assert!(
        decision.deliver(&limits, callback).is_err(),
        "arbitrary fleet events cannot bypass bounds through the journal"
    );
}

#[test]
fn a_retained_model_context_counts_its_owned_names() {
    use jig_domain_people as people;
    let row = Record::Core(jig_domain_core::Record::People(people::Stored::Answer {
        key: people::RequestKey { person: 1, key: [1; 16] },
        ask: Box::new(people::Ask::SetRoles { project: 1, holdings: Box::new([]) }),
        models: Box::new([Box::from(b"small".as_slice())]),
        outcome: people::Outcome::RolesSet { project: 1 },
        at: Wall::EPOCH,
    }));
    let expected = u64::try_from(size_of::<Box<[u8]>>() + b"small".len()).expect("one name fits");
    assert_eq!(temper_engine_domain::record_bytes(&row), Some(expected));
}

#[test]
fn a_program_declaration_counts_both_retained_names_and_models() {
    use jig_domain_people as people;
    let name = b"work-login";
    let row = Record::Core(jig_domain_core::Record::People(people::Stored::Answer {
        key: people::RequestKey { person: 1, key: [1; 16] },
        ask: Box::new(people::Ask::AccountChange {
            holder: people::AccountHolder::Own,
            change: people::AccountChange::DeclareProgram { provider: 1, join: None, name: name.as_slice().into() },
        }),
        models: Box::new([Box::from(b"small".as_slice())]),
        outcome: people::Outcome::AccountProgramDeclared { source: 1, name: name.as_slice().into() },
        at: Wall::EPOCH,
    }));
    let expected = size_of::<Box<[u8]>>() + b"small".len() + 2 * name.len();
    assert_eq!(temper_engine_domain::record_bytes(&row), Some(u64::try_from(expected).expect("bounded owned bytes")));
}
#[test]
fn an_issued_program_counts_its_retained_name() {
    let name = b"work-login";
    let row = Record::Core(jig_domain_core::Record::Accounts(jig_domain_accounts::Stored::Issued {
        number: 1,
        holder: jig_domain_accounts::Holder::Party(1),
        name: name.as_slice().into(),
        signed_out: true,
    }));
    assert_eq!(temper_engine_domain::record_bytes(&row), Some(skein_lib::len_u64(name.len())));
}

#[test]
fn a_program_sign_out_counts_its_retained_worker_failures() {
    use jig_domain_people as people;
    let name = b"worker-a";
    let row = Record::Core(jig_domain_core::Record::People(people::Stored::Answer {
        key: people::RequestKey { person: 1, key: [1; 16] },
        ask: Box::new(people::Ask::AccountSignOut { holder: people::AccountHolder::Own, source: 1 }),
        models: Box::new([]),
        outcome: people::Outcome::AccountSignedOut {
            confirmed: false,
            failed_hosts: Box::new([people::AccountHostFailure {
                name: name.as_slice().into(),
                why: people::AccountHostWhy::NotInContact,
            }]),
        },
        at: Wall::EPOCH,
    }));
    let expected = size_of::<people::AccountHostFailure>() + name.len();
    assert_eq!(temper_engine_domain::record_bytes(&row), Some(u64::try_from(expected).expect("bounded owned bytes")));
}
