//! Public identities, titles, phases and times (temper/docs/design/client/browser.md, 8.3).
use super::Draw;
use crate::words::{TimeForm, Word};
use alloc::boxed::Box;
use jig_client_protocol_wire::public_v1::{PublicExecutor, PublicLine, PublicPhase};
use skein_lib::Wall;
use skein_ui::{Builder, Role, Run, phrases::Hole};
use temper_client_domain::address::Public;
impl Draw<'_> {
    pub(super) fn identity(&self, id: u64, goal: bool) -> Box<[u8]> {
        self.words.say(if goal { Word::PublicGoal } else { Word::PublicTask }, &[Hole::Name(short(id))])
    }
    pub(super) fn task_link(&mut self, b: &mut Builder, task: u64, goal: bool) {
        self.link(b, &self.identity(task, goal), Public::Task { project: self.project, task });
    }
    pub(super) fn folded(&self, b: &mut Builder, count: u32) {
        if count == 0 {
            return;
        }
        b.open(Role::Paragraph);
        self.words.text(b, Word::PublicFolded, &[Hole::Number(u64::from(count))]);
        b.close();
    }
    pub(super) fn line(&mut self, b: &mut Builder, line: &PublicLine) {
        b.open(Role::Group);
        b.style(crate::look::PUBLIC_ROW);
        b.open(Role::Group);
        b.style(crate::look::PUBLIC_TITLE);
        self.task_link(b, line.task(), line.goal());
        b.open(Role::Paragraph);
        b.text(Run::Plain, line.title());
        if line.title_cut() {
            self.words.text(b, Word::PublicTitleCut, &[]);
        }
        b.close();
        b.close();
        b.open(Role::Group);
        b.style(crate::look::PUBLIC_META);
        let executor = match line.executor() {
            PublicExecutor::Agent => Word::PublicAgent,
            PublicExecutor::Procedure => Word::PublicProcedure,
            PublicExecutor::Person => Word::PublicPerson,
            PublicExecutor::Job => Word::PublicJob,
        };
        self.text(b, executor);
        self.text(b, Word::PublicSeparator);
        self.text(b, phase(line.phase()));
        let since = Wall::from_nanos(line.changed().saturating_mul(1_000_000_000));
        let duration = self.times.say(self.words, TimeForm::Span, since);
        self.text(b, Word::PublicSeparator);
        self.words.text(b, Word::PublicFor, &[Hole::Name(duration)]);
        self.text(b, Word::PublicSeparator);
        self.words.text(b, Word::ChatbarSpend, &[Hole::Number(line.spent()), Hole::Number(line.budget())]);
        if let Some(priority) = line.priority() {
            self.text(b, Word::PublicSeparator);
            self.words.text(b, Word::Priority, &[Hole::Number(u64::from(*priority))]);
        }
        b.close();
        b.close();
    }
}
fn phase(phase: &PublicPhase) -> Word {
    match phase {
        PublicPhase::Waiting => Word::PhaseWaiting,
        PublicPhase::Idle => Word::PhaseIdle,
        PublicPhase::Due => Word::PhaseDue,
        PublicPhase::Preparing => Word::PhasePreparing,
        PublicPhase::Claimed => Word::PhaseClaimed,
        PublicPhase::Running => Word::Running,
        PublicPhase::BackingOff => Word::PhaseRetrying,
        PublicPhase::Active => Word::PhaseActive,
        PublicPhase::Closing => Word::PhaseClosing,
        PublicPhase::Held => Word::PhaseHeld,
        PublicPhase::Ended => Word::PublicEnded,
        PublicPhase::Done => Word::PublicDone,
        PublicPhase::Failed => Word::PublicFailed,
        PublicPhase::Cancelled => Word::PublicCancelled,
    }
}
fn short(id: u64) -> Box<[u8]> {
    let path = temper_client_domain::address::print_public(&Public::Board { project: id });
    path.strip_prefix(b"/public/projects/").expect("public project path").get(..8).expect("public identifier").into()
}
