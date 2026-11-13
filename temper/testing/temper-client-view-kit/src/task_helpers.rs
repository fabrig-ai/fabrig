use super::task as f;
use temper_client_domain::Domain;

/// A held agent task keeps the cards shared by the other task pages (5.11).
#[must_use]
pub fn held_agent_root() -> Domain {
    let root = f::held_root();
    let mut task = root.pages().task().expect("task page").document(root.client()).expect("task").clone();
    task.priority = None;
    f::supplied_root(task, true)
}
