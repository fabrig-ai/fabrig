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
                // Page and session handles, plus the launchpad's left-off and second project's status reads.
                handles: 18,
                watches: 18,
                turn_bytes: 65536,
                backoff: jig_client::Backoff { first: Duration::from_millis(10), most: Duration::from_millis(80) },
                authority: 12,
                heartbeat: Duration::from_millis(100),
                reads: 12, // The launchpad's two cards coexist with Home and session reads.
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
    ways_setting(drafts, 0)
}
fn ways_setting(drafts: bool, way: u32) -> Box<[u8]> {
    use temper_client_protocol_wire::{self as wire, v1 as w};
    let value =
        w::ProjectWays::new(&w::CEILINGS, w::ProjectWaysParts { drafts, goals: false, changes: false, way }).unwrap();
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

#[derive(Clone, Copy)]
#[expect(clippy::struct_excessive_bools, reason = "independent gates of the goal command and sheet")]
pub struct GoalCase {
    pub body: Option<&'static [u8]>,
    pub repository: bool,
    pub drafts: bool,
    pub linked: bool,
    pub asking: bool,
    pub ways: bool,
    pub funds: u64,
    pub reserved: u64,
}
impl GoalCase {
    pub const READY: Self = Self {
        body: Some(b"# Plan\n\nWords\n"),
        repository: true,
        drafts: true,
        linked: false,
        asking: false,
        ways: true,
        funds: 100,
        reserved: 0,
    };

    #[expect(clippy::too_many_lines, reason = "the fixture answers the chat's public reads and watches")]
    pub fn root(self) -> Domain {
        use jig_client::{Down, Readable, Up, Watchable};
        use temper_client_domain::Address;
        use temper_client_protocol_wire::{self as wire, v1 as w};
        let mut root = signed_out();
        let mut out = event(
            &mut root,
            Event::Start {
                address: Address::Chat { task: 9 },
                kept: List::uncapped(),
                with: Some(jig_client::With::Transport),
            },
        );
        append(&mut out, client(&mut root, jig_client::Event::Up { up: Up::Served { served: true, versions: None } }));
        let link = jig_client_protocol_link::Limits::new(limits().client.domain, 8).unwrap();
        let mut task =
            jig_client_protocol_link::translate::documents::task(jig_client_test_records::task_record(), &link)
                .unwrap();
        task.chip.task = 9;
        task.project = 1;
        task.label = 7;
        task.chip.executor = c::ChipExecutor::Agent { charter: if self.asking { 1 } else { 12 } };
        task.chip.details.phase = c::ChipPhase::Active(c::RunStage::Idle);
        task.chip.phase = c::Phase::Active;
        task.flags.conversation = true;
        task.result = None;
        task.run_wait = None;
        task.requester = c::Requester::Person(73);
        task.delegates = List::uncapped();
        task.acts = List::from_boxed(Box::new([c::Offered {
            act: c::ActKind::Cancel,
            admission: c::Admission::Allowed,
            effect: None,
        }]));
        let mut me = me(&[b"Demo"]);
        let project = me.projects.get_mut(0).unwrap();
        let mut chat = super::foot::template(6);
        chat.executor = c::TemplateExecutor::Charter(12);
        let mut asking = super::foot::template(5);
        asking.executor = c::TemplateExecutor::Charter(1);
        project.templates = List::uncapped();
        project.templates.push(chat).unwrap();
        project.templates.push(asking).unwrap();
        if self.ways {
            for (number, name) in [(8, b"Custom".as_slice()), (4, b"goal for a draft"), (2, b"goal")] {
                let mut way = super::foot::template(number);
                way.name = c::DocumentText::complete(c::Text::new(name.into(), 32).unwrap());
                way.tracked = Some(1);
                project.templates.push(way).unwrap();
            }
        }
        if self.reserved > 0 {
            project.pool = Some(c::PoolLedger {
                source: c::Funder::Pool { project: 1, person: 73, period: 0 },
                parent: None,
                numbers: c::Numbers { budget: self.reserved, spent: 0, spent_below: 0, reserved: self.reserved },
                left: 0,
                closed: false,
                began: None,
                next: None,
            });
        }
        project.creating = Some(c::Creating {
            within: c::CreateBounds {
                authority: Box::new(task.authority.clone()),
                pool_left: self.funds,
                charters: List::from_boxed(Box::new([1])),
                procedures: List::uncapped(),
                latest: None,
                budget: self.funds,
                maximum_budget: 100,
                maximum_deadline: None,
            },
            beyond: c::Beyond::Refused(c::Refusal::NotOffered),
        });
        while let Some(request) = out.pop() {
            let Request::Unlinked { request: jig_client::UnlinkedRequest::Down { down } } = request else { continue };
            let up = match down {
                Down::Open { watch, subject: Watchable::Jig(c::Subject::Me) } => {
                    Up::Watched { watch, event: c::WatchEvent::Delivered(c::Delivery::Me(Box::new(me.clone()))) }
                }
                Down::Open { watch, subject: Watchable::Jig(c::Subject::Task { task: 9 }) } => {
                    Up::Watched { watch, event: c::WatchEvent::Delivered(c::Delivery::Task(Box::new(task.clone()))) }
                }
                Down::Open { watch, subject: Watchable::Jig(c::Subject::Chips { .. }) } if self.linked => Up::Watched {
                    watch,
                    event: c::WatchEvent::Delivered(c::Delivery::Chips(List::from_boxed(Box::new([goal_chip(
                        task.chip.clone(),
                    )])))),
                },
                Down::Open { watch, subject: Watchable::Family { record, .. } } => {
                    let bytes = match wire::decode_watching(&record.bytes, &w::CEILINGS).unwrap().watch() {
                        w::Watch::Forges => wire::forges(
                            &w::Forges::new(
                                &w::CEILINGS,
                                w::ForgesParts { forges: goal_forges(self.repository), linked: false },
                            )
                            .unwrap(),
                        ),
                        w::Watch::Dismissed => {
                            let mut dismissed = List::with_capacity(wire::DISMISSED);
                            if self.linked {
                                dismissed
                                    .push(w::PersonFact::GoalFrom(
                                        w::GoalFrom::new(
                                            &w::CEILINGS,
                                            w::GoalFromParts { project: 1, chat: 9, goal: 25, at: 1 },
                                        )
                                        .unwrap(),
                                    ))
                                    .unwrap();
                            }
                            wire::dismissals(
                                &w::Dismissals::new(&w::CEILINGS, w::DismissalsParts { dismissed }).unwrap(),
                            )
                        }
                        w::Watch::Change(_)
                        | w::Watch::Follows(_)
                        | w::Watch::Landing(_)
                        | w::Watch::Landings(_)
                        | w::Watch::Changes(_)
                        | w::Watch::Status(_)
                        | w::Watch::Away
                        | w::Watch::Decided(_)
                        | w::Watch::Trials
                        | w::Watch::Approvals
                        | w::Watch::Ways(_)
                        | w::Watch::Drafts(_) => continue,
                    };
                    Up::FamilyDelivered {
                        watch,
                        record: jig_client::Family { family: wire::FORGE, version: wire::CLIENT_VERSION, bytes },
                    }
                }
                Down::Read { read, what: Readable::Family { record, .. } } if ways_read(&record.bytes) => {
                    Up::ReadDone {
                        read,
                        result: jig_client::Readout::Family {
                            family: wire::FORGE,
                            record: Some(ways_setting(self.drafts, 4)),
                        },
                    }
                }
                Down::Read { read, what: Readable::Jig(c::Read::Notes { scope, .. }) } => {
                    let mut rows = List::uncapped();
                    if let Some(body) = self.body {
                        rows.push(c::Note {
                            number: 1,
                            revision: c::Revision { value: 1 },
                            scope,
                            description: c::DocumentText::complete(
                                c::Text::new(b"Plan".as_slice().into(), 32).unwrap(),
                            ),
                            body: c::DocumentText::complete(c::Text::new(body.into(), 32).unwrap()),
                            writer: c::NoteAuthor::Task { task: 9, attempt: 1 },
                            tasks: List::uncapped(),
                            changed: None,
                            acts: List::uncapped(),
                            cut: false,
                        })
                        .unwrap();
                    }
                    Up::ReadDone {
                        read,
                        result: jig_client::Readout::Jig(c::ReadResult::Value(c::ReadValue::Notes(Box::new(
                            c::NotePage { rows, before: None, after: None, cut: false },
                        )))),
                    }
                }
                Down::Save { .. }
                | Down::Erase { .. }
                | Down::Send { .. }
                | Down::Open { .. }
                | Down::Continue { .. }
                | Down::Close { .. }
                | Down::Read { .. } => continue,
            };
            append(&mut out, client(&mut root, jig_client::Event::Up { up }));
        }
        assert_eq!(root.pages().address(), &Address::Chat { task: 9 });
        root
    }
}
pub(super) fn goal_chip(mut chip: c::Chip) -> c::Chip {
    chip.task = 25;
    chip.title =
        c::DocumentText::complete(c::Text::new(b"Farewell line for the hello page".as_slice().into(), 64).unwrap());
    chip
}
fn append(out: &mut Queue<Request>, mut more: Queue<Request>) {
    while let Some(request) = more.pop() {
        out.push(request);
    }
}
pub(super) fn goal_forges(repository: bool) -> List<temper_client_protocol_wire::v1::ForgeRow> {
    use temper_client_protocol_wire::v1 as w;
    let mut repositories = List::with_capacity(1);
    if repository {
        repositories
            .push(
                w::Adoption::new(
                    &w::CEILINGS,
                    w::AdoptionParts { owner: b"work".as_slice().into(), name: b"shop".as_slice().into(), project: 1 },
                )
                .unwrap(),
            )
            .unwrap();
    }
    List::from_boxed(Box::new([w::ForgeRow::new(
        &w::CEILINGS,
        w::ForgeRowParts {
            number: 2,
            host: b"git.example".as_slice().into(),
            kind: w::Kind::Forgejo,
            mode: w::Mode::Owned,
            connection: w::Connection::Connected(
                w::Connected::new(&w::CEILINGS, w::ConnectedParts { id: 1, login: b"temper".as_slice().into() })
                    .unwrap(),
            ),
            offer: w::Offer::NotOffered,
            first: None,
            starts_with: w::StartsWith::Repository,
            installations: List::with_capacity(0),
            ended: None,
            app: None,
            web: None,
            repositories,
            owned: None,
            admin: false,
            ended_stop: None,
            read_only: false,
            makings: List::with_capacity(0),
        },
    )
    .unwrap()]))
}
