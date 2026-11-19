//! Lower owners of service composition behaviors (temper/docs/design/testing/real-loop.md).
pub mod github;
use serde_json::json;
use std::cell::Cell;
thread_local! { static ACTIVE: Cell<bool> = const { Cell::new(false) }; }
pub(crate) fn configure(value: &mut temper_playground::documents::Configuration) {
    if !ACTIVE.with(Cell::get) {
        return;
    }
    for (name, count) in [
        ("people.inbox_tasks", 16),
        ("tasks.tasks", 8),
        ("tasks.project_tasks", 8),
        ("tasks.tree_tasks", 8),
        ("tasks.batch", 4),
        ("authority.batch", 4),
        ("tasks.delegates", 5),
        ("people.operations", 6),
        ("documents.rows", 2),
        ("documents.lines", 3),
    ] {
        value.jig_over(name, count);
    }
    let policy = &mut value["world_settings"]["policies"][0]["policy"];
    policy["ceiling"]["budget"]["spend"] = 500.into();
    policy["roles"][0]["authority"]["budget"]["spend"] = 500.into();
    policy["period_spend"] = 1000.into();
    policy["roles"][0]["period_spend"] = 1000.into();
    let kinds = json!([{"kind":"charter","number":1},{"kind":"role","number":0}]);
    value["world_settings"]["policies"][0]["policy"]["ceiling"]["delegation"]["kinds"] = kinds.clone();
    value["world_settings"]["policies"][0]["policy"]["roles"][0]["authority"]["delegation"]["kinds"] = kinds;
}
