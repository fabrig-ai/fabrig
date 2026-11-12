//! Public forge mechanics and commits (temper/docs/design/client/forge/README.md, 9.1 and 9.3).
use super::Draw;
use crate::words::{TimeForm, Word};
use jig_client_protocol_wire::public_v1::PublicApplication;
use skein_lib::Wall;
use skein_ui::{Builder, Role, phrases::Hole};
use temper_client_domain::{address::Public, forge::words::ChangeWord as W};
use temper_client_protocol_wire::public_v1 as f;
impl Draw<'_> {
    fn forge_words(&self, b: &mut Builder, word: W, holes: &[Hole]) {
        b.text(skein_ui::Run::Plain, &super::forge_text(self.words, word, holes));
    }
    fn task(&mut self, b: &mut Builder, app: &PublicApplication, index: u16, goal: bool) {
        if let Some(task) = app.tasks().get(u32::from(index)) {
            self.task_link(b, *task, goal);
        }
    }
    fn landing_link(&mut self, b: &mut Builder, app: &PublicApplication, index: u16) {
        if let Some(task) = app.tasks().get(u32::from(index)) {
            self.link(b, &self.identity(*task, false), Public::Landing { project: self.project, task: *task });
        }
    }
    pub(super) fn queues(&mut self, b: &mut Builder, app: &PublicApplication, landings: &f::PublicLandings) {
        for queue in landings.queues() {
            b.open(Role::Region);
            self.paragraph(b, Word::PublicQueue);
            b.open(Role::List);
            b.style(crate::look::PUBLIC_ROWS);
            for change in queue.changes() {
                b.open(Role::Item);
                b.style(crate::look::PUBLIC_ROW);
                b.open(Role::Paragraph);
                b.style(crate::look::PUBLIC_LINE);
                self.task(b, app, change.task(), false);
                self.text(b, Word::ToolSeparator);
                self.task(b, app, change.goal(), true);
                self.text(b, Word::ToolSeparator);
                let time = self.times.public_time(self.words, TimeForm::Ago, seconds(change.ready_since()));
                self.words.text(b, Word::PublicReadySince, &[Hole::Name(time)]);
                b.close();
                b.close();
            }
            b.close();
            b.close();
        }
    }
    pub(super) fn landings(&mut self, b: &mut Builder, app: &PublicApplication, landings: &f::PublicLandings) {
        if let Some(turn) = landings.turn() {
            b.open(Role::Paragraph);
            self.text(b, Word::PublicTurn);
            self.text(b, Word::ToolSeparator);
            self.landing_link(b, app, *turn);
            b.close();
        }
        self.queues(b, app, landings);
        if landings.landings().is_empty() {
            self.paragraph(b, Word::PublicNoLandings);
        }
        b.open(Role::List);
        b.style(crate::look::PUBLIC_ROWS);
        for landing in landings.landings() {
            b.open(Role::Item);
            b.style(crate::look::PUBLIC_ROW);
            b.open(Role::Paragraph);
            b.style(crate::look::PUBLIC_LINE);
            self.forge_words(b, landing_phase(landing.phase()), &[]);
            for task in landing.members() {
                self.text(b, Word::ToolSeparator);
                self.landing_link(b, app, *task);
            }
            for hash in landing.landed() {
                if let Some(member) = landing.members().as_slice().first()
                    && let Some(task) = app.tasks().get(u32::from(*member))
                {
                    self.text(b, Word::ToolSeparator);
                    self.link(b, &short(hash), Public::Landing { project: self.project, task: *task });
                }
            }
            b.close();
            b.close();
        }
        b.close();
    }
    pub(super) fn change(&mut self, b: &mut Builder, _app: &PublicApplication, change: &f::PublicChange) {
        b.open(Role::Region);
        b.style(crate::look::PUBLIC_SECTION);
        b.open(Role::Paragraph);
        self.forge_words(b, step(change.step()), &[]);
        b.close();
        b.open(Role::Paragraph);
        self.forge_words(b, W::Ci, &[]);
        let ci = match change.ci() {
            f::PublicCi::Unknown => W::NotRead,
            f::PublicCi::Pending => W::Running,
            f::PublicCi::Passed => W::Passed,
            f::PublicCi::Failed => W::Failed,
        };
        self.text(b, Word::ToolSeparator);
        self.forge_words(b, ci, &[]);
        b.close();
        b.open(Role::List);
        b.style(crate::look::PUBLIC_ROWS);
        for gate in change.gates() {
            b.open(Role::Item);
            b.style(crate::look::PUBLIC_ROW);
            b.open(Role::Paragraph);
            b.style(crate::look::PUBLIC_LINE);
            self.forge_words(b, gate_kind(gate.kind(), gate.blocks()), &[Hole::Number(gate.number())]);
            let verdict = match gate.verdict() {
                f::PublicVerdict::None => W::NotYet,
                f::PublicVerdict::Pending => W::Waiting,
                f::PublicVerdict::Approved => W::Approved,
                f::PublicVerdict::Changes => W::Changes,
            };
            self.text(b, Word::ToolSeparator);
            self.forge_words(b, verdict, &[]);
            b.close();
            b.close();
        }
        b.close();
        if let Some(hash) = change.merge().as_ref().or(change.head().as_ref()) {
            b.open(Role::Paragraph);
            self.words.text(b, Word::PublicChange, &[Hole::Name(short(hash).as_slice().into())]);
            b.close();
        }
        b.close();
    }
    pub(super) fn landing(&mut self, b: &mut Builder, app: &PublicApplication, landing: &f::PublicLanding) {
        b.open(Role::Paragraph);
        self.forge_words(b, landing_phase(landing.phase()), &[]);
        b.close();
        if landing.holds_turn() {
            self.paragraph(b, Word::PublicTurn);
        }
        b.open(Role::List);
        b.style(crate::look::PUBLIC_ROWS);
        for member in landing.members() {
            b.open(Role::Item);
            b.style(crate::look::PUBLIC_ROW);
            b.open(Role::Paragraph);
            b.style(crate::look::PUBLIC_LINE);
            self.task(b, app, member.task(), false);
            if let Some(hash) = member.landed().as_ref().or(member.candidate().as_ref()) {
                self.text(b, Word::ToolSeparator);
                self.words.text(b, Word::PublicChange, &[Hole::Name(short(hash).as_slice().into())]);
            }
            b.close();
            b.open(Role::List);
            b.style(crate::look::PUBLIC_ROWS);
            for gate in member.gates() {
                b.open(Role::Item);
                b.style(crate::look::PUBLIC_ROW);
                b.open(Role::Paragraph);
                b.style(crate::look::PUBLIC_LINE);
                self.forge_words(b, gate_kind(gate.kind(), gate.blocks()), &[Hole::Number(gate.number())]);
                let proof = match gate.proof() {
                    f::PublicProof::Unkeyed | f::PublicProof::Keyed => W::NotYet,
                    f::PublicProof::Dispatched | f::PublicProof::Waiting => W::Running,
                    f::PublicProof::Passed => W::Passed,
                    f::PublicProof::Failed => W::Failed,
                };
                self.text(b, Word::ToolSeparator);
                self.forge_words(b, proof, &[]);
                b.close();
                b.close();
            }
            b.close();
            b.close();
        }
        b.close();
        if landing.stale_proofs() > 0 {
            b.open(Role::Paragraph);
            self.words.text(b, Word::PublicStaleProofs, &[Hole::Number(u64::from(landing.stale_proofs()))]);
            b.close();
        }
    }
}
fn seconds(seconds: u64) -> Wall {
    Wall::from_nanos(seconds.saturating_mul(1_000_000_000))
}
fn gate_kind(kind: &f::PublicGateKind, blocks: bool) -> W {
    match (kind, blocks) {
        (f::PublicGateKind::Review, true) => W::ReviewBlocking,
        (f::PublicGateKind::Review, false) => W::ReviewAdvisory,
        (f::PublicGateKind::Approval, true) => W::ApprovalBlocking,
        (f::PublicGateKind::Approval, false) => W::ApprovalAdvisory,
        (f::PublicGateKind::Check, true) => W::CheckBlocking,
        (f::PublicGateKind::Check, false) => W::CheckAdvisory,
    }
}
fn step(step: &f::PublicStep) -> W {
    match step {
        f::PublicStep::Producing => W::HeldProducing,
        f::PublicStep::Opening => W::HeldOpening,
        f::PublicStep::Recreating => W::HeldRecreating,
        f::PublicStep::Reopening => W::HeldReopening,
        f::PublicStep::Checking => W::HeldChecking,
        f::PublicStep::Gating => W::HeldGating,
        f::PublicStep::Queued => W::HeldQueued,
        f::PublicStep::First => W::HeldFirst,
        f::PublicStep::Updating => W::HeldUpdating,
        f::PublicStep::Resolving => W::HeldResolving,
        f::PublicStep::Repairing => W::HeldRepairing,
        f::PublicStep::Landing => W::HeldLanding,
        f::PublicStep::Landed => W::HeldLanded,
        f::PublicStep::Ended => W::EndedLine,
        f::PublicStep::Ready => W::HeldReady,
        f::PublicStep::Preparing => W::HeldPreparing,
        f::PublicStep::Verifying => W::HeldVerifying,
        f::PublicStep::Red => W::HeldRed,
        f::PublicStep::Cleared => W::HeldCleared,
    }
}
fn landing_phase(phase: &f::PublicLandingPhase) -> W {
    match phase {
        f::PublicLandingPhase::Waiting => W::Waiting,
        f::PublicLandingPhase::Preparing => W::HeldPreparing,
        f::PublicLandingPhase::Proving => W::HeldChecking,
        f::PublicLandingPhase::Proved => W::Passed,
        f::PublicLandingPhase::Turn => W::HeldFirst,
        f::PublicLandingPhase::Landed => W::HeldLanded,
        f::PublicLandingPhase::Held => W::Stalled,
        f::PublicLandingPhase::Cancelled => W::EndedLine,
    }
}
pub(crate) fn short(hash: &[u8; 32]) -> [u8; 7] {
    let mut digits = [0; 7];
    for (position, digit) in digits.iter_mut().enumerate() {
        let byte = hash.get(position.checked_div(2).expect("nonzero divisor")).expect("short hash byte");
        let nibble = if position.is_multiple_of(2) { byte >> 4_u8 } else { byte & 15_u8 };
        *digit = *b"0123456789abcdef".get(usize::from(nibble)).expect("hex digit");
    }
    digits
}
