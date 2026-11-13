#![allow(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    reason = "test assertions inspect semantic trees and their bound controls"
)]
pub(super) use super::fixtures::welcome::{model_address, model_with_forge, owned_adopted_rows};
use super::{connect_forge as forge, fixtures as f};
use skein_ui::{Act, Role, State};
use temper_client_domain::{self as d, Address, ChatsFilter, Step, pages};
use temper_client_protocol_wire::v1 as w;

const HERE: u16 = 3;
fn waiting_for_deployment(step: Step) -> (d::Domain, skein_lib::Token) {
    let limits = f::limits();
    let (frame, _, _) = f::root_at(Address::Chats { filter: ChatsFilter::All }, &limits);
    let me = frame.pages().me(frame.client()).unwrap().clone();
    let mut root = d::unlinked(&limits, f::SEED);
    let mut out = skein_lib::Queue::uncapped();
    for event in [
        d::Event::Client { event: jig_client::Event::StartUnlinked { kept: skein_lib::List::with_capacity(0) } },
        d::Event::Start {
            address: Address::Welcome { step },
            kept: skein_lib::List::with_capacity(0),
            with: Some(jig_client::With::Transport),
        },
        d::Event::Client {
            event: jig_client::Event::Up { up: jig_client::Up::Served { served: true, versions: None } },
        },
    ] {
        d::step(&mut root, &skein_lib::Env { now: skein_lib::Time::ZERO, wall: f::NOW, limits }, event, &mut out);
    }
    let mut watch = None;
    for request in &out {
        if let d::Request::Unlinked {
            request:
                jig_client::UnlinkedRequest::Down {
                    down:
                        jig_client::Down::Open {
                            watch: token,
                            subject: jig_client::Watchable::Jig(jig_client::contract::Subject::Me),
                        },
                },
        } = request
        {
            watch = Some(*token);
        }
    }
    let watch = watch.unwrap();
    d::step(
        &mut root,
        &skein_lib::Env { now: skein_lib::Time::ZERO, wall: f::NOW, limits },
        d::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched {
                    watch,
                    event: jig_client::contract::WatchEvent::Delivered(jig_client::contract::Delivery::Me(Box::new(
                        me,
                    ))),
                },
            },
        },
        &mut out,
    );
    while let Some(request) = out.pop() {
        if let d::Request::Unlinked {
            request:
                jig_client::UnlinkedRequest::Down {
                    down: jig_client::Down::Read { read, what: jig_client::Readable::Family { record, .. } },
                },
        } = request
            && temper_client_protocol_wire::decode_reading(&record.bytes, &w::CEILINGS).unwrap().read()
                == &w::Read::Deployment
        {
            return (root, read);
        }
    }
    panic!("live link reads deployment");
}
fn temper_and_end(root: &d::Domain) {
    for view in [f::page_view(root), f::browser_page_view(root)] {
        assert_eq!(f::text(view.tree()).as_ref(), b"temperLive");
        super::frame::assert_top_end(&view, &[(Role::Region, None)]);
        let main = view.tree().nodes().iter().find(|node| node.role == Role::Main).unwrap();
        assert_eq!(main.size, 1, "the main has no children");
        assert!(f::find(view.tree(), Role::Link, b"temper").is_some());
        assert!(!view.tree().nodes().iter().any(|node| node.role == Role::Heading { level: 1 }));
    }
}
#[test]
fn before_the_read_the_frame_draws_temper_and_the_end() {
    for step in [Step::Welcome, Step::WelcomeAsked, Step::Forge { forge: None }, Step::Model { way: None }, Step::Ready]
    {
        let (root, _) = waiting_for_deployment(step);
        temper_and_end(&root);
    }
}
#[test]
fn no_welcome_draws_temper_and_the_end() {
    for step in [Step::Welcome, Step::WelcomeAsked, Step::Forge { forge: None }, Step::Model { way: None }, Step::Ready]
    {
        let (mut root, read) = waiting_for_deployment(step);
        f::deliver_deployment(&mut root, read, false, false);
        temper_and_end(&root);
    }
}
#[test]
fn an_answer_showing_the_welcome_draws_its_frame_and_heading() {
    let (mut root, read) = waiting_for_deployment(Step::Welcome);
    f::deliver_deployment(&mut root, read, true, true);
    for view in [f::page_view(&root), f::browser_page_view(&root)] {
        node(view.tree(), Role::Heading { level: 1 }, "Welcome to temper");
        node(view.tree(), Role::List, "Steps");
        assert!(!String::from_utf8_lossy(&f::text(view.tree())).contains("temper box"));
    }
}
fn choice_cards(tree: &skein_ui::Tree) -> Vec<&[u8]> {
    let choices = tree.nodes().iter().find(|n| n.styles.as_slice().contains(&crate::look::FLOW_CHOICES)).unwrap();
    tree.nodes()
        .iter()
        .filter(|n| forge::parent(tree, n.id) == Some(choices.id))
        .map(|n| n.name.as_deref().unwrap())
        .collect()
}

#[test]
fn the_forge_step_shows_choices_after_reset() {
    let root = f::forge_root(Address::Welcome { step: Step::Forge { forge: None } }, skein_lib::List::with_capacity(0));
    assert!(root.forge().playground().is_none());
    assert_eq!(d::address::print(root.pages().address()).as_ref(), b"/welcome?at=forge");
    for view in [f::page_view(&root), f::browser_page_view(&root)] {
        assert_eq!(choice_cards(view.tree()), [b"Forgejo".as_slice(), b"GitHub"]);
        let github = node(view.tree(), Role::Button, "Connect GitHub");
        let run = node(view.tree(), Role::Link, "Run on this machine");
        assert!(styled(view.tree(), run, crate::look::FLOW_PRIMARY));
        assert!(!styled(view.tree(), github, crate::look::FLOW_PRIMARY));
        node(view.tree(), Role::Paragraph, FORGEJO_CARD);
        node(view.tree(), Role::Paragraph, "In this tab, on GitHub · about two minutes");
        assert!(f::find(view.tree(), Role::Link, b"Or connect your deployment").is_none());
        assert!(f::find(view.tree(), Role::Dialog, b"Forgejo on this machine").is_none());
    }
    assert_eq!(press(&root, Role::Link, "Run on this machine"), d::Action::Pages(pages::Act::Go { address: SHEET }));
}

const FORGEJO_CARD: &str = "A forge of your own: temper starts it on this machine and manages it. \
    You can import existing repositories into it, from GitHub or elsewhere.";
const SHEET: Address = Address::Welcome { step: Step::Forge { forge: Some(d::Choice::Owned) } };
fn sheet_root() -> d::Domain {
    f::forge_root(SHEET, skein_lib::List::with_capacity(0))
}
fn sheet(tree: &skein_ui::Tree) -> &[skein_ui::Node] {
    let dialog = tree.nodes().iter().position(|n| n.role == Role::Dialog).expect("the sheet");
    let size = usize::try_from(tree.nodes()[dialog].size).unwrap();
    &tree.nodes()[dialog..dialog.checked_add(size).unwrap()]
}
fn named(nodes: &[skein_ui::Node], role: Role) -> Vec<&[u8]> {
    nodes.iter().filter(|n| n.role == role).map(|n| n.name.as_deref().unwrap()).collect()
}
fn start(root: &mut d::Domain, start: d::forge::forges::Start) {
    f::root_event(
        root,
        d::Event::Act { action: d::Action::Forge(d::forge::Act::Forges(d::forge::forges::Act::Start(start))) },
    );
}

#[test]
fn the_sheet_opens_over_the_forge_choices() {
    let root = sheet_root();
    assert_eq!(d::address::print(root.pages().address()).as_ref(), b"/welcome?at=forge&forge=owned");
    for view in [f::page_view(&root), f::browser_page_view(&root)] {
        assert_eq!(choice_cards(view.tree()), [b"Forgejo".as_slice(), b"GitHub"]);
        assert_eq!(skein_ui::outline::check(view.tree()), None);
        let nodes = sheet(view.tree());
        assert!(nodes[0].styles.as_slice().contains(&crate::look::WELCOME_SHEET));
        assert_eq!(nodes[0].name.as_deref(), Some(b"Forgejo on this machine".as_slice()));
        assert_eq!(named(nodes, Role::Heading { level: 2 }), [b"Forgejo on this machine".as_slice()]);
        assert_eq!(named(nodes, Role::Heading { level: 3 }), [b"Start it with".as_slice()]);
        assert_eq!(
            named(nodes, Role::Paragraph),
            [
                b"temper starts it here and connects to it; you connect your model meanwhile.".as_slice(),
                b"A README and a starter CI check.",
                b"Copied into your Forgejo from GitHub or another git host, to play around with. Nothing is sent back.",
                "Your first project opens with no repository. Make or copy one later from temper’s UI, or adopt one you push yourself."
                    .as_bytes(),
            ]
        );
        assert_eq!(
            named(nodes, Role::Choice),
            [b"An empty repository".as_slice(), b"Copies of public repositories", b"Nothing for now"]
        );
        let chosen: Vec<_> = nodes
            .iter()
            .filter(|n| n.role == Role::Choice && n.states.as_slice().contains(&State::Selected))
            .map(|n| n.name.as_deref().unwrap())
            .collect();
        assert_eq!(chosen, [b"An empty repository".as_slice()]);
        assert_eq!(named(nodes, Role::TextField), [b"Organization".as_slice(), b"Name"]);
        let values: Vec<_> = nodes
            .iter()
            .filter(|n| n.role == Role::TextField)
            .map(|n| n.value.as_ref().unwrap().text.as_ref())
            .collect();
        assert_eq!(values, [b"work".as_slice(), b"hello"]);
        assert_eq!(named(nodes, Role::TextArea).len(), 0);
        assert_eq!(named(nodes, Role::Alert).len(), 0);
        assert_eq!(named(nodes, Role::Button), [b"Cancel".as_slice(), b"Run on this machine"]);
        let cancel = nodes.iter().find(|n| n.role == Role::Button && n.name.as_deref() == Some(b"Cancel")).unwrap();
        assert_eq!(cancel.shortcut, Some(skein_ui::Shortcut::ESCAPE));
        let run = nodes.iter().find(|n| n.name.as_deref() == Some(b"Run on this machine")).unwrap();
        assert!(run.styles.as_slice().contains(&crate::look::FLOW_PRIMARY));
    }
    assert_eq!(
        press(&root, Role::Button, "Cancel"),
        d::Action::Pages(pages::Act::Go { address: Address::Welcome { step: Step::Forge { forge: None } } })
    );
    assert_eq!(
        press(&root, Role::Button, "Run on this machine"),
        d::Action::Forge(d::forge::Act::Forges(d::forge::forges::Act::RunOwned))
    );
    let mut pending = sheet_root();
    f::root_event(
        &mut pending,
        d::Event::Act { action: d::Action::Forge(d::forge::Act::Forges(d::forge::forges::Act::RunOwned)) },
    );
    assert!(pending.forge().set_up().is_some_and(|last| last.progress.is_some()));
    let view = f::page_view(&pending);
    let run = sheet(view.tree()).iter().find(|n| n.name.as_deref() == Some(b"Run on this machine")).unwrap();
    assert!(run.states.as_slice().contains(&State::Disabled), "a pending set up disables Run");
    let mut view = f::page_view(&root);
    let copies = node(view.tree(), Role::Choice, "Copies of public repositories");
    assert_eq!(
        crate::act(&mut view, copies, Act::Toggle),
        Some(d::Action::Forge(d::forge::Act::Forges(d::forge::forges::Act::Start(d::forge::forges::Start::Copies))))
    );
}

#[test]
fn the_sheet_shows_a_new_forges_refusal_above_its_foot() {
    let mut root = sheet_root();
    let token = forge::attempt(forge::act(&mut root, d::forge::Act::Forges(d::forge::forges::Act::RunOwned)));
    forge::answer(
        &mut root,
        token,
        w::Outcome::SetUpRefused(
            w::SetUpRefused::new(&w::CEILINGS, w::SetUpRefusedParts { why: w::SetUpRefusal::Busy }).unwrap(),
        ),
    );
    let view = f::page_view(&root);
    let nodes = sheet(view.tree());
    let refusal = d::forge::words::set_up_refusal(d::forge::forges::SetUpRefusal::Busy);
    assert_eq!(named(nodes, Role::Alert), [refusal]);
    let alert = nodes.iter().position(|n| n.role == Role::Alert).unwrap();
    let foot = nodes.iter().position(|n| n.styles.as_slice().contains(&crate::look::WELCOME_SHEET_FOOT)).unwrap();
    assert!(alert < foot);
    let run = nodes.iter().find(|n| n.name.as_deref() == Some(b"Run on this machine")).unwrap();
    assert!(!run.states.as_slice().contains(&State::Disabled), "a refused set up may run again");
}
#[test]
fn the_sheets_copies_take_addresses_and_are_not_made_yet() {
    let mut root = sheet_root();
    start(&mut root, d::forge::forges::Start::Copies);
    let mut view = f::page_view(&root);
    let nodes = sheet(view.tree());
    assert_eq!(named(nodes, Role::TextField).len(), 0);
    assert_eq!(named(nodes, Role::TextArea), [b"Their addresses, one per line".as_slice()]);
    let area = nodes.iter().find(|n| n.role == Role::TextArea).unwrap();
    assert_eq!(area.placeholder.as_deref(), Some(b"github.com/owner/name".as_slice()));
    let area = area.id;
    assert_eq!(
        crate::act(&mut view, area, Act::Input { value: Box::from(b"github.com/a/b".as_slice()) }),
        Some(d::Action::Forge(d::forge::Act::CopiesWrite { text: Box::from(b"github.com/a/b".as_slice()) }))
    );
    f::root_event(
        &mut root,
        d::Event::Act {
            action: d::Action::Forge(d::forge::Act::CopiesWrite { text: Box::from(b"github.com/a/b".as_slice()) }),
        },
    );
    f::root_event(
        &mut root,
        d::Event::Act { action: d::Action::Forge(d::forge::Act::Forges(d::forge::forges::Act::RunOwned)) },
    );
    let view = f::page_view(&root);
    let nodes = sheet(view.tree());
    assert_eq!(
        nodes.iter().find(|n| n.role == Role::TextArea).unwrap().value.as_ref().unwrap().text.as_ref(),
        b"github.com/a/b"
    );
    assert_eq!(named(nodes, Role::Alert).len(), 0);
    start(&mut root, d::forge::forges::Start::Nothing);
    let view = f::page_view(&root);
    let nodes = sheet(view.tree());
    assert_eq!(named(nodes, Role::Alert).len(), 0);
    assert_eq!(named(nodes, Role::TextArea).len(), 0);
    assert_eq!(named(nodes, Role::TextField).len(), 0);
}

fn model_view(rows: skein_lib::List<w::ForgeRow>) -> crate::View {
    f::page_view(&f::forge_root(Address::Welcome { step: Step::Model { way: None } }, rows))
}
fn forge_region(tree: &skein_ui::Tree) -> &[skein_ui::Node] {
    let Some(start) = tree
        .nodes()
        .iter()
        .position(|n| n.role == Role::Complementary && n.name.as_deref() == Some(b"Forge and model"))
    else {
        return &[];
    };
    &tree.nodes()[start..start.checked_add(usize::try_from(tree.nodes()[start].size).unwrap()).unwrap()]
}
fn pane_lines(tree: &skein_ui::Tree) -> Vec<&skein_ui::Node> {
    let pane = node(tree, Role::Complementary, "Forge and model");
    forge_region(tree)
        .iter()
        .filter(|n| {
            (n.role == Role::Paragraph || n.styles.as_slice().contains(&crate::look::ACCOUNT_LINE))
                && forge::parent(tree, n.id) == Some(pane)
        })
        .collect()
}
fn model_pane(tree: &skein_ui::Tree, expected: &str) {
    let pane = node(tree, Role::Complementary, "Forge and model");
    let main = tree.nodes().iter().find(|n| n.role == Role::Main).unwrap().id;
    assert_eq!(forge::parent(tree, pane), Some(main));
    assert!(styled(tree, pane, crate::look::WELCOME_STATUS));
    let lines = pane_lines(tree);
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[1].name.as_deref(), Some(expected.as_bytes()));
    assert!(f::find(tree, Role::Link, b"Connect a model").is_none());
    assert!(!forge_region(tree).iter().any(|n| n.styles.as_slice().contains(&crate::look::WELCOME_END)));
}
fn lines_in_pane(tree: &skein_ui::Tree, expected: &str) -> skein_ui::NodeId {
    let lines = pane_lines(tree);
    let [forge, model] = lines.as_slice() else { panic!("pane has Forge and Model lines") };
    assert_eq!(forge.name.as_deref(), Some(expected.as_bytes()));
    assert_eq!(model.name.as_deref(), Some(b"Model none yet".as_slice()));
    assert!(f::find(tree, Role::Link, b"Connect a model").is_none());
    forge.id
}
fn each_step(tree: &skein_ui::Tree, shown: bool) {
    assert_eq!(f::find(tree, Role::Disclosure, b"Each step").is_some(), shown);
}

#[test]
fn model_without_a_persons_forge_keeps_both_none_yet_lines() {
    let root = f::forge_root(Address::Welcome { step: Step::Model { way: None } }, skein_lib::List::with_capacity(0));
    for view in [f::page_view(&root), f::browser_page_view(&root)] {
        lines_in_pane(view.tree(), "Forge none yet");
        node(view.tree(), Role::Link, "Connect a forge");
        each_step(view.tree(), false);
        assert!(terminal_print(&view).contains("Forge none yet · "));
    }
}

#[test]
fn model_names_the_first_repository_while_forgejo_sets_up() {
    let (mut root, watch) = f::forge_root_watch(
        Address::Welcome { step: Step::Forge { forge: None } },
        forge::rows(w::Connection::NotSetUp, w::Offer::Offered),
    );
    f::root_event(&mut root, d::Event::Act { action: d::Action::Forge(d::forge::Act::SetUpAtOnce { forge: HERE }) });
    // The setup request supplies the names until the watch's row carries them.
    forge::deliver(
        &mut root,
        watch,
        forge::rows(
            w::Connection::SettingUp(
                w::SettingUp::new(&w::CEILINGS, w::SettingUpParts { step: None, fetched: None }).unwrap(),
            ),
            w::Offer::Offered,
        ),
    );
    f::root_event(
        &mut root,
        d::Event::Went { place: d::Place::Address(Address::Welcome { step: Step::Model { way: None } }) },
    );
    let view = f::page_view(&root);
    lines_in_pane(view.tree(), "Forgejo on this machine · work/hello · Setting up");
    each_step(view.tree(), true);
}

#[test]
fn model_ticks_the_first_repository_made() {
    let view = f::browser_page_view(&f::forge_root(
        Address::Welcome { step: Step::Model { way: None } },
        forge::first_rows(forge::connected(), forge::made()),
    ));
    let line = lines_in_pane(view.tree(), "Forgejo on this machine · connected as temper · work/hello made");
    let line = view.tree().nodes().iter().find(|n| n.id == line).unwrap();
    assert!(line.styles.as_slice().contains(&crate::look::FLOW_DONE));
    let index = view.tree().nodes().iter().position(|n| n.id == line.id).unwrap();
    let runs: Vec<_> = view.tree().nodes()[index + 1..]
        .iter()
        .take(3)
        .flat_map(|n| n.text.iter().map(|r| (r.run, r.text.as_ref())))
        .collect();
    assert_eq!(
        runs,
        [
            (skein_ui::Run::Strong, b"Forgejo on this machine".as_slice()),
            (skein_ui::Run::Plain, b"connected as temper".as_slice()),
            (skein_ui::Run::Plain, b"work/hello made".as_slice())
        ]
    );
    each_step(view.tree(), false);
}

#[test]
fn model_ticks_a_connected_github_without_a_repository() {
    let view = model_view(forge::github_rows(true));
    lines_in_pane(view.tree(), "GitHub · Connected as my-app[bot] (9)");
    each_step(view.tree(), false);
}

#[test]
fn an_installed_github_without_a_repository_offers_continue() {
    let root = f::forge_root(Address::Welcome { step: Step::Forge { forge: None } }, forge::github_rows(true));
    for view in [f::page_view(&root), f::browser_page_view(&root)] {
        node(view.tree(), Role::Link, "Continue");
        for name in [b"Go on".as_slice(), b"Start over"] {
            assert!(f::find(view.tree(), Role::Button, name).is_none());
        }
    }
}
#[test]
fn model_keeps_github_connecting_until_an_installation_is_accepted() {
    let view = model_view(forge::github_rows(false));
    lines_in_pane(view.tree(), "GitHub · Connecting: no installation accepted yet");
    each_step(view.tree(), true);
}

#[test]
fn model_says_which_setup_failed_and_keeps_its_steps() {
    let view = model_view(forge::rows(
        w::Connection::SetUpFailed(
            w::SetUpFailed::new(&w::CEILINGS, w::SetUpFailedParts { why: b"fetch failed".as_slice().into(), at: None })
                .unwrap(),
        ),
        w::Offer::Offered,
    ));
    let line = lines_in_pane(view.tree(), "Forgejo on this machine · Set up failed: fetch failed");
    let line = view.tree().nodes().iter().find(|n| n.id == line).unwrap();
    assert!(!line.styles.as_slice().contains(&crate::look::FLOW_DONE));
    each_step(view.tree(), true);
}

#[test]
fn model_names_the_failed_first_repository_step() {
    for (step, connection, failed) in [
        (
            w::CreateStep::Admin,
            "Forgejo on this machine · work/hello · Set up failed: checking the admin's token",
            "Making work/hello failed: checking the admin's token",
        ),
        (
            w::CreateStep::Org,
            "Forgejo on this machine · work/hello · Set up failed: making the organization",
            "Making work/hello failed: making the organization",
        ),
        (
            w::CreateStep::Repository,
            "Forgejo on this machine · work/hello · Set up failed: making the repository",
            "Making work/hello failed: making the repository",
        ),
        (
            w::CreateStep::File,
            "Forgejo on this machine · work/hello · Set up failed: adding the starter workflow",
            "Making work/hello failed: adding the starter workflow",
        ),
    ] {
        let view = model_view(forge::first_rows(
            w::Connection::Connecting,
            w::FirstState::Failed(w::CreateFailed::new(&w::CEILINGS, w::CreateFailedParts { step }).unwrap()),
        ));
        lines_in_pane(view.tree(), connection);
        each_step(view.tree(), true);
        forge::words_in_order(view.tree(), &["failed", failed]);
        node(view.tree(), Role::Status, "failed");
    }
}

#[test]
fn model_keeps_steps_while_the_first_repository_is_making() {
    let view = model_view(forge::first_rows(w::Connection::Connecting, w::FirstState::Making));
    lines_in_pane(view.tree(), "Forgejo on this machine · work/hello · Connecting");
    each_step(view.tree(), true);
    node(view.tree(), Role::Status, "under way");
}

fn node(tree: &skein_ui::Tree, role: Role, name: &str) -> skein_ui::NodeId {
    f::find(tree, role, name.as_bytes()).unwrap_or_else(|| panic!("missing {role:?}: {name}"))
}
fn styled(tree: &skein_ui::Tree, id: skein_ui::NodeId, style: skein_ui::Style) -> bool {
    tree.nodes().iter().find(|n| n.id == id).unwrap().styles.as_slice().contains(&style)
}
fn press(root: &d::Domain, role: Role, name: &str) -> d::Action {
    let mut view = f::page_view(root);
    let id = node(view.tree(), role, name);
    crate::act(&mut view, id, Act::Press).expect("bound control")
}
#[test]
fn the_welcome_frame_is_the_brand_the_steps_and_the_link_alone() {
    let root = f::welcome_root(Step::Welcome, true);
    let mut view = f::page_view(&root);
    let brand = f::find(view.tree(), Role::Link, b"temper").unwrap();
    assert_eq!(crate::href(&mut view, brand).as_deref(), Some(b"/".as_slice()));
    let tree = view.tree();
    assert!(
        tree.nodes()
            .iter()
            .find(|n| n.role == Role::Heading { level: 1 })
            .unwrap()
            .styles
            .as_slice()
            .contains(&crate::look::WELCOME_INTRO)
    );
    super::frame::assert_top_end(&view, &[(Role::Region, None)]);
    let banner = tree.nodes().iter().position(|n| n.role == Role::Banner).unwrap();
    assert_eq!(tree.nodes()[banner].styles.as_slice(), [crate::look::TOP]);
    let brand = &tree.nodes()[banner + 1];
    assert_eq!(brand.name.as_deref(), Some(b"temper".as_slice()));
    let banner = &tree.nodes()[banner..banner + usize::try_from(tree.nodes()[banner].size).unwrap()];
    let mut child = 1;
    for (role, style) in
        [(Role::Link, crate::look::BRAND), (Role::List, crate::look::FLOW_STEPS), (Role::Slot, crate::look::TOP_END)]
    {
        assert_eq!(banner[child].role, role);
        assert!(banner[child].styles.as_slice().contains(&style));
        child += usize::try_from(banner[child].size).unwrap();
    }
    assert_eq!(child, banner.len(), "the end finishes the banner");
    assert!(banner.iter().any(|n| n.role == Role::List && n.name.as_deref() == Some(b"Steps".as_slice())));
    for role in [Role::Disclosure, Role::Navigation] {
        assert!(!tree.nodes().iter().any(|node| node.role == role));
    }
    assert!(f::find(tree, Role::Region, b"Playground").is_none());
    assert!(!String::from_utf8_lossy(&f::text(tree)).contains("No model yet"));
    assert_eq!(tree.nodes().iter().filter(|node| node.role == Role::Region).count(), 3);
}
#[test]
fn every_welcome_step_keeps_the_person_and_sign_out_alert_out_of_the_bar() {
    for step in [Step::Welcome, Step::WelcomeAsked, Step::Forge { forge: None }, Step::Model { way: None }, Step::Ready]
    {
        let mut root = f::welcome_root(step, true);
        f::root_event(&mut root, d::Event::Client { event: jig_client::Event::SignOutLost });
        for view in [f::page_view(&root), f::browser_page_view(&root)] {
            super::frame::assert_top_end(&view, &[(Role::Region, None)]);
            assert!(f::find(view.tree(), Role::Button, b"Sign out").is_none());
            assert!(f::find(view.tree(), Role::Alert, b"Sign out did not reach the engine.").is_none());
            let mut print = skein_ui_text::Print::new(&view.limits().tree).unwrap();
            skein_ui_text::render(view.tree(), b"temper", None, &mut print);
            assert!(!String::from_utf8_lossy(print.text.as_slice()).contains("Sign out"));
        }
    }
}
#[test]
fn the_box_paragraph_follows_the_cards_note_only_on_the_welcome_step() {
    for offered in [true, false] {
        let root = if offered {
            f::welcome_root(Step::WelcomeAsked, true)
        } else {
            f::forge_root_accounts(
                Address::Welcome { step: Step::WelcomeAsked },
                forge::rows(w::Connection::Connecting, w::Offer::Offered),
            )
            .0
        };
        for view in [f::page_view(&root), f::browser_page_view(&root)] {
            let tree = view.tree();
            let line = tree.nodes().iter().find(|n| styled(tree, n.id, crate::look::WELCOME_BOX_NOTE)).unwrap();
            assert_eq!(line.role, Role::Paragraph);
            let main = tree.nodes().iter().find(|n| n.role == Role::Main).unwrap();
            assert_eq!(forge::parent(tree, line.id), Some(main.id));
            let before = if offered { crate::look::WELCOME_AFTER } else { crate::look::WELCOME_CHOICES };
            let previous = tree.nodes().iter().find(|n| styled(tree, n.id, before)).unwrap();
            let start = tree.nodes().iter().position(|n| n.id == previous.id).unwrap();
            assert_eq!(tree.nodes()[start + usize::try_from(previous.size).unwrap()].id, line.id);
            let start = tree.nodes().iter().position(|n| n.id == line.id).unwrap();
            let runs: Vec<_> = tree.nodes()[start + 1].text.iter().map(|r| (r.run, r.text.as_ref())).collect();
            assert_eq!(
                runs,
                [
                    (skein_ui::Run::Strong, b"temper box".as_slice()),
                    (skein_ui::Run::Plain, " · on this machine, to try it · more later, from Settings · ".as_bytes()),
                ]
            );
            let server = &tree.nodes()[start + 2];
            assert_eq!(server.name.as_deref(), Some("Run temper on a server ↗".as_bytes()));
            assert!(server.external.as_deref().is_some_and(|a| a.ends_with(b"/docs/help/server.md")));
            let mut print = skein_ui_text::Print::new(&view.limits().tree).unwrap();
            skein_ui_text::render(tree, b"temper", None, &mut print);
            let printed = String::from_utf8_lossy(print.text.as_slice());
            let preceding = if offered { "playground then steps aside." } else { "Go on setting up" };
            assert!(printed.find(preceding).unwrap() < printed.find("temper box").unwrap());
            assert!(printed.contains("Run temper on a server"));
        }
    }
    for step in [Step::Forge { forge: None }, Step::Model { way: None }, Step::Ready] {
        let root = f::welcome_root(step, true);
        for view in [f::page_view(&root), f::browser_page_view(&root)] {
            assert!(!String::from_utf8_lossy(&f::text(view.tree())).contains("temper box"));
        }
    }
}
#[test]
fn an_empty_store_offers_both_cards() {
    let root = f::welcome_root(Step::Welcome, false);
    assert!(root.pages().welcome().unwrap().decided);
    let view = f::page_view(&root);
    let mut bytes = Vec::new();
    for node in view.tree().nodes() {
        if node.role == Role::Button && node.text.is_empty() {
            bytes.extend_from_slice(node.name.as_deref().unwrap());
        }
        for run in &node.text {
            bytes.extend_from_slice(&run.text);
        }
    }
    let words = String::from_utf8(bytes).unwrap();
    let mut remaining = words.as_str();
    for phrase in [
        "Welcome",
        "Forge",
        "Model",
        "Ready",
        "Welcome to temper",
        "Reliable, scalable and flexible software factory.",
        "You describe what you want and how you want it done, and it happens.",
        "Set up temper",
        "Your forge: a Forgejo temper runs for you, or GitHub. Your model: ChatGPT or Claude.",
        "About five minutes",
        "Set up temper",
        "Try the playground",
        "A scripted model and a forge, built in. A taste of how temper feels, with nothing to connect.",
        "Fully offline · Ready at once",
        "Try the playground",
        "Tried the playground first? Set up whenever you like; the playground then steps aside.",
    ] {
        let at = remaining.find(phrase).unwrap_or_else(|| panic!("missing or out of order: {phrase}"));
        remaining = &remaining[at + phrase.len()..];
    }
    let primary: Vec<_> = view
        .tree()
        .nodes()
        .iter()
        .filter(|n| n.styles.as_slice().contains(&crate::look::FLOW_PRIMARY))
        .map(|n| n.name.as_deref().unwrap())
        .collect();
    assert_eq!(primary, [b"Set up temper".as_slice()]);
    let playground = node(view.tree(), Role::Button, "Try the playground");
    assert!(styled(view.tree(), playground, crate::look::FLOW_SECONDARY));
    assert_eq!(press(&root, Role::Button, "Try the playground"), d::Action::Pages(pages::Act::Playground));
    assert_eq!(
        press(&root, Role::Link, "Set up temper"),
        d::Action::Pages(pages::Act::Go { address: Address::Welcome { step: Step::Forge { forge: None } } })
    );
}
#[test]
fn the_welcome_tagline_and_lead_are_named_and_emphasize_what_and_how() {
    let view = f::page_view(&f::welcome_root(Step::Welcome, true));
    let tree = view.tree();
    let first = node(tree, Role::Paragraph, "Reliable, scalable and flexible software factory.");
    let second = node(tree, Role::Paragraph, "You describe what you want and how you want it done, and it happens.");
    for (id, style) in [(first, crate::look::WELCOME_TAGLINE), (second, crate::look::WELCOME_LEAD)] {
        let paragraph = tree.nodes().iter().find(|n| n.id == id).unwrap();
        assert!(paragraph.styles.as_slice().contains(&style));
    }
    let start = tree.nodes().iter().position(|n| n.id == first).unwrap();
    assert_eq!(tree.nodes()[start + usize::try_from(tree.nodes()[start].size).unwrap()].id, second);
    let start = tree.nodes().iter().position(|n| n.id == second).unwrap();
    let text = &tree.nodes()[start + 1];
    assert_eq!(text.role, Role::Text);
    let runs: Vec<_> = text.text.iter().map(|r| (r.run, r.text.as_ref())).collect();
    assert_eq!(
        runs,
        [
            (skein_ui::Run::Plain, b"You describe ".as_slice()),
            (skein_ui::Run::Strong, b"what".as_slice()),
            (skein_ui::Run::Plain, b" you want and ".as_slice()),
            (skein_ui::Run::Strong, b"how".as_slice()),
            (skein_ui::Run::Plain, b" you want it done, and it happens.".as_slice()),
        ]
    );
}
#[test]
fn the_welcome_shows_its_heading_alone_until_it_decides() {
    let (mut root, _, _) = f::root_at(Address::Chats { filter: ChatsFilter::All }, &f::limits());
    f::root_event(&mut root, d::Event::Went { place: d::Place::Address(Address::Welcome { step: Step::Welcome }) });
    assert!(!root.pages().welcome().unwrap().decided);
    let view = f::page_view(&root);
    node(view.tree(), Role::Heading { level: 1 }, "Welcome to temper");
    let regions: Vec<_> = view.tree().nodes().iter().filter(|node| node.role == Role::Region).collect();
    assert_eq!(regions.len(), 1);
    assert_eq!(regions[0].region, Some(crate::frame::link_region()));
    assert!(
        !view
            .tree()
            .nodes()
            .iter()
            .any(|node| node.role == Role::Paragraph && node.styles.as_slice().contains(&crate::look::WELCOME_LEAD))
    );
}
#[test]
fn the_steps_mark_the_current_one() {
    for (step, name) in [
        (Step::Welcome, "Welcome"),
        (Step::WelcomeAsked, "Welcome"),
        (Step::Forge { forge: None }, "Forge"),
        (Step::Forge { forge: Some(d::Choice::Owned) }, "Forge"),
        (Step::Model { way: None }, "Model"),
        (Step::Model { way: Some(d::ModelWay { provider: 17, kind: d::WayKind::Key, source: None }) }, "Model"),
        (Step::Ready, "Ready"),
    ] {
        let root = f::welcome_root(step, true);
        for mut view in [f::page_view(&root), f::browser_page_view(&root)] {
            let tree = view.tree();
            node(tree, Role::List, "Steps");
            let current: Vec<_> =
                tree.nodes().iter().filter(|n| n.states.as_slice().contains(&State::Current)).collect();
            assert_eq!(current.len(), 1);
            assert_eq!(current[0].role, Role::Item);
            assert!(!current[0].styles.as_slice().contains(&crate::look::FLOW_DONE));
            let list =
                tree.nodes().iter().position(|n| n.role == Role::List && n.name.as_deref() == Some(b"Steps")).unwrap();
            let inside = &tree.nodes()[list..list + usize::try_from(tree.nodes()[list].size).unwrap()];
            let links: Vec<_> =
                inside.iter().filter(|n| n.role == Role::Link).map(|n| (n.id, n.name.clone().unwrap())).collect();
            assert_eq!(links.len(), 4);
            let current_link = inside.iter().find(|n| forge::parent(tree, n.id) == Some(current[0].id)).unwrap();
            assert_eq!(current_link.name.as_deref(), Some(name.as_bytes()));
            for ((id, label), (expected, step)) in links.into_iter().zip([
                ("Welcome", Step::WelcomeAsked),
                ("Forge", Step::Forge { forge: None }),
                ("Model", Step::Model { way: None }),
                ("Ready", Step::Ready),
            ]) {
                assert_eq!(label.as_ref(), expected.as_bytes());
                let address = Address::Welcome { step };
                assert_eq!(crate::href(&mut view, id).as_deref(), Some(d::address::print(&address).as_ref()));
                assert_eq!(crate::act(&mut view, id, Act::Press), Some(d::Action::Pages(pages::Act::Go { address })));
            }
        }
    }
}

#[test]
fn the_steps_tick_forge_and_model_facts_in_both_shells() {
    for step in [Step::WelcomeAsked, Step::Forge { forge: None }, Step::Model { way: None }, Step::Ready] {
        for kept in [false, true] {
            for usable in [false, true] {
                let rows = if kept {
                    forge::rows(w::Connection::Connecting, w::Offer::Offered)
                } else {
                    skein_lib::List::with_capacity(0)
                };
                let (mut root, accounts) = f::forge_root_accounts(Address::Welcome { step }, rows);
                if usable {
                    f::welcome_account(&mut root, accounts);
                }
                let expected = pages::welcome::step_marks(step, kept, usable);
                for (view, terminal) in [(f::page_view(&root), true), (f::browser_page_view(&root), false)] {
                    let tree = view.tree();
                    let items: Vec<_> =
                        tree.nodes().iter().filter(|n| n.styles.as_slice().contains(&crate::look::FLOW_STEP)).collect();
                    assert_eq!(items.len(), 4);
                    for ((item, mark), label) in
                        items.into_iter().zip(expected).zip(["Welcome", "Forge", "Model", "Ready"])
                    {
                        assert_eq!(item.states.as_slice().contains(&State::Current), mark.current);
                        assert_eq!(item.styles.as_slice().contains(&crate::look::FLOW_DONE), mark.ticked);
                        let link = tree.nodes().iter().find(|n| forge::parent(tree, n.id) == Some(item.id)).unwrap();
                        let words = f::nodes_text(
                            &tree.nodes()[tree.nodes().iter().position(|n| n.id == link.id).unwrap()..]
                                [..usize::try_from(link.size).unwrap()],
                        );
                        let ticked = crate::accounts::words::join(&["✓ ".as_bytes(), label.as_bytes()]);
                        assert_eq!(
                            words.as_ref(),
                            if terminal && mark.ticked { ticked.as_ref() } else { label.as_bytes() }
                        );
                    }
                    let mut print = skein_ui_text::Print::new(&view.limits().tree).unwrap();
                    skein_ui_text::render(tree, b"temper", None, &mut print);
                    let text = String::from_utf8_lossy(print.text.as_slice());
                    assert!(text.contains("Steps:"));
                    for (mark, label) in expected.into_iter().zip(["Welcome", "Forge", "Model", "Ready"]) {
                        let ticked = crate::accounts::words::join(&["✓ ".as_bytes(), label.as_bytes()]);
                        assert_eq!(
                            skein_lib::bytes::find(print.text.as_slice(), &ticked).is_some(),
                            terminal && mark.ticked
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn the_welcome_brand_goes_to_the_inbox() {
    for step in [Step::Welcome, Step::WelcomeAsked, Step::Forge { forge: None }, Step::Model { way: None }, Step::Ready]
    {
        let root = f::welcome_root(step, true);
        for mut view in [f::page_view(&root), f::browser_page_view(&root)] {
            let brand = node(view.tree(), Role::Link, "temper");
            assert_eq!(crate::href(&mut view, brand).as_deref(), Some(b"/".as_slice()));
            assert_eq!(
                crate::act(&mut view, brand, Act::Press),
                Some(d::Action::Pages(pages::Act::Go { address: Address::Inbox { filter: d::address::Filter::ALL } }))
            );
        }
    }
}

#[test]
fn the_asked_welcome_resumes_a_kept_forge_without_the_playground() {
    for usable in [false, true] {
        let (mut root, accounts) = f::forge_root_accounts(
            Address::Welcome { step: Step::WelcomeAsked },
            forge::rows(w::Connection::Connecting, w::Offer::Offered),
        );
        if usable {
            f::welcome_account(&mut root, accounts);
        }
        for mut view in [f::page_view(&root), f::browser_page_view(&root)] {
            node(view.tree(), Role::Region, "Set up temper");
            let go_on = node(view.tree(), Role::Link, "Go on setting up");
            assert!(styled(view.tree(), go_on, crate::look::FLOW_PRIMARY));
            let step = if usable { Step::Ready } else { Step::Model { way: None } };
            assert_eq!(
                crate::act(&mut view, go_on, Act::Press),
                Some(d::Action::Pages(pages::Act::Go { address: Address::Welcome { step } }))
            );
            assert!(f::find(view.tree(), Role::Region, b"Try the playground").is_none());
            assert!(
                f::find(
                    view.tree(),
                    Role::Paragraph,
                    b"Tried the playground first? Set up whenever you like; the playground then steps aside."
                )
                .is_none()
            );
        }
    }
}

#[test]
fn a_real_model_alone_keeps_the_playground_in_the_foot() {
    for step in [Step::Forge { forge: None }, Step::Model { way: None }] {
        let (mut root, accounts) = f::forge_root_accounts(Address::Welcome { step }, skein_lib::List::with_capacity(0));
        f::welcome_account(&mut root, accounts);
        for mut view in [f::page_view(&root), f::browser_page_view(&root)] {
            let playground = node(view.tree(), Role::Button, "Try the playground instead");
            assert_eq!(crate::act(&mut view, playground, Act::Press), Some(d::Action::Pages(pages::Act::Playground)));
        }
    }
}
#[test]
fn the_model_step_offers_the_accounts_ways() {
    let mut root = super::connect_model::offered_root();
    f::root_event(
        &mut root,
        d::Event::Went { place: d::Place::Address(Address::Welcome { step: Step::Model { way: None } }) },
    );
    let view = f::page_view(&root);
    node(view.tree(), Role::Heading { level: 1 }, "Connect a model");
    node(view.tree(), Role::Paragraph, "temper’s agents run on a plan you sign in to.");
    node(view.tree(), Role::Heading { level: 2 }, "ChatGPT");
    node(view.tree(), Role::Paragraph, "Your ChatGPT plan: sign in, and temper’s agents use its included usage.");
    node(view.tree(), Role::Paragraph, "Sign in, in a new tab");
    let sign_in = node(view.tree(), Role::Button, "Continue with ChatGPT");
    assert!(styled(view.tree(), sign_in, crate::look::FLOW_PRIMARY));
    assert!(f::find(view.tree(), Role::Button, "This browser isn't on temper's computer ›".as_bytes()).is_none());
    // A plan alone: no key's line or link on the welcome.
    assert!(!String::from_utf8_lossy(&f::text(view.tree())).contains("key"));
    assert!(f::find(view.tree(), Role::Link, b"Use an OpenAI API key").is_none());
    let d::Action::Pages(pages::Act::ConnectModel(pages::connect_model::Act::SignIn { .. })) =
        press(&root, Role::Button, "Continue with ChatGPT")
    else {
        panic!("ChatGPT's sign-in")
    };
}
#[test]
fn the_model_step_offers_each_plan_and_no_key() {
    for lendable in [false, true] {
        let (mut root, _) =
            super::connect_model::terminal_root(true, super::connect_model::terminal_document(lendable));
        super::connect_model::assert_terminal_cards(&root, true, lendable);
        let view = f::page_view(&root);
        assert!(!String::from_utf8_lossy(&f::text(view.tree())).contains("key"));
        if lendable {
            let action = press(&root, Role::Button, "Sign in to Claude Code");
            f::root_event(&mut root, d::Event::Act { action });
            let pending = root.pages().connect_model().unwrap().request.pending.as_ref().unwrap();
            assert_eq!(
                pending.ask,
                jig_client::contract::Ask::AccountChange {
                    holder: jig_client::contract::AskAccountHolder::Own,
                    change: jig_client::contract::AccountChange::LoginProgram {
                        provider: 18,
                        name: b"claude-pat".as_slice().into()
                    },
                }
            );
        }
    }
}
#[test]
fn model_keeps_the_terminal_login_note_across_documents_and_reopening() {
    super::connect_model::terminal_note_states(true);
}
#[test]
fn model_terminal_login_endings_and_refusals_put_the_cards_back() {
    super::connect_model::terminal_ending_rows(true);
}
#[test]
fn a_signed_in_terminal_login_stays_usable_until_continue() {
    use jig_client::contract as c;
    for email in [None, Some(b"pat@example.com".as_slice())] {
        let mut doc = super::accounts::card_document(b"anthropic", b"claude-pat", c::AccountKind::Program);
        let row = doc.accounts.get_mut(0).unwrap();
        row.provider = Some(18);
        row.facts = super::accounts::email_facts(email);
        row.sources.get_mut(0).unwrap().number = 43;
        doc.cut = false;
        doc.settled = true;
        doc.signing.clear();
        doc.lent_end = Some(c::AccountLentEnd { number: 91, end: c::LoginEnd::SignedIn { source: 43 } });
        let (mut root, _) = super::connect_model::terminal_root(true, doc);
        let expected = if email.is_some() {
            "Claude subscription · through Claude Code, signed in · pat@example.com"
        } else {
            "Claude subscription · through Claude Code, signed in"
        };
        assert_eq!(
            crate::pages::accounts::first_usable_short(&root, &f::words()).unwrap().as_ref(),
            expected.as_bytes()
        );
        for (browser, mut view) in [(false, f::page_view(&root)), (true, f::browser_page_view(&root))] {
            let tree = view.tree();
            node(tree, Role::Status, expected);
            let next = node(tree, Role::Link, "Continue");
            assert!(styled(tree, next, crate::look::FLOW_PRIMARY));
            assert!(f::find(tree, Role::Region, b"Claude").is_none());
            assert!(
                f::find(tree, Role::Paragraph, "temper’s agents run on a plan you sign in to.".as_bytes()).is_none()
            );
            if browser {
                assert!(tree.nodes().iter().enumerate().any(|(i, n)| {
                    n.styles.as_slice().contains(&crate::look::FLOW_MUTED)
                        && super::inbox::subtree(tree, i)
                            .iter()
                            .flat_map(|n| n.text.iter().flat_map(|r| r.text.iter().copied()))
                            .collect::<Vec<_>>()
                            .starts_with(b"through Claude Code, signed in")
                }));
            } else {
                assert!(terminal_print(&view).contains("✓ Claude subscription · through Claude Code, signed in"));
            }
            assert_eq!(
                crate::act(&mut view, next, Act::Press),
                Some(d::Action::Pages(pages::Act::Go { address: Address::Welcome { step: Step::Ready } }))
            );
        }
        f::root_event(&mut root, d::Event::Went { place: d::Place::Address(Address::Welcome { step: Step::Ready }) });
        assert_eq!(root.pages().address(), &Address::Welcome { step: Step::Ready });
    }
}
#[test]
fn each_later_step_offers_the_playground_only_for_an_empty_store() {
    for (step, back) in
        [(Step::Forge { forge: None }, Step::Welcome), (Step::Model { way: None }, Step::Forge { forge: None })]
    {
        for playground in [false, true] {
            let root = f::welcome_root(step, playground);
            assert_eq!(
                press(&root, Role::Link, "‹ Back"),
                d::Action::Pages(pages::Act::Go { address: Address::Welcome { step: back } })
            );
            let view = f::page_view(&root);
            assert!(f::find(view.tree(), Role::Link, b"Later").is_none());
            let id = node(view.tree(), Role::Link, "‹ Back");
            assert!(styled(view.tree(), id, crate::look::FLOW_QUIET));
            assert!(f::find(view.tree(), Role::Button, b"Try the playground instead").is_some());
            {
                let id = node(view.tree(), Role::Button, "Try the playground instead");
                assert!(
                    !view
                        .tree()
                        .nodes()
                        .iter()
                        .find(|n| n.id == id)
                        .unwrap()
                        .styles
                        .as_slice()
                        .contains(&crate::look::FLOW_PRIMARY)
                );
                assert_eq!(
                    press(&root, Role::Button, "Try the playground instead"),
                    d::Action::Pages(pages::Act::Playground)
                );
            }
        }
    }
}
#[test]
fn an_unavailable_welcome_keeps_the_signed_out_and_out_of_date_frame() {
    for event in [
        d::Event::SignedOut,
        d::Event::Client {
            event: jig_client::Event::Up { up: jig_client::Up::Served { served: false, versions: None } },
        },
    ] {
        let mut root = f::welcome_root(Step::Welcome, true);
        f::root_event(&mut root, event);
        let view = f::page_view(&root);
        assert!(f::find(view.tree(), Role::Disclosure, b"Menu").is_none());
        assert_eq!(f::styles(view.tree(), Role::Banner).as_ref(), [Box::from([crate::look::TOP])]);
        assert!(f::find(view.tree(), Role::Heading { level: 1 }, b"Welcome to temper").is_none());
    }
}

fn terminal_print(view: &crate::View) -> String {
    let mut print = skein_ui_text::Print::new(&view.limits().tree).unwrap();
    skein_ui_text::render(view.tree(), b"temper", None, &mut print);
    String::from_utf8(print.text.as_slice().to_vec()).unwrap()
}
fn ready_model_fixture() -> (d::Domain, skein_lib::Token) {
    let (mut root, watch) = super::connect_model::flow_fixture(d::WayKind::SignIn);
    f::root_event(&mut root, d::Event::Went { place: d::Place::Address(Address::Welcome { step: Step::Ready }) });
    (root, watch)
}
fn signing_document() -> jig_client::contract::Accounts {
    let mut doc =
        super::accounts::card_document(b"openai", b"alice@example.com", jig_client::contract::AccountKind::SignIn);
    doc.accounts.clear();
    doc.usable = false;
    doc
}
#[test]
fn ready_waits_for_the_accounts_document_before_drawing_a_model_line() {
    for step in [Step::Ready, Step::Model { way: None }] {
        let (mut root, _) = ready_model_fixture();
        f::root_event(&mut root, d::Event::Went { place: d::Place::Address(Address::Welcome { step }) });
        for view in [f::page_view(&root), f::browser_page_view(&root)] {
            assert_eq!(pane_lines(view.tree()).len(), 1);
            assert!(
                !view.tree().nodes().iter().any(|n| n.name.as_deref().is_some_and(|name| name.starts_with(b"Model ")))
            );
            assert!(f::find(view.tree(), Role::Link, b"Connect a model").is_none());
        }
    }
}
#[test]
fn ready_names_an_own_sign_in_under_way_without_a_connect_act() {
    let (mut root, watch) = ready_model_fixture();
    super::accounts::deliver(&mut root, watch, signing_document());
    for view in [f::page_view(&root), f::browser_page_view(&root)] {
        let tree = view.tree();
        let line = node(tree, Role::Paragraph, "Model ChatGPT plan · signing in…");
        let pane = forge::parent(tree, line).unwrap();
        assert!(styled(tree, pane, crate::look::WELCOME_STATUS));
        assert!(f::find(tree, Role::Link, b"Connect a model").is_none());
        assert!(terminal_print(&view).contains("Model ChatGPT plan · signing in…"));
    }
}
#[test]
fn ready_does_not_name_another_holders_sign_in() {
    let (mut root, watch) = ready_model_fixture();
    let mut doc = signing_document();
    doc.signing.get_mut(0).unwrap().holder = jig_client::contract::AccountHolder::Deployment;
    super::accounts::deliver(&mut root, watch, doc);
    let view = f::page_view(&root);
    node(view.tree(), Role::Slot, "Model none yet");
    node(view.tree(), Role::Link, "Connect a model");
}
#[test]
fn ready_names_a_usable_account_before_a_sign_in_under_way() {
    let (mut root, watch) = ready_model_fixture();
    super::accounts::deliver(
        &mut root,
        watch,
        super::accounts::card_document(b"openai", b"alice@example.com", jig_client::contract::AccountKind::SignIn),
    );
    let view = f::page_view(&root);
    node(view.tree(), Role::Paragraph, "ChatGPT plan · alice@example.com");
    assert!(f::find(view.tree(), Role::Link, b"Connect a model").is_none());
}

fn configured_model_document() -> jig_client::contract::Accounts {
    let mut document = signing_document();
    document.signing.clear();
    document.usable = true;
    document
}
fn configured_model_fixture(address: Address) -> d::Domain {
    let (mut root, watch) = f::forge_root_accounts(
        address,
        super::connect_forge::rows(
            temper_client_protocol_wire::v1::Connection::Connecting,
            temper_client_protocol_wire::v1::Offer::Offered,
        ),
    );
    let document = configured_model_document();
    super::accounts::deliver(&mut root, watch, document);
    root
}
#[test]
fn ready_names_the_configured_model_without_an_account_row() {
    let root = configured_model_fixture(Address::Welcome { step: Step::Ready });
    let view = f::page_view(&root);
    node(view.tree(), Role::Paragraph, "the default model");
    assert!(f::find(view.tree(), Role::Link, b"Connect a model").is_none());
}

#[test]
fn ready_names_the_new_chat_templates_model_and_effort() {
    use jig_client::contract as c;
    let text_bytes = f::limits().client.domain.text;
    let (mut root, me_watch, _, _, accounts_watch) =
        f::root_at_full(Address::Welcome { step: Step::Ready }, &f::limits(), b"Shop");
    let template_root = f::chat::messaging_root();
    let mut me = pages::accounts::current_me(template_root.client()).unwrap().clone();
    let project = me.projects.get_mut(0).unwrap();
    let template = project.templates.get_mut(0).unwrap();
    template.llm = Some(c::LlmChoice {
        later: None,
        account: None,
        main: c::ModelEffort {
            model: c::Text::new(b"configured-model".as_slice().into(), text_bytes).unwrap(),
            effort: Some(c::Text::new(b"high".as_slice().into(), text_bytes).unwrap()),
        },
        subagents: skein_lib::List::with_capacity(0),
    });
    f::root_event(
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
    let document = configured_model_document();
    super::accounts::deliver(&mut root, accounts_watch.unwrap(), document);
    let view = f::page_view(&root);
    node(view.tree(), Role::Paragraph, "configured-model · high");
}

#[test]
fn selected_forge_choices_have_one_back_to_choose() {
    for choice in [d::Choice::Forge(2), d::Choice::Forge(3), d::Choice::Elsewhere] {
        let root = f::forge_root(
            Address::Welcome { step: Step::Forge { forge: Some(choice) } },
            forge::rows(w::Connection::NotSetUp, w::Offer::Offered),
        );
        let view = f::browser_page_view(&root);
        assert!(f::find(view.tree(), Role::Link, "‹ All forges".as_bytes()).is_none());
        assert_eq!(view.tree().nodes().iter().filter(|n| n.name.as_deref() == Some("‹ Back".as_bytes())).count(), 1);
        assert_eq!(
            press(&root, Role::Link, "‹ Back"),
            d::Action::Pages(pages::Act::Go { address: Address::Welcome { step: Step::Forge { forge: None } } })
        );
        if choice == d::Choice::Elsewhere {
            node(view.tree(), Role::Heading { level: 2 }, "A Forgejo elsewhere");
        }
        if choice == d::Choice::Forge(2) {
            node(view.tree(), Role::Heading { level: 2 }, "An App of your own");
            assert!(f::find(view.tree(), Role::Heading { level: 2 }, b"GitHub").is_none());
        }
    }
}

#[test]
fn ready_has_a_model_back_link_and_the_offered_playground() {
    for (rows, offered) in [(skein_lib::List::with_capacity(0), true), (owned_adopted_rows(), false)] {
        let root = f::forge_root(Address::Welcome { step: Step::Ready }, rows);
        for mut view in [f::page_view(&root), f::browser_page_view(&root)] {
            let tree = view.tree();
            let back = node(tree, Role::Link, "‹ Back");
            assert!(has_style(tree, crate::look::WELCOME_FOOT));
            let playground = f::find(tree, Role::Button, b"Try the playground instead");
            assert_eq!(playground.is_some(), offered);
            assert_eq!(
                crate::act(&mut view, back, Act::Press),
                Some(d::Action::Pages(pages::Act::Go {
                    address: Address::Welcome { step: Step::Model { way: None } }
                }))
            );
            if let Some(playground) = playground {
                assert_eq!(
                    crate::act(&mut view, playground, Act::Press),
                    Some(d::Action::Pages(pages::Act::Playground))
                );
            }
        }
    }
}

#[test]
fn a_kept_real_forge_hides_the_playground() {
    let mut cases = Vec::new();
    cases.push(forge::rows(forge::connected(), w::Offer::Offered));
    for app in [w::AppWay::Manifest, w::AppWay::Pasted] {
        let mut rows = forge::github_rows(false);
        let mut parts = rows.get(0).unwrap().clone().into_parts();
        parts.app = Some(app);
        *rows.get_mut(0).unwrap() = w::ForgeRow::new(&w::CEILINGS, parts).unwrap();
        cases.push(rows);
    }
    for rows in cases {
        for step in [Step::Forge { forge: None }, Step::Model { way: None }] {
            let root = f::playground_forges_root(Address::Welcome { step }, rows.clone());
            assert!(root.forge().playground().is_some(), "the cached answer has not been reread yet");
            let view = f::page_view(&root);
            assert!(f::find(view.tree(), Role::Button, b"Try the playground instead").is_none(), "{:?}", root.forges());
        }
    }
}
#[test]
fn ready_under_way_lists_model_forge_and_project_with_folded_progress() {
    let (mut root, account) = f::forge_root_accounts(
        Address::Welcome { step: Step::Ready },
        forge::first_rows(w::Connection::Connecting, w::FirstState::Making),
    );
    f::welcome_account(&mut root, account);
    let view = f::page_view(&root);
    node(view.tree(), Role::Heading { level: 1 }, "Almost ready");
    let wait = "Your first project can be created once your forge and model are ready.";
    forge::words_in_order(
        view.tree(),
        &[
            wait,
            "Your first project",
            "Create your project",
            "Rename it whenever you like.",
            "Forgejo on this machine",
            "Account 2",
        ],
    );
    let lead = node(view.tree(), Role::Paragraph, wait);
    let lead = view.tree().nodes().iter().find(|n| n.id == lead).unwrap();
    assert!(lead.styles.as_slice().contains(&crate::look::WELCOME_LEAD));
    let disclosure = node(view.tree(), Role::Disclosure, "Each step");
    assert!(
        !view.tree().nodes().iter().find(|n| n.id == disclosure).unwrap().states.as_slice().contains(&State::Expanded)
    );
    assert!(!view.tree().nodes().iter().any(|n| n.styles.as_slice().contains(&crate::look::FLOW_CARD)));
    let open = node(view.tree(), Role::Button, "Create your project");
    assert!(view.tree().nodes().iter().find(|n| n.id == open).unwrap().states.as_slice().contains(&State::Disabled));
    // Its name is written while the rest gets ready; Enter makes nothing yet.
    let field = node(view.tree(), Role::TextField, "Your first project");
    let field = view.tree().nodes().iter().find(|n| n.id == field).unwrap();
    assert!(!field.states.as_slice().contains(&State::Disabled));
    assert!(!field.states.as_slice().contains(&State::SubmitsOnEnter));
}

#[test]
fn welcome_keeps_failed_setup_steps_at_model_forge_and_ready() {
    for step in [Step::Model { way: None }, Step::Forge { forge: Some(d::Choice::Forge(HERE)) }, Step::Ready] {
        for (rows, failed, why) in forge::failure_cases() {
            let view = f::page_view(&f::forge_root(Address::Welcome { step }, rows));
            forge::failed_progress(view.tree(), failed, why);
        }
    }
}

fn in_welcome(root: &mut d::Domain, kind: d::WayKind) {
    f::root_event(root, d::Event::Went { place: d::Place::Address(model_address(kind)) });
}
fn model_back(root: &d::Domain) {
    for mut view in [f::page_view(root), f::browser_page_view(root)] {
        let tree = view.tree();
        assert_eq!(tree.nodes().iter().filter(|n| n.name.as_deref() == Some("‹ Back".as_bytes())).count(), 1);
        assert!(f::find(tree, Role::Link, b"All ways").is_none());
        assert!(!String::from_utf8_lossy(&f::text(tree)).contains("All ways"));
        let back = node(tree, Role::Link, "‹ Back");
        assert_eq!(
            crate::act(&mut view, back, Act::Press),
            Some(d::Action::Pages(pages::Act::Go { address: Address::Welcome { step: Step::Model { way: None } } }))
        );
    }
}
#[test]
fn model_choose_has_direct_ways_and_the_persons_default_holder() {
    use jig_client::contract as c;
    let mut root = super::connect_model::offered_root();
    f::root_event(
        &mut root,
        d::Event::Went { place: d::Place::Address(Address::Welcome { step: Step::Model { way: None } }) },
    );
    let view = f::page_view(&root);
    let tree = view.tree();
    for name in ["Use your accounts", "Use deployment accounts"] {
        assert!(f::find(tree, Role::Button, name.as_bytes()).is_none());
    }
    assert!(f::find(tree, Role::Region, b"Choose").is_none());
    assert_eq!(root.pages().connect_model().unwrap().holder, c::AskAccountHolder::Own);
    assert_eq!(tree.nodes().iter().filter(|n| n.styles.as_slice().contains(&crate::look::FLOW_CHOICE)).count(), 1);
    assert_eq!(
        press(&root, Role::Link, "‹ Back"),
        d::Action::Pages(pages::Act::Go { address: Address::Welcome { step: Step::Forge { forge: None } } })
    );
}
#[test]
fn model_details_and_signing_use_one_back_to_their_choose() {
    for kind in [d::WayKind::Key, d::WayKind::Program, d::WayKind::SignIn] {
        let (mut root, _) = super::connect_model::flow_fixture(kind);
        in_welcome(&mut root, kind);
        model_back(&root);
        let view = f::page_view(&root);
        let tree = view.tree();
        if kind == d::WayKind::Key {
            node(tree, Role::Heading { level: 1 }, "An OpenAI API key");
            let field = node(tree, Role::TextField, "Key, pasted once");
            let field = tree.nodes().iter().find(|n| n.id == field).unwrap();
            assert_eq!(field.placeholder, None);
            assert_eq!(field.entry, Some(skein_ui::EntryKind::Secret));
            assert!(f::find(tree, Role::TextField, b"Name").is_none());
            node(tree, Role::Button, "Add key");
            let words = String::from_utf8(f::text(tree).into_vec()).unwrap();
            assert!(words.contains(
                "From OpenAI’s platform, under API keys. What temper’s agents use is billed to the key’s organization."
            ));
            for absent in ["A pasted value is sent once", "A key reads Ready until a run is refused"] {
                assert!(!words.contains(absent), "drew {absent}");
            }
        } else if kind == d::WayKind::SignIn {
            node(tree, Role::Heading { level: 1 }, "Signing in to ChatGPT");
        }
    }
}
#[test]
fn an_anthropic_key_in_the_welcome_names_its_console_and_brand() {
    let mut root = super::connect_model::plans_root();
    f::root_event(
        &mut root,
        d::Event::Went {
            place: d::Place::Address(Address::Welcome {
                step: Step::Model { way: Some(d::ModelWay { provider: 18, kind: d::WayKind::Key, source: None }) },
            }),
        },
    );
    model_back(&root);
    for view in [f::page_view(&root), f::browser_page_view(&root)] {
        node(view.tree(), Role::Heading { level: 1 }, "An Anthropic API key");
        let words = String::from_utf8(f::text(view.tree()).into_vec()).unwrap();
        assert!(words.contains(
            "From the Claude Console, under API keys. What temper’s agents use is billed to the key’s organization."
        ));
        assert!(
            words.contains("Nothing checks a key when it is added: a wrong one shows once Anthropic refuses a run.")
        );
        assert!(!words.contains("OpenAI"));
    }
}
fn model_ending_in_welcome(end: jig_client::contract::AccountEnd) -> d::Domain {
    let mut root = super::connect_model::ending(end, b"alice@example.com");
    in_welcome(&mut root, d::WayKind::SignIn);
    root
}
#[test]
fn a_made_model_keeps_the_welcomes_connected_frame() {
    let root = model_ending_in_welcome(jig_client::contract::AccountEnd::Made { account: 1, declined: false });
    let view = f::page_view(&root);
    node(view.tree(), Role::Heading { level: 1 }, "Connect a model");
    node(view.tree(), Role::Status, "ChatGPT plan · alice@example.com");
    node(view.tree(), Role::Link, "Continue");
}
#[test]
fn a_failed_model_keeps_the_welcomes_sign_in_back() {
    use jig_client::contract::{AccountEnd, AccountFailure};
    let root = model_ending_in_welcome(AccountEnd::Failed { failure: AccountFailure::Misconfigured });
    model_back(&root);
    node(f::page_view(&root).tree(), Role::Heading { level: 1 }, "Signing in to ChatGPT");
}

fn github_adopted_rows() -> skein_lib::List<w::ForgeRow> {
    let mut rows = forge::github_rows(true);
    let mut parts = rows.get(0).unwrap().clone().into_parts();
    parts.repositories = skein_lib::List::with_capacity(1);
    parts
        .repositories
        .push(
            w::Adoption::new(
                &w::CEILINGS,
                w::AdoptionParts {
                    owner: b"sam".as_slice().into(),
                    name: b"widget".as_slice().into(),
                    project: f::PROJECT,
                },
            )
            .unwrap(),
        )
        .unwrap();
    *rows.get_mut(0).unwrap() = w::ForgeRow::new(&w::CEILINGS, parts).unwrap();
    rows
}
#[test]
fn model_ticks_connected_github_even_with_an_adopted_repository() {
    let view = model_view(github_adopted_rows());
    let id = lines_in_pane(view.tree(), "GitHub · Connected as my-app[bot] (9)");
    assert!(
        view.tree().nodes().iter().find(|n| n.id == id).unwrap().styles.as_slice().contains(&crate::look::FLOW_DONE)
    );
    each_step(view.tree(), false);
}

#[test]
fn managed_forgejo_ready_uses_the_default_checklist() {
    // Forge 4 is Forgejo elsewhere in the standalone configuration.
    const ELSEWHERE: u16 = 4;
    let mut rows = owned_adopted_rows();
    let mut parts = rows.get(0).unwrap().clone().into_parts();
    parts.number = ELSEWHERE;
    parts.mode = w::Mode::Managed;
    parts.admin = true;
    *rows.get_mut(0).unwrap() = w::ForgeRow::new(&w::CEILINGS, parts).unwrap();
    let root = ready_root(rows);
    let view = f::page_view(&root);
    node(view.tree(), Role::Heading { level: 1 }, "Ready");
    forge::words_in_order(
        view.tree(),
        &["Your first project", "Create your project", "Forgejo · connected as temper", "Account 2"],
    );
    assert_eq!(press(&root, Role::Button, "Create your project"), d::Action::Pages(pages::Act::MakeDefault));
}
#[test]
fn github_ready_has_the_same_checklist_and_project_control_as_owned_forgejo() {
    for (rows, heading, forge_line, project_line) in [
        (github_adopted_rows(), "Ready", "GitHub · Connected as my-app[bot] (9)", "Your first project"),
        (
            owned_adopted_rows(),
            "Ready",
            "Forgejo on this machine · connected as temper · work/hello made",
            "Your first project",
        ),
    ] {
        let root = ready_root(rows);
        let view = f::page_view(&root);
        node(view.tree(), Role::Heading { level: 1 }, heading);
        forge::words_in_order(view.tree(), &[project_line, "Create your project", forge_line, "Account 2"]);
        assert_eq!(press(&root, Role::Button, "Create your project"), d::Action::Pages(pages::Act::MakeDefault));
        assert!(
            f::find(view.tree(), Role::Link, "Open on GitHub ↗".as_bytes()).is_none(),
            "no repository web address in the row"
        );
    }
}
fn ready_ending(end: jig_client::contract::AccountEnd, message: &str, key: bool) {
    ready_ending_with_role(end, message, key, Role::Alert);
}
fn ready_status_ending(end: jig_client::contract::AccountEnd, message: &str) {
    ready_ending_with_role(end, message, false, Role::Status);
}
fn ready_ending_with_role(end: jig_client::contract::AccountEnd, message: &str, key: bool, role: Role) {
    use jig_client::contract as c;
    let (mut root, watch) = ready_model_fixture();
    let mut doc = signing_document();
    doc.signing.clear();
    doc.last_end = Some(c::AccountLastEnd { source: 43, end });
    super::accounts::deliver(&mut root, watch, doc);
    let view = f::page_view(&root);
    forge::words_in_order(view.tree(), &["Model none yet", "Connect a model", message, "Continue with ChatGPT"]);
    let tree = view.tree();
    let nodes = tree.nodes();
    let line = nodes.iter().position(|n| n.name.as_deref() == Some(b"Model none yet")).unwrap();
    let line_end = line.checked_add(usize::try_from(nodes[line].size).unwrap()).unwrap();
    assert!(
        nodes[line.checked_add(1).unwrap()..line_end]
            .iter()
            .any(|n| n.role == Role::Link && n.name.as_deref() == Some(b"Connect a model"))
    );
    assert!(f::node_text(tree, line).starts_with("Model none yet · ".as_bytes()));
    let ending = line_end;
    assert_eq!(nodes[ending].role, Role::Slot);
    assert!(nodes[ending].styles.as_slice().contains(&crate::look::WELCOME_END));
    let end = ending.checked_add(usize::try_from(nodes[ending].size).unwrap()).unwrap();
    assert!(
        (ending..end)
            .any(|index| nodes[index].role == role && f::node_text(tree, index).as_ref() == message.as_bytes()),
        "the ending keeps its Model role and words: {role:?} {message}"
    );
    assert!(
        nodes[ending..end]
            .iter()
            .any(|n| n.role == Role::Button && n.name.as_deref() == Some(b"Continue with ChatGPT"))
    );
    let create = nodes
        .iter()
        .find(|node| node.role == Role::Button && node.name.as_deref() == Some(b"Create your project"))
        .expect("Ready's Create");
    assert!(create.states.as_slice().contains(&State::Disabled));
    assert_eq!(f::find(view.tree(), Role::Link, b"Use an API key").is_some(), key);
    if key {
        assert_eq!(
            press(&root, Role::Link, "Use an API key"),
            d::Action::Pages(pages::Act::Go { address: model_address(d::WayKind::Key) })
        );
    }
    let action = press(&root, Role::Button, "Continue with ChatGPT");
    assert_eq!(
        action,
        d::Action::Pages(pages::Act::ConnectModel(pages::connect_model::Act::SignIn {
            name: root.pages().account_name(),
            way: pages::accounts::Way::SignIn { provider: 17, plan: true }
        }))
    );
    let limits = f::limits();
    let mut out = skein_lib::Queue::uncapped();
    d::step(
        &mut root,
        &skein_lib::Env { now: skein_lib::Time::ZERO, wall: f::NOW, limits },
        d::Event::Act { action },
        &mut out,
    );
    let mut started = false;
    for request in &out {
        if let d::Request::Show { address: Address::Welcome { step: Step::Model { way: Some(_) } }, .. } = request {
            started = true;
        }
    }
    assert!(started, "Ready's retry starts the actual model flow");
}
#[test]
fn ready_draws_a_declined_sign_in_with_retry_and_key() {
    ready_ending(
        jig_client::contract::AccountEnd::Made { account: 1, declined: true },
        "You signed in without plan usage.",
        true,
    );
}
#[test]
fn ready_draws_a_held_sign_in_with_retry() {
    ready_ending(
        jig_client::contract::AccountEnd::Refused {
            refusal: jig_client::contract::AccountRefusal::Held { provider: 17 },
        },
        "Another person here holds that ChatGPT account. To use one of yours, sign out of ChatGPT in your browser first, then Continue with ChatGPT.",
        false,
    );
}
#[test]
fn ready_draws_a_misconfigured_sign_in_with_retry() {
    ready_ending(
        jig_client::contract::AccountEnd::Failed { failure: jig_client::contract::AccountFailure::Misconfigured },
        "OpenAI refused temper’s sign-in. Check the configuration’s registration.",
        false,
    );
}

#[test]
fn ready_draws_every_other_sign_in_ending_with_its_model_role_and_retry() {
    use jig_client::contract::{self as c, AccountEnd as E, AccountFailure as F, AccountRefusal as R};
    for (end, message) in [
        (E::Refused { refusal: R::NotLimited }, "This source is not limited."),
        (E::Refused { refusal: R::Uses }, "The provider does not permit these uses."),
        (E::Refused { refusal: R::NotKept { why: c::AccountKeyFailure::TooLong } }, "The key is too long."),
        (E::Refused { refusal: R::NotKept { why: c::AccountKeyFailure::NotStored } }, "The key could not be stored."),
        (E::Refused { refusal: R::Limit { bound: c::AccountBound::Accounts } }, "Account limit reached."),
        (E::Refused { refusal: R::Limit { bound: c::AccountBound::HolderAccounts } }, "Holder account limit reached."),
        (E::Refused { refusal: R::Limit { bound: c::AccountBound::Sources } }, "Source limit reached."),
        (E::Refused { refusal: R::Limit { bound: c::AccountBound::SignIns } }, "Sign-in limit reached."),
        (E::Refused { refusal: R::Provider { provider: 17 } }, "Unknown or unavailable LLM provider 17"),
        (E::Refused { refusal: R::Subject }, "This provider subject cannot be joined."),
        (E::Refused { refusal: R::Unknown }, "Unknown account or source."),
        (E::Refused { refusal: R::Numbers }, "No account or source numbers remain."),
        (E::Refused { refusal: R::Kind { kind: c::AccountKind::Program } }, "Source kind refused: Program login"),
        (E::Refused { refusal: R::Closing }, "Closing"),
        (E::Refused { refusal: R::NotStarted }, "Sign-in has not started."),
        (E::Refused { refusal: R::NotSignedIn }, "Not signed in"),
        (E::Refused { refusal: R::NotOurs }, "Not this application's record"),
        (E::Refused { refusal: R::Cancelled }, "The move was cancelled"),
        (E::Refused { refusal: R::Moving }, "Moving to another deployment"),
        (E::Failed { failure: F::Later { after: None } }, "Try later"),
        (
            E::Failed { failure: F::Later { after: Some(skein_lib::Duration::from_secs(5)) } },
            "Try laterTry again after 5",
        ),
        (E::Failed { failure: F::SignInAgain }, "Needs sign-in"),
        (E::Failed { failure: F::MissingScope { scope: 19 } }, "Missing required scope 19"),
        (
            E::Failed { failure: F::Unloaded { at: c::AccountPlace::File, why: c::AccountUnread::Unreadable } },
            "Cannot load File: Unreadable",
        ),
        (E::Failed { failure: F::Cancelled }, "Cancelled"),
    ] {
        ready_status_ending(end, message);
    }
}

#[test]
fn the_setup_card_names_supported_models_for_every_provider_catalogue() {
    use jig_client::contract as c;
    for (sign_in, program, key) in
        [(true, false, false), (false, true, false), (false, false, true), (false, false, false)]
    {
        let (mut root, watch, _) = f::root_at(Address::Welcome { step: Step::Welcome }, &f::domain_limits());
        let mut me = root.pages().me(root.client()).unwrap().clone();
        me.providers = skein_lib::List::with_capacity(1);
        if sign_in || program || key {
            let mut programs = skein_lib::List::with_capacity(1);
            if program {
                programs.push(c::Text::new(Box::from(b"claude_code".as_slice()), 32).unwrap()).unwrap();
            }
            me.providers
                .push(c::ProviderOffer {
                    provider: 20,
                    name: super::connect_model::text(b"Other model"),
                    key_bytes: 4096,
                    sign_in,
                    signed_in_plan: true,
                    key,
                    programs,
                    programs_cut: false,
                    none: false,
                    manage_usage: None,
                })
                .unwrap();
        }
        f::root_event(
            &mut root,
            d::Event::Client {
                event: jig_client::Event::Up {
                    up: jig_client::Up::Watched {
                        watch,
                        event: c::WatchEvent::Delivered(c::Delivery::Me(Box::new(me))),
                    },
                },
            },
        );
        for view in [f::page_view(&root), f::browser_page_view(&root)] {
            let block = view
                .tree()
                .nodes()
                .iter()
                .position(|n| n.styles.as_slice().contains(&crate::look::WELCOME_SETUP_DESCRIPTION))
                .unwrap();
            let lines: Vec<_> = view
                .tree()
                .nodes()
                .iter()
                .enumerate()
                .skip(block + 1)
                .take(usize::try_from(view.tree().nodes()[block].size).unwrap() - 1)
                .filter(|(_, n)| n.role == Role::Paragraph)
                .map(|(at, _)| f::node_text(view.tree(), at))
                .collect();
            assert_eq!(lines.len(), 1);
            assert_eq!(
                lines[0].as_ref(),
                b"Your forge: a Forgejo temper runs for you, or GitHub. Your model: ChatGPT or Claude."
            );
        }
    }
}

#[test]
fn welcomes_five_steps_use_the_owned_pins_without_a_future_login() {
    for (pins, runner, fetching, fetched) in [
        (true, true, "Fetching Forgejo 17.0.0 and its runner 11.2.1", "Fetched Forgejo 17.0.0 and its runner 11.2.1"),
        (true, false, "Fetching Forgejo and its runner", "Fetched Forgejo and its runner"),
        (false, false, "Fetching Forgejo and its runner", "Fetched Forgejo and its runner"),
    ] {
        for (step, expected) in [(w::SetUpStep::Fetch, fetching), (w::SetUpStep::Prepare, fetched)] {
            let connection = w::Connection::SettingUp(
                w::SettingUp::new(&w::CEILINGS, w::SettingUpParts { step: Some(step), fetched: None }).unwrap(),
            );
            let root =
                f::forge_root(Address::Welcome { step: Step::Ready }, forge::pinned_rows(connection, pins, runner));
            for view in [f::page_view(&root), f::browser_page_view(&root)] {
                forge::words_in_order(
                    view.tree(),
                    &[expected, "Preparing", "Starting Forgejo and its runner", "Connecting", "Making"],
                );
                assert!(!String::from_utf8_lossy(&f::main_text(view.tree())).contains(" sam"));
            }
        }
    }
}

#[test]
fn ready_offers_forgejo_sign_in_only_when_its_owned_forge_is_connected() {
    for (rows, offered) in [
        (owned_adopted_rows(), true),
        (forge::first_rows(w::Connection::Connecting, w::FirstState::Making), false),
        (
            forge::pinned_rows(
                w::Connection::SettingUp(
                    w::SettingUp::new(
                        &w::CEILINGS,
                        w::SettingUpParts { step: Some(w::SetUpStep::Fetch), fetched: None },
                    )
                    .unwrap(),
                ),
                true,
                true,
            ),
            false,
        ),
        (github_adopted_rows(), false),
    ] {
        let (mut root, accounts) = f::forge_root_accounts(Address::Welcome { step: Step::Ready }, rows);
        f::welcome_account(&mut root, accounts);
        for facts_present in [true, false] {
            let mut view = f::view_of(f::browser_words());
            if facts_present {
                let mut facts = crate::forge_sign_in::Forges::new();
                facts.read("3 forgejo http://127.0.0.1:3000 owned");
                facts.read("2 github http://127.0.0.1:3001 other");
                view.forge_sign_in(facts);
            }
            f::build(&mut view, &root);
            let link = f::find(view.tree(), Role::Link, "Open Forgejo ↗".as_bytes());
            assert_eq!(link.is_some(), offered && facts_present);
            if let Some(link) = link {
                assert_eq!(
                    crate::href(&mut view, link).as_deref(),
                    Some(b"http://127.0.0.1:3000/user/oauth2/temper".as_slice())
                );
                forge::words_in_order(view.tree(), &["Create your project", "Forgejo", "Open Forgejo ↗", "Account 2"]);
            }
        }
    }
}

#[test]
fn ready_offers_connect_a_model_as_a_terminal_command() {
    let (mut root, watch) = ready_model_fixture();
    let mut doc = signing_document();
    doc.signing.clear();
    super::accounts::deliver(&mut root, watch, doc);
    let mut view = f::page_view(&root);
    let control = node(view.tree(), Role::Link, "Connect a model");
    let (node, first) = f::terminal_command(
        view.tree(),
        &view.limits().tree,
        control,
        b"Connect a model",
        "Model none yet · ".as_bytes(),
    );
    assert_eq!(
        crate::act(&mut view, node, first),
        Some(d::Action::Pages(pages::Act::Go { address: Address::Welcome { step: Step::Model { way: None } } }))
    );
}

#[test]
fn an_unfinished_store_opens_at_its_step() {
    for (rows, step, heading) in [
        (forge::rows(w::Connection::NotSetUp, w::Offer::Offered), Step::Forge { forge: None }, "Connect a forge"),
        (owned_adopted_rows(), Step::Model { way: None }, "Connect a model"),
    ] {
        let mut root = f::forge_root(Address::Welcome { step: Step::Ready }, rows);
        assert_eq!(pages::welcome::decision(&root), Some(pages::welcome::FirstRun::Unfinished { step }));
        let address = first_run_destination(&mut root);
        assert!(f::find(f::page_view(&root).tree(), Role::Link, b"Go on setting up").is_some());
        assert_eq!(address, Address::Welcome { step });
        f::root_event(&mut root, d::Event::Went { place: d::Place::Address(address) });
        let view = f::page_view(&root);
        node(view.tree(), Role::Heading { level: 1 }, heading);
        assert!(f::find(view.tree(), Role::Button, b"Try the playground instead").is_none(), "{:?}", root.forges());
    }
}

#[test]
fn ready_making_and_its_endings_keep_one_retry_act() {
    for refused in [true, false] {
        let mut root = ready_root(owned_adopted_rows());
        let mut view = f::page_view(&root);
        let id = node(view.tree(), Role::Button, "Create your project");
        let action = crate::act(&mut view, id, Act::Press).unwrap();
        let mut out = skein_lib::Queue::uncapped();
        d::step(
            &mut root,
            &skein_lib::Env { now: skein_lib::Time::ZERO, wall: f::NOW, limits: f::domain_limits() },
            d::Event::Act { action },
            &mut out,
        );
        let mut attempt = None;
        while let Some(request) = out.pop() {
            if let d::Request::Unlinked {
                request: jig_client::UnlinkedRequest::Down { down: jig_client::Down::Send { attempt: sent, key, .. } },
            } = request
            {
                attempt = Some((sent, key));
            }
        }
        let view = f::page_view(&root);
        let id = node(view.tree(), Role::Button, "Creating My project…");
        assert!(view.tree().disabled(id));
        let field = node(view.tree(), Role::TextField, "Your first project");
        assert!(view.tree().disabled(field));
        if refused {
            f::root_event(
                &mut root,
                d::Event::Client {
                    event: jig_client::Event::Up {
                        up: jig_client::Up::Answer {
                            attempt: attempt.unwrap().0,
                            envelope: skein_lib::RequestEnvelope::Final,
                            answer: Some(jig_client::Answered::Jig(jig_client::contract::Answer::Refused {
                                refusal: jig_client::contract::Refusal::NameTaken,
                                at: None,
                            })),
                            why: None,
                        },
                    },
                },
            );
        } else {
            f::root_event(
                &mut root,
                d::Event::Client {
                    event: jig_client::Event::Up { up: jig_client::Up::NotKept { key: attempt.unwrap().1 } },
                },
            );
        }
        let mut view = f::page_view(&root);
        let id = node(view.tree(), Role::Button, "Create your project");
        assert!(!view.tree().disabled(id));
        assert_eq!(crate::act(&mut view, id, Act::Press), Some(d::Action::Pages(pages::Act::MakeDefault)));
        let text = f::text(view.tree());
        let text = core::str::from_utf8(&text).unwrap();
        if refused {
            assert!(text.contains("Your project was not made: "));
            assert!(text.contains("Another project has that name"));
        } else {
            assert!(text.contains("It is not known whether your project was made."));
        }
    }
}

#[test]
fn ready_names_its_first_project_in_a_form() {
    for (rows, offered, unnamed) in [
        (owned_adopted_rows(), "My project", "A name for it: My project, or one of yours."),
        (forge::first_rows(forge::connected(), forge::made()), "Hello", "A name for it: Hello, or one of yours."),
    ] {
        let mut root = ready_root(rows);
        let mut view = f::page_view(&root);
        let tree = view.tree();
        node(
            tree,
            Role::Paragraph,
            if offered == "Hello" {
                "Everything is connected. Create your first project, with work/hello in it, and start working."
            } else {
                "Everything is connected. Create your first project and start working."
            },
        );
        let form = node(tree, Role::Form, "Your first project");
        let form = tree.nodes().iter().position(|n| n.id == form).unwrap();
        assert!(tree.nodes()[form].styles.as_slice().contains(&crate::look::WELCOME_PROJECT));
        let end = form.checked_add(usize::try_from(tree.nodes()[form].size).unwrap()).unwrap();
        let inside = &tree.nodes()[form..end];
        let field = inside.iter().find(|n| n.role == Role::TextField).unwrap();
        assert_eq!(field.name.as_deref(), Some(b"Your first project".as_slice()));
        assert_eq!(field.value.as_ref().unwrap().text.as_ref(), offered.as_bytes());
        assert!(field.states.as_slice().contains(&State::SubmitsOnEnter));
        let field = field.id;
        for name in ["Create your project", "Rename it whenever you like."] {
            assert!(inside.iter().any(|n| n.name.as_deref() == Some(name.as_bytes())), "{name} in the form");
        }
        // Writing names it; Enter is Create.
        assert_eq!(
            crate::act(&mut view, field, Act::Input { value: b"Shop".as_slice().into() }),
            Some(d::Action::Pages(pages::Act::DefaultName { words: b"Shop".as_slice().into() }))
        );
        assert_eq!(crate::act(&mut view, field, Act::Submit), Some(d::Action::Pages(pages::Act::MakeDefault)));
        // No name: Create says so, and makes nothing.
        f::root_event(
            &mut root,
            d::Event::Act { action: d::Action::Pages(pages::Act::DefaultName { words: b" ".as_slice().into() }) },
        );
        f::root_event(&mut root, d::Event::Act { action: d::Action::Pages(pages::Act::MakeDefault) });
        let view = f::page_view(&root);
        node(view.tree(), Role::Alert, unnamed);
        let field = node(view.tree(), Role::TextField, "Your first project");
        assert!(
            view.tree().nodes().iter().find(|n| n.id == field).unwrap().states.as_slice().contains(&State::Invalid)
        );
    }
}

#[test]
fn forge_step_draws_its_status_in_the_column_and_continues() {
    let address = Address::Welcome { step: Step::Forge { forge: Some(d::Choice::Forge(HERE)) } };
    let mut github = forge::github_rows(true);
    let mut parts = github.get(0).unwrap().clone().into_parts();
    let accepted = parts.installations.get(0).unwrap().clone();
    *parts.installations.get_mut(0).unwrap() = parts.installations.get(1).unwrap().clone();
    *parts.installations.get_mut(1).unwrap() = accepted;
    *github.get_mut(0).unwrap() = w::ForgeRow::new(&w::CEILINGS, parts).unwrap();
    let mut no_first = forge::first_rows(forge::connected(), forge::made());
    let mut parts = no_first.get(0).unwrap().clone().into_parts();
    parts.first = None;
    *no_first.get_mut(0).unwrap() = w::ForgeRow::new(&w::CEILINGS, parts).unwrap();
    for (rows, address, line, done) in [
        (
            forge::first_rows(w::Connection::Connecting, w::FirstState::Making),
            address.clone(),
            "Forgejo on this machine · Connecting",
            false,
        ),
        (
            forge::first_rows(forge::connected(), w::FirstState::Making),
            address.clone(),
            "Forgejo on this machine · Connected as temper (42)",
            false,
        ),
        (
            forge::first_rows(forge::connected(), forge::made()),
            Address::Welcome { step: Step::Forge { forge: None } },
            "Forgejo on this machine · connected as temper · work/hello made",
            true,
        ),
        (
            no_first,
            Address::Welcome { step: Step::Forge { forge: None } },
            "Forgejo on this machine · connected as temper",
            true,
        ),
        (github, Address::Welcome { step: Step::Forge { forge: None } }, "GitHub · temper’s App on acme", true),
    ] {
        for usable in [false, true] {
            let (mut root, accounts) = f::forge_root_accounts(address.clone(), rows.clone());
            if usable {
                f::welcome_account(&mut root, accounts);
            }
            for (browser, mut view) in [(false, f::page_view(&root)), (true, f::browser_page_view(&root))] {
                let tree = view.tree();
                node(tree, Role::Heading { level: 1 }, "Connect a forge");
                node(tree, Role::Paragraph, "One forge is enough to begin; add others later, from Settings.");
                assert!(f::find(tree, Role::Link, b"View").is_none());
                let status = tree
                    .nodes()
                    .iter()
                    .find(|n| n.styles.as_slice().contains(&crate::look::WELCOME_FORGE_STATUS))
                    .expect("status in the column");
                assert_eq!(status.role, Role::Slot);
                assert_eq!(status.styles.as_slice().contains(&crate::look::FLOW_DONE), done);
                let main = tree.nodes().iter().find(|n| n.role == Role::Main).unwrap();
                assert_eq!(forge::parent(tree, status.id), Some(main.id));
                let paragraph = node(tree, Role::Paragraph, line);
                assert_eq!(forge::parent(tree, paragraph), Some(status.id));
                assert!(tree.nodes().iter().any(|n| n.styles.as_slice().contains(&crate::look::WELCOME_FOOT)));
                assert!(!tree.nodes().iter().any(|n| n.styles.as_slice().contains(&crate::look::WELCOME_STATUS)));
                // The terminal has one joined run; the browser has decorative markers and styled words.
                let text = tree.nodes().iter().position(|n| n.id == paragraph).unwrap();
                if browser {
                    let children = &tree.nodes()[text + 1..text + usize::try_from(tree.nodes()[text].size).unwrap()];
                    assert!(children.iter().any(|n| n.styles.as_slice().contains(&if done {
                        crate::look::WELCOME_OK
                    } else {
                        crate::look::WELCOME_SPIN
                    })));
                    assert!(children.iter().any(|n| n.text.iter().any(|r| r.run == skein_ui::Run::Strong)));
                } else {
                    let mut expected = if done { String::from("✓ ") } else { String::new() };
                    expected.push_str(line);
                    assert_eq!(f::node_text(tree, text).as_ref(), expected.as_bytes());
                    assert!(!tree.nodes().iter().any(|n| n.styles.as_slice().contains(&crate::look::WELCOME_SPIN)));
                }
                each_step(tree, !done);
                let continue_id = node(tree, Role::Link, "Continue");
                assert!(
                    tree.nodes()
                        .iter()
                        .find(|n| n.id == continue_id)
                        .unwrap()
                        .styles
                        .as_slice()
                        .contains(&crate::look::FLOW_PRIMARY)
                );
                assert_eq!(
                    crate::act(&mut view, continue_id, Act::Press),
                    Some(d::Action::Pages(pages::Act::Go {
                        address: Address::Welcome {
                            step: if usable { Step::Ready } else { Step::Model { way: None } }
                        }
                    }))
                );
            }
        }
    }
}

#[test]
fn forge_step_keeps_recovery_installation_and_admin_steps_without_view() {
    let mut cases = Vec::from([
        (forge::rows(w::Connection::Connecting, w::Offer::Offered), None, "FORGE · not finished"),
        (forge::github_rows(false), None, "FORGE · not finished"),
        (forge::github_rows(false), Some(d::Choice::Forge(2)), "Install temper’s App"),
    ]);
    for (rows, _, _) in forge::failure_cases() {
        cases.push((rows, Some(d::Choice::Forge(HERE)), "Set up failed"));
    }
    let admin = forge::rows(
        w::Connection::Stopped(
            w::Stopped::new(&w::CEILINGS, w::StoppedParts { reason: b"stopped".as_slice().into() }).unwrap(),
        ),
        w::Offer::Offered,
    );
    let mut parts = admin.get(1).unwrap().clone().into_parts();
    parts.admin = true;
    let mut rows = skein_lib::List::with_capacity(1);
    rows.push(w::ForgeRow::new(&w::CEILINGS, parts).unwrap()).unwrap();
    cases.push((
        rows,
        Some(d::Choice::Forge(HERE)),
        "Scopes: write:admin, read:user, write:organization, write:repository",
    ));
    for (rows, choice, expected) in cases {
        let root = f::forge_root(Address::Welcome { step: Step::Forge { forge: choice } }, rows);
        for view in [f::page_view(&root), f::browser_page_view(&root)] {
            let tree = view.tree();
            assert!(String::from_utf8_lossy(&f::main_text(tree)).contains(expected), "{expected}");
            node(
                tree,
                Role::Paragraph,
                "Where your code lives: you tell temper what you want, agents push and land code.",
            );
            assert!(f::find(tree, Role::Link, b"View").is_none());
            assert!(f::find(tree, Role::Link, b"Continue").is_none());
            assert!(!tree.nodes().iter().any(|n| n.styles.as_slice().contains(&crate::look::WELCOME_FORGE_STATUS)));
        }
    }
}

#[test]
fn forge_status_keeps_the_complete_login_and_repository() {
    // English fixtures accept a name of 64 bytes; an org/name occupies the joined 129-byte hole.
    const NAME_BYTES: usize = 64;
    let login = "l".repeat(NAME_BYTES);
    let org = "o".repeat(NAME_BYTES);
    let repository = "r".repeat(NAME_BYTES);
    let mut rows = forge::first_rows(forge::connected(), forge::made());
    let mut parts = rows.get(0).unwrap().clone().into_parts();
    parts.connection = w::Connection::Connected(
        w::Connected::new(&w::CEILINGS, w::ConnectedParts { id: 42, login: login.as_bytes().into() }).unwrap(),
    );
    let mut first = parts.first.take().unwrap().into_parts();
    first.org = org.as_bytes().into();
    first.repository = repository.as_bytes().into();
    parts.first = Some(w::FirstRepository::new(&w::CEILINGS, first).unwrap());
    *rows.get_mut(0).unwrap() = w::ForgeRow::new(&w::CEILINGS, parts).unwrap();
    let root = f::forge_root(Address::Welcome { step: Step::Forge { forge: None } }, rows);
    let mut expected = String::from("Forgejo on this machine · connected as ");
    for part in [&login, " · ", &org, "/", &repository, " made"] {
        expected.push_str(part);
    }
    for view in [f::page_view(&root), f::browser_page_view(&root)] {
        node(view.tree(), Role::Paragraph, &expected);
    }
}

fn start_model(root: &mut d::Domain) {
    use pages::{accounts::Way, connect_model::Act as A};
    let way = Way::SignIn { provider: 17, plan: true };
    let act = A::SignIn { name: [1; 16], way };
    let attempt = super::connect_model::sent(root, act);
    super::connect_model::answer(
        root,
        attempt,
        jig_client::contract::DocumentOutcome::AccountSignInStarted {
            source: 43,
            address: b"https://issuer.example/visit".as_slice().into(),
        },
    );
}
fn has_style(tree: &skein_ui::Tree, style: skein_ui::Style) -> bool {
    tree.nodes().iter().any(|n| n.styles.as_slice().contains(&style))
}

#[test]
fn model_shapes_keep_the_forge_beside_the_column_in_both_shells() {
    use crate::look;
    use jig_client::contract as c;
    for (case, heading, panel, message) in [
        (0_u8, "Connect a model", false, "temper’s agents run on a plan you sign in to."),
        (
            1,
            "An OpenAI API key",
            true,
            "Nothing checks a key when it is added: a wrong one shows once OpenAI refuses a run.",
        ),
        (2, "Signing in to ChatGPT", true, "Your ChatGPT plan."),
        (3, "Connect a model", false, "ChatGPT’s sign-in"),
        (5, "Connect a model", false, "Finishing your sign-in…"),
        (
            6,
            "Your Claude subscription",
            true,
            "temper program-login --config CONFIG --data DATA --cache CACHE claude-pat",
        ),
        (7, "Signing in to ChatGPT", true, "You signed in without plan usage."),
        (8, "Signing in to ChatGPT", true, "Another person here holds that ChatGPT account."),
        (9, "Signing in to ChatGPT", true, "Needs sign-in"),
    ] {
        let kind = match case {
            1 => d::WayKind::Key,
            6 => d::WayKind::Program,
            _ => d::WayKind::SignIn,
        };
        let (mut root, watch) = model_with_forge(kind);
        let mut empty = signing_document();
        empty.signing.clear();
        super::accounts::deliver(&mut root, watch, empty);
        if case == 0 {
            f::root_event(
                &mut root,
                d::Event::Went { place: d::Place::Address(Address::Welcome { step: Step::Model { way: None } }) },
            );
        }
        if (3..=5).contains(&case) || case >= 7 {
            start_model(&mut root);
        }
        if case == 5 {
            f::root_event(
                &mut root,
                d::Event::SignInAddress { value: skein_lib::Secret::new(b"fixture".as_slice().into()) },
            );
            let attempt = super::connect_model::sent(&mut root, pages::connect_model::Act::Finish { name: [2; 16] });
            super::connect_model::answer(&mut root, attempt, c::DocumentOutcome::AccountFinishing { source: 43 });
        }
        if case >= 7 {
            let mut doc = super::accounts::card_document(b"openai", b"", c::AccountKind::SignIn);
            doc.usable = false;
            doc.accounts.clear();
            doc.signing.clear();
            doc.last_end = Some(c::AccountLastEnd {
                source: 43,
                end: match case {
                    7 => c::AccountEnd::Made { account: 1, declined: true },
                    8 => c::AccountEnd::Refused { refusal: c::AccountRefusal::Held { provider: 17 } },
                    _ => c::AccountEnd::Failed { failure: c::AccountFailure::SignInAgain },
                },
            });
            super::accounts::deliver(&mut root, watch, doc);
        }
        if case == 3 || case == 5 {
            super::accounts::deliver(&mut root, watch, signing_document());
        }
        for (browser, view) in [(false, f::page_view(&root)), (true, f::browser_page_view(&root))] {
            let tree = view.tree();
            model_pane(
                tree,
                if case == 3 || case == 5 { "Model ChatGPT plan · signing in…" } else { "Model none yet" },
            );
            node(tree, Role::Heading { level: 1 }, heading);
            assert_eq!(
                tree.nodes().iter().filter(|n| n.styles.as_slice().contains(&look::WELCOME_PANEL)).count(),
                usize::from(panel)
            );
            assert!(!has_style(tree, look::WELCOME_LEAD) || case == 0);
            assert!(has_style(tree, look::WELCOME_FOOT));
            let text = String::from_utf8_lossy(&f::text(tree)).into_owned();
            assert!(text.contains(message), "case {case}: {text}");
            assert!(f::find(tree, Role::Link, "‹ All ways".as_bytes()).is_none());
            if case == 3 || case == 5 {
                assert!(has_style(tree, look::WELCOME_AWAY));
                assert_eq!(has_style(tree, look::WELCOME_SPIN), browser);
                assert!(!text.contains("LLM account sign-in started"));
                assert!(!text.contains("Pending"));
                assert!(!text.contains("Mockup:"));
                assert!(f::find(tree, Role::Link, b"Open sign-in page").is_none());
                let mut print = skein_ui_text::Print::new(&view.limits().tree).unwrap();
                skein_ui_text::render(tree, b"temper", None, &mut print);
                assert!(String::from_utf8_lossy(print.text.as_slice()).contains(message));
            }
            if case == 3 {
                node(tree, Role::Disclosure, "Signed in, but the page couldn’t be reached?");
            }
        }
    }
}

#[test]
fn model_usable_replaces_every_way_and_continues_to_ready_in_both_shells() {
    use crate::look;
    use jig_client::contract as c;
    for kind in [None, Some(d::WayKind::SignIn), Some(d::WayKind::Key), Some(d::WayKind::Program)] {
        let (mut root, watch) = model_with_forge(kind.unwrap_or(d::WayKind::SignIn));
        if kind.is_none() {
            f::root_event(
                &mut root,
                d::Event::Went { place: d::Place::Address(Address::Welcome { step: Step::Model { way: None } }) },
            );
        }
        let mut doc = super::accounts::card_document(b"openai", b"", c::AccountKind::SignIn);
        doc.usable = true;
        super::accounts::deliver(&mut root, watch, doc);
        for mut view in [f::page_view(&root), f::browser_page_view(&root)] {
            let tree = view.tree();
            model_pane(tree, "ChatGPT plan · trying: ChatGPT answers");
            node(tree, Role::Heading { level: 1 }, "Connect a model");
            node(tree, Role::Status, "ChatGPT plan");
            assert!(has_style(tree, look::FLOW_DONE));
            assert!(!has_style(tree, look::WELCOME_PANEL));
            assert!(!has_style(tree, look::WELCOME_LEAD));
            assert!(!has_style(tree, look::MODEL_CHOICES));
            assert!(!has_style(tree, look::MODEL_FACTS));
            assert!(String::from_utf8_lossy(&f::text(tree)).contains("✓"));
            node(tree, Role::Complementary, "Forge and model");
            for absent in ["Continue with ChatGPT", "Manage usage", "All ways"] {
                assert!(f::find(tree, Role::Button, absent.as_bytes()).is_none());
                assert!(f::find(tree, Role::Link, absent.as_bytes()).is_none());
            }
            let id = node(tree, Role::Link, "Continue");
            assert_eq!(
                crate::act(&mut view, id, Act::Press),
                Some(d::Action::Pages(pages::Act::Go { address: Address::Welcome { step: Step::Ready } }))
            );
        }
    }
}

#[test]
fn model_panels_enable_the_primary_only_after_a_secret_is_written() {
    for paste in [false, true] {
        let (mut root, _) = model_with_forge(if paste { d::WayKind::SignIn } else { d::WayKind::Key });
        if paste {
            start_model(&mut root);
        }
        let field = if paste { "Address of the page that couldn’t be reached" } else { "Key, pasted once" };
        let primary = if paste { "Finish connecting" } else { "Add key" };
        for written in [false, true] {
            if written {
                f::root_event(
                    &mut root,
                    if paste {
                        d::Event::SignInAddress { value: skein_lib::Secret::new(b"fixture".as_slice().into()) }
                    } else {
                        d::Event::AccountKey { value: skein_lib::Secret::new(b"fixture".as_slice().into()) }
                    },
                );
            }
            for (browser, mut view) in [(false, f::page_view(&root)), (true, f::browser_page_view(&root))] {
                let tree = view.tree();
                let field = node(tree, Role::TextField, field);
                let entry = tree.nodes().iter().find(|n| n.id == field).unwrap();
                assert_eq!(entry.entry, Some(skein_ui::EntryKind::Secret));
                assert_eq!(entry.value.as_ref().unwrap().text.as_ref(), b"");
                let button = node(tree, Role::Button, primary);
                assert_eq!(tree.disabled(button), !written);
                assert!(
                    tree.nodes()
                        .iter()
                        .find(|n| n.id == button)
                        .unwrap()
                        .styles
                        .as_slice()
                        .contains(&crate::look::FLOW_PRIMARY)
                );
                assert!(
                    !tree.nodes().iter().find(|n| n.id == button).unwrap().states.as_slice().contains(&State::Pressed)
                );
                if !paste {
                    let back = node(tree, Role::Link, "‹ Back");
                    if !browser {
                        f::terminal_command(tree, &view.limits().tree, back, "‹ Back".as_bytes(), primary.as_bytes());
                    }
                    assert_eq!(
                        crate::act(&mut view, back, Act::Press),
                        Some(d::Action::Pages(pages::Act::Go {
                            address: Address::Welcome { step: Step::Model { way: None } }
                        }))
                    );
                }
            }
        }
    }
}

#[test]
fn model_paste_refusals_and_endings_keep_their_words() {
    use c::SignInRedirectWhy as R;
    use jig_client::contract as c;
    for (case, message, field) in [
        (0_u8, "That's longer than a sign-in's address. Copy the address bar alone.", true),
        (
            1,
            "That isn't this sign-in's address. Copy the whole address of the page that couldn't be reached, from the address bar.",
            true,
        ),
        (2, "That address is from an earlier sign-in. Paste the one from this sign-in's tab, or start again.", true),
        (3, "That's too many addresses from other sign-ins. Continue with ChatGPT to start again.", false),
        (4, "This sign-in has ended.", false),
        (5, "That address wasn't sent. Paste it again.", true),
        (6, "That address wasn't sent. Paste it again.", true),
        (7, "This sign-in has ended.", false),
    ] {
        let (mut root, watch) = model_with_forge(d::WayKind::SignIn);
        start_model(&mut root);
        f::root_event(
            &mut root,
            d::Event::SignInAddress {
                value: skein_lib::Secret::new(if case == 0 {
                    Box::from([b'x'; 4097])
                } else {
                    b"fixture".as_slice().into()
                }),
            },
        );
        if case != 0 && case != 7 {
            let attempt = super::connect_model::sent(&mut root, pages::connect_model::Act::Finish { name: [2; 16] });
            let answer = match case {
                1..=4 => Some(jig_client::Answered::Jig(c::Answer::Refused {
                    refusal: c::Refusal::SignInRedirect {
                        why: match case {
                            1 => R::Address,
                            2 => R::State,
                            3 => R::TooMany,
                            _ => R::NotWaiting,
                        },
                    },
                    at: None,
                })),
                _ => None,
            };
            f::root_event(
                &mut root,
                d::Event::Client {
                    event: jig_client::Event::Up {
                        up: jig_client::Up::Answer {
                            attempt,
                            answer,
                            why: None,
                            envelope: if case == 5 {
                                skein_lib::RequestEnvelope::Lost
                            } else {
                                skein_lib::RequestEnvelope::Final
                            },
                        },
                    },
                },
            );
        }
        if case == 7 {
            let mut doc = super::accounts::card_document(b"openai", b"", c::AccountKind::SignIn);
            doc.usable = false;
            doc.accounts.clear();
            doc.signing.clear();
            doc.signing
                .push(c::AccountSignIn { source: 43, provider: 17, holder: c::AccountHolder::Party(73) })
                .unwrap();
            super::accounts::deliver(&mut root, watch, doc.clone());
            doc.signing.clear();
            doc.cut = false;
            super::accounts::deliver(&mut root, watch, doc);
        }
        for view in [f::page_view(&root), f::browser_page_view(&root)] {
            let tree = view.tree();
            node(tree, Role::Heading { level: 1 }, if case == 7 { "Signing in to ChatGPT" } else { "Connect a model" });
            let inside = tree.nodes();
            assert!(inside.iter().any(|n| n.name.as_deref() == Some(message.as_bytes())), "case {case}: {message}");
            assert_eq!(inside.iter().any(|n| n.role == Role::TextField), field);
            if !field {
                node(tree, Role::Button, "Continue with ChatGPT");
            }
            node(tree, Role::Complementary, "Forge and model");
        }
    }
}

#[test]
#[expect(clippy::too_many_lines, reason = "one sequence checks Claude’s four frames and standalone in both shells")]
fn welcome_claude_states_keep_the_command_and_check_in_both_shells() {
    use jig_client::contract as c;
    use pages::{accounts::Way, connect_model::Act as A};
    for frame in 0_u8..4 {
        let (mut root, watch) = model_with_forge(d::WayKind::Program);
        let mut doc = pages::accounts::document(root.client()).unwrap().clone();
        doc.usable = false;
        doc.settled = false;
        f::root_event(
            &mut root,
            d::Event::Client {
                event: jig_client::Event::Up {
                    up: jig_client::Up::Watched {
                        watch,
                        event: c::WatchEvent::Delivered(c::Delivery::Accounts(Box::new(doc.clone()))),
                    },
                },
            },
        );
        if frame > 0 {
            let attempt = super::connect_model::sent(
                &mut root,
                A::SignedIn { name: [1; 16], way: Way::Program { provider: 17 } },
            );
            f::root_event(
                &mut root,
                d::Event::Went { place: d::Place::Address(Address::Welcome { step: Step::Ready }) },
            );
            if frame > 1 {
                let limits = f::limits();
                let mut out = skein_lib::Queue::uncapped();
                d::step(
                    &mut root,
                    &skein_lib::Env { now: skein_lib::Time::ZERO, wall: f::NOW, limits },
                    d::Event::Client {
                        event: jig_client::Event::Up {
                            up: jig_client::Up::Answer {
                                attempt,
                                envelope: skein_lib::RequestEnvelope::Final,
                                why: None,
                                answer: Some(jig_client::Answered::Jig(c::Answer::Done(c::Outcome::Document(
                                    c::DocumentOutcome::AccountProgramDeclared {
                                        source: 43,
                                        name: b"claude-pat".as_slice().into(),
                                    },
                                )))),
                            },
                        },
                    },
                    &mut out,
                );
                if frame == 3 {
                    d::step(
                        &mut root,
                        &skein_lib::Env { now: skein_lib::Time::ZERO, wall: f::NOW, limits },
                        d::Event::Client { event: jig_client::Event::Stored },
                        &mut out,
                    );
                    let attempt = out
                        .iter()
                        .find_map(|request| match request {
                            d::Request::Unlinked {
                                request:
                                    jig_client::UnlinkedRequest::Down { down: jig_client::Down::Send { attempt, .. } },
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
                            | d::Request::Push(_)
                            | d::Request::Fact { .. } => None,
                        })
                        .expect("chained check sent");
                    super::connect_model::answer(&mut root, attempt, c::DocumentOutcome::AccountChanged);
                    doc.settled = true;
                    f::root_event(
                        &mut root,
                        d::Event::Client {
                            event: jig_client::Event::Up {
                                up: jig_client::Up::Watched {
                                    watch,
                                    event: c::WatchEvent::Delivered(c::Delivery::Accounts(Box::new(doc))),
                                },
                            },
                        },
                    );
                }
            }
        }
        for view in [f::page_view(&root), f::browser_page_view(&root)] {
            let tree = view.tree();
            node(
                tree,
                Role::Heading { level: 1 },
                if frame == 0 {
                    "Your Claude subscription"
                } else if frame == 3 {
                    "Almost ready"
                } else {
                    "Ready"
                },
            );
            assert!(f::find(tree, Role::TextField, b"Name").is_none(), "the default has no field");
            if frame == 1 || frame == 2 {
                node(tree, Role::Paragraph, "Claude subscription · trying: Signed in to your subscription");
                assert!(f::find(tree, Role::Button, "I’ve signed in".as_bytes()).is_none());
                assert!(!has_style(tree, crate::look::WELCOME_PANEL));
                assert!(has_style(tree, crate::look::WELCOME_STATUS));
            } else if frame == 3 {
                node(tree, Role::Paragraph, "Claude subscription · Claude didn’t answer");
                node(tree, Role::Paragraph, "Claude Code isn’t signed in here yet. Sign in again, then try again.");
                node(tree, Role::Button, "Try again");
                node(tree, Role::Link, "Sign in again");
                assert!(!has_style(tree, crate::look::WELCOME_CMD));
                assert!(!has_style(tree, crate::look::WELCOME_PANEL));
                assert!(!String::from_utf8_lossy(&f::text(tree)).contains("temper program-login"));
            } else {
                assert!(
                    String::from_utf8_lossy(&f::text(tree))
                        .contains("temper program-login --config CONFIG --data DATA --cache CACHE claude-pat")
                );
                assert!(has_style(tree, crate::look::WELCOME_CMD));
                node(tree, Role::Button, "I’ve signed in");
                node(tree, Role::Link, "‹ Back");
            }
        }
        let browser = f::browser_page_view(&root);
        assert_eq!(has_style(browser.tree(), crate::look::FLOW_BUSY), frame == 1 || frame == 2);
        let terminal = f::page_view(&root);
        assert!(!has_style(terminal.tree(), crate::look::WELCOME_SPIN));
        if frame == 2 {
            f::root_event(
                &mut root,
                d::Event::Went {
                    place: d::Place::Address(Address::ConnectModel {
                        way: Some(pages::connect_model::model_way(Way::Program { provider: 17 })),
                    }),
                },
            );
            let standalone = f::page_view(&root);
            node(standalone.tree(), Role::TextField, "Name");
            assert!(!String::from_utf8_lossy(&f::text(standalone.tree())).contains("Declared your Claude"));
        }
    }
}

#[test]
fn ready_orders_forge_then_model_for_every_readiness_combination() {
    for step in [Step::Ready, Step::Model { way: None }] {
        for forge_done in [false, true] {
            for model_done in [false, true] {
                let rows = forge::first_rows(
                    if forge_done { forge::connected() } else { w::Connection::Connecting },
                    if forge_done { forge::made() } else { w::FirstState::Making },
                );
                let (mut root, watch) = f::forge_root_accounts(Address::Welcome { step }, rows);
                let mut doc = super::accounts::card_document(b"openai", b"", jig_client::contract::AccountKind::SignIn);
                doc.usable = model_done;
                doc.signing.clear();
                if !model_done {
                    doc.accounts.clear();
                }
                super::accounts::deliver(&mut root, watch, doc);
                for view in [f::page_view(&root), f::browser_page_view(&root)] {
                    let tree = view.tree();
                    let model_name = if model_done { "ChatGPT plan" } else { "Model none yet" };
                    if step == Step::Ready {
                        forge::words_in_order(tree, &["Your first project", "Forgejo on this machine", model_name]);
                    }
                    assert_eq!(
                        f::find(tree, Role::Link, b"Connect a model").is_some(),
                        step == Step::Ready && !model_done
                    );
                    let lines = pane_lines(tree);
                    let [forge, model] = lines.as_slice() else { panic!("pane has Forge and Model lines") };
                    assert!(forge.name.as_deref().unwrap().starts_with(b"Forgejo on this machine"));
                    assert!(model.name.as_deref().unwrap().starts_with(model_name.as_bytes()));
                    assert_eq!(forge.styles.as_slice().contains(&crate::look::FLOW_DONE), forge_done);
                    assert!(!model.styles.as_slice().contains(&crate::look::FLOW_DONE));
                    assert_eq!(tree.nodes().iter().any(|n| n.role == Role::Disclosure), model_done || !forge_done);
                    let printed = terminal_print(&view);
                    let printed_pane = printed.split("Forge and model:\n").nth(1).unwrap();
                    assert!(printed_pane.find("Forgejo").unwrap() < printed_pane.find(model_name).unwrap());
                }
            }
        }
    }
}

#[test]
fn ready_names_a_programs_login_and_a_keys_billing() {
    use jig_client::contract::AccountKind as K;
    for (provider, display, kind, email, expected) in [
        (
            b"anthropic".as_slice(),
            b"claude-owner".as_slice(),
            K::Program,
            None,
            "Claude subscription · through Claude Code, signed in",
        ),
        (
            b"anthropic",
            b"claude-owner",
            K::Program,
            Some(b"alice@example.com".as_slice()),
            "Claude subscription · through Claude Code, signed in · alice@example.com",
        ),
        (b"openai", b"", K::Key, None, "OpenAI API key · billed to the key’s organization"),
        (b"openai", b"", K::SignIn, None, "ChatGPT plan"),
        (b"openai", b"Legacy", K::SignIn, None, "ChatGPT plan · Legacy"),
        (b"openai", b"", K::SignIn, Some(b"alice@example.com".as_slice()), "ChatGPT plan · alice@example.com"),
        (
            b"openai",
            b"Different display",
            K::SignIn,
            Some(b"alice@example.com".as_slice()),
            "ChatGPT plan · alice@example.com",
        ),
    ] {
        let (mut root, watch) = ready_model_fixture();
        let mut doc = super::accounts::card_document(provider, display, kind);
        doc.accounts.get_mut(0).unwrap().facts = super::accounts::email_facts(email);
        super::accounts::deliver(&mut root, watch, doc);
        for (terminal, view) in [(true, f::page_view(&root)), (false, f::browser_page_view(&root))] {
            node(view.tree(), Role::Paragraph, expected);
            if terminal {
                assert!(terminal_print(&view).contains(expected));
            }
        }
    }
}

#[test]
fn ready_none_keeps_the_forge_link_on_its_line() {
    let (mut root, watch) =
        f::forge_root_accounts(Address::Welcome { step: Step::Ready }, skein_lib::List::with_capacity(0));
    let mut doc = signing_document();
    doc.signing.clear();
    super::accounts::deliver(&mut root, watch, doc);
    for mut view in [f::page_view(&root), f::browser_page_view(&root)] {
        let tree = view.tree();
        let summary = node(tree, Role::Slot, "Forge none yet");
        assert!(styled(tree, summary, crate::look::ACCOUNT_LINE));
        let link = node(tree, Role::Link, "Connect a forge");
        assert_eq!(forge::parent(tree, link), Some(summary));
        let printed = terminal_print(&view);
        assert!(printed.contains("Forge none yet · "), "{printed}");
        assert_eq!(
            crate::act(&mut view, link, Act::Press),
            Some(d::Action::Pages(pages::Act::Go { address: Address::Welcome { step: Step::Forge { forge: None } } }))
        );
    }
}

#[test]
fn ready_names_the_forge_and_only_a_made_first_repository() {
    for (rows, expected, underway) in [
        (
            forge::rows(forge::connected(), w::Offer::Offered),
            "Forgejo on this machine · Connected as temper (42)",
            false,
        ),
        (
            forge::first_rows(forge::connected(), forge::made()),
            "Forgejo on this machine · connected as temper · work/hello made",
            false,
        ),
        (
            forge::first_rows(forge::connected(), w::FirstState::Making),
            "Forgejo on this machine · work/hello · Connected as temper (42)",
            true,
        ),
    ] {
        let root = ready_root(rows);
        for words in [f::words(), f::browser_words()] {
            let terminal = words.shell == crate::words::Shell::Terminal;
            let mut view = f::view_of(words);
            let mut facts = crate::forge_sign_in::Forges::new();
            facts.read("3 forgejo http://127.0.0.1:3000 owned");
            view.forge_sign_in(facts);
            f::build(&mut view, &root);
            let tree = view.tree();
            let summary = node(tree, Role::Paragraph, expected);
            assert_eq!(tree.nodes().iter().filter(|n| n.role == Role::Disclosure).count(), 1 + usize::from(underway));
            let link = node(tree, Role::Link, "Open Forgejo ↗");
            assert_eq!(forge::parent(tree, summary), forge::parent(tree, link));
            assert!(link > summary);
            if terminal {
                assert!(terminal_print(&view).contains(expected));
            }
        }
    }
}

#[test]
fn ready_setup_note_follows_the_form_only_while_underway() {
    let failed = forge::failure_cases().into_iter().next().unwrap().0;
    for (rows, noted) in [
        (skein_lib::List::with_capacity(0), false),
        (forge::first_rows(w::Connection::Connecting, w::FirstState::Making), true),
        (forge::first_rows(forge::connected(), w::FirstState::Making), false),
        (forge::first_rows(forge::connected(), forge::made()), false),
        (failed, false),
    ] {
        let root = ready_root(rows);
        for view in [f::page_view(&root), f::browser_page_view(&root)] {
            let tree = view.tree();
            let note = f::find(tree, Role::Paragraph, b"Forgejo is still setting up; this page moves on by itself.");
            assert_eq!(note.is_some(), noted);
            if let Some(note) = note {
                assert!(styled(tree, note, crate::look::FLOW_FACTS));
                let form = node(tree, Role::Form, "Your first project");
                assert_eq!(forge::parent(tree, note), forge::parent(tree, form));
                assert!(note > form);
            }
        }
    }
}

#[test]
fn welcome_sheet_alerts_are_above_the_foot_in_both_shells() {
    use d::forge::{
        Act as A,
        forges::{Act as F, SetupField, Start},
    };
    let seventeen = b"a/b\n".repeat(17);
    let long = b"?".repeat(256);
    for (choice, action, message) in [
        (
            Start::Empty,
            A::SetupWrite { field: SetupField::Organization, text: b"".as_slice().into() },
            "Both an organization and a name.".to_owned(),
        ),
        (
            Start::Empty,
            A::SetupWrite { field: SetupField::Repository, text: b"".as_slice().into() },
            "Both an organization and a name.".to_owned(),
        ),
        (
            Start::Copies,
            A::CopiesWrite { text: b" \n".as_slice().into() },
            "At least one address, as github.com/owner/name.".to_owned(),
        ),
        (
            Start::Copies,
            A::CopiesWrite { text: b" bad/address/extra/part ".as_slice().into() },
            "Not an address: bad/address/extra/part. Write it as github.com/owner/name.".to_owned(),
        ),
        (Start::Copies, A::CopiesWrite { text: seventeen.into() }, "At most 16 addresses.".to_owned()),
        (
            Start::Copies,
            A::CopiesWrite { text: long.into() },
            [
                "Not an address: ",
                &"?".repeat(usize::try_from(f::words().name_bytes).unwrap()),
                ". Write it as github.com/owner/name.",
            ]
            .concat(),
        ),
    ] {
        let mut root = sheet_root();
        start(&mut root, choice);
        for action in [action, A::Forges(F::RunOwned)] {
            f::root_event(&mut root, d::Event::Act { action: d::Action::Forge(action) });
        }
        for view in [f::page_view(&root), f::browser_page_view(&root)] {
            let nodes = sheet(view.tree());
            assert_eq!(named(nodes, Role::Alert), [message.as_bytes()]);
            let alert = nodes.iter().position(|n| n.role == Role::Alert).unwrap();
            let cancel = nodes.iter().position(|n| n.name.as_deref() == Some(b"Cancel")).unwrap();
            assert!(alert < cancel);
            assert!(terminal_print(&view).contains(&message));
        }
    }
}

#[test]
fn ready_copies_name_the_project_and_lead_in_both_shells() {
    for (addresses, lead) in [
        (
            [b"https://github.com/acme/widget".as_slice()].as_slice(),
            "Everything is connected. Create your first project, with acme/widget in it, and start working.",
        ),
        (
            [b"https://github.com/acme/widget".as_slice(), b"https://source.invalid/team/tool".as_slice()].as_slice(),
            "Everything is connected. Create your first project, with 2 repositories in it, and start working.",
        ),
    ] {
        let mut rows = owned_adopted_rows();
        let mut parts = rows.get(0).unwrap().clone().into_parts();
        parts.first = None;
        parts.repositories.clear();
        let mut copies = skein_lib::List::with_capacity(temper_client_protocol_wire::FIRST_COPIES);
        for address in addresses {
            copies.push((*address).into()).unwrap();
        }
        parts.starts_with = w::StartsWith::Copies(
            w::CopyAddresses::new(&w::CEILINGS, w::CopyAddressesParts { addresses: copies }).unwrap(),
        );
        *rows.get_mut(0).unwrap() = w::ForgeRow::new(&w::CEILINGS, parts).unwrap();
        let root = ready_root(rows);
        for view in [f::page_view(&root), f::browser_page_view(&root)] {
            node(view.tree(), Role::Paragraph, lead);
            node(view.tree(), Role::Paragraph, "Forgejo on this machine · Connected as temper (42)");
            let field = view
                .tree()
                .nodes()
                .iter()
                .find(|node| node.role == Role::TextField && node.name.as_deref() == Some(b"Your first project"))
                .unwrap();
            assert_eq!(field.value.as_ref().unwrap().text.as_ref(), b"Widget");
            let text = String::from_utf8_lossy(&f::text(view.tree())).into_owned();
            assert!(!text.contains(" made"));
            assert!(!text.contains("Copying"));
        }
    }
}

#[test]
fn the_command_shows_directory_placeholders_without_words() {
    let (root, _) = model_with_forge(d::WayKind::Program);
    assert_eq!(
        super::connect_model::command_line(&root),
        b"temper program-login --config CONFIG --data DATA --cache CACHE claude-pat"
    );
}
#[test]
fn the_command_takes_the_words_first() {
    let (mut root, _) = model_with_forge(d::WayKind::Program);
    super::connect_model::answer_environment(&mut root, super::connect_model::ENVIRONMENT);
    assert_eq!(
        super::connect_model::command_line(&root),
        b"TEMPER_CONFIG='/config space' TEMPER_DATA='/data' TEMPER_CACHE='/cache' temper program-login claude-pat"
    );
}

#[test]
fn welcome_waiting_orders_the_note_link_and_folded_paste_in_both_shells() {
    let (mut root, _) = model_with_forge(d::WayKind::SignIn);
    start_model(&mut root);
    super::connect_model::assert_waiting(&root);
}

#[test]
fn ready_keeps_declare_and_check_refusals_under_models_line() {
    use jig_client::contract as c;
    use pages::{accounts::Way, connect_model::Act as A};
    for (checking, expected) in [
        (false, "This provider offers you no program login. Choose another way to connect."),
        (true, "This act is not offered"),
    ] {
        let (mut root, watch) = model_with_forge(d::WayKind::Program);
        let mut doc = super::accounts::card_document(b"anthropic", b"claude-pat", c::AccountKind::Program);
        doc.signing.clear();
        doc.cut = false;
        doc.usable = false;
        doc.settled = false;
        if !checking {
            doc.accounts.clear();
        }
        super::accounts::deliver(&mut root, watch, doc);
        let attempt =
            super::connect_model::sent(&mut root, A::SignedIn { name: [1; 16], way: Way::Program { provider: 17 } });
        let checking_now = match root.pages().connect_model().unwrap().check {
            pages::connect_model::ProgramCheck::Checking { .. } => true,
            pages::connect_model::ProgramCheck::Idle
            | pages::connect_model::ProgramCheck::Declaring
            | pages::connect_model::ProgramCheck::NotYet => false,
        };
        assert_eq!(checking_now, checking);
        f::root_event(&mut root, d::Event::Went { place: d::Place::Address(Address::Welcome { step: Step::Ready }) });
        f::root_event(
            &mut root,
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
        );
        for view in [f::page_view(&root), f::browser_page_view(&root)] {
            let tree = view.tree();
            let pane = node(tree, Role::Complementary, "Forge and model");
            let model = if checking {
                node(tree, Role::Paragraph, "Claude subscription · trying: Claude Code starts on this machine")
            } else {
                node(tree, Role::Slot, "Model none yet")
            };
            let refusal = node(tree, Role::Status, expected);
            assert_eq!(forge::parent(tree, model), Some(pane));
            assert!(refusal > model);
            assert_eq!(forge::parent(tree, refusal), Some(pane));
        }
    }
}

#[test]
fn welcome_prints_the_deployments_program_command_or_the_placeholders() {
    for line in [
        None,
        Some(b"'/box/temper' program-login --config '/box/config' --data '/box/data' --cache '/box/cache'".as_slice()),
    ] {
        let (mut root, watch) = super::connect_model::program_root_with_line(model_address(d::WayKind::Program), line);
        let mut doc = super::accounts::document(0);
        doc.usable = false;
        doc.settled = true;
        super::accounts::deliver(&mut root, watch, doc);
        for view in [f::page_view(&root), f::browser_page_view(&root)] {
            let shown = String::from_utf8(f::text(view.tree()).into_vec()).unwrap();
            let expected = match line {
                Some(_) => {
                    "On this machine, run: '/box/temper' program-login --config '/box/config' --data '/box/data' --cache '/box/cache' claude-pat"
                }
                None => {
                    "On each host, with its directories: temper program-login --config CONFIG --data DATA --cache CACHE claude-pat"
                }
            };
            assert!(shown.contains(expected), "{shown}");
        }
    }
}

pub(super) use crate::parts_tests::welcome::{first_run_destination, ready_root};
