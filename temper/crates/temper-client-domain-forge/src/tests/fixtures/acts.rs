//! Task and person-task builders.
use super::{PARTY, chat};
use jig_client::contract::*;
use skein_lib::List;
const TEXT_BYTES: u32 = 32;
const ONE_OFFER: u32 = 1;
const EMPTY_LIST: u32 = 0;
pub const TASK: u64 = 9;
pub const MESSAGE: u64 = 3;
pub const REVISION: Revision = Revision { value: 7 };
pub const PROPOSAL: ProposalKey = ProposalKey { proposer: Proposer::Task(TASK), number: MESSAGE };
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
pub fn spec() -> SpecificationDocument {
    SpecificationDocument::complete(Specification {
        asks: false,
        words: text(b"Task"),
        parameters: List::with_capacity(EMPTY_LIST),
        inputs: List::with_capacity(EMPTY_LIST),
    })
}
pub fn person(act: ActKind) -> ReadValue {
    ReadValue::PersonTask(Box::new(PersonTask {
        task: chip(),
        revision: REVISION,
        requester: Requester::Person(PARTY),
        spec: spec(),
        address: PersonAddress::Party(PARTY),
        taken_by: None,
        since: None,
        stall: None,
        deadline: None,
        contract: TaskContractDocument::Report { words: 32 },
        results: List::with_capacity(0),
        cut: false,
        acts: offered(act, Admission::Allowed),
    }))
}
