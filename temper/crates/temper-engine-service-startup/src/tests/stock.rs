use jig_protocol_config as config;
pub(super) fn generated() -> Box<[u8]> {
    temper_config_shell::init_document(false)
}
pub(super) fn engine(document: &[u8]) -> config::Engine {
    let parsed =
        config::document(document, &config::DocumentLimits { bytes: 1_048_576, tokens: 16_384, depth: 32 }).unwrap();
    crate::startup::engine(&parsed, parsed.member(0, b"engine").unwrap(), &config::Directories::working()).unwrap()
}

/// Files named by the generated init document, with fixture digest and secret bytes.
/// Complete-start checks consume these directly rather than parsing a second time
/// just to discover the fixture's paths.
pub(super) fn files() -> crate::settings::Files {
    let mut entries = skein_lib::List::with_capacity(3);
    for path in [b"worker.digest".as_slice(), b"worker.secret", b"/config/agent.json"] {
        entries.push((path.into(), skein_lib::Secret::new(Box::new([b'0'; 64])))).unwrap();
    }
    crate::settings::Files { entries }
}
