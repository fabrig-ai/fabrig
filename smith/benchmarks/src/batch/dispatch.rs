//! A settled attempt's result and owned files (smith/docs/design/benchmarks/README.md, 5.3, and smith/docs/design/benchmarks/isolation.md, 1).

use std::path::Path;

use crate::{Artifact, AttemptResult, End, Fault, format, grade::Snapshot, process::ProcessResult, run::Request};

/// Files consulted only after an attempt's process tree has settled.
pub trait DispatchFiles {
    fn read_result(&self, file: &Path) -> Result<AttemptResult, Fault>;
    fn exists(&self, path: &Path) -> bool;
    fn snapshot(&mut self, directory: &Path) -> Result<Snapshot, Fault>;
    fn archive(&mut self, directory: &Path, name: &str, bytes: &[u8]) -> Result<Artifact, Fault>;
    fn remove_scratch(&mut self, path: &Path) -> Result<(), Fault>;
    fn replace_result(&mut self, file: &Path, result: &AttemptResult) -> Result<(), Fault>;
}

/// Recover the archive and remove owned scratch after settlement.
pub fn dispatch_result(
    files: &mut impl DispatchFiles,
    state: &Path,
    request: &Request,
    process: &ProcessResult,
    existed: bool,
) -> Result<AttemptResult, Fault> {
    let scratch = crate::run::scratch_root(state, request)?;
    let directory = state.join("runs").join(&request.identity.run).join(&request.identity.attempt);
    let result_file = directory.join("result.json");
    let mut result = match files.read_result(&result_file) {
        Ok(result) => result,
        Err(error) if !files.exists(&result_file) => crate::run::recover(
            request,
            process,
            format!("attempt ended {:?} without a result: {error}", process.exit),
            files.snapshot(&directory)?,
            |name, bytes| files.archive(&directory, name, bytes),
        )?,
        Err(error) => return Err(format(format!("attempt's existing result is invalid: {error}"))),
    };
    if !existed
        && files.exists(&scratch)
        && let Err(error) = files.remove_scratch(&scratch)
    {
        result.outcome.end = End::HarnessError { what: format!("dispatcher workspace deletion failed: {error}") };
        files.replace_result(&result_file, &result)?;
    }
    Ok(result)
}
