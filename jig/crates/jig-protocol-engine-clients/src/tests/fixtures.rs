#[path = "fixtures/create.rs"]
mod create;
pub use create::*;
#[path = "fixtures/operations.rs"]
mod operations;
pub use operations::*;
#[path = "fixtures/envelope.rs"]
mod envelope;
use super::super::*;
pub use envelope::*;
use skein_lib::{Duration, List, Secret, Wall, Writer};
pub const NUMBER: u64 = 41;
pub const SIGN_IN_EXPIRES: Wall = Wall::from_nanos(86_400_000_000_000);
// Keep the old intake's 256-byte cap, enough for these small message records.
pub const MESSAGE_BYTES: u32 = 256;
pub fn message_bytes(size: usize) -> Box<[u8]> {
    let mut bytes = Writer::new(size);
    for _ in 0..size {
        bytes.put(&[0]).expect("bounded bytes");
    }
    bytes.finish()
}
pub fn wire() -> w::Limits {
    w::Limits {
        keyed_request: 256,
        owned_record: 256,
        ending_rows: 128,
        message_words: 64,
        amend_reason: 64,
        ..w::CEILINGS
    }
}
pub fn settings() -> Settings {
    let mut served = List::with_capacity(2);
    served
        .push(crate::Served {
            family: None,
            version: jig_client_protocol_wire::VERSION,
            layout: jig_client_protocol_wire::LAYOUT,
        })
        .expect("one jig range");
    served.push(crate::Served { family: Some(9), version: 2, layout: [1; 32] }).expect("one family range");
    Settings {
        private: crate::Private::none(),
        provider: None,
        addresses: Box::new([]),
        deployment_provider: 2,
        link: None,
        link_secret: None,
        local_secret: None,
        approval: false,
        heartbeat: Duration::from_secs(5),
        rows: 32,
        served,
        providers: List::with_capacity(0),
        seed: Secret::new(Box::new([7; 32])),
        stream: 31,
        models: models(),
        key_bytes: 7934,
        redirect_bytes: 4096,
        moved_bytes: 70000,
        public: List::with_capacity(0),
        public_bounds: jig_client_protocol_wire::PublicIndexBounds {
            goals: 0,
            ended: 0,
            lines: 0,
            depth: 0,
            projects: 0,
        },
    }
}
/// Two configured models: one with efforts, provider, cache-write price and description,
/// one with none of those optional values.
pub fn models() -> Box<[crate::Model]> {
    Box::new([
        crate::Model {
            new_until: None,
            number: 1,
            source: jig_domain_core_vocabulary::settings::ModelSource::Stock,
            estimated: false,
            effort: Some(Box::from(b"low".as_slice())),
            shown: None,
            picker: true,
            name: Box::from(b"careful".as_slice()),
            provider: Some(3),
            window: 200_000,
            output: 32_000,
            efforts: Box::new([Box::from(b"low".as_slice()), Box::from(b"high".as_slice())]),
            prices: crate::Prices { input: 3, cached: 1, cache_write: Some(5), output: 15, unit: 1_000_000 },
            description: Some(Box::from(b"For careful work".as_slice())),
        },
        crate::Model {
            new_until: None,
            number: 1,
            source: jig_domain_core_vocabulary::settings::ModelSource::Stock,
            estimated: false,
            effort: None,
            shown: None,
            picker: true,
            name: Box::from(b"quick".as_slice()),
            provider: None,
            window: 8192,
            output: 1024,
            efforts: Box::new([]),
            prices: crate::Prices { input: 1, cached: 1, cache_write: None, output: 2, unit: 1000 },
            description: None,
        },
    ])
}
pub fn env() -> Env<Limits> {
    Env {
        now: Time::ZERO,
        wall: Wall::EPOCH,
        limits: Limits {
            public: crate::PublicLimits::OFF,
            streams: 2,
            watches_per_stream: 4,
            queued_bytes: 65536,
            identity_bytes: 64,
            families: 2,
            providers: 0,
            mints: 2,
            provider_codes: 16,
            provider_tokens: 64,
            exchanges: 4,
            exchanges_per_client: 2,
            reads_per_client: 32,
            openings_per_client: 31,
            message_bytes: MESSAGE_BYTES,
            wire: wire(),
            link_span: Duration::from_secs(300),
        },
    }
}
pub fn start() -> w::Ask {
    w::Ask::Message(w::Message { project: 3, task: 8, words: Box::from(b"hello".as_slice()) })
}
pub fn revision(value: u64) -> w::Revision {
    w::Revision { value }
}
pub fn amendment(release: Option<w::Revision>, reason: &[u8]) -> w::Ask {
    let value = Box::new(w::Amend {
        project: 3,
        task: 8,
        revision: Some(revision(6)),
        reason: Box::from(reason),
        change: w::Amendment {
            llm: None,
            tracked: None,
            addressed: None,
            spec: None,
            wake: None,
            dependencies: Some(List::with_capacity(0)),
            authority: None,
            instructions: Some(Box::new([])),
        },
    });
    match release {
        Some(revision) => compound(
            3,
            &[
                w::Operation::Amend(value),
                w::Operation::Release(w::Release { project: 3, task: 8, revision, again: None }),
            ],
        ),
        None => w::Ask::Amend(value),
    }
}
pub struct Exchange {
    pub component: Component,
    pub env: Env<Limits>,
    pub out: Queue<Request>,
    pub token: Box<[u8]>,
    pub via: crate::Via,
    pub source: Option<::core::net::IpAddr>,
}
impl Exchange {
    pub fn new() -> Self {
        Self::with_env(env())
    }
    pub fn with_env(env: Env<Limits>) -> Self {
        Self::configured(env, settings())
    }
    pub fn configured(env: Env<Limits>, settings: Settings) -> Self {
        let mut out = Queue::with_capacity(8);
        let component = Component::new(settings, &env, &mut out).expect("starts");
        out.pop();
        Self { component, env, out, token: sign_in_token(NUMBER), via: crate::Via::Listener, source: None }
    }
    pub fn send(&mut self, message: crate::Message) -> Token {
        let owner = self.component.name();
        self.component.down(
            &self.env,
            Event::Transport(crate::FromTransport::Exchange {
                via: self.via,
                source: self.source,
                browser: false,
                exchange: owner,
                credential: Some(Secret::new(self.token.clone())),
                message,
            }),
            &mut self.out,
        );
        owner
    }
    pub fn stream_open(&mut self) -> Token {
        self.stream_open_version(None)
    }
    pub fn stream_open_version(&mut self, version: Option<u32>) -> Token {
        let owner = self.component.name();
        self.component.down(
            &self.env,
            Event::Transport(crate::FromTransport::StreamOpen {
                version,
                stream: owner,
                replaces: None,
                credential: Some(Secret::new(self.token.clone())),
            }),
            &mut self.out,
        );
        owner
    }
    pub fn authenticate(&mut self) {
        self.authenticate_until(SIGN_IN_EXPIRES);
    }
    pub fn authenticate_until(&mut self, expires: Wall) {
        self.authenticate_sign_in_until(NUMBER, expires);
    }
    pub fn authenticate_sign_in_until(&mut self, sign_in: u64, expires: Wall) {
        let Some(Request::Secrets(SecretsRequest::Get { owner })) = self.out.pop() else {
            panic!("digest get");
        };
        assert_eq!(owner, SecretOwner::SignIn(sign_in));
        self.component.down(
            &self.env,
            Event::Secrets(SecretsAnswer::Got {
                owner,
                value: Some(Kept { digest: sign_in::digest(&self.token), expires }),
            }),
            &mut self.out,
        );
        assert!(self.component.up(&self.env, &mut self.out));
    }
    pub fn response(&self, owner: Token) -> (u16, &[u8]) {
        let connection = self.component.connections.get(&owner).expect("connection");
        match connection.response.as_ref() {
            Some(crate::Reply::Record(body)) => (200, body),
            Some(crate::Reply::NotARecord) => (400, &[]),
            Some(crate::Reply::Conflict) => (409, &[]),
            Some(crate::Reply::StreamsFull { .. }) => panic!("inspect the typed stream refusal"),
            None => match connection.ended {
                Some(crate::Ended::SignedOut) => (401, &[]),
                Some(crate::Ended::Busy | crate::Ended::Stopping) => (409, &[]),
                None => panic!("response"),
            },
        }
    }
}

pub fn sign_in_token(sign_in: u64) -> Box<[u8]> {
    let mut token = Writer::new(40);
    token.put(&sign_in.to_be_bytes()).expect("number");
    token.put(&[7; 32]).expect("random bytes");
    token.finish()
}

pub fn other_asks() -> [(w::Ask, people::Ask); 17] {
    let mut holdings = List::with_capacity(1);
    holdings
        .push(w::Holding { party: 7, role: w::HeldRole::Policy(w::HeldRolePolicy { role: u32::MAX }) })
        .expect("room");
    let mut references = List::with_capacity(1);
    references.push(u64::MAX).expect("room");
    [
        write_case(),
        answer_question_case(),
        take_case(),
        hand_back_case(),
        release_case(),
        stop_case(),
        cancel_case(),
        take_over_case(),
        set_pool_case(),
        mark_seen_case(),
        answer_person_case(),
        decide_proposal_case(),
        set_roles_case(holdings),
        make_service_case(),
        change_policy_case(),
        delete_note_case(),
        edit_note_case(references),
    ]
}
fn write_case() -> (w::Ask, people::Ask) {
    use jig_domain_people as p;
    (
        w::Ask::Message(w::Message { project: 3, task: 8, words: Box::from(b"say".as_slice()) }),
        p::Ask::Say { project: 3, task: 8, words: Box::from(b"say".as_slice()) },
    )
}
fn answer_question_case() -> (w::Ask, people::Ask) {
    use jig_domain_people as p;
    (
        answer(w::AnswerOn::Question(w::AnswerQuestion {
            task: 8,
            message: 19,
            words: Box::from(b"answer".as_slice()),
        })),
        p::Ask::AnswerQuestion { project: 3, task: 8, question: 19, words: Box::from(b"answer".as_slice()) },
    )
}
fn take_case() -> (w::Ask, people::Ask) {
    use jig_domain_people as p;
    (w::Ask::Take(w::Take { project: 3, task: 8 }), p::Ask::TakePerson { project: 3, task: 8 })
}
fn hand_back_case() -> (w::Ask, people::Ask) {
    use jig_domain_people as p;
    (w::Ask::HandBack(w::HandBack { project: 3, task: 8 }), p::Ask::HandBackPerson { project: 3, task: 8 })
}
fn release_case() -> (w::Ask, people::Ask) {
    use jig_domain_people as p;
    (
        w::Ask::Release(w::Release { project: 3, task: 8, revision: revision(6), again: Some(false) }),
        p::Ask::Release { project: 3, task: 8, revision: 6, again: Some(false) },
    )
}
fn stop_case() -> (w::Ask, people::Ask) {
    use jig_domain_people as p;
    (
        w::Ask::Stop(w::Stop { project: 3, task: 8, reason: Box::from(b"Parking the task".as_slice()) }),
        p::Ask::Stop { project: 3, task: 8, reason: Box::from(b"Parking the task".as_slice()) },
    )
}
fn cancel_case() -> (w::Ask, people::Ask) {
    use jig_domain_people as p;
    (
        w::Ask::Cancel(w::Cancel { grace: None, project: 3, task: 8, reason: Box::from(b"cancel".as_slice()) }),
        p::Ask::Cancel { project: 3, task: 8, reason: Box::from(b"cancel".as_slice()), grace: None },
    )
}
fn take_over_case() -> (w::Ask, people::Ask) {
    use jig_domain_people as p;
    (
        w::Ask::MoveTask(w::Move { project: 3, task: 8, reason: Box::from(b"take".as_slice()) }),
        p::Ask::Move { project: 3, task: 8, reason: Box::from(b"take".as_slice()) },
    )
}
fn set_pool_case() -> (w::Ask, people::Ask) {
    use jig_domain_people as p;
    (
        w::Ask::SetPool(w::SetPool { project: 3, party: u64::MAX, budget: u64::MAX }),
        p::Ask::SetPool { project: 3, person: u64::MAX, budget: u64::MAX },
    )
}
fn mark_seen_case() -> (w::Ask, people::Ask) {
    use jig_domain_people as p;
    (
        w::Ask::Seen(w::Seen {
            through: w::SeenThrough::Conversation(w::SeenConversation { task: 8, through: u64::MAX }),
        }),
        p::Ask::MarkSeen { seen: p::Seen::Conversation { task: 8, through: u64::MAX } },
    )
}
fn answer_person_case() -> (w::Ask, people::Ask) {
    use jig_domain_people as p;
    (
        answer(w::AnswerOn::Person(w::AnswerPerson {
            task: 8,
            revision: revision(6),
            answer: w::PersonAnswer::Approved(w::PersonAnswerApproved {
                state: Box::new([255, 0]),
                words: Some(Box::new([])),
            }),
        })),
        p::Ask::AnswerPerson {
            project: 3,
            task: 8,
            revision: 6,
            result: p::PersonResult::Approved { state: Box::new([255, 0]), words: Some(Box::new([])) },
        },
    )
}
fn decide_proposal_case() -> (w::Ask, people::Ask) {
    use jig_domain_people as p;
    (
        decide(w::DecideOn::Proposal(w::DecideProposal {
            proposal: w::ProposalKey { proposer: w::Proposer::Party(w::ProposerParty { value: u64::MAX }), number: 5 },
            decision: w::Decision::Accepted,
            running: None,
        })),
        p::Ask::DecideProposal {
            project: 3,
            proposer: p::Proposer::Party(u64::MAX),
            proposal: 5,
            decision: p::ProposalDecision::Accept,
            running: None,
        },
    )
}
fn set_roles_case(holdings: List<w::Holding>) -> (w::Ask, people::Ask) {
    use jig_domain_people as p;
    (
        w::Ask::SetRoles(w::SetRoles { project: 3, holdings }),
        p::Ask::SetRoles {
            project: 3,
            holdings: Box::new([p::Holding { person: 7, role: p::Role::Policy { role: u32::MAX } }]),
        },
    )
}
fn make_service_case() -> (w::Ask, people::Ask) {
    use jig_domain_people as p;
    (
        w::Ask::MakeService(w::MakeService {
            project: 3,
            name: Box::from(b"service".as_slice()),
            role: w::HeldRole::Owner,
        }),
        p::Ask::MakeService { project: 3, name: Box::from(b"service".as_slice()), role: p::Role::Owner },
    )
}
fn change_policy_case() -> (w::Ask, people::Ask) {
    use jig_domain_people as p;
    (
        w::Ask::ChangePolicy(w::ChangePolicy {
            project: 3,
            edit: w::PolicyEdit::ProjectSpend(w::PolicyEditProjectSpend { period_spend: u64::MAX }),
        }),
        p::Ask::ChangePolicy { project: 3, change: p::PolicyChange::ProjectSpend { period_spend: u64::MAX } },
    )
}
fn delete_note_case() -> (w::Ask, people::Ask) {
    use jig_domain_people as p;
    (
        w::Ask::DeleteNote(w::DeleteNote {
            project: 3,
            note: 9,
            scope: w::NoteScope::Task(w::NoteScopeTask { project: 3, task: 8 }),
            revision: revision(u64::from(u32::MAX)),
        }),
        p::Ask::EditNote {
            project: 3,
            name: 9,
            scope: Box::new(p::NoteScope::Goal { goal: 8 }),
            change: Box::new(p::NoteChange::Delete { recalled: u32::MAX }),
        },
    )
}
fn edit_note_case(references: List<u64>) -> (w::Ask, people::Ask) {
    use jig_domain_people as p;
    (
        w::Ask::EditNote(w::EditNote {
            project: 3,
            note: 9,
            scope: w::NoteScope::Deployment,
            revision: revision(6),
            description: Box::new([]),
            body: Box::new([]),
            references,
        }),
        p::Ask::EditNote {
            project: 3,
            name: 9,
            scope: Box::new(p::NoteScope::Deployment),
            change: Box::new(p::NoteChange::Correct {
                description: Box::new([]),
                body: Box::new([]),
                references: Box::new([u64::MAX]),
                recalled: 6,
            }),
        },
    )
}

pub fn authority_ask() -> (w::Ask, people::Ask) {
    let pattern = w::Pattern {
        segments: List::with_capacity(0),
        last: w::PatternEnd::Open(w::PatternEndOpen { value: Box::new([255, 0]) }),
    };
    let grant = w::Grant { connector: 5, kind: 6, pattern: pattern.clone() };
    let mut writer = Writer::new(skein_lib::length::usize_of(
        grant.measure(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture"),
    ));
    grant.write(&jig_client_protocol_wire::CEILINGS, &mut writer).expect("grant bytes");
    let bytes = writer.finish();
    let grants = piece(&bytes);
    let resource = w::ResourceScope { connector: 7, pattern };
    let mut writer = Writer::new(skein_lib::length::usize_of(
        resource.measure(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture"),
    ));
    resource.write(&jig_client_protocol_wire::CEILINGS, &mut writer).expect("scope bytes");
    let bytes = writer.finish();
    let note_resources = piece(&bytes);
    let mut kinds = List::with_capacity(1);
    kinds.push(w::AuthorityExecutor::Role(w::AuthorityExecutorRole { value: u32::MAX })).expect("room");
    let authority = w::Authority {
        models: None,
        tools: u64::MAX,
        grants,
        delegation: w::Delegation { kinds, tasks: 9, depth: 10 },
        spend: u64::MAX,
        deadline: Some(99),
        notes: 255,
        note_resources,
    };
    let request = w::Ask::ChangePolicy(w::ChangePolicy {
        project: 3,
        edit: w::PolicyEdit::Role(w::PolicyEditRoleValue {
            value: w::PolicyEditRole { number: 11, authority, period_spend: 12, requests: 13, decides: 14 },
        }),
    });
    (request, expected_authority())
}
fn piece(bytes: &[u8]) -> Box<[u8]> {
    let mut writer = Writer::new(bytes.len().checked_add(4).expect("piece size"));
    skein_codec::piece::write(&mut writer, bytes).expect("piece");
    writer.finish()
}
fn expected_authority() -> people::Ask {
    use jig_domain_people as p;
    p::Ask::ChangePolicy {
        project: 3,
        change: p::PolicyChange::Role(p::PolicyRole {
            number: 11,
            period_spend: 12,
            requests: 13,
            decides: 14,
            authority: p::Authority {
                tools: u64::MAX,
                models: None,
                grants: Box::new([p::Grant {
                    connector: 5,
                    kind: 6,
                    pattern: p::Pattern { segments: Box::new([]), last: p::Last::Open(Box::new([255, 0])) },
                }]),
                delegation: p::Delegation { kinds: Box::new([p::Executor::Role(u32::MAX)]), tasks: 9, depth: 10 },
                spend: u64::MAX,
                deadline: Some(Wall::from_nanos(99)),
                notes: 255,
                note_resources: Box::new([p::ResourceScope {
                    connector: 7,
                    pattern: p::Pattern { segments: Box::new([]), last: p::Last::Open(Box::new([255, 0])) },
                }]),
            },
        }),
    }
}
pub fn malformed_piece() -> w::Ask {
    w::Ask::ChangePolicy(w::ChangePolicy {
        project: 3,
        edit: w::PolicyEdit::Projections(w::PolicyEditProjections { grants: Box::new([0, 0, 0, 2, 0]) }),
    })
}

#[path = "fixtures/answers.rs"]
mod answers;
pub use answers::*;
impl Exchange {
    pub fn ask(&mut self) -> (Token, ReplyTo) {
        self.ask_version(jig_client_protocol_wire::VERSION)
    }
    pub fn ask_version(&mut self, version: u32) -> (Token, ReplyTo) {
        let body = keyed(w::Owner::Jig, version, encode_ask(&start()));
        let owner = self.send(crate::Message::Request(body));
        self.authenticate();
        let Some(Request::Core(event)) = self.out.pop() else {
            panic!("core ask");
        };
        let core::Event::People(people::Event::Ask { reply_to, .. }) = *event else {
            panic!("expected ask");
        };
        (owner, reply_to)
    }
    pub fn reply(&mut self, to: ReplyTo, reply: people::Reply) {
        self.component.down(&self.env, Event::Reply { to, reply }, &mut self.out);
    }
    pub fn answer(&self, owner: Token) -> w::KeyedAnswer {
        let (status, bytes) = self.response(owner);
        assert_eq!(status, 200);
        w::KeyedAnswer::decode(&self.env.limits.wire, &mut skein_lib::Reader::new(bytes)).expect("wire answer")
    }
    pub fn final_answer(&self, owner: Token) -> w::Final {
        final_record(self.answer(owner), &self.env.limits.wire)
    }
    pub fn application(&mut self) -> (Token, ReplyTo) {
        let family = w::Owner::Family(w::Family { number: 9 });
        let owner = self.send(crate::Message::Request(keyed(family, 2, Box::from(b"abc".as_slice()))));
        self.authenticate();
        let Some(Request::Application(crate::ToApplication::Request { reply_to, .. })) = self.out.pop() else {
            panic!("application ask");
        };
        (owner, reply_to)
    }
}

#[path = "fixtures/client.rs"]
mod client;
pub use client::client_limits;

#[path = "fixtures/services.rs"]
mod services;
pub use services::*;

#[path = "fixtures/documents.rs"]
mod documents;
pub use documents::*;

#[path = "fixtures/live.rs"]
mod live;
pub use live::*;

#[path = "fixtures/objects.rs"]
mod objects;
pub use objects::*;

#[path = "fixtures/pages.rs"]
mod pages;
pub use pages::*;

#[path = "fixtures/reads.rs"]
mod reads;
pub use reads::*;
#[path = "fixtures/watches.rs"]
mod watches;
pub use watches::*;
#[path = "fixtures/closing.rs"]
mod closing;
pub use closing::*;

pub fn translated(reply: &people::Reply, limits: &w::Limits) -> w::Final {
    let record = crate::translate::answers::answer(reply, &models(), limits).expect("translated answer");
    final_record(record, limits)
}
pub fn final_record(record: w::KeyedAnswer, limits: &w::Limits) -> w::Final {
    match record {
        w::KeyedAnswer::FinalAnswer(value) => {
            assert_eq!((&value.owner), &w::Owner::Jig);
            assert_eq!(value.version, jig_client_protocol_wire::VERSION);
            w::Final::decode(limits, &mut skein_lib::Reader::new(value.record.as_ref())).expect("jig answer")
        }
        other @ (w::KeyedAnswer::NotServed(_)
        | w::KeyedAnswer::Busy
        | w::KeyedAnswer::AnswersFull(_)
        | w::KeyedAnswer::NotReady
        | w::KeyedAnswer::SignedOut) => {
            panic!("expected final, got {other:?}")
        }
    }
}

impl Exchange {
    pub fn send_as(&mut self, message: crate::Message, browser: bool, credential: Option<Secret>) -> Token {
        let owner = self.component.name();
        self.component.down(
            &self.env,
            Event::Transport(crate::FromTransport::Exchange {
                exchange: owner,
                credential,
                browser,
                via: self.via,
                source: self.source,
                message,
            }),
            &mut self.out,
        );
        owner
    }
    pub fn approval_whose(&mut self) -> ReplyTo {
        let Some(Request::Core(event)) = self.out.pop() else { panic!("Whose event") };
        let core::Event::People(people::Event::Whose { reply_to, sign_in }) = *event else {
            panic!("take must ask Whose before SignIn")
        };
        assert_eq!(sign_in, NUMBER);
        reply_to
    }
    pub fn approval_sign_in(&mut self, identity: &people::Identity) -> ReplyTo {
        let Some(Request::Core(event)) = self.out.pop() else { panic!("SignIn event") };
        let core::Event::SignIn { until: None, reply_to, identity: given, held } = *event else {
            panic!("ordinary SignIn")
        };
        assert_eq!(&given, identity);
        assert_eq!(held, None);
        reply_to
    }
}
