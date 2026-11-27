//! Same-step gets and the core's archived reads (protocol/store/README.md, 3, and protocol/store/loads.md, 1).
use super::fixtures::{self, ALICE, ENDED};
use crate::{Event, Get, Phase, Request, Row, Store, Write, codec, ended, keys};
use jig_domain_core_vocabulary::{CoreKey, CoreRecord, Key, Record, engine::store as engine_store};
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

fn ended_key() -> Key {
    Key::Tasks(tasks::Key::Ended(ENDED))
}

pub(super) fn records(store: &mut Store, key: Key) -> Box<[Record]> {
    let (get, pending) = ended::start(ENDED_READ, engine_store::Range::Record(key.clone())).expect("point get");
    let Request::Get { owner, key: Get::Jig(found) } = &get else { panic!("jig get") };
    assert_eq!((*owner, found), (ENDED_READ, &key));
    let mut events = fixtures::run(store, get).into_iter();
    let Some(Event::Got { owner, row }) = events.next() else { panic!("one get terminal") };
    assert_eq!(owner, ENDED_READ);
    assert!(events.next().is_none());
    let ended::Step::Answer(engine_store::Answer::Rows(engine_store::Rows::Records { records, more: false })) =
        ended::got(owner, pending, row)
    else {
        panic!("one records answer")
    };
    records
}

fn check_record(key: Key, record: Record) {
    let mut store = fixtures::opened();
    fixtures::committed(&mut store, Box::new([fixtures::header(1), fixtures::save(record.clone())]));
    assert_eq!(records(&mut store, key).as_ref(), &[record]);
}

#[test]
fn an_ended_task_record_load_answers_its_row() {
    check_record(ended_key(), fixtures::ended_task(ENDED));
}

#[test]
fn an_ended_read_position_record_load_answers_its_row() {
    let key = Key::People(people::Key::EndedRead { task: ENDED, party: ALICE });
    let row = Record::People(people::Stored::EndedRead { task: ENDED, party: ALICE, through: 3 });
    let mut store = fixtures::opened();
    fixtures::committed(
        &mut store,
        Box::new([fixtures::header(1), fixtures::save(fixtures::ended_task(ENDED)), fixtures::save(row.clone())]),
    );
    assert_eq!(records(&mut store, key).as_ref(), &[row]);
}

#[test]
fn an_escalation_decision_record_load_answers_its_row() {
    check_record(
        Key::Core(CoreKey::EscalationDecision { task: 11, revision: 3 }),
        Record::Core(CoreRecord::EscalationDecision(fixtures::core_rows::escalation())),
    );
}

#[test]
fn a_proposal_decision_record_load_answers_its_row() {
    check_record(
        Key::Core(CoreKey::ProposalDecision { proposer: people::Proposer::Party(12), proposal: 13 }),
        Record::Core(CoreRecord::ProposalDecision(fixtures::core_rows::proposal())),
    );
}

#[test]
fn a_missing_ended_record_answers_none_without_busy() {
    let mut store = fixtures::opened();
    fixtures::committed(&mut store, Box::new([fixtures::header(1), fixtures::save(fixtures::person(ALICE))]));
    assert!(records(&mut store, ended_key()).is_empty());
}

fn planted_note(value: Box<[u8]>) -> Store {
    let mut stores = fixtures::stores();
    fixtures::plant(
        &mut stores,
        &[
            fixtures::format_put(fixtures::DEPLOYMENT),
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
        Request::Open {
            purpose: crate::Purpose::Serve,
            owner: OWNER,
            root: fixtures::ROOT,
            installation: fixtures::INSTALLATION,
        },
    );
    let [Event::Opened { .. }, Event::Census { .. }] = &*events else { panic!("fixture opens: {events:?}") };
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
