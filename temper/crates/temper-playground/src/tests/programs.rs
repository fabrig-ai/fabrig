//! Shared inputs to the composition's programs (temper/docs/design/playground.md, 3).
use super::{Parameters, install, parameters, prepared_schemas, with};
use crate::{stories, story};
use skein_codec::schema::Schema;
use skein_lib::Wall;
use std::{ptr, rc::Rc};
use temper_engine_service_vocabulary::store::OWNER_SETS;

#[test]
fn prepared_schemas_share_the_binary_texts_in_order() {
    let schemas = prepared_schemas();
    assert!(ptr::eq(schemas, prepared_schemas()), "opens share one schema slice");
    let expected = OWNER_SETS.prepare();
    assert!(schemas.iter().map(Schema::text).eq(expected.iter().map(Schema::text)));
}

fn assert_parameters(expected: &Parameters) {
    let current = parameters();
    assert_eq!(current.deployment, expected.deployment);
    assert_eq!(current.seed, expected.seed);
    assert_eq!(current.links, expected.links);
    assert!(Rc::ptr_eq(&current.disk, &expected.disk));
}

#[test]
fn scoped_worlds_restore_the_installed_parameters() {
    let installed = story::parameters(&story::world(stories::A.start(7, Wall::EPOCH)));
    let scoped = story::parameters(&story::world(stories::A.start(11, Wall::EPOCH)));
    install(installed.clone());
    with(scoped.clone(), || {
        assert_parameters(&scoped);
        with(installed.clone(), || assert_parameters(&installed));
        assert_parameters(&scoped);
    });
    assert_parameters(&installed);
}
