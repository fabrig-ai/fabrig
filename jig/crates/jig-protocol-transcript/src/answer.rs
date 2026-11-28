//! Classify stored results by the smith shapes vocabulary (smith/docs/design/shapes.md, 3).

use smith_transcript::v8 as smith;

pub(super) fn word(returned: &smith::Returned) -> &'static [u8] {
    match returned {
        smith::Returned::Outcome(outcome) => owned(&outcome.value),
        smith::Returned::Said(_) => b"delegated",
        smith::Returned::Invalid(invalid) => match &invalid.problem {
            smith::CallProblem::UnknownTool => b"unknown_tool",
            smith::CallProblem::NotAnObject
            | smith::CallProblem::Missing(_)
            | smith::CallProblem::WrongType(_)
            | smith::CallProblem::BadValue(_)
            | smith::CallProblem::Nesting => b"malformed",
            smith::CallProblem::TooLarge | smith::CallProblem::Oversize(_) => b"call_too_large",
            smith::CallProblem::CutOff(_) => b"cut_off",
            smith::CallProblem::JobsFull(_) => b"jobs_full",
            smith::CallProblem::UnknownJob(_) => b"unknown_job",
        },
        smith::Returned::NotRun(not_run) => match &not_run.why {
            smith::NotRunWhy::Stopped => b"not_run_stopped",
            smith::NotRunWhy::OutsideChoice(_) => b"not_run_outside_choice",
            smith::NotRunWhy::AfterFailedWrite(_) => b"not_run_after_failure",
        },
        smith::Returned::Withdrawn => b"withdrawn",
    }
}

fn owned(outcome: &smith::Outcome) -> &'static [u8] {
    match outcome {
        smith::Outcome::Read(_)
        | smith::Outcome::Listed(_)
        | smith::Outcome::Found(smith::Found { timed_out: false, .. })
        | smith::Outcome::Written(_)
        | smith::Outcome::Edited(_)
        | smith::Outcome::Removed => b"ok",
        smith::Outcome::Found(smith::Found { timed_out: true, .. }) => b"search_timed_out",
        smith::Outcome::Running(_) => b"running",
        smith::Outcome::Exited(command) => exit(&command.exit),
        smith::Outcome::Job(job) => match &job.end {
            smith::JobEnd::Running => b"running",
            smith::JobEnd::Ended(ended) => exit(&ended.exit),
            smith::JobEnd::Stopped => b"stopped",
            smith::JobEnd::Cancelled => b"cancelled",
        },
        smith::Outcome::Interrupted(_) => b"interrupted",
        smith::Outcome::NotGranted => b"not_granted",
        smith::Outcome::TooLong => b"too_long",
        smith::Outcome::Outside(_) => b"outside",
        smith::Outcome::ReadOnly(_) => b"read_only",
        smith::Outcome::Protected => b"protected",
        smith::Outcome::NotFound(_) => b"not_found",
        smith::Outcome::NotFile => b"not_file",
        smith::Outcome::NotDirectory(_) => b"not_directory",
        smith::Outcome::Linked => b"linked",
        smith::Outcome::NotEmpty => b"not_empty",
        smith::Outcome::NulByte => b"nul_byte",
        smith::Outcome::TooLarge(_) => b"too_large",
        smith::Outcome::NotRead => b"not_read",
        smith::Outcome::Stale => b"stale",
        smith::Outcome::EditFailed(failure) => match &failure.why {
            smith::Mismatch::NoMatch => b"edit_no_match",
            smith::Mismatch::Ambiguous(_) => b"edit_ambiguous",
            smith::Mismatch::Unchanged => b"edit_unchanged",
        },
        smith::Outcome::Failed(_) => b"failed",
        smith::Outcome::TimedOut => b"timed_out",
        smith::Outcome::Cancelled => b"cancelled",
        smith::Outcome::Busy => b"busy",
    }
}

fn exit(exit: &smith::Exit) -> &'static [u8] {
    match exit {
        smith::Exit::Code(smith::ExitCode { code: 0 }) => b"exited",
        smith::Exit::Code(_) => b"exit_nonzero",
        smith::Exit::Signal(_) => b"signalled",
        smith::Exit::TimedOut => b"command_timed_out",
    }
}
