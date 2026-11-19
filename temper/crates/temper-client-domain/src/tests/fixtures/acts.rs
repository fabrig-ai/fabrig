use jig_client_protocol_wire::Record as _;
use super::*;
use crate::pages::{self, Act, Pages, inbox::Card};
use jig_client::contract::*;
const TEXT_BYTES: u32 = 32;
const ONE_OFFER: u32 = 1;
const EMPTY_LIST: u32 = 0;
const OUTPUTS: u32 = 32;
const SENDS: u32 = 4;
const AUTH: [u8; 40] = [7; 40];
const STREAM: [u8; 16] = [7; 16];
const NO_NEEDS: u32 = 0;
const NO_UPDATES: u32 = 0;
const RESULTS_SEEN: u64 = 0;
pub const TASK: u64 = 9;
pub const MESSAGE: u64 = 3;
pub const REVISION: Revision = Revision { value: 7 };
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
pub fn proposal(act: ActKind, admission: Admission) -> ReadValue {
    ReadValue::Proposal(Box::new(Proposal {
        projects: List::with_capacity(0),
        key: PROPOSAL,
        project: PROJECT,
        reason: DocumentText::complete(text(b"Make a goal")),
        action: ProposalAction::Batch(List::with_capacity(EMPTY_LIST)),
        as_holder: true,
        waiting: ProposalWaiting { holder: HistoryHolder::Person(PARTY), since: Wall::EPOCH },
        funding: List::with_capacity(EMPTY_LIST),
        acts: offered(act, admission),
    }))
}
pub fn amendment(proposer: Proposer) -> ReadValue {
    let task = chat::task();
    let before = AmendmentState {
        addressed: None,
        taken_by: None,
        instructions: None,
        spec: task.spec,
        authority: task.authority,
        wake: task.wake,
        dependencies: List::uncapped(),
        cut: false,
    };
    let mut after = before.clone();
    after.spec = spec();
    after.authority.authority.spend = 150;
    let ReadValue::Proposal(mut proposal) = proposal(ActKind::Accept, Admission::Allowed) else {
        panic!("proposal fixture")
    };
    proposal.key.proposer = proposer;
    proposal.action = ProposalAction::Amend {
        task: TaskLink::Live(task.chip),
        addressed: None,
        before: Box::new(before),
        after: Box::new(after),
        reason: DocumentText::complete(text(b"Widen this task")),
        release: None,
    };
    proposal.acts = List::from_boxed(Box::new([
        Offered { act: ActKind::Accept, admission: Admission::Allowed, effect: None },
        Offered { act: ActKind::Reject, admission: Admission::Allowed, effect: None },
        Offered { act: ActKind::Withdraw, admission: Admission::Allowed, effect: None },
    ]));
    ReadValue::Proposal(proposal)
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
pub fn pages(object: ObjectKey, page: ReadValue) -> Pages {
    let limits = limits();
    let mut pages = Pages::new();
    let mut out = Queue::with_capacity(OUTPUTS);
    pages::step(
        &mut pages,
        &env(&limits),
        crate::unlinked(&limits, SEED).client(),
        pages::Event::Went { address: Address::Inbox { filter: crate::address::Filter::ALL } },
        &mut out,
    );
    let inbox = pages.inbox.as_mut().expect("inbox");
    inbox
        .cards
        .push(Card {
            withdraw_unread: false,
            whole: None,
            task: None,
            object,
            group: InboxGroup::Proposals,
            project: Some(PROJECT),
            position: None,
            unread: None,
            handle: jig_client::Handle { asker: crate::owner::Owner::Pages.asker(), number: MESSAGE },
            page: Some(page),
            words: None,
            key: None,
            outcome: None,
        })
        .expect("one card");
    pages
}
pub fn act(pages: &mut Pages, act: Act) -> Queue<pages::Request> {
    let limits = limits();
    let mut out = Queue::with_capacity(OUTPUTS);
    pages::step(pages, &env(&limits), crate::unlinked(&limits, SEED).client(), pages::Event::Act { act }, &mut out);
    out
}
pub fn decide(object: ObjectKey, act: ActKind) -> Act {
    Act::Decide { object, act, revision: Some(REVISION) }
}
pub fn sent(out: &mut Queue<pages::Request>) -> Box<[Ask]> {
    let mut sent = List::with_capacity(SENDS);
    while let Some(request) = out.pop() {
        if let pages::Request::Client(jig_client::Event::Send { request: jig_client::Ask::Jig(ask), .. }) = request {
            sent.push(*ask).expect("bounded sends");
        }
    }
    sent.into_boxed()
}
const HEARTBEAT: Duration = Duration::from_millis(100);
const RETENTION: Duration = Duration::from_secs(60);
pub fn live_client() -> jig_client::Client {
    use jig_client_protocol_wire as w;
    use skein_lib::{Secret, Writer};
    let mut client = jig_client::Client::new(&limits().client, SEED).expect("client");
    let mut out = Queue::uncapped();
    client.step(
        Time::ZERO,
        Wall::EPOCH,
        jig_client::Event::SignIn { with: jig_client::With::Held(Secret::new(Box::new(AUTH))) },
        &mut out,
    );
    client.settle(Time::ZERO, Wall::EPOCH, &mut out);
    let served = w::Served {
            owner: w::Owner::Jig,
            first: jig_client_protocol_wire::VERSION,
            last: jig_client_protocol_wire::VERSION,
        };
    let mut rows = List::with_capacity(ONE_OFFER);
    rows.push(served).expect("one version");
    let hello = jig_client_test_records::opened(
        w::Hello { stream: STREAM, served: rows, heartbeat: HEARTBEAT },
        jig_client_test_records::frame_record(),
    );
    let mut bytes = Writer::new(usize::try_from(hello.measure(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture")).expect("hello fits"));
    hello.write(&jig_client_protocol_wire::CEILINGS, &mut bytes).expect("encoded hello");
    client.step(
        Time::ZERO,
        Wall::EPOCH,
        jig_client::Event::Streamed { message: jig_client::Streamed::Record(bytes.finish()) },
        &mut out,
    );
    let empty = w::DocumentText { bytes: Box::from([]), cut: false };
    let party = w::MeParty { number: PARTY, kind: w::PartyKind::Person, login: empty.clone(), name: empty };
    let me = w::Me {
            providers: Box::new([]),
            providers_cut: false,
            key_bytes: 256,
            redirect_bytes: 256,
            words_bytes: 8192,
            party,
            projects: Box::from([]),
            needs: w::Needs {
                    questions: NO_NEEDS,
                    proposals: NO_NEEDS,
                    escalations: NO_NEEDS,
                    person_tasks: NO_NEEDS,
                },
            needs_total: NO_NEEDS,
            updates: w::Count { value: NO_UPDATES, limited: false },
            results_seen: RESULTS_SEEN,
            cut: false,
            retention: RETENTION,
            deployment_role: None,
        };
    let value = w::DeliveryValue::Me(me);
    let mut bytes = Writer::new(usize::try_from(value.measure(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture")).expect("me fits"));
    value.write(&jig_client_protocol_wire::CEILINGS, &mut bytes).expect("encoded me");
    let document = w::Owned { owner: w::Owner::Jig, version: jig_client_protocol_wire::VERSION, record: bytes.finish() };
    let delivery = w::StreamEvent::Delivery(
        w::Delivery { watch: 0, document },
    );
    let mut bytes = Writer::new(usize::try_from(delivery.measure(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture")).expect("delivery fits"));
    delivery.write(&jig_client_protocol_wire::CEILINGS, &mut bytes).expect("encoded delivery");
    client.step(
        Time::ZERO,
        Wall::EPOCH,
        jig_client::Event::Streamed { message: jig_client::Streamed::Record(bytes.finish()) },
        &mut out,
    );
    client.settle(Time::ZERO, Wall::EPOCH, &mut out);
    assert_eq!(client.link().state, jig_client::Link::Live);
    client
}
