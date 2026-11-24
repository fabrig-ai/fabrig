//! Stream and remaining-document fixtures (protocol/client/wire.md, 1, and protocol/client/README.md, 5).
use super::*;

pub fn stream_limits() -> Limits {
    let mut limits = CEILINGS;
    limits.owned_record = 32;
    limits.hello_served = 2;
    limits.run_text_text = 32;
    limits.call_identity_id = 32;
    limits.current_call_tool = 32;
    limits.current_call_input = 32;
    limits.call_tool = 32;
    limits.call_input = 32;
    limits.call_outcome_some = 32;
    check_limits(&limits).expect("stream fixture limits below ceilings");
    limits
}

pub fn wire_family() -> Owner {
    Owner::Family(Family { number: u16::MAX })
}

pub fn served(owner: Owner) -> Served {
    Served { owner, first: u32::MAX, last: u32::MAX }
}

pub fn hello() -> Hello {
    let mut owners = List::with_capacity(2);
    owners.push(served(Owner::Jig)).expect("first owner");
    owners.push(served(wire_family())).expect("second owner");
    Hello { stream: [255; 16], served: owners, heartbeat: skein_lib::Duration::from_nanos(u64::MAX) }
}

pub fn stream_owned(limits: &Limits) -> Owned {
    Owned { owner: wire_family(), version: u32::MAX, record: bytes(limits.owned_record) }
}

pub fn current_call(limits: &Limits) -> CurrentCall {
    let identity = CallIdentity {
        owner: u64::MAX,
        sequence: u32::MAX,
        retry: u32::MAX,
        id: ceiling_text(limits.call_identity_id),
    };
    CurrentCall {
        identity,
        tool: ceiling_text(limits.current_call_tool),
        input: ceiling_text(limits.current_call_input),
        cut: true,
        began: None,
    }
}

pub fn decoded_call(limits: &Limits) -> Call {
    Call {
        number: u64::MAX,
        tool: ceiling_text(limits.call_tool),
        input: ceiling_text(limits.call_input),
        outcome: Some(ceiling_text(limits.call_outcome_some)),
        failed: true,
        objects: List::with_capacity(0),
        parts: Box::from([]),
    }
}

pub fn raw_live_call(limits: &Limits) -> LiveCall {
    LiveCall::Raw(LiveCallRaw { value: current_call(limits) })
}

pub fn decoded_live_call(limits: &Limits) -> LiveCall {
    LiveCall::Decoded(LiveCallDecoded { value: decoded_call(limits) })
}

pub fn run_text(limits: &Limits, call: LiveCall) -> RunText {
    let turn = ProvisionalTurnName { attempt: u64::MAX, number: u32::MAX, retry: u32::MAX };
    RunText {
        writing: None,
        attempt: u64::MAX,
        began: Some(u64::MAX),
        turn,
        from: u32::MAX,
        through: u32::MAX,
        text: ceiling_text(limits.run_text_text),
        cut: true,
        call: Some(call),
    }
}

pub fn ended(why: WatchEnd) -> WatchEnded {
    WatchEnded { watch: u64::MAX, why }
}

pub fn stream_events(limits: &Limits) -> [StreamEvent; 5] {
    let delivery = Delivery { watch: u64::MAX, document: stream_owned(limits) };
    [
        StreamEvent::Hello(hello()),
        StreamEvent::Delivery(delivery),
        StreamEvent::Text(stream_text(limits, raw_live_call(limits))),
        StreamEvent::Text(stream_text(limits, decoded_live_call(limits))),
        StreamEvent::Ended(ended(WatchEnd::SubjectEnded)),
    ]
}

pub fn tree_subject() -> Subject {
    Subject::Tree(SubjectTree { task: u64::MAX, window: u32::MAX })
}

pub fn open_tree() -> Open {
    let subject = tree_subject();
    let record = encoded(&subject);
    let subject = Owned { owner: Owner::Jig, version: VERSION, record };
    Open { stream: [7; 16], watch: u64::MAX, subject }
}

pub fn waiting_account_state(until: Option<u64>) -> AccountState {
    AccountState::Later(AccountStateLater { until })
}

pub fn account(state: AccountState) -> Account {
    Account {
        list: None,
        facts: List::with_capacity(0),
        provider_name: None,
        manage_usage: None,
        number: u32::MAX,
        place: 0,
        uses: None,
        provider: None,
        display: DocumentText { bytes: Box::from([]), cut: false },
        holder: AccountHolder::Deployment,
        sources: Box::from([]),
        generation: Some(u64::MAX),
        state,
        plan: false,
        usable: true,
        expiring: false,
        attention: true,
        cut: true,
        spent_until: None,
        runs: Box::from([]),
        waiting: Box::from([]),
    }
}

pub fn ended_goal_holding() -> GoalHolding {
    let mut path = List::with_capacity(2);
    path.push(Box::from(b"one/two".as_slice())).expect("first literal segment");
    path.push(Box::from([255, 0])).expect("second opaque segment");
    GoalHolding::Write(GoalHoldingWrite { connector: u16::MAX, path, kind: 0 })
}

pub fn stream_text(limits: &Limits, call: LiveCall) -> StreamText {
    StreamText { watch: 37, value: run_text(limits, call) }
}
