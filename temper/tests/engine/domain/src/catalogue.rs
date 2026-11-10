//! Catalogue routing before the commit (jig/docs/design/domain/engine/settings.md, 11).
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
    super::ask(&mut domain, &env, &mut decision, 0, core::Ask::Catalogue { models: models.clone() });
    crate::decision::accept_routed(
        &mut domain.journal,
        &mut domain.core.counters,
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
