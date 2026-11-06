//! Public forge mechanics and links (temper/docs/design/client/forge.md, 11.1 and 11.3).
use super::{Draw, external};
use crate::words::{TimeForm, Word};
use alloc::boxed::Box;
use jig_client_protocol_wire::public_v1::PublicApplication;
use skein_lib::{Wall, Writer};
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
            for change in queue.changes() {
                b.open(Role::Item);
                self.text(b, Word::PublicChangeTask);
                self.text(b, Word::PublicSeparator);
                self.task(b, app, change.task(), false);
                self.text(b, Word::PublicSeparator);
                self.text(b, Word::PublicForGoal);
                self.text(b, Word::PublicSeparator);
                self.task(b, app, change.goal(), true);
                self.text(b, Word::PublicSeparator);
                let time = self.times.say(self.words, TimeForm::Ago, seconds(change.ready_since()));
                self.words.text(b, Word::PublicReadySince, &[Hole::Name(time)]);
                b.close();
            }
            b.close();
            b.close();
        }
    }
    pub(super) fn landings(&mut self, b: &mut Builder, app: &PublicApplication, landings: &f::PublicLandings) {
        if landings.turn().is_none() && landings.queues().is_empty() && landings.landings().is_empty() {
            self.paragraph(b, Word::PublicNoLandings);
        }
        if let Some(turn) = landings.turn() {
            b.open(Role::Paragraph);
            self.text(b, Word::PublicTurn);
            self.text(b, Word::PublicSeparator);
            self.landing_link(b, app, *turn);
            b.close();
        }
        self.queues(b, app, landings);
        b.open(Role::List);
        for landing in landings.landings() {
            b.open(Role::Item);
            b.open(Role::Paragraph);
            self.forge_words(b, landing_phase(landing.phase()), &[]);
            b.close();
            for task in landing.members() {
                b.open(Role::Paragraph);
                self.text(b, Word::PublicLandingDetails);
                self.text(b, Word::PublicSeparator);
                self.landing_link(b, app, *task);
                b.close();
            }
            for hash in landing.landed() {
                b.open(Role::Paragraph);
                self.text(b, Word::PublicLandedCommit);
                self.text(b, Word::PublicSeparator);
                if let Some(member) = landing.members().as_slice().first()
                    && let Some(task) = app.tasks().get(u32::from(*member))
                {
                    self.link(
                        b,
                        hex(hash).get(..8).expect("short hash"),
                        Public::Landing { project: self.project, task: *task },
                    );
                }
                b.close();
            }
            b.close();
        }
        b.close();
    }
    pub(super) fn change(&mut self, b: &mut Builder, _app: &PublicApplication, change: &f::PublicChange) {
        b.open(Role::Region);
        b.style(crate::look::CHANGE);
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
        self.text(b, Word::PublicSeparator);
        self.forge_words(b, ci, &[]);
        b.close();
        self.change_gates(b, change);
        self.pull(b, change.pull().as_deref());
        self.commits(
            b,
            change.repository(),
            &[(Word::PublicHeadCommit, change.head().as_ref()), (Word::PublicMergeCommit, change.merge().as_ref())],
        );
        b.close();
    }
    fn change_gates(&self, b: &mut Builder, change: &f::PublicChange) {
        b.open(Role::List);
        for gate in change.gates() {
            b.open(Role::Item);
            self.forge_words(b, gate_kind(gate.kind(), gate.blocks()), &[Hole::Number(gate.number())]);
            let verdict = match gate.verdict() {
                f::PublicVerdict::None => W::NotYet,
                f::PublicVerdict::Pending => W::Waiting,
                f::PublicVerdict::Approved => W::Approved,
                f::PublicVerdict::Changes => W::Changes,
            };
            self.text(b, Word::PublicSeparator);
            self.forge_words(b, verdict, &[]);
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
        if landing.members().is_empty() {
            self.paragraph(b, Word::PublicNoMembers);
        }
        b.open(Role::List);
        for member in landing.members() {
            b.open(Role::Item);
            self.member(b, app, member);
            b.close();
        }
        b.close();
        b.open(Role::Paragraph);
        self.words.text(b, Word::PublicStaleProofs, &[Hole::Number(u64::from(landing.stale_proofs()))]);
        b.close();
        self.paragraph(b, Word::PublicProofExplanation);
    }
    fn member(&mut self, b: &mut Builder, app: &PublicApplication, member: &f::PublicMember) {
        b.open(Role::Paragraph);
        self.text(b, Word::PublicChangeTask);
        self.text(b, Word::PublicSeparator);
        self.task(b, app, member.task(), false);
        b.close();
        self.pull(b, member.pull().as_deref());
        self.commits(
            b,
            member.repository(),
            &[
                (Word::PublicBaseCommit, member.base().as_ref()),
                (Word::PublicCandidateCommit, member.candidate().as_ref()),
                (Word::PublicLandedCommit, member.landed().as_ref()),
            ],
        );
        b.open(Role::List);
        for gate in member.gates() {
            b.open(Role::Item);
            self.forge_words(b, gate_kind(gate.kind(), gate.blocks()), &[Hole::Number(gate.number())]);
            self.text(b, Word::PublicSeparator);
            let proof = match gate.proof() {
                f::PublicProof::Unkeyed | f::PublicProof::Keyed => W::NotYet,
                f::PublicProof::Dispatched | f::PublicProof::Waiting => W::Running,
                f::PublicProof::Passed => W::Passed,
                f::PublicProof::Failed => W::Failed,
            };
            match gate.run() {
                Some(run) => external(b, &super::forge_text(self.words, proof, &[]), run),
                None => self.forge_words(b, proof, &[]),
            }
            b.close();
        }
        b.close();
    }
    fn pull(&self, b: &mut Builder, pull: Option<&[u8]>) {
        if let Some(pull) = pull {
            b.open(Role::Paragraph);
            external(b, &self.words.say(Word::PublicChange, &[]), pull);
            b.close();
        }
    }
    fn commits(&self, b: &mut Builder, repository: &[u8], fields: &[(Word, Option<&[u8; 32]>)]) {
        for (index, (label, hash)) in fields.iter().enumerate() {
            let Some(hash) = hash else { continue };
            let before = fields.get(..index).expect("enumerated prefix");
            let mut seen = false;
            for (_, previous) in before {
                if previous == &Some(*hash) {
                    seen = true;
                }
            }
            if seen {
                continue;
            }
            b.open(Role::Paragraph);
            self.text(b, *label);
            for (other, next) in fields.get(index.checked_add(1).expect("bounded fields")..).expect("field suffix") {
                if next == &Some(*hash) {
                    self.text(b, Word::PublicSeparator);
                    self.text(b, *other);
                }
            }
            self.text(b, Word::PublicSeparator);
            commit(b, repository, hash);
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
fn commit(b: &mut Builder, repository: &[u8], hash: &[u8; 32]) {
    if !skein_ui::markdown::safe_url(repository) {
        return;
    }
    let hash = hex(hash);
    let mut writer = Writer::new(
        repository
            .len()
            .checked_add(b"/commit/".len())
            .expect("bounded repository")
            .checked_add(hash.len())
            .expect("bounded commit address"),
    );
    writer.put(repository).expect("repository");
    writer.put(b"/commit/").expect("commit path");
    writer.put(&hash).expect("hash");
    external(b, hash.get(..8).expect("short hash"), &writer.finish());
}
fn hex(bytes: &[u8; 32]) -> Box<[u8]> {
    let mut size = 20_usize;
    for byte in bytes.get(20..).expect("fixed hash") {
        if *byte != 0 {
            size = 32;
        }
    }
    let mut writer = Writer::new(size.checked_mul(2).expect("fixed hash"));
    for byte in bytes.get(..size).expect("fixed hash") {
        for nibble in [byte >> 4_u8, byte & 15_u8] {
            let byte = *b"0123456789abcdef".get(usize::from(nibble)).expect("hex digit");
            writer.put(&[byte]).expect("fixed hash");
        }
    }
    writer.finish()
}
