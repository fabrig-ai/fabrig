//! Small local receiving limits and typed chat setup (domain/host.md, sections 6 and 8).
//! The caller supplies the native agent profile; the preset owns only local policy bounds.
use crate::{Config, Contract, Limits};
use alloc::boxed::Box;
use skein_lib::Duration;
use smith_domain::run::{self, charter::Endpoint, outcome};

/// Small local limits with one inline child using the supplied native receiving profile.
#[must_use]
pub fn small(smith: smith_domain::Limits) -> Limits {
    Limits {
        smith,
        agent: crate::AgentLimits::inline(smith, 2, Duration::from_secs(60), Duration::from_secs(5))
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

/// A chat with one priced model and one requested account.
#[must_use]
pub fn config() -> Config {
    Config {
        reserve: run::charter::Reserve { turns: 0, time: Duration::ZERO },
        chat: Box::from(&b"main"[..]),
        instructions: Box::from(&b"Assist the person"[..]),
        brief: run::charter::Brief {
            sections: Box::new([run::charter::Section {
                title: Box::from(&b"Context"[..]),
                text: Box::from(&b"Work carefully"[..]),
            }]),
        },
        protocol_allowances: Box::new([0]),
        progress: Box::new([smith_domain::session::Progress {
            head: Duration::from_secs(60),
            idle: Duration::from_secs(30),
        }]),
        efforts: Box::new([Box::new([])]),
        models: Box::new([run::charter::Llm {
            effort: None,
            prices: run::Prices { input: 1, cached: 1, cache_write: None, output: 1, unit: 1_000_000 },
            dialect: 1,
            account: Some(7),
            endpoint: Endpoint(4),
            model: Box::from(&b"small"[..]),
            window: 8192,
            output: 128,
        }]),
        sub_agents: false,
        budget: run::Budget { turns: 8, spend: 1, time: Duration::from_secs(60) },
        conventions: Some(run::Conventions { guide: Box::from(&b"GUIDE.md"[..]), checks: Box::from(&b"check"[..]) }),
        contract: Contract::Report(outcome::TextSpec { max: 256, fields: Box::new([]) }),
        deliver: None,
        title_field: Box::from(&b"title"[..]),
        live_text: false,
        mode: crate::Mode::Chat { waiting: Duration::from_secs(30) },
        resume: true,
        accounts: Box::new([7]),
        workspace: None,
        push: None,
    }
}

/// One first turn with a native opener and no messages, usage or spend.
#[must_use]
pub fn turn() -> smith_domain::Turn {
    smith_domain::Turn {
        model: b"model".as_slice().into(),
        effort: None,
        kind: smith_domain::session::record::Kind::First(Box::new(smith_domain::session::record::Opener {
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
