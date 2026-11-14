use super::*;
pub const KEY: [u8; 32] = [23; 32];
// Two simultaneous answers of at most 8 KiB each, enough for the tiny index below.
pub const BOUND: u32 = 8192;
pub const BURST: u64 = 16384;
pub const INDEX: wire::PublicIndexBounds =
    wire::PublicIndexBounds { goals: 2, ended: 2, lines: 2, depth: 2, projects: 2 };
pub const PROJECT: u32 = 0xfedc_ba98;
pub const TASK: u64 = 0xfedc_ba98_7654_3210;
pub const PARENT: u64 = 0xdead_beef_abcd_1234;
pub fn limits() -> crate::PublicLimits {
    crate::PublicLimits {
        reads: 2,
        rate: 8,
        rate_burst: 8,
        bytes: BURST,
        bytes_burst: BURST,
        answer_bytes: BOUND,
        max_age: 5,
    }
}
pub fn settings() -> Settings {
    let mut settings = f::settings();
    settings.public_bounds = INDEX;
    settings.public = List::with_capacity(4);
    for mut kind in crate::jig_kinds(&INDEX) {
        kind.bound = BOUND;
        settings.public.push(kind).unwrap();
    }
    settings
        .public
        .push(crate::PublicKind { owner: 10, kind: 1, first: 2, last: 3, task: true, bound: BOUND })
        .unwrap();
    settings
}
pub fn key() -> Secret {
    Secret::new(Box::new(KEY))
}
pub fn names() -> Names {
    Names::new(&key())
}
pub fn kept() -> Kept {
    Kept { digest: key(), expires: Wall::from_nanos(u64::MAX) }
}
pub fn read(kind: u16, ids: &[u64]) -> PublicRead {
    PublicRead { owner: 0, version: 1, kind, ids: Box::from(ids) }
}
pub struct Step {
    pub component: Component,
    pub env: Env<Limits>,
    pub out: Queue<Request>,
    pub next: u64,
}
impl Step {
    pub fn new() -> Self {
        Self::configured(limits(), settings(), true)
    }
    pub fn configured(limits: crate::PublicLimits, settings: Settings, held: bool) -> Self {
        let mut env = f::env();
        env.limits.public = limits;
        let mut out = Queue::uncapped();
        let component = Component::new(settings, &env, &mut out).expect("public settings");
        out.pop().expect("get public key");
        let mut step = Self { component, env, out, next: 100 };
        if held {
            step.down(Event::Secrets(SecretsAnswer::Got { owner: SecretOwner::PublicNames, value: Some(kept()) }));
        }
        step
    }
    pub fn down(&mut self, event: Event) {
        self.component.down(&self.env, event, &mut self.out);
    }
    pub fn send(&mut self, read: PublicRead) -> Token {
        let exchange = Token::new(self.next);
        self.next = self.next.checked_add(1).unwrap();
        self.down(Event::Transport(FromTransport::PublicRead { exchange, read }));
        exchange
    }
    pub fn admitted(&mut self) -> (ReplyTo, c::PublicRead) {
        let Some(Request::Core(event)) = self.out.pop() else {
            panic!("public core request");
        };
        let c::Event::PublicRead { reply, read } = *event else {
            panic!("public read");
        };
        (reply, read)
    }
    pub fn reply(&mut self, exchange: Token) -> PublicReply {
        let Some(Request::Transport(ToTransport::Public { exchange: to, reply })) = self.out.pop() else {
            panic!("public reply");
        };
        assert_eq!(to, exchange);
        assert!(self.out.is_empty(), "nothing handed to the engine");
        reply
    }
    pub fn answer(&mut self, reply: ReplyTo, answer: c::PublicAnswer) {
        self.down(Event::Public { reply, answer });
    }
    pub fn document(&mut self, reply: ReplyTo, document: c::PublicDocument) {
        self.answer(reply, c::PublicAnswer::Document { decided: Time::from_nanos(123), document });
    }
    pub fn bytes(&self) -> u64 {
        self.component.public.available_bytes()
    }
}
pub fn projects() -> c::PublicDocument {
    c::PublicDocument::Projects(c::PublicProjects {
        projects: Box::new([c::PublicProject { project: PROJECT, name: Box::from("Public project") }]),
    })
}
pub fn line(task: u64) -> c::PublicLine {
    c::PublicLine {
        task,
        title: c::documents::Text { bytes: Box::from(b"Public task".as_slice()), cut: false },
        parent: Some(PARENT),
        goal: false,
        priority: Some(1),
        executor: c::PublicExecutor::Job,
        phase: c::PublicPhase::Running,
        hold: Some(c::PublicHold::Budget),
        made: 1,
        changed: 2,
        active: Some(3),
        ended: None,
        spent: None,
        delegates: c::PublicCounts { working: 1, waiting: 2, held: 3, closing: 4, ended: 5 },
    }
}
pub fn board() -> c::PublicDocument {
    c::PublicDocument::Board(c::PublicBoard {
        project: PROJECT,
        name: Box::from("Public project"),
        live: Box::new([c::PublicGoal { line: line(TASK), tree: Box::new([line(PARENT)]), folded: 1 }]),
        live_folded: 2,
        ended: Box::new([c::PublicGoal { line: line(PARENT), tree: Box::new([]), folded: 3 }]),
    })
}
pub fn card() -> c::PublicDocument {
    c::PublicDocument::Card(c::PublicCard {
        line: line(TASK),
        lineage: Box::new([PARENT]),
        delegates: Box::new([line(PARENT)]),
        folded: 1,
    })
}
pub fn bytes(reply: PublicReply) -> Box<[u8]> {
    let PublicReply::Document { bytes, decided } = reply else {
        panic!("public document");
    };
    assert_eq!(decided, Time::from_nanos(123));
    bytes
}

pub fn frame_public_identifiers(value: &wire::v22::Me, limits: &wire::v22::Limits) -> [Option<u64>; 2] {
    let mut reader = Reader::new(value.projects());
    let mut ids = [None; 2];
    for id in &mut ids {
        let project = skein_codec::piece::split(&mut reader).unwrap().unwrap();
        let project = wire::v22::MeProject::decode(limits, &mut Reader::new(&project)).unwrap();
        *id = *project.public();
    }
    ids
}
