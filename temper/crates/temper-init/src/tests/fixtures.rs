use crate::{Content, Directories, Facts, File, Ports};
use core::num::NonZeroU16;
use serde_json::Value;

pub const OWNED: &[u8] = include_bytes!("../../../../docs/examples/owned.json");
pub const AGENT: &[u8] = include_bytes!("../../../../docs/examples/owned/agent.json");
pub const DIRECTORIES: Directories<'static> = Directories { config: b"/c/temper", data: b"/d/temper/" };

pub fn facts() -> Facts<'static> {
    Facts {
        binary: b"/opt/temper/bin/temper",
        git: Some(b"/usr/bin/git"),
        claude: None,
        login: Some(b"ada"),
        sign_in: true,
        ports: Ports {
            clients: NonZeroU16::new(8367).expect("the default port is nonzero"),
            owned: NonZeroU16::new(8368).expect("the owned forge port is nonzero"),
            forge: NonZeroU16::new(8369).expect("the fake forge port is nonzero"),
            llm: NonZeroU16::new(8370).expect("the default LLM port is nonzero"),
            webhooks: NonZeroU16::new(8371).expect("the default webhooks port is nonzero"),
        },
    }
}

pub fn cases() -> [(&'static str, Facts<'static>); 4] {
    [
        ("neither, default port", facts()),
        ("no git on PATH", Facts { git: None, ..facts() }),
        (
            "claude, kernel port",
            Facts {
                claude: Some(b"/opt/claude/bin/claude"),
                ports: Ports {
                    clients: NonZeroU16::new(41023).expect("the kernel-port fixture is nonzero"),
                    owned: NonZeroU16::new(41026).expect("the kernel owned forge port is nonzero"),
                    forge: NonZeroU16::new(41024).expect("the kernel forge port is nonzero"),
                    llm: NonZeroU16::new(65535).expect("the kernel LLM port is nonzero"),
                    webhooks: NonZeroU16::new(41025).expect("the kernel webhooks port is nonzero"),
                },
                ..facts()
            },
        ),
        ("claude, default port", Facts { claude: Some(b"/opt/claude/bin/claude"), ..facts() }),
    ]
}

pub fn file<'a>(files: &'a [File], name: &[u8]) -> &'a File {
    for file in files {
        if file.name == name {
            return file;
        }
    }
    unreachable!("the requested file is one init writes")
}

pub fn parse(bytes: &[u8]) -> Value {
    serde_json::from_slice(bytes).expect("a whole JSON document")
}

#[expect(clippy::disallowed_methods, reason = "tests inspect the JSON text at the protocol boundary")]
pub fn text(bytes: &[u8]) -> &str {
    core::str::from_utf8(bytes).expect("the test's paths are UTF-8")
}

pub fn content<'a>(answer: &'a crate::Answer, name: &[u8]) -> &'a [u8] {
    match &file(&answer.files, name).content {
        Content::Bytes { bytes, .. } => bytes,
        Content::Link { .. } => unreachable!("the requested file holds bytes"),
    }
}

pub fn bare_facts<'a>(facts: &Facts<'a>) -> crate::Bare<'a> {
    crate::Bare {
        binary: facts.binary,
        git: facts.git,
        claude: facts.claude,
        clients: facts.ports.clients,
        webhooks: facts.ports.webhooks,
    }
}

/// Retain the shipped profile with inert endpoints and the world's three fake model names.
pub fn agent_settings(mut document: Value) -> smith_agent_world::Settings {
    use serde_json::json;
    use smith_agent_world::{Job, Settings};
    // Destinations are inert: retain the shipped profile and model quantities,
    // but consult no programs directory, trust file or name service.
    document["programs"] = json!({});
    for endpoint in document["endpoints"].as_array_mut().expect("endpoints") {
        endpoint.as_object_mut().expect("endpoint").remove("program");
        endpoint["address"] = json!("127.0.0.1:9");
        endpoint["transport"] = json!("plaintext");
        endpoint.as_object_mut().expect("endpoint").remove("server_name");
    }
    let configuration = smith_agent_shell::config::from_value(document).expect("stock profile derives");
    let mut settings = Settings {
        limits: configuration.service.limits.domain,
        budget: configuration.service.limits.domain.run.budget,
        window: configuration.declarations.endpoints[0].limits.window,
        job: Job::Reporting,
        writable: false,
        network: skein_world::domain::Span::millis(1, 1),
        ..Settings::calm(144)
    };
    // The shared world installs one endpoint, independently of the shipped destinations.
    settings.limits.configured_endpoint_bytes =
        u64::try_from(size_of::<smith_domain::EndpointLimits>()).expect("endpoint cell");
    settings.provider.latency_min = skein_lib::Duration::from_millis(100);
    settings.provider.latency_max = settings.provider.latency_min;
    settings
}

/// A scripted provider tool call with whole JSON arguments.
pub fn call(name: &[u8], arguments: &Value) -> skein_fake_llm_domain::api::Line {
    skein_fake_llm_domain::api::Line::Call {
        name: name.into(),
        arguments: serde_json::to_vec(arguments).expect("call JSON").into(),
    }
}
