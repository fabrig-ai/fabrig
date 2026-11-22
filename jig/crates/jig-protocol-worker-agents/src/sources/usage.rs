//! Inspection and the usage child share the check's resources and deadline
//! (jig/docs/design/domain/hosts/README.md, 5.1).
use super::{Resources, binding_error, bounded, directory_name, reason};
use alloc::boxed::Box;
use jig_domain_host::sources;
use jig_domain_link::{FactWindow, SourceFailure, SourceName, UsageOutcome, UsageWindow};
use skein_io::{self as io, file, kernel::Fd};
use skein_lib::{List, Queue, Time, Token, Wall};
use smith_host_domain::{self as smith, ProgramUsage};
use smith_host_protocol::{SourceInspection, UsageRead};

#[derive(Debug)]
pub(super) struct Reading {
    read: u64,
    io_owner: Token,
    source: smith::SourceName,
    root: Fd,
    deadline: Time,
    asked: Wall,
    inspection: Option<SourceInspection>,
    child: Option<UsageRead>,
    cancelled: bool,
}
impl Reading {
    #[expect(clippy::too_many_arguments, reason = "the source binding, ask time and inspection queue cross together")]
    pub(super) fn new(
        read: u64,
        io_owner: Token,
        source: SourceName,
        root: Fd,
        deadline: Time,
        asked: Wall,
        resources: Option<&Resources>,
        files: &mut Queue<file::Request>,
    ) -> Result<Self, UsageOutcome> {
        let Some(resources) = resources else { return Err(unconfigured(b"no program configured")) };
        let Some(programs) = &resources.programs else { return Err(unconfigured(b"no program configured")) };
        let Some(parent) = resources.parent else {
            return Err(unconfigured(b"programs parent directory is unavailable"));
        };
        let Some(source) = smith::SourceName::new(source.as_bytes()) else {
            return Err(unconfigured(b"invalid source name"));
        };
        let inspection = SourceInspection::new(
            io_owner,
            parent,
            Box::from(directory_name(&programs.directory)),
            source,
            resources.user,
            deadline,
            files,
        );
        Ok(Self {
            read,
            io_owner,
            source,
            root,
            deadline,
            asked,
            inspection: Some(inspection),
            child: None,
            cancelled: false,
        })
    }
    pub(super) fn owns(&self, owner: Token) -> bool {
        match &self.child {
            Some(child) => child.owns(owner),
            None => self.io_owner == owner,
        }
    }
    pub(super) fn receive_io(&mut self, event: io::Event, below: &mut Queue<io::Request>) {
        self.child.as_mut().expect("spawned usage child").from_io(event, below);
    }
    pub(super) fn receive_file(&mut self, event: file::Event, files: &mut Queue<file::Request>) {
        self.inspection.as_mut().expect("active usage inspection").from_file(event, files);
    }
    pub(super) fn fire(&mut self, now: Time, below: &mut Queue<io::Request>) {
        if let Some(inspection) = &mut self.inspection {
            inspection.fire(now);
        }
        if let Some(child) = &mut self.child {
            child.fire(now, below);
        }
    }
    pub(super) fn next_deadline(&self) -> Option<Time> {
        match &self.inspection {
            Some(inspection) => inspection.next_deadline(),
            None => self.child.as_ref()?.next_deadline(),
        }
    }
    pub(super) fn cancel(&mut self, read: u64, below: &mut Queue<io::Request>) {
        if self.read != read {
            return;
        }
        self.cancelled = true;
        if let Some(inspection) = &mut self.inspection {
            // Inspection retains its file ownership until the pending request settles.
            inspection.fire(self.deadline);
        }
        if let Some(child) = &mut self.child {
            child.cancel(below);
        }
    }
    pub(super) fn advance(
        &mut self,
        resources: &Resources,
        detail: u32,
        below: &mut Queue<io::Request>,
    ) -> Option<(Token, sources::Answer)> {
        if let Some(inspection) = &mut self.inspection {
            let result = inspection.answer()?;
            self.inspection = None;
            if self.cancelled {
                return Some(self.answer(UsageOutcome::Failed { reason: SourceFailure::Closed, detail: Box::new([]) }));
            }
            match result {
                Ok(()) => {}
                Err(check) => {
                    let outcome = match check {
                        smith::ProgramCheck::NotSignedIn => UsageOutcome::NotSignedIn,
                        smith::ProgramCheck::Failed(failure) => UsageOutcome::Failed {
                            reason: reason(failure.reason),
                            detail: bounded(failure.detail, detail),
                        },
                        smith::ProgramCheck::Usable { .. } | smith::ProgramCheck::SignedInOtherwise(_) => {
                            unreachable!("inspection only reports absent or failed directories")
                        }
                    };
                    return Some(self.answer(outcome));
                }
            }
        }
        if self.child.is_none() {
            match UsageRead::new(
                self.io_owner,
                resources.programs.as_ref().expect("configured usage program"),
                self.source,
                &resources.environment,
                self.root,
                Box::from(b".".as_slice()),
                self.deadline,
                self.asked,
                detail,
                below,
            ) {
                Ok(child) => self.child = Some(child),
                Err(error) => return Some(self.answer(super::bound_usage(unconfigured(binding_error(error)), detail))),
            }
        }
        let outcome = translate(self.child.as_mut()?.answer()?, detail);
        Some(self.answer(outcome))
    }
    fn answer(&self, outcome: UsageOutcome) -> (Token, sources::Answer) {
        (Token::new(self.read), sources::Answer::Usage(outcome))
    }
}
fn unconfigured(detail: &[u8]) -> UsageOutcome {
    UsageOutcome::Failed { reason: SourceFailure::Unconfigured, detail: Box::from(detail) }
}
pub(super) fn translate(answer: ProgramUsage, bound: u32) -> UsageOutcome {
    match answer {
        ProgramUsage::Read(windows) => {
            let mut translated = List::with_capacity(u32::try_from(windows.len()).expect("bounded usage windows"));
            for value in windows {
                translated.push(window(value)).expect("usage window capacity");
            }
            UsageOutcome::Read(translated.into_boxed())
        }
        ProgramUsage::NotSignedIn => UsageOutcome::NotSignedIn,
        ProgramUsage::Spent => UsageOutcome::Spent,
        ProgramUsage::Failed(failure) => {
            UsageOutcome::Failed { reason: reason(failure.reason), detail: bounded(failure.detail, bound) }
        }
    }
}
/// Map smith's allowance state to jig's registry without changing its observations.
#[must_use]
fn window(value: smith::UsageWindow) -> UsageWindow {
    let state = match value.state {
        smith::UsageState::Open => jig_domain_link::WINDOW_OPEN,
        smith::UsageState::Warned => jig_domain_link::WINDOW_WARNED,
        smith::UsageState::Limited => jig_domain_link::WINDOW_LIMITED,
    };
    UsageWindow {
        label: value.label,
        window: FactWindow { length: value.length, used: value.used, resets: value.resets, state },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn usage_failures_keep_the_source_reason_and_bound_the_detail() {
        for (failure, expected) in [
            (smith::FailureReason::Missing, SourceFailure::Missing),
            (smith::FailureReason::Deadline, SourceFailure::Deadline),
            (smith::FailureReason::Unreadable, SourceFailure::Unreadable),
            (smith::FailureReason::Exited, SourceFailure::Exited),
        ] {
            assert_eq!(
                translate(
                    ProgramUsage::Failed(smith::ProgramFailure {
                        reason: failure,
                        detail: b"abcdef".as_slice().into()
                    }),
                    4
                ),
                UsageOutcome::Failed { reason: expected, detail: b"abcd".as_slice().into() }
            );
        }
    }
    #[test]
    fn a_warned_usage_window_keeps_its_reset_and_missing_observations() {
        let reset = Some(Wall::from_nanos(123));
        assert_eq!(
            translate(
                ProgramUsage::Read(Box::new([smith::UsageWindow {
                    label: b"seven_day".as_slice().into(),
                    length: None,
                    used: None,
                    resets: reset,
                    state: smith::UsageState::Warned,
                }])),
                4
            ),
            UsageOutcome::Read(Box::new([UsageWindow {
                label: b"seven_day".as_slice().into(),
                window: FactWindow { length: None, used: None, resets: reset, state: jig_domain_link::WINDOW_WARNED },
            }]))
        );
    }
}
