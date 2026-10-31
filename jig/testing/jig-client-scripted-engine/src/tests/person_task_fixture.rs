use jig_client_domain::contract::*;
use skein_lib::{List, Wall, bytes::copy_of};
fn doc(bytes: &[u8]) -> DocumentText {
    DocumentText { bytes: Text::new(copy_of(bytes), 8).expect("fixture text fits"), cut: true }
}
fn chip(n: u64) -> Chip {
    Chip {
        task: n,
        title: doc(b"chip"),
        executor: Executor::Party { party: u64::MAX },
        phase: Phase::Waiting,
        details: Box::new(crate::tests::chip_details(OwnerChipPhase::Waiting {
            dependencies: List::with_capacity(3),
            more: false,
            holds: false,
        })),
    }
}
pub(crate) fn value() -> OwnerPersonTask {
    let mut fields = List::with_capacity(2);
    fields.push(FieldRule { name: copy_of(&[255, 0, 47]), max: u32::MAX }).expect("field fits");
    let mut choices = List::with_capacity(3);
    choices.push(VerdictChoice { code: u32::MAX, words: u32::MAX, followups: u32::MAX, fields }).expect("choice fits");
    let mut result_fields = List::with_capacity(3);
    result_fields.push(ResultField { name: copy_of(b"field"), value: copy_of(&[255, 0]) }).expect("result field fits");
    let result = Result {
        task: chip(2),
        ending: Ending::Done,
        value: Some(ResultValue::OwnerVerdict { verdict: u32::MAX, words: doc(b"why"), fields: result_fields }),
        at: None,
        details: None,
        acts: List::with_capacity(3),
    };
    let mut results = List::with_capacity(3);
    results.push(OwnerShownResult { task: OwnerTaskLink::Live(chip(2)), result: None }).expect("missing result fits");
    results
        .push(OwnerShownResult {
            task: OwnerTaskLink::Stored { number: 2, status: None },
            result: Some(Box::new(result)),
        })
        .expect("present result fits");
    OwnerPersonTask {
        task: chip(8),
        revision: Revision { value: u64::MAX },
        requester: OwnerRequester::Deployment { project: u32::MAX },
        spec: SpecificationDocument {
            specification: DocumentSpecification {
                asks: false,
                words: doc(b"spec"),
                parameters: List::with_capacity(2),
                inputs: List::with_capacity(3),
            },
            cut: true,
        },
        address: PersonAddress::Role(u32::MAX),
        taken_by: None,
        since: Some(Wall::EPOCH),
        stall: None,
        deadline: None,
        contract: TaskContractDocument::Verdict { choices, cut: true },
        results,
        cut: true,
        acts: List::with_capacity(3),
    }
}
