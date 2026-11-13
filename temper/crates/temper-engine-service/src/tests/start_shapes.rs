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
