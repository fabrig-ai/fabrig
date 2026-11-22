//! Faults in the forge's own words (testing/fakes/forge.md, 2.4).

use alloc::boxed::Box;
use skein_lib::bytes::copy_of;
use skein_lib::{Id, Time, Token};

use crate::api::{Git, Op, Read, Want, Write};
use crate::domain::Domain;
use crate::store::Repository;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Fell {
    pub at: Time,
    pub until: Option<Time>,
    pub fault: Fault,
    pub repository: Option<Box<[u8]>>,
    pub user: Option<u64>,
    pub call: Option<Called>,
    pub reply_to: Option<Token>,
    pub pull: Option<u64>,
    pub commit: Option<u64>,
    pub run: Option<u64>,
    pub runner: Option<u32>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Called {
    Read,
    Write,
    Merge,
    Git,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Fault {
    Busy,
    Unavailable,
    TimedOut,
    Landing,
    Late,
    Limited,
    Crowded,
    HookLate,
    HookLost,
    HookDropped,
    Silent,
    Rerun,
    SetUp,
    Lost,
    RunnerWait,
    HeadBehind,
    BaseBehind,
    MergeLate,
    CloseRefused,
}

impl Fault {
    #[must_use]
    pub const fn family(self) -> &'static str {
        match self {
            Self::Busy
            | Self::Unavailable
            | Self::TimedOut
            | Self::Landing
            | Self::Late
            | Self::Limited
            | Self::Crowded => "busy",
            Self::HookLate | Self::HookLost | Self::HookDropped => "hooks",
            Self::Silent | Self::Rerun | Self::SetUp | Self::Lost | Self::RunnerWait => "ci",
            Self::HeadBehind | Self::BaseBehind | Self::MergeLate => "lagging",
            Self::CloseRefused => "refusing",
        }
    }

    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Busy => "no_room",
            Self::Unavailable => "unavailable",
            Self::TimedOut => "timed_out",
            Self::Landing => "landing",
            Self::Late | Self::HookLate => "late",
            Self::Limited => "limited",
            Self::Crowded => "crowded",
            Self::HookLost | Self::Lost => "lost",
            Self::HookDropped => "dropped",
            Self::Silent => "silent",
            Self::Rerun => "rerun",
            Self::SetUp => "set_up",
            Self::RunnerWait => "runner_wait",
            Self::HeadBehind => "head_behind",
            Self::BaseBehind => "base_behind",
            Self::MergeLate => "merge_late",
            Self::CloseRefused => "close_refused",
        }
    }
}

impl Fell {
    pub(crate) fn new(at: Time, fault: Fault) -> Self {
        Self {
            at,
            until: None,
            fault,
            repository: None,
            user: None,
            call: None,
            reply_to: None,
            pull: None,
            commit: None,
            run: None,
            runner: None,
        }
    }

    pub(crate) fn call(at: Time, user: u64, repository: &[u8], op: &Op, reply_to: Token) -> Self {
        let mut fell = Self::new(at, Fault::Busy);
        fell.repository = Some(copy_of(repository));
        fell.user = Some(user);
        fell.reply_to = Some(reply_to);
        fell.call = Some(match op {
            Op::Read(read) => {
                fell.read_subject(read);
                Called::Read
            }
            Op::Git(git) => {
                fell.commit = match git {
                    Git::Push { commit, .. } | Git::PushNew { commit, .. } | Git::Create { commit, .. } => {
                        Some(*commit)
                    }
                    Git::Fetch { want } => match want {
                        Want::Commit(commit) => Some(*commit),
                        Want::Branch(_) | Want::Default => None,
                    },
                    Git::Clone => None,
                };
                Called::Git
            }
            Op::Write(write) => match write {
                Write::Merge { number, head, .. } => {
                    fell.pull = Some(*number);
                    fell.commit = Some(*head);
                    Called::Merge
                }
                Write::Enqueue { number, head } => {
                    fell.pull = Some(*number);
                    fell.commit = Some(*head);
                    Called::Write
                }
                Write::SetReviewers { number, .. }
                | Write::RequestReviewers { number, .. }
                | Write::RemoveReviewers { number, .. }
                | Write::Review { number, .. }
                | Write::Submit { number, .. }
                | Write::Update { number, .. } => {
                    fell.pull = Some(*number);
                    Called::Write
                }
                Write::CreateBranch { commit, .. } | Write::Status { commit, .. } => {
                    fell.commit = Some(*commit);
                    Called::Write
                }
                Write::Cancel { run } => {
                    fell.run = Some(*run);
                    Called::Write
                }
                Write::Copy { .. }
                | Write::DeleteRepository
                | Write::SetProtection { .. }
                | Write::CreateIssue { .. }
                | Write::Comment { .. }
                | Write::EditItem { .. }
                | Write::SetDependencies { .. }
                | Write::EditComment { .. }
                | Write::DeleteComment { .. }
                | Write::SetLabels { .. }
                | Write::AddLabels { .. }
                | Write::RemoveLabels { .. }
                | Write::DefineLabel { .. }
                | Write::OpenPull { .. }
                | Write::Close { .. }
                | Write::Reopen { .. }
                | Write::DeleteBranch { .. }
                | Write::Dispatch { .. }
                | Write::Rerun { .. }
                | Write::PutPage { .. }
                | Write::DeletePage { .. } => Called::Write,
            },
        });
        fell
    }

    fn read_subject(&mut self, read: &Read) {
        match read {
            Read::Pull { number } | Read::PullFiles { number, .. } => self.pull = Some(*number),
            Read::Checks { commit } | Read::Statuses { commit } | Read::Tree { commit } | Read::File { commit, .. } => {
                self.commit = Some(*commit);
            }
            Read::Runs { commit, .. } => self.commit = *commit,
            Read::Run { run } | Read::Jobs { run } | Read::Artifacts { run } | Read::Artifact { run, .. } => {
                self.run = Some(*run);
            }
            Read::ArtifactById { id } => self.run = Some(*id),
            Read::Commits { .. }
            | Read::Locate
            | Read::Items { .. }
            | Read::Item { .. }
            | Read::Comment { .. }
            | Read::Dependencies { .. }
            | Read::Labels
            | Read::Compare { .. }
            | Read::Job { .. }
            | Read::Log { .. }
            | Read::GitTree { .. }
            | Read::Comments { .. }
            | Read::FirstPull { .. }
            | Read::OpenPulls { .. }
            | Read::Protection { .. }
            | Read::Settings
            | Read::Collaborators
            | Read::PullFor { .. }
            | Read::Permission { .. }
            | Read::ListingPermission
            | Read::Branch { .. }
            | Read::Pages { .. }
            | Read::Page { .. } => {}
        }
    }

    pub(crate) fn repository(domain: &Domain, at: Time, fault: Fault, id: Id<Repository>) -> Self {
        let mut fell = Self::new(at, fault);
        fell.repository = Some(copy_of(&domain.repositories.get(id).expect("retained repository").name));
        fell
    }
}

impl Domain {
    /// The next fault, oldest first. Images retain neither these records nor their loss count.
    pub fn pop_fell(&mut self) -> Option<Fell> {
        self.fell.pop()
    }

    #[must_use]
    pub fn fell_lost(&self) -> u64 {
        self.fell_lost
    }

    pub(crate) fn record_fell(&mut self, fell: Fell) {
        if self.fell.try_push(fell).is_err() {
            self.fell_lost = self.fell_lost.saturating_add(1);
        }
    }

    pub(crate) fn call_fell(&mut self, subject: &Fell, fault: Fault, until: Option<Time>) {
        self.record_fell(Fell { until, fault, ..subject.clone() });
    }

    pub(crate) fn ci_fell(&mut self, at: Time, fault: Fault, id: Id<Repository>, commit: u64, run: u64) {
        let mut fell = Fell::repository(self, at, fault, id);
        fell.commit = Some(commit);
        fell.run = Some(run);
        self.record_fell(fell);
    }
}
