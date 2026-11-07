//! The conversation header says the public helper's facts with existing phrases.
use super::{Words, name};
use crate::words::phrases::jig::JigPhrase as P;
use jig_client::words::{StatusWord, Word};
use jig_client::{
    contract::{RetryNotice, WaitedFor, Whom},
    facts::ChatStatus,
};
use skein_lib::{Duration, Wall};
use skein_ui::{Builder, phrases::Hole};
impl Words {
    pub(crate) fn chat_status(&self, b: &mut Builder, status: &ChatStatus, wall: Wall) {
        match status {
            ChatStatus::HostRefused(why) => {
                self.text(b, P::Word(Word::Status(StatusWord::HostRefused)), &[]);
                self.host_refused(b, *why);
            }
            ChatStatus::Starting => self.text(b, P::Word(Word::Status(StatusWord::Starting)), &[]),
            ChatStatus::Writing => self.text(b, P::Word(Word::Status(StatusWord::Writing)), &[]),
            ChatStatus::Tool { tool, since } => {
                self.text(b, P::Word(Word::Status(StatusWord::Tool)), &[name(tool)]);
                self.status_span(b, *since, wall);
            }
            ChatStatus::Working { since } => {
                self.text(b, P::Word(Word::status(status)), &[]);
                self.status_span(b, *since, wall);
            }
            ChatStatus::WaitingForYou { kind, entries } => {
                self.text(b, P::Word(Word::Status(StatusWord::WaitingForYou)), &[]);
                let phrase = match kind {
                    WaitedFor::Question => P::Question,
                    WaitedFor::Proposal => P::Proposal,
                    WaitedFor::Escalation => P::Escalation,
                    WaitedFor::PersonTask => P::PersonTask,
                };
                self.text(b, phrase, &[]);
                self.text(b, P::Needs, &[Hole::Number(u64::from(*entries))]);
            }
            ChatStatus::WaitingFor(whom) => match whom {
                Whom::Person(party) => self.text(b, P::WaitingParty, &[Hole::Number(*party)]),
                Whom::Role { project, role } => {
                    self.text(b, P::WaitingRole, &[Hole::Number(u64::from(*project)), Hole::Number(u64::from(*role))]);
                }
            },
            ChatStatus::Parked => self.text(b, P::Word(Word::Status(StatusWord::Parked)), &[]),
            ChatStatus::Retrying { until, notice } => {
                self.text(b, P::Word(Word::status(status)), &[]);
                self.wall(b, *until);
                if let Some(notice) = notice {
                    match notice {
                        RetryNotice::OwnerFailed { class, count, allowance, reason } => {
                            crate::tree::fragments::chip::hold_failure(b, *class, self);
                            self.text(
                                b,
                                P::RunRetryFailed,
                                &[Hole::Number(u64::from(*count)), Hole::Number(u64::from(*allowance))],
                            );
                            crate::tree::fragments::chip::retry_reason(b, reason.as_ref(), self);
                        }
                        RetryNotice::Failed { class, count, allowance, reason } => {
                            crate::tree::cards::chip::failure(b, *class, self);
                            self.text(
                                b,
                                P::RunRetryFailed,
                                &[Hole::Number(u64::from(*count)), Hole::Number(u64::from(*allowance))],
                            );
                            crate::tree::fragments::chip::retry_reason(b, reason.as_ref(), self);
                        }
                        RetryNotice::Refused { count, reason } => {
                            self.text(b, P::RunRetryRefused, &[Hole::Number(u64::from(*count))]);
                            crate::tree::fragments::chip::retry_reason(b, reason.as_ref(), self);
                        }
                    }
                }
            }
            ChatStatus::Held(reason) => self.hold_category(b, *reason),
            ChatStatus::RunWaiting(_) | ChatStatus::Closing => self.text(b, P::Word(Word::status(status)), &[]),
            ChatStatus::Ended { at } => {
                self.text(b, P::Word(Word::status(status)), &[]);
                self.wall(b, *at);
            }
        }
    }
    fn host_refused(&self, b: &mut Builder, why: jig_client::contract::TaskWaitRefused) {
        use jig_client::contract::{HostKind, TaskWaitRefused as R};
        match why {
            R::Hello { kind, at, why, .. } => {
                let phrase = match kind {
                    HostKind::Engine => P::RefusedEngineAt,
                    HostKind::Worker => P::RefusedWorkerAt,
                };
                self.text(b, phrase, &[Hole::Wall(self.adjusted_wall(at))]);
                crate::tree::fragments::deployment::hello_refusal(b, self, why);
            }
            R::Bounds { kind, why, .. } => {
                let phrase = match kind {
                    HostKind::Engine => P::BoundsEngine,
                    HostKind::Worker => P::BoundsWorker,
                };
                self.text(b, phrase, &[]);
                if let Some(why) = why {
                    crate::tree::fragments::deployment::hello_refusal(
                        b,
                        self,
                        jig_client::contract::HelloRefusal::Agent { why },
                    );
                }
            }
        }
    }
    fn status_span(&self, b: &mut Builder, since: Wall, wall: Wall) {
        self.text(
            b,
            P::ElapsedSpan,
            &[Hole::Span(Duration::from_nanos(wall.as_nanos().saturating_sub(since.as_nanos())))],
        );
    }
}
