//! Catalogue replacement and numbered holes (jig/docs/design/domain/engine/settings-catalogue.md, 1).
#![expect(clippy::disallowed_macros, reason = "ordinary tests make exact wire-bound byte fixtures")]
use super::fixtures as f;
use crate::{Event, Message, Request, translate};
use alloc::boxed::Box;
use jig_client_protocol_wire as w;
use jig_client_protocol_wire::Record as _;
use jig_domain_core_vocabulary::{
    self as core,
    settings::{CatalogueModel, ModelSource},
};
use jig_domain_people as p;
use skein_lib::{List, Reader, Writer};

// Three numbered positions: careful, a hole, and the newly added model.
const POSITIONS: usize = 3;
const NEW: &[u8] = b"new-model";

fn entry(number: u32, name: Box<[u8]>) -> CatalogueModel {
    let model = core::Model {
        provider: 3,
        dialect: 2,
        endpoint: 1,
        program: None,
        name,
        cache_life_ms: 0,
        window: 200_000,
        output: 32_000,
        input_price: 3,
        cached_price: 1,
        cache_write_price: Some(5),
        output_price: 15,
        price_unit: 1_000_000,
        efforts: Box::new([]),
        effort: None,
    };
    CatalogueModel {
        first_read: None,
        number,
        model,
        shown: None,
        description: None,
        picker: true,
        withdrawn: false,
        source: ModelSource::Stock,
        estimated: false,
    }
}

fn catalogue() -> Box<[CatalogueModel]> {
    let mut added = entry(3, NEW.into());
    added.shown = Some(b"New model".as_slice().into());
    added.description = Some(b"Added to the catalogue".as_slice().into());
    added.picker = false;
    added.source = ModelSource::List;
    added.first_read = Some(skein_lib::Wall::from_nanos(123));
    Box::new([added, entry(1, b"careful".as_slice().into())])
}

fn create() -> w::Ask {
    let limits = f::wire();
    let (w::Ask::Create(create), _) = f::create_ask() else { panic!("create fixture") };
    let bytes = skein_codec::piece::split(&mut Reader::new(create.tasks.as_ref())).unwrap().unwrap();
    let mut task = w::NewTask::decode(&limits, &mut Reader::new(&bytes)).unwrap();
    task.llm = Some(w::LlmChoice {
        later: None,
        account: None,
        main: w::ModelEffort { model: NEW.into(), effort: None },
        subagents: List::with_capacity(0),
    });
    let task = task;
    let mut writer = Writer::new(
        usize::try_from(task.measure(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture")).unwrap(),
    );
    task.write(&jig_client_protocol_wire::CEILINGS, &mut writer).unwrap();
    w::Ask::Create(w::Create { project: f::PROJECT, tasks: f::pieces(&[writer.finish()]) })
}

fn model_exchange(mut env: skein_lib::Env<crate::Limits>) -> f::Exchange {
    env.limits.wire.owned_record = w::CEILINGS.owned_record;
    f::Exchange::with_env(env)
}

fn me_models(exchange: &mut f::Exchange, model: u8) -> List<w::MeModel> {
    let (owner, reply, _) = exchange.read_until(&w::Read::Me, skein_lib::Wall::from_nanos(u64::MAX));
    let mut frame = f::frame_two_projects();
    for project in frame.projects.iter_mut() {
        project.offer = List::from_boxed(Box::new([core::Offered { model, accounts: List::with_capacity(0) }]));
    }
    exchange.release_read(reply, core::reads::Answer::Document(Box::new(core::documents::Document::Frame(frame))));
    let w::ReadAnswer::Value(value) = exchange.read_answer(owner) else { panic!("me read") };
    let w::ReadValue::Me(me) =
        w::ReadValue::decode(&exchange.env.limits.wire, &mut Reader::new(value.record.as_ref())).unwrap()
    else {
        panic!("me")
    };
    let project = skein_codec::piece::split(&mut Reader::new(me.projects.as_ref())).unwrap().unwrap();
    w::MeProject::decode(&exchange.env.limits.wire, &mut Reader::new(&project)).unwrap().models
}

#[test]
fn a_catalogue_replaces_the_models_a_request_names() {
    for clock in [0, 123, 9_000_000_000_000_000] {
        let mut env = f::env();
        env.wall = skein_lib::Wall::from_nanos(clock);
        env.now = skein_lib::Time::from_nanos(clock);
        env.limits.wire.keyed_request = w::CEILINGS.keyed_request;
        env.limits.message_bytes = w::CEILINGS.keyed_request;
        let mut exchange = model_exchange(env);
        let message = || {
            let keyed = w::Keyed {
                key: [5; 16],
                owner: w::Owner::Jig,
                version: jig_client_protocol_wire::VERSION,
                request: f::encode_ask(&create()),
                value: None,
            };
            let mut writer = Writer::new(
                usize::try_from(keyed.measure(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture")).unwrap(),
            );
            keyed.write(&jig_client_protocol_wire::CEILINGS, &mut writer).unwrap();
            Message::Request(writer.finish())
        };
        let owner = exchange.send(message());
        exchange.authenticate_until(skein_lib::Wall::from_nanos(u64::MAX));
        let (status, bytes) = exchange.response(owner);
        assert_eq!(status, 200);
        let answer = w::KeyedAnswer::decode(&f::wire(), &mut Reader::new(bytes)).unwrap();
        let w::Final::Refused(answer) = f::final_record(answer, &f::wire()) else { panic!("unconfigured refusal") };
        let w::Refusal::Llm(refusal) = &answer.refusal else { panic!("LLM refusal") };
        assert_eq!((refusal.model.as_ref(), (&refusal.why)), (NEW, &w::LlmWhy::Unconfigured));
        exchange.component.up(&exchange.env, &mut exchange.out);
        while exchange.out.pop().is_some() {}
        exchange.component.down(
            &exchange.env,
            Event::Catalogue { models: translate::llm::catalogue(&catalogue()) },
            &mut exchange.out,
        );
        assert!(exchange.out.is_empty(), "the core's me mark drives watchers");
        exchange.send(message());
        exchange.authenticate_until(skein_lib::Wall::from_nanos(u64::MAX));
        let Some(Request::Core(event)) = exchange.out.pop() else { panic!("admitted create") };
        let core::Event::People(p::Event::Ask { ask: p::Ask::Create { batch, .. }, .. }) = *event else {
            panic!("create")
        };
        assert_eq!(batch.first().unwrap().llm.as_ref().unwrap().main.model, 2);
        let offered = me_models(&mut exchange, 2);
        let added = offered.get(0).unwrap();
        assert_eq!(added.name.as_ref(), NEW);
        assert_eq!(added.new_until, Some(123 + core::settings::catalogue::NEW_FOR.as_nanos()));
    }
}

#[test]
fn a_hole_is_no_model() {
    let models = translate::llm::catalogue(&catalogue());
    assert_eq!(models.len(), POSITIONS);
    assert_eq!(models.first().unwrap().name.as_ref(), b"careful");
    assert!(models.get(1).unwrap().name.is_empty());
    assert_eq!(models.first().unwrap().new_until, None);
    assert_eq!(models.get(1).unwrap().new_until, None);
    let added = models.get(2).unwrap();
    assert_eq!(added.name.as_ref(), NEW);
    assert_eq!(added.new_until, Some(skein_lib::Wall::from_nanos(123 + core::settings::catalogue::NEW_FOR.as_nanos())));
    assert_eq!(added.provider, Some(3));
    assert_eq!((added.window, added.output), (200_000, 32_000));
    assert_eq!(added.prices, crate::Prices { input: 3, cached: 1, cache_write: Some(5), output: 15, unit: 1_000_000 });
    assert_eq!(added.shown.as_deref(), Some(b"New model".as_slice()));
    assert_eq!(added.description.as_deref(), Some(b"Added to the catalogue".as_slice()));
    assert!(!added.picker);
    let names = translate::llm::authority(jig_domain_authority::Models::from_parts(0b111, false), &models);
    let Some(w::ModelSet::Names(names)) = names else { panic!("named authority") };
    assert_eq!(names.names.as_slice(), &[Box::<[u8]>::from(b"careful".as_slice()), NEW.into()]);
    let offer = [
        core::Offered { model: 1, accounts: List::with_capacity(0) },
        core::Offered { model: 2, accounts: List::from_boxed(Box::new([7])) },
    ];
    let shown = translate::llm::models(&offer, &models).unwrap();
    assert_eq!(shown.len(), 1);
    assert_eq!(shown.get(0).unwrap().name.as_ref(), NEW);
    assert_eq!(shown.get(0).unwrap().accounts.as_slice(), &[7]);
    assert_eq!(
        translate::llm::refusal(1, p::LlmWhy::Charter, &models),
        Err(w::Problem { path: w::paths::REFUSAL_LLM_MODEL, reason: skein_codec::Reason::Bound })
    );
    let w::Refusal::Llm(refusal) = translate::llm::refusal(2, p::LlmWhy::Charter, &models).unwrap() else {
        panic!("named refusal")
    };
    assert_eq!(refusal.model.as_ref(), NEW);
}

// The numbered catalogue has seven stock models and deployment models 39 through MODELS.
const STOCK: u32 = 7;
const FIRST_ADDED: u32 = 39;
const ADDED_INDEX: u8 = 38;
const HOLE_INDEX: usize = 7;

#[test]
fn clients_keep_catalogue_numbers_across_holes_and_in_me() {
    let mut entries = List::with_capacity(jig_domain_tasks::MODELS);
    for number in (1..=STOCK).chain(FIRST_ADDED..=jig_domain_tasks::MODELS) {
        let name = skein_lib::bytes::concat(&[b"m", skein_lib::Decimal::of(u64::from(number)).as_bytes()]);
        entries.push(entry(number, name)).expect("one entry per model number");
    }
    let models = translate::llm::catalogue(entries.as_slice());
    let mut exchange = model_exchange(f::env());
    exchange.component.down(&exchange.env, Event::Catalogue { models }, &mut exchange.out);
    let table = &exchange.component.settings.models;
    let added = table.get(usize::from(ADDED_INDEX)).expect("added model's slot");
    assert_eq!((added.name.as_ref(), added.number), (b"m39".as_slice(), FIRST_ADDED));
    let hole = table.get(HOLE_INDEX).expect("hole's slot");
    assert_eq!((hole.name.as_ref(), hole.number), (b"".as_slice(), 0));
    let offered = me_models(&mut exchange, ADDED_INDEX);
    let [added] = offered.as_slice() else { panic!("one offered model") };
    assert_eq!((added.name.as_ref(), added.number), (b"m39".as_slice(), FIRST_ADDED));
}

#[test]
fn a_catalogue_at_the_wire_bound_replaces_the_models() {
    let mut exchange = f::Exchange::new();
    let mut models = translate::llm::catalogue(&catalogue());
    models.last_mut().unwrap().shown =
        Some(vec![b'n'; usize::try_from(exchange.env.limits.wire.me_model_shown_some).unwrap()].into_boxed_slice());
    exchange.component.down(&exchange.env, Event::Catalogue { models }, &mut exchange.out);
    assert_eq!(
        exchange.component.settings.models.last().unwrap().shown.as_ref().unwrap().len(),
        usize::try_from(exchange.env.limits.wire.me_model_shown_some).unwrap()
    );
}
