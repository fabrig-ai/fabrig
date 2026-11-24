use hearth_builder_world::fixtures;
use skein_io::kernel::Exit;
use std::os::unix::fs::PermissionsExt;

#[test]
fn test_mins_closure_is_recorded_as_mmdebstraps_recording_resolved_it() {
    let (repository, rendered) = fixtures::resolved_test_min("record-closure");
    let record = fixtures::read_test_min(&repository);
    assert_eq!(record.closure, fixtures::recorded_closure());
    assert_eq!(
        record
            .sources
            .iter()
            .map(|line| (line.uri.as_ref(), line.suite.as_ref(), line.components.as_ref()))
            .collect::<Vec<_>>(),
        [(
            b"http://snapshot.debian.org/archive/debian/20261017T143929Z/".as_slice(),
            b"forky".as_slice(),
            [b"main".as_slice().into()].as_slice()
        )]
    );
    let text = String::from_utf8_lossy(&rendered.stdout);
    assert!(text.lines().any(|line| line.starts_with("packages ") && line.ends_with(" known")), "{text}");

    let digests = fixtures::mirror_indices();
    assert_eq!(
        record
            .sources
            .iter()
            .flat_map(|line| line.indices.iter())
            .map(|index| (index.path.as_ref(), index.sha256.0))
            .collect::<Vec<_>>(),
        digests
    );
    for (_, digest) in digests {
        let (path, record) = fixtures::cached_index(&repository, digest);
        assert_eq!(skein_lib::sha256::digest(&std::fs::read(&path).expect("kept index")), digest);
        assert_eq!(std::fs::metadata(&path).expect("index mode").permissions().mode() & 0o777, 0o644);
        assert_eq!(record.sha256.0, digest);
        assert_eq!(
            record.origin,
            hearth_protocol_tree::cache_file::Origin::Index { resolution: fixtures::TEST_MIN_RESOLUTION }
        );
    }
    assert_eq!(rendered.exit, Exit::Code(0));
}
