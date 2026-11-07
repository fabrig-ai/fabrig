//! Catalogue routing before the commit (jig/docs/design/domain/engine.md, 17.11).
use super::fixtures;
use crate::{Request, Write};
use jig_domain_core::{
    self as core,
    settings::{CatalogueModel, ModelSource},
};
use skein_lib::Queue;

#[test]
fn an_ask_for_the_catalogue_tells_the_store_then_the_clients() {
    let mut domain = fixtures::domain();
    let env = fixtures::env();
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
    let mut requests = Queue::uncapped();
    requests.push(core::Request::Ask { connector: 0, ask: core::Ask::Catalogue { models: models.clone() } });
    let mut decision = domain.journal.decide();
    decision.push_write(Write::Wires);
    crate::routes::route_core(&mut domain, &env, &mut decision, requests, &mut Queue::uncapped());
    domain.journal.accept(decision);
    let mut out = Queue::uncapped();
    crate::release(&mut domain, &env, &mut out);
    let Some(Request::StoreCatalogue { names }) = out.pop() else { panic!("store catalogue first") };
    assert_eq!(names.as_ref(), &[core::settings::ModelName { name: Box::default(), efforts: Box::default() }]);
    let Some(Request::ClientsCatalogue { models: sent }) = out.pop() else { panic!("clients catalogue second") };
    assert_eq!(sent, models);
    let Some(Request::Commit { .. }) = out.pop() else { panic!("the commit follows both replacements") };
}
