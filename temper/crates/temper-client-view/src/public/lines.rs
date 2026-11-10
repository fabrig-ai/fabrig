//! Public identities, rows and times (temper/docs/design/client/web/public.md, 3).
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
        b.style(crate::look::PUBLIC_FACTS);
        self.words.text(b, Word::PublicFolded, &[Hole::Number(u64::from(count))]);
        b.close();
    }
    /// Draw an open list item's row, leaving room for its nested tree.
    pub(super) fn row(&mut self, b: &mut Builder, line: &PublicLine, executor: bool) {
        b.style(crate::look::PUBLIC_ROW);
        b.open(Role::Paragraph);
        b.style(crate::look::PUBLIC_LINE);
        self.task_link(b, line.task(), line.goal());
        b.text(Run::Plain, b" ");
        b.text(Run::Plain, line.title());
        if line.title_cut() {
            self.words.text(b, Word::PublicTitleCut, &[]);
        }
        b.close();
        b.open(Role::Paragraph);
        b.style(crate::look::PUBLIC_FACTS);
        if executor {
            let executor = match line.executor() {
                PublicExecutor::Agent => Word::PublicAgent,
                PublicExecutor::Procedure => Word::PublicProcedure,
                PublicExecutor::Person => Word::PublicPerson,
                PublicExecutor::Job => Word::PublicJob,
            };
            self.text(b, executor);
            self.text(b, Word::ToolSeparator);
        }
        self.text(b, phase(line.phase(), line.goal()));
        self.text(b, Word::ToolSeparator);
        let ended = match line.phase() {
            PublicPhase::Done | PublicPhase::Failed | PublicPhase::Cancelled => true,
            PublicPhase::Waiting
            | PublicPhase::Idle
            | PublicPhase::Due
            | PublicPhase::Preparing
            | PublicPhase::Claimed
            | PublicPhase::Running
            | PublicPhase::BackingOff
            | PublicPhase::Active
            | PublicPhase::Closing
            | PublicPhase::Held => false,
        };
        let seconds = if ended { line.ended().unwrap_or(line.changed()) } else { line.changed() };
        let since = Wall::from_nanos(seconds.saturating_mul(1_000_000_000));
        let duration = self.times.public_time(self.words, if ended { TimeForm::Ago } else { TimeForm::Span }, since);
        if ended {
            b.text(Run::Plain, &duration);
        } else {
            self.words.text(b, Word::PublicFor, &[Hole::Name(duration)]);
        }
        if let Some(spent) = line.spent() {
            self.text(b, Word::ToolSeparator);
            self.words.text(b, Word::RowSpent, &[Hole::Number(*spent)]);
        }
        b.close();
    }
}
fn phase(phase: &PublicPhase, goal: bool) -> Word {
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
        PublicPhase::Done => {
            if goal {
                Word::PublicLanded
            } else {
                Word::PublicDone
            }
        }
        PublicPhase::Failed => Word::PublicFailed,
        PublicPhase::Cancelled => Word::PublicEnded,
    }
}
fn short(id: u64) -> Box<[u8]> {
    let path = temper_client_domain::address::print_public(&Public::Board { project: id });
    path.strip_prefix(b"/public/projects/").expect("public project path").get(..8).expect("public identifier").into()
}
