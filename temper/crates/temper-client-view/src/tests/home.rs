use super::fixtures as f;
use skein_ui::{Act, Role};
use temper_client_domain::{self as d, Address, ChatsFilter};

const UNKNOWN: u32 = 99;

// Home's try survives leaving Ready (temper/docs/design/domain/trial.md, 6.3).
fn trial_home(beginning: bool) -> d::Domain {
    let (mut root, accounts) = home_root(forge_rows(true), !beginning, !beginning);
    super::accounts::deliver(
        &mut root,
        accounts,
        super::accounts::card_document(b"anthropic", b"owner", c::AccountKind::Program),
    );
    assert_eq!(d::pages::home::facts(&root, f::PROJECT, f::NOW).to_begin(), beginning);
    root
}

fn trial_rows(root: &mut d::Domain, state: Option<d::forge::trials::State>) {
    let rows: Box<[d::forge::trials::Row]> = match state {
        Some(state) => Box::new([d::forge::trials::Row {
            account: 1,
            task: Some(91),
            took: None,
            step: d::forge::trials::Step::Answers,
            state,
        }]),
        None => Box::default(),
    };
    drive(
        root,
        d::Event::Forge(d::forge::Event::Trials { event: d::forge::trials::Event::Rows { rows }, adoption: None }),
    );
}

fn failed_trial() -> d::forge::trials::State {
    d::forge::trials::State::Failed { why: d::forge::trials::Why::Waited }
}

const TRY_STOPPED: &str = "Claude didn’t answer when tried";

#[test]
#[expect(clippy::disallowed_methods, reason = "test assertions inspect semantic node positions and attributes")]
fn home_says_a_try_that_stopped() {
    for beginning in [false, true] {
        let mut root = trial_home(beginning);
        trial_rows(&mut root, Some(failed_trial()));
        for view in [f::page_view(&root), f::browser_page_view(&root)] {
            let intro = styled(view.tree(), crate::look::HOME_INTRO);
            let nodes = subtree(view.tree(), intro.id);
            let direct = children(nodes);
            let mut last = direct.iter().rev();
            assert_eq!(last.next().expect("retry").name.as_deref(), Some(b"Try again".as_slice()));
            assert_eq!(last.next().expect("stopped line").name.as_deref(), Some(TRY_STOPPED.as_bytes()));
            let line = nodes
                .iter()
                .position(|n| n.role == Role::Paragraph && n.name.as_deref() == Some(TRY_STOPPED.as_bytes()))
                .expect("stopped try under intro");
            let retry = nodes
                .iter()
                .position(|n| n.role == Role::Button && n.name.as_deref() == Some(b"Try again"))
                .expect("retry under intro");
            assert!(line < retry);
            assert!(nodes.get(line).unwrap().styles.as_slice().contains(&crate::look::HOME_BYLINE));
            assert!(nodes.iter().take(line).any(|n| n.role == Role::Heading { level: 1 }));
            let output = printed(&view);
            let line = skein_lib::bytes::find(&output, TRY_STOPPED.as_bytes()).expect("printed stopped try");
            let retry = skein_lib::bytes::find(&output, b"Try again").expect("printed retry");
            assert!(line < retry);
        }
    }
}

fn silent_trial(state: d::forge::trials::State) {
    for beginning in [false, true] {
        let mut root = trial_home(beginning);
        trial_rows(&mut root, Some(state));
        for view in [f::page_view(&root), f::browser_page_view(&root)] {
            absent(&f::main_text(view.tree()), TRY_STOPPED);
            absent(&f::main_text(view.tree()), "Try again");
        }
    }
}

#[test]
fn answered_says_nothing_more() {
    silent_trial(d::forge::trials::State::Answered { seconds: 6 });
}

#[test]
fn running_says_nothing() {
    silent_trial(d::forge::trials::State::Running);
}

#[test]
fn try_again_runs_another_and_the_line_goes() {
    for beginning in [false, true] {
        for browser in [false, true] {
            let mut root = trial_home(beginning);
            trial_rows(&mut root, Some(failed_trial()));
            let mut view = if browser { f::browser_page_view(&root) } else { f::page_view(&root) };
            let retry = main_find(view.tree(), Role::Button, b"Try again");
            let action = crate::act(&mut view, retry, Act::Press).expect("retry acts");
            assert_eq!(action, d::Action::Forge(d::forge::Act::TryAccount { account: 1, again: true }));
            drive(&mut root, d::Event::Act { action });
            assert_eq!(root.forge().trial_asked(), Some(1));
            for running in [false, true] {
                if running {
                    trial_rows(&mut root, Some(d::forge::trials::State::Running));
                    assert_eq!(root.forge().trial_asked(), None);
                }
                f::build(&mut view, &root);
                absent(&f::main_text(view.tree()), TRY_STOPPED);
                absent(&f::main_text(view.tree()), "Try again");
            }
        }
    }
}

#[test]
fn a_home_with_no_trial_reads_as_today() {
    for beginning in [false, true] {
        let mut root = trial_home(beginning);
        let before = [f::page_view(&root), f::browser_page_view(&root)];
        trial_rows(&mut root, Some(failed_trial()));
        trial_rows(&mut root, None);
        for (before, after) in before.iter().zip([f::page_view(&root), f::browser_page_view(&root)]) {
            absent(&f::main_text(after.tree()), TRY_STOPPED);
            absent(&f::main_text(after.tree()), "Try again");
            assert_eq!(before.tree().nodes(), after.tree().nodes(), "no trial changes no node");
        }
    }
}

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
        let (mut root, read) = missing_project(address, UNKNOWN);
        answer_missing_project(&mut root, read, c::ReadNone::Unknown);
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

fn missing_project(address: Address, project: u32) -> (d::Domain, Token) {
    let limits = f::domain_limits();
    let (mut root, _, _) = f::root_at(Address::AllHome, &limits);
    let mut out = Queue::uncapped();
    d::step(
        &mut root,
        &skein_lib::Env { now: skein_lib::Time::ZERO, wall: f::NOW, limits },
        d::Event::Went { place: d::Place::Address(address) },
        &mut out,
    );
    let mut token = None;
    for request in &out {
        if let d::Request::Unlinked {
            request:
                jig_client::UnlinkedRequest::Down {
                    down:
                        jig_client::Down::Read {
                            read,
                            what: jig_client::Readable::Jig(c::Read::Project { project: number }),
                        },
                },
        } = request
            && *number == project
        {
            assert!(token.replace(*read).is_none(), "one project read");
        }
    }
    (root, token.expect("project read"))
}

fn answer_missing_project(root: &mut d::Domain, read: Token, why: c::ReadNone) {
    let mut out = Queue::uncapped();
    d::step(
        root,
        &skein_lib::Env { now: skein_lib::Time::ZERO, wall: f::NOW, limits: f::domain_limits() },
        d::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::ReadDone { read, result: jig_client::Readout::Jig(c::ReadResult::None(why)) },
            },
        },
        &mut out,
    );
}

#[test]
fn missing_project_pages_wait_then_name_deleted_projects_in_both_shells() {
    const PROJECT: u32 = 3;
    for address in [Address::Home { project: PROJECT }, Address::Settings { project: PROJECT }] {
        let (mut root, read) = missing_project(address, PROJECT);
        for view in [f::page_view(&root), f::browser_page_view(&root)] {
            let main = main_nodes(view.tree());
            let mut headings = 0_u32;
            for node in main {
                if node.role == (Role::Heading { level: 1 }) {
                    headings = headings.checked_add(1).expect("headings fit");
                }
                assert!(
                    node.role != Role::Paragraph
                        && node.role != Role::Link
                        && node.role != Role::Button
                        && node.role != Role::Navigation,
                    "only the heading while reading, found {:?}",
                    node.role
                );
            }
            assert_eq!(headings, 1);
        }
        answer_missing_project(&mut root, read, c::ReadNone::Gone);
        for mut view in [f::page_view(&root), f::browser_page_view(&root)] {
            let text = f::main_text(view.tree());
            assert!(skein_lib::bytes::find(&text, b"Project 3 was deleted.").is_some());
            assert!(skein_lib::bytes::find(&text, b"not one of yours").is_none());
            assert!(skein_lib::bytes::find(&printed(&view), b"Project 3 was deleted.").is_some());
            let link = main_find(view.tree(), Role::Link, b"Chats");
            assert_eq!(
                crate::act(&mut view, link, Act::Press),
                Some(d::Action::Pages(d::pages::Act::Go { address: Address::Chats { filter: ChatsFilter::All } }))
            );
        }
    }
}

#[test]
fn missing_project_none_answers_other_than_gone_keep_not_yours() {
    for address in [Address::Home { project: 3 }, Address::Settings { project: 3 }] {
        for why in [c::ReadNone::Unknown, c::ReadNone::Refused, c::ReadNone::TooLarge { bytes: 1 }] {
            let (mut root, read) = missing_project(address.clone(), 3);
            answer_missing_project(&mut root, read, why);
            for view in [f::page_view(&root), f::browser_page_view(&root)] {
                let text = f::main_text(view.tree());
                assert!(skein_lib::bytes::find(&text, b"Project 3 is not one of yours.").is_some());
                let _link = main_find(view.tree(), Role::Link, b"Chats");
            }
        }
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
        let (mut root, watch) =
            f::forge_root_accounts(Address::Accounts, super::connect_forge::rows(connection, w::Offer::Offered));
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
    // Home, stock ways, goal presence, Goals, standing Needs, trials and a running trial task (client/pages/README.md).
    limits.client.domain.handles = 18;
    limits.client.domain.watches = 16;
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
    decided: bool,
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
    shown: bool,
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
        f::home_root(&home_limits(), rows, dismissed, status.is_some_and(|fixture| fixture.shown));
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
    member.templates = f::asking_me(template_me.clone()).projects.get(0).unwrap().templates.clone();
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
            new_until: None,
            number: 1,
            source: jig_client::contract::ModelSource::Stock,
            estimated: false,
            effort: None,
            shown: None,
            accounts: List::with_capacity(0),
            picker: true,
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
            setting: c::Setting::Application { kind: 2, scope: f::PROJECT, number: 1 },
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
                if status.and_then(|s| s.away.as_ref()).is_none()
                    && needs.is_some_and(|n| n.decided)
                    && Some(&subject) == d::forge::decided::watch(f::PROJECT, &home_limits().forge).as_ref() =>
            {
                drive(
                    root,
                    d::Event::Client {
                        event: jig_client::Event::Up {
                            up: jig_client::Up::FamilyDelivered {
                                watch,
                                record: jig_client::Family {
                                    family: wire::FORGE,
                                    version: wire::CLIENT_VERSION,
                                    bytes: wire::decisions(
                                        &w::Decisions::new(
                                            &w::CEILINGS,
                                            w::DecisionsParts {
                                                project: f::PROJECT,
                                                rows: List::with_capacity(0),
                                                asked: List::with_capacity(0),
                                                cut: false,
                                            },
                                        )
                                        .unwrap(),
                                    ),
                                },
                            },
                        },
                    },
                )
            }
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
                            w::DecisionsParts {
                                project: f::PROJECT,
                                rows: fixture.rows.clone(),
                                asked: List::with_capacity(0),
                                cut: false,
                            },
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
                if fixture.state == d::pages::decided::DecisionTask::Unknown {
                    continue;
                }
                let mut document = f::task::document();
                document.chip.task = task;
                document.chip.title = f::task::document_words(b"welcome-stall");
                match fixture.state {
                    d::pages::decided::DecisionTask::Unknown => unreachable!(),
                    d::pages::decided::DecisionTask::Live => {}
                    d::pages::decided::DecisionTask::Done(at) | d::pages::decided::DecisionTask::Ended(at) => {
                        document.chip.details.phase = c::ChipPhase::Ended;
                        document.chip.details.details.ended_at = Some(at);
                        document.chip.details.details.ending =
                            if let d::pages::decided::DecisionTask::Done(_) = fixture.state {
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
            jig_client::Down::Open { watch, subject: jig_client::Watchable::Jig(c::Subject::Tasks { .. }) }
                if chats.is_some() =>
            {
                drive(
                    root,
                    d::Event::Client {
                        event: jig_client::Event::Up {
                            up: jig_client::Up::Watched {
                                watch,
                                event: c::WatchEvent::Delivered(c::Delivery::Tasks(Box::new(
                                    chats.map_or(c::Tasks { rows: List::with_capacity(0), more: false }, |chats| {
                                        chats.live.clone()
                                    }),
                                ))),
                            },
                        },
                    },
                )
            }
            jig_client::Down::Close { watch } => drive(
                root,
                d::Event::Client {
                    event: jig_client::Event::Up { up: jig_client::Up::WatchEnded { watch, end: c::WatchEnd::Closed } },
                },
            ),
            jig_client::Down::Save { .. }
            | jig_client::Down::Erase { .. }
            | jig_client::Down::Send { .. }
            | jig_client::Down::Open { .. }
            | jig_client::Down::Continue { .. }
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
    tree.nodes().iter().find(|node| node.styles.as_slice().contains(&style)).expect("styled Home node")
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

fn assert_first_card_action(view: &mut crate::View, name: &[u8], action: d::pages::Act) {
    let card = styled(view.tree(), crate::look::NEED_CARD);
    let mut id = None;
    for node in subtree(view.tree(), card.id) {
        if node.role == Role::Button && node.name.as_deref() == Some(name) {
            id = Some(node.id);
        }
    }
    assert_eq!(crate::act(view, id.unwrap(), Act::Press), Some(d::Action::Pages(action)));
}
fn home_foot(tree: &skein_ui::Tree) -> &skein_ui::Node {
    let mut foot = None;
    for node in children(main_nodes(tree)) {
        if node.styles.as_slice().contains(&crate::look::HOME_FACTS) {
            foot = Some(node);
        }
    }
    foot.unwrap()
}

#[test]
#[expect(clippy::disallowed_methods, reason = "test assertions inspect semantic nodes and their event bindings")]
fn home_ready_headline_and_done_steps_hold_in_both_shells() {
    for configured in [false, true] {
        for repository in [false, true] {
            for chat in [false, true] {
                let (root, _) =
                    home_root(if repository { forge_rows(true) } else { List::with_capacity(0) }, configured, chat);
                let left = u8::from(!configured) + u8::from(!repository);
                let headline = if left > 0 { "Shop is ready." } else { "Shop" };
                for view in [f::page_view(&root), f::browser_page_view(&root)] {
                    main_find(view.tree(), Role::Heading { level: 1 }, headline.as_bytes());
                    assert_eq!(
                        main_nodes(view.tree())
                            .iter()
                            .filter(|node| node.styles.as_slice().contains(&crate::look::HOME_DONE))
                            .count(),
                        if left == 0 { 0 } else { usize::from(2 - left) }
                    );
                    assert_eq!(
                        main_nodes(view.tree())
                            .iter()
                            .any(|node| node.role == Role::Button && node.name.as_deref() == Some(b"Start")),
                        !configured
                    );
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
                        absent(&main, "Start a chat");
                        absent(&main, "Ask about the code or ask for a change.");
                        if repository {
                            contains(&main, "work/hello is in Shop");
                            absent(&main, "CI ran on main and passed");
                            contains(&main, "temper can change its code now.");
                        }
                    }
                }
                let terminal = printed(&f::page_view(&root));
                assert!(terminal.strip_prefix(b"== ").unwrap().starts_with(headline.as_bytes()));
                absent(&terminal, "Connected");
                for words in ["To begin", "1. ", "2. "] {
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
                | d::Request::Screen { .. }
                | d::Request::KeepLook { .. }
                | d::Request::KeepSignIn { .. }
                | d::Request::Tab { .. }
                | d::Request::Copy { .. }
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
                contains(&words, "temper can change its code once it is in.");
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
                        "Until one is in, temper talks and plans but changes no code."
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
                d::pages::Act::Go { address: Address::Ask { project: f::PROJECT } },
            );
            assert!(
                !main_nodes(view.tree())
                    .iter()
                    .any(|node| node.role == Role::Link && node.name.as_deref() == Some(b"New chat"))
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
    assert_eq!(steps.len(), 2);
    for (step, done) in steps.iter().zip([true, false]) {
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
    assert_eq!(children(subtree(tree, steps[0].id))[0].name.as_deref(), Some(b"work/hello is in Shop".as_slice()));
    assert_eq!(children(subtree(tree, steps[1].id))[0].name.as_deref(), Some(b"Tell temper how you work".as_slice()));
    assert_action(&mut view, Role::Button, b"Skip", d::pages::Act::SkipToBegin);
    let mut terminal = f::view();
    let mut sign_in = crate::forge_sign_in::Forges::new();
    sign_in.read("3 forgejo http://forge.example:3000 owned");
    terminal.forge_sign_in(sign_in);
    f::build(&mut terminal, &root);
    let printed = printed(&terminal);
    contains(&printed, "To begin");
    contains(&printed, "Optional Skip\n");
    contains(&printed, "2. Tell temper how you work");
    contains(&printed, "1. Done. work/hello is in Shop");
    absent(&printed, "3. Start a chat");
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
        absent(&text, "Until one is in, temper talks and plans but changes no code.");
        absent(&text, "Shop has no repository yet.");
        absent(&text, "connected as");
    }
    let root = f::root_at(Address::Home { project: f::PROJECT }, &f::domain_limits()).0;
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
    home_with_needs(forge_rows(true), !beginning, true, Some(NeedsFixture { objects, cut, read, decided: true })).0
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
    children(main_nodes(tree))
        .iter()
        .filter(|node| !node.styles.as_slice().contains(&crate::look::ASK_LINE))
        .map(|node| node.styles.as_slice()[0])
        .collect()
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
            let styles = section_styles(view.tree());
            assert_eq!(styles.contains(&crate::look::HOME_NEEDS), !objects.is_empty());
            assert!(
                !main_nodes(view.tree()).iter().any(|node| node.styles.as_slice().contains(&crate::look::HOME_LEDE))
            );
            absent(&f::main_text(view.tree()), "Work carries on");
            assert_eq!(f::find(view.tree(), Role::Region, b"Needs you").is_some(), !objects.is_empty());
        }
    }
}
#[test]
#[expect(clippy::disallowed_methods, reason = "the table inspects semantic cards in both shells")]
fn first_need_keeps_card_order_navigation_and_terminal_words() {
    for beginning in [false, true] {
        let root = everyday(beginning, &[PARSED], false, true);
        for (mut view, browser) in [(f::page_view(&root), false), (f::browser_page_view(&root), true)] {
            let nodes = main_nodes(view.tree());
            assert_eq!(
                nodes.iter().filter(|n| n.styles.as_slice().contains(&crate::look::NEED_CARD)).count(),
                usize::from(!beginning)
            );
            if beginning {
                continue;
            }
            let card = styled(view.tree(), crate::look::NEED_CARD);
            let nodes = subtree(view.tree(), card.id);
            assert!(!nodes.iter().any(|n| n.styles.as_slice().contains(&crate::look::CHIP)));
            assert!(nodes.iter().any(|n| n.styles.as_slice().contains(&crate::look::NEED_FROM)));
            assert!(nodes.iter().any(|n| n.name.as_deref() == Some(b"Which license?")));
            assert_eq!(nodes.iter().any(|n| n.styles.as_slice().contains(&crate::look::NEED_MORE)), browser);
            assert_action(
                &mut view,
                Role::Button,
                b"MIT",
                d::pages::Act::Needs(d::pages::needs::Act::Choose { object: PARSED, words: b"MIT".as_slice().into() }),
            );
            assert_first_card_action(
                &mut view,
                b"Tell it something else",
                d::pages::Act::Needs(d::pages::needs::Act::Open { object: PARSED }),
            );
            if browser {
                assert_first_card_action(
                    &mut view,
                    b"More",
                    d::pages::Act::Needs(d::pages::needs::Act::Open { object: PARSED }),
                );
            }
            assert_action(&mut view, Role::Link, b"All needs", inbox_act());
            absent(&printed(&view), "Answer in a row");
        }
    }
    let mut root = everyday(false, &[], false, true);
    let row = super::needs::approval_row();
    super::needs::approvals(&mut root, core::slice::from_ref(&row), 0);
    for view in [f::page_view(&root), f::browser_page_view(&root)] {
        contains(&f::main_text(view.tree()), "Needs you · 1");
        let id = f::find(view.tree(), Role::Link, "Review on Forgejo ↗".as_bytes()).unwrap();
        assert_eq!(view.tree().external(id), Some(row.address.as_ref()));
    }
}
#[test]
fn unread_need_waits_in_both_shells_and_keeps_unknown_headline() {
    let root = everyday(false, &QUESTIONS[..1], false, false);
    for view in [f::page_view(&root), f::browser_page_view(&root)] {
        main_find(view.tree(), Role::Heading { level: 1 }, b"Shop");
        assert!(!section_styles(view.tree()).contains(&crate::look::HOME_NEEDS));
    }
}
#[test]
#[expect(clippy::disallowed_methods, clippy::disallowed_macros, reason = "the table inspects card counts and actions")]
fn additional_needs_stop_at_three_and_link_to_the_project() {
    for (count, more) in [(1, None), (3, None), (4, Some("One more")), (5, Some("2 more")), (7, Some("4 more"))] {
        let root = everyday(false, &QUESTIONS[..count], false, true);
        for mut view in [f::page_view(&root), f::browser_page_view(&root)] {
            let region = styled(view.tree(), crate::look::HOME_NEEDS);
            let nodes = subtree(view.tree(), region.id);
            assert_eq!(
                nodes.iter().filter(|n| n.styles.as_slice().contains(&crate::look::NEED_CARD)).count(),
                count.min(3)
            );
            assert!(!nodes.iter().any(|n| n.styles.as_slice().contains(&crate::look::CHIP)));
            assert!(!nodes.iter().any(|n| n.role == Role::List));
            let heading = format!("Needs you · {count}");
            main_find(view.tree(), Role::Heading { level: 2 }, heading.as_bytes());
            assert_action(&mut view, Role::Link, b"All needs", inbox_act());
            if let Some(more) = more {
                assert_action(&mut view, Role::Link, more.as_bytes(), inbox_act());
            }
        }
    }
}
#[test]
fn nonquestion_need_keeps_its_kind_and_deciding_act() {
    for (object, kind, action) in [
        (c::ObjectKey::PersonTask(f::INBOX_TASK), "a person task", "Open"),
        (c::ObjectKey::Escalation(f::INBOX_TASK), "held", "Release"),
        (
            c::ObjectKey::Proposal(c::ProposalKey {
                proposer: c::Proposer::Task(f::INBOX_TASK),
                number: f::INBOX_QUESTION,
            }),
            "a proposal",
            "Accept",
        ),
    ] {
        let root = everyday(false, &[object], false, true);
        for view in [f::page_view(&root), f::browser_page_view(&root)] {
            contains(&f::main_text(view.tree()), kind);
            main_find(view.tree(), Role::Button, action.as_bytes());
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
        Some(NeedsFixture { objects: &[], cut: false, read: true, decided: true }),
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
fn sections_are_hidden_while_to_begin_shows() {
    for (objects, chats_count, need_line, chat_line) in [
        (&[][..], 0, "", "No chats yet."),
        (&[PARSED][..], 1, "Which license?", "README"),
        (&[PARSED, QUESTIONS[0], QUESTIONS[1]][..], 1, "Which license?", "README"),
    ] {
        for shown in [true, false] {
            let mut root = home_documents(
                forge_rows(true),
                false,
                false,
                Some(NeedsFixture { objects, cut: false, read: true, decided: true }),
                Some(chat_fixture(chats_count, 0, false, false)),
            )
            .0;
            if !shown {
                drive(&mut root, d::Event::Act { action: d::Action::Pages(d::pages::Act::SkipToBegin) });
            }
            for view in [f::page_view(&root), f::browser_page_view(&root)] {
                let styles = section_styles(view.tree());
                assert_eq!(
                    styles.contains(&crate::look::HOME_NEEDS),
                    !shown && !objects.is_empty(),
                    "needs: {objects:?}, shown={shown}"
                );
                assert_eq!(styles.contains(&crate::look::HOME_CHATS), !shown, "chats: {objects:?}, shown={shown}");
                let nodes = main_nodes(view.tree());
                if shown {
                    assert!(!nodes.iter().any(|node| node.role == Role::Heading { level: 2 }
                        && node.name.as_deref() == Some("Needs you · 1".as_bytes())));
                    assert!(
                        !nodes
                            .iter()
                            .any(|node| node.role == Role::Link && node.name.as_deref() == Some(b"All 1 chats"))
                    );
                    absent(&f::main_text(view.tree()), "Which license?");
                } else {
                    if !objects.is_empty() {
                        let needs = styled(view.tree(), crate::look::HOME_NEEDS);
                        contains(&f::nodes_text(subtree(view.tree(), needs.id)), need_line);
                    }
                    let chats = styled(view.tree(), crate::look::HOME_CHATS);
                    contains(&f::nodes_text(subtree(view.tree(), chats.id)), chat_line);
                    if chats_count == 0 {
                        assert!(!nodes.iter().any(|node| node.role == Role::Link
                            && node.name.as_deref().is_some_and(|name| name.starts_with(b"All "))));
                    } else {
                        main_find(view.tree(), Role::Link, b"All 1 chats");
                    }
                }
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
            b"Tell it something else",
            d::pages::Act::Needs(d::pages::needs::Act::Open { object: QUESTIONS[6] }),
        );
    }
}
#[test]
fn ask_line_reads_back_types_and_sends_on_both_homes() {
    for address in [Address::Home { project: f::PROJECT }, Address::AllHome] {
        for words in [b"".as_slice(), b"What landed?"] {
            let mut root = everyday(false, &[], false, true);
            if address == Address::AllHome {
                drive(
                    &mut root,
                    d::Event::Act { action: d::Action::Pages(d::pages::Act::Go { address: address.clone() }) },
                );
            }
            let mut view = f::browser_page_view(&root);
            let field = f::find(view.tree(), Role::TextField, b"Ask temper").unwrap();
            let action = crate::act(&mut view, field, Act::Input { value: words.into() }).unwrap();
            assert_eq!(action, d::Action::Pages(d::pages::Act::AskLine { words: words.into() }));
            drive(&mut root, d::Event::Act { action });
            let mut view = f::browser_page_view(&root);
            let field = f::find(view.tree(), Role::TextField, b"Ask temper").unwrap();
            assert_eq!(view.tree().find(field).unwrap().value.as_ref().unwrap().text.as_ref(), words);
            let ask = d::Action::Pages(d::pages::Act::AskWith { words: words.into() });
            assert_eq!(crate::act(&mut view, field, Act::Submit), Some(ask.clone()));
            assert_action(&mut view, Role::Button, b"Ask", d::pages::Act::AskWith { words: words.into() });
            assert!(f::find(f::page_view(&root).tree(), Role::TextField, b"Ask temper").is_none());
            let parts = children(main_nodes(view.tree()));
            assert!(parts.last().unwrap().styles.as_slice().contains(&crate::look::ASK_LINE));
        }
    }
    let root = everyday(true, &[], false, true);
    assert!(f::find(f::browser_page_view(&root).tree(), Role::TextField, b"Ask temper").is_none());
}

#[test]
fn home_notes_name_where_the_remaining_steps_stay() {
    for (configured, repository, note) in [
        (false, false, "Two steps left. Skip them if you like; they stay in How work runs and Ask temper."),
        (false, true, "One step left. Skip it if you like; it stays in Ask temper."),
        (true, false, "One step left. Skip it if you like; it stays in How work runs."),
    ] {
        let (root, _) = home_root(forge_rows(repository), configured, false);
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
        Some(NeedsFixture { objects: &[], cut: false, read: true, decided: true }),
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
            | d::Request::Screen { .. }
            | d::Request::KeepLook { .. }
            | d::Request::KeepSignIn { .. }
            | d::Request::Tab { .. }
            | d::Request::Copy { .. }
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
fn settings_ask_link_opens_the_delivered_asking_chat_in_both_shells() {
    let (mut root, me_watch, _) = f::root_at(Address::Settings { project: f::PROJECT }, &f::domain_limits());
    let template = f::chat::messaging_root();
    let me = f::asking_me(template.pages().me(template.client()).expect("me").clone());
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
    rows.push(row).expect("one asking chat");
    deliver(
        &mut root,
        live.expect("Settings watches chats"),
        c::Delivery::Tasks(Box::new(c::Tasks { rows, more: false })),
    );
    for mut view in [f::page_view(&root), f::browser_page_view(&root)] {
        let link = f::find(view.tree(), Role::Link, b"Ask temper").unwrap();
        assert_eq!(crate::href(&mut view, link).as_deref(), Some(b"/chats/9".as_slice()));
        assert_eq!(
            crate::act(&mut view, link, Act::Press),
            Some(d::Action::Pages(d::pages::Act::Go { address: Address::Chat { task } }))
        );
        let menu = f::find(view.tree(), Role::MenuItem, b"Ask temper").unwrap();
        assert_eq!(
            crate::act(&mut view, menu, Act::Press),
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
        shown: goals,
        goal_rows: None,
        away: None,
    };
    home_documents_with_status(
        forge_rows(true),
        !beginning,
        !beginning,
        Some(NeedsFixture { objects: &[], cut: false, read: true, decided: true }),
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
            let foot = home_foot(view.tree());
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
        let foot = home_foot(view.tree());
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
fn parsed_home_card_prints_choices_and_opens_needs_for_words() {
    use d::pages::needs::Act as N;
    let root = everyday(false, &[PARSED, QUESTIONS[0]], false, true);
    for mut view in [f::page_view(&root), f::browser_page_view(&root)] {
        main_find(view.tree(), Role::Paragraph, b"Which license?");
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
        assert_first_card_action(
            &mut view,
            b"Tell it something else",
            d::pages::Act::Needs(N::Open { object: PARSED }),
        );
        assert!(f::find(view.tree(), Role::Button, b"Later").is_none());
        assert!(f::find(view.tree(), Role::Disclosure, b"What the agent saw").is_none());
    }
    let print = printed(&f::page_view(&root));
    for words in
        ["Which license?", "MIT", "Apache", "Tell it something else", "Add a README · a question · under 1 min"]
    {
        contains(&print, words);
    }
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
        assert_eq!(parts[2].styles.as_slice(), &[crate::look::NEED_CARD]);
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
    let mut root = everyday(false, &[PARSED, QUESTIONS[0]], false, true);
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
fn later_home_card_keeps_its_card_and_meta() {
    use d::pages::needs::Act as N;
    let mut root = everyday(false, &[PARSED, QUESTIONS[0]], false, true);
    needs_act(&mut root, N::Later { object: PARSED });
    for view in [f::page_view(&root), f::browser_page_view(&root)] {
        let mut last = None;
        for node in main_nodes(view.tree()) {
            if node.styles.as_slice().contains(&crate::look::NEED_CARD) {
                last = Some(node);
            }
        }
        let card = last.unwrap();
        let text = f::nodes_text(subtree(view.tree(), card.id));
        contains(&text, "Which license?");
        contains(&text, "later");
    }
}
#[test]
fn phone_cards_keep_recommended_act_and_more() {
    for objects in [&[PARSED][..], &[PARSED, QUESTIONS[0]][..]] {
        let root = everyday(false, objects, false, true);
        let mut view = f::browser_page_view(&root);
        let card = styled(view.tree(), crate::look::NEED_CARD);
        let nodes = subtree(view.tree(), card.id);
        let mut primary = false;
        let mut more = false;
        for node in nodes {
            primary |= node.styles.as_slice().contains(&crate::look::RECOMMENDED);
            more |= node.styles.as_slice().contains(&crate::look::NEED_MORE);
        }
        assert!(primary);
        assert!(more);
        assert_first_card_action(
            &mut view,
            b"More",
            d::pages::Act::Needs(d::pages::needs::Act::Open { object: PARSED }),
        );
        absent(&f::main_text(view.tree()), "Answer, one at a time");
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
        let card = styled(view.tree(), crate::look::NEED_CARD);
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
    for view in [f::page_view(&root), f::browser_page_view(&root)] {
        main_find(view.tree(), Role::Paragraph, b"31 of 40 used.");
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
        shown: !beginning,
        goal_rows: Some(goals),
        away: None,
    };
    home_documents_with_status(
        forge_rows(true),
        !beginning,
        !beginning,
        Some(NeedsFixture {
            objects: &[c::ObjectKey::Question { task: 9, message: 1 }],
            cut: false,
            read: true,
            decided: true,
        }),
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
        main_find(view.tree(), Role::Button, b"Tell it something else");
        let row = main_find(view.tree(), Role::Link, b"README \xc2\xb7 needs you");
        assert_eq!(
            crate::act(&mut view, row, Act::Press),
            Some(d::Action::Pages(d::pages::Act::Go { address: Address::Task { task: 9 } }))
        );
        let goals = f::find(view.tree(), Role::Link, b"Work").unwrap();
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
#[expect(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::disallowed_macros,
    reason = "ordinary tests compare navigation subtrees in both shells"
)]
fn to_begin_cuts_the_rail_on_home_and_ask_alone() {
    for (address, skipped, cut) in [
        (Address::Home { project: f::PROJECT }, false, true),
        (Address::Ask { project: f::PROJECT }, false, true),
        (Address::Home { project: f::PROJECT }, true, false),
        (Address::Chats { filter: ChatsFilter::All }, false, false),
        (Address::Chat { task: 9 }, false, false),
    ] {
        let mut root = home_goals(1, 1, true);
        if skipped {
            drive(&mut root, d::Event::Act { action: d::Action::Pages(d::pages::Act::SkipToBegin) });
        }
        let mut out = drive(&mut root, d::Event::Went { place: d::Place::Address(address.clone()) });
        settle_home(&mut root, &mut out, None, Some(&chat_fixture(0, 0, false, false)), None);
        for mut view in [f::page_view(&root), f::browser_page_view(&root)] {
            if address == (Address::Home { project: f::PROJECT }) && !skipped {
                let styles = section_styles(view.tree());
                for style in [crate::look::HOME_NEEDS, crate::look::HOME_CHATS, crate::look::GOALS_SECTION] {
                    assert!(!styles.contains(&style));
                }
            }
            let nav = styled(view.tree(), crate::look::RAIL_NAV);
            let rows = children(subtree(view.tree(), nav.id));
            let entries: Vec<_> = rows.iter().map(|node| (node.role, node.name.as_deref())).collect();
            let mut expected = vec![(Role::Link, Some(b"Home".as_slice()))];
            if !cut {
                expected.extend([(Role::Link, Some(b"Needs you".as_slice())), (Role::Link, Some(b"Work".as_slice()))]);
            }
            expected.extend([
                (Role::Link, Some(b"Ask temper".as_slice())),
                (Role::Slot, None),
                (Role::Link, Some(b"How work runs".as_slice())),
            ]);
            assert_eq!(entries, expected, "{address:?}, skipped={skipped}");
            let separator = rows.get(rows.len().checked_sub(2).unwrap()).unwrap();
            assert_eq!(separator.styles.as_slice(), &[crate::look::RAIL_SEP]);
            let how = rows.last().unwrap().id;
            assert_eq!(
                view.tree().nodes().iter().any(|node| node.styles.as_slice().contains(&crate::look::RAIL_NEW)),
                !cut
            );
            let foot = styled(view.tree(), crate::look::RAIL_FOOT);
            assert!(
                subtree(view.tree(), foot.id)
                    .iter()
                    .any(|node| node.role == Role::Link && node.name.as_deref() == Some(b"Deployment"))
            );
            assert_eq!(
                crate::act(&mut view, how, Act::Press),
                Some(d::Action::Pages(d::pages::Act::Go { address: Address::Settings { project: f::PROJECT } }))
            );
        }
        let view = f::browser_page_view(&root);
        let tabs = styled(view.tree(), crate::look::TABBAR);
        assert_eq!(tabs.styles.as_slice().contains(&crate::look::TABBAR_BEGINNING), cut);
        let links = children(subtree(view.tree(), tabs.id));
        let entries: Vec<_> = links.iter().filter_map(|node| node.name.as_deref()).collect();
        assert_eq!(
            entries,
            if cut {
                vec![b"Home".as_slice(), b"Ask temper", b"Deployment"]
            } else {
                vec![b"Home".as_slice(), b"Needs", b"Work", b"Ask temper"]
            },
            "{address:?}"
        );
        if cut {
            assert_eq!(f::nodes_text(subtree(view.tree(), links[1].id)).as_ref(), b"Ask");
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
            "temper can change its code now, on your Forgejo; nothing goes back where it was copied from.",
        ),
        (
            2,
            "acme/widget, acme/other are in Shop",
            "temper can change their code now, on your Forgejo; nothing goes back where they were copied from.",
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
                let heading = skein_lib::bytes::concat(&[b"1. Done. ", title.as_bytes()]);
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
            b"Your Forgejo has none yet. Until one is in, temper talks and plans but changes no code.",
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
    state: d::pages::decided::DecisionTask,
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
                    reach: w::Reach::new(
                        &w::CEILINGS,
                        w::ReachParts {
                            kind: w::ReachKind::Wording,
                            class: w::ReachClass::Decide,
                            reason: w::ReachReason::Kind,
                            under: 0,
                            sets: 0,
                            matched: 0,
                            away: false,
                        },
                    )
                    .unwrap(),
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
                    landed: 1,
                    off: None,
                },
            )
            .unwrap(),
        ),
        rows,
        state: d::pages::decided::DecisionTask::Live,
    }
}
fn away_home(away: AwayFixture, beginning: bool, objects: &[c::ObjectKey], parts: w::ProjectStatusParts) -> d::Domain {
    away_home_with_needs(away, beginning, NeedsFixture { objects, cut: false, read: true, decided: true }, parts)
}
fn away_home_with_needs(
    away: AwayFixture,
    beginning: bool,
    needs: NeedsFixture<'_>,
    parts: w::ProjectStatusParts,
) -> d::Domain {
    let fixture = StatusFixture {
        document: w::ProjectStatus::new(&w::CEILINGS, parts).unwrap(),
        answered: true,
        goals: false,
        shown: false,
        goal_rows: None,
        away: Some(away),
    };
    home_documents_with_status(
        forge_rows(true),
        !beginning,
        !beginning,
        Some(needs),
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
fn the_everyday_kicker_with_rows_and_no_spell() {
    let mut away = away_fixture(1);
    away.spell = None;
    let root = default_away(away);
    for browser in [false, true] {
        assert_eq!(kicker(&root, browser).as_ref(), "Monday, 00:00 · you last looked 9 hours ago".as_bytes());
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
        main_find(
            status_view(&root, true).tree(),
            Role::Heading { level: 1 },
            if delivered {
                b"Four things were decided for you."
            } else {
                b"The last change landed 6 minutes ago. Nothing needs you."
            },
        );
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
fn spell_intro_without_decisions_uses_the_needs_sentence() {
    for (landed, objects, off, headline, kicker_words) in [
        (
            58,
            &QUESTIONS[..1],
            false,
            "57 changes landed. One thing needs you.",
            "While you were away · Sun 23:10 to now",
        ),
        (
            58,
            &QUESTIONS[..0],
            true,
            "57 changes landed. Nothing needs you.",
            "While you were away · Sun 23:10 to Sun 23:50",
        ),
        (1, &QUESTIONS[..1], false, "One thing needs you.", "While you were away · Sun 23:10 to now"),
        (1, &QUESTIONS[..0], true, "Nothing needs you.", "While you were away · Sun 23:10 to Sun 23:50"),
    ] {
        let mut away = away_fixture(0);
        if off {
            let mut spell = away.spell.take().unwrap().into_parts();
            spell.off = Some(STATUS_NOW - 10 * STATUS_MINUTE);
            away.spell = Some(w::AwayRow::new(&w::CEILINGS, spell).unwrap());
        }
        let mut parts = status_parts();
        parts.landed = landed;
        let root = away_home(away, false, objects, parts);
        for browser in [false, true] {
            let view = status_view(&root, browser);
            assert_eq!(kicker(&root, browser).as_ref(), kicker_words.as_bytes());
            main_find(view.tree(), Role::Heading { level: 1 }, headline.as_bytes());
            for node in main_nodes(view.tree()) {
                assert!(!node.styles.as_slice().contains(&crate::look::DECIDED));
            }
        }
    }
}
#[test]
fn needs_head_is_still_during_the_spell_with_or_without_decisions() {
    for rows in [0, 1] {
        for (spell, cut, heading) in [
            (true, false, "Still needs you · 2"),
            (true, true, "Still needs you · 2+"),
            (false, false, "Needs you · 2"),
            (false, true, "Needs you · 2+"),
        ] {
            let mut away = away_fixture(rows);
            if !spell {
                away.spell = None;
            }
            let root = away_home_with_needs(
                away,
                false,
                NeedsFixture { objects: &QUESTIONS[..2], cut, read: true, decided: true },
                status_parts(),
            );
            for browser in [false, true] {
                main_find(status_view(&root, browser).tree(), Role::Heading { level: 2 }, heading.as_bytes());
            }
        }
    }
}
#[test]
fn no_decided_region_with_no_row_or_to_begin() {
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
    for (kind, class, reason, away_on, allowed) in [
        (
            w::ReachKind::Tests,
            w::ReachClass::Decide,
            w::ReachReason::Kind,
            false,
            Some("Allowed: test-only changes are decided and listed."),
        ),
        (
            w::ReachKind::Retry,
            w::ReachClass::Ask,
            w::ReachReason::Kind,
            true,
            Some("Allowed: retries and splits are decided while you’re away."),
        ),
        (
            w::ReachKind::Implementation,
            w::ReachClass::Decide,
            w::ReachReason::Set(w::ReachSet::new(&w::CEILINGS, w::ReachSetParts { number: 1 }).unwrap()),
            false,
            Some("Allowed: changes to a path set are decided and listed."),
        ),
        (
            w::ReachKind::Tests,
            w::ReachClass::Decide,
            w::ReachReason::Set(w::ReachSet::new(&w::CEILINGS, w::ReachSetParts { number: 0 }).unwrap()),
            false,
            None,
        ),
        (w::ReachKind::Retry, w::ReachClass::Ask, w::ReachReason::Kind, false, None),
    ] {
        let mut away = away_fixture(1);
        let mut parts = away.rows.get(0).unwrap().clone().into_parts();
        let mut reach = parts.reach.into_parts();
        reach.kind = kind;
        reach.class = class;
        reach.reason = reason;
        reach.away = away_on;
        reach.under = 2;
        parts.reach = w::Reach::new(&w::CEILINGS, reach).unwrap();
        *away.rows.get_mut(0).unwrap() = w::Decision::new(&w::CEILINGS, parts).unwrap();
        let root = default_away(away);
        for browser in [false, true] {
            let view = status_view(&root, browser);
            let row = styled(view.tree(), crate::look::DECISION);
            let text = f::nodes_text(subtree(view.tree(), row.id));
            contains(&text, "welcome-stall · Sun 23:05");
            contains(&text, "Fix in the store or serve?");
            contains(&text, "In serve");
            contains(&text, "The store changes a record.");
            main_find(view.tree(), Role::Button, b"Overturn");
            let mut line = None;
            for node in main_nodes(view.tree()) {
                if node.styles.as_slice().contains(&crate::look::DECISION_ALLOWED) {
                    line = node.name.as_deref();
                }
            }
            assert_eq!(
                line,
                match allowed {
                    Some(allowed) => Some(allowed.as_bytes()),
                    None => None,
                }
            );
            if let Some(allowed) = allowed {
                let took = skein_lib::bytes::find(&text, b"In serve").unwrap();
                let allowed_at = skein_lib::bytes::find(&text, allowed.as_bytes()).unwrap();
                let why = skein_lib::bytes::find(&text, b"The store changes a record.").unwrap();
                assert!(took < allowed_at && allowed_at < why);
            }
            if !browser {
                let printed = printed(&view);
                contains(&printed, "welcome-stall · Sun 23:05 Fix in the store or serve? In serve");
                if let Some(allowed) = allowed {
                    contains(&printed, allowed);
                }
                contains(&printed, "The store changes a record.");
            }
        }
    }
}
#[test]
fn an_unread_task_says_task_n() {
    let mut away = away_fixture(1);
    away.state = d::pages::decided::DecisionTask::Unknown;
    let root = default_away(away);
    contains(&f::main_text(status_view(&root, true).tree()), "Task 99 · Sun 23:05");
}
fn assert_decision_end(state: d::pages::decided::DecisionTask, overturned: bool, late: &str) {
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
        d::pages::decided::DecisionTask::Done(skein_lib::Wall::from_nanos(STATUS_NOW)),
        false,
        "landed 00:00 · a new goal can undo it",
    );
}
#[test]
fn ended_says_ended() {
    assert_decision_end(
        d::pages::decided::DecisionTask::Ended(skein_lib::Wall::from_nanos(STATUS_NOW)),
        false,
        "ended 00:00 · a new goal can undo it",
    );
}
#[test]
fn overturned_says_it_asks_again_and_strikes_took() {
    assert_decision_end(d::pages::decided::DecisionTask::Live, true, "Overturned: it asks you again");
}
#[test]
fn ended_and_overturned_says_landed_and_strikes_took() {
    assert_decision_end(
        d::pages::decided::DecisionTask::Done(skein_lib::Wall::from_nanos(STATUS_NOW)),
        true,
        "landed 00:00 · a new goal can undo it",
    );
}
#[test]
fn away_needs_head_is_still_only_while_needs_show() {
    for objects in [&[][..], &QUESTIONS[..2]] {
        let root = away_home(away_fixture(1), false, objects, status_parts());
        let heading = "Still needs you · 2";
        for browser in [false, true] {
            let view = status_view(&root, browser);
            assert_eq!(
                f::find(view.tree(), Role::Heading { level: 2 }, heading.as_bytes()).is_some(),
                !objects.is_empty()
            );
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
        d::pages::decided::DecisionTask::Unknown,
        d::pages::decided::DecisionTask::Done(skein_lib::Wall::EPOCH),
        d::pages::decided::DecisionTask::Ended(skein_lib::Wall::EPOCH),
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

fn unanswered_home() -> f::ReadPage {
    let mut page = f::ReadPage::new(Address::Home { project: f::PROJECT });
    page.drop_inbox();
    page
}
fn answered_home(closed: &[jig_client::contract::TaskLine]) -> f::ReadPage {
    let mut page = f::ReadPage::new(Address::Home { project: f::PROJECT });
    page.live_chats(&[]);
    page.closed(closed);
    page.refresh_inbox();
    page.sections(false);
    page
}
fn home_empty(root: &d::Domain, expected: bool) {
    for view in [f::page_view(root), f::browser_page_view(root)] {
        let main = f::main_text(view.tree());
        assert_eq!(
            skein_lib::bytes::find(&main, b"No chats yet.").is_some(),
            expected,
            "{main:?} facts={:?}",
            d::pages::home::facts(root, f::PROJECT, f::NOW)
        );
        assert!(
            f::find(view.tree(), Role::Heading { level: 1 }, if expected { b"Nothing needs you." } else { b"Shop" })
                .is_some()
        );
    }
}
#[test]
fn behind_with_unanswered_reads_home_claims_no_empty() {
    let page = unanswered_home();
    home_empty(&page.root, false);
    for view in [f::page_view(&page.root), f::browser_page_view(&page.root)] {
        assert!(f::find(view.tree(), Role::Status, b"Behind").is_some());
        assert!(f::find(view.tree(), Role::Region, b"Needs you").is_none());
        assert!(f::find(view.tree(), Role::Heading { level: 2 }, b"Chats").is_some());
    }
}
#[test]
fn answered_empty_reads_still_say_so_on_home() {
    let mut page = answered_home(&[]);
    page.drop_inbox();
    assert!(f::find(f::page_view(&page.root).tree(), Role::Status, b"Behind").is_some());
    home_empty(&page.root, true);
}
#[test]
fn home_keeps_what_it_knew_through_a_dropped_link_and_a_lost_read() {
    let mut ended = super::chats::line();
    ended.ended_at = Some(f::NOW);
    let mut page = answered_home(core::slice::from_ref(&ended));
    page.up(jig_client::Up::WatchEnded { watch: page.chats.unwrap(), end: jig_client::contract::WatchEnd::Dropped });
    page.collect();
    let read = page.reads.last().unwrap().0;
    page.answer(read, jig_client::contract::ReadResult::None(jig_client::contract::ReadNone::Unreachable));
    for view in [f::page_view(&page.root), f::browser_page_view(&page.root)] {
        assert!(skein_lib::bytes::find(&f::main_text(view.tree()), ended.chip.title.bytes()).is_some());
    }
    let mut page = answered_home(&[]);
    home_empty(&page.root, true);
    page.up(jig_client::Up::WatchEnded { watch: page.chats.unwrap(), end: jig_client::contract::WatchEnd::Dropped });
    page.collect();
    page.drop_inbox();
    assert!(!page.root.pages().inbox().unwrap().delivered(page.root.client()));
    home_empty(&page.root, true);
}
