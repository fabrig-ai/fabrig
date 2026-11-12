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
            whole: None,
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
    use jig_client_protocol_wire::v20 as w;
    use skein_lib::{Reader, Secret, Writer};
    let mut client = jig_client::Client::new(&limits().client, SEED).expect("client");
    let mut out = Queue::uncapped();
    client.step(
        Time::ZERO,
        Wall::EPOCH,
        jig_client::Event::SignIn { with: jig_client::With::Held(Secret::new(Box::new(AUTH))) },
        &mut out,
    );
    let served = w::Served::new(
        &w::CEILINGS,
        w::ServedParts {
            owner: w::Owner::Jig,
            first: jig_client_protocol_wire::VERSION,
            last: jig_client_protocol_wire::VERSION,
        },
    )
    .expect("served");
    let mut rows = List::with_capacity(ONE_OFFER);
    rows.push(served).expect("one version");
    let hello = w::StreamEvent::Hello(
        w::Hello::new(&w::CEILINGS, w::HelloParts { stream: STREAM, served: rows, heartbeat: HEARTBEAT })
            .expect("hello"),
    );
    let mut bytes = Writer::new(usize::try_from(hello.measure()).expect("hello fits"));
    hello.encode(&mut bytes).expect("encoded hello");
    client.step(
        Time::ZERO,
        Wall::EPOCH,
        jig_client::Event::Streamed { message: jig_client::Streamed::Record(bytes.finish()) },
        &mut out,
    );
    let mut watch = None;
    while let Some(request) = out.pop() {
        if let jig_client::Request::Exchange { message: jig_client::Message::Watch(bytes), .. } = request {
            watch = Some(w::Open::decode(&w::CEILINGS, &mut Reader::new(&bytes)).expect("watch").watch());
        }
    }
    let empty = w::DocumentText::new(&w::CEILINGS, w::DocumentTextParts { bytes: Box::from([]), cut: false })
        .expect("empty words");
    let party = w::MeParty::new(
        &w::CEILINGS,
        w::MePartyParts { number: PARTY, kind: w::PartyKind::Person, login: empty.clone(), name: empty },
    )
    .expect("party");
    let me = w::Me::new(
        &w::CEILINGS,
        w::MeParts {
            providers: Box::new([]),
            providers_cut: false,
            key_bytes: 256,
            redirect_bytes: 256,
            words_bytes: 8192,
            party,
            projects: Box::from([]),
            needs: w::Needs::new(
                &w::CEILINGS,
                w::NeedsParts {
                    questions: NO_NEEDS,
                    proposals: NO_NEEDS,
                    escalations: NO_NEEDS,
                    person_tasks: NO_NEEDS,
                },
            )
            .expect("needs"),
            needs_total: NO_NEEDS,
            updates: w::Count::new(&w::CEILINGS, w::CountParts { value: NO_UPDATES, limited: false }).expect("count"),
            results_seen: RESULTS_SEEN,
            cut: false,
            retention: RETENTION,
            deployment_role: None,
        },
    )
    .expect("me");
    let value = w::DeliveryValue::Me(me);
    let mut bytes = Writer::new(usize::try_from(value.measure()).expect("me fits"));
    value.encode(&mut bytes).expect("encoded me");
    let document = w::Owned::new(
        &w::CEILINGS,
        w::OwnedParts { owner: w::Owner::Jig, version: jig_client_protocol_wire::VERSION, record: bytes.finish() },
    )
    .expect("owned");
    let delivery = w::StreamEvent::Delivery(
        w::Delivery::new(&w::CEILINGS, w::DeliveryParts { watch: watch.expect("me watch"), document })
            .expect("delivery"),
    );
    let mut bytes = Writer::new(usize::try_from(delivery.measure()).expect("delivery fits"));
    delivery.encode(&mut bytes).expect("encoded delivery");
    client.step(
        Time::ZERO,
        Wall::EPOCH,
        jig_client::Event::Streamed { message: jig_client::Streamed::Record(bytes.finish()) },
        &mut out,
    );
    assert_eq!(client.link().state, jig_client::Link::Live);
    client
}
