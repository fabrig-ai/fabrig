//! Same-step gets and the core's archived reads (protocol/store.md, 3 and 7.1).
use super::fixtures::{self, ALICE, ENDED};
use crate::{Event, Get, Phase, Request, Row, Store, Write, codec, ended, keys};
use jig_domain_core::{self as core, CoreKey, CoreRecord, Key, Now, Record};
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use skein_lib::Token;

const OWNER: Token = Token::new(7);
const ENDED_READ: Token = Token::new(8);

fn get(store: &mut Store, key: Get) -> Box<[Event]> {
    fixtures::run(store, Request::Get { owner: OWNER, key })
}

fn got(events: Box<[Event]>) -> Option<Row> {
    let mut events = events.into_iter();
    let Some(Event::Got { owner, row }) = events.next() else { panic!("one got terminal") };
    assert_eq!(owner, OWNER);
    assert!(events.next().is_none());
    row
}

fn note_key() -> Get {
    Get::Jig(fixtures::notes_entry(17).key())
}

fn notes_store() -> Store {
    let mut store = fixtures::opened();
    fixtures::committed(&mut store, Box::new([fixtures::header(1), fixtures::save(fixtures::notes_entry(17))]));
    store
}

#[test]
fn a_get_answers_the_record_by_its_key() {
    let mut store = notes_store();
    let Some(Row::Jig(row)) = got(get(&mut store, note_key())) else { panic!("typed note") };
    assert_eq!(row, fixtures::notes_entry(17));
}

#[test]
fn a_get_of_a_missing_key_answers_none() {
    let mut store = notes_store();
    assert!(got(get(&mut store, Get::Jig(fixtures::notes_entry(19).key()))).is_none());
}

#[test]
fn set_visibility_round_trips_as_a_kept_ask() {
    for public in [false, true] {
        let record = Record::People(people::Stored::Answer {
            key: people::RequestKey { person: ALICE, key: [17; 16] },
            ask: Box::new(people::Ask::SetVisibility { project: u32::MAX, public }),
            models: Box::new([]),
            outcome: people::Outcome::Refused(people::Refusal::NotOffered),
            at: skein_lib::Wall::EPOCH,
        });
        let mut store = fixtures::opened();
        fixtures::committed(&mut store, Box::new([fixtures::header(1), fixtures::save(record.clone())]));
        let Some(Row::Jig(found)) = got(get(&mut store, Get::Jig(record.key()))) else { panic!("kept ask") };
        assert_eq!(found, record);
    }
}

#[test]
fn a_public_row_round_trips_under_its_project_key() {
    let mut previous = None;
    for project in [1, 256, u32::MAX] {
        let record = Record::People(people::Stored::Public { project });
        let bytes = keys::encode(&record.key(), fixtures::KEY_LIMIT).expect("public key");
        assert_eq!(bytes.get(..3), Some([keys::JIG, 2, 10].as_slice()));
        assert_eq!(bytes.get(3..), Some(u64::from(project).to_be_bytes().as_slice()));
        assert_eq!(keys::decode(&bytes), Some(Key::People(people::Key::Public(project))));
        if let Some(previous) = previous {
            assert!(previous < bytes, "public keys sort by project");
        }
        let mut before = None;
        for key in [
            people::Key::Roles(project),
            people::Key::Policy(project),
            people::Key::Deletion(project),
            people::Key::Public(project),
        ] {
            let key = keys::encode(&Key::People(key), fixtures::KEY_LIMIT).expect("project key");
            assert_eq!(key.get(3..), bytes.get(3..), "same project suffix");
            if let Some(before) = before {
                assert!(before < key, "public follows the project's deletion");
            }
            before = Some(key);
        }
        previous = Some(bytes);
        let mut store = fixtures::opened();
        fixtures::committed(&mut store, Box::new([fixtures::header(1), fixtures::save(record.clone())]));
        let Some(Row::Jig(found)) = got(get(&mut store, Get::Jig(record.key()))) else { panic!("public row") };
        assert_eq!(found, record);
        fixtures::committed(&mut store, Box::new([fixtures::header(2), Write::Erase(record.key())]));
        assert!(got(get(&mut store, Get::Jig(record.key()))).is_none());
    }
}

#[test]
fn an_application_get_answers_its_bytes() {
    let mut store = fixtures::opened();
    fixtures::committed(
        &mut store,
        Box::new([fixtures::header(1), Write::Put { key: Box::new([41]), value: Box::new([42, 43]) }]),
    );
    let Some(Row::Application { key, value }) = got(get(&mut store, Get::Application(Box::new([41])))) else {
        panic!("application bytes");
    };
    assert_eq!(key.as_ref(), &[41]);
    assert_eq!(value.as_ref(), &[42, 43]);
}

fn seen_conversation() -> Now {
    Now::SeenConversation { request: ENDED_READ, task: ENDED }
}

fn person_task() -> Now {
    Now::EndedPersonTask { request: ENDED_READ, person: ALICE, project: 2, task: ENDED }
}

fn proposal() -> Now {
    Now::EndedProposal {
        request: ENDED_READ,
        person: ALICE,
        project: 2,
        proposer: people::Proposer::Party(12),
        proposal: 13,
    }
}

fn escalation() -> Now {
    Now::EndedEscalation { request: ENDED_READ, person: ALICE, project: 2, task: 11, revision: 3 }
}

fn ended_record() -> Box<tasks::TaskRecord> {
    let Record::Tasks(tasks::Stored::Ended(row)) = fixtures::ended_task(ENDED) else { panic!("ended fixture") };
    row
}

fn ended_answer(now: Now, record: Record) -> core::Event {
    let mut store = fixtures::opened();
    fixtures::committed(&mut store, Box::new([fixtures::header(1), fixtures::save(record)]));
    let key = ended::key(&now).expect("ended key");
    let Some(Row::Jig(row)) = got(get(&mut store, Get::Jig(key))) else { panic!("ended record exists") };
    ended::answer(now, Some(row)).expect("ended answer")
}

#[test]
fn a_seen_conversation_is_answered_with_its_ended_task() {
    let event = ended_answer(seen_conversation(), fixtures::ended_task(ENDED));
    let core::Event::SeenConversationLoaded { request, row, busy } = event else { panic!("seen chat event") };
    assert_eq!(request, ENDED_READ);
    assert_eq!(row, Some(ended_record()));
    assert!(!busy);
}

#[test]
fn an_ended_task_read_is_answered_with_its_ended_row() {
    let now = Now::EndedTask { request: ENDED_READ, task: ENDED };
    let event = ended_answer(now, fixtures::ended_task(ENDED));
    let core::Event::EndedTaskLoaded { request, row, busy } = event else { panic!("ended task event") };
    assert_eq!(request, ENDED_READ);
    assert_eq!(row, Some(ended_record()));
    assert!(!busy);
}

#[test]
fn a_ended_escalation_is_answered_with_its_decision() {
    let record = Record::Core(CoreRecord::EscalationDecision(fixtures::core_rows::escalation()));
    let event = ended_answer(escalation(), record);
    let core::Event::EndedEscalation { request, person, project, task, revision, row, busy } = event else {
        panic!("ended escalation event");
    };
    assert_eq!((request, person, project, task, revision), (ENDED_READ, ALICE, 2, 11, 3));
    assert_eq!(row, Some(fixtures::core_rows::escalation()));
    assert!(!busy);
}

#[test]
fn a_ended_proposal_is_answered_with_its_decision() {
    let record = Record::Core(CoreRecord::ProposalDecision(fixtures::core_rows::proposal()));
    let event = ended_answer(proposal(), record);
    let core::Event::EndedProposal { request, person, project, proposer, proposal, row, busy } = event else {
        panic!("ended proposal event");
    };
    assert_eq!((request, person, project, proposer, proposal), (ENDED_READ, ALICE, 2, people::Proposer::Party(12), 13));
    assert_eq!(row, Some(fixtures::core_rows::proposal()));
    assert!(!busy);
}

#[test]
fn a_ended_person_task_is_answered_with_its_ended_task() {
    let event = ended_answer(person_task(), fixtures::ended_task(ENDED));
    let core::Event::EndedPersonTask { request, person, project, task, row, busy } = event else {
        panic!("ended person task event");
    };
    assert_eq!((request, person, project, task), (ENDED_READ, ALICE, 2, ENDED));
    assert_eq!(row, Some(ended_record()));
    assert!(!busy);
}

#[test]
fn ended_reads_name_their_store_keys() {
    for (now, expected) in [
        (Now::EndedTask { request: ENDED_READ, task: ENDED }, Key::Tasks(tasks::Key::Ended(ENDED))),
        (seen_conversation(), Key::Tasks(tasks::Key::Ended(ENDED))),
        (person_task(), Key::Tasks(tasks::Key::Ended(ENDED))),
        (proposal(), Key::Core(CoreKey::ProposalDecision { proposer: people::Proposer::Party(12), proposal: 13 })),
        (escalation(), Key::Core(CoreKey::EscalationDecision { task: 11, revision: 3 })),
    ] {
        assert_eq!(ended::key(&now), Some(expected));
    }
}

#[test]
fn a_missing_ended_record_answers_none_without_busy() {
    let event = ended::answer(seen_conversation(), None).expect("seen chat answer");
    let core::Event::SeenConversationLoaded { request, row: None, busy: false } = event else {
        panic!("missing row is not busy");
    };
    assert_eq!(request, ENDED_READ);
}

#[test]
fn a_ended_row_from_another_family_answers_none() {
    let row = Some(fixtures::person(ALICE));
    let event = ended::answer(proposal(), row).expect("proposal answer");
    let core::Event::EndedProposal { request, row: None, busy: false, .. } = event else {
        panic!("another family's row is absent");
    };
    assert_eq!(request, ENDED_READ);
}

#[test]
fn an_unrelated_core_output_has_no_ended_translation() {
    assert_eq!(ended::key(&Now::RestoreRefused), None);
    assert!(ended::answer(Now::RestoreRefused, None).is_none());
}

fn planted_note(value: Box<[u8]>) -> Store {
    let mut stores = fixtures::stores();
    fixtures::plant(
        &mut stores,
        &[
            fixtures::format_put(fixtures::DEPLOYMENT),
            fixtures::layouts_put(),
            fixtures::header_put(fixtures::DEPLOYMENT, 1),
            fixtures::installation_put(fixtures::INSTALLATION),
            skein_kv::Op::Put {
                key: keys::encode(&fixtures::notes_entry(17).key(), fixtures::KEY_LIMIT).expect("note key fits"),
                value,
            },
        ],
    );
    let mut store = fixtures::over(stores);
    let events = fixtures::run(
        &mut store,
        Request::Open { owner: OWNER, root: fixtures::ROOT, installation: fixtures::INSTALLATION },
    );
    let [Event::Opened { .. }] = &*events else { panic!("fixture opens: {events:?}") };
    store
}

fn assert_corrupt(store: &Store, events: &[Event]) {
    let [Event::Corrupt { owner }] = events else { panic!("one corrupt terminal: {events:?}") };
    assert_eq!(*owner, OWNER);
    assert_eq!(store.phase, Phase::Stopped);
}

#[test]
fn an_undecodable_get_value_is_corrupt() {
    let mut store = planted_note(Box::new([255]));
    let events = get(&mut store, note_key());
    assert_corrupt(&store, &events);
}

#[test]
fn a_get_value_stored_under_another_records_key_is_corrupt() {
    let limits = codec::limits(&fixtures::CORE_LIMITS).expect("fixture limits");
    let value = codec::encode(&fixtures::notes_entry(19), &limits, &[]).expect("note fits");
    let mut store = planted_note(value);
    let events = get(&mut store, note_key());
    assert_corrupt(&store, &events);
}

#[test]
fn nothing_is_answered_after_a_corrupt_get() {
    let mut store = planted_note(Box::new([255]));
    let events = get(&mut store, note_key());
    assert_corrupt(&store, &events);
    assert!(get(&mut store, note_key()).is_empty());
}
