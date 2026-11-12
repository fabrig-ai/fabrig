use crate::startup::{Shape, nearest};

#[test]
fn each_document_shape_is_read_from_its_members() {
    for (document, expected) in [
        (br#"{"engine":{"hosts":{"listener":[{"in_process":"workers"}]}}}"#.as_slice(), Shape::Standalone),
        (br#"{"engine":{"hosts":{"listener":[{"socket":"127.0.0.1:0"}]}}}"#.as_slice(), Shape::EngineAlone),
        (br#"{"workers":[{"engine":{"socket":"127.0.0.1:7100"}}]}"#.as_slice(), Shape::WorkerAlone),
    ] {
        assert_eq!(nearest(document), Some(expected));
    }
    for (document, expected) in [
        (br#"{"workers":[{}],"application":null}"#.as_slice(), Some(Shape::WorkerAlone)),
        (b"[]".as_slice(), None),
        (b"not json".as_slice(), None),
    ] {
        assert_eq!(nearest(document), expected);
    }
}
