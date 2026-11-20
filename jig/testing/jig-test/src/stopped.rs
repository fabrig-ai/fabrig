//! One safe terminal line, after the shell drains (protocol/service.md, 7).
use jig_domain_core::start;
use jig_protocol_store::{Check, OpenRefusal, restore};
use jig_test_service::{End, Stop};
use std::io::ErrorKind;

pub(crate) fn line(end: Option<&End>, output_failed: Option<ErrorKind>) -> Option<String> {
    let reason = match end {
        Some(End::Stopped(stop)) => Some(reason(stop)),
        Some(End::Drained | End::Aborted) => None,
        None => Some(String::from("no service end reason")),
    };
    let reason = match (reason, output_failed) {
        (Some(reason), Some(error)) => format!("{reason}; output_failed: {error:?}"),
        (Some(reason), None) => reason,
        (None, Some(error)) => format!("output_failed: {error:?}"),
        (None, None) => return None,
    };
    Some(format!("jig-test: stopped: {reason}"))
}

fn reason(stop: &Stop) -> String {
    match stop {
        Stop::CommitFailed { number } => format!("CommitFailed: commit {number}"),
        // Paths can contain application-supplied names. The diagnostic kinds
        // carry only static words and numbers, not configuration values.
        Stop::Start(problems) => {
            let kinds: Vec<_> = problems.problems.iter().map(|problem| problem.what).collect();
            format!("Start: {kinds:?}, past={}; paths omitted", problems.past)
        }
        Stop::Listener { component, error } => {
            let text = match component {
                Some(component) => format!("Listener: an io event rejected by {component}"),
                None => String::from("Listener: an io event no component owns"),
            };
            match error {
                Some(error) => format!("{text}; io error: {error:?}"),
                None => text,
            }
        }
        Stop::StoreOpen { phase, failure } => format!("StoreOpen: phase={phase:?}; {}", open_failure(failure)),
        Stop::StorePage { phase, failure } => format!("StorePage: phase={phase:?}; {}", page_failure(failure)),
        Stop::StartCore { phase, failure } => format!("StartCore: phase={phase:?}; {}", core_failure(failure)),
    }
}

fn row(site: &restore::RowSite) -> String {
    format!("family={:?}, key_bytes={}, truncated={}; key omitted", site.family, site.key_bytes, site.truncated)
}

fn page_failure(failure: &restore::Failure) -> String {
    let cause = match &failure.cause {
        restore::Cause::Join { target, failure } => format!("Join: {failure:?}; target: {}", row(target)),
        cause @ (restore::Cause::Phase { .. }
        | restore::Cause::Range(_)
        | restore::Cause::Walk(_)
        | restore::Cause::Decode(_)
        | restore::Cause::History { .. }
        | restore::Cause::RowPastPage { .. }
        | restore::Cause::Arithmetic(_)) => format!("{cause:?}"),
    };
    match &failure.row {
        Some(site) => format!("{cause}; row: {}", row(site)),
        None => cause,
    }
}

fn open_failure(failure: &restore::OpenFailure) -> String {
    match failure {
        restore::OpenFailure::Missing(site) => format!("Missing: {}", row(site)),
        restore::OpenFailure::Data(failure) => format!("Data: {}", page_failure(failure)),
        restore::OpenFailure::Refused(refusal) => {
            let text = match refusal {
                OpenRefusal::Job { owner, .. } => format!("Job: owner={owner:?}; job, key and words omitted"),
                OpenRefusal::Check { why: Check::Load { key } } => format!("Check: Load; key_bytes={}", key.len()),
                OpenRefusal::Check { why: Check::Private { .. } } => String::from("Check: Private; name omitted"),
                OpenRefusal::Layout { value_type, id, values, why, .. } => {
                    format!("Layout: value_type={value_type}, id={id}, values={values:?}, why={why:?}; family omitted")
                }
                refusal @ (OpenRefusal::Unready { .. }
                | OpenRefusal::Format { .. }
                | OpenRefusal::Deployment
                | OpenRefusal::Numbers { .. }
                | OpenRefusal::Failed
                | OpenRefusal::OlderBuild { .. }
                | OpenRefusal::Generations { .. }
                | OpenRefusal::Ahead { .. }
                | OpenRefusal::Set { .. }) => format!("{refusal:?}"),
            };
            format!("Refused: {text}")
        }
    }
}

// Logical keys can include a note's resource pattern. Never debug those bytes.
fn key(key: &jig_domain_core::Key) -> String {
    match key {
        jig_domain_core::Key::Notes(_) => String::from("Notes (key omitted)"),
        key @ (jig_domain_core::Key::Core(_)
        | jig_domain_core::Key::Tasks(_)
        | jig_domain_core::Key::People(_)
        | jig_domain_core::Key::Accounts(_)) => format!("{key:?}"),
    }
}

fn core_failure(failure: &start::Failure) -> String {
    match failure {
        start::Failure::LiveSet { set, key: value, held, limit } => {
            format!("LiveSet: set={set:?}, key={}, held={held}, limit={limit}", key(value))
        }
        start::Failure::Accounting { key: value } => format!("Accounting: key={}", key(value)),
        start::Failure::LlmChoice { key: value } => format!("LlmChoice: key={}", key(value)),
        start::Failure::Link { record, linked } => {
            let linked = match linked {
                start::Linked::Record(value) => key(value),
                start::Linked::Holder(_) => String::from("Holder (resource omitted)"),
                linked @ (start::Linked::Charter { .. }
                | start::Linked::Project(_)
                | start::Linked::Connector(_)
                | start::Linked::SettingsProposal { .. }) => format!("{linked:?}"),
            };
            format!("Link: record={}, linked={linked}", key(record))
        }
        failure @ (start::Failure::Numbers { .. }
        | start::Failure::Taken { .. }
        | start::Failure::Settings { .. }
        | start::Failure::Application { .. }
        | start::Failure::ProjectPolicy { .. }
        | start::Failure::HeldPastLimit { .. }
        | start::Failure::NeedsPastLimit { .. }
        | start::Failure::AccountsPastLimit { .. }) => format!("{failure:?}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jig_protocol_config::{Path, Problems, What};

    #[test]
    fn every_stop_has_one_safe_terminal_line() {
        let mut problems = Problems::empty();
        problems.push(Path::root().member(b"private-configuration-value"), What::Missing);
        let cases = [
            (Stop::CommitFailed { number: 17 }, "CommitFailed: commit 17"),
            (Stop::Start(problems), "Start: [Missing], past=0; paths omitted"),
            (Stop::Listener { component: None, error: None }, "Listener: an io event no component owns"),
            (
                Stop::Listener { component: Some("clients"), error: Some(skein_io::Error::Other) },
                "Listener: an io event rejected by clients; io error: Other",
            ),
            (Stop::Listener { component: Some("workers"), error: None }, "Listener: an io event rejected by workers"),
            (
                Stop::StoreOpen {
                    phase: Some(start::Phase::LoadCore),
                    failure: restore::OpenFailure::Refused(OpenRefusal::Failed),
                },
                "StoreOpen: phase=Some(LoadCore); Refused: Failed",
            ),
            (
                Stop::StorePage {
                    phase: None,
                    failure: restore::Failure {
                        cause: restore::Cause::Decode(restore::DecodeFailure::Value),
                        row: Some(restore::RowSite {
                            family: Some(jig_protocol_store::Family::Accounts),
                            key: b"private-record-key".as_slice().into(),
                            key_bytes: 18,
                            truncated: false,
                        }),
                    },
                },
                "StorePage: phase=None; Decode(Value); row: family=Some(Accounts), key_bytes=18, truncated=false; key omitted",
            ),
            (
                Stop::StartCore {
                    phase: Some(start::Phase::Link),
                    failure: start::Failure::ProjectPolicy { project: 3 },
                },
                "StartCore: phase=Some(Link); ProjectPolicy { project: 3 }",
            ),
        ];
        for (stop, reason) in cases {
            assert_eq!(line(Some(&End::Stopped(stop)), None), Some(format!("jig-test: stopped: {reason}")));
        }
        assert_eq!(line(Some(&End::Drained), None), None);
        assert_eq!(line(Some(&End::Aborted), None), None);
        assert_eq!(
            line(Some(&End::Drained), Some(ErrorKind::BrokenPipe)),
            Some(String::from("jig-test: stopped: output_failed: BrokenPipe"))
        );
        assert_eq!(
            line(Some(&End::Stopped(Stop::CommitFailed { number: 2 })), Some(ErrorKind::BrokenPipe)),
            Some(String::from("jig-test: stopped: CommitFailed: commit 2; output_failed: BrokenPipe"))
        );
    }
}
