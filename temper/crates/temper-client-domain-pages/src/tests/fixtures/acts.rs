//! Documents and requests used by page fixtures.
use super::{PARTY, chat};
use jig_client::contract::*;
use skein_lib::{List, Queue, Wall};
const TEXT_BYTES: u32 = 32;
const ONE_OFFER: u32 = 1;
const SENDS: u32 = 4;
pub const TASK: u64 = 9;
pub const MESSAGE: u64 = 3;
pub const NEW_REVISION: Revision = Revision { value: 8 };
pub const PROPOSAL: ProposalKey = ProposalKey { proposer: Proposer::Task(TASK), number: MESSAGE };
pub const QUESTION: ObjectKey = ObjectKey::Question { task: TASK, message: MESSAGE };
pub fn text(words: &[u8]) -> Text {
    Text::new(Box::from(words), TEXT_BYTES).expect("fixture words fit")
}
pub fn offered(act: ActKind, admission: Admission) -> List<Offered> {
    let mut offers = List::with_capacity(ONE_OFFER);
    offers
        .push(Offered {
            act,
            admission,
            effect: if act == ActKind::Release { Some(Effect::Release(ReleaseEffect::Judged)) } else { None },
        })
        .expect("one offer");
    offers
}
pub fn chip() -> Chip {
    let mut chip = chat::task().chip;
    chip.executor = ChipExecutor::Party { party: PARTY };
    chip.phase = Phase::Held;
    chip
}
pub fn question() -> ReadValue {
    ReadValue::Question(Box::new(Question {
        task: chip(),
        from: None,
        message: MESSAGE,
        words: DocumentText::complete(text(b"Which?")),
        since: Wall::EPOCH,
        acts: offered(ActKind::Answer, Admission::Allowed),
        ending: None,
    }))
}
pub fn held(act: ActKind) -> ReadValue {
    let mut task = chat::task();
    task.hold = Some(Box::new(ChipHold {
        placed: None,
        words: None,
        circle: None,
        account: None,
        source: None,
        reason: HoldCategory::Failures { class: HoldFailure::Agent },
        revision: NEW_REVISION.value,
        before: PriorPhase::Active(RunStage::Running { attempt: 1 }),
        escalation: None,
        tries: FailureTries { transient: 0, permanent: 0, run: 0, agent: 0, lost: 0, invalid: 0, infrastructure: 0 },
        hold_wait_since: None,
    }));
    ReadValue::Escalation(Box::new(Escalation { task: Box::new(task), acts: offered(act, Admission::Allowed) }))
}
pub fn sent(out: &mut Queue<crate::pages::Request>) -> Box<[Ask]> {
    let mut sent = List::with_capacity(SENDS);
    while let Some(request) = out.pop() {
        if let crate::pages::Request::Client(jig_client::Event::Send { request: jig_client::Ask::Jig(ask), .. }) =
            request
        {
            sent.push(*ask).expect("bounded sends");
        }
    }
    sent.into_boxed()
}
