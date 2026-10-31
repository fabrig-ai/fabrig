//! Compact exact lifecycle; whole metadata is expanded on the task page.
//! See client/domain/objects.md, 3, and domain/tasks.md, 5.
use crate::words::Words;
use crate::words::phrases::jig::JigPhrase;
use jig_client::contract::Phase;
use jig_client::words::Word;
use skein_ui::Builder;
use skein_ui::phrases::Hole;

use jig_client::contract::{ChipPhase, ClosingStage, RunStage};

pub(crate) fn brief(b: &mut Builder, phase: &ChipPhase, words: &Words) {
    match phase {
        ChipPhase::Waiting { more, holds, .. } => {
            words.text(b, JigPhrase::Word(Word::Phase(Phase::Waiting)), &[]);
            if *more {
                words.text(b, JigPhrase::ChipDependenciesCut, &[]);
            }
            if *holds {
                words.text(b, JigPhrase::ChipWaitingHolds, &[]);
            }
        }
        ChipPhase::Active(stage) => match stage {
            RunStage::Idle => words.text(b, JigPhrase::RunIdle, &[]),
            RunStage::Due => words.text(b, JigPhrase::RunDue, &[]),
            RunStage::Preparing => words.text(b, JigPhrase::RunPreparing, &[]),
            RunStage::Claimed { attempt } => words.text(b, JigPhrase::RunClaimed, &[Hole::Number(*attempt)]),
            RunStage::Running { attempt } => words.text(b, JigPhrase::RunOwnerActive, &[Hole::Number(*attempt)]),
            RunStage::BackingOff { until } => {
                words.text(b, JigPhrase::RunBackingOff, &[Hole::Wall(words.adjusted_wall(*until))]);
            }
        },
        ChipPhase::Person { address, taken_by, since } => {
            crate::tree::fragments::proposed_task::executor(b, jig_client::contract::Executor::Person(*address), words);
            if let Some(party) = taken_by {
                words.text(b, JigPhrase::TakenByNumber, &[Hole::Number(*party)]);
            }
            if let Some(at) = since {
                words.text(b, JigPhrase::Since, &[Hole::Wall(words.adjusted_wall(*at))]);
            }
        }
        ChipPhase::Closing(stage) => {
            words.text(b, JigPhrase::Word(Word::Phase(Phase::Closing)), &[]);
            match stage {
                ClosingStage::Run { attempt } => words.text(b, JigPhrase::ClosingRun, &[Hole::Number(*attempt)]),
                ClosingStage::Delegates => words.text(b, JigPhrase::ClosingDelegates, &[]),
                ClosingStage::Effects => words.text(b, JigPhrase::ClosingStageEffects, &[]),
                ClosingStage::Releases => words.text(b, JigPhrase::ClosingReleases, &[]),
                ClosingStage::Settled => words.text(b, JigPhrase::ClosingSettled, &[]),
            }
        }
        ChipPhase::Held { reason, revision } => {
            words.text(b, JigPhrase::Word(Word::Phase(Phase::Held)), &[]);
            crate::tree::fragments::chip::reason(b, *reason, words);
            words.text(b, JigPhrase::Revision, &[Hole::Number(*revision)]);
        }
        ChipPhase::Ended => words.text(b, JigPhrase::Word(Word::Phase(Phase::Ended)), &[]),
    }
}
