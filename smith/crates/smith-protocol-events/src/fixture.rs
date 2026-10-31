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
