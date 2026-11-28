//! Catalogue routing before the commit (jig/docs/design/domain/engine/settings-catalogue.md, 1).
use crate::{PartyRequest, Request, StoreRequest};
use jig_domain_core::{
    self as core,
    settings::{CatalogueModel, ModelSource},
};
use skein_lib::Queue;

#[test]
fn an_ask_for_the_catalogue_tells_the_store_then_the_clients() {
    let (mut domain, env) = crate::tests::resume_fixture::root();
    let models: Box<[CatalogueModel]> = Box::new([CatalogueModel {
        first_read: None,
        number: 1,
        model: core::Model::hole(),
        shown: None,
        description: None,
        picker: false,
        withdrawn: false,
        source: ModelSource::Stock,
        estimated: false,
    }]);
    let mut decision = crate::Decision::decide(&mut domain.journal, &env.limits.journal);
    decision.write(&env.limits.journal, crate::Write::Wires).unwrap();
    super::application(&mut domain, &env, &mut decision, 0, core::Ask::Catalogue { models: models.clone() });
    crate::decision::accept_routed(
        &mut domain.journal,
        &mut domain.engine.core_mut().counters,
        &env.limits.journal,
        decision,
        false,
    )
    .unwrap();
    let mut out = Queue::uncapped();
    crate::release(&mut domain, &env, &mut out);
    let Some(Request::Store(StoreRequest::Catalogue { names })) = out.pop() else { panic!("store catalogue first") };
    assert_eq!(names.as_ref(), &[core::settings::ModelName { name: Box::default(), efforts: Box::default() }]);
    let Some(Request::Party(PartyRequest::Catalogue { models: sent })) = out.pop() else {
        panic!("clients catalogue second")
    };
    assert_eq!(sent, models);
    let Some(Request::Store(StoreRequest::Commit { .. })) = out.pop() else {
        panic!("the commit follows both replacements")
    };
}

// Seven stock models and the deployment's models 39 through MODELS (settings-catalogue.md, 1).
const STOCK: u32 = 7;
const FIRST_ADDED: u32 = 39;
const LIMIT: u32 = jig_domain_tasks::MODELS;
const DENSE: u32 = 3;
const BEFORE_ADDED: usize = 38;

fn numbered_models(stock: u32, highest: u32) -> Box<[CatalogueModel]> {
    let mut models = skein_lib::List::with_capacity(highest);
    for number in (1..=stock).chain(FIRST_ADDED..=highest) {
        let name = skein_lib::bytes::concat(&[b"m", skein_lib::Decimal::of(u64::from(number)).as_bytes()]);
        let efforts =
            if number.is_multiple_of(2) { Box::default() } else { Box::from([Box::from(b"high".as_slice())]) };
        models
            .push(CatalogueModel {
                first_read: None,
                number,
                model: core::Model { name, efforts, ..core::Model::hole() },
                shown: None,
                description: None,
                picker: true,
                withdrawn: false,
                source: if number <= stock { ModelSource::Stock } else { ModelSource::Person },
                estimated: false,
            })
            .expect("one row per numbered model");
    }
    models.into_boxed()
}

#[test]
fn the_catalogue_routes_names_and_efforts_by_model_number() {
    use skein_lib::length::usize_of;

    for (stock, highest, holes) in [(STOCK, LIMIT, usize_of(STOCK)..BEFORE_ADDED), (DENSE, DENSE, 0..0)] {
        let models = numbered_models(stock, highest);
        let (mut domain, env) = crate::tests::resume_fixture::root();
        let mut decision = crate::Decision::decide(&mut domain.journal, &env.limits.journal);
        super::application(&mut domain, &env, &mut decision, 0, core::Ask::Catalogue { models: models.clone() });
        let mut out = Queue::uncapped();
        crate::release(&mut domain, &env, &mut out);
        let Some(Request::Store(StoreRequest::Catalogue { names })) = out.pop() else { panic!("store catalogue") };
        assert_eq!(names.len(), usize_of(highest));
        for row in &models {
            let slot = names.get(usize_of(row.number.checked_sub(1).expect("numbered model"))).expect("model slot");
            assert_eq!(slot.name, row.model.name, "name at model number {}", row.number);
            assert_eq!(slot.efforts, row.model.efforts, "efforts at model number {}", row.number);
        }
        for hole in names.get(holes).expect("holes within the catalogue") {
            assert_eq!(hole, &core::settings::ModelName { name: Box::default(), efforts: Box::default() });
        }
        let Some(Request::Party(PartyRequest::Catalogue { models: sent })) = out.pop() else {
            panic!("clients catalogue")
        };
        assert_eq!(sent, models);
    }
}
