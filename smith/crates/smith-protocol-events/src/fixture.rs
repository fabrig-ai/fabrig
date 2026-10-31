//! Named typed inputs shared by projection and composition tests.

use alloc::boxed::Box;
use smith_domain::llm;

/// An ordinary main completion with an empty input and a named affinity.
#[must_use]
pub fn prompt(thread: u32) -> llm::Prompt {
    llm::Prompt {
        effort: Some(b"medium".as_slice().into()),
        purpose: llm::Purpose::Ordinary { guard: None },
        endpoint: llm::Endpoint(0),
        affinity: llm::Affinity { key: [0; 16], thread },
        asker: llm::Asker::Main,
        model: b"model-7".as_slice().into(),
        system: Box::new([]),
        tools: smith_domain::tools::Grants { inspect: false, modify: false, shell: false },
        namespace: smith_domain::tools::Namespace::NoWorkspace,
        served: Box::new([]),
        choice: llm::ToolChoice::Only(Box::new([])),
        messages: Box::new([]),
        max_tokens: 7,
    }
}

/// A complete scalar destination with a one-second write deadline.
#[must_use]
pub fn event_limits() -> crate::EventLimits {
    crate::EventLimits {
        sinks: crate::sinks::fixture::small(),
        capture: crate::sinks::Capture::None,
        front: false,
        write_deadline: skein_lib::Duration::from_secs(1),
    }
}

/// A finite report run with declared model prices and a full input window.
#[must_use]
pub fn priced_charter() -> smith_domain::run::Charter {
    use skein_lib::Duration;
    use smith_domain::run;
    run::Charter {
        mode: run::Mode::Finite,
        reserve: run::charter::Reserve { turns: 0, time: Duration::ZERO },
        instructions: Box::default(),
        brief: run::Brief {
            sections: Box::new([run::Section { title: Box::default(), text: b"Review the change.".as_slice().into() }]),
        },
        conventions: None,
        resume: false,
        waiting: Duration::from_secs(300),
        grants: run::charter::Grants {
            wait: true,
            deliver: None,
            agents: false,
            host_tools: Box::default(),
            tools: run::charter::Tools { inspect: false, modify: false, shell: false },
        },
        outcome: run::outcome::OutcomeSpec {
            change: None,
            verdicts: Box::default(),
            failure: None,
            report: Some(run::outcome::TextSpec { max: 4096, fields: Box::default() }),
        },
        budget: run::Budget { turns: 512, spend: 25_000_000, time: Duration::from_secs(3600) },
        llm: run::charter::Llm {
            effort: None,
            prices: run::Prices {
                input: 1_250_000,
                cached: 125_000,
                cache_write: None,
                output: 10_000_000,
                unit: 1_000_000,
            },
            account: Some(0),
            endpoint: run::charter::Endpoint(1),
            model: b"gpt-5".as_slice().into(),
            window: 2_097_152,
            output: 65_536,
            dialect: 1,
        },
        models: Box::default(),
    }
}
