//! The open and commit catalogue (jig/docs/design/protocol/store/records.md).
use super::{fixtures, model, policy, template};
use crate::{Event, Request, Restore, Row, Store, Write, codec, keys};
use jig_domain_authority::Models;
use jig_domain_core_vocabulary::{self as core, settings as s};
use jig_domain_tasks::{MODELS, ModelEffort};
use skein_lib::{List, Token};

// A person's setting takes the highest free number (domain/engine/settings.md, 11).
const PERSON: u32 = MODELS;
const PERSON_INDEX: u8 = 63;
const PERSON_BIT: u64 = 1 << PERSON_INDEX;
const OWNER: Token = Token::new(1);
// The fixture saves one template, one policy and one model change.
const STORED_SETTINGS: u32 = 3;

fn defaults() -> [core::Model; 1] {
    [model()]
}

fn changed(setting: s::Setting, value: s::Value) -> core::Record {
    core::Record::Core(core::CoreRecord::Settings(s::Record::Changed {
        setting,
        changed: s::Changed { revision: 1, value: Some(value) },
    }))
}

fn personal_model(withdrawn: bool) -> core::Record {
    let mut model = model();
    model.name = Box::from(b"personal".as_slice());
    changed(
        s::Setting::Model(PERSON),
        s::Value::Model(Box::new(s::CatalogueModel {
            first_read: Some(skein_lib::Wall::from_nanos(123)),
            number: PERSON,
            model,
            shown: None,
            description: None,
            picker: true,
            withdrawn,
            source: s::ModelSource::Person,
            estimated: true,
        })),
    )
}

fn names() -> Box<[s::ModelName]> {
    let mut models = List::with_capacity(MODELS);
    for position in 0..MODELS {
        let model = if position == 0 {
            model()
        } else if position == u32::from(PERSON_INDEX) {
            core::Model { name: Box::from(b"personal".as_slice()), ..model() }
        } else {
            core::Model::hole()
        };
        models.push(model).unwrap();
    }
    codec::names(&models.into_boxed())
}

fn template_choice(model: u8) -> core::Record {
    let mut template = template();
    template.llm = Some(jig_domain_tasks::LlmChoice {
        later: None,
        account: None,
        main: ModelEffort { model, effort: Some(1) },
        subagents: Box::new([]),
    });
    changed(s::Setting::Template(template.number), s::Value::Template(Box::new(template)))
}

fn template_record() -> core::Record {
    template_choice(PERSON_INDEX)
}

fn policy_record() -> core::Record {
    let mut policy = policy();
    policy.policy.ceiling.models = Models::Named(PERSON_BIT);
    policy.policy.ceiling.delegation.kinds = Box::new([]);
    for role in &mut policy.policy.roles {
        role.authority.models = Models::Named(PERSON_BIT);
        role.authority.delegation.kinds = Box::new([]);
    }
    changed(s::Setting::Policy(1), s::Value::Policy(Box::new(policy)))
}

fn open(stores: crate::Stores) -> Store {
    let mut store = Store::over(&fixtures::store_limits(), fixtures::APPLICATION, &defaults(), stores);
    let events = fixtures::run(
        &mut store,
        Request::Open { owner: OWNER, root: fixtures::ROOT, installation: fixtures::INSTALLATION },
    );
    let [Event::Opened { .. }] = &*events else { panic!("open: {events:?}") };
    store
}

fn restored(store: &mut Store) -> Box<[core::Record]> {
    let mut result = List::with_capacity(STORED_SETTINGS);
    let mut after = None;
    loop {
        let events = fixtures::run(
            store,
            Request::Restore {
                owner: OWNER,
                range: Restore::Jig(Box::new([core::RestoreRange::Settings])),
                after,
                rows: fixtures::store_limits().store.page.rows,
                bytes: u64::MAX,
            },
        );
        let [Event::Page { items, next, .. }] = &*events else { panic!("page: {events:?}") };
        for item in items {
            let crate::restore::Item::Row(Row::Jig(row)) = item else { panic!("settings row") };
            result.push(row.clone()).unwrap();
        }
        after = next.clone();
        if after.is_none() {
            break;
        }
    }
    result.into_boxed()
}

fn saved(withdrawn: bool) -> Store {
    let mut store = open(fixtures::stores());
    store.catalogue(names());
    fixtures::committed(
        &mut store,
        Box::new([
            fixtures::header(1),
            Write::Save(template_record()),
            Write::Save(policy_record()),
            Write::Save(personal_model(withdrawn)),
        ]),
    );
    store
}

#[test]
fn a_persons_model_isnt_read_as_gone_after_a_restart() {
    let mut store = open(saved(false).into_stores());
    let rows = restored(&mut store);
    assert!(rows.contains(&personal_model(false)), "model first read stays whole");
    assert!(rows.contains(&template_record()), "template choice keeps position 63 and its effort");
    assert!(rows.contains(&policy_record()), "policy ceiling and roles keep bit 63");
}

#[test]
fn a_withdrawn_model_keeps_its_name_and_position() {
    let mut store = open(saved(true).into_stores());
    let rows = restored(&mut store);
    assert!(rows.contains(&personal_model(true)), "withdrawn setting stays whole");
    assert!(rows.contains(&template_record()), "withdrawn model's held choice keeps its position");
    assert!(rows.contains(&policy_record()), "withdrawn model's named policy keeps its position");
}

fn saved_template() -> Store {
    let mut store = open(fixtures::stores());
    store.catalogue(names());
    fixtures::committed(&mut store, Box::new([fixtures::header(1), Write::Save(template_record())]));
    store
}

#[test]
fn a_later_catalogue_is_the_vocabulary_of_the_next_commit() {
    let mut store = saved_template();
    let row = template_record();
    let key = keys::encode(&row.key(), fixtures::KEY_LIMIT).unwrap();
    let bytes = fixtures::get_kv(&mut store, &key).unwrap();
    assert_eq!(codec::decode(&row.key(), &bytes, &store.codec_limits, &names()), Some(row));
}

#[test]
fn a_name_that_left_stays_gone() {
    let mut store = open(saved_template().into_stores());
    assert_eq!(restored(&mut store).as_ref(), [template_choice(ModelEffort::GONE)]);
}

#[test]
fn a_hole_never_matches_an_empty_model_name() {
    assert_eq!(s::model_bit(b"", &names()), 0);
}
