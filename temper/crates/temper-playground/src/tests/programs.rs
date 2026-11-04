//! Factory parameters for the worker and scoped worlds (temper/docs/design/playground.md, 3).
use super::{Parameters, install, parameters, with};
use crate::{stories, story};
use skein_lib::Wall;
use std::rc::Rc;

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
