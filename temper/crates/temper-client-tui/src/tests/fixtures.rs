use jig_client::contract as c;
use skein_lib::{Duration, Env, List, Queue, RequestLimits, Secret, Time, Token, Wall, Writer};
use temper_client_domain::{Domain, Event, Limits, Request, forge};
pub const ADDRESS: &[u8] = b"127.0.0.1:8367";
pub const NOW: Wall = Wall::EPOCH;
pub const SEED: u64 = 11;
#[expect(clippy::arithmetic_side_effects, reason = "the request count is the nonzero constant declared here")]
pub fn limits() -> Limits {
    {
        let requests = RequestLimits {
            requests: 4,
            bytes: 262_144,
            out: 16,
            first: Duration::from_millis(10),
            most: Duration::from_millis(80),
            margin: Duration::from_secs(1),
        };
        let client = jig_client::Limits {
            domain: jig_client::DomainLimits {
                accounts: jig_client::AccountCounts { accounts: 16, sources: 8, sign_ins: 4 },
                owners: &[jig_client::Spoken {
                    family: temper_client_protocol_wire::FORGE,
                    version: temper_client_protocol_wire::CLIENT_VERSION,
                }],
                requests,
                ask_bytes: requests.bytes / u64::from(requests.requests),
                // The page handles, project-needs, session Away and Home decisions watches, trials and a running trial task.
                handles: 19, // Four findings subjects and one live-goals menu.
                watches: 19, // Four findings subjects and one live-goals menu.
                turn_bytes: 65536,
                backoff: jig_client::Backoff { first: Duration::from_millis(10), most: Duration::from_millis(80) },
                authority: 12,
                heartbeat: Duration::from_millis(100),
                reads: 8,
                parts: 8,
                delegates: 2,
                document_bytes: 65536,
                records: 2,
                kept: 4,
                projects: 2,
                text: 32,
                long_text: 32,
                turn_text: 32,
                rows: 2,
                fields: 4,
                segments: 9,
            },
            exchanges: 8,
        };

        Limits { forge: forge::Limits { field_bytes: 256, checks: 8 }, client, row_uses: 8, window_bytes: None }
    }
}

pub fn words() -> crate::words::Words {
    let shared = temper_client_view::words::Words::english(
        temper_client_protocol_wire::number(temper_client_protocol_wire::Owner::Forge),
        limits().client.domain.text,
        temper_client_view::words::Shell::Terminal,
    )
    .expect("view English fits");
    crate::words::Words::english(shared, &limits()).expect("TUI English fits")
}
pub fn view() -> crate::View {
    crate::View::new(&crate::limits(&limits(), &skein_ui::Limits::EMPTY, 512).expect("tree limits"), words())
}
pub fn event(root: &mut Domain, event: Event) -> Queue<Request> {
    let mut out = Queue::uncapped();
    temper_client_domain::step(root, &Env { now: Time::ZERO, wall: NOW, limits: limits() }, event, &mut out);
    out
}
pub fn client(root: &mut Domain, event: jig_client::Event) -> Queue<Request> {
    self::event(root, Event::Client { event })
}
pub fn signed_out() -> Domain {
    let mut root = temper_client_domain::unlinked(&limits(), SEED);
    client(&mut root, jig_client::Event::StartUnlinked { kept: List::with_capacity(0) });
    root
}
pub fn starting() -> (Domain, Token) {
    let mut root = signed_out();
    let mut out = event(
        &mut root,
        Event::Start {
            address: temper_client_domain::Address::Inbox { filter: temper_client_domain::address::Filter::ALL },
            kept: List::with_capacity(0),
            with: Some(jig_client::With::Transport),
        },
    );
    let mut watch = None;
    for _ in 0..out.len() {
        if let Some(Request::Unlinked {
            request:
                jig_client::UnlinkedRequest::Down {
                    down: jig_client::Down::Open { watch: token, subject: jig_client::Watchable::Jig(c::Subject::Me) },
                },
        }) = out.pop()
        {
            watch = Some(token);
        }
    }
    (root, watch.expect("me watch"))
}
pub fn live(me: Option<c::Me>) -> (Domain, Token) {
    let (mut root, watch) = starting();
    client(&mut root, jig_client::Event::Up { up: jig_client::Up::Served { served: true, versions: None } });
    match me {
        Some(me) => deliver(&mut root, watch, me),
        None => {
            deliver(&mut root, watch, self::me(&[]));
            client(&mut root, jig_client::Event::Unwatch { handle: temper_client_domain::pages::me_handle() });
        }
    }
    (root, watch)
}
pub fn deliver(root: &mut Domain, watch: Token, me: c::Me) {
    client(
        root,
        jig_client::Event::Up {
            up: jig_client::Up::Watched { watch, event: c::WatchEvent::Delivered(c::Delivery::Me(Box::new(me))) },
        },
    );
}
pub fn asking() -> (Domain, Token) {
    let mut root = temper_client_domain::new(&limits(), SEED);
    let mut out = event(
        &mut root,
        Event::Start {
            address: temper_client_domain::Address::Inbox { filter: temper_client_domain::address::Filter::ALL },
            kept: List::with_capacity(0),
            with: Some(jig_client::With::Link { secret: Secret::new(Box::new([0; 32])), held: None }),
        },
    );
    let mut exchange = None;
    for _ in 0..out.len() {
        if let Some(Request::Client {
            request: jig_client::Request::Exchange { exchange: token, message: jig_client::Message::SignInLink(_), .. },
        }) = out.pop()
        {
            assert!(exchange.replace(token).is_none());
        }
    }
    (root, exchange.expect("link exchange"))
}
pub fn answer(root: &mut Domain, exchange: Token, answer: jig_client_protocol_wire::SignInAnswer) {
    let mut writer = Writer::new(usize::try_from(answer.measure()).expect("answer fits"));
    answer.encode(&mut writer).expect("answer room");
    client(root, jig_client::Event::Reply { exchange, message: writer.finish() });
}
pub fn refused(answer: jig_client_protocol_wire::SignInAnswer) -> Domain {
    let (mut root, exchange) = asking();
    self::answer(&mut root, exchange, answer);
    assert_eq!(root.client().link().state, jig_client::Link::SignedOut);
    root
}
pub fn full() -> jig_client_protocol_wire::SignInAnswer {
    use jig_client_protocol_wire::{SignInAnswer, SignInsFull, SignInsFullParts};
    SignInAnswer::Full(
        SignInsFull::new(
            &jig_client_protocol_wire::CEILINGS,
            SignInsFullParts { sign_ins: u32::MAX, earliest: 0, link_until: 0 },
        )
        .expect("full fields"),
    )
}
pub fn me(names: &[&[u8]]) -> c::Me {
    use c::{DocumentText, HeldRole, Me, MeParty, MeProject, Needs, PartyKind};
    let mut projects = List::with_capacity(limits().client.domain.projects);
    for (index, name) in names.iter().enumerate() {
        projects
            .push(MeProject {
                left_out: 0,
                defaults: List::with_capacity(0),
                public: None,
                project: u32::try_from(index).unwrap().checked_add(1).unwrap(),
                name: DocumentText::complete(c::Text::new(Box::from(*name), 32).expect("name fits")),
                role: HeldRole::Policy { role: 1 },
                pool: None,
                templates: List::with_capacity(0),
                creating: None,
                models: List::with_capacity(0),
            })
            .expect("project room");
    }
    Me {
        providers: List::with_capacity(0),
        providers_cut: false,
        key_bytes: 256,
        redirect_bytes: 256,
        words_bytes: 8192,
        party: MeParty {
            number: 73,
            kind: PartyKind::Person,
            login: DocumentText::complete(c::Text::new(Box::from(*b"owner"), 32).expect("login fits")),
            name: DocumentText::complete(c::Text::new(Box::from(*b"Owner"), 32).expect("name fits")),
        },
        projects,
        needs: Needs { questions: 0, proposals: 0, escalations: 0, person_tasks: 0 },
        needs_total: 0,
        updates: c::Count { value: 0, limited: false },
        results_seen: 0,
        cut: false,
        retention: Duration::from_secs(1000),
        deployment_role: None,
    }
}
pub fn index(tree: &skein_ui::Tree, id: skein_ui::NodeId) -> usize {
    for (index, node) in tree.nodes().iter().enumerate() {
        if node.id == id {
            return index;
        }
    }
    panic!("fixture node exists")
}
pub fn count(tree: &skein_ui::Tree, role: skein_ui::Role) -> usize {
    let mut count = 0_usize;
    for node in tree.nodes() {
        if node.role == role {
            count = count.checked_add(1).unwrap();
        }
    }
    count
}
pub fn text(tree: &skein_ui::Tree, node: &skein_ui::Node) -> Box<[u8]> {
    let index = index(tree, node.id);
    let end = index.checked_add(usize::try_from(node.size).unwrap()).unwrap();
    let mut bytes = 0_usize;
    for child in &tree.nodes()[index..end] {
        for run in &child.text {
            bytes = bytes.checked_add(run.text.len()).unwrap();
        }
    }
    let mut writer = Writer::new(bytes);
    for child in &tree.nodes()[index..end] {
        for run in &child.text {
            writer.put(&run.text).unwrap();
        }
    }
    writer.finish()
}
pub fn find<'a>(tree: &'a skein_ui::Tree, role: skein_ui::Role, words: &[u8]) -> &'a skein_ui::Node {
    for node in tree.nodes() {
        if node.role == role && (node.name.as_deref() == Some(words) || text(tree, node).as_ref() == words) {
            return node;
        }
    }
    panic!("node says expected words: {role:?} {words:?}")
}
pub fn build(view: &mut crate::View, tui: &crate::Tui, root: &Domain) -> Queue<skein_ui::Patch> {
    let mut out = Queue::with_capacity(crate::limits(&limits(), &skein_ui::Limits::EMPTY, 512).unwrap().patches);
    crate::build(view, tui, root, NOW, &mut out);
    out
}

// Kind 7's project value, delivered through the forge's public read envelope.
pub fn draft_setting(drafts: bool) -> Box<[u8]> {
    use temper_client_protocol_wire::{self as wire, v1 as w};
    let value = w::ProjectWays::new(&w::CEILINGS, w::ProjectWaysParts { drafts, goals: false, changes: false, way: 0 })
        .unwrap();
    let mut writer = Writer::new(usize::try_from(value.measure()).unwrap());
    value.encode(&mut writer).unwrap();
    wire::setting_answer(
        &w::SettingAnswer::new(&w::CEILINGS, w::SettingAnswerParts { value: Some(writer.finish()) }).unwrap(),
    )
}

pub fn ways_read(record: &[u8]) -> bool {
    use temper_client_protocol_wire::{self as wire, v1 as w};
    if let w::Read::Setting(setting) = wire::decode_reading(record, &w::CEILINGS).unwrap().read() {
        setting.kind() == 7
    } else {
        false
    }
}
