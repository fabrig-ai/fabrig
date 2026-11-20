use crate::startup::{Problems, Shape, lines, unserved};
use jig_protocol_config::Path;
use skein_lib::List;

#[test]
fn the_engine_service_refuses_the_other_document_shapes() {
    let mut failures = List::with_capacity(3);
    for (shape, refused) in [(Shape::Standalone, true), (Shape::WorkerAlone, true), (Shape::EngineAlone, false)] {
        let found = unserved(shape);
        if found.is_some() != refused {
            failures.push(shape).unwrap();
        }
        if let Some(found) = found {
            assert_eq!(found.path, Path::root().member(b"workers"));
            assert!(found.file.is_none());
            let mut problems = Problems::empty();
            problems.problems.push(found).unwrap();
            assert_eq!(lines(&problems).as_ref(), b"workers: this build serves an engine alone\n");
        }
    }
    assert!(failures.is_empty(), "wrong service shape acceptance: {failures:?}");
}

#[test]
fn an_engine_without_an_agent_derives_from_stock() {
    use super::stock;
    use crate::{settings, startup};
    use jig_protocol_config as config;
    let mut input: serde_json::Value = serde_json::from_slice(&stock::generated()).unwrap();
    input["workers"] = serde_json::from_slice(b"[]").unwrap();
    input["engine"]["hosts"]["listener"] = serde_json::from_slice(br#"[{"socket":"127.0.0.1:7200"}]"#).unwrap();
    let input = serde_json::to_vec(&input).unwrap();
    let dirs = config::Directories::working();
    let seeds = settings::Seeds {
        deployment: [0; 16],
        seed: 1,
        crypto: skein_lib::Secret::new(Box::new([0; 32])),
        link_secret: None,
        local_secret: None,
        installation: [1; 16],
    };
    let checked = startup::checked_start_with(
        &input,
        &dirs,
        &stock::files(),
        None,
        &seeds,
        List::with_capacity(0),
        None,
        &crate::stock::Given::Stock,
    )
    .unwrap();
    assert_eq!(checked.engine.jig.limits.hub.turn_bytes, 18_431_842);
    assert_eq!(checked.engine.jig.limits.core.tasks.message_bytes, 63_738);
}
