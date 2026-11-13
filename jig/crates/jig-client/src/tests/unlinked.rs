use super::fixtures::{self, PARTY, STOP, TASK};
use crate::*;
use skein_lib::{Duration, List, Queue, Time, Token, Wall};
const ME_WATCH: Token = Token::new(1);
const ENGINE_KEY: skein_lib::RequestKey = [1; 16];
const RETENTION: Duration = Duration::from_secs(60);
struct Client {
    machine: crate::Client,
    out: Queue<UnlinkedRequest>,
    requests: List<UnlinkedRequest>,
}
impl Client {
    fn step(&mut self, event: Event) {
        self.machine.step_unlinked(Time::ZERO, Wall::EPOCH, event, &mut self.out);
        self.collect();
    }
    fn fire(&mut self, now: Time) {
        self.machine.fire_unlinked(now, Wall::EPOCH, &mut self.out);
        self.collect();
    }
    fn collect(&mut self) {
        self.requests = List::with_capacity(self.out.len());
        while let Some(request) = self.out.pop() {
            self.requests.push(request).unwrap();
        }
    }
    fn requests(&self) -> &[UnlinkedRequest] {
        self.requests.as_slice()
    }
}
fn new() -> Client {
    let limits = fixtures::limits();
    Client {
        machine: crate::Client::unlinked(&limits, 7).unwrap(),
        out: Queue::uncapped(),
        requests: List::with_capacity(0),
    }
}
fn client() -> Client {
    let mut c = new();
    c.step(Event::StartUnlinked { kept: List::with_capacity(0) });
    c.step(Event::Up {
        up: Up::Served { served: true, versions: Some(contract::VersionRange { first: 21, last: 21 }) },
    });
    c.step(Event::SignIn { with: With::Transport });
    c
}
fn live() -> Client {
    let mut c = client();
    c.step(Event::Up {
        up: Up::Watched {
            watch: ME_WATCH,
            event: contract::WatchEvent::Delivered(contract::Delivery::Me(Box::new(contract::Me {
                providers: List::with_capacity(0),
                providers_cut: false,
                key_bytes: 256,
                redirect_bytes: 4096,
                words_bytes: 8192,
                party: contract::MeParty {
                    number: PARTY,
                    kind: contract::PartyKind::Person,
                    login: contract::DocumentText::complete(contract::Text::new(Box::new([]), 0).unwrap()),
                    name: contract::DocumentText::complete(contract::Text::new(Box::new([]), 0).unwrap()),
                },
                projects: List::with_capacity(0),
                needs: contract::Needs { questions: 0, proposals: 0, escalations: 0, person_tasks: 0 },
                needs_total: 0,
                updates: contract::Count { value: 0, limited: false },
                results_seen: 0,
                cut: false,
                retention: RETENTION,
                deployment_role: None,
            }))),
        },
    });
    c
}
#[test]
fn an_unlinked_client_gives_its_watch_as_the_domain_s_opening() {
    let c = client();
    let [
        UnlinkedRequest::Told { told: Told::Link { link: Link::Starting } },
        UnlinkedRequest::Down { down: Down::Open { watch, subject } },
    ] = c.requests()
    else {
        panic!("one opening: {:?}", c.requests());
    };
    assert_eq!((*watch, subject), (ME_WATCH, &Watchable::Jig(contract::Subject::Me)));
}
#[test]
fn an_unlinked_client_takes_an_answer_as_the_domain_s() {
    for what in
        [contract::Read::Project { project: fixtures::PROJECT }, contract::Read::Turn { task: 7, attempt: 44, turn: 3 }]
    {
        let mut c = live();
        c.step(Event::Read { handle: TASK, read: Readable::Jig(what) });
        let [UnlinkedRequest::Down { down: Down::Read { read, .. } }] = c.requests() else {
            panic!("one read: {:?}", c.requests());
        };
        let read = *read;
        let page = Readout::Jig(contract::ReadResult::None(contract::ReadNone::Unknown));
        c.step(Event::Up { up: Up::ReadDone { read, result: page.clone() } });
        let [UnlinkedRequest::Told { told: Told::Page { handle, page: told_page } }] = c.requests() else {
            panic!("one page: {:?}", c.requests());
        };
        assert_eq!((*handle, told_page), (TASK, &page));
    }
}
#[test]
fn an_unlinked_client_s_records_are_the_table_s() {
    let mut c = new();
    let record = skein_lib::RequestRecord {
        key: ENGINE_KEY,
        scope: PARTY,
        first_sent: Some(Wall::EPOCH),
        request: Held { key: STOP, request: fixtures::stop() },
    };
    let mut kept = List::with_capacity(1);
    kept.push(record).unwrap();
    c.step(Event::StartUnlinked { kept });
    let [
        UnlinkedRequest::Told { told: Told::Kept { key, request } },
        UnlinkedRequest::Told { told: Told::Pending { key: pending } },
    ] = c.requests()
    else {
        panic!("one restored request: {:?}", c.requests());
    };
    assert_eq!((*key, request, *pending), (STOP, &fixtures::stop(), STOP));
    assert_eq!(c.machine.pending(STOP), Some(Pending { progress: Progress::Parked, party: PARTY, owner: None }));
    assert_eq!(c.machine.link().credential, None);
}

#[test]
fn an_unlinked_sign_out_its_world_loses_keeps_the_party_signed_in() {
    let mut c = live();
    c.step(Event::SignOut);
    assert_eq!(c.requests().len(), 1);
    assert_eq!(c.requests(), &[UnlinkedRequest::SignOut]);
    c.step(Event::SignOutLost);
    let [UnlinkedRequest::Told { told: Told::Link { link } }] = c.requests() else { panic!("link tells the loss") };
    assert_eq!(*link, Link::Live);
    assert_eq!(c.machine.link().party, Some(PARTY));
    assert!(c.machine.link().sign_out_lost);
    c.step(Event::Up { up: Up::SignedIn });
    assert!(!c.machine.link().sign_out_lost);
}

#[test]
fn an_unlinked_authorization_is_lost() {
    let mut c = client();
    c.step(Event::Authorize { query: Box::new([]) });
    assert_eq!(c.requests(), &[UnlinkedRequest::Authorized(Authorized::Lost)]);
}

#[test]
fn an_unlinked_client_fires_a_dropped_watch_s_backoff() {
    let mut c = live();
    c.step(Event::Up { up: Up::WatchEnded { watch: ME_WATCH, end: contract::WatchEnd::Dropped } });
    let until = c.machine.next_deadline().expect("dropped watch schedules a retry");
    c.fire(Time::from_nanos(until.as_nanos() - 1));
    assert_eq!((c.requests()).len(), 0);
    c.fire(until);
    let [UnlinkedRequest::Down { down: Down::Open { watch, subject } }] = c.requests() else {
        panic!("one reopening: {:?}", c.requests());
    };
    assert_ne!(*watch, ME_WATCH);
    assert_eq!(subject, &Watchable::Jig(contract::Subject::Me));
    c.fire(until);
    assert_eq!((c.requests()).len(), 0);
}

#[test]
#[should_panic(expected = "a linked client steps with step")]
fn stepping_a_linked_client_with_the_unlinked_pair_is_a_fault() {
    let mut c = crate::Client::new(&fixtures::limits(), 7).unwrap();
    c.step_unlinked(Time::ZERO, Wall::EPOCH, Event::SignOut, &mut Queue::uncapped());
}

#[test]
#[should_panic(expected = "a linked client steps with step")]
fn firing_a_linked_client_with_the_unlinked_pair_is_a_fault() {
    let mut c = crate::Client::new(&fixtures::limits(), 7).unwrap();
    c.fire_unlinked(Time::ZERO, Wall::EPOCH, &mut Queue::uncapped());
}

#[test]
#[should_panic(expected = "an unlinked client steps with step_unlinked")]
fn stepping_an_unlinked_client_with_the_linked_pair_is_a_fault() {
    let mut c = crate::Client::unlinked(&fixtures::limits(), 7).unwrap();
    c.step(Time::ZERO, Wall::EPOCH, Event::SignOut, &mut Queue::uncapped());
}

#[test]
#[should_panic(expected = "an unlinked client steps with step_unlinked")]
fn firing_an_unlinked_client_with_the_linked_pair_is_a_fault() {
    let mut c = crate::Client::unlinked(&fixtures::limits(), 7).unwrap();
    c.fire(Time::ZERO, Wall::EPOCH, &mut Queue::uncapped());
}

#[test]
fn an_unlinked_approval_is_carried_by_the_host() {
    let code = super::approval::code();
    for (event, act, answer) in [
        (Event::AskApproval, ApprovalAct::Ask, Approval::NotAsked(NotAsked::Refused)),
        (
            Event::ReadApproval { code },
            ApprovalAct::Read { code },
            Approval::Read { code, state: ApprovalState::Unknown },
        ),
        (
            Event::DecideApproval { code, approve: true },
            ApprovalAct::Decide { code, approve: true },
            Approval::Decided { code, decided: Decided::Approved },
        ),
    ] {
        let mut c = new();
        c.step(event);
        assert_eq!(c.requests(), &[UnlinkedRequest::ApprovalAct(act)]);
        let expected = match &answer {
            Approval::NotAsked(why) => Approval::NotAsked(*why),
            Approval::Read { code, state } => Approval::Read { code: *code, state: *state },
            Approval::Decided { code, decided } => Approval::Decided { code: *code, decided: *decided },
            Approval::Asked { secret, code, expires } => {
                Approval::Asked { secret: secret.clone(), code: *code, expires: *expires }
            }
        };
        c.step(Event::ApprovalAnswer { answer });
        assert_eq!(c.requests(), &[UnlinkedRequest::Approval(expected)]);
    }
}
#[test]
#[should_panic(expected = "require an unlinked client")]
fn an_approval_answer_to_a_linked_client_is_a_fault() {
    let mut c = fixtures::Client::new();
    c.step(Event::ApprovalAnswer { answer: Approval::NotAsked(NotAsked::Lost) });
}
