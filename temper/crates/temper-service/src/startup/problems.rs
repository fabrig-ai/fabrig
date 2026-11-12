//! Joined startup problems (protocol/services.md, 6).
pub use temper_engine_service::startup::{Problem, Problems, What, lines};
pub(super) fn from_worker(found: temper_service_core::startup::problems::SharedProblems) -> Problems {
    let mut result = Problems::empty();
    for problem in found.problems.into_boxed() {
        result.push_file(problem.path, problem.what.extend(), problem.file);
    }
    result.past = result.past.saturating_add(found.past);
    result
}
