//! Binary defaults and the box's constant (protocol/configuration/providers.md, 2 and 4).
use alloc::boxed::Box;
use jig_protocol_config::{self as config, ProviderSetting};
use skein_lib::{Duration, bytes};

/// The stock's `OpenAI` and Anthropic providers, in jig's list form.
#[must_use]
#[expect(clippy::disallowed_macros, reason = "the stock providers are embedded in the binary")]
pub fn providers() -> Box<[ProviderSetting]> {
    let document = config::document(
        include_bytes!("providers.json"),
        &config::DocumentLimits { bytes: 16_384, tokens: 2048, depth: 16 },
    )
    .expect("the stock provider document is bounded JSON");
    let mut problems = config::Problems::empty();
    let values = config::llm::providers_setting(&document, 0, &config::Path::root(), &mut problems)
        .expect("the stock providers read");
    assert!(problems.is_empty(), "the stock providers are checked");
    values
}

/// The box's immutable provider at the listener chosen before its worker is built.
#[must_use]
pub fn scripted(address: core::net::SocketAddr) -> ProviderSetting {
    ProviderSetting {
        provider: config::Provider {
            name: bytes::copy_of(b"demo"),
            number: 3,
            endpoint: Some(bytes::copy_of(b"demo")),
            direct: None,
            list: None,
            signed_in_plan: false,
            manage_usage: None,
            sign_in: None,
            keys: true,
            programs: Box::new([]),
        },
        endpoints: Box::new([config::Endpoint {
            name: bytes::copy_of(b"demo"),
            number: 3,
            dialect: 3,
            provider: bytes::copy_of(b"demo"),
            program: None,
            address: config::EndpointAddress::Socket(address),
            transport: config::Transport::Plaintext,
            headers: Box::new([]),
            connect: Duration::from_millis(10_000),
            handshake: Duration::from_millis(10_000),
        }]),
        withdrawn: false,
    }
}
