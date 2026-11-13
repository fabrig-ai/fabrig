use super::fixtures;
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
                owned: NonZeroU16::new(8368).unwrap(),
                forge: NonZeroU16::new(8369).unwrap(),
                llm: NonZeroU16::new(8370).unwrap(),
                webhooks: NonZeroU16::new(8371).unwrap(),
            },
        },
        &temper_init::Directories { config: b"/config", data: b"/data" },
        &[0; 32],
        &[1; 32],
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
    fixtures::parse_engine(&fixtures::value(document, b"engine"))
}
