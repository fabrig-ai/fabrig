//! Shared client fixtures for the settings wire contract.

use jig_client_domain::contract as c;
use jig_client_protocol_link::{Limits, translate};
use jig_client_protocol_wire as w;
use jig_client_protocol_wire::Record as _;
use jig_client_scripted_engine::wire;
use jig_client_test_records::settings as values;
use skein_lib::{List, Writer};
macro_rules! encoded {
    ($value:expr) => {{
        let value = $value;
        let mut writer = Writer::new(skein_lib::length::usize_of(
            value.measure(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture"),
        ));
        value.write(&jig_client_protocol_wire::CEILINGS, &mut writer).expect("measured fixture");
        writer.finish()
    }};
}
fn answer(value: &w::Final, limits: &Limits) -> c::Answer {
    let record = encoded!(value);
    let owned = w::Owned { owner: w::Owner::Jig, version: jig_client_protocol_wire::VERSION, record };
    translate::answers::answer(w::KeyedAnswer::FinalAnswer(owned), limits).expect("public answer")
}
fn outcome(value: &w::Outcome, limits: &Limits) -> Result<c::Outcome, ()> {
    let final_value = w::Final::Done(w::Done { outcome: value.clone() });
    match answer(&final_value, limits) {
        c::Answer::Done(value) => Ok(value),
        c::Answer::Refused { .. } => Err(()),
    }
}
fn refusal(value: &w::Refusal, limits: &Limits) -> Result<c::Refusal, ()> {
    let final_value = w::Final::Refused(w::Refused { refusal: value.clone(), at: None });
    match answer(&final_value, limits) {
        c::Answer::Refused { refusal, .. } => Ok(refusal),
        c::Answer::Done(_) => Err(()),
    }
}
#[test]
fn an_undeclared_default_speed_is_refused_as_shape_across_the_wire() {
    use jig_core_kit::world::{Setup, World};
    use jig_domain_core::{self as core, settings as s};
    use jig_domain_people as p;
    use jig_engine_clients_kit::{Peer, fixtures};
    use jig_test_domain as root;
    use skein_lib::Reader;

    let mut setup = Setup { seed: 6002, require_assignment: false, ..Setup::default() };
    setup.limits.core.run_bytes = 1024;
    let mut world = World::new(setup);
    assert!(world.domain.ready(), "start: {:?}", world.domain.start_failure());
    let owner = world.sign_in.expect("owner sign-in");
    world.send(root::Event::Core(core::Event::People(p::Event::Roles {
        project: 0,
        holdings: Box::new([p::Holding { person: 1, role: p::Role::Owner }]),
    })));
    let setting = s::Setting::Model(1);
    let limits = fixtures::limits().wire;
    let current = world.domain.settings().value(setting).expect("catalogue model");
    let value = jig_protocol_engine_clients::translate::settings::value(current, &[], &limits).unwrap();
    let w::SettingValue::Model(mut model) = value else { panic!("model value") };
    model.model.speeds = List::from_boxed(Box::new([w::SettingSpeed {
        name: b"fast".as_slice().into(),
        also: List::with_capacity(0),
        input_price: model.model.input_price,
        cached_price: model.model.cached_price,
        cache_write_price: model.model.cache_write_price,
        output_price: model.model.output_price,
    }]));
    model.model.speed = Some(b"unoffered".as_slice().into());
    let setting = w::Setting::Model(w::SettingNumber { number: 1 });
    let item =
        w::SettingItem { setting, edit: w::SettingEdit::Set(w::SettingSet { value: w::SettingValue::Model(model) }) };
    let piece = encoded!(&item);
    let mut pieces = Writer::new(4 + piece.len());
    pieces.put(&u32::try_from(piece.len()).unwrap().to_be_bytes()).unwrap();
    pieces.put(&piece).unwrap();
    let ask =
        w::Ask::ChangeSettings(w::ChangeSettings { against: 1, items: pieces.finish(), running: w::Running::Stays });
    let request =
        w::Keyed { key: [1; 16], owner: w::Owner::Jig, version: w::VERSION, request: encoded!(&ask), value: None };
    let mut peer = Peer::started(&fixtures::settings());
    let bearer = fixtures::signed_in(&mut peer);
    let exchange = peer.post(b"/jig/v1/request", encoded!(&request), Some(&bearer));
    let Some(core::Event::ChangeSettings { reply_to, key, digest, change, .. }) = peer.take_core() else {
        panic!("model set reaches the core")
    };
    let reply_token = reply_to.into_token();
    world.send(root::Event::Core(core::Event::ChangeSettings {
        reply_to: skein_lib::ReplyTo::new(reply_token),
        sign_in: owner,
        key,
        digest,
        change,
    }));
    let reply = world.people_answers.last().expect("core's reply").clone();
    peer.answer_request(skein_lib::ReplyTo::new(reply_token), reply);
    let bytes = peer.answered(exchange).expect("wire answer");
    let record = w::KeyedAnswer::decode(&limits, &mut Reader::new(bytes)).unwrap();
    let link = Limits::new(jig_client_kit::test::limits().client.domain, 8).unwrap();
    let answer = translate::answers::answer(record, &link).unwrap();
    assert_eq!(
        answer,
        c::Answer::Refused {
            at: None,
            refusal: c::Refusal::Malformed {
                at: 0,
                why: c::Malformed::Settings(c::MalformedSettings {
                    setting: c::Setting::Model(c::SettingNumber { number: 1 }),
                    fault: c::SettingFault::Shape,
                    detail: None,
                }),
            },
        }
    );
}

#[test]
fn settings_contract_fixtures_keep_the_ask_answers_documents_and_selectors() {
    let mut client = jig_client_kit::test::limits().client;
    client.domain.rows = 8;
    let limits = Limits::new(client.domain, client.exchanges).expect("client limits");
    let t = limits.wire();
    let ask = c::Ask::ChangeSettings(values::settings_change());
    let wire = translate::asks::ask(&ask, t).expect("settings ask");
    // The retained ask follows the same contract as a newly sent one.
    let w::Ask::ChangeSettings(wire) = wire else {
        panic!("settings ask");
    };
    assert_eq!(translate::settings::restore_change_settings(&wire, &limits), Ok(values::settings_change()));
    for read in [
        c::Read::Settings,
        c::Read::Setting(c::ReadSetting { setting: c::Setting::Rules, revision: 6, proposal: None }),
        c::Read::Revision(c::ReadRevision { number: 6 }),
        c::Read::Revisions(c::ReadRevisions { before: Some(8) }),
    ] {
        drop(translate::reads::read(&read));
    }
    assert_eq!(translate::watches::subject(&c::Subject::Settings), w::Subject::Settings);
    let changed = w::Outcome::SettingsChanged(w::OutcomeSettingsChanged { revision: 6 });
    assert_eq!(outcome(&changed, &limits), Ok(c::Outcome::SettingsChanged(c::OutcomeSettingsChanged { revision: 6 })));
    let stale = w::Refusal::Stale(w::RefusalStale { newest: 9 });
    assert_eq!(refusal(&stale, &limits), Ok(c::Refusal::Stale(c::RefusalStale { newest: 9 })));
    let mut tasks = List::with_capacity(1);
    tasks.push(11).expect("one holder");
    let full = w::Refusal::Full(w::RefusalFull { revision: 2, tasks: tasks.clone(), more: 3 });
    assert_eq!(refusal(&full, &limits), Ok(c::Refusal::Full(c::RefusalFull { revision: 2, tasks, more: 3 })));
    for fault in [
        c::SettingFault::Shape,
        c::SettingFault::Count,
        c::SettingFault::Reference,
        c::SettingFault::Rules,
        c::SettingFault::Budget,
        c::SettingFault::Grace,
        c::SettingFault::Owner,
        c::SettingFault::Judged,
        c::SettingFault::Linked,
        c::SettingFault::Range,
    ] {
        let why = c::MalformedSettings { setting: c::Setting::Rules, fault, detail: None };
        let wire = w::Refusal::Malformed(w::RefusalMalformed {
            at: 0,
            why: w::Malformed::Settings(wire::wire_malformed_settings(&why)),
        });
        assert_eq!(refusal(&wire, &limits), Ok(c::Refusal::Malformed { at: 0, why: c::Malformed::Settings(why) }));
    }
    let unchanged = w::Refusal::Malformed(w::RefusalMalformed { at: 0, why: w::Malformed::SettingsUnchanged });
    assert_eq!(refusal(&unchanged, &limits), Ok(c::Refusal::Malformed { at: 0, why: c::Malformed::SettingsUnchanged }));
    let document = values::settings_document();
    let settings = wire::wire_settings_document(&document);
    let bytes = encoded!(&w::ReadValue::Settings(settings.clone()));
    assert_eq!(translate::reads::value(&bytes, &limits), Ok(c::ReadValue::Settings(Box::new(document))));
    let bytes = encoded!(&w::DeliveryValue::Settings(settings));
    assert_eq!(
        translate::watches::delivery(&bytes, &limits),
        Ok(c::Delivery::Settings(Box::new(values::settings_document())))
    );
    let mut value = values::settings_values().into_boxed().into_vec();
    let document = c::SettingDocument {
        overlay: None,
        setting: c::Setting::Rules,
        revision: 6,
        value: Some(value.pop().expect("rules")),
        application: None,
        proposed: None,
    };
    let bytes = encoded!(&w::ReadValue::Setting(wire::wire_setting_document(&document, t).expect("setting")));
    assert_eq!(translate::reads::value(&bytes, &limits), Ok(c::ReadValue::Setting(Box::new(document))));
    for answer in [
        c::RevisionAnswer::Revision(values::settings_revision()),
        c::RevisionAnswer::Forgotten(c::RevisionForgotten { oldest: 4 }),
        c::RevisionAnswer::Unknown,
    ] {
        let bytes = encoded!(&w::ReadValue::Revision(w::ReadRevisionAnswer {
            value: wire::wire_revision_answer(&answer, t).expect("revision")
        }));
        assert_eq!(translate::reads::value(&bytes, &limits), Ok(c::ReadValue::Revision(Box::new(answer))));
    }
    let page = values::settings_page();
    let bytes = encoded!(&w::ReadValue::Revisions(wire::wire_revisions_page(&page)));
    assert_eq!(translate::reads::value(&bytes, &limits), Ok(c::ReadValue::Revisions(Box::new(page))));
}

#[test]
fn engine_settings_reads_and_deliveries_decode_at_the_client_end() {
    use jig_domain_core::{documents as d, reads, settings as s};
    use jig_protocol_engine_clients::translate::settings as engine;
    let limits = Limits::new(jig_client_kit::test::limits().client.domain, 4).expect("client limits");
    let t = limits.wire();
    let head = s::Head {
        number: 6,
        against: 5,
        at: skein_lib::Wall::EPOCH,
        by: s::Decider::Release,
        proposal: None,
        moved: false,
    };
    let mut settings = List::with_capacity(1);
    settings
        .push(d::SettingLine {
            name: None,
            setting: s::Setting::Rules,
            shown: None,
            follows: d::Follows::Default,
            last: Some(6),
            application: None,
        })
        .expect("rules line");
    let mut holding = List::with_capacity(1);
    holding.push(d::HeldRevision { head: head.clone(), holders: 2 }).expect("held newest");
    let source = d::Settings { newest: 6, settings, held: holding, pending: List::with_capacity(0) };
    let wire = engine::document(&source).expect("engine inventory");
    let bytes = encoded!(&w::ReadValue::Settings(wire.clone()));
    let c::ReadValue::Settings(document) = translate::reads::value(&bytes, &limits).expect("client inventory") else {
        panic!("settings inventory")
    };
    assert_eq!(document.newest, 6);
    assert_eq!(document.held.get(0).expect("held revision").holders, 2);
    assert_eq!(document.settings.get(0).expect("rules line").last, Some(6));
    let bytes = encoded!(&w::DeliveryValue::Settings(wire));
    assert_eq!(translate::watches::delivery(&bytes, &limits), Ok(c::Delivery::Settings(document)));

    let mut config = jig_test_profile::config(9371);
    config.core.defaults.rules.ceiling.models = jig_domain_authority::Models::ALL;
    let spend = config.core.defaults.rules.period_spend;
    let source = reads::SettingDocument {
        overlay: None,
        setting: s::Setting::Rules,
        revision: 6,
        value: Some(s::Value::Rules(Box::new(config.core.defaults.rules))),
        application: None,
        proposed: None,
    };
    let wire = engine::setting_document(&source, &[], t).expect("engine whole setting");
    let bytes = encoded!(&w::ReadValue::Setting(wire));
    let c::ReadValue::Setting(document) = translate::reads::value(&bytes, &limits).expect("client whole setting")
    else {
        panic!("setting")
    };
    let Some(c::SettingValue::Rules(rules)) = document.value else { panic!("rules") };
    assert_eq!(rules.period_spend, spend);

    for (source, expected) in [
        (reads::RevisionAnswer::Unknown, c::RevisionAnswer::Unknown),
        (
            reads::RevisionAnswer::Forgotten { oldest: 4 },
            c::RevisionAnswer::Forgotten(c::RevisionForgotten { oldest: 4 }),
        ),
    ] {
        let wire = w::ReadRevisionAnswer { value: engine::revision(&source, &[], t).expect("engine revision answer") };
        let bytes = encoded!(&w::ReadValue::Revision(wire));
        assert_eq!(translate::reads::value(&bytes, &limits), Ok(c::ReadValue::Revision(Box::new(expected))));
    }
    let mut rows = List::with_capacity(1);
    rows.push(reads::RevisionRow { head, items: List::with_capacity(0), more: 3 }).expect("revision row");
    let page = reads::RevisionsPage { rows, next: Some(6), forgotten: false };
    let bytes = encoded!(&w::ReadValue::Revisions(engine::revisions(&page)));
    let c::ReadValue::Revisions(page) = translate::reads::value(&bytes, &limits).expect("client page") else {
        panic!("revisions page")
    };
    assert_eq!(page.next, Some(6));
    assert_eq!(page.rows.get(0).expect("revision row").more, 3);
}

#[test]
fn an_owner_moves_running_work_through_the_client_and_hears_both_counts() {
    use jig_client::{Asker, Client, Event, Key, Told};
    use jig_client_kit::{
        harness::ClientId,
        joint::{engine::Driver, session::Session},
    };
    use skein_lib::{Time, Wall};
    let now = Time::from_nanos(17);
    let wall = Wall::from_nanos(1_000_000_000_000);
    let (configuration, owner_limits) = jig_core_kit::peers::fixture(8455, 0);
    let driver =
        Driver::new(configuration, owner_limits, jig_core_kit::peers::Peers::new(Vec::new(), Vec::new()), now, wall);
    let mut limits = jig_client_kit::test::limits();
    // The real deployment's frame uses the joint world's document bounds.
    limits.client.domain.turn_text = 4096;
    limits.client.domain.turn_bytes = 4096;
    limits.client.domain.rows = 32;
    limits.client.domain.projects = 64;
    limits.jig.roles = 64;
    limits.client.domain.text = 4096;
    limits.client.domain.long_text = 4096;
    limits.client.domain.document_bytes = 1_048_576;
    let mut client = Client::unlinked(&limits.client, 8455).expect("public client");
    let codec = jig_protocol_transcript::Limits::new(
        1_048_576,
        c::TurnLimits { messages: 64, blocks: 16, text: 65_536, outcome: 65_536 },
    );
    let mut session = Session::new(driver, limits, codec);
    session.bind(
        ClientId(0),
        1,
        jig_domain_people::Identity {
            key: jig_domain_people::IdentityKey { provider: 0, subject: 7_u64.to_be_bytes().into() },
            login: Box::from(b"login".as_slice()),
            name: Box::from(b"Person".as_slice()),
        },
        now,
        wall,
    );
    move_exchange(&mut session, &mut client, now, wall, Event::StartUnlinked { kept: List::with_capacity(0) });
    let key = Key { asker: Asker::JIG, number: 1 };
    let newest = session.driver.world.domain.settings().newest();
    let told = move_exchange(
        &mut session,
        &mut client,
        now,
        wall,
        Event::Send {
            key,
            name: Some([7; 16]),
            request: jig_client::Ask::Jig(Box::new(c::Ask::MoveToNewest(c::MoveToNewest { revision: newest }))),
        },
    );
    assert!(told.iter().any(|told| matches!(told, Told::Answered { key: answered, outcome: jig_client::Outcome::Answered(jig_client::Answered::Jig(c::Answer::Done(c::Outcome::MovedToNewest(value)))) } if *answered == key && *value == c::OutcomeMovedToNewest { newest, moved: 0, stayed: 0 })));
}

fn move_exchange(
    session: &mut jig_client_kit::joint::session::Session,
    client: &mut jig_client::Client,
    now: skein_lib::Time,
    wall: skein_lib::Wall,
    event: jig_client::Event,
) -> Vec<jig_client::Told> {
    let mut out = skein_lib::Queue::uncapped();
    client.step_unlinked(now, wall, event, &mut out);
    let mut told = Vec::new();
    let mut kept = std::collections::BTreeMap::new();
    for _ in 0..16 {
        while let Some(request) = out.pop() {
            if let jig_client::UnlinkedRequest::Told { told: value } = request {
                told.push(value);
            } else if let jig_client::UnlinkedRequest::Down { down: jig_client::Down::Save { record } } = request {
                let _previous = kept.insert(record.key, record);
            } else if let jig_client::UnlinkedRequest::Down { down: jig_client::Down::Erase { key } } = request {
                let _previous = kept.remove(&key);
            } else {
                session
                    .send(
                        jig_client_kit::harness::ClientId(0),
                        now,
                        wall,
                        jig_client_test_domain::Request::Unlinked { request },
                    )
                    .expect("public request");
            }
        }
        let callbacks = session.poll(now, wall).expect("real engine replies");
        if callbacks.is_empty() {
            return told;
        }
        for (owner, callback) in callbacks {
            session.received(owner, now, wall, &callback);
            let jig_client_test_domain::Event::Client { event } = callback else { panic!("client callback") };
            client.step_unlinked(now, wall, event, &mut out);
        }
    }
    panic!("public exchange did not settle");
}
