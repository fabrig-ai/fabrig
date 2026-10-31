#![expect(
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    clippy::disallowed_types,
    reason = "programming-model.md, section 10.2: deterministic test harnesses are ordinary Rust"
)]
use super::{
    Answer, Ask, AskRefusal, Charter, EXPIRY, Event, FieldRule, Harness, Invalid, LIMITS, Limits, Refusal, Request,
    Returned, Token, agents, bytes, charter, families,
};
use alloc::boxed::Box;
fn large_host_charter(instructions: &[u8]) -> Charter {
    let mut selected = charter();
    selected.grants.host_tools = Box::default();
    selected.instructions = bytes(instructions);
    selected.outcome.change = None;
    selected.grants.deliver = None;
    selected.outcome.report = Some(crate::outcome::TextSpec {
        max: 32,
        fields: (0_u32..20)
            .map(|index| FieldRule { name: format!("long_contract_field_{index:02}").into_bytes().into(), max: 32 })
            .collect::<Vec<_>>()
            .into_boxed_slice(),
    });
    selected
}

#[test]
fn charter_rendering_at_the_host_edge_is_admitted_and_one_byte_more_refuses() {
    let selected = large_host_charter(b"base");
    let receiving = Limits { run_bytes: 16_384, outcome_bytes: 8192, ..LIMITS };
    let edge = crate::prompt::host_bytes(&selected, None, receiving.guide_bytes).expect("rendered host")
        + receiving.schema_costs.finish(&selected.outcome).expect("schema")
        + receiving.schema_costs.finish_text;
    let receiving = Limits { host_part: edge, ..receiving };
    let mut admitted = Harness::new(receiving);
    let requests = admitted.start_workspace(1, selected, None);
    let Some(Request::Admitted { .. }) = requests.first() else { panic!("exact edge: {requests:?}") };
    let selected = large_host_charter(b"basex");
    let mut refused = Harness::new(receiving);
    let requests = refused.start_workspace(1, selected, None);
    let [Request::Answer { answer: Answer::Refused(Refusal::Invalid(Invalid::HostPart { bound })), .. }] =
        &requests[..]
    else {
        panic!("one byte over: {requests:?}")
    };
    assert_eq!(*bound, edge);
}

#[test]
fn a_charter_model_below_the_derived_room_is_refused_before_preparation() {
    let selected = charter();
    let [numerator, denominator] = LIMITS.compaction_threshold;
    let room = u64::from(selected.llm.window) * u64::from(numerator) / u64::from(denominator)
        - 2 * u64::from(selected.llm.output);
    let receiving = Limits { room: room + 1, ..LIMITS };
    let mut refused = Harness::new(receiving);
    let requests = refused.start(1, selected);
    let [Request::Answer { answer: Answer::Refused(Refusal::Invalid(Invalid::Room { bound })), .. }] = &requests[..]
    else {
        panic!("room refusal: {requests:?}")
    };
    assert_eq!(*bound, room + 1);
}

#[test]
fn a_sub_agent_brief_and_its_inherited_context_must_fit_the_host_part_before_any_open() {
    for extra in [0, 1] {
        let bound = 10_000_u64;
        let receiving = Limits { host_part: bound, ..LIMITS };
        let mut harness = Harness::new(receiving);
        let (_, main) = harness.running_on(1, 100, agents());
        let requests = harness.step(Event::Delegated {
            name: crate::CallName { activation: 1, completion: 1, position: 0 },
            conversation: main,
            call: Token::new(7),
            ask: Ask::SubAgent {
                brief: vec![b'x'; usize::try_from(bound + extra).expect("small test bound")].into(),
                families: families(true, false, false),
                llm: None,
                share: crate::Share::UNASKED,
            },
            deadline: EXPIRY,
        });
        let [
            Request::Return {
                call,
                result: Returned::Refused { refusal: AskRefusal::TooLarge { bound: seen } },
                spent: 0,
            },
        ] = &requests[..]
        else {
            panic!("no child opens: {requests:?}")
        };
        assert_eq!(*call, Token::new(7));
        assert_eq!(*seen, bound);
    }
}
