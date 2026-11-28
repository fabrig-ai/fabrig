//! The open and commit catalogue (jig/docs/design/protocol/store/records.md).
use super::{fixtures, model, policy, template};
use crate::{Event, Request, Restore, Row, Store, Write, codec, keys};
use jig_domain_authority::Models;
use jig_domain_core_vocabulary::{self as core, settings as s};
use jig_domain_tasks::{MODELS, ModelEffort};
use skein_lib::{List, Token};

// A person's setting takes the highest free number (domain/engine/settings-catalogue.md, 1).
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

fn model_entry(number: u32, name: Box<[u8]>, source: s::ModelSource, withdrawn: bool) -> s::CatalogueModel {
    s::CatalogueModel {
        first_read: Some(skein_lib::Wall::from_nanos(123)),
        number,
        model: core::Model { name, ..model() },
        shown: None,
        description: None,
        picker: true,
        withdrawn,
        source,
        estimated: true,
    }
}

fn model_changed(entry: s::CatalogueModel) -> core::Record {
    changed(s::Setting::Model(entry.number), s::Value::Model(Box::new(entry)))
}

fn personal_model(withdrawn: bool) -> core::Record {
    model_changed(model_entry(PERSON, b"personal".as_slice().into(), s::ModelSource::Person, withdrawn))
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
        main: ModelEffort { model, effort: Some(1), speed: None },
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

fn open(checked: &crate::Checked, stores: crate::Stores) -> Store {
    open_with_defaults(checked, stores, &defaults())
}

fn open_with_defaults(checked: &crate::Checked, stores: crate::Stores, defaults: &[core::Model]) -> Store {
    let mut store = Store::over(checked, fixtures::APPLICATION, defaults, stores);
    let events = fixtures::run(
        &mut store,
        Request::Open {
            purpose: crate::Purpose::Serve,
            owner: OWNER,
            root: fixtures::ROOT,
            installation: fixtures::INSTALLATION,
        },
    );
    let [Event::Opened { .. }, Event::Census { .. }] = &*events else { panic!("open: {events:?}") };
    store
}

fn restored(store: &mut Store) -> Box<[core::Record]> {
    restored_settings(store, STORED_SETTINGS)
}

fn restored_settings(store: &mut Store, rows: u32) -> Box<[core::Record]> {
    let mut result = List::with_capacity(rows);
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
    let mut store = open(&crate::check(&fixtures::store_limits()).expect("fixture store limits"), fixtures::stores());
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
    let saved = saved(false);
    let mut store = open(&saved.checked(), saved.into_stores());
    let rows = restored(&mut store);
    assert!(rows.contains(&personal_model(false)), "model first read stays whole");
    assert!(rows.contains(&template_record()), "template choice keeps position 63 and its effort");
    assert!(rows.contains(&policy_record()), "policy ceiling and roles keep bit 63");
}

#[test]
fn a_withdrawn_model_keeps_its_name_and_position() {
    let saved = saved(true);
    let mut store = open(&saved.checked(), saved.into_stores());
    let rows = restored(&mut store);
    assert!(rows.contains(&personal_model(true)), "withdrawn setting stays whole");
    assert!(rows.contains(&template_record()), "withdrawn model's held choice keeps its position");
    assert!(rows.contains(&policy_record()), "withdrawn model's named policy keeps its position");
}

fn saved_template() -> Store {
    let mut store = open(&crate::check(&fixtures::store_limits()).expect("fixture store limits"), fixtures::stores());
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

// Seven defaults, then deployment changes at numbers 39 through MODELS.
const STOCK: u32 = 7;
const FIRST_ADDED: u32 = 39;
const ADDED_INDEX: u8 = 38;
const POSITIONS: usize = 64; // Highest model number, MODELS.
const GAPPED_SETTINGS: u32 = 27; // 26 added models and one template, no policy.

fn numbered_catalogue() -> Box<[s::CatalogueModel]> {
    let mut entries = List::with_capacity(MODELS);
    for number in (1..=STOCK).chain(FIRST_ADDED..=MODELS) {
        let name = skein_lib::bytes::concat(&[b"m", skein_lib::Decimal::of(u64::from(number)).as_bytes()]);
        let source = if number <= STOCK { s::ModelSource::Stock } else { s::ModelSource::Person };
        entries.push(model_entry(number, name, source, false)).expect("one entry per number");
    }
    entries.into_boxed()
}

#[test]
fn a_choice_keeps_its_model_number_across_a_gapped_restart() {
    use skein_lib::length::usize_of;

    let limits = fixtures::registered_store_limits();
    let checked = crate::check(&limits).expect("fixture store limits");
    let stores = crate::Stores {
        store: skein_kv::Store::memory(&limits.store),
        secrets: skein_kv::Store::memory(&limits.secrets),
    };
    let entries = numbered_catalogue();
    let (stock, added) = entries.split_at_checked(usize_of(STOCK)).expect("seven stock entries");
    let defaults = s::catalogue::rows(stock);
    let mut store = open_with_defaults(&checked, stores, &defaults);
    store.catalogue(s::model_names(&s::catalogue::rows(&entries)));
    // Each commit has one header and room for the remaining writes to be model changes.
    let writes = limits.commits.writes;
    let changes_per_commit = writes.checked_sub(1).expect("header uses one write");
    let mut commit = 1_u64;
    for chunk in added.chunks(usize_of(changes_per_commit)) {
        let mut batch = List::with_capacity(writes);
        batch.push(fixtures::header(commit)).expect("header write");
        for entry in chunk {
            batch.push(Write::Save(model_changed(entry.clone()))).expect("model change write");
        }
        fixtures::committed(&mut store, batch.into_boxed());
        commit = commit.checked_add(1).expect("next commit");
    }
    let choice = template_choice(ADDED_INDEX);
    fixtures::committed(&mut store, Box::new([fixtures::header(commit), Write::Save(choice.clone())]));

    let mut store = open_with_defaults(&checked, store.into_stores(), &defaults);
    let rows = restored_settings(&mut store, GAPPED_SETTINGS);
    assert!(rows.contains(&choice), "template choice keeps m39's index and effort");
    assert_eq!(store.names.len(), POSITIONS);
    let index = usize::from(ADDED_INDEX);
    let added = store.names.get(index).expect("model 39's slot");
    assert_eq!(added.name.as_ref(), b"m39");
    assert_eq!(added.efforts.as_ref(), &[Box::from(b"low".as_slice()), Box::from(b"high".as_slice())]);
    for hole in store.names.get(usize_of(STOCK)..index).expect("holes within the catalogue") {
        assert_eq!(hole, &s::ModelName { name: Box::default(), efforts: Box::default() });
    }
}

#[test]
fn a_name_that_left_stays_gone() {
    let saved = saved_template();
    let mut store = open(&saved.checked(), saved.into_stores());
    assert_eq!(restored(&mut store).as_ref(), [template_choice(ModelEffort::GONE)]);
}

#[test]
fn a_hole_never_matches_an_empty_model_name() {
    assert_eq!(s::model_bit(b"", &names()), 0);
}
