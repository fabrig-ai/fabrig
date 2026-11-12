//! Small local receiving limits and typed chat setup (domain/host.md, sections 6 and 8).
//! The caller supplies the native agent profile; the preset owns only local policy bounds.
//! Shared fixtures (smith/docs/design/testing.md, section 7).

#![forbid(unsafe_code)]

extern crate alloc;

use alloc::boxed::Box;
use skein_lib::Duration;
use smith_domain::run::charter::Endpoint;
use smith_local_domain::Limits;

/// Small local limits with one inline child using the supplied native receiving profile.
#[must_use]
pub fn small(smith: smith_domain::Limits) -> Limits {
    Limits {
        smith,
        agent: smith_local_domain::AgentLimits::inline(smith, 2, Duration::from_secs(60), Duration::from_secs(5))
            .expect("bounded inline child"),
        endpoints: Box::new([Endpoint(4)]),
        chat_bytes: 64,
        text_bytes: 1024,
        models: 2,
        line_bytes: 256,
        show_bytes: 1024,
        unsaved: 2,
        facts: 16,
    }
}

/// One first turn with a native opener and no messages, usage or spend.
#[must_use]
pub fn turn() -> smith_domain::Turn {
    smith_domain::Turn {
        model: b"model".as_slice().into(),
        effort: None,
        kind: smith_domain::session::record::Kind::First(Box::new(smith_domain::session::record::Opener {
            system: None,
            affinity: smith_domain::llm::Affinity { key: [0; 16], thread: 0 },
            account: None,
            backend: smith_domain::session::record::Backend::Direct,
        })),
        version: smith_domain::session::record::VERSION,
        endpoint: Some(smith_domain::session::llm::Endpoint(4)),
        dialect: 1,
        sequence: 1,
        usage: smith_domain::session::llm::Usage::ZERO,
        spent: 0,
        jobs: 0,
        messages: Box::default(),
    }
}

/// A selected empty chat with no transcript or delivery records.
#[must_use]
pub fn loaded() -> smith_local_domain::Event {
    smith_local_domain::Event::Loaded { state: None, transcript: None, deliveries: Box::new([]) }
}
