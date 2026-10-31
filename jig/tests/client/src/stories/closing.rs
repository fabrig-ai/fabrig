//! A close confirmation disposes of each live delegate in one request.
use super::*;
pub const CHAT: u64 = 9;
pub const CANCELLED: u64 = 10;
pub const MOVED: u64 = 11;
pub const PARTY: u64 = 7;
const PROJECT: u32 = 1;
const TAKEOVER_BUDGET: u64 = 1;
const OPERATION_COUNT: u32 = 3;
const NO_MESSAGE: u64 = 0;
#[must_use]
pub fn world(seed: u64) -> World<Test, Scripted> {
    let mut world = super::world(seed);
    let mut chat = super::conversations::chat(CHAT, b"Chat nine", Wall::EPOCH);
    chat.acts = list([Offered { effect: None, act: ActKind::Cancel, admission: Admission::Allowed }]);
    let mut cancelled = task();
    cancelled.chip.task = CANCELLED;
    cancelled.chip.title = DocumentText::complete(text(b"Cancelled delegate"));
    cancelled.requester = OwnerRequester::Task(CHAT);
    let mut moved = task();
    moved.chip.task = MOVED;
    moved.chip.title = DocumentText::complete(text(b"Moved delegate"));
    moved.requester = OwnerRequester::Task(CHAT);
    chat.delegates = list([
        LiveDelegate { chip: cancelled.chip.clone(), takeover_budget: TAKEOVER_BUDGET },
        LiveDelegate { chip: moved.chip.clone(), takeover_budget: TAKEOVER_BUDGET },
    ]);
    let ChipPhase::Owner { details, .. } = chat.chip.details.as_mut() else { unreachable!("owner chat") };
    details.live_delegates = 2;
    for value in [chat.clone(), cancelled.clone(), moved.clone()] {
        world
            .front()
            .engine
            .act(
                skein_lib::Time::ZERO,
                Wall::EPOCH,
                jig_client_scripted_engine::Change::Object {
                    project: PROJECT,
                    key: ObjectKey::Task(value.chip.task),
                    value: ReadValue::Task(Box::new(value)),
                },
            )
            .expect("close fixture fits");
    }
    let reason = text(b"finished");
    let ask = Ask::Compound {
        project: PROJECT,
        operations: list([
            Operation::Cancel { project: PROJECT, task: CANCELLED, reason: reason.clone(), grace: None },
            Operation::Move { project: PROJECT, task: MOVED, reason: reason.clone() },
            Operation::Cancel { project: PROJECT, task: CHAT, reason: reason.clone(), grace: None },
        ]),
    };
    cancelled.chip.phase = Phase::Ended;
    cancelled.result = Some(TaskResult::Ending(Box::new(OwnerChipEnding::Cancelled {
        reason: DocumentText::complete(reason),
        result: None,
        by: OwnerRequester::Person(PARTY),
        at: Wall::EPOCH,
    })));
    let ChipPhase::Owner { phase, details } = cancelled.chip.details.as_mut() else { unreachable!("owner delegate") };
    *phase = OwnerChipPhase::Ended;
    let Some(TaskResult::Ending(ending)) = &cancelled.result else { unreachable!("cancelled delegate") };
    details.ending = Some(*ending.clone());
    details.ended_at = Some(Wall::EPOCH);
    moved.requester = OwnerRequester::Person(PARTY);
    chat.chip.phase = Phase::Closing;
    let ChipPhase::Owner { phase, details } = chat.chip.details.as_mut() else { unreachable!("owner chat") };
    *phase = OwnerChipPhase::Closing(ClosingStage::Delegates);
    details.live_delegates = 0;
    chat.delegates.clear();
    world
        .front()
        .engine
        .script(jig_client_scripted_engine::Rule {
            party: Some(PARTY),
            ask,
            answer: Answer::Done(Outcome::Owner(OwnerOutcome::Compound {
                project: PROJECT,
                operations: OPERATION_COUNT,
                first_message: NO_MESSAGE,
            })),
            changes: [chat, cancelled, moved]
                .into_iter()
                .map(|value| jig_client_scripted_engine::Change::Object {
                    project: PROJECT,
                    key: ObjectKey::Task(value.chip.task),
                    value: ReadValue::Task(Box::new(value)),
                })
                .collect(),
        })
        .expect("close rule fits");
    world
}
