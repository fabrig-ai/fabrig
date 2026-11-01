//! The group writer and the setup reader share the entry form (protocol/setup.md, 8).
use crate::route::group_file;
use temper_forge_setup::{file, limits, problem::Problems, setup::Setup};

fn round_trip(source: &[u8], playground: bool) -> Box<[u8]> {
    let directories = jig_protocol_config::Directories::given(b"/config", b"/data", b"/cache").unwrap();
    let document = jig_protocol_config::document(
        source,
        &jig_protocol_config::DocumentLimits { bytes: file::BYTES, tokens: file::BYTES, depth: 32 },
    )
    .unwrap();
    let setup = Setup::read(&document, 0, &limits::small(), &directories, &mut Problems::empty()).unwrap();
    let entry = setup.entries[0].clone();
    let mut settings = super::fixtures::engine();
    settings.forges[0].number = entry.forge;
    settings.setup = Box::new([crate::settings::Setup { entry: entry.clone(), handover: None }]);
    let records = crate::startup::seed::seed(&[], &[], &settings).unwrap();
    let bytes = group_file::render(entry.forge, records[0].setup.as_ref().unwrap(), playground);
    let read = file::read(&bytes, entry.forge, &limits::small(), &directories).unwrap();
    assert_eq!(read.entry, entry);
    assert_eq!(read.playground, playground);
    bytes
}

#[test]
#[expect(clippy::disallowed_macros, reason = "the test varies the setup document fixture's binary spelling")]
fn an_owned_record_renders_an_entry_file_read_reads_back() {
    for binary in ["fetched", "/opt/forgejo"] {
        let source = std::format!(
            r#"[{{"forge":7,"start":"asked","directory":"/data/owned",
            "bot":{{"login":"temper","email":"temper@example.invalid"}},
            "mode":{{"owned":{{"forgejo":"{binary}",
                "runner":{{"binary":"{binary}","labels":["host:host"],"capacity":2,"path":"/usr/bin:/bin"}},
                "listen":{{"socket":"[::1]:3000"}},"person":{{"login":"person","email":"person@example.invalid"}},
                "mirror":{{"forgejo":"https://mirror.example.invalid:8443","runner":"https://runner.example.invalid"}},
                "public":"https://forge.example.invalid:9443"}}}}}}]"#
        );
        let bytes = round_trip(source.as_bytes(), false);
        assert!(skein_lib::bytes::find(&bytes, binary.as_bytes()).is_some());
    }
}

#[test]
fn a_fake_record_renders_with_playground_true() {
    round_trip(
        br#"[{"forge":8,"directory":"/data/playground","bot":{"login":"temper","email":"temper@example.invalid"},
        "mode":{"fake":{"listen":{"socket":"127.0.0.1:8369"},"git":"/data/bin/git","owner":"playground",
        "repositories":[{"name":"widget","landing":"main"},{"name":"other","landing":"dev"}]}}}]"#,
        true,
    );
}

#[test]
fn a_managed_record_renders_without_start() {
    for source in [
        br#"[{"forge":9,"bot":{"login":"temper","email":"temper@example.invalid"},"mode":"managed"}]"#.as_slice(),
        br#"[{"forge":9,"directory":"/data/managed","bot":{"login":"temper","email":"temper@example.invalid"},
        "mode":{"managed":{"admin_token":"/data/managed/admin"}},"orgs":["work","other"]}]"#,
    ] {
        let bytes = round_trip(source, false);
        assert!(skein_lib::bytes::find(&bytes, b"start").is_none());
    }
}
