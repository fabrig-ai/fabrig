use super::fixtures as f;
use skein_ui::{Act, Role};
use temper_client_domain::{self as d, Address, ChatsFilter};

const UNKNOWN: u32 = 99;

fn printed(view: &crate::View) -> Box<[u8]> {
    let mut print = skein_ui_text::Print::new(&view.limits().tree).expect("terminal bounds");
    skein_ui_text::render(view.tree(), b"temper", None, &mut print);
    skein_lib::bytes::copy_of(print.text.as_slice())
}

fn main_nodes(tree: &skein_ui::Tree) -> &[skein_ui::Node] {
    let mut main = None;
    for (index, node) in tree.nodes().iter().enumerate() {
        if node.role == Role::Main {
            let end = index.checked_add(usize::try_from(node.size).expect("main size")).expect("main end");
            main = tree.nodes().get(index..end);
        }
    }
    main.expect("main subtree")
}

fn main_find(tree: &skein_ui::Tree, role: Role, name: &[u8]) -> skein_ui::NodeId {
    let mut found = None;
    for node in main_nodes(tree) {
        if node.role == role && node.name.as_deref() == Some(name) {
            found = Some(node.id);
        }
    }
    found.expect("main control")
}

#[test]
fn project_pages_draw_their_heading_in_both_shells() {
    for (address, heading) in [
        (Address::Home { project: f::PROJECT }, b"Shop".as_slice()),
        (Address::Settings { project: f::PROJECT }, b"How work runs in Shop.".as_slice()),
    ] {
        let (root, _, _) = f::root_at(address, &f::domain_limits());
        for view in [f::page_view(&root), f::browser_page_view(&root)] {
            let tree = view.tree();
            let main = main_nodes(tree);
            assert!(main.first().expect("main").styles.as_slice().contains(&crate::look::MAIN));
            let _heading = main_find(tree, Role::Heading { level: 1 }, heading);
            assert!(skein_lib::bytes::find(&f::main_text(tree), b"not one of yours").is_none());
        }
        let view = f::page_view(&root);
        let printed = printed(&view);
        assert!(skein_lib::bytes::find(&printed, heading).is_some());
    }
}

#[test]
fn unlisted_projects_explain_and_link_to_chats() {
    for address in [
        Address::Home { project: UNKNOWN },
        Address::Settings { project: UNKNOWN },
        Address::Repositories { project: UNKNOWN },
    ] {
        let (root, _, _) = f::root_at(address, &f::domain_limits());
        for mut view in [f::page_view(&root), f::browser_page_view(&root)] {
            let main = f::main_text(view.tree());
            assert!(skein_lib::bytes::find(&main, b"Project 99 is not one of yours.").is_some());
            let link = main_find(view.tree(), Role::Link, b"Chats");
            assert_eq!(
                crate::act(&mut view, link, Act::Press),
                Some(d::Action::Pages(d::pages::Act::Go { address: Address::Chats { filter: ChatsFilter::All } }))
            );
        }
        let view = f::page_view(&root);
        assert!(skein_lib::bytes::find(&printed(&view), b"Project 99 is not one of yours.").is_some());
    }
}

#[test]
fn settings_links_carry_the_forge_and_account_marks_in_both_shells() {
    use jig_client::contract::AccountState;
    use temper_client_protocol_wire::v1 as w;
    for (connection, account, style, forge_words, account_words) in [
        (
            w::Connection::Connecting,
            AccountState::Later { until: Some(f::NOW) },
            crate::look::NAV_LIMITED,
            b"a forge is setting up".as_slice(),
            b"an account is limited".as_slice(),
        ),
        (
            w::Connection::Stopped(w::Stopped::new(&w::CEILINGS, w::StoppedParts { reason: Box::default() }).unwrap()),
            AccountState::SignInAgain,
            crate::look::NAV_NEEDS,
            b"a forge needs you",
            b"an account needs you",
        ),
    ] {
        let (mut root, watch) = f::forge_root_accounts(
            Address::Settings { project: f::PROJECT },
            super::connect_forge::rows(connection, w::Offer::Offered),
        );
        let mut accounts = super::accounts::document(1);
        accounts.accounts.get_mut(0).unwrap().sources.get_mut(0).unwrap().state = account;
        super::accounts::deliver(&mut root, watch, accounts);
        for view in [f::page_view(&root), f::browser_page_view(&root)] {
            for (name, expected) in [(b"Forges".as_slice(), forge_words), (b"Accounts", account_words)] {
                let link = main_find(view.tree(), Role::Link, name);
                let mut marks = 0_u32;
                for node in subtree(view.tree(), link) {
                    if node.styles.as_slice().contains(&style) {
                        marks += 1;
                        assert_eq!(node.role, Role::Text);
                        assert_eq!(node.name.as_deref(), Some(expected));
                        assert_eq!(f::nodes_text(core::slice::from_ref(node)).as_ref(), expected);
                    }
                }
                assert_eq!(marks, 1, "one mark on each settings link");
            }
        }
        let text = printed(&f::page_view(&root));
        for expected in [forge_words, account_words] {
            assert!(skein_lib::bytes::find(&text, expected).is_some());
        }
    }
}

fn subtree(tree: &skein_ui::Tree, id: skein_ui::NodeId) -> &[skein_ui::Node] {
    for (at, node) in tree.nodes().iter().enumerate() {
        if node.id == id {
            let end = at.checked_add(usize::try_from(node.size).expect("subtree size")).expect("subtree end");
            return &tree.nodes()[at..end];
        }
    }
    panic!("subtree root exists")
}

use jig_client::contract as c;
use skein_lib::{List, Queue, Token};
use temper_client_protocol_wire::{self as wire, v1 as w};

fn home_limits() -> d::Limits {
    let mut limits = f::domain_limits();
    // Home, goal presence, the Goals page and the standing Needs watch (README.md, 3.5).
    limits.client.domain.handles = 15;
    limits.client.domain.watches = 13;
    limits.client.domain.reads = 16;
    limits
}
fn drive(root: &mut d::Domain, event: d::Event) -> Queue<d::Request> {
    let mut out = Queue::uncapped();
    d::step(root, &skein_lib::Env { now: skein_lib::Time::ZERO, wall: f::NOW, limits: home_limits() }, event, &mut out);
    out
}
fn deliver(root: &mut d::Domain, watch: Token, value: c::Delivery) {
    drive(
        root,
        d::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched { watch, event: c::WatchEvent::Delivered(value) },
            },
        },
    );
}
fn home_root(rows: List<w::ForgeRow>, configured: bool, chat: bool) -> (d::Domain, Token) {
    home_with_needs(rows, configured, chat, None)
}
struct NeedsFixture<'a> {
    objects: &'a [c::ObjectKey],
    cut: bool,
    read: bool,
}
fn home_with_needs(
    rows: List<w::ForgeRow>,
    configured: bool,
    chat: bool,
    needs: Option<NeedsFixture<'_>>,
) -> (d::Domain, Token) {
    home_documents(rows, configured, chat, needs, None)
}
struct ChatFixture {
    live: c::Tasks,
    ended: c::TaskPage,
}
fn home_documents(
    rows: List<w::ForgeRow>,
    configured: bool,
    chat: bool,
    needs: Option<NeedsFixture<'_>>,
    chats: Option<ChatFixture>,
) -> (d::Domain, Token) {
    home_documents_with_reads(rows, configured, chat, needs, chats, Some(&[]), true)
}
fn home_documents_with_reads(
    rows: List<w::ForgeRow>,
    configured: bool,
    chat: bool,
    needs: Option<NeedsFixture<'_>>,
    chats: Option<ChatFixture>,
    dismissed: Option<&[u32]>,
    settings_delivered: bool,
) -> (d::Domain, Token) {
    home_documents_with_status(rows, configured, chat, needs, chats, dismissed, settings_delivered, None)
}
struct StatusFixture {
    document: w::ProjectStatus,
    answered: bool,
    goals: bool,
    goal_rows: Option<ChatFixture>,
    away: Option<AwayFixture>,
}
#[expect(
    clippy::disallowed_methods,
    clippy::too_many_arguments,
    clippy::too_many_lines,
    reason = "one fixture delivers Home documents and their public replies together"
)]
fn home_documents_with_status(
    rows: List<w::ForgeRow>,
    configured: bool,
    chat: bool,
    needs: Option<NeedsFixture<'_>>,
    chats: Option<ChatFixture>,
    dismissed: Option<&[u32]>,
    settings_delivered: bool,
    status: Option<&StatusFixture>,
) -> (d::Domain, Token) {
    let (mut root, me_watch, accounts, settings, needs_watch, away_watch) =
        f::home_root(&home_limits(), rows, dismissed, status.is_some_and(|fixture| fixture.goals));
    if let Some(spell) = status.and_then(|s| s.away.as_ref()).and_then(|a| a.spell.clone()) {
        drive(
            &mut root,
            d::Event::Client {
                event: jig_client::Event::Up {
                    up: jig_client::Up::FamilyDelivered {
                        watch: away_watch,
                        record: jig_client::Family {
                            family: wire::FORGE,
                            version: wire::CLIENT_VERSION,
                            bytes: wire::aways(
                                &w::Aways::new(
                                    &w::CEILINGS,
                                    w::AwaysParts { rows: List::from_boxed(Box::new([spell])) },
                                )
                                .unwrap(),
                            ),
                        },
                    },
                },
            },
        );
    }
    let template_root = f::chat::messaging_root();
    let template_me = template_root.pages().me(template_root.client()).unwrap();
    let mut me = root.pages().me(root.client()).unwrap().clone();
    let member = me.projects.get_mut(0).unwrap();
    member.role = c::HeldRole::Owner;
    member.templates = template_me.projects.get(0).unwrap().templates.clone();
    if status.is_some_and(|fixture| fixture.goals) {
        let mut goal = member.templates.get(0).unwrap().clone();
        goal.number = jig_client::shapes::GOAL;
        goal.tracked = Some(1);
        let mut templates = List::uncapped();
        for template in &member.templates {
            templates.push(template.clone()).unwrap();
        }
        templates.push(goal).unwrap();
        member.templates = templates;
    }
    member.models = List::with_capacity(1);
    member
        .models
        .push(c::MeModel {
            name: c::Text::new(b"gpt-6-luna".as_slice().into(), 32).unwrap(),
            provider: Some(17),
            description: None,
            prices: c::ModelPrices { input: 0, cached: 0, cache_write: None, output: 0, unit: 1 },
            window: 200_000,
            output: 16_384,
            efforts: List::with_capacity(0),
        })
        .unwrap();
    let mut out = drive(
        &mut root,
        d::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched {
                    watch: me_watch,
                    event: c::WatchEvent::Delivered(c::Delivery::Me(Box::new(me))),
                },
            },
        },
    );
    let mut entered =
        drive(&mut root, d::Event::Went { place: d::Place::Address(Address::Home { project: f::PROJECT }) });
    while let Some(request) = entered.pop() {
        out.push(request);
    }
    let mut delivered = drive(
        &mut root,
        d::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched {
                    watch: needs_watch,
                    event: c::WatchEvent::Delivered(c::Delivery::Inbox(Box::new(super::documents::inbox(
                        List::with_capacity(0),
                        0,
                        c::Count { value: 0, limited: false },
                        0,
                        false,
                    )))),
                },
            },
        },
    );
    while let Some(request) = delivered.pop() {
        out.push(request);
    }
    let mut live = None;
    for request in &out {
        if let d::Request::Unlinked {
            request:
                jig_client::UnlinkedRequest::Down {
                    down: jig_client::Down::Open { watch, subject: jig_client::Watchable::Jig(subject) },
                },
        } = request
        {
            match subject {
                c::Subject::Tasks { query, .. } if query.tracked != Some(true) => live = Some(*watch),
                c::Subject::Tasks { .. }
                | c::Subject::Settings
                | c::Subject::Me
                | c::Subject::Task { .. }
                | c::Subject::Tree { .. }
                | c::Subject::Chips { .. }
                | c::Subject::Inbox { .. }
                | c::Subject::Run { .. }
                | c::Subject::Deployment
                | c::Subject::Accounts => {}
            }
        }
    }
    let mut lines = List::with_capacity(1);
    lines
        .push(c::SettingLine {
            setting: c::Setting::Rules,
            name: None,
            follows: if configured { c::SettingFollows::Change } else { c::SettingFollows::Default },
            last: None,
            application: None,
        })
        .unwrap();
    if settings_delivered {
        deliver(
            &mut root,
            settings,
            c::Delivery::Settings(Box::new(c::SettingsDocument {
                newest: 0,
                settings: lines,
                held: List::with_capacity(0),
                pending: List::with_capacity(0),
            })),
        );
    }
    let mut rows = List::with_capacity(1);
    if chat {
        rows.push(super::chats::line()).unwrap();
    }
    let mut refreshed = drive(
        &mut root,
        d::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched {
                    watch: live.expect("Home chats"),
                    event: c::WatchEvent::Delivered(c::Delivery::Tasks(Box::new(
                        chats.as_ref().map_or(c::Tasks { rows, more: false }, |chats| chats.live.clone()),
                    ))),
                },
            },
        },
    );
    while let Some(request) = refreshed.pop() {
        out.push(request);
    }
    settle_home(&mut root, &mut out, needs.as_ref(), chats.as_ref(), status);
    (root, accounts)
}
fn read_done(root: &mut d::Domain, read: Token, value: c::ReadValue) -> Queue<d::Request> {
    drive(
        root,
        d::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::ReadDone { read, result: jig_client::Readout::Jig(c::ReadResult::Value(value)) },
            },
        },
    )
}
fn home_need_page(object: c::ObjectKey) -> c::ReadValue {
    let mut task = f::task::document();
    task.chip.title = f::task::document_words(b"Add a README");
    match object {
        c::ObjectKey::Question { task: number, message } => {
            task.chip.task = number;
            if message == 7 {
                task.chip.title = f::task::document_words(b"");
            }
            c::ReadValue::Question(Box::new(c::Question {
                task: task.chip,
                from: None,
                message,
                words: c::DocumentText::complete(
                    c::Text::new((if message == 8 { HOME_QUESTION } else { b"Which license?" }).into(), 4096).unwrap(),
                ),
                since: skein_lib::Wall::EPOCH,
                acts: {
                    let mut acts = List::with_capacity(1);
                    acts.push(c::Offered { act: c::ActKind::Answer, admission: c::Admission::Allowed, effect: None })
                        .unwrap();
                    acts
                },
                ending: None,
            }))
        }
        c::ObjectKey::PersonTask(_) => c::ReadValue::PersonTask(Box::new(c::PersonTask {
            task: task.chip,
            revision: task.spec_revision,
            requester: task.requester,
            spec: task.spec,
            address: c::PersonAddress::Party(73),
            taken_by: None,
            since: Some(skein_lib::Wall::EPOCH),
            stall: None,
            deadline: None,
            contract: task.result_contract,
            results: List::with_capacity(0),
            cut: false,
            acts: List::with_capacity(0),
        })),
        c::ObjectKey::Escalation(number) => {
            let held = f::task::held_root();
            let mut task = held.pages().task().unwrap().document(held.client()).unwrap().clone();
            task.chip.task = number;
            task.chip.title = f::task::document_words(b"Add a README");
            if number == 10 {
                task.hold.as_mut().unwrap().reason = c::HoldCategory::Budget;
                task.budget.numbers.spent = 31;
                task.budget.numbers.budget = 40;
            }
            let mut acts = List::with_capacity(2);
            for act in [c::ActKind::Release, c::ActKind::LeaveHeld] {
                acts.push(c::Offered { act, admission: c::Admission::Allowed, effect: None }).unwrap();
            }
            c::ReadValue::Escalation(Box::new(c::Escalation { task: Box::new(task), acts }))
        }
        c::ObjectKey::Proposal(_) => super::cards::batch_proposal(),
        c::ObjectKey::Task(_) | c::ObjectKey::Result(_) | c::ObjectKey::Note { .. } => {
            c::ReadValue::Task(Box::new(task))
        }
    }
}
#[expect(
    clippy::disallowed_methods,
    clippy::too_many_lines,
    reason = "test fixtures and assertions inspect semantic nodes and public event deliveries"
)]
fn settle_home(
    root: &mut d::Domain,
    out: &mut Queue<d::Request>,
    needs: Option<&NeedsFixture<'_>>,
    chats: Option<&ChatFixture>,
    status: Option<&StatusFixture>,
) {
    while let Some(request) = out.pop() {
        let d::Request::Unlinked { request: jig_client::UnlinkedRequest::Down { down } } = request else { continue };
        let mut more = match down {
            jig_client::Down::Open { watch, subject }
                if status.and_then(|s| s.away.as_ref()).is_some()
                    && (Some(&subject) == d::forge::away::watch(&home_limits().forge).as_ref()
                        || Some(&subject) == d::forge::decided::watch(f::PROJECT, &home_limits().forge).as_ref()) =>
            {
                let fixture = status.unwrap().away.as_ref().unwrap();
                let bytes = if Some(&subject) == d::forge::away::watch(&home_limits().forge).as_ref() {
                    let Some(spell) = fixture.spell.clone() else { continue };
                    wire::aways(
                        &w::Aways::new(&w::CEILINGS, w::AwaysParts { rows: List::from_boxed(Box::new([spell])) })
                            .unwrap(),
                    )
                } else if Some(&subject) == d::forge::decided::watch(f::PROJECT, &home_limits().forge).as_ref() {
                    wire::decisions(
                        &w::Decisions::new(
                            &w::CEILINGS,
                            w::DecisionsParts { project: f::PROJECT, rows: fixture.rows.clone() },
                        )
                        .unwrap(),
                    )
                } else {
                    unreachable!("away fixture watch")
                };
                drive(
                    root,
                    d::Event::Client {
                        event: jig_client::Event::Up {
                            up: jig_client::Up::FamilyDelivered {
                                watch,
                                record: jig_client::Family {
                                    family: wire::FORGE,
                                    version: wire::CLIENT_VERSION,
                                    bytes,
                                },
                            },
                        },
                    },
                )
            }

            jig_client::Down::Open { watch, subject }
                if Some(&subject) == d::forge::status::watch(f::PROJECT, &home_limits().forge).as_ref()
                    && status.is_some() =>
            {
                let status = status.unwrap();
                drive(
                    root,
                    d::Event::Client {
                        event: jig_client::Event::Up {
                            up: jig_client::Up::FamilyDelivered {
                                watch,
                                record: jig_client::Family {
                                    family: wire::FORGE,
                                    version: wire::CLIENT_VERSION,
                                    bytes: wire::project_status(&status.document),
                                },
                            },
                        },
                    },
                )
            }
            jig_client::Down::Send { attempt, request, .. }
                if Some(&request) == d::forge::status::ask(f::PROJECT, &home_limits().forge).as_ref()
                    && status.is_some_and(|status| status.answered) =>
            {
                drive(
                    root,
                    d::Event::Client {
                        event: jig_client::Event::Up {
                            up: jig_client::Up::Answer {
                                attempt,
                                envelope: skein_lib::RequestEnvelope::Final,
                                why: None,
                                answer: Some(jig_client::Answered::Family(jig_client::Family {
                                    family: wire::FORGE,
                                    version: wire::CLIENT_VERSION,
                                    bytes: wire::answer(
                                        &w::Answer::new(&w::CEILINGS, w::AnswerParts { outcome: w::Outcome::Looked })
                                            .unwrap(),
                                    ),
                                })),
                            },
                        },
                    },
                )
            }
            jig_client::Down::Open { watch, subject: jig_client::Watchable::Jig(c::Subject::Tasks { query, .. }) }
                if query.tracked == Some(true) =>
            {
                if let Some(goals) = status.and_then(|fixture| fixture.goal_rows.as_ref()) {
                    let mut more = drive(
                        root,
                        d::Event::Client {
                            event: jig_client::Event::Up {
                                up: jig_client::Up::Watched {
                                    watch,
                                    event: c::WatchEvent::Delivered(c::Delivery::Tasks(Box::new(goals.live.clone()))),
                                },
                            },
                        },
                    );
                    while let Some(request) = more.pop() {
                        out.push(request);
                    }
                    continue;
                }
                let mut rows = List::uncapped();
                if status.is_some_and(|fixture| fixture.goals) {
                    let mut goal = super::chats::line();
                    goal.tracked = Some(1);
                    rows.push(goal).unwrap();
                }
                drive(
                    root,
                    d::Event::Client {
                        event: jig_client::Event::Up {
                            up: jig_client::Up::Watched {
                                watch,
                                event: c::WatchEvent::Delivered(c::Delivery::Tasks(Box::new(c::Tasks {
                                    rows,
                                    more: false,
                                }))),
                            },
                        },
                    },
                )
            }

            jig_client::Down::Open { watch, subject: jig_client::Watchable::Jig(c::Subject::Inbox { .. }) }
                if needs.is_some() =>
            {
                let value =
                    super::documents::inbox(List::with_capacity(0), 0, c::Count { value: 0, limited: false }, 0, false);
                drive(
                    root,
                    d::Event::Client {
                        event: jig_client::Event::Up {
                            up: jig_client::Up::Watched {
                                watch,
                                event: c::WatchEvent::Delivered(c::Delivery::Inbox(Box::new(value))),
                            },
                        },
                    },
                )
            }
            jig_client::Down::Read { read, what: jig_client::Readable::Jig(c::Read::Inbox { group, .. }) } => {
                let Some(needs) = needs else { continue };
                let mut rows = List::with_capacity(u32::try_from(needs.objects.len()).unwrap());
                if group == c::InboxReadGroup::Needs {
                    for object in needs.objects {
                        rows.push(super::documents::entry(*object, f::PROJECT, 1, f::NOW, None)).unwrap();
                    }
                }
                read_done(
                    root,
                    read,
                    c::ReadValue::Inbox(Box::new(c::InboxPage {
                        rows,
                        cut: group == c::InboxReadGroup::Needs && needs.cut,
                        before: None,
                        after: None,
                        selection: status.filter(|fixture| fixture.goals).map(|_| {
                            List::from_boxed(Box::new([c::InboxSelection {
                                cursor: c::InboxCursor::Number { number: 1 },
                                goals: List::from_boxed(Box::new([9])),
                            }]))
                        }),
                    })),
                )
            }
            jig_client::Down::Read { read, what: jig_client::Readable::Jig(c::Read::Question { task, message }) } => {
                if !needs.is_some_and(|needs| needs.read) {
                    continue;
                }
                read_done(root, read, home_need_page(c::ObjectKey::Question { task, message }))
            }
            jig_client::Down::Read { read, what: jig_client::Readable::Jig(c::Read::Proposal { proposal }) } => {
                if !needs.is_some_and(|needs| needs.read) {
                    continue;
                }
                read_done(root, read, home_need_page(c::ObjectKey::Proposal(proposal)))
            }
            jig_client::Down::Read { read, what: jig_client::Readable::Jig(c::Read::Escalation { task }) } => {
                if !needs.is_some_and(|needs| needs.read) {
                    continue;
                }
                read_done(root, read, home_need_page(c::ObjectKey::Escalation(task)))
            }
            jig_client::Down::Read { read, what: jig_client::Readable::Jig(c::Read::Task { task }) }
                if status.and_then(|s| s.away.as_ref()).is_some() =>
            {
                let fixture = status.unwrap().away.as_ref().unwrap();
                if fixture.state == d::pages::home::DecisionTask::Unknown {
                    continue;
                }
                let mut document = f::task::document();
                document.chip.task = task;
                document.chip.title = f::task::document_words(b"welcome-stall");
                match fixture.state {
                    d::pages::home::DecisionTask::Unknown => unreachable!(),
                    d::pages::home::DecisionTask::Live => {}
                    d::pages::home::DecisionTask::Done(at) | d::pages::home::DecisionTask::Ended(at) => {
                        document.chip.details.phase = c::ChipPhase::Ended;
                        document.chip.details.details.ended_at = Some(at);
                        document.chip.details.details.ending =
                            if let d::pages::home::DecisionTask::Done(_) = fixture.state {
                                Some(c::ChipEnding::Done(c::ResultValue::Report(f::task::document_words(b"Landed."))))
                            } else {
                                None
                            };
                    }
                }
                read_done(root, read, c::ReadValue::Task(Box::new(document)))
            }
            jig_client::Down::Read { read, what: jig_client::Readable::Jig(c::Read::PersonTask { task }) } => {
                if !needs.is_some_and(|needs| needs.read) {
                    continue;
                }
                read_done(root, read, home_need_page(c::ObjectKey::PersonTask(task)))
            }
            jig_client::Down::Read { read, what: jig_client::Readable::Jig(c::Read::Tasks { query, .. }) }
                if query.tracked == Some(true) && status.and_then(|fixture| fixture.goal_rows.as_ref()).is_some() =>
            {
                read_done(
                    root,
                    read,
                    c::ReadValue::Tasks(Box::new(status.unwrap().goal_rows.as_ref().unwrap().ended.clone())),
                )
            }
            jig_client::Down::Read { read, what: jig_client::Readable::Jig(c::Read::Notes { scope, .. }) }
                if status.is_some_and(|fixture| fixture.goals) =>
            {
                read_done(root, read, goal_notes(scope))
            }
            jig_client::Down::Read { read, what: jig_client::Readable::Jig(c::Read::Tasks { .. }) } => read_done(
                root,
                read,
                c::ReadValue::Tasks(Box::new(chats.map_or(
                    c::TaskPage {
                        live: List::with_capacity(0),
                        live_after: None,
                        ended: c::EndedGroup::Rows { rows: List::with_capacity(0), after: None, cut: false },
                    },
                    |chats| chats.ended.clone(),
                ))),
            ),
            jig_client::Down::Save { .. }
            | jig_client::Down::Erase { .. }
            | jig_client::Down::Send { .. }
            | jig_client::Down::Open { .. }
            | jig_client::Down::Continue { .. }
            | jig_client::Down::Close { .. }
            | jig_client::Down::Read { .. } => continue,
        };
        while let Some(request) = more.pop() {
            out.push(request);
        }
    }
}
fn forge_rows(in_project: bool) -> List<w::ForgeRow> {
    let mut rows = super::connect_forge::first_rows(super::connect_forge::connected(), super::connect_forge::made());
    if in_project {
        let mut parts = rows.get(0).unwrap().clone().into_parts();
        parts.repositories = List::with_capacity(1);
        parts
            .repositories
            .push(
                w::Adoption::new(
                    &w::CEILINGS,
                    w::AdoptionParts {
                        owner: b"work".as_slice().into(),
                        name: b"hello".as_slice().into(),
                        project: f::PROJECT,
                    },
                )
                .unwrap(),
            )
            .unwrap();
        *rows.get_mut(0).unwrap() = w::ForgeRow::new(&w::CEILINGS, parts).unwrap();
    }
    rows
}
fn contains(text: &[u8], words: &str) {
    assert!(skein_lib::bytes::find(text, words.as_bytes()).is_some(), "missing {words}: {text:?}");
}
fn absent(text: &[u8], words: &str) {
    assert!(skein_lib::bytes::find(text, words.as_bytes()).is_none(), "unexpected {words}");
}
#[expect(clippy::disallowed_methods, reason = "test assertions inspect semantic nodes and their event bindings")]
fn styled(tree: &skein_ui::Tree, style: skein_ui::Style) -> &skein_ui::Node {
    main_nodes(tree).iter().find(|node| node.styles.as_slice().contains(&style)).expect("styled Home node")
}
fn children(nodes: &[skein_ui::Node]) -> Box<[&skein_ui::Node]> {
    let mut children = List::with_capacity(u32::try_from(nodes.len()).unwrap());
    let mut at = 1;
    while at < nodes.len() {
        children.push(&nodes[at]).unwrap();
        at = at.checked_add(usize::try_from(nodes[at].size).unwrap()).unwrap();
    }
    children.into_boxed()
}
fn assert_action(view: &mut crate::View, role: Role, name: &[u8], act: d::pages::Act) {
    let id = main_find(view.tree(), role, name);
    let Some(d::Event::Act { action }) = crate::event(view, id, Act::Press) else { panic!("page act event") };
    assert_eq!(action, d::Action::Pages(act.clone()));
    assert_eq!(crate::act(view, id, Act::Press), Some(d::Action::Pages(act)));
}

#[test]
#[expect(clippy::disallowed_methods, reason = "test assertions inspect semantic nodes and their event bindings")]
fn home_ready_headline_and_done_steps_hold_in_both_shells() {
    for configured in [false, true] {
        for repository in [false, true] {
            for chat in [false, true] {
                let (root, _) =
                    home_root(if repository { forge_rows(true) } else { List::with_capacity(0) }, configured, chat);
                let left = u8::from(!configured) + u8::from(!repository) + u8::from(!chat);
                let headline = if left > 0 { "Shop is ready." } else { "Shop" };
                for view in [f::page_view(&root), f::browser_page_view(&root)] {
                    main_find(view.tree(), Role::Heading { level: 1 }, headline.as_bytes());
                    assert_eq!(
                        main_nodes(view.tree())
                            .iter()
                            .filter(|node| node.styles.as_slice().contains(&crate::look::HOME_DONE))
                            .count(),
                        if left == 0 { 0 } else { usize::from(3 - left) }
                    );
                    for (role, name, done) in
                        [(Role::Button, b"Start".as_slice(), configured), (Role::Link, b"New chat", chat)]
                    {
                        assert_eq!(
                            main_nodes(view.tree())
                                .iter()
                                .any(|node| node.role == role && node.name.as_deref() == Some(name)),
                            !done
                        );
                    }
                    let main = f::main_text(view.tree());
                    if left > 0 {
                        assert_eq!(
                            skein_lib::bytes::find(
                                &main,
                                b"In your own words: how you plan, review and land work. temper proposes how it will run that, and nothing changes until you accept."
                            )
                            .is_some(),
                            !configured
                        );
                        assert_eq!(
                            skein_lib::bytes::find(&main, b"Ask a question or ask for a change.").is_some(),
                            !chat
                        );
                        if repository {
                            contains(&main, "work/hello is in Shop");
                            absent(&main, "CI ran on main and passed");
                            contains(&main, "Chats can change its code now.");
                        }
                    }
                }
                let terminal = printed(&f::page_view(&root));
                assert!(terminal.strip_prefix(b"== ").unwrap().starts_with(headline.as_bytes()));
                absent(&terminal, "Connected");
                for words in ["To begin", "1. ", "2. ", "3. "] {
                    if left > 0 {
                        contains(&terminal, words);
                    } else {
                        absent(&terminal, words);
                    }
                }
            }
        }
    }
}

#[expect(clippy::disallowed_methods, reason = "test assertions inspect semantic nodes and their event bindings")]
fn adopting(root: &mut d::Domain, refused: Option<w::ForgeReason>) {
    let mut out = drive(root, d::Event::Act { action: d::Action::Pages(d::pages::Act::AdoptFirst) });
    let mut stored = drive(root, d::Event::Client { event: jig_client::Event::Stored });
    while let Some(request) = stored.pop() {
        out.push(request);
    }
    if let Some(reason) = refused {
        let attempt = out
            .iter()
            .find_map(|request| match request {
                d::Request::Unlinked {
                    request: jig_client::UnlinkedRequest::Down { down: jig_client::Down::Send { attempt, .. } },
                } => Some(*attempt),
                d::Request::Client { .. }
                | d::Request::Unlinked { .. }
                | d::Request::Show { .. }
                | d::Request::Leave { .. }
                | d::Request::SignOut
                | d::Request::Dropped { .. }
                | d::Request::Tab { .. }
                | d::Request::Fact { .. } => None,
            })
            .expect("adoption request");
        let answer = w::Answer::new(
            &w::CEILINGS,
            w::AnswerParts {
                outcome: w::Outcome::ForgeRefused(
                    w::ForgeRefused::new(&w::CEILINGS, w::ForgeRefusedParts { reason }).unwrap(),
                ),
            },
        )
        .unwrap();
        drive(
            root,
            d::Event::Client {
                event: jig_client::Event::Up {
                    up: jig_client::Up::Answer {
                        attempt,
                        envelope: skein_lib::RequestEnvelope::Final,
                        why: None,
                        answer: Some(jig_client::Answered::Family(jig_client::Family {
                            family: wire::FORGE,
                            version: wire::CLIENT_VERSION,
                            bytes: wire::answer(&answer),
                        })),
                    },
                },
            },
        );
    }
}
#[test]
#[expect(clippy::disallowed_methods, reason = "test assertions inspect semantic nodes and their event bindings")]
#[expect(clippy::too_many_lines, reason = "the repository state matrix also checks every shared forge refusal")]
fn home_repository_states_keep_their_words_and_events() {
    for state in ["offered", "adopting", "refused", "none", "github"] {
        let mut rows = if state == "none" || state == "github" { List::with_capacity(1) } else { forge_rows(false) };
        if state == "github" {
            let mut parts = super::connect_forge::row(
                2,
                w::Kind::Github,
                w::Mode::Managed,
                super::connect_forge::connected(),
                w::Offer::NotOffered,
            )
            .into_parts();
            parts.installations = List::with_capacity(1);
            parts
                .installations
                .push(
                    w::Installation::new(
                        &w::CEILINGS,
                        w::InstallationParts {
                            id: 1,
                            account: Some(
                                w::Account::new(
                                    &w::CEILINGS,
                                    w::AccountParts { id: 73, login: b"octo".as_slice().into() },
                                )
                                .unwrap(),
                            ),
                            suspended: false,
                            stray: false,
                        },
                    )
                    .unwrap(),
                )
                .unwrap();
            rows.push(w::ForgeRow::new(&w::CEILINGS, parts).unwrap()).unwrap();
        }
        let (mut root, _) = home_root(rows, false, false);
        if state == "adopting" || state == "refused" {
            adopting(&mut root, if state == "refused" { Some(w::ForgeReason::HomeHeld) } else { None });
        }
        for mut view in [f::page_view(&root), f::browser_page_view(&root)] {
            let words = f::main_text(view.tree());
            absent(&words, "Shop has no repository yet.");
            if state == "adopting" {
                contains(&words, "Adopting work/hello into Shop");
                contains(&words, "Chats can change its code once it is in.");
                assert!(!main_nodes(view.tree()).iter().any(|node| node.name.as_deref() == Some(b"Adopt")));
            } else if state == "offered" || state == "refused" {
                contains(&words, "work/hello, made on your Forgejo at set up, is in no project yet.");
                assert_action(&mut view, Role::Button, b"Adopt", d::pages::Act::AdoptFirst);
                if state == "refused" {
                    let mut b = skein_ui::Builder::new(&f::tree_limits());
                    crate::forge::page::outcome(
                        &mut b,
                        (b"work", b"hello", b"temper/", b"3"),
                        &d::forge::Answer::ForgeRefused(d::forge::Reason::HomeHeld),
                        &f::words(),
                        512,
                    );
                    let expected = f::text(&b.finish());
                    assert!(skein_lib::bytes::find(&words, &expected).is_some());
                }
            } else {
                contains(
                    &words,
                    if state == "github" {
                        "temper’s App reaches its repositories on octo."
                    } else {
                        "Until one is in, chats talk and plan but change no code."
                    },
                );
                assert_action(
                    &mut view,
                    Role::Link,
                    b"Adopt",
                    d::pages::Act::Go { address: Address::Adopt { project: f::PROJECT } },
                );
            }
            assert_action(
                &mut view,
                Role::Button,
                b"Start",
                d::pages::Act::Go { address: Address::Configure { project: f::PROJECT } },
            );
            assert_action(
                &mut view,
                Role::Link,
                b"New chat",
                d::pages::Act::Go { address: Address::NewChat { draft: false } },
            );
        }
        let terminal = printed(&f::page_view(&root));
        contains(&terminal, if state == "adopting" { "Adopting work/hello into Shop" } else { "Adopt a repository" });
    }
    for (wire_reason, reason) in [
        (w::ForgeReason::NoSuchForge, d::forge::Reason::NoSuchForge),
        (w::ForgeReason::NotFound, d::forge::Reason::NotFound),
        (w::ForgeReason::NoAccess, d::forge::Reason::NoAccess),
        (w::ForgeReason::Refused, d::forge::Reason::Refused),
        (w::ForgeReason::AnotherProject, d::forge::Reason::AnotherProject),
        (w::ForgeReason::HomeHeld, d::forge::Reason::HomeHeld),
        (w::ForgeReason::PrefixOccupied, d::forge::Reason::PrefixOccupied),
        (w::ForgeReason::InFlight, d::forge::Reason::InFlight),
        (w::ForgeReason::TooLarge, d::forge::Reason::TooLarge),
        (w::ForgeReason::Unreachable, d::forge::Reason::Unreachable),
        (w::ForgeReason::InvalidAnswer, d::forge::Reason::InvalidAnswer),
    ] {
        let (mut root, _) = home_root(forge_rows(false), false, false);
        adopting(&mut root, Some(wire_reason));
        let mut b = skein_ui::Builder::new(&f::tree_limits());
        crate::forge::page::outcome(
            &mut b,
            (b"work", b"hello", b"temper/", b"3"),
            &d::forge::Answer::ForgeRefused(reason),
            &f::words(),
            512,
        );
        let expected = f::text(&b.finish());
        for view in [f::page_view(&root), f::browser_page_view(&root)] {
            assert!(skein_lib::bytes::find(&f::main_text(view.tree()), &expected).is_some());
        }
    }
}

#[test]
#[expect(clippy::disallowed_methods, reason = "test assertions inspect semantic nodes and their event bindings")]
fn home_outline_matches_the_requested_main_and_terminal_lines_join() {
    let (mut root, accounts) = home_root(forge_rows(true), false, false);
    super::accounts::deliver(
        &mut root,
        accounts,
        super::accounts::card_document(b"openai", b"", c::AccountKind::SignIn),
    );
    let mut view = f::view_of(f::browser_words());
    let mut sign_in = crate::forge_sign_in::Forges::new();
    sign_in.read("3 forgejo http://forge.example:3000 owned");
    view.forge_sign_in(sign_in);
    f::build(&mut view, &root);
    let tree = view.tree();
    let main = main_nodes(tree);
    assert_eq!(main[0].styles.as_slice(), &[crate::look::MAIN, crate::look::HOME]);
    let top = children(main);
    assert_eq!(top.len(), 2);
    for (node, (style, role)) in
        top.iter().zip([(crate::look::HOME_INTRO, Role::Slot), (crate::look::HOME_BEGIN, Role::Region)])
    {
        assert_eq!(node.styles.as_slice(), &[style]);
        assert_eq!(node.role, role);
    }
    let intro = children(subtree(tree, top[0].id));
    assert_eq!(intro.len(), 3);
    for (node, role) in intro.iter().zip([Role::Paragraph, Role::Heading { level: 1 }, Role::Slot]) {
        assert_eq!(node.role, role);
    }
    assert_eq!(intro[0].styles.as_slice(), &[crate::look::HOME_KICKER]);
    assert_eq!(intro[2].styles.as_slice(), &[crate::look::HOME_LEDES]);
    contains(
        &f::node_text(tree, tree.nodes().iter().position(|node| node.id == intro[2].id).unwrap()),
        "Your agents run on your ChatGPT plan, and Forgejo on this machine.",
    );
    let begin = children(subtree(tree, top[1].id));
    assert_eq!(begin.len(), 3);
    assert_eq!(begin[0].styles.as_slice(), &[crate::look::HOME_HEAD]);
    let head = children(subtree(tree, begin[0].id));
    assert_eq!(head.len(), 3);
    assert_eq!((head[0].role, head[0].name.as_deref()), (Role::Heading { level: 2 }, Some(b"To begin".as_slice())));
    assert_eq!(head[1].role, Role::Text);
    assert_eq!(f::nodes_text(subtree(tree, head[1].id)).as_ref(), b"Optional");
    assert_eq!(head[1].styles.as_slice(), &[crate::look::HOME_TAG]);
    assert_eq!((head[2].role, head[2].name.as_deref()), (Role::Button, Some(b"Skip".as_slice())));
    assert_eq!(head[2].styles.as_slice(), &[crate::look::HOME_SKIP]);
    assert_eq!(begin[1].styles.as_slice(), &[crate::look::HOME_NOTE]);
    assert_eq!(begin[2].role, Role::List);
    assert_eq!(begin[2].styles.as_slice(), &[crate::look::HOME_STEPS]);
    let steps = children(subtree(tree, begin[2].id));
    assert_eq!(steps.len(), 3);
    for (step, done) in steps.iter().zip([false, true, false]) {
        assert_eq!(step.role, Role::Item);
        assert!(step.styles.as_slice().contains(&crate::look::HOME_STEP));
        assert_eq!(step.styles.as_slice().contains(&crate::look::HOME_DONE), done);
        let parts = children(subtree(tree, step.id));
        assert_eq!(parts[0].role, Role::Heading { level: 3 });
        assert_eq!(parts[1].role, Role::Paragraph);
        assert_eq!(parts.len(), if done { 2 } else { 3 });
        if !done {
            assert_eq!(parts[2].styles.as_slice(), &[crate::look::HOME_ACTS]);
        }
    }
    assert_action(&mut view, Role::Button, b"Skip", d::pages::Act::SkipToBegin);
    assert!(
        main_nodes(view.tree())
            .iter()
            .find(|node| node.role == Role::Link && node.name.as_deref() == Some(b"New chat"))
            .unwrap()
            .styles
            .is_empty()
    );
    let mut terminal = f::view();
    let mut sign_in = crate::forge_sign_in::Forges::new();
    sign_in.read("3 forgejo http://forge.example:3000 owned");
    terminal.forge_sign_in(sign_in);
    f::build(&mut terminal, &root);
    let printed = printed(&terminal);
    contains(&printed, "To begin");
    contains(&printed, "Optional Skip\n");
    contains(&printed, "1. Tell temper how you work");
    contains(&printed, "2. Done. work/hello is in Shop");
    contains(&printed, "3. Start a chat");
    absent(&printed, "Connected");
    absent(&printed, "Change model");
}

#[test]
fn home_kicker_tracks_the_current_day_and_minute() {
    let (root, _) = home_root(List::with_capacity(0), false, false);
    let mut view = f::browser_page_view(&root);
    let kicker = styled(view.tree(), crate::look::HOME_KICKER).name.as_deref().unwrap();
    contains(kicker, "Thursday,");
    contains(kicker, "00:00");
    absent(kicker, "Shop");
    assert_eq!(view.next_change(), Some(skein_lib::Wall::from_nanos(60_000_000_000)));
    crate::build(&mut view, &root, skein_lib::Wall::from_nanos(60_000_000_000), &mut Queue::uncapped());
    contains(styled(view.tree(), crate::look::HOME_KICKER).name.as_deref().unwrap(), "00:01");
}

#[test]
fn home_unknown_and_ended_forges_draw_only_known_facts() {
    let (mut root, _, _, _, _, watch) =
        f::root_at_with_forges(Address::Home { project: f::PROJECT }, &f::domain_limits(), b"Shop", None, None, None);
    // An ended watch cannot claim that the project has no repository.
    let watch = watch.expect("forges watch");
    drive(
        &mut root,
        d::Event::Client {
            event: jig_client::Event::Up { up: jig_client::Up::WatchEnded { watch, end: c::WatchEnd::NotAllowed } },
        },
    );
    for view in [f::page_view(&root), f::browser_page_view(&root)] {
        let text = f::main_text(view.tree());
        absent(&text, "Until one is in, chats talk and plan but change no code.");
        absent(&text, "Shop has no repository yet.");
        absent(&text, "connected as");
    }
    let mut root = d::unlinked(&f::domain_limits(), f::SEED);
    drive(
        &mut root,
        d::Event::Start { address: Address::Home { project: f::PROJECT }, kept: List::with_capacity(0), with: None },
    );
    for event in [
        jig_client::Event::StartUnlinked { kept: List::with_capacity(0) },
        jig_client::Event::SignIn { with: jig_client::With::Transport },
        jig_client::Event::Up { up: jig_client::Up::Served { served: true, versions: None } },
    ] {
        drive(&mut root, d::Event::Client { event });
    }
    for view in [f::page_view(&root), f::browser_page_view(&root)] {
        let text = f::main_text(view.tree());
        absent(&text, "Adopt a repository");
        absent(&text, "Until one is in");
        absent(&text, "has no repository yet");
        absent(&text, "To begin");
    }
}

// One featured need plus six others proves Home's five-row limit on both sides.
const QUESTIONS: [c::ObjectKey; 7] = [
    c::ObjectKey::Question { task: f::INBOX_TASK, message: 1 },
    c::ObjectKey::Question { task: f::INBOX_TASK, message: 2 },
    c::ObjectKey::Question { task: f::INBOX_TASK, message: 3 },
    c::ObjectKey::Question { task: f::INBOX_TASK, message: 4 },
    c::ObjectKey::Question { task: f::INBOX_TASK, message: 5 },
    c::ObjectKey::Question { task: f::INBOX_TASK, message: 6 },
    c::ObjectKey::Question { task: f::INBOX_TASK, message: 7 },
];
fn everyday(beginning: bool, objects: &[c::ObjectKey], cut: bool, read: bool) -> d::Domain {
    home_with_needs(forge_rows(true), !beginning, true, Some(NeedsFixture { objects, cut, read })).0
}
fn inbox_act() -> d::pages::Act {
    d::pages::Act::Go {
        address: Address::Inbox { filter: d::address::Filter { project: Some(f::PROJECT), kind: None, goal: None } },
    }
}
#[expect(
    clippy::disallowed_methods,
    reason = "test fixtures and assertions inspect semantic nodes and public event deliveries"
)]
fn section_styles(tree: &skein_ui::Tree) -> Box<[skein_ui::Style]> {
    children(main_nodes(tree)).iter().map(|node| node.styles.as_slice()[0]).collect()
}
#[test]
#[expect(
    clippy::disallowed_methods,
    reason = "test fixtures and assertions inspect semantic nodes and public event deliveries"
)]
fn everyday_headlines_count_known_needs_and_mark_cut_reads() {
    let questions: Box<[_]> = (1..=11).map(|message| c::ObjectKey::Question { task: f::INBOX_TASK, message }).collect();
    for (objects, cut, headline) in [
        (&QUESTIONS[..0], false, "Nothing needs you."),
        (&QUESTIONS[..1], false, "One thing needs you."),
        (&questions[..2], false, "Two things need you."),
        (&questions[..3], false, "Three things need you."),
        (&questions[..4], false, "Four things need you."),
        (&questions[..5], false, "Five things need you."),
        (&questions[..6], false, "Six things need you."),
        (&questions[..7], false, "Seven things need you."),
        (&questions[..8], false, "Eight things need you."),
        (&questions[..9], false, "Nine things need you."),
        (&questions[..10], false, "10 things need you."),
        (&questions[..11], false, "11 things need you."),
        (&QUESTIONS[..1], true, "1+ things need you."),
        (&QUESTIONS[..3], true, "3+ things need you."),
    ] {
        let root = everyday(false, objects, cut, true);
        for view in [f::page_view(&root), f::browser_page_view(&root)] {
            main_find(view.tree(), Role::Heading { level: 1 }, headline.as_bytes());
            assert_eq!(
                section_styles(view.tree()).as_ref(),
                [crate::look::HOME_INTRO, crate::look::HOME_NEEDS, crate::look::HOME_CHATS]
            );
            assert!(
                !main_nodes(view.tree()).iter().any(|node| node.styles.as_slice().contains(&crate::look::HOME_LEDE))
            );
            if objects.is_empty() {
                main_find(
                    view.tree(),
                    Role::Paragraph,
                    b"Nothing needs you. Work carries on, and this page tells you what changed when you come back.",
                );
                let heading = main_find(view.tree(), Role::Heading { level: 2 }, b"Needs you");
                assert!(!subtree(view.tree(), heading)[0].styles.as_slice().contains(&crate::look::HOME_SIGNAL));
            }
        }
    }
}
#[test]
#[expect(
    clippy::disallowed_methods,
    reason = "test fixtures and assertions inspect semantic nodes and public event deliveries"
)]
fn first_need_keeps_card_order_navigation_and_terminal_words() {
    for beginning in [false, true] {
        let root = everyday(beginning, &QUESTIONS[..1], false, true);
        for (mut view, browser) in [(f::page_view(&root), false), (f::browser_page_view(&root), true)] {
            let tree = view.tree();
            let top = section_styles(tree);
            let expected: &[skein_ui::Style] = if beginning {
                &[crate::look::HOME_INTRO, crate::look::HOME_BEGIN, crate::look::HOME_NEEDS, crate::look::HOME_CHATS]
            } else {
                &[crate::look::HOME_INTRO, crate::look::HOME_NEEDS, crate::look::HOME_CHATS]
            };
            assert_eq!(top.as_ref(), expected);
            let region = styled(tree, crate::look::HOME_NEEDS);
            assert_eq!(region.role, Role::Region);
            assert_eq!(region.name.as_deref(), Some(b"Needs you".as_slice()));
            let parts = children(subtree(tree, region.id));
            assert_eq!(parts.len(), if browser && !beginning { 3 } else { 2 });
            assert_eq!(parts[0].styles.as_slice(), &[crate::look::HOME_HEAD]);
            assert_eq!(parts[1].styles.as_slice(), &[crate::look::ASK]);
            let heading = children(subtree(tree, parts[0].id));
            assert_eq!(heading.len(), 1);
            assert_eq!(heading[0].name.as_deref(), Some("Needs you · 1".as_bytes()));
            assert_eq!(heading[0].styles.as_slice(), &[crate::look::HOME_SIGNAL]);
            let card = children(subtree(tree, parts[1].id));
            assert_eq!(
                card.iter().map(|node| node.styles.as_slice()[0]).collect::<Box<[_]>>().as_ref(),
                [crate::look::ASK_META, crate::look::ASK_QUESTION, crate::look::ASK_CHOICES]
            );
            assert_eq!(card[0].name.as_deref(), Some("Add a README · a question · under 1 min".as_bytes()));
            assert_eq!(card[1].name.as_deref(), Some(b"Which license?".as_slice()));
            let choices = children(subtree(tree, card[2].id));
            assert_eq!(choices.len(), 2);
            assert_eq!(choices[0].styles.as_slice(), &[crate::look::ASK_QUIET]);
            if browser && !beginning {
                assert_eq!(parts[2].styles.as_slice(), &[crate::look::HOME_CTA]);
                let spans = children(subtree(tree, parts[2].id));
                assert_eq!(spans.len(), 2);
                assert_eq!(f::nodes_text(subtree(tree, spans[0].id)).as_ref(), b"Add a README: Which license?");
                assert_eq!(spans[1].styles.as_slice(), &[crate::look::HOME_CTA_GO]);
                assert_action(&mut view, Role::Link, b"Add a README: Which license? Answer", inbox_act());
            }
            assert_action(
                &mut view,
                Role::Link,
                b"Add a README",
                d::pages::Act::Go { address: Address::Task { task: f::INBOX_TASK } },
            );
            assert_action(
                &mut view,
                Role::Button,
                b"Reply",
                d::pages::Act::Needs(d::pages::needs::Act::OpenReply { object: QUESTIONS[0] }),
            );
            assert_action(
                &mut view,
                Role::Link,
                b"All 1 chats",
                d::pages::Act::Go { address: Address::Chats { filter: ChatsFilter::All } },
            );
        }
        let print = printed(&f::page_view(&root));
        for words in ["Needs you · 1", "Add a README · a question · under 1 min", "Which license?", "Reply", "Chats"]
        {
            contains(&print, words);
        }
        absent(&print, "Answer");
    }
}
#[test]
#[expect(
    clippy::disallowed_methods,
    reason = "test fixtures and assertions inspect semantic nodes and public event deliveries"
)]
fn unread_need_has_only_meta_and_unknown_headline() {
    let root = everyday(false, &QUESTIONS[..1], false, false);
    for mut view in [f::page_view(&root), f::browser_page_view(&root)] {
        main_find(view.tree(), Role::Heading { level: 1 }, b"Shop");
        let card = children(subtree(view.tree(), styled(view.tree(), crate::look::ASK).id));
        assert_eq!(card.len(), 1);
        assert_eq!(card[0].styles.as_slice(), &[crate::look::ASK_META]);
        assert_eq!(card[0].name.as_deref(), Some("T9 · a question".as_bytes()));
        assert!(!main_nodes(view.tree()).iter().any(|node| node.styles.as_slice().contains(&crate::look::HOME_CTA)));
        assert_action(
            &mut view,
            Role::Link,
            b"T9",
            d::pages::Act::Go { address: Address::Task { task: f::INBOX_TASK } },
        );
    }
}
#[test]
#[expect(
    clippy::disallowed_methods,
    reason = "test fixtures and assertions inspect semantic nodes and public event deliveries"
)]
fn additional_needs_have_five_rows_and_browser_answer_link() {
    // Six total fits exactly; seven total must still draw only five secondary rows.
    for objects in [&QUESTIONS[..3], &QUESTIONS[..6], &QUESTIONS[..7]] {
        let root = everyday(false, objects, false, true);
        for (mut view, browser) in [(f::page_view(&root), false), (f::browser_page_view(&root), true)] {
            let region = styled(view.tree(), crate::look::HOME_NEEDS);
            let rows = subtree(view.tree(), region.id).iter().find(|node| node.role == Role::List).unwrap();
            let items = children(subtree(view.tree(), rows.id));
            assert_eq!(items.len(), (objects.len() - 1).min(5));
            let ids: Box<[_]> = items
                .iter()
                .map(|item| {
                    assert_eq!(item.role, Role::Item);
                    let link = children(subtree(view.tree(), item.id));
                    assert_eq!(link.len(), 1);
                    assert_eq!(link[0].role, Role::Link);
                    let spans = children(subtree(view.tree(), link[0].id));
                    assert_eq!(spans.len(), 2);
                    assert_eq!(spans[1].styles.as_slice(), &[crate::look::HOME_WHEN]);
                    link[0].id
                })
                .collect();
            for (id, object) in ids.iter().zip(objects.iter().skip(1)) {
                assert_eq!(
                    crate::act(&mut view, *id, Act::Press),
                    Some(d::Action::Pages(d::pages::Act::Needs(d::pages::needs::Act::Open { object: *object })))
                );
            }
            assert_eq!(
                main_nodes(view.tree()).iter().any(|node| node.styles.as_slice().contains(&crate::look::HOME_IN_A_ROW)),
                browser
            );
            if browser {
                assert_action(&mut view, Role::Link, b"Answer in a row", inbox_act());
            }
        }
    }
}
#[test]
#[expect(
    clippy::disallowed_methods,
    reason = "test fixtures and assertions inspect semantic nodes and public event deliveries"
)]
fn nonquestion_need_opens_inbox_and_keeps_its_lowercase_kind() {
    for (object, kind) in [
        (c::ObjectKey::PersonTask(f::INBOX_TASK), "a person task"),
        (c::ObjectKey::Escalation(f::INBOX_TASK), "held"),
        (
            c::ObjectKey::Proposal(c::ProposalKey {
                proposer: c::Proposer::Task(f::INBOX_TASK),
                number: f::INBOX_QUESTION,
            }),
            "a proposal",
        ),
    ] {
        let root = everyday(false, &[object], false, true);
        for mut view in [f::page_view(&root), f::browser_page_view(&root)] {
            contains(&f::main_text(view.tree()), kind);
            assert_action(
                &mut view,
                Role::Button,
                b"Open",
                d::pages::Act::Go { address: Address::Task { task: f::INBOX_TASK } },
            );
            assert!(
                !main_nodes(view.tree())
                    .iter()
                    .any(|node| node.role == Role::Button && node.name.as_deref() == Some(b"Reply"))
            );
        }
    }
}

// Home's span cases build four minutes after its fixture's epoch.
const FOUR_MINUTES: skein_lib::Wall = skein_lib::Wall::from_nanos(240_000_000_000);
fn timed_view(root: &d::Domain, browser: bool) -> crate::View {
    let mut view = f::view_of(if browser { f::browser_words() } else { f::words() });
    crate::build(&mut view, root, FOUR_MINUTES, &mut Queue::uncapped());
    view
}
// Six rows is one beyond Home's five-row cap; a seventh can be an ended chat.
const CHAT_TITLES: [&[u8]; 7] = [b"README", b"Rust", b"Repository", b"Tests", b"Docs", b"Checks", b"Release"];
fn chat_fixture(live: usize, ended: usize, more: bool, after: bool) -> ChatFixture {
    let mut live_rows = List::with_capacity(u32::try_from(live).unwrap());
    let mut closed_rows = List::with_capacity(u32::try_from(ended).unwrap());
    for (index, title) in CHAT_TITLES.iter().take(live.checked_add(ended).unwrap()).enumerate() {
        let mut row = super::chats::line();
        row.chip.task = row.chip.task.checked_add(u64::try_from(index).unwrap()).unwrap();
        row.chip.title = f::task::document_words(title);
        if index == 0 {
            row.waits_on = c::WaitsOn::Party {
                whom: c::Whom::Person(73),
                entries: 1,
                task: row.chip.task,
                kind: c::WaitedFor::Question,
            };
        }
        if index < live {
            live_rows.push(row).unwrap();
        } else {
            row.ended_at = Some(skein_lib::Wall::from_nanos(180_000_000_000));
            closed_rows.push(row).unwrap();
        }
    }
    ChatFixture {
        live: c::Tasks { rows: live_rows, more },
        ended: c::TaskPage {
            live: List::with_capacity(0),
            live_after: None,
            ended: c::EndedGroup::Rows {
                rows: closed_rows,
                after: if after { Some(c::Cursor::new(b"next".as_slice().into(), 32).unwrap()) } else { None },
                cut: false,
            },
        },
    }
}
fn home_chats(chats: ChatFixture) -> d::Domain {
    home_documents(
        forge_rows(true),
        true,
        true,
        Some(NeedsFixture { objects: &[], cut: false, read: true }),
        Some(chats),
    )
    .0
}
#[test]
fn chats_keep_live_then_ended_status_age_names_and_events() {
    for (more, after, name, label) in [
        (false, false, "All 3 chats", "All 3"),
        (true, false, "All 3+ chats", "All 3+"),
        (false, true, "All 3+ chats", "All 3+"),
    ] {
        let root = home_chats(chat_fixture(2, 1, more, after));
        for browser in [false, true] {
            let mut view = timed_view(&root, browser);
            let region = styled(view.tree(), crate::look::HOME_CHATS);
            assert_eq!(region.role, Role::Region);
            assert_eq!(region.name.as_deref(), Some(b"Chats".as_slice()));
            let parts = children(subtree(view.tree(), region.id));
            assert_eq!(parts.len(), 3);
            assert_eq!(parts[0].styles.as_slice(), &[crate::look::HOME_HEAD]);
            assert_eq!(parts[1].styles.as_slice(), &[crate::look::HOME_ROWS]);
            let heading = children(subtree(view.tree(), parts[0].id));
            assert_eq!(heading[0].role, Role::Heading { level: 2 });
            assert_eq!(heading[1].name.as_deref(), Some(name.as_bytes()));
            assert_eq!(f::nodes_text(subtree(view.tree(), heading[1].id)).as_ref(), label.as_bytes());
            let rows = children(subtree(view.tree(), parts[1].id));
            assert_eq!(rows.len(), 3);
            for (row, (name, title, when, task)) in rows.iter().zip([
                ("README, waiting on you, 4 min", "README", "waiting on you · 4 min", 9),
                ("Rust, Parked, 4 min", "Rust", "Parked · 4 min", 10),
                ("Repository, closed, 1 min", "Repository", "closed · 1 min", 11),
            ]) {
                assert_eq!(row.role, Role::Item);
                let links = children(subtree(view.tree(), row.id));
                assert_eq!(links.len(), 1);
                assert_eq!(links[0].role, Role::Link);
                assert_eq!(links[0].name.as_deref(), Some(name.as_bytes()));
                let spans = children(subtree(view.tree(), links[0].id));
                assert_eq!(f::nodes_text(subtree(view.tree(), spans[0].id)).as_ref(), title.as_bytes());
                assert_eq!(spans[1].styles.as_slice(), &[crate::look::HOME_WHEN]);
                contains(&f::nodes_text(subtree(view.tree(), spans[1].id)), when);
                let id = links[0].id;
                let Some(d::Event::Act { action }) = crate::event(&view, id, Act::Press) else {
                    panic!("chat navigation");
                };
                assert_eq!(action, d::Action::Pages(d::pages::Act::Go { address: Address::Chat { task } }));
            }
            assert_action(
                &mut view,
                Role::Link,
                name.as_bytes(),
                d::pages::Act::Go { address: Address::Chats { filter: ChatsFilter::All } },
            );
            if !browser {
                let print = printed(&view);
                contains(&print, "README · waiting on you · 4 min");
                contains(&print, "Repository · closed · 1 min");
            }
        }
    }
}
#[test]
fn chat_rows_stop_at_five_without_changing_the_total() {
    for (live, ended, label) in [(5, 0, "All 5 chats"), (6, 0, "All 6 chats"), (2, 4, "All 6 chats")] {
        let root = home_chats(chat_fixture(live, ended, false, false));
        for view in [f::page_view(&root), f::browser_page_view(&root)] {
            let region = styled(view.tree(), crate::look::HOME_CHATS);
            let parts = children(subtree(view.tree(), region.id));
            assert_eq!(children(subtree(view.tree(), parts[1].id)).len(), 5);
            main_find(view.tree(), Role::Link, label.as_bytes());
            absent(&f::main_text(view.tree()), "Checks");
        }
    }
}
#[test]
#[expect(
    clippy::disallowed_methods,
    reason = "test fixtures and assertions inspect semantic nodes and public event deliveries"
)]
fn empty_sections_are_hidden_only_while_to_begin_shows() {
    for shown in [false, true] {
        let mut root =
            home_with_needs(forge_rows(true), true, false, Some(NeedsFixture { objects: &[], cut: false, read: true }))
                .0;
        if !shown {
            drive(&mut root, d::Event::Act { action: d::Action::Pages(d::pages::Act::SkipToBegin) });
        }
        for view in [f::page_view(&root), f::browser_page_view(&root)] {
            let styles = section_styles(view.tree());
            assert_eq!(styles.contains(&crate::look::HOME_NEEDS), !shown);
            assert_eq!(styles.contains(&crate::look::HOME_CHATS), !shown);
            if !shown {
                main_find(view.tree(), Role::Paragraph, b"No chats yet.");
                assert!(!main_nodes(view.tree()).iter().any(|node| node.role == Role::Link
                    && node.name.as_deref().is_some_and(|name| name.starts_with(b"All "))));
            }
        }
    }
}

#[test]
fn read_need_without_a_title_links_by_task_number() {
    let root = everyday(false, &QUESTIONS[6..], false, true);
    for mut view in [f::page_view(&root), f::browser_page_view(&root)] {
        assert_action(
            &mut view,
            Role::Link,
            b"Task 9",
            d::pages::Act::Go { address: Address::Task { task: f::INBOX_TASK } },
        );
        assert_action(
            &mut view,
            Role::Button,
            b"Reply",
            d::pages::Act::Needs(d::pages::needs::Act::OpenReply { object: QUESTIONS[6] }),
        );
    }
}
#[test]
fn phone_answer_name_holds_the_joined_title_and_words() {
    // The fixture's ordinary Name bound is 2 * 64 + 1; two Names,
    // the colon and space, and " Answer" are 267 bytes (README.md, 3.5).
    const ANSWER_NAME: usize = 267;
    let w = f::browser_words();
    let mut title = skein_lib::bytes::zeroed(usize::try_from(w.name_bytes).unwrap());
    title.fill(b'T');
    let mut question = skein_lib::bytes::zeroed(usize::try_from(w.name_bytes).unwrap());
    question.fill(b'Q');
    let line = w.say(
        crate::words::Word::HomeAskLine,
        &[skein_ui::phrases::Hole::Name(title), skein_ui::phrases::Hole::Name(question)],
    );
    let name = w.say(
        crate::words::Word::HomeAnswerName,
        &[skein_ui::phrases::Hole::Name(line), skein_ui::phrases::Hole::Name(b"Answer".as_slice().into())],
    );
    assert_eq!(name.len(), ANSWER_NAME);
    assert!(name.ends_with(b" Answer"));
}

#[test]
fn home_notes_name_where_the_remaining_steps_stay() {
    for (configured, repository, chat, note) in [
        (false, false, false, "Three steps left. Skip them if you like; they stay in Settings and Chats."),
        (false, true, false, "Two steps left. Skip them if you like; they stay in Settings and Chats."),
        (false, false, true, "Two steps left. Skip them if you like; they stay in Settings."),
        (false, true, true, "One step left. Skip it if you like; it stays in Settings."),
        (true, false, true, "One step left. Skip it if you like; it stays in Settings."),
        (true, true, false, "One step left. Skip it if you like; it stays in Chats."),
    ] {
        let (root, _) = home_root(forge_rows(repository), configured, chat);
        for view in [f::page_view(&root), f::browser_page_view(&root)] {
            let node = styled(view.tree(), crate::look::HOME_NOTE);
            assert_eq!(node.role, Role::Paragraph);
            assert_eq!(node.name.as_deref(), Some(note.as_bytes()));
        }
    }
}

#[test]
fn home_waits_for_settings_and_dismissals_before_the_card() {
    for (dismissed, settings) in [(None, true), (Some(&[][..]), false)] {
        let (root, _) = home_documents_with_reads(forge_rows(true), false, false, None, None, dismissed, settings);
        for view in [f::page_view(&root), f::browser_page_view(&root)] {
            let text = f::main_text(view.tree());
            absent(&text, "To begin");
            absent(&text, "Shop is ready.");
        }
    }
}

#[test]
fn home_skipped_project_draws_the_everyday_intro() {
    let (root, _) = home_documents_with_reads(
        forge_rows(true),
        false,
        false,
        Some(NeedsFixture { objects: &[], cut: false, read: true }),
        None,
        Some(&[f::PROJECT]),
        true,
    );
    for view in [f::page_view(&root), f::browser_page_view(&root)] {
        let text = f::main_text(view.tree());
        absent(&text, "To begin");
        absent(&text, "Shop is ready.");
        main_find(view.tree(), Role::Heading { level: 1 }, b"Nothing needs you.");
        absent(&text, "Connected");
    }
}

#[test]
#[expect(clippy::disallowed_methods, reason = "test assertions inspect semantic nodes and request attempts")]
fn home_skip_hides_the_card_and_refusal_offers_retry() {
    let (mut root, _) = home_root(forge_rows(true), false, false);
    for mut view in [f::page_view(&root), f::browser_page_view(&root)] {
        assert_action(&mut view, Role::Button, b"Skip", d::pages::Act::SkipToBegin);
    }
    let mut out = drive(&mut root, d::Event::Act { action: d::Action::Pages(d::pages::Act::SkipToBegin) });
    for view in [f::page_view(&root), f::browser_page_view(&root)] {
        absent(&f::main_text(view.tree()), "To begin");
    }
    let mut stored = drive(&mut root, d::Event::Client { event: jig_client::Event::Stored });
    while let Some(request) = stored.pop() {
        out.push(request);
    }
    let attempt = out
        .iter()
        .find_map(|request| match request {
            d::Request::Unlinked {
                request: jig_client::UnlinkedRequest::Down { down: jig_client::Down::Send { attempt, .. } },
            } => Some(*attempt),
            d::Request::Client { .. }
            | d::Request::Unlinked { .. }
            | d::Request::Show { .. }
            | d::Request::Leave { .. }
            | d::Request::SignOut
            | d::Request::Dropped { .. }
            | d::Request::Tab { .. }
            | d::Request::Fact { .. } => None,
        })
        .expect("Skip request");
    drive(
        &mut root,
        d::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Answer {
                    attempt,
                    envelope: skein_lib::RequestEnvelope::Final,
                    answer: Some(jig_client::Answered::Jig(c::Answer::Refused {
                        refusal: c::Refusal::NotOffered,
                        at: None,
                    })),
                    why: None,
                },
            },
        },
    );
    for mut view in [f::page_view(&root), f::browser_page_view(&root)] {
        let begin = styled(view.tree(), crate::look::HOME_BEGIN);
        let parts = children(subtree(view.tree(), begin.id));
        assert_eq!(parts[1].styles.as_slice(), &[crate::look::HOME_NOTE]);
        assert_eq!(parts[2].role, Role::Alert);
        assert_eq!(parts[2].name.as_deref(), Some(b"temper couldn't keep that. Try Skip again.".as_slice()));
        assert_action(&mut view, Role::Button, b"Skip", d::pages::Act::SkipToBegin);
    }
}

fn home_accounts_document(holdings: &[(&[u8], c::AccountKind, bool)]) -> c::Accounts {
    let mut doc = super::accounts::document(0);
    doc.accounts = List::with_capacity(u32::try_from(holdings.len()).expect("fixture count"));
    for (index, (provider, kind, usable)) in holdings.iter().enumerate() {
        let mut row =
            super::accounts::card_document(provider, b"", *kind).accounts.get(0).expect("fixture row").clone();
        row.number = u32::try_from(index).expect("fixture index").checked_add(1).expect("fixture number");
        row.sources.get_mut(0).expect("fixture source").usable = *usable;
        doc.accounts.push(row).expect("fixture rows fit");
    }
    doc
}

#[test]
fn home_more_accounts_names_usable_holdings() {
    use c::AccountKind::{Key, Program, SignIn};
    type Holding = (&'static [u8], c::AccountKind, bool);
    let cases: &[(&[Holding], &str)] = &[
        (&[], "Add another account"),
        (&[(b"openai", SignIn, true)], "Add Claude or another plan"),
        (&[(b"anthropic", Program, true)], "Add ChatGPT or another plan"),
        (&[(b"openai", SignIn, true), (b"anthropic", Program, true)], "Add another account"),
        (&[(b"openai", Key, true)], "Add another account"),
        (&[(b"anthropic", Key, true)], "Add another account"),
        (&[(b"openai", SignIn, false), (b"anthropic", Program, false)], "Add another account"),
        (&[(b"openai", SignIn, true), (b"anthropic", Program, false)], "Add Claude or another plan"),
        (&[(b"openai", SignIn, false), (b"anthropic", Program, true)], "Add ChatGPT or another plan"),
        (&[(b"openai", SignIn, true), (b"openai", SignIn, true)], "Add Claude or another plan"),
        (&[(b"anthropic", Program, true), (b"anthropic", Program, true)], "Add ChatGPT or another plan"),
        (&[(b"custom", SignIn, true)], "Add another account"),
        (&[(b"openai", SignIn, true), (b"custom", Program, true)], "Add another account"),
        (&[(b"anthropic", Program, true), (b"openai", Key, true)], "Add another account"),
        (&[(b"openai", SignIn, true), (b"openai", Key, true)], "Add another account"),
    ];
    for (holdings, label) in cases {
        let (mut root, watch) = home_root(forge_rows(true), false, false);
        super::accounts::deliver(&mut root, watch, home_accounts_document(holdings));
        for mut view in [f::page_view(&root), f::browser_page_view(&root)] {
            let wrapper = styled(view.tree(), crate::look::HOME_LEDES);
            let parts = children(subtree(view.tree(), wrapper.id));
            let link = parts.last().expect("Accounts link");
            assert_eq!(link.role, Role::Link);
            assert_eq!(link.name.as_deref(), Some(label.as_bytes()), "{holdings:?}");
            let id = link.id;
            assert_eq!(crate::href(&mut view, id).as_deref(), Some(b"/accounts".as_slice()));
            assert_eq!(
                crate::act(&mut view, id, Act::Press),
                Some(d::Action::Pages(d::pages::Act::Go { address: Address::Accounts }))
            );
        }
    }
}

#[test]
fn home_more_accounts_reads_every_source_and_ignores_playground() {
    let (mut root, watch) = home_root(forge_rows(true), false, false);
    let mut doc = home_accounts_document(&[(b"openai", c::AccountKind::SignIn, true)]);
    let row = doc.accounts.get_mut(0).expect("account");
    let mut key = row.sources.get(0).expect("source").clone();
    row.sources = List::with_capacity(2);
    row.sources.push(key.clone()).expect("plan source");
    key.kind = c::AccountKind::Key;
    key.number = 2;
    row.sources.push(key).expect("key source");
    super::accounts::deliver(&mut root, watch, doc);
    main_find(f::page_view(&root).tree(), Role::Link, b"Add another account");

    let (mut root, _, _, _, watch) = f::root_at_with_playground(
        Address::Home { project: f::PROJECT },
        &f::domain_limits(),
        b"Shop",
        Some((f::PROJECT, true)),
    );
    let playground = root.forge().playground_provider().expect("playground provider");
    let mut doc = home_accounts_document(&[
        (b"openai", c::AccountKind::SignIn, true),
        (b"anthropic", c::AccountKind::Program, true),
    ]);
    doc.accounts.get_mut(0).expect("playground row").provider = Some(playground);
    doc.accounts.get_mut(1).expect("real row").provider = Some(playground.checked_add(1).expect("provider"));
    super::accounts::deliver(&mut root, watch.expect("accounts watch"), doc);
    assert_eq!(crate::pages::accounts::home_accounts(&root), (false, crate::words::Word::HomeAddChatGpt));
}

#[test]
fn home_lede_joins_only_the_facts_it_holds() {
    use c::AccountKind::{Key, Program, SignIn};
    for (holdings, start) in [
        (&[][..], ""),
        (&[(b"openai".as_slice(), SignIn, true)][..], "Your agents run on your ChatGPT plan"),
        (&[(b"anthropic".as_slice(), Program, true)][..], "Your agents run on your Claude subscription"),
        (&[(b"openai".as_slice(), Key, true)][..], "Your agents run on an API key"),
        (&[(b"anthropic".as_slice(), Key, true)][..], "Your agents run on an API key"),
        (&[(b"openai".as_slice(), Key, false)][..], ""),
    ] {
        for forge in ["here", "remote", "none", "disconnected", "fake"] {
            let mut rows = forge_rows(true);
            let mut parts = rows.get(0).expect("forge row").clone().into_parts();
            match forge {
                "remote" => parts.mode = w::Mode::Managed,
                "disconnected" => parts.connection = w::Connection::NotConnected,
                "fake" => parts.mode = w::Mode::Fake,
                _ => {}
            }
            *rows.get_mut(0).expect("forge row") = w::ForgeRow::new(&w::CEILINGS, parts).expect("forge");
            if forge == "none" {
                rows = List::with_capacity(0);
            }
            let end = match (forge, start.is_empty()) {
                ("here", false) => ", and Forgejo on this machine",
                ("here", true) => "Forgejo runs on this machine",
                ("remote", false) => ", and Forgejo holds your code",
                ("remote", true) => "Forgejo holds your code",
                _ => "",
            };
            let expected: Box<[u8]> = if start.is_empty() && end.is_empty() {
                Box::default()
            } else {
                crate::accounts::words::join(&[start.as_bytes(), end.as_bytes(), b"."])
            };
            let (mut root, watch) = home_root(rows, false, false);
            super::accounts::deliver(&mut root, watch, home_accounts_document(holdings));
            for view in [f::page_view(&root), f::browser_page_view(&root)] {
                let wrapper = styled(view.tree(), crate::look::HOME_LEDES);
                let parts = children(subtree(view.tree(), wrapper.id));
                assert_eq!(parts.len(), if expected.is_empty() { 1 } else { 2 });
                if !expected.is_empty() {
                    assert_eq!(parts[0].role, Role::Paragraph);
                    assert_eq!(parts[0].styles.as_slice(), &[crate::look::HOME_LEDE]);
                    assert_eq!(parts[0].name.as_deref(), Some(expected.as_ref()), "{forge} {holdings:?}");
                }
            }
        }
    }
}

#[test]
fn home_accounts_follow_the_lede_in_the_tree_and_terminal() {
    let (mut root, watch) = home_root(forge_rows(true), false, false);
    super::accounts::deliver(&mut root, watch, home_accounts_document(&[(b"openai", c::AccountKind::SignIn, true)]));
    for mut view in [f::page_view(&root), f::browser_page_view(&root)] {
        assert_eq!(crate::look::NAMES.style(crate::look::HOME_LEDES), Some(b"temper-home-ledes".as_slice()));
        let wrapper = styled(view.tree(), crate::look::HOME_LEDES);
        assert_eq!(wrapper.role, Role::Slot);
        let parts = children(subtree(view.tree(), wrapper.id));
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[0].role, Role::Paragraph);
        assert_eq!(parts[1].role, Role::Link);
        let id = parts[1].id;
        let (node, act) =
            f::terminal_command(view.tree(), &view.limits().tree, id, b"Add Claude or another plan", b"Shop is ready.");
        assert_eq!(
            crate::act(&mut view, node, act),
            Some(d::Action::Pages(d::pages::Act::Go { address: Address::Accounts }))
        );
    }
    let print = printed(&f::page_view(&root));
    let lede = skein_lib::bytes::find(&print, b"Your agents run on your ChatGPT plan, and Forgejo on this machine.")
        .expect("lede");
    let link = skein_lib::bytes::find(&print, b"Add Claude or another plan").expect("link");
    let begin = skein_lib::bytes::find(&print, b"To begin:").expect("card");
    assert!(lede < link && link < begin);
    drive(&mut root, d::Event::Act { action: d::Action::Pages(d::pages::Act::SkipToBegin) });
    for view in [f::page_view(&root), f::browser_page_view(&root)] {
        for node in main_nodes(view.tree()) {
            assert!(!node.styles.as_slice().contains(&crate::look::HOME_LEDES));
        }
        absent(&f::main_text(view.tree()), "Add Claude or another plan");
    }
}

#[test]
fn settings_configure_link_opens_the_delivered_configuring_chat_in_both_shells() {
    let (mut root, me_watch, _) = f::root_at(Address::Settings { project: f::PROJECT }, &f::domain_limits());
    let template = f::chat::messaging_root();
    let me = f::configuring_me(template.pages().me(template.client()).expect("me").clone());
    f::deliver_me(&mut root, me_watch, me);
    // Leave and return so the test observes the watch that Settings itself owns.
    drive(&mut root, d::Event::Went { place: d::Place::Address(Address::Accounts) });
    let out = drive(&mut root, d::Event::Went { place: d::Place::Address(Address::Settings { project: f::PROJECT }) });
    let mut live = None;
    for request in &out {
        if let d::Request::Unlinked {
            request:
                jig_client::UnlinkedRequest::Down {
                    down:
                        jig_client::Down::Open { watch, subject: jig_client::Watchable::Jig(c::Subject::Tasks { .. }) },
                },
        } = request
        {
            live = Some(*watch);
        }
    }
    let mut row = super::chats::line();
    row.chip.executor = c::ChipExecutor::Agent { charter: 37 };
    let task = row.chip.task;
    let mut rows = List::with_capacity(1);
    rows.push(row).expect("one configuring chat");
    deliver(
        &mut root,
        live.expect("Settings watches chats"),
        c::Delivery::Tasks(Box::new(c::Tasks { rows, more: false })),
    );
    for mut view in [f::page_view(&root), f::browser_page_view(&root)] {
        let link = main_find(view.tree(), Role::Link, b"Go on");
        assert_eq!(crate::href(&mut view, link).as_deref(), Some(b"/chats/9".as_slice()));
        assert_eq!(
            crate::act(&mut view, link, Act::Press),
            Some(d::Action::Pages(d::pages::Act::Go { address: Address::Chat { task } }))
        );
        let text = f::main_text(view.tree());
        assert!(skein_lib::bytes::find(&text, b"Hello").is_none(), "the watched row is not drawn on Settings");
    }
}

// Four epoch days leave room for the table's three-day previous sitting.
const STATUS_NOW: u64 = 345_600_000_000_000;
const STATUS_MINUTE: u64 = 60_000_000_000;
const CHAT_SENTENCE: &str = "A chat suits one change. For bigger work, a chat can write a draft beside it and start a goal from it: a plan, its tasks, landing.";
fn status_parts() -> w::ProjectStatusParts {
    w::ProjectStatusParts {
        project: f::PROJECT,
        landed: 3,
        last_landed: Some(STATUS_NOW - 6 * STATUS_MINUTE),
        brief: None,
        previous: Some(STATUS_NOW - 9 * 60 * STATUS_MINUTE),
        seen: 1,
        queued: 3,
        red: false,
        branch: b"main".as_slice().into(),
    }
}
fn status_home(parts: w::ProjectStatusParts, answered: bool, beginning: bool, goals: bool) -> d::Domain {
    let fixture = StatusFixture {
        document: w::ProjectStatus::new(&w::CEILINGS, parts).unwrap(),
        answered,
        goals,
        goal_rows: None,
        away: None,
    };
    home_documents_with_status(
        forge_rows(true),
        !beginning,
        !beginning,
        Some(NeedsFixture { objects: &[], cut: false, read: true }),
        None,
        Some(&[]),
        true,
        Some(&fixture),
    )
    .0
}
fn status_view(root: &d::Domain, browser: bool) -> crate::View {
    let mut view = f::view_of(if browser { f::browser_words() } else { f::words() });
    crate::build(&mut view, root, skein_lib::Wall::from_nanos(STATUS_NOW), &mut Queue::uncapped());
    view
}
fn kicker(root: &d::Domain, browser: bool) -> Box<[u8]> {
    let view = status_view(root, browser);
    styled(view.tree(), crate::look::HOME_KICKER).name.clone().unwrap()
}
#[test]
fn the_kicker_says_when_you_last_looked() {
    // The fractional spans prove the displayed unit is rounded down.
    for (seconds, expected) in [
        (61, "Monday, 00:00 · you last looked a minute ago"),
        (179, "Monday, 00:00 · you last looked 2 minutes ago"),
        (3601, "Monday, 00:00 · you last looked an hour ago"),
        (35_999, "Monday, 00:00 · you last looked 9 hours ago"),
        (86_401, "Monday, 00:00 · you last looked a day ago"),
        (259_201, "Monday, 00:00 · you last looked 3 days ago"),
    ] {
        let mut parts = status_parts();
        parts.previous = Some(STATUS_NOW - seconds * 1_000_000_000);
        let root = status_home(parts, true, false, false);
        for browser in [false, true] {
            assert_eq!(kicker(&root, browser).as_ref(), expected.as_bytes());
            contains(&printed(&status_view(&root, browser)), expected);
        }
    }
}
#[test]
fn no_previous_says_nothing() {
    let mut parts = status_parts();
    parts.previous = None;
    let root = status_home(parts, true, false, false);
    for browser in [false, true] {
        assert_eq!(kicker(&root, browser).as_ref(), "Monday, 00:00".as_bytes());
    }
}
#[test]
fn under_a_minute_says_nothing() {
    for seconds in [0, 59, 60] {
        let mut parts = status_parts();
        parts.previous = Some(STATUS_NOW - seconds * 1_000_000_000);
        let root = status_home(parts, true, false, false);
        for browser in [false, true] {
            absent(&kicker(&root, browser), "you last looked");
        }
    }
}
#[test]
fn the_headline_counts_what_landed() {
    for (landed, seen, headline) in [
        (0, 0, "Nothing needs you."),
        (3, 3, "The last change landed 6 minutes ago. Nothing needs you."),
        (3, 2, "One change landed since you last looked. Nothing needs you."),
        (5, 2, "3 changes landed since you last looked. Nothing needs you."),
    ] {
        let mut parts = status_parts();
        parts.landed = landed;
        parts.seen = seen;
        let root = status_home(parts, true, false, false);
        for browser in [false, true] {
            let view = status_view(&root, browser);
            main_find(view.tree(), Role::Heading { level: 1 }, headline.as_bytes());
            contains(&printed(&view), headline);
        }
    }
}
#[test]
fn the_headline_is_as_now_until_status_is_shown() {
    for root in
        [Box::new(everyday(false, &[], false, true)), Box::new(status_home(status_parts(), false, false, false))]
    {
        for browser in [false, true] {
            let view = status_view(&root, browser);
            main_find(view.tree(), Role::Heading { level: 1 }, b"Nothing needs you.");
            let text = f::main_text(view.tree());
            absent(&text, "you last looked");
            absent(&text, "Landing:");
        }
    }
}
#[test]
fn the_foot_says_what_is_queued() {
    for (queued, last, red, branch, expected) in [
        (
            3,
            Some(STATUS_NOW - 6 * STATUS_MINUTE),
            false,
            b"main".as_slice(),
            "Landing: 3 queued · last landed 6 min ago · Changes",
        ),
        (
            0,
            Some(STATUS_NOW - 60 * STATUS_MINUTE),
            false,
            b"main".as_slice(),
            "Landing: main last moved 1 h ago · nothing queued · Changes",
        ),
        (0, None, false, b"main".as_slice(), "Landing: nothing has landed yet · Changes"),
        (0, None, true, b"main".as_slice(), "Landing: nothing has landed yet · main red · Changes"),
        (0, None, true, b"".as_slice(), "Landing: nothing has landed yet · landing red · Changes"),
    ] {
        let mut parts = status_parts();
        parts.queued = queued;
        parts.last_landed = last;
        parts.red = red;
        parts.branch = branch.into();
        let root = status_home(parts, true, false, false);
        for browser in [false, true] {
            let view = status_view(&root, browser);
            let foot = children(main_nodes(view.tree())).last().copied().unwrap();
            assert_eq!(foot.role, Role::Paragraph);
            assert_eq!(foot.styles.as_slice(), &[crate::look::HOME_FACTS]);
            let nodes = subtree(view.tree(), foot.id);
            assert_eq!(f::nodes_text(nodes).as_ref(), expected.as_bytes());
            if red {
                let signal = styled(view.tree(), crate::look::CHANGES_RED);
                assert!(f::nodes_text(subtree(view.tree(), signal.id)).starts_with(" · ".as_bytes()));
            }
            contains(&printed(&view), "Landing:");
        }
    }
}
#[test]
fn the_foot_links_to_changes() {
    let root = status_home(status_parts(), true, false, false);
    for browser in [false, true] {
        let mut view = status_view(&root, browser);
        let foot = children(main_nodes(view.tree())).last().copied().unwrap();
        let mut link = None;
        for node in subtree(view.tree(), foot.id) {
            if node.role == Role::Link {
                link = Some(node.id);
            }
        }
        let link = link.unwrap();
        assert_eq!(crate::href(&mut view, link).as_deref(), Some(b"/projects/1/changes".as_slice()));
        assert_eq!(
            crate::act(&mut view, link, Act::Press),
            Some(d::Action::Pages(d::pages::Act::Go { address: Address::Changes { project: f::PROJECT } }))
        );
    }
}
#[test]
fn no_foot_while_to_begin_shows() {
    let root = status_home(status_parts(), true, true, false);
    for browser in [false, true] {
        let view = status_view(&root, browser);
        main_find(view.tree(), Role::Heading { level: 1 }, b"Shop is ready.");
        absent(&f::main_text(view.tree()), "Landing:");
    }
}
#[test]
fn chats_says_its_sentence_with_no_goal() {
    for mut root in
        [Box::new(home_chats(chat_fixture(1, 1, false, false))), Box::new(home_chats(chat_fixture(0, 0, false, false)))]
    {
        drive(&mut root, d::Event::Act { action: d::Action::Pages(d::pages::Act::SkipToBegin) });
        for view in [f::page_view(&root), f::browser_page_view(&root)] {
            main_find(view.tree(), Role::Paragraph, CHAT_SENTENCE.as_bytes());
            let region = styled(view.tree(), crate::look::HOME_CHATS);
            let last = children(subtree(view.tree(), region.id)).last().copied().unwrap();
            assert_eq!(last.name.as_deref(), Some(CHAT_SENTENCE.as_bytes()));
            assert_eq!(last.styles.as_slice(), &[crate::look::HOME_FACTS]);
        }
    }
}
#[test]
fn chats_has_no_sentence_with_a_goal() {
    let root = status_home(status_parts(), true, false, true);
    for browser in [false, true] {
        let view = status_view(&root, browser);
        absent(&f::main_text(view.tree()), CHAT_SENTENCE);
    }
}
const HOME_QUESTION: &[u8] = b"Which license?\n\nThe README needs a license.\n\nSaw: No license file.\nChoice: MIT (recommended)\nChoice: Apache";
const PARSED: c::ObjectKey = c::ObjectKey::Question { task: f::INBOX_TASK, message: 8 };
fn needs_act(root: &mut d::Domain, act: d::pages::needs::Act) {
    drive(root, d::Event::Act { action: d::Action::Pages(d::pages::Act::Needs(act)) });
}
#[test]
#[expect(clippy::disallowed_methods, reason = "test assertions inspect semantic subtrees and public action deliveries")]
fn parsed_home_card_replies_in_place_and_prints_choices() {
    use d::pages::needs::Act as N;
    let mut root = everyday(false, &[PARSED, QUESTIONS[0]], false, true);
    for mut view in [f::page_view(&root), f::browser_page_view(&root)] {
        main_find(view.tree(), Role::Paragraph, b"Which license?");
        main_find(view.tree(), Role::Paragraph, b"The README needs a license.");
        let saw = styled(view.tree(), crate::look::ASK_SAW);
        assert_eq!(saw.role, Role::Disclosure);
        assert_eq!(saw.name.as_deref(), Some(b"What the agent saw".as_slice()));
        assert!(!saw.states.as_slice().contains(&skein_ui::State::Expanded));
        assert_eq!(styled(view.tree(), crate::look::RECOMMENDED).name.as_deref(), Some(b"MIT".as_slice()));
        main_find(view.tree(), Role::Text, b"recommended");
        assert_action(
            &mut view,
            Role::Button,
            b"MIT",
            d::pages::Act::Needs(N::Choose { object: PARSED, words: b"MIT".as_slice().into() }),
        );
        assert_action(
            &mut view,
            Role::Button,
            b"Apache",
            d::pages::Act::Needs(N::Choose { object: PARSED, words: b"Apache".as_slice().into() }),
        );
        assert_action(&mut view, Role::Button, b"Reply", d::pages::Act::Needs(N::OpenReply { object: PARSED }));
        assert_action(&mut view, Role::Button, b"Later", d::pages::Act::Needs(N::Later { object: PARSED }));
    }
    let print = printed(&f::page_view(&root));
    for words in ["Which license?", "MIT", "Apache", "Reply", "Later", "Which license? · a question · under 1 min"] {
        contains(&print, words);
    }
    needs_act(&mut root, N::OpenReply { object: PARSED });
    drive(
        &mut root,
        d::Event::Act {
            action: d::Action::Pages(d::pages::Act::CardWords { object: PARSED, words: b"Use MIT".as_slice().into() }),
        },
    );
    for mut view in [f::page_view(&root), f::browser_page_view(&root)] {
        let id = main_find(view.tree(), Role::TextArea, b"Your reply");
        let field = &subtree(view.tree(), id)[0];
        assert_eq!(field.value.as_ref().unwrap().text.as_ref(), b"Use MIT");
        assert!(field.focus.is_some());
        assert_action(&mut view, Role::Button, b"Send reply", d::pages::Act::Needs(N::SendReply { object: PARSED }));
        assert_action(&mut view, Role::Button, b"Cancel", d::pages::Act::Needs(N::CancelReply));
    }
    needs_act(&mut root, N::CancelReply);
    assert!(
        !main_nodes(f::page_view(&root).tree()).iter().any(|n| n.styles.as_slice().contains(&crate::look::ASK_REPLY))
    );
}
#[test]
#[expect(clippy::disallowed_methods, reason = "test assertions inspect semantic subtrees and public action deliveries")]
fn home_answered_lines_precede_card_count_falls_and_undo_restores_it() {
    use d::pages::needs::Act as N;
    let mut root = everyday(false, &[PARSED, QUESTIONS[0]], false, true);
    needs_act(&mut root, N::Choose { object: PARSED, words: b"MIT".as_slice().into() });
    for mut view in [f::page_view(&root), f::browser_page_view(&root)] {
        main_find(view.tree(), Role::Heading { level: 1 }, b"One thing needs you.");
        main_find(view.tree(), Role::Heading { level: 2 }, "Needs you · 1".as_bytes());
        let parts = children(subtree(view.tree(), styled(view.tree(), crate::look::HOME_NEEDS).id));
        assert_eq!(parts[0].styles.as_slice(), &[crate::look::HOME_HEAD]);
        assert_eq!(parts[1].styles.as_slice(), &[crate::look::ANSWERED]);
        assert_eq!(parts[2].styles.as_slice(), &[crate::look::ASK]);
        assert_action(&mut view, Role::Button, b"Undo", d::pages::Act::Needs(N::Undo { object: PARSED }));
    }
    contains(&printed(&f::page_view(&root)), "Add a README: MIT");
    needs_act(&mut root, N::Undo { object: PARSED });
    for view in [f::page_view(&root), f::browser_page_view(&root)] {
        main_find(view.tree(), Role::Heading { level: 1 }, b"Two things need you.");
        assert!(!main_nodes(view.tree()).iter().any(|n| n.styles.as_slice().contains(&crate::look::ANSWERED)));
    }
}
#[test]
#[expect(clippy::disallowed_methods, reason = "test assertions inspect semantic subtrees and public action deliveries")]
fn home_answered_window_expires_and_sets_the_next_change() {
    use d::pages::needs::{ANSWERED_ON_HOME, Act as N};
    let mut root = everyday(false, &[PARSED], false, true);
    needs_act(&mut root, N::Choose { object: PARSED, words: b"MIT".as_slice().into() });
    let end = skein_lib::Wall::from_nanos(f::NOW.as_nanos().checked_add(ANSWERED_ON_HOME.as_nanos()).unwrap());
    for browser in [false, true] {
        let mut view = f::view_of(if browser { f::browser_words() } else { f::words() });
        crate::build(&mut view, &root, skein_lib::Wall::from_nanos(end.as_nanos() - 1), &mut Queue::uncapped());
        styled(view.tree(), crate::look::ANSWERED);
        assert_eq!(view.next_change(), Some(end));
        crate::build(&mut view, &root, end, &mut Queue::uncapped());
        assert!(!main_nodes(view.tree()).iter().any(|n| n.styles.as_slice().contains(&crate::look::ANSWERED)));
    }
}
#[test]
fn later_home_card_moves_to_a_line_then_returns_with_its_meta() {
    use d::pages::needs::Act as N;
    let mut root = everyday(false, &[PARSED, QUESTIONS[0]], false, true);
    needs_act(&mut root, N::Later { object: PARSED });
    for mut view in [f::page_view(&root), f::browser_page_view(&root)] {
        let rows = styled(view.tree(), crate::look::HOME_ROWS);
        contains(&f::nodes_text(subtree(view.tree(), rows.id)), "Which license?");
        assert_action(
            &mut view,
            Role::Link,
            "Which license? · a question · under 1 min".as_bytes(),
            d::pages::Act::Needs(N::Open { object: PARSED }),
        );
    }
    needs_act(&mut root, N::Later { object: QUESTIONS[0] });
    for view in [f::page_view(&root), f::browser_page_view(&root)] {
        assert!(styled(view.tree(), crate::look::ASK_META).name.as_ref().unwrap().ends_with(" · later".as_bytes()));
    }
}
#[test]
fn beginning_keeps_only_the_card_and_no_phone_block_or_rows() {
    let root = everyday(true, &[PARSED, QUESTIONS[0], QUESTIONS[1]], false, true);
    for view in [f::page_view(&root), f::browser_page_view(&root)] {
        let parts = children(subtree(view.tree(), styled(view.tree(), crate::look::HOME_NEEDS).id));
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[0].styles.as_slice(), &[crate::look::HOME_HEAD]);
        assert_eq!(parts[1].styles.as_slice(), &[crate::look::ASK]);
        absent(&f::main_text(view.tree()), "Answer in a row");
    }
}
#[test]
fn phone_block_uses_short_and_the_open_count() {
    for objects in [&[PARSED][..], &[PARSED, QUESTIONS[0]][..]] {
        let root = everyday(false, objects, false, true);
        let mut view = f::browser_page_view(&root);
        let go = if objects.len() == 1 { "Answer" } else { "Answer, one at a time" };
        let name = crate::accounts::words::join(&[b"Add a README: Which license? ", go.as_bytes()]);
        assert_action(&mut view, Role::Link, &name, inbox_act());
        assert_eq!(
            f::nodes_text(subtree(view.tree(), styled(view.tree(), crate::look::HOME_CTA_GO).id)).as_ref(),
            go.as_bytes()
        );
        absent(&f::main_text(f::page_view(&root).tree()), "Answer, one at a time");
    }
}

#[test]
#[expect(clippy::disallowed_methods, reason = "test assertions inspect semantic subtrees and public action deliveries")]
fn home_refusal_reopens_the_card_with_the_engines_words() {
    use d::pages::needs::{Act as N, HOLD};
    let mut root = everyday(false, &[PARSED], false, true);
    needs_act(&mut root, N::Choose { object: PARSED, words: b"MIT".as_slice().into() });
    let env = skein_lib::Env { now: skein_lib::Time::ZERO.saturating_add(HOLD), wall: f::NOW, limits: home_limits() };
    let mut out = Queue::uncapped();
    d::fire(&mut root, &env, &mut out);
    d::step(&mut root, &env, d::Event::Client { event: jig_client::Event::Stored }, &mut out);
    let attempt = out
        .iter()
        .find_map(|request| {
            if let d::Request::Unlinked {
                request: jig_client::UnlinkedRequest::Down { down: jig_client::Down::Send { attempt, .. } },
            } = request
            {
                Some(*attempt)
            } else {
                None
            }
        })
        .expect("answer sent");
    d::step(
        &mut root,
        &env,
        d::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Answer {
                    attempt,
                    envelope: skein_lib::RequestEnvelope::Final,
                    why: None,
                    answer: Some(jig_client::Answered::Jig(c::Answer::Refused {
                        refusal: c::Refusal::NotOffered,
                        at: None,
                    })),
                },
            },
        },
        &mut out,
    );
    for view in [f::page_view(&root), f::browser_page_view(&root)] {
        let card = styled(view.tree(), crate::look::ASK);
        contains(&f::nodes_text(subtree(view.tree(), card.id)), "not offered");
        main_find(view.tree(), Role::Heading { level: 1 }, b"One thing needs you.");
    }
}
#[test]
#[expect(clippy::disallowed_methods, reason = "test assertions inspect semantic subtrees and public action deliveries")]
fn home_funds_and_proposal_keep_decisions_confirmation_and_short_lines() {
    let proposal = c::ObjectKey::Proposal(c::ProposalKey {
        proposer: c::Proposer::Task(f::INBOX_TASK),
        number: f::INBOX_QUESTION,
    });
    let funds = c::ObjectKey::Escalation(10);
    for (object, name, kind) in
        [(proposal, b"Accept".as_slice(), c::ActKind::Accept), (funds, b"Release", c::ActKind::Release)]
    {
        let mut root = everyday(false, &[object], false, true);
        for mut view in [f::page_view(&root), f::browser_page_view(&root)] {
            let button = main_find(view.tree(), Role::Button, name);
            let Some(d::Action::Pages(action @ d::pages::Act::Decide { act, .. })) =
                crate::act(&mut view, button, Act::Press)
            else {
                panic!("decision")
            };
            assert_eq!(act, kind);
            if object == funds {
                main_find(view.tree(), Role::Paragraph, b"31 of 40 used.");
                main_find(view.tree(), Role::Disclosure, b"Leave held");
            }
            drive(&mut root, d::Event::Act { action: d::Action::Pages(action) });
            let mut confirmed = f::browser_page_view(&root);
            assert!(main_nodes(confirmed.tree()).iter().any(|node| node.role == Role::Dialog));
            assert_action(&mut confirmed, Role::Button, name, d::pages::Act::Confirm);
            drive(&mut root, d::Event::Act { action: d::Action::Pages(d::pages::Act::Cancel) });
        }
    }
    let root = everyday(false, &[PARSED, funds], false, true);
    for mut view in [f::page_view(&root), f::browser_page_view(&root)] {
        assert_action(
            &mut view,
            Role::Link,
            "Add a README: 31 of 40 used · funds".as_bytes(),
            d::pages::Act::Needs(d::pages::needs::Act::Open { object: funds }),
        );
    }
}

fn goal_notes(scope: c::NoteScope) -> c::ReadValue {
    let mut rows = List::uncapped();
    if scope == (c::NoteScope::Task { project: f::PROJECT, task: 9 }) {
        rows.push(c::Note {
            number: 1,
            revision: c::Revision { value: 1 },
            scope,
            description: f::task::document_words(b"report: A goal report."),
            body: f::task::document_words(b""),
            writer: c::NoteAuthor::Task { task: 9, attempt: 1 },
            tasks: List::with_capacity(0),
            changed: Some(f::NOW),
            acts: List::with_capacity(0),
            cut: false,
        })
        .unwrap();
    }
    c::ReadValue::Notes(Box::new(c::NotePage { rows, before: None, after: None, cut: false }))
}

fn home_goals(live: usize, ended: usize, beginning: bool) -> d::Domain {
    let mut goals = chat_fixture(live, ended, false, false);
    for row in goals.live.rows.iter_mut() {
        row.tracked = Some(1);
    }
    if let c::EndedGroup::Rows { rows, .. } = &mut goals.ended.ended {
        for row in rows.iter_mut() {
            row.tracked = Some(1);
            row.chip.details.phase = c::ChipPhase::Ended;
            row.chip.details.details.ending =
                Some(c::ChipEnding::Done(c::ResultValue::Report(f::task::document_words(b"Landed."))));
        }
    }
    let fixture = StatusFixture {
        document: w::ProjectStatus::new(&w::CEILINGS, status_parts()).unwrap(),
        answered: true,
        goals: true,
        goal_rows: Some(goals),
        away: None,
    };
    home_documents_with_status(
        forge_rows(true),
        !beginning,
        !beginning,
        Some(NeedsFixture { objects: &[c::ObjectKey::Question { task: 9, message: 1 }], cut: false, read: true }),
        Some(chat_fixture(1, 0, false, false)),
        Some(&[]),
        true,
        Some(&fixture),
    )
    .0
}

#[test]
fn goals_takes_chats_place() {
    let root = home_goals(2, 3, false);
    assert!(root.pages().goals().unwrap().report(9).is_some(), "Home reads the goal report");
    assert!(root.pages().goals().unwrap().needs_person(9), "Home reads the goal needs");
    for mut view in [f::page_view(&root), f::browser_page_view(&root)] {
        let region = main_find(view.tree(), Role::Region, b"Goals");
        main_find(view.tree(), Role::Heading { level: 2 }, b"Goals");
        for node in main_nodes(view.tree()) {
            assert!(node.role != Role::Region || node.name.as_deref() != Some(b"Chats"));
        }
        absent(&f::main_text(view.tree()), CHAT_SENTENCE);
        let text = f::nodes_text(subtree(view.tree(), region));
        contains(&text, "A goal report.");
        contains(&text, "needs you");
        contains(&text, "planning");
        contains(&text, "landed");
        main_find(view.tree(), Role::Button, b"Reply");
        let row = main_find(view.tree(), Role::Link, b"README \xc2\xb7 needs you");
        assert_eq!(
            crate::act(&mut view, row, Act::Press),
            Some(d::Action::Pages(d::pages::Act::Go { address: Address::Task { task: 9 } }))
        );
        let goals = f::find(view.tree(), Role::Link, b"Goals").unwrap();
        assert!(!subtree(view.tree(), goals)[0].states.as_slice().contains(&skein_ui::State::Current));
        contains(&printed(&view), "A goal report.");
    }
}

#[test]
fn goals_rows_are_live_first_and_at_least_five() {
    for (live, ended, shown) in [(2, 5, 5), (6, 1, 6), (0, 3, 3), (1, 1, 2)] {
        let root = home_goals(live, ended, false);
        for view in [f::page_view(&root), f::browser_page_view(&root)] {
            let region = main_find(view.tree(), Role::Region, b"Goals");
            let mut rows = List::uncapped();
            for node in subtree(view.tree(), region) {
                if node.styles.as_slice().contains(&crate::look::GOALS_ROW) {
                    rows.push(node).unwrap();
                }
            }
            assert_eq!(usize::try_from(rows.len()).unwrap(), shown, "max(five, live), limited by available rows");
            for (row, title) in rows.iter().zip(CHAT_TITLES) {
                assert!(row.name.as_deref().unwrap().starts_with(title), "live then ended in page order");
            }
        }
    }
}

#[test]
fn all_n_links_to_goals() {
    let root = home_goals(2, 5, false);
    assert_eq!(root.pages().goals().unwrap().count(root.client()), 7);
    for mut view in [f::page_view(&root), f::browser_page_view(&root)] {
        let link = main_find(view.tree(), Role::Link, b"All 7");
        assert_eq!(crate::href(&mut view, link).as_deref(), Some(b"/projects/1/goals".as_slice()));
        assert_eq!(
            crate::act(&mut view, link, Act::Press),
            Some(d::Action::Pages(d::pages::Act::Go {
                address: Address::Goals { project: f::PROJECT, filter: d::GoalsFilter::All },
            }))
        );
    }
}

#[test]
fn to_begin_keeps_chats() {
    let root = home_goals(1, 1, true);
    assert!(root.has_goals(f::PROJECT));
    for view in [f::page_view(&root), f::browser_page_view(&root)] {
        main_find(view.tree(), Role::Heading { level: 2 }, b"To begin");
        main_find(view.tree(), Role::Region, b"Chats");
        for node in main_nodes(view.tree()) {
            assert!(node.role != Role::Region || node.name.as_deref() != Some(b"Goals"));
        }
    }
}

const BRIEF_SENTENCE: &str = "41 changes landed overnight.";
const BRIEF_LEDE: &str =
    "first-run-projects is six of nine tasks in. Its store task hit the wasm size limit twice and was split in two.";

fn brief_parts(landed: u64, lede: &str, at: u64) -> w::ProjectStatusParts {
    let mut parts = status_parts();
    parts.brief = Some(
        w::StatusBrief::new(
            &w::CEILINGS,
            w::StatusBriefParts {
                sentence: BRIEF_SENTENCE.as_bytes().into(),
                lede: lede.as_bytes().into(),
                at,
                landed,
            },
        )
        .unwrap(),
    );
    parts
}

fn no_brief_prose(view: &crate::View) {
    for style in [crate::look::HOME_LEDE, crate::look::HOME_BYLINE] {
        for node in main_nodes(view.tree()) {
            assert!(!node.styles.as_slice().contains(&style));
        }
    }
    let text = f::main_text(view.tree());
    absent(&text, BRIEF_SENTENCE);
    absent(&text, BRIEF_LEDE);
    absent(&text, "Written by the status agent");
}

#[test]
fn a_brief_above_seen_says_its_words() {
    let root = status_home(brief_parts(2, BRIEF_LEDE, STATUS_NOW), true, false, false);
    for browser in [false, true] {
        let view = status_view(&root, browser);
        main_find(view.tree(), Role::Heading { level: 1 }, b"41 changes landed overnight. Nothing needs you.");
        let lede = styled(view.tree(), crate::look::HOME_LEDE);
        let byline = styled(view.tree(), crate::look::HOME_BYLINE);
        assert_eq!(lede.name.as_deref(), Some(BRIEF_LEDE.as_bytes()));
        assert_eq!(byline.name.as_deref(), Some(b"Written by the status agent at 00:00".as_slice()));
        let intro = children(subtree(view.tree(), styled(view.tree(), crate::look::HOME_INTRO).id));
        if browser {
            assert_eq!(intro.len(), 4);
            assert_eq!(intro[2].id, lede.id);
            assert_eq!(intro[3].id, byline.id);
        } else {
            assert_eq!(intro[1].role, Role::Paragraph);
            let expected = crate::accounts::words::join(&[
                b"Monday, 00:00 \xc2\xb7 you last looked 9 hours ago 41 changes landed overnight. Nothing needs you. ",
                BRIEF_LEDE.as_bytes(),
            ]);
            assert_eq!(f::nodes_text(subtree(view.tree(), intro[1].id)).as_ref(), expected.as_ref());
        }
        let print = printed(&view);
        let kicker = skein_lib::bytes::find(&print, b"you last looked").unwrap();
        let headline = if browser {
            kicker
        } else {
            skein_lib::bytes::find_from(&print, BRIEF_SENTENCE.as_bytes(), kicker).unwrap()
        };
        let lede = skein_lib::bytes::find(&print, BRIEF_LEDE.as_bytes()).unwrap();
        let byline = skein_lib::bytes::find(&print, b"Written by the status agent").unwrap();
        assert!(kicker <= headline && headline < lede && lede < byline);
        absent(&print, "2 changes landed since");
    }
}

#[test]
fn an_old_brief_with_new_landings_uses_the_computed_words() {
    for landed in [0, 1] {
        let root =
            status_home(brief_parts(landed, BRIEF_LEDE, STATUS_NOW - 10 * 60 * STATUS_MINUTE), true, false, false);
        for browser in [false, true] {
            let view = status_view(&root, browser);
            main_find(
                view.tree(),
                Role::Heading { level: 1 },
                b"2 changes landed since you last looked. Nothing needs you.",
            );
            no_brief_prose(&view);
        }
    }
}

#[test]
fn no_brief_no_lede_no_byline() {
    for root in [
        Box::new(status_home(status_parts(), true, false, false)),
        Box::new(status_home(brief_parts(2, BRIEF_LEDE, STATUS_NOW), false, false, false)),
    ] {
        for browser in [false, true] {
            no_brief_prose(&status_view(&root, browser));
        }
    }
}

#[test]
fn an_empty_lede_keeps_the_byline() {
    let root = status_home(brief_parts(2, "", STATUS_NOW), true, false, false);
    for browser in [false, true] {
        let view = status_view(&root, browser);
        for node in main_nodes(view.tree()) {
            assert!(!node.styles.as_slice().contains(&crate::look::HOME_LEDE));
        }
        assert_eq!(
            styled(view.tree(), crate::look::HOME_BYLINE).name.as_deref(),
            Some(b"Written by the status agent at 00:00".as_slice())
        );
    }
}

#[test]
fn the_byline_says_its_clock() {
    for (now, at, expected) in [
        (
            STATUS_NOW + 8 * 60 * STATUS_MINUTE,
            STATUS_NOW + (7 * 60 + 42) * STATUS_MINUTE,
            "Written by the status agent at 07:42",
        ),
        (
            STATUS_NOW + 8 * 60 * STATUS_MINUTE,
            STATUS_NOW - (24 * 60 - 7 * 60 - 42) * STATUS_MINUTE,
            "Written by the status agent at Sun 07:42",
        ),
    ] {
        let root = status_home(brief_parts(2, BRIEF_LEDE, at), true, false, false);
        for words in [f::words(), f::browser_words()] {
            let mut view = f::view_of(words);
            crate::build(&mut view, &root, skein_lib::Wall::from_nanos(now), &mut Queue::uncapped());
            assert_eq!(styled(view.tree(), crate::look::HOME_BYLINE).name.as_deref(), Some(expected.as_bytes()));
        }
    }
}

const COPY_ADDRESSES: [&[u8]; 2] = [b"https://github.com/acme/widget", b"https://github.com/acme/other"];
fn copy_forges(states: &[Option<w::MakingState>], unasked: bool) -> List<w::ForgeRow> {
    let rows = forge_rows(false);
    let mut row = rows.get(0).unwrap().clone().into_parts();
    row.first = None;
    let mut addresses = List::with_capacity(2);
    for address in COPY_ADDRESSES.iter().take(states.len()) {
        addresses.push((*address).into()).unwrap();
    }
    row.starts_with =
        w::StartsWith::Copies(w::CopyAddresses::new(&w::CEILINGS, w::CopyAddressesParts { addresses }).unwrap());
    row.makings = List::with_capacity(2);
    row.repositories = List::with_capacity(2);
    for (address, state) in COPY_ADDRESSES.iter().zip(states) {
        let (org, name) = d::forge::making::target(address).unwrap();
        if let Some(state) = state {
            row.makings
                .push(
                    w::RepositoryMaking::new(
                        &w::CEILINGS,
                        w::RepositoryMakingParts {
                            org,
                            name,
                            state: state.clone(),
                            from: w::MakeFrom::Copy(
                                w::MakeCopy::new(&w::CEILINGS, w::MakeCopyParts { address: (*address).into() })
                                    .unwrap(),
                            ),
                            into: Some(
                                w::MakeInto::new(
                                    &w::CEILINGS,
                                    w::MakeIntoParts { project: f::PROJECT, home: *address == COPY_ADDRESSES[0] },
                                )
                                .unwrap(),
                            ),
                        },
                    )
                    .unwrap(),
                )
                .unwrap();
        } else if !unasked {
            row.repositories
                .push(
                    w::Adoption::new(&w::CEILINGS, w::AdoptionParts { owner: org, name, project: f::PROJECT }).unwrap(),
                )
                .unwrap();
        }
    }
    List::from_boxed(Box::new([w::ForgeRow::new(&w::CEILINGS, row).unwrap()]))
}
#[test]
fn home_copy_lines_keep_order_completion_and_retry_in_browser_and_terminal() {
    let failed = w::MakingState::Failed(
        w::MakingFailed::new(
            &w::CEILINGS,
            w::MakingFailedParts {
                step: w::MakeStep::Repository,
                reason: w::MakeFailure::Taken,
                words: b"name taken".as_slice().into(),
            },
        )
        .unwrap(),
    );
    let made =
        w::MakingMade::new(&w::CEILINGS, w::MakingMadeParts { id: 1, branch: b"main".as_slice().into() }).unwrap();
    for (states, expected, done, retry) in [
        (
            [Some(w::MakingState::Making), Some(w::MakingState::Waiting)],
            ["Copying acme/widget from github.com…", "Waiting to copy"],
            false,
            false,
        ),
        ([None, Some(failed)], ["acme/widget is in Shop", "acme/other failed at repository: name taken"], true, true),
        (
            [Some(w::MakingState::Made(made.clone())), Some(w::MakingState::Adopting(made))],
            ["Adopting acme/widget into Shop…", "Adopting acme/other into Shop…"],
            false,
            false,
        ),
    ] {
        let (root, _) = home_root(copy_forges(&states, false), false, false);
        for (mut view, browser) in [(f::page_view(&root), false), (f::browser_page_view(&root), true)] {
            let text = f::main_text(view.tree());
            let first = skein_lib::bytes::find(&text, expected[0].as_bytes()).expect("first copy's words");
            let second = skein_lib::bytes::find(&text, expected[1].as_bytes()).expect("second copy's words");
            assert!(first < second);
            let line = styled(view.tree(), crate::look::HOME_COPY);
            assert_eq!(line.styles.as_slice().contains(&crate::look::HOME_DONE), done);
            assert_eq!(crate::look::NAMES.style(crate::look::HOME_COPY), Some(b"temper-home-copy".as_slice()));
            if retry {
                assert_action(
                    &mut view,
                    Role::Button,
                    b"Copy again",
                    d::pages::Act::CopyAgain { org: b"acme".as_slice().into(), name: b"other".as_slice().into() },
                );
            } else {
                absent(&text, "Copy again");
            }
            if !browser {
                let text = printed(&view);
                contains(&text, expected[0]);
                contains(&text, expected[1]);
                if done {
                    contains(&text, "Done. acme/widget is in Shop");
                }
            }
        }
    }
}
#[test]
fn home_completed_copies_name_the_sources_and_explain_where_chats_write_in_both_shells() {
    for (count, title, detail) in [
        (
            1,
            "acme/widget is in Shop",
            "Chats can change its code now, on your Forgejo; nothing goes back where it was copied from.",
        ),
        (
            2,
            "acme/widget, acme/other are in Shop",
            "Chats can change their code now, on your Forgejo; nothing goes back where they were copied from.",
        ),
    ] {
        let (root, _) = home_root(copy_forges(&[None, None][..count], false), false, false);
        for (view, browser) in [(f::page_view(&root), false), (f::browser_page_view(&root), true)] {
            let text = f::main_text(view.tree());
            contains(&text, title);
            main_find(view.tree(), Role::Paragraph, detail.as_bytes());
            absent(&text, "temper can change its code now;");
            let step = styled(view.tree(), crate::look::HOME_DONE);
            assert_eq!(step.role, Role::Item);
            assert_eq!(step.styles.as_slice(), &[crate::look::HOME_STEP, crate::look::HOME_DONE]);
            if !browser {
                let heading = skein_lib::bytes::concat(&[b"2. Done. ", title.as_bytes()]);
                assert!(skein_lib::bytes::find(&printed(&view), &heading).is_some());
            }
        }
    }
}
#[test]
fn home_unasked_copies_offer_the_filled_addresses_and_one_copy_act_in_both_shells() {
    let (root, _) = home_root(copy_forges(&[None, None], true), false, false);
    for mut view in [f::page_view(&root), f::browser_page_view(&root)] {
        contains(&f::main_text(view.tree()), "Copy https://github.com/acme/widget");
        main_find(view.tree(), Role::Paragraph, b"https://github.com/acme/other");
        assert_action(&mut view, Role::Button, b"Copy", d::pages::Act::CopyOffered);
        let button = main_find(view.tree(), Role::Button, b"Copy");
        assert!(subtree(view.tree(), button)[0].styles.as_slice().contains(&crate::look::HOME_PRIMARY));
    }
    contains(&printed(&f::page_view(&root)), "Copy https://github.com/acme/widget");
    contains(&printed(&f::page_view(&root)), "https://github.com/acme/other");
}

fn add_rows() -> List<w::ForgeRow> {
    let mut row = forge_rows(false).get(0).unwrap().clone().into_parts();
    row.first = None;
    row.starts_with = w::StartsWith::Nothing;
    List::from_boxed(Box::new([w::ForgeRow::new(&w::CEILINGS, row).unwrap()]))
}
fn add_act(root: &mut d::Domain, act: d::pages::Act) -> Queue<d::Request> {
    drive(root, d::Event::Act { action: d::Action::Pages(act) })
}
#[test]
fn home_add_step_offers_small_make_copy_and_quiet_adoption_in_both_shells() {
    use d::forge::forges::Start;
    let (root, _) = home_root(add_rows(), false, false);
    for mut view in [f::page_view(&root), f::browser_page_view(&root)] {
        contains(&f::main_text(view.tree()), "Add a repository");
        main_find(
            view.tree(),
            Role::Paragraph,
            b"Your Forgejo has none yet. Until one is in, chats talk and plan but change no code.",
        );
        for (name, start) in [(b"Make an empty one".as_slice(), Start::Empty), (b"Copy a public one", Start::Copies)] {
            assert_action(&mut view, Role::Button, name, d::pages::Act::AddOpen { start });
            let node = main_find(view.tree(), Role::Button, name);
            assert_eq!(subtree(view.tree(), node)[0].styles.as_slice(), &[crate::look::HOME_SMALL]);
        }
        assert_action(
            &mut view,
            Role::Link,
            "Adopt another…".as_bytes(),
            d::pages::Act::Go { address: Address::Adopt { project: f::PROJECT } },
        );
        let node = main_find(view.tree(), Role::Link, "Adopt another…".as_bytes());
        assert!(subtree(view.tree(), node)[0].styles.as_slice().contains(&crate::look::HOME_QUIET));
    }
    contains(&printed(&f::page_view(&root)), "Add a repository");
}
#[test]
fn home_add_sheet_keeps_choices_fields_cancel_escape_and_input_in_both_shells() {
    use d::forge::forges::Start;
    use d::pages::home::AddField;
    for (start, role, name, field, value) in [
        (Start::Empty, Role::TextField, "Organization", AddField::Org, "work"),
        (Start::Copies, Role::TextArea, "Their addresses, one per line", AddField::Addresses, ""),
    ] {
        let (mut root, _) = home_root(add_rows(), false, false);
        add_act(&mut root, d::pages::Act::AddOpen { start });
        for mut view in [f::page_view(&root), f::browser_page_view(&root)] {
            main_find(view.tree(), Role::Dialog, b"Add a repository to Shop");
            main_find(view.tree(), Role::Paragraph, b"Made or copied on your Forgejo, then in this project at once.");
            main_find(view.tree(), Role::ChoiceGroup, b"A repository");
            absent(&f::main_text(view.tree()), "Nothing for now");
            for (choice, name) in
                [(Start::Empty, b"An empty repository".as_slice()), (Start::Copies, b"Copies of public repositories")]
            {
                let node = main_find(view.tree(), Role::Choice, name);
                assert_eq!(
                    subtree(view.tree(), node)[0].states.as_slice().contains(&skein_ui::State::Selected),
                    choice == start
                );
                assert_eq!(
                    crate::act(&mut view, node, Act::Toggle),
                    Some(d::Action::Pages(d::pages::Act::AddChoose { start: choice }))
                );
            }
            let input = main_find(view.tree(), role, name.as_bytes());
            assert_eq!(subtree(view.tree(), input)[0].value.as_ref().unwrap().text.as_ref(), value.as_bytes());
            assert_eq!(
                crate::act(&mut view, input, Act::Input { value: b"written".as_slice().into() }),
                Some(d::Action::Pages(d::pages::Act::AddWrite { field, words: b"written".as_slice().into() }))
            );
            assert_eq!(
                crate::act(&mut view, input, Act::Submit),
                if role == Role::TextField { Some(d::Action::Pages(d::pages::Act::AddSubmit)) } else { None }
            );
            let cancel = main_find(view.tree(), Role::Button, b"Cancel");
            assert_eq!(subtree(view.tree(), cancel)[0].shortcut, Some(skein_ui::Shortcut::ESCAPE));
            assert_action(&mut view, Role::Button, b"Cancel", d::pages::Act::AddClose);
            assert_action(&mut view, Role::Button, b"Add it", d::pages::Act::AddSubmit);
        }
        let text = printed(&f::page_view(&root));
        contains(&text, "Add a repository to Shop");
        contains(&text, name);
        add_act(&mut root, d::pages::Act::AddClose);
        assert!(root.pages().home().unwrap().add.is_none());
    }
}
#[test]
fn home_add_sheet_says_welcome_validation_alerts_in_both_shells() {
    use d::forge::forges::Start;
    use d::pages::home::AddField;
    for (start, field, text, expected) in [
        (Start::Empty, AddField::Org, "", "Both an organization and a name."),
        (Start::Copies, AddField::Addresses, "", "At least one address, as github.com/owner/name."),
        (Start::Copies, AddField::Addresses, "bad", "Not an address: bad. Write it as github.com/owner/name."),
    ] {
        let (mut root, _) = home_root(add_rows(), false, false);
        add_act(&mut root, d::pages::Act::AddOpen { start });
        add_act(&mut root, d::pages::Act::AddWrite { field, words: text.as_bytes().into() });
        add_act(&mut root, d::pages::Act::AddSubmit);
        for view in [f::page_view(&root), f::browser_page_view(&root)] {
            main_find(view.tree(), Role::Dialog, b"Add a repository to Shop");
            main_find(view.tree(), Role::Alert, expected.as_bytes());
        }
        contains(&printed(&f::page_view(&root)), expected);
    }
}
#[test]
fn home_empty_making_has_its_own_words_in_both_shells() {
    let mut row = add_rows().get(0).unwrap().clone().into_parts();
    row.makings = List::from_boxed(Box::new([w::RepositoryMaking::new(
        &w::CEILINGS,
        w::RepositoryMakingParts {
            org: b"work".as_slice().into(),
            name: b"hello".as_slice().into(),
            from: w::MakeFrom::Empty,
            into: Some(w::MakeInto::new(&w::CEILINGS, w::MakeIntoParts { project: f::PROJECT, home: true }).unwrap()),
            state: w::MakingState::Making,
        },
    )
    .unwrap()]));
    let (root, _) = home_root(List::from_boxed(Box::new([w::ForgeRow::new(&w::CEILINGS, row).unwrap()])), false, false);
    for view in [f::page_view(&root), f::browser_page_view(&root)] {
        main_find(view.tree(), Role::Paragraph, "Making work/hello…".as_bytes());
        absent(&f::main_text(view.tree()), "Copy");
        contains(&f::main_text(view.tree()), "Add a repository");
    }
    contains(&printed(&f::page_view(&root)), "Making work/hello…");
}

#[test]
fn home_add_waits_for_kept_and_reports_forge_refusals_in_both_shells() {
    use d::forge::forges::Start;
    for (why, words) in [
        (None, None),
        (Some(w::MakeRefusal::NoSuchForge), Some("no such forge")),
        (Some(w::MakeRefusal::NotOffered), Some("not offered")),
        (Some(w::MakeRefusal::NotOwner), Some("not owner")),
        (Some(w::MakeRefusal::Invalid), Some("invalid")),
        (Some(w::MakeRefusal::HomeHeld), Some("The project has a home already.")),
        (Some(w::MakeRefusal::Full), Some("full")),
    ] {
        let (mut root, _) = home_root(add_rows(), false, false);
        add_act(&mut root, d::pages::Act::AddOpen { start: Start::Empty });
        let mut sent = add_act(&mut root, d::pages::Act::AddSubmit);
        for view in [f::page_view(&root), f::browser_page_view(&root)] {
            let add = main_find(view.tree(), Role::Button, b"Add it");
            assert!(subtree(view.tree(), add)[0].states.as_slice().contains(&skein_ui::State::Disabled));
            main_find(view.tree(), Role::Dialog, b"Add a repository to Shop");
        }
        let mut stored = drive(&mut root, d::Event::Client { event: jig_client::Event::Stored });
        while let Some(request) = stored.pop() {
            sent.push(request);
        }
        let mut attempt = None;
        while let Some(request) = sent.pop() {
            if let d::Request::Unlinked {
                request: jig_client::UnlinkedRequest::Down { down: jig_client::Down::Send { attempt: found, .. } },
            } = request
            {
                attempt = Some(found);
            }
        }
        let outcome = match why {
            Some(why) => {
                w::Outcome::MakeRefused(w::MakeRefused::new(&w::CEILINGS, w::MakeRefusedParts { why }).unwrap())
            }
            None => w::Outcome::MakeKept,
        };
        let record = wire::answer(&w::Answer::new(&w::CEILINGS, w::AnswerParts { outcome }).unwrap());
        drive(
            &mut root,
            d::Event::Client {
                event: jig_client::Event::Up {
                    up: jig_client::Up::Answer {
                        attempt: attempt.unwrap(),
                        envelope: skein_lib::RequestEnvelope::Final,
                        why: None,
                        answer: Some(jig_client::Answered::Family(jig_client::Family {
                            family: wire::FORGE,
                            version: wire::CLIENT_VERSION,
                            bytes: record,
                        })),
                    },
                },
            },
        );
        for view in [f::page_view(&root), f::browser_page_view(&root)] {
            match words {
                Some(words) => {
                    let alert = skein_lib::bytes::concat(&[b"Could not add the repository: ", words.as_bytes()]);
                    main_find(view.tree(), Role::Alert, &alert);
                    let add = main_find(view.tree(), Role::Button, b"Add it");
                    assert!(!subtree(view.tree(), add)[0].states.as_slice().contains(&skein_ui::State::Disabled));
                }
                None => assert!(root.pages().home().unwrap().add.is_none()),
            }
        }
    }
}

struct AwayFixture {
    spell: Option<w::AwayRow>,
    rows: List<w::Decision>,
    state: d::pages::home::DecisionTask,
}
fn away_fixture(count: u32) -> AwayFixture {
    let mut rows = List::with_capacity(count);
    for number in 1..=count {
        rows.push(
            w::Decision::new(
                &w::CEILINGS,
                w::DecisionParts {
                    number,
                    task: 99,
                    at: STATUS_NOW - 55 * STATUS_MINUTE,
                    question: b"Fix in the store or serve?".as_slice().into(),
                    took: b"In serve".as_slice().into(),
                    why: b"The store changes a record.".as_slice().into(),
                    overturned: false,
                },
            )
            .unwrap(),
        )
        .unwrap();
    }
    AwayFixture {
        spell: Some(
            w::AwayRow::new(
                &w::CEILINGS,
                w::AwayRowParts {
                    project: f::PROJECT,
                    since: STATUS_NOW - 50 * STATUS_MINUTE,
                    until: None,
                    kinds: 7,
                    landed: 1,
                    off: None,
                },
            )
            .unwrap(),
        ),
        rows,
        state: d::pages::home::DecisionTask::Live,
    }
}
fn away_home(away: AwayFixture, beginning: bool, objects: &[c::ObjectKey], parts: w::ProjectStatusParts) -> d::Domain {
    let fixture = StatusFixture {
        document: w::ProjectStatus::new(&w::CEILINGS, parts).unwrap(),
        answered: true,
        goals: false,
        goal_rows: None,
        away: Some(away),
    };
    home_documents_with_status(
        forge_rows(true),
        !beginning,
        !beginning,
        Some(NeedsFixture { objects, cut: false, read: true }),
        None,
        Some(&[]),
        true,
        Some(&fixture),
    )
    .0
}
fn default_away(away: AwayFixture) -> d::Domain {
    away_home(away, false, &[], status_parts())
}
#[test]
fn the_kicker_reads_since_to_now() {
    let root = default_away(away_fixture(4));
    for browser in [false, true] {
        assert_eq!(kicker(&root, browser).as_ref(), "While you were away · Sun 23:10 to now".as_bytes());
    }
}
#[test]
fn the_kicker_reads_since_to_its_end_once_off() {
    for until in [false, true] {
        let mut away = away_fixture(1);
        let mut spell = away.spell.take().unwrap().into_parts();
        let end = STATUS_NOW - 10 * STATUS_MINUTE;
        if until {
            spell.until = Some(end);
        } else {
            spell.off = Some(end);
        }
        away.spell = Some(w::AwayRow::new(&w::CEILINGS, spell).unwrap());
        let root = default_away(away);
        for browser in [false, true] {
            assert_eq!(kicker(&root, browser).as_ref(), "While you were away · Sun 23:10 to Sun 23:50".as_bytes());
        }
    }
}
#[test]
fn the_kicker_alone_with_no_row() {
    let mut away = away_fixture(1);
    away.spell = None;
    let root = default_away(away);
    for browser in [false, true] {
        assert_eq!(kicker(&root, browser).as_ref(), b"While you were away");
    }
}
#[test]
fn the_headline_says_landed_then_decided() {
    for (count, landed, headline) in [
        (4, 58, "57 changes landed. Four things were decided for you."),
        (10, 2, "One change landed. 10 things were decided for you."),
    ] {
        let mut parts = status_parts();
        parts.landed = landed;
        let root = away_home(away_fixture(count), false, &[], parts);
        for browser in [false, true] {
            main_find(status_view(&root, browser).tree(), Role::Heading { level: 1 }, headline.as_bytes());
        }
    }
}
#[test]
fn none_for_no_changes() {
    for delivered in [false, true] {
        let mut away = away_fixture(4);
        if !delivered {
            away.spell = None;
        }
        let mut parts = status_parts();
        parts.landed = 1;
        let root = away_home(away, false, &[], parts);
        main_find(status_view(&root, true).tree(), Role::Heading { level: 1 }, b"Four things were decided for you.");
    }
}
#[test]
fn one_thing_was_decided() {
    let root = default_away(away_fixture(1));
    for browser in [false, true] {
        main_find(
            status_view(&root, browser).tree(),
            Role::Heading { level: 1 },
            b"2 changes landed. One thing was decided for you.",
        );
    }
}
#[test]
fn lede_and_byline_stand() {
    let root = away_home(away_fixture(1), false, &[], brief_parts(2, BRIEF_LEDE, STATUS_NOW));
    for browser in [false, true] {
        let view = status_view(&root, browser);
        assert_eq!(styled(view.tree(), crate::look::HOME_LEDE).name.as_deref(), Some(BRIEF_LEDE.as_bytes()));
        assert_eq!(
            styled(view.tree(), crate::look::HOME_BYLINE).name.as_deref(),
            Some(b"Written by the status agent at 00:00".as_slice())
        );
    }
}
#[test]
fn no_form_with_no_row_showing() {
    for beginning in [false, true] {
        let root = away_home(away_fixture(u32::from(beginning)), beginning, &[], status_parts());
        let view = status_view(&root, true);
        absent(&f::main_text(view.tree()), "decided for you");
        for node in main_nodes(view.tree()) {
            assert!(!node.styles.as_slice().contains(&crate::look::DECIDED));
        }
    }
}
#[test]
fn the_section_comes_first_after_the_intro() {
    let root = default_away(away_fixture(1));
    for browser in [false, true] {
        let view = status_view(&root, browser);
        let sections = children(main_nodes(view.tree()));
        assert!(sections[0].styles.as_slice().contains(&crate::look::HOME_INTRO));
        assert_eq!(sections[1].role, Role::Region);
        assert_eq!(sections[1].name.as_deref(), Some("Decided for you · 1".as_bytes()));
        main_find(view.tree(), Role::Heading { level: 2 }, "Decided for you · 1".as_bytes());
    }
}
#[test]
fn its_count_is_every_row_showing() {
    let root = default_away(away_fixture(10));
    let view = status_view(&root, true);
    main_find(view.tree(), Role::Heading { level: 2 }, "Decided for you · 10".as_bytes());
    let mut count = 0_u32;
    for node in main_nodes(view.tree()) {
        if node.styles.as_slice().contains(&crate::look::DECISION) {
            count += 1;
        }
    }
    assert_eq!(count, 7);
}
#[test]
fn limits_goes_to_settings() {
    let root = default_away(away_fixture(1));
    for browser in [false, true] {
        let mut view = status_view(&root, browser);
        let id = main_find(view.tree(), Role::Button, b"Limits");
        let (node, act) = if browser {
            (id, Act::Press)
        } else {
            f::terminal_command(view.tree(), &view.limits().tree, id, b"Limits", "Decided for you · 1".as_bytes())
        };
        assert_eq!(
            crate::act(&mut view, node, act),
            Some(d::Action::Pages(d::pages::Act::Away(d::pages::away::Act::Limits { project: f::PROJECT })))
        );
    }
}

#[test]
fn a_row_says_title_and_clock() {
    let root = default_away(away_fixture(1));
    for browser in [false, true] {
        let view = status_view(&root, browser);
        let row = styled(view.tree(), crate::look::DECISION);
        let text = f::nodes_text(subtree(view.tree(), row.id));
        contains(&text, "welcome-stall · Sun 23:05");
        contains(&text, "Fix in the store or serve?");
        contains(&text, "In serve");
        contains(&text, "The store changes a record.");
        main_find(view.tree(), Role::Button, b"Overturn");
        if !browser {
            contains(
                &printed(&view),
                "welcome-stall · Sun 23:05 Fix in the store or serve? In serve The store changes a record.",
            );
        }
    }
}
#[test]
fn an_unread_task_says_task_n() {
    let mut away = away_fixture(1);
    away.state = d::pages::home::DecisionTask::Unknown;
    let root = default_away(away);
    contains(&f::main_text(status_view(&root, true).tree()), "Task 99 · Sun 23:05");
}
fn assert_decision_end(state: d::pages::home::DecisionTask, overturned: bool, late: &str) {
    let mut away = away_fixture(1);
    away.state = state;
    let mut row = away.rows.get(0).unwrap().clone().into_parts();
    row.overturned = overturned;
    *away.rows.get_mut(0).unwrap() = w::Decision::new(&w::CEILINGS, row).unwrap();
    let root = default_away(away);
    for browser in [false, true] {
        let view = status_view(&root, browser);
        assert_eq!(styled(view.tree(), crate::look::DECISION_LATE).name.as_deref(), Some(late.as_bytes()));
        assert_eq!(
            styled(view.tree(), crate::look::DECISION).styles.as_slice().contains(&crate::look::DECISION_OVERTURNED),
            overturned
        );
        assert_eq!(styled(view.tree(), crate::look::DECISION_TOOK).name.as_deref(), Some(b"In serve".as_slice()));
    }
}
#[test]
fn done_says_landed() {
    assert_decision_end(
        d::pages::home::DecisionTask::Done(skein_lib::Wall::from_nanos(STATUS_NOW)),
        false,
        "landed 00:00 · a new goal can undo it",
    );
}
#[test]
fn ended_says_ended() {
    assert_decision_end(
        d::pages::home::DecisionTask::Ended(skein_lib::Wall::from_nanos(STATUS_NOW)),
        false,
        "ended 00:00 · a new goal can undo it",
    );
}
#[test]
fn overturned_says_it_asks_again_and_strikes_took() {
    assert_decision_end(d::pages::home::DecisionTask::Live, true, "Overturned: it asks you again");
}
#[test]
fn ended_and_overturned_says_landed_and_strikes_took() {
    assert_decision_end(
        d::pages::home::DecisionTask::Done(skein_lib::Wall::from_nanos(STATUS_NOW)),
        true,
        "landed 00:00 · a new goal can undo it",
    );
}
#[test]
fn needs_head_says_still() {
    for objects in [&[][..], &QUESTIONS[..2]] {
        let root = away_home(away_fixture(1), false, objects, status_parts());
        let heading = if objects.is_empty() { "Still needs you" } else { "Still needs you · 2" };
        for browser in [false, true] {
            main_find(status_view(&root, browser).tree(), Role::Heading { level: 2 }, heading.as_bytes());
        }
    }
}

#[test]
fn the_oldest_expiry_wakes_home() {
    let mut away = away_fixture(1);
    let mut row = away.rows.get(0).unwrap().clone().into_parts();
    row.at = STATUS_NOW - 86_400_000_000_000 + 30_000_000_000;
    *away.rows.get_mut(0).unwrap() = w::Decision::new(&w::CEILINGS, row).unwrap();
    let root = default_away(away);
    for browser in [false, true] {
        assert_eq!(
            status_view(&root, browser).next_change(),
            Some(skein_lib::Wall::from_nanos(STATUS_NOW + 30_000_000_000))
        );
    }
}

#[test]
fn a_live_row_offers_overturn() {
    let root = default_away(away_fixture(1));
    for browser in [false, true] {
        let mut view = status_view(&root, browser);
        let id = main_find(view.tree(), Role::Button, b"Overturn");
        assert_eq!(
            crate::act(&mut view, id, Act::Press),
            Some(d::Action::Pages(d::pages::Act::Away(d::pages::away::Act::Overturn {
                project: f::PROJECT,
                number: 1
            })))
        );
    }
}
#[test]
fn an_overturned_row_offers_none() {
    let mut away = away_fixture(1);
    let mut row = away.rows.get(0).unwrap().clone().into_parts();
    row.overturned = true;
    *away.rows.get_mut(0).unwrap() = w::Decision::new(&w::CEILINGS, row).unwrap();
    let root = default_away(away);
    for browser in [false, true] {
        let view = status_view(&root, browser);
        assert!(f::find(view.tree(), Role::Button, b"Overturn").is_none());
    }
}
#[test]
fn only_live_rows_offer_overturn() {
    for state in [
        d::pages::home::DecisionTask::Unknown,
        d::pages::home::DecisionTask::Done(skein_lib::Wall::EPOCH),
        d::pages::home::DecisionTask::Ended(skein_lib::Wall::EPOCH),
    ] {
        let mut away = away_fixture(1);
        away.state = state;
        let root = default_away(away);
        for browser in [false, true] {
            assert!(f::find(status_view(&root, browser).tree(), Role::Button, b"Overturn").is_none());
        }
    }
}
fn overturn_drive(root: &mut d::Domain, event: d::Event) -> Queue<d::Request> {
    let mut limits = home_limits();
    limits.client.domain.text = 512;
    let mut out = Queue::uncapped();
    d::step(
        root,
        &skein_lib::Env { now: skein_lib::Time::ZERO, wall: skein_lib::Wall::from_nanos(STATUS_NOW), limits },
        event,
        &mut out,
    );
    out
}
fn overturn_attempt(out: &Queue<d::Request>) -> Token {
    for request in out {
        if let d::Request::Unlinked {
            request: jig_client::UnlinkedRequest::Down { down: jig_client::Down::Send { attempt, .. } },
        } = request
        {
            return *attempt;
        }
    }
    panic!("overturn attempt")
}
pub(super) fn overturn_answer(outcome: w::Outcome, message: Option<c::Answer>) -> d::Domain {
    let mut root = default_away(away_fixture(2));
    overturn_drive(
        &mut root,
        d::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::FamilyServed {
                    family: wire::FORGE,
                    versions: Some(c::VersionRange { first: wire::CLIENT_VERSION, last: wire::CLIENT_VERSION }),
                },
            },
        },
    );
    let out = overturn_drive(
        &mut root,
        d::Event::Act {
            action: d::Action::Pages(d::pages::Act::Away(d::pages::away::Act::Overturn {
                project: f::PROJECT,
                number: 1,
            })),
        },
    );
    let attempt = overturn_attempt(&out);
    let out = overturn_drive(
        &mut root,
        d::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Answer {
                    attempt,
                    envelope: skein_lib::RequestEnvelope::Final,
                    why: None,
                    answer: Some(jig_client::Answered::Family(jig_client::Family {
                        family: wire::FORGE,
                        version: wire::CLIENT_VERSION,
                        bytes: wire::answer(&w::Answer::new(&w::CEILINGS, w::AnswerParts { outcome }).unwrap()),
                    })),
                },
            },
        },
    );
    if let Some(answer) = message {
        let attempt = overturn_attempt(&out);
        overturn_drive(
            &mut root,
            d::Event::Client {
                event: jig_client::Event::Up {
                    up: jig_client::Up::Answer {
                        attempt,
                        envelope: skein_lib::RequestEnvelope::Final,
                        why: None,
                        answer: Some(jig_client::Answered::Jig(answer)),
                    },
                },
            },
        );
    }
    root
}
#[test]
fn another_overturn_disables_the_button() {
    let root = overturn_answer(w::Outcome::Overturned, None);
    for browser in [false, true] {
        let view = status_view(&root, browser);
        let at = main_find(view.tree(), Role::Button, b"Overturn");
        assert!(
            subtree(view.tree(), at)
                .first()
                .expect("decision button")
                .states
                .as_slice()
                .contains(&skein_ui::State::Disabled)
        );
    }
}

#[test]
fn a_brief_written_since_the_last_look_shows_with_nothing_new_landed() {
    let mut parts = brief_parts(1, BRIEF_LEDE, STATUS_NOW);
    parts.landed = 1;
    parts.seen = 1;
    let root = status_home(parts, true, false, false);
    for browser in [false, true] {
        let view = status_view(&root, browser);
        contains(&f::main_text(view.tree()), BRIEF_SENTENCE);
        assert_eq!(styled(view.tree(), crate::look::HOME_LEDE).name.as_deref(), Some(BRIEF_LEDE.as_bytes()));
        assert_eq!(
            styled(view.tree(), crate::look::HOME_BYLINE).name.as_deref(),
            Some(b"Written by the status agent at 00:00".as_slice())
        );
    }
}

#[test]
fn an_old_brief_gives_way_to_the_last_landing() {
    for previous in [STATUS_NOW - STATUS_MINUTE, STATUS_NOW] {
        let mut parts = brief_parts(1, BRIEF_LEDE, STATUS_NOW - STATUS_MINUTE);
        parts.landed = 1;
        parts.seen = 1;
        parts.previous = Some(previous);
        let root = status_home(parts, true, false, false);
        for browser in [false, true] {
            let view = status_view(&root, browser);
            contains(&f::main_text(view.tree()), "The last change landed 6 minutes ago. Nothing needs you.");
            no_brief_prose(&view);
        }
    }
}

#[test]
fn a_landing_never_stands_under_nothing_needs_you_alone() {
    for (landed, last, beginning, expected) in [
        (1, Some(STATUS_NOW - 6 * STATUS_MINUTE), false, "The last change landed 6 minutes ago. Nothing needs you."),
        (1, Some(STATUS_NOW - STATUS_MINUTE), false, "The last change landed a minute ago. Nothing needs you."),
        (1, Some(STATUS_NOW - 1), false, "The last change landed just now. Nothing needs you."),
        (0, None, false, "Nothing needs you."),
        (1, Some(STATUS_NOW), true, "Shop is ready."),
    ] {
        let mut parts = status_parts();
        parts.landed = landed;
        parts.seen = landed;
        parts.last_landed = last;
        let root = status_home(parts, true, beginning, false);
        for browser in [false, true] {
            let mut view = status_view(&root, browser);
            main_find(view.tree(), Role::Heading { level: 1 }, expected.as_bytes());
            contains(&printed(&view), expected);
            if !beginning {
                no_brief_prose(&view);
            }
            if landed > 0 && !beginning {
                assert!(view.next_change().is_some());
                crate::build(
                    &mut view,
                    &root,
                    skein_lib::Wall::from_nanos(STATUS_NOW + STATUS_MINUTE),
                    &mut Queue::uncapped(),
                );
                let text = f::main_text(view.tree());
                if last == Some(STATUS_NOW - 6 * STATUS_MINUTE) {
                    contains(&text, "The last change landed 7 minutes ago.");
                }
            }
        }
    }
}
