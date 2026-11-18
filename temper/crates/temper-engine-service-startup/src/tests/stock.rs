use jig_protocol_config as config;
pub(super) fn generated() -> Box<[u8]> {
    use core::num::NonZeroU16;
    let answer = temper_init::files(
        &temper_init::Facts {
            sign_in: false,
            binary: b"/bin/temper",
            git: Some(b"/bin/git"),
            claude: None,
            login: Some(b"ada"),
            ports: temper_init::Ports {
                clients: NonZeroU16::new(8367).unwrap(),
                webhooks: NonZeroU16::new(8371).unwrap(),
            },
        },
        &temper_init::Directories { config: b"/config", data: b"/data" },
        &[0; 32],
    )
    .unwrap();
    for file in answer.files {
        if file.name == b"temper.json" {
            let temper_init::Content::Bytes { bytes, .. } = file.content else { panic!("document bytes") };
            return bytes;
        }
    }
    panic!("init writes temper.json")
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
