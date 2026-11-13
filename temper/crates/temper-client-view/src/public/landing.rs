//! Landing mechanics without forge addresses or configured names (client/forge/README.md, 9.3).
use super::Draw;
use skein_ui::{Builder, Role, phrases::Hole};
use temper_client_domain::forge::words::ChangeWord as W;
use temper_client_protocol_wire::public_v1 as f;

impl Draw<'_> {
    pub(super) fn landing_details(&mut self, b: &mut Builder, landing: &f::PublicLanding) {
        if let Some(combined) = landing.combined() {
            b.open(Role::Paragraph);
            self.forge_words(
                b,
                match combined {
                    f::PublicCombined::Merge => W::Merge,
                    f::PublicCombined::Squash => W::Squash,
                },
                &[],
            );
            b.close();
        }
        if let Some(step) = landing.step() {
            b.open(Role::Paragraph);
            if *step == f::PublicLandingStep::Probing
                && let Some(search) = landing.search()
                && let Some(prefix) = search.probe()
            {
                self.forge_words(b, W::ProbingAt, &[Hole::Name(super::forge::short(prefix).as_slice().into())]);
            } else {
                self.forge_words(b, step_word(step), &[]);
            }
            b.close();
        }
        if let Some(search) = landing.search() {
            for (word, bound) in [(W::SearchGreen, search.green()), (W::SearchRed, search.red())] {
                b.open(Role::Paragraph);
                self.forge_words(
                    b,
                    word,
                    &[
                        Hole::Number(u64::from(bound.member())),
                        Hole::Name(super::forge::short(bound.prefix()).as_slice().into()),
                    ],
                );
                b.close();
            }
        }
    }
    pub(super) fn member_details(&mut self, b: &mut Builder, member: &f::PublicMember) {
        if let Some(head) = member.head() {
            b.open(Role::Paragraph);
            self.forge_words(b, W::RoundHead, &[Hole::Name(super::forge::short(head).as_slice().into())]);
            b.close();
        }
        if let Some(judgement) = member.judgement() {
            let status = match judgement.verdict() {
                f::PublicCi::Unknown => W::NotYet,
                f::PublicCi::Pending => W::Waiting,
                f::PublicCi::Passed => W::Passed,
                f::PublicCi::Failed => W::Failed,
            };
            let status = super::forge_text(self.words, status, &[]);
            b.open(Role::Paragraph);
            self.forge_words(
                b,
                W::PrefixJudgement,
                &[Hole::Name(super::forge::short(judgement.prefix()).as_slice().into()), Hole::Name(status)],
            );
            b.close();
        }
        if let Some(left) = member.left() {
            b.open(Role::Paragraph);
            let (word, number) = match left {
                f::PublicLeave::Conflict => (W::LeftConflict, None),
                f::PublicLeave::HeadMoved => (W::LeftHeadMoved, None),
                f::PublicLeave::Cancelled => (W::LeftCancelled, None),
                f::PublicLeave::Held => (W::LeftHeld, None),
                f::PublicLeave::Refused(gate) => (W::LeftRefused, Some(gate.gate())),
                f::PublicLeave::Failed(gate) => (W::LeftFailed, Some(gate.gate())),
                f::PublicLeave::Blamed(gate) => {
                    self.forge_words(
                        b,
                        W::LeftBlamed,
                        &[
                            Hole::Number(gate.gate()),
                            Hole::Name(super::forge::short(gate.red()).as_slice().into()),
                            Hole::Name(super::forge::short(gate.green()).as_slice().into()),
                        ],
                    );
                    b.close();
                    return;
                }
            };
            match number {
                Some(number) => self.forge_words(b, word, &[Hole::Number(number)]),
                None => self.forge_words(b, word, &[]),
            }
            b.close();
        }
    }
}
fn step_word(step: &f::PublicLandingStep) -> W {
    match step {
        f::PublicLandingStep::Waiting => W::Waiting,
        f::PublicLandingStep::Preparing => W::HeldPreparing,
        f::PublicLandingStep::Proving => W::Checking,
        f::PublicLandingStep::Probing => W::Probing,
        f::PublicLandingStep::Proved => W::Proved,
        f::PublicLandingStep::Turn => W::HeldLanding,
        f::PublicLandingStep::Landed => W::Landed,
        f::PublicLandingStep::Held => W::LeftHeld,
        f::PublicLandingStep::Cancelled => W::Cancelled,
        f::PublicLandingStep::Judging => W::Judging,
        f::PublicLandingStep::Leaving => W::Leaving,
        f::PublicLandingStep::Recording => W::Recording,
        f::PublicLandingStep::Telling => W::Telling,
    }
}
