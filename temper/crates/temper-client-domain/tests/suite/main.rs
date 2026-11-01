#[path = "../../../../testing/client_documents.rs"]
#[expect(dead_code, reason = "shared document fixtures also supply the other client suites")]
mod documents;

mod change;
mod task_pages;
