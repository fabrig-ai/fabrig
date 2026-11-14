//! Negative controls for the session's append-only window contract (smith/docs/design/domain/session.md, section 10).

use smith_session_world::bytes;

use smith_domain_session::llm::{Block, Endpoint, Message, Prompt, Role};
use smith_session_world::check_prompt_extends;

fn prompt() -> Prompt {
    Prompt {
        effort: None,
        purpose: smith_domain_session::llm::Purpose::Ordinary { guard: None },
        choice: smith_domain_session::llm::ToolChoice::Auto,
        protocol_allowance: 0,
        namespace: smith_domain_tools::Namespace::One,
        affinity: smith_domain_session::llm::Affinity { key: [0; 16], thread: 0 },
        endpoint: Endpoint(0),
        model: bytes(b"model"),
        system: bytes(b"system"),
        tools: smith_domain_tools::Grants { inspect: true, modify: false, shell: false },
        delegated: Box::new([]),
        messages: Box::new([Message {
            role: Role::User,
            content: Box::new([Block::Text { text: bytes(b"first"), replay: None }]),
        }]),
        max_tokens: 1,
    }
}

#[test]
fn changed_window_system_is_rejected() {
    let mut changed = prompt();
    changed.system = bytes(b"different");
    assert_eq!(check_prompt_extends(&prompt(), &changed), Err("a window keeps its system text".into()));
}

#[test]
fn edited_window_history_is_rejected() {
    let mut changed = prompt();
    changed.messages[0].content = Box::new([]);
    assert_eq!(check_prompt_extends(&prompt(), &changed), Err("a window extends its history".into()));
}

#[test]
fn changed_window_affinity_is_rejected() {
    let previous = prompt();
    let mut changed = prompt();
    changed.affinity.thread = 1;
    assert_eq!(check_prompt_extends(&previous, &changed), Err("a window keeps its affinity".into()));
}

#[test]
fn a_window_boundary_without_a_compaction_is_rejected() {
    let previous = prompt();
    let mut changed = prompt();
    changed.messages[0].content = Box::new([Block::Text { text: bytes(b"summary"), replay: None }]);
    assert_eq!(
        smith_session_world::check_prompt_boundary(&previous, &changed, None),
        Err("a window extends its history".into()),
    );
}

#[test]
fn a_prompt_past_its_window_is_rejected() {
    assert_eq!(
        smith_session_world::check_prompt_bound(1_001, 1_000),
        Err("no prompt whose bound exceeds its window".into()),
    );
}
