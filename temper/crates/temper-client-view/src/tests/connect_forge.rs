#![allow(clippy::disallowed_types, clippy::disallowed_methods, reason = "test assertions inspect the semantic tree")]
use super::fixtures as f;
use skein_lib::List;
use skein_ui::{Act, Role, State};
use temper_client_domain::{self as d, Address, Choice, Step, pages};
use temper_client_protocol_wire::v1 as w;

pub(super) use super::fixtures::connect_forge::{
    GITHUB, HERE, act, answer, answer_event, attempt, connected, deliver, drive, failed_progress, failure_cases,
    first_rows, github_rows, made, node, parent, pinned_rows, reread_deployment, row, rows, words_in_order,
};

fn root(choice: Option<Choice>, connection: w::Connection, offer: w::Offer) -> d::Domain {
    f::forge_root(Address::ConnectForge { forge: choice }, rows(connection, offer))
}
fn here(connection: w::Connection) -> d::Domain {
    root(Some(Choice::Forge(HERE)), connection, w::Offer::Offered)
}

fn press(root: &d::Domain, role: Role, name: &[u8]) -> d::Action {
    let mut view = f::page_view(root);
    let id = node(view.tree(), role, name);
    crate::act(&mut view, id, if role == Role::Choice { Act::Toggle } else { Act::Press }).expect("bound control")
}

#[test]
fn connect_offers_the_three_kinds() {
    let root = f::forge_root(Address::ConnectForge { forge: None }, List::with_capacity(0));
    let view = f::page_view(&root);
    words_in_order(
        view.tree(),
        &[
            "Connect a forge",
            "Where your code lives",
            "Forgejo",
            "Run Forgejo here",
            "GitHub",
            "Connect GitHub",
            "Or a server of your own",
            "Your Forgejo",
            "Connect your Forgejo",
        ],
    );
    assert!(f::find(view.tree(), Role::List, b"Steps").is_none());
    let card = view.tree().find(node(view.tree(), Role::Region, b"Forgejo")).unwrap();
    assert!(card.styles.as_slice().contains(&crate::look::CONNECT_DEFAULT));
    assert_eq!(
        press(&root, Role::Link, b"Connect your Forgejo"),
        d::Action::Pages(pages::Act::Go { address: Address::ConnectForge { forge: Some(Choice::Elsewhere) } })
    );
    assert_eq!(
        press(&root, Role::Link, b"Run Forgejo here"),
        d::Action::Pages(pages::Act::Go { address: Address::ConnectForge { forge: Some(Choice::Owned) } })
    );
}

fn deployment_root(address: Address, owned: bool) -> d::Domain {
    let (mut root, me, _, _, _, _) =
        f::root_at_with_forges(address, &f::domain_limits(), b"Shop", None, None, Some(List::with_capacity(0)));
    reread_deployment(&mut root, me, owned);
    root
}

#[test]
fn connect_offers_forgejo_on_this_machine_from_the_deployment() {
    for owned in [false, true] {
        let root = deployment_root(Address::ConnectForge { forge: None }, owned);
        for view in [f::page_view(&root), f::browser_page_view(&root)] {
            assert_eq!(f::find(view.tree(), Role::Heading { level: 2 }, b"Forgejo").is_some(), owned);
            node(view.tree(), Role::Button, b"Connect GitHub");
            node(view.tree(), Role::Link, b"Connect your Forgejo");
        }
    }
}
#[test]
fn welcome_offers_run_on_this_machine_from_the_deployment() {
    for owned in [false, true] {
        let root = deployment_root(Address::Welcome { step: Step::Forge { forge: None } }, owned);
        for view in [f::page_view(&root), f::browser_page_view(&root)] {
            assert_eq!(f::find(view.tree(), Role::Link, b"Run on this machine").is_some(), owned);
            node(view.tree(), Role::Button, b"Connect GitHub");
        }
    }
}

#[test]
fn github_cards_offer_connect_github_alone_in_both_flows() {
    for address in [Address::ConnectForge { forge: None }, Address::Welcome { step: Step::Forge { forge: None } }] {
        let root = f::forge_root(address, List::with_capacity(0));
        let view = f::browser_page_view(&root);
        let tree = view.tree();
        let card = node(tree, Role::Region, b"GitHub");
        let start = tree.nodes().iter().position(|n| n.id == card).unwrap();
        let end = start + usize::try_from(tree.nodes()[start].size).unwrap();
        let actions: Vec<_> = tree.nodes()[start..end]
            .iter()
            .filter(|n| n.role == Role::Button || n.role == Role::Link)
            .map(|n| n.name.as_deref().unwrap())
            .collect();
        assert_eq!(actions, [b"Connect GitHub".as_slice()]);
    }
}
#[test]
fn github_details_offer_only_the_app_of_ones_own_with_a_masked_key() {
    let root = root(Some(Choice::Forge(GITHUB)), w::Connection::NotSetUp, w::Offer::Offered);
    let view = f::browser_page_view(&root);
    let words = String::from_utf8_lossy(&f::text(view.tree())).into_owned();
    assert!(!words.contains("organization"));
    assert!(!words.contains("Connect GitHub"));
    assert!(!words.contains("not available yet"));
    node(view.tree(), Role::Heading { level: 2 }, b"An App of your own");
    let button = node(view.tree(), Role::Button, b"Connect the App");
    assert!(!view.tree().nodes().iter().find(|n| n.id == button).unwrap().states.as_slice().contains(&State::Disabled));
    node(view.tree(), Role::TextField, b"App id");
    let key = node(view.tree(), Role::TextField, b"Private key");
    assert_eq!(view.tree().nodes().iter().find(|n| n.id == key).unwrap().entry, Some(skein_ui::EntryKind::Secret));
}
#[test]
fn the_terminal_shows_ctl_or_the_web_page_instead_of_the_key_field() {
    let view = f::page_view(&root(Some(Choice::Forge(GITHUB)), w::Connection::NotSetUp, w::Offer::Offered));
    assert!(f::find(view.tree(), Role::TextField, b"App id").is_none());
    assert!(f::find(view.tree(), Role::TextField, b"Private key").is_none());
    assert!(f::find(view.tree(), Role::Button, b"Connect the App").is_none());
    words_in_order(
        view.tree(),
        &[
            "Paste the key on this page in a browser, or run temper ctl DIR forge connect 2 github-app ID FILE --key KEY, FILE the key's file, readable by you alone.",
        ],
    );
}

#[test]
fn forgejo_here_details_say_the_first_repository_and_set_it_up() {
    let mut root = here(w::Connection::NotSetUp);
    let view = f::page_view(&root);
    words_in_order(
        view.tree(),
        &[
            "Forgejo on this machine",
            "temper box downloads Forgejo and its runner, starts them, and makes your first repository, with a README and a CI workflow. No restart.",
            "Its CI jobs run on this machine, as you, with no isolation.",
            "Forgejo stops when temper box stops. Its repositories are kept in temper’s data directory.",
        ],
    );
    assert!(view.tree().nodes().iter().any(|n| n.role == Role::TextField));
    let action = press(&root, Role::Button, b"Set up a Forgejo here");
    assert_eq!(action, d::Action::Forge(d::forge::Act::SetUp { forge: HERE }));
    f::root_event(&mut root, d::Event::Act { action });
    let view = f::page_view(&root);
    let id = node(view.tree(), Role::Button, b"Set up a Forgejo here");
    assert!(view.tree().nodes().iter().find(|n| n.id == id).unwrap().states.as_slice().contains(&State::Disabled));
}
#[test]
fn forgejo_here_without_the_box_says_to_start_temper_with_the_box() {
    let view = f::page_view(&root(Some(Choice::Forge(HERE)), w::Connection::NotSetUp, w::Offer::NotUnderTheBox));
    node(view.tree(), Role::Heading { level: 2 }, b"Set it up under temper box");
    let details =
        view.tree().nodes().iter().find(|n| n.styles.as_slice().contains(&crate::look::FLOW_DETAILS)).unwrap();
    let paragraphs: Vec<_> = view
        .tree()
        .nodes()
        .iter()
        .enumerate()
        .filter(|(_, n)| n.role == Role::Paragraph && parent(view.tree(), n.id) == Some(details.id))
        .collect();
    assert_eq!(paragraphs.len(), 1);
    let (index, paragraph) = paragraphs[0];
    assert_eq!(
        f::node_text(view.tree(), index).as_ref(),
        b"temper was not started by temper box, so it does not start a Forgejo itself. Start temper with temper box to set one up here."
    );
    assert!(paragraph.text.iter().all(|run| run.run == skein_ui::Run::Plain));
    assert!(f::find(view.tree(), Role::Button, b"Set up a Forgejo here").is_none());
    assert!(
        !view.tree().nodes().iter().any(|n| n.role == Role::Button && parent(view.tree(), n.id) == Some(details.id))
    );
    assert!(!view.tree().nodes().iter().any(|n| n.styles.as_slice().contains(&crate::look::FLOW_CODE)));
}
#[test]
fn settings_opens_the_welcome_sheet_and_cancel_returns_to_choose() {
    let root = f::forge_root(Address::ConnectForge { forge: Some(Choice::Owned) }, List::with_capacity(0));
    for view in [f::page_view(&root), f::browser_page_view(&root)] {
        let tree = view.tree();
        node(tree, Role::Dialog, b"Forgejo on this machine");
        node(tree, Role::Link, b"Run Forgejo here");
        node(tree, Role::Choice, b"An empty repository");
        node(tree, Role::Button, b"Run on this machine");
        assert!(f::find(tree, Role::List, b"Steps").is_none());
    }
    assert_eq!(
        press(&root, Role::Button, b"Cancel"),
        d::Action::Pages(pages::Act::Go { address: Address::ConnectForge { forge: None } })
    );
}

#[test]
fn connected_cards_move_the_default_and_offer_forgejos_sign_in() {
    let root = f::forge_root(Address::ConnectForge { forge: None }, first_rows(connected(), made()));
    let mut view = f::view_of(f::browser_words());
    let mut facts = crate::forge_sign_in::Forges::new();
    facts.read("3 forgejo https://forgejo.example owned");
    view.forge_sign_in(facts);
    let tree = f::build(&mut view, &root);
    words_in_order(tree, &["Forgejo", "✓ Connected", "work/hello", "Open Forgejo ↗", "GitHub", "Connect GitHub"]);
    let owned = tree.find(node(tree, Role::Region, b"Forgejo")).unwrap();
    assert!(!owned.styles.as_slice().contains(&crate::look::CONNECT_DEFAULT));
    let github = tree.find(node(tree, Role::Region, b"GitHub")).unwrap();
    assert!(github.styles.as_slice().contains(&crate::look::CONNECT_DEFAULT));
    let open = tree.find(node(tree, Role::Link, "Open Forgejo ↗".as_bytes())).unwrap();
    assert_eq!(open.external.as_deref(), Some(b"https://forgejo.example/user/oauth2/temper".as_slice()));
}
#[test]
fn elsewhere_details_give_the_next_unused_number() {
    let view = f::page_view(&root(Some(Choice::Elsewhere), w::Connection::NotSetUp, w::Offer::Offered));
    words_in_order(
        view.tree(),
        &[
            "A Forgejo elsewhere",
            "In ~/.config/temper/temper.json, with your host and unused numbers:",
            "application.forges",
            "\"number\": 4",
            "\"kind\": \"forgejo\"",
            "\"host\": \"forge.example\"",
            "\"log_origins\": null",
            "application.identities",
            "\"number\": 4, \"forge\": 4",
            "application.setup",
            "Without it, temper takes its bot’s tokens only.",
            "\"forge\": 4",
            "\"mode\": \"managed\"",
            "Restart temper box, then connect it from this page.",
        ],
    );
}
#[test]
fn elsewhere_numbers_its_steps_and_copies_each_whole_command() {
    let root = f::forge_root(Address::ConnectForge { forge: Some(Choice::Elsewhere) }, List::with_capacity(0));
    let mut view = f::browser_page_view(&root);
    let tree = view.tree();
    let list = tree.nodes().iter().find(|n| n.styles.as_slice().contains(&crate::look::CONNECT_STEPS)).unwrap();
    assert_eq!(list.ordered, Some(1));
    assert_eq!(tree.nodes().iter().filter(|n| n.role == Role::Item).count(), 4);
    assert!(f::find(tree, Role::List, b"Steps").is_none());
    let codes: Vec<_> = tree
        .nodes()
        .iter()
        .enumerate()
        .filter(|(_, n)| n.role == Role::Code)
        .map(|(i, _)| f::node_text(tree, i))
        .collect();
    let copies: Vec<_> = tree
        .nodes()
        .iter()
        .filter(|n| n.role == Role::Button && n.name.as_deref() == Some(b"Copy"))
        .map(|n| n.id)
        .collect();
    assert_eq!(codes.len(), 3);
    assert_eq!(copies.len(), 3);
    for (code, button) in codes.iter().zip(copies) {
        assert_eq!(crate::act(&mut view, button, Act::Press), Some(d::Action::Copy { words: code.clone() }));
    }
    assert_eq!(
        press(&root, Role::Link, b"Other forges"),
        d::Action::Pages(pages::Act::Go { address: Address::ConnectForge { forge: None } })
    );
}

fn welcome_here(connection: w::Connection) -> d::Domain {
    f::forge_root(
        Address::Welcome { step: Step::Forge { forge: Some(Choice::Forge(HERE)) } },
        rows(connection, w::Offer::Offered),
    )
}
#[test]
fn setting_up_marks_the_rows_step() {
    for (step, done) in
        [(None, 0), (Some(w::SetUpStep::Fetch), 0), (Some(w::SetUpStep::Prepare), 1), (Some(w::SetUpStep::Run), 2)]
    {
        let view = f::page_view(&welcome_here(w::Connection::SettingUp(
            w::SettingUp::new(&w::CEILINGS, w::SettingUpParts { step, fetched: None }).unwrap(),
        )));
        node(view.tree(), Role::Disclosure, b"Each step");
        assert_eq!(
            view.tree().nodes().iter().filter(|n| n.role == Role::Status && n.name.as_deref() == Some(b"done")).count(),
            done
        );
        node(view.tree(), Role::Status, b"under way");
        words_in_order(
            view.tree(),
            &[
                if done == 0 { "Fetching Forgejo and its runner" } else { "Fetched Forgejo and its runner" },
                "Preparing: its database, temper’s admin, your account",
                "Starting Forgejo and its runner",
                "Connecting as temper",
                "Making the first repository",
            ],
        );
    }
    let view = f::page_view(&welcome_here(w::Connection::Connecting));
    assert_eq!(
        view.tree().nodes().iter().filter(|n| n.role == Role::Status && n.name.as_deref() == Some(b"done")).count(),
        3
    );
}
#[test]
fn set_up_failed_offers_set_up_again() {
    let root = here(w::Connection::SetUpFailed(
        w::SetUpFailed::new(&w::CEILINGS, w::SetUpFailedParts { why: b"fetch failed".as_slice().into(), at: None })
            .unwrap(),
    ));
    let view = f::page_view(&root);
    let reason = node(view.tree(), Role::Alert, b"Set up failed: fetch failed");
    assert!(
        view.tree()
            .nodes()
            .iter()
            .find(|n| n.id == reason)
            .unwrap()
            .styles
            .as_slice()
            .contains(&crate::look::FLOW_FAILED)
    );
    let retry = node(view.tree(), Role::Button, b"Try again");
    assert!(
        view.tree()
            .nodes()
            .iter()
            .find(|n| n.id == retry)
            .unwrap()
            .styles
            .as_slice()
            .contains(&crate::look::FLOW_SECONDARY)
    );
    assert!(view.tree().nodes().iter().any(|n| n.styles.as_slice().contains(&crate::look::CONNECT_LIVE)));
    assert_eq!(press(&root, Role::Button, b"Try again"), d::Action::Forge(d::forge::Act::SetUp { forge: HERE }));
}
#[test]
fn ready_names_the_forge_and_offers_adopt_a_repository() {
    let root = here(connected());
    let view = f::page_view(&root);
    assert_eq!(root.pages().address(), &Address::Forges);
    words_in_order(view.tree(), &["Connected Forgejo on this machine.", "temper · user 42"]);
    assert_eq!(
        press(&root, Role::Link, b"Adopt a repository"),
        d::Action::Pages(pages::Act::Go { address: Address::Adopt { project: f::PROJECT } })
    );
}

#[test]
fn a_forge_under_way_offers_start_over_in_the_welcome() {
    let root = unfinished_root(Address::Welcome { step: Step::Forge { forge: None } });
    let view = f::page_view(&root);
    words_in_order(view.tree(), &["FORGE · not finished", "Start over"]);
    assert!(f::find(view.tree(), Role::Button, b"Go on").is_none());
    assert_eq!(
        press(&root, Role::Button, b"Start over"),
        d::Action::Forge(d::forge::Act::Forges(d::forge::forges::Act::StartOver {
            forge: HERE,
            end: d::forge::forges::StartOverEnd::Welcome
        }))
    );
}

#[test]
fn the_model_step_follows_the_persons_forge() {
    let mut forges = rows(w::Connection::Connecting, w::Offer::Offered);
    // Model follows the person's forge, even with another forge still connecting.
    *forges.get_mut(0).unwrap() = github_rows(true).get(0).unwrap().clone();
    let root = f::forge_root(Address::Welcome { step: Step::Model { way: None } }, forges);
    let view = f::page_view(&root);
    node(view.tree(), Role::Paragraph, "GitHub · Connected as my-app[bot] (9)".as_bytes());
    assert!(f::find(view.tree(), Role::Disclosure, b"Each step").is_none());
    node(view.tree(), Role::Heading { level: 1 }, b"Connect a model");
    assert_eq!(
        view.tree().nodes().iter().filter(|n| n.role == Role::Status && n.name.as_deref() == Some(b"done")).count(),
        0
    );
}

#[test]
fn the_welcomes_ready_names_the_forge_and_the_model() {
    let (mut root, accounts) =
        f::forge_root_accounts(Address::Welcome { step: Step::Ready }, rows(connected(), w::Offer::Offered));
    f::welcome_account(&mut root, accounts);
    let view = f::page_view(&root);
    node(view.tree(), Role::Heading { level: 1 }, b"Ready");
    node(view.tree(), Role::Paragraph, "Forgejo on this machine · Connected as temper (42)".as_bytes());
    let steps = view.tree().nodes().iter().filter(|n| n.styles.as_slice().contains(&crate::look::FLOW_STEP));
    for step in steps {
        let current = step.states.as_slice().contains(&State::Current);
        assert_eq!(step.styles.as_slice().contains(&crate::look::FLOW_DONE), !current);
    }
    // The account by its kind and who it is; the model is chosen elsewhere
    // (choice's tests name each kind).
    let account = crate::pages::accounts::first_usable_short(&root, &f::words()).unwrap();
    let trying = [account.as_ref(), " · trying: Playground answers · one short message".as_bytes()].concat();
    node(view.tree(), Role::Paragraph, &trying);
}
#[test]
fn ready_without_a_forge_or_model_offers_the_two_steps() {
    let root = f::welcome_root(Step::Ready, false);
    let view = f::page_view(&root);
    node(view.tree(), Role::Slot, b"Forge none yet");
    node(view.tree(), Role::Slot, b"Model none yet");
    node(view.tree(), Role::Link, b"Connect a forge");
    node(view.tree(), Role::Link, b"Connect a model");
}

#[test]
fn the_welcomes_ready_follows_a_forge_under_way() {
    let root =
        f::forge_root(Address::Welcome { step: Step::Ready }, rows(w::Connection::Connecting, w::Offer::Offered));
    let view = f::page_view(&root);
    node(view.tree(), Role::Paragraph, "Forgejo on this machine · Connecting".as_bytes());
    node(view.tree(), Role::Disclosure, b"Each step");
    assert!(f::find(view.tree(), Role::Link, b"Adopt a repository").is_none());
}

#[test]
fn the_forge_pages_have_navigation_and_the_playground_banner() {
    for address in [Address::ConnectForge { forge: None }, Address::Forges] {
        let root = f::playground_frame_root(address, f::PROJECT, true, None, None);
        assert!(root.forge().playground().is_some());
        let view = f::page_view(&root);
        node(view.tree(), Role::Navigation, b"Shop");
        assert!(f::find(view.tree(), Role::Region, b"Playground").is_none());
    }
}

#[test]
fn a_set_up_refusal_is_said_below_its_button() {
    use jig_client::{Answered, Down, Family};
    use skein_lib::{Env, Queue, RequestEnvelope, Time};
    use temper_client_protocol_wire as wire;
    let mut root = here(w::Connection::NotSetUp);
    let limits = f::domain_limits();
    let mut out = Queue::uncapped();
    d::step(
        &mut root,
        &Env { now: Time::ZERO, wall: f::NOW, limits },
        d::Event::Act { action: d::Action::Forge(d::forge::Act::SetUp { forge: HERE }) },
        &mut out,
    );
    let mut sent = None;
    while let Some(request) = out.pop() {
        if let d::Request::Unlinked {
            request: jig_client::UnlinkedRequest::Down { down: Down::Send { attempt, .. } },
        } = request
        {
            sent = Some(attempt);
        }
    }
    let wire_limits = d::forge::wire_limits(&limits.forge).unwrap();
    let outcome = w::Outcome::SetUpRefused(
        w::SetUpRefused::new(&wire_limits, w::SetUpRefusedParts { why: w::SetUpRefusal::Busy }).unwrap(),
    );
    f::root_event(
        &mut root,
        d::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Answer {
                    attempt: sent.expect("set up sent"),
                    envelope: RequestEnvelope::Final,
                    answer: Some(Answered::Family(Family {
                        family: wire::FORGE,
                        version: wire::CLIENT_VERSION,
                        bytes: wire::answer(&w::Answer::new(&wire_limits, w::AnswerParts { outcome }).unwrap()),
                    })),
                    why: None,
                },
            },
        },
    );
    let view = f::page_view(&root);
    let refusal = d::forge::words::set_up_refusal(d::forge::forges::SetUpRefusal::Busy);
    node(view.tree(), Role::Alert, refusal);
    let button = node(view.tree(), Role::Button, b"Set up a Forgejo here");
    assert!(!view.tree().nodes().iter().find(|n| n.id == button).unwrap().states.as_slice().contains(&State::Disabled));
}

fn has_state(tree: &skein_ui::Tree, role: Role, name: &[u8], state: State) -> bool {
    let id = node(tree, role, name);
    tree.nodes().iter().find(|n| n.id == id).unwrap().states.as_slice().contains(&state)
}
#[test]
fn details_show_the_set_ups_fields_with_their_defaults() {
    let mut root = here(w::Connection::NotSetUp);
    let view = f::page_view(&root);
    for (name, value) in [(b"Organization".as_slice(), b"work".as_slice()), (b"Repository", b"hello")] {
        let id = node(view.tree(), Role::TextField, name);
        assert_eq!(
            view.tree().nodes().iter().find(|n| n.id == id).unwrap().value.as_ref().unwrap().text.as_ref(),
            value
        );
    }
    let nodes = view.tree().nodes();
    let org = nodes.iter().find(|n| n.role == Role::TextField && n.name.as_deref() == Some(b"Organization")).unwrap();
    let repository =
        nodes.iter().find(|n| n.role == Role::TextField && n.name.as_deref() == Some(b"Repository")).unwrap();
    let grandparent = |id| parent(view.tree(), parent(view.tree(), id).unwrap());
    assert_eq!(grandparent(org.id), grandparent(repository.id));
    let pair = grandparent(org.id).unwrap();
    assert!(nodes.iter().find(|n| n.id == pair).unwrap().styles.as_slice().contains(&crate::look::FLOW_PAIR));
    let primary: Vec<_> = nodes
        .iter()
        .filter(|n| n.styles.as_slice().contains(&crate::look::FLOW_PRIMARY))
        .map(|n| n.name.as_deref().unwrap())
        .collect();
    assert_eq!(primary, [b"Set up a Forgejo here".as_slice()]);
    for field in [org, repository] {
        let wrapper = parent(view.tree(), field.id).unwrap();
        let wrapper = nodes.iter().find(|n| n.id == wrapper).unwrap();
        assert!(wrapper.styles.as_slice().contains(&crate::look::FLOW_FIELD));
    }
    let facts = nodes.iter().find(|n| n.styles.as_slice().contains(&crate::look::FLOW_FACTS)).unwrap();
    assert_eq!(facts.role, Role::List);
    let items: Vec<_> = nodes.iter().filter(|n| parent(view.tree(), n.id) == Some(facts.id)).collect();
    assert_eq!(items.len(), 2);
    assert!(items.iter().all(|n| n.role == Role::Item));
    assert!(!view.tree().nodes().iter().any(|node| node.role == Role::ChoiceGroup));
    let mut view = f::page_view(&root);
    let id = node(view.tree(), Role::TextField, b"Repository");
    let action = crate::act(&mut view, id, Act::Input { value: Box::default() }).unwrap();
    f::root_event(&mut root, d::Event::Act { action });
    act(&mut root, d::forge::Act::SetUp { forge: HERE });
    let view = f::page_view(&root);
    assert!(has_state(view.tree(), Role::TextField, b"Repository", State::Invalid));
    node(view.tree(), Role::Alert, b"Required.");
}
#[test]
fn an_invalid_refusal_marks_the_organization_and_repository() {
    let mut root = here(w::Connection::NotSetUp);
    let token = attempt(act(&mut root, d::forge::Act::SetUp { forge: HERE }));
    answer(
        &mut root,
        token,
        w::Outcome::SetUpRefused(
            w::SetUpRefused::new(&w::CEILINGS, w::SetUpRefusedParts { why: w::SetUpRefusal::Invalid }).unwrap(),
        ),
    );
    let view = f::page_view(&root);
    for name in [b"Organization".as_slice(), b"Repository"] {
        assert!(has_state(view.tree(), Role::TextField, name, State::Invalid));
    }
    node(view.tree(), Role::Alert, b"invalid: the organization, repository or project name is empty or too long");
    assert_eq!(root.forge().setup_form().text(d::forge::forges::SetupField::Repository), b"hello");
}
#[test]
fn setting_up_names_its_first_repository() {
    let root = f::forge_root(
        Address::ConnectForge { forge: Some(Choice::Forge(HERE)) },
        first_rows(w::Connection::Connecting, w::FirstState::Making),
    );
    let view = f::page_view(&root);
    words_in_order(view.tree(), &["Making work/hello"]);
    assert_eq!(
        view.tree().nodes().iter().filter(|n| n.role == Role::Status && n.name.as_deref() == Some(b"done")).count(),
        0_usize
    );
    assert!(view.tree().nodes().iter().any(|n| n.states.as_slice().contains(&State::Current)));
}
#[test]
fn setting_up_names_the_repository_without_the_playground_project() {
    let root = f::root_at_with_forges(
        Address::ConnectForge { forge: Some(Choice::Forge(HERE)) },
        &f::domain_limits(),
        b"Project 1",
        Some((f::PROJECT, true, w::PlaygroundState::Running)),
        None,
        Some(first_rows(w::Connection::Connecting, w::FirstState::Making)),
    )
    .0;
    for words in [f::words(), f::browser_words()] {
        let mut view = f::view_of(words);
        f::build(&mut view, &root);
        words_in_order(view.tree(), &["Making work/hello"]);
    }
}
#[test]
fn setting_up_uses_the_sent_fields_until_the_row_carries_them() {
    let mut root = here(w::Connection::NotSetUp);
    act(&mut root, d::forge::Act::SetUp { forge: HERE });
    // The flow keeps the submitted fields even when it is showing Details.
    let w::SetUpFirst::Empty(first) = &root.forge().set_up().unwrap().first else { panic!("empty setup") };
    assert_eq!(first.org(), b"work");
    let (mut root, watch) = f::forge_root_watch(
        Address::ConnectForge { forge: Some(Choice::Forge(HERE)) },
        rows(w::Connection::NotSetUp, w::Offer::Offered),
    );
    act(&mut root, d::forge::Act::SetUp { forge: HERE });
    deliver(&mut root, watch, rows(w::Connection::Connecting, w::Offer::Offered));
    let view = f::page_view(&root);
    words_in_order(view.tree(), &["Making work/hello"]);
    words_in_order(view.tree(), &["Starting Forgejo and its runner", "Connecting"]);
    assert!(!f::text(view.tree()).windows(b"Connecting, and making".len()).any(|w| w == b"Connecting, and making"));
}
#[test]
fn ready_names_the_connection_and_made_repository_and_offers_adoption() {
    let root =
        f::forge_root(Address::ConnectForge { forge: Some(Choice::Forge(HERE)) }, first_rows(connected(), made()));
    let view = f::page_view(&root);
    words_in_order(view.tree(), &["Connected Forgejo on this machine.", "Adopt a repository", "work/hello"]);
    assert_eq!(
        press(&root, Role::Link, b"Adopt a repository"),
        d::Action::Pages(pages::Act::Go { address: Address::Adopt { project: f::PROJECT } })
    );
    assert!(f::find(view.tree(), Role::Link, b"Open Shop").is_none());
}
#[test]
fn a_failed_first_repository_says_its_step_with_set_up_again() {
    for (step, expected) in [
        (w::CreateStep::Admin, b"Making work/hello failed: checking the admin's token".as_slice()),
        (w::CreateStep::Org, b"Making work/hello failed: making the organization"),
        (w::CreateStep::Repository, b"Making work/hello failed: making the repository"),
        (w::CreateStep::File, b"Making work/hello failed: adding the starter workflow"),
    ] {
        for address in
            [Address::ConnectForge { forge: Some(Choice::Forge(HERE)) }, Address::Welcome { step: Step::Ready }]
        {
            let root = f::forge_root(
                address,
                first_rows(
                    w::Connection::Connecting,
                    w::FirstState::Failed(
                        w::CreateFailed::new(&w::CEILINGS, w::CreateFailedParts { step: step.clone() }).unwrap(),
                    ),
                ),
            );
            let view = f::page_view(&root);
            node(view.tree(), Role::Alert, expected);
            node(
                view.tree(),
                Role::Button,
                if *root.pages().address() == Address::Forges { b"Try again" } else { b"Set up again" },
            );
            if *root.pages().address() == Address::Forges {
                assert!(view.tree().nodes().iter().any(|n| n.styles.as_slice().contains(&crate::look::CONNECT_LIVE)));
            } else {
                node(view.tree(), Role::Disclosure, b"Each step");
            }
        }
    }
}
#[test]
fn the_forges_row_names_the_first_repository() {
    for (state, expected, first) in [
        (w::FirstState::Making, b"work/hello".as_slice(), b"Making work/hello".as_slice()),
        (made(), "work/hello".as_bytes(), "work/hello".as_bytes()),
    ] {
        let root = f::forge_root(Address::Forges, first_rows(connected(), state.clone()));
        let view = f::page_view(&root);
        let id = node(view.tree(), Role::Disclosure, b"Forgejo on this machine");
        let row = view.tree().nodes().iter().find(|n| n.id == id).unwrap();
        let description = row.description.as_deref().unwrap();
        assert!(description.ends_with(expected));
        assert_eq!(row.states.as_slice().contains(&State::Expanded), state != made());
        if state == made() {
            node(view.tree(), Role::Description, first);
        } else {
            words_in_order(view.tree(), &["Making work/hello"]);
        }
    }
}

#[test]
fn connecting_on_github_offers_install_with_its_way_back() {
    for address in [
        Address::ConnectForge { forge: Some(Choice::Forge(GITHUB)) },
        Address::Welcome { step: Step::Forge { forge: Some(Choice::Forge(GITHUB)) } },
    ] {
        let root = f::forge_root(address.clone(), github_rows(false));
        let view = f::browser_page_view(&root);
        words_in_order(
            view.tree(),
            &["Install temper’s App", "my-app is connected to temper, and installed nowhere yet.", "Install on GitHub"],
        );
        let mut view = f::browser_page_view(&root);
        let button = node(view.tree(), Role::Button, b"Install on GitHub");
        assert_eq!(
            crate::act(&mut view, button, Act::Press),
            Some(d::Action::Forge(d::forge::Act::Install {
                forge: GITHUB,
                back: d::address::print(root.pages().address())
            }))
        );
    }
}
#[test]
fn an_accepted_installation_makes_github_ready_without_a_picker() {
    let root = f::forge_root(Address::ConnectForge { forge: Some(Choice::Forge(GITHUB)) }, github_rows(true));
    let view = f::browser_page_view(&root);
    words_in_order(view.tree(), &["Connected GitHub.", "Adopt a repository", "my-app[bot] · user 9"]);
    assert!(f::find(view.tree(), Role::ChoiceGroup, b"Account").is_none());
}
#[test]
#[expect(clippy::disallowed_macros, reason = "the test captures Debug to verify secret redaction")]
fn the_private_key_moves_as_a_secret_and_never_enters_the_view() {
    let mut root = root(Some(Choice::Forge(GITHUB)), w::Connection::NotSetUp, w::Offer::Offered);
    let view = f::browser_page_view(&root);
    let id = node(view.tree(), Role::TextField, b"App id");
    let event = crate::event(&view, id, Act::Input { value: b"7".as_slice().into() }).unwrap();
    f::root_event(&mut root, event);
    let key = node(view.tree(), Role::TextField, b"Private key");
    let event = crate::event(&view, key, Act::Input { value: b"private-app-fixture\nsecond line\n".as_slice().into() })
        .unwrap();
    assert!(!format!("{event:?}").contains("private-app-fixture"));
    let d::Event::Forge(d::forge::Event::AppKey { .. }) = &event else {
        panic!("the input moves straight into a secret key event");
    };
    f::root_event(&mut root, event);
    let view = f::browser_page_view(&root);
    assert!(!format!("{view:?}").contains("private-app-fixture"));
    let field = view
        .tree()
        .nodes()
        .iter()
        .find(|n| n.role == Role::TextField && n.name.as_deref() == Some(b"Private key"))
        .unwrap();
    assert!(field.value.as_ref().unwrap().text.is_empty());
    let written = field.value.as_ref().unwrap().written;
    f::root_event(&mut root, d::Event::Act { action: d::Action::Forge(d::forge::Act::ConnectApp { forge: GITHUB }) });
    let view = f::browser_page_view(&root);
    let field = view
        .tree()
        .nodes()
        .iter()
        .find(|n| n.role == Role::TextField && n.name.as_deref() == Some(b"Private key"))
        .unwrap();
    assert!(field.value.as_ref().unwrap().written > written);
    assert!(!root.forge().app_form().has_key());
}
#[test]
fn a_connect_refusal_is_said_below_its_button() {
    let mut root = root(Some(Choice::Forge(GITHUB)), w::Connection::NotSetUp, w::Offer::Offered);
    f::root_event(
        &mut root,
        d::Event::Act { action: d::Action::Forge(d::forge::Act::AppWrite { text: b"7".as_slice().into() }) },
    );
    f::root_event(
        &mut root,
        d::Event::Forge(d::forge::Event::AppKey { value: skein_lib::Secret::new(b"fake-key".as_slice().into()) }),
    );
    let mut out = skein_lib::Queue::uncapped();
    d::step(
        &mut root,
        &skein_lib::Env { now: skein_lib::Time::ZERO, wall: f::NOW, limits: f::limits() },
        d::Event::Act { action: d::Action::Forge(d::forge::Act::ConnectApp { forge: GITHUB }) },
        &mut out,
    );
    let mut attempt = None;
    while let Some(request) = out.pop() {
        if let d::Request::Unlinked {
            request: jig_client::UnlinkedRequest::Down { down: jig_client::Down::Send { attempt: value, .. } },
        } = request
        {
            attempt = Some(value);
        }
    }
    f::root_event(
        &mut root,
        d::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Answer {
                    attempt: attempt.unwrap(),
                    envelope: skein_lib::RequestEnvelope::Final,
                    answer: Some(jig_client::Answered::Family(jig_client::Family {
                        family: temper_client_protocol_wire::FORGE,
                        version: temper_client_protocol_wire::CLIENT_VERSION,
                        bytes: temper_client_protocol_wire::answer(
                            &w::Answer::new(
                                &w::CEILINGS,
                                w::AnswerParts {
                                    outcome: w::Outcome::ConnectRefused(
                                        w::ConnectRefused::new(
                                            &w::CEILINGS,
                                            w::ConnectRefusedParts { why: w::ForgeRefusal::NoKey },
                                        )
                                        .unwrap(),
                                    ),
                                },
                            )
                            .unwrap(),
                        ),
                    })),
                    why: None,
                },
            },
        },
    );
    let view = f::browser_page_view(&root);
    words_in_order(view.tree(), &["Connect the App", "The key did not reach temper: paste it again."]);
}

fn manifest_started(address: Address) -> (d::Domain, skein_lib::Queue<d::Request>) {
    let mut root = f::forge_root(address, List::with_capacity(0));
    let mut view = f::browser_page_view(&root);
    let button = node(view.tree(), Role::Button, b"Connect GitHub");
    let action = crate::act(&mut view, button, Act::Press).expect("live manifest button");
    let out = drive(&mut root, d::Event::Act { action });
    (root, out)
}
#[test]
fn connect_github_sends_the_flow_address_at_its_forge() {
    for (address, back) in [
        (Address::ConnectForge { forge: None }, b"/forges".as_slice()),
        (Address::Welcome { step: Step::Forge { forge: None } }, b"/welcome?at=forge".as_slice()),
    ] {
        let (_, mut out) = manifest_started(address);
        let mut sent = None;
        while let Some(request) = out.pop() {
            if let d::Request::Unlinked {
                request: jig_client::UnlinkedRequest::Down { down: jig_client::Down::Send { request, value, .. } },
            } = request
            {
                assert!(value.is_none());
                let jig_client::Ask::Family { record, .. } = request else { panic!("family") };
                sent = Some(
                    temper_client_protocol_wire::decode_requesting(&record.bytes, &w::CEILINGS)
                        .unwrap()
                        .into_parts()
                        .request,
                );
            }
        }
        let w::Request::Connect(connect) = sent.expect("manifest Connect") else { panic!("Connect") };
        let w::Making::New(_) = connect.forge() else { panic!("new forge") };
        let w::How::GithubManifest(manifest) = connect.how() else { panic!("manifest") };
        assert_eq!(manifest.back().as_deref(), Some(back));
    }
}
#[test]
fn connect_github_is_disabled_until_the_answer() {
    let (root, _) = manifest_started(Address::ConnectForge { forge: None });
    let view = f::browser_page_view(&root);
    let button = node(view.tree(), Role::Button, b"Connect GitHub");
    assert!(view.tree().nodes().iter().find(|n| n.id == button).unwrap().states.as_slice().contains(&State::Disabled));
}
#[test]
fn connect_github_refusal_shows_under_its_button() {
    let (mut root, out) = manifest_started(Address::ConnectForge { forge: None });
    answer(
        &mut root,
        attempt(out),
        w::Outcome::ConnectRefused(
            w::ConnectRefused::new(&w::CEILINGS, w::ConnectRefusedParts { why: w::ForgeRefusal::Busy }).unwrap(),
        ),
    );
    let view = f::browser_page_view(&root);
    words_in_order(view.tree(), &["Connect GitHub", "temper is busy with forge 0: try again when it ends."]);
}
#[test]
fn connect_github_link_leaves_the_tab_once() {
    let (mut root, out) = manifest_started(Address::ConnectForge { forge: None });
    let attempt = attempt(out);
    let outcome = w::Outcome::Link(
        w::Link::new(&w::CEILINGS, w::LinkParts { address: b"/forges/2/github/manifest?state=fake".as_slice().into() })
            .unwrap(),
    );
    let mut out = drive(&mut root, answer_event(attempt, outcome.clone()));
    let mut leaves = Vec::new();
    while let Some(request) = out.pop() {
        if let d::Request::Leave { to, .. } = request {
            leaves.push(to);
        }
    }
    assert_eq!(leaves, [Box::<[u8]>::from(b"/forges/2/github/manifest?state=fake".as_slice())]);
    let mut out = drive(&mut root, answer_event(attempt, outcome));
    while let Some(request) = out.pop() {
        if let d::Request::Leave { .. } = request {
            panic!("only the first answer may leave the page");
        }
    }
    let view = f::browser_page_view(&root);
    // The tab leaves by the path; no external link is drawn for it.
    assert!(view.tree().nodes().iter().all(|n| n.external.is_none()));
}
#[test]
fn a_whole_address_link_keeps_its_external_link_meanwhile() {
    let (mut root, out) = manifest_started(Address::ConnectForge { forge: None });
    let attempt = attempt(out);
    let address = b"https://github.com/apps/temper-fake/installations/new?state=fake";
    let outcome =
        w::Outcome::Link(w::Link::new(&w::CEILINGS, w::LinkParts { address: address.as_slice().into() }).unwrap());
    drive(&mut root, answer_event(attempt, outcome));
    let view = f::browser_page_view(&root);
    let link = node(view.tree(), Role::Link, b"Connect GitHub");
    assert_eq!(
        view.tree().nodes().iter().find(|n| n.id == link).unwrap().external.as_deref(),
        Some(address.as_slice())
    );
}
#[test]
fn connecting_uses_the_apps_way_for_its_words() {
    for (app, words) in [
        (
            Some(w::AppWay::Manifest),
            "my-app is made on GitHub, under your account, and installed nowhere yet. Install it, and choose the repositories temper may use.",
        ),
        (
            Some(w::AppWay::Pasted),
            "my-app is connected to temper, and installed nowhere yet. Install it, and choose the repositories temper may use.",
        ),
        (
            None,
            "my-app is connected to temper, and installed nowhere yet. Install it, and choose the repositories temper may use.",
        ),
    ] {
        let mut parts = github_rows(false).get(0).unwrap().clone().into_parts();
        parts.app = app;
        let mut rows = List::with_capacity(1);
        rows.push(w::ForgeRow::new(&w::CEILINGS, parts).unwrap()).unwrap();
        let root = f::forge_root(Address::ConnectForge { forge: Some(Choice::Forge(GITHUB)) }, rows);
        words_in_order(f::browser_page_view(&root).tree(), &["Install temper’s App", words, "Install on GitHub"]);
    }
}

#[test]
fn delivered_steps_finish_when_the_first_repository_is_made() {
    let (mut root, watch) = f::forge_root_watch(
        Address::Welcome { step: Step::Forge { forge: Some(Choice::Forge(HERE)) } },
        rows(w::Connection::NotSetUp, w::Offer::Offered),
    );
    let token = attempt(act(&mut root, d::forge::Act::SetUp { forge: HERE }));
    answer(&mut root, token, w::Outcome::SettingUp);
    for (connection, done) in [
        (
            w::Connection::SettingUp(
                w::SettingUp::new(&w::CEILINGS, w::SettingUpParts { step: Some(w::SetUpStep::Fetch), fetched: None })
                    .unwrap(),
            ),
            0,
        ),
        (
            w::Connection::SettingUp(
                w::SettingUp::new(&w::CEILINGS, w::SettingUpParts { step: Some(w::SetUpStep::Prepare), fetched: None })
                    .unwrap(),
            ),
            1,
        ),
        (
            w::Connection::SettingUp(
                w::SettingUp::new(&w::CEILINGS, w::SettingUpParts { step: Some(w::SetUpStep::Run), fetched: None })
                    .unwrap(),
            ),
            2,
        ),
        (w::Connection::Connecting, 3),
        (connected(), 4),
    ] {
        deliver(&mut root, watch, first_rows(connection, w::FirstState::Making));
        let view = f::page_view(&root);
        let list = view
            .tree()
            .nodes()
            .iter()
            .position(|n| n.role == Role::List && n.name.as_deref() == Some(b"Each step"))
            .unwrap();
        let nodes = &view.tree().nodes()
            [list..list.checked_add(usize::try_from(view.tree().nodes()[list].size).unwrap()).unwrap()];
        assert_eq!(nodes.iter().filter(|n| n.role == Role::Item).count(), 5);
        assert_eq!(nodes.iter().filter(|n| n.role == Role::Status && n.name.as_deref() == Some(b"done")).count(), done);
        assert_eq!(
            nodes.iter().filter(|n| n.role == Role::Status && n.name.as_deref() == Some(b"under way")).count(),
            1
        );
    }
    deliver(&mut root, watch, first_rows(connected(), made()));
    assert!(pages::welcome::forge_connected(d::forge::connect::row(HERE, root.forges(), None).unwrap()));
}

#[test]
fn an_unknown_setup_stage_claims_no_specific_current_step() {
    let view = f::page_view(&welcome_here(w::Connection::SettingUp(
        w::SettingUp::new(&w::CEILINGS, w::SettingUpParts { step: None, fetched: None }).unwrap(),
    )));
    let list = view
        .tree()
        .nodes()
        .iter()
        .position(|n| n.role == Role::List && n.name.as_deref() == Some(b"Each step"))
        .unwrap();
    let nodes =
        &view.tree().nodes()[list..list.checked_add(usize::try_from(view.tree().nodes()[list].size).unwrap()).unwrap()];
    assert!(!nodes.iter().any(|n| n.states.as_slice().contains(&State::Current)));
    node(view.tree(), Role::Status, b"under way");
}

#[test]
fn standalone_setup_names_the_requested_target_before_the_row_has_it() {
    let (mut root, watch) = f::forge_root_watch(
        Address::ConnectForge { forge: Some(Choice::Forge(HERE)) },
        rows(w::Connection::NotSetUp, w::Offer::Offered),
    );
    let token = attempt(act(&mut root, d::forge::Act::SetUp { forge: HERE }));
    answer(&mut root, token, w::Outcome::SettingUp);
    deliver(&mut root, watch, rows(w::Connection::Connecting, w::Offer::Offered));
    let view = f::page_view(&root);
    let text = f::text(view.tree());
    let text = core::str::from_utf8(&text).unwrap();
    assert!(text.contains("Making work/hello"));
    assert!(!text.contains("Making the first repository"));
}

#[test]
fn the_connect_page_has_no_steps_bar_on_github_either() {
    for rows in [rows(w::Connection::NotSetUp, w::Offer::Offered), github_rows(false), github_rows(true)] {
        let root = f::forge_root(Address::ConnectForge { forge: Some(Choice::Forge(GITHUB)) }, rows);
        let view = f::browser_page_view(&root);
        assert!(f::find(view.tree(), Role::List, b"Steps").is_none());
        node(
            view.tree(),
            Role::Heading { level: 1 },
            if *root.pages().address() == Address::Forges { b"Where your code lives." } else { b"Connect a forge" },
        );
    }
}

// The mockup's failed frame, and each other step of the same five-step flow.

#[test]
fn failed_setup_keeps_all_steps_and_marks_only_its_failure() {
    for (rows, failed, why) in failure_cases() {
        let root = f::forge_root(Address::Welcome { step: Step::Forge { forge: Some(Choice::Forge(HERE)) } }, rows);
        let view = f::page_view(&root);
        failed_progress(view.tree(), failed, why);
    }
}

#[test]
fn setup_failure_words_keep_the_full_bounded_reason() {
    let words = f::words();
    let bound = usize::try_from(temper_client_protocol_wire::SET_UP_WHY_BYTES).unwrap();
    for (bytes, reason) in [(b"x".repeat(bound), "x".repeat(bound)), ([0xff].repeat(bound), "�".repeat(bound))] {
        let shown = d::forge::words::lossy(&bytes);
        let text = words.say(crate::words::Word::ForgePrepareFailed, &[skein_ui::phrases::Hole::Name(shown)]);
        let mut expected = String::from("Preparing failed: ");
        expected.push_str(&reason);
        assert_eq!(text.as_ref(), expected.as_bytes());
    }
}

const ADMIN_TOKEN: &[u8] = b"admin-token-placeholder-0123";
fn admin_row(
    connection: w::Connection,
    stop: Option<w::AdminStop>,
    first: Option<w::FirstRepository>,
    web: bool,
) -> w::ForgeRow {
    let mut parts = row(HERE, w::Kind::Forgejo, w::Mode::Managed, connection, w::Offer::NotOffered).into_parts();
    parts.admin = true;
    parts.ended_stop = stop;
    parts.first = first;
    parts.ended = Some(
        w::ConnectStopped::new(&w::CEILINGS, w::ConnectStoppedParts { why: b"Forgejo answered 500".as_slice().into() })
            .unwrap(),
    );
    if web {
        parts.web = Some(b"https://git.example.org".as_slice().into());
    }
    w::ForgeRow::new(&w::CEILINGS, parts).unwrap()
}
fn admin_root(choice: Option<Choice>, row: w::ForgeRow) -> d::Domain {
    let mut rows = List::with_capacity(1);
    rows.push(row).unwrap();
    f::forge_root(Address::ConnectForge { forge: choice }, rows)
}
fn admin_input(root: &mut d::Domain, name: &[u8], value: &[u8]) {
    let view = f::page_view(root);
    let id = node(view.tree(), Role::TextField, name);
    let event = crate::event(&view, id, Act::Input { value: value.into() }).unwrap();
    f::root_event(root, event);
}
fn admin_send(root: &mut d::Domain) {
    let action = press(root, Role::Button, b"Connect as its admin");
    f::root_event(root, d::Event::Act { action });
}
fn no_admin_token(tree: &skein_ui::Tree) {
    for node in tree.nodes() {
        for value in [node.name.as_deref(), node.description.as_deref(), node.value.as_ref().map(|v| v.text.as_ref())]
            .into_iter()
            .flatten()
        {
            assert!(skein_lib::bytes::find(value, ADMIN_TOKEN).is_none());
        }
        for run in &node.text {
            assert!(skein_lib::bytes::find(&run.text, ADMIN_TOKEN).is_none());
        }
    }
}
#[test]
fn managed_details_ask_for_the_address_git_and_web() {
    let root = f::forge_root(Address::ConnectForge { forge: Some(Choice::Managed) }, List::with_capacity(0));
    let mut view = f::page_view(&root);
    for (name, field) in [
        (b"Its address".as_slice(), d::forge::admin::PlaceField::Host),
        (b"Its git base", d::forge::admin::PlaceField::Git),
        (b"Its web address", d::forge::admin::PlaceField::Web),
    ] {
        let id = node(view.tree(), Role::TextField, name);
        assert_eq!(
            crate::act(&mut view, id, Act::Input { value: b"forge.example".as_slice().into() }),
            Some(d::Action::Forge(d::forge::Act::Forges(d::forge::forges::Act::PlaceWrite {
                field,
                text: b"forge.example".as_slice().into()
            })))
        );
    }
    node(view.tree(), Role::TextField, "The site admin’s token".as_bytes());
}

#[test]
fn admin_details_keep_the_token_out_of_actions_and_rendered_pages() {
    let mut root = admin_root(Some(Choice::Forge(HERE)), admin_row(w::Connection::NotConnected, None, None, true));
    let mut before = f::page_view(&root);
    let tree = before.tree();
    assert!(tree.nodes().iter().any(|n| n.styles.as_slice().contains(&crate::look::CONNECT_PANEL)));
    for name in [b"Organization".as_slice(), b"First repository", b"Runner label"] {
        node(tree, Role::TextField, name);
    }
    let field = node(before.tree(), Role::TextField, "The site admin’s token".as_bytes());
    assert!(crate::act(&mut before, field, Act::Input { value: ADMIN_TOKEN.into() }).is_none());
    admin_input(&mut root, "The site admin’s token".as_bytes(), ADMIN_TOKEN);
    let view = f::page_view(&root);
    let field = view
        .tree()
        .nodes()
        .iter()
        .find(|n| n.id == node(view.tree(), Role::TextField, "The site admin’s token".as_bytes()))
        .unwrap();
    assert_eq!(field.entry, Some(skein_ui::EntryKind::Secret));
    no_admin_token(view.tree());
    assert!(f::find(view.tree(), Role::Choice, "With its bot’s tokens".as_bytes()).is_none());
    admin_input(&mut root, b"Runner label", b"our-runner");
    node(f::page_view(&root).tree(), Role::Disclosure, b"Its runner label: our-runner");
    for name in [b"Organization".as_slice(), b"First repository", b"Runner label"] {
        node(view.tree(), Role::TextField, name);
    }
    assert_eq!(
        press(&root, Role::Link, b"Other forges"),
        d::Action::Pages(pages::Act::Go { address: Address::ConnectForge { forge: None } })
    );
    f::root_event(
        &mut root,
        d::Event::Went {
            place: d::Place::Address(Address::Welcome { step: Step::Forge { forge: Some(Choice::Forge(HERE)) } }),
        },
    );
    assert_eq!(
        press(&root, Role::Link, b"Other forges"),
        d::Action::Pages(pages::Act::Go { address: Address::Welcome { step: Step::Forge { forge: None } } })
    );
    admin_send(&mut root);
    assert!(!root.forge().admin_form().has_token());
    no_admin_token(f::page_view(&root).tree());
}
#[test]
fn admin_local_refusals_mark_the_field_and_say_why() {
    for (field, value, refusal) in [
        ("The site admin’s token", b"".as_slice(), "Required."),
        ("The site admin’s token", &[b'x'; 4091], "The pasted values are past 4 KiB, the most a request carries."),
        ("Organization", b"", "Required."),
        ("First repository", b"", "Required."),
        ("Runner label", b"a b", "Letters, digits, dots, dashes and underscores, at most 64."),
    ] {
        let mut root = admin_root(Some(Choice::Forge(HERE)), admin_row(w::Connection::NotConnected, None, None, true));
        admin_input(&mut root, field.as_bytes(), value);
        admin_send(&mut root);
        let view = f::page_view(&root);
        words_in_order(view.tree(), &[refusal]);
        assert!(
            view.tree()
                .nodes()
                .iter()
                .find(|n| n.id == node(view.tree(), Role::TextField, field.as_bytes()))
                .unwrap()
                .states
                .as_slice()
                .contains(&State::Invalid)
        );
        no_admin_token(view.tree());
    }
}
#[test]
fn a_first_organization_named_as_the_bot_marks_the_organization_invalid() {
    let mut root = admin_root(Some(Choice::Forge(HERE)), admin_row(w::Connection::NotConnected, None, None, true));
    admin_input(&mut root, "The site admin’s token".as_bytes(), ADMIN_TOKEN);
    admin_input(&mut root, b"Organization", b"temper-bot");
    let action = press(&root, Role::Button, b"Connect as its admin");
    let token = attempt(drive(&mut root, d::Event::Act { action }));
    answer(
        &mut root,
        token,
        w::Outcome::ConnectRefused(
            w::ConnectRefused::new(&w::CEILINGS, w::ConnectRefusedParts { why: w::ForgeRefusal::FirstIsBot }).unwrap(),
        ),
    );
    let view = f::page_view(&root);
    words_in_order(
        view.tree(),
        &[
            "That organization has the name of temper's bot on this forge, and on Forgejo users and organizations share names: name another organization.",
        ],
    );
    assert!(has_state(view.tree(), Role::TextField, b"Organization", State::Invalid));
    for name in [b"First repository".as_slice(), b"First repository", b"Runner label"] {
        assert!(!has_state(view.tree(), Role::TextField, name, State::Invalid));
    }
    assert!(!root.forge().admin_form().has_token());
    no_admin_token(view.tree());
}
#[test]
fn admin_connecting_names_the_forge_and_first_repository() {
    let root = admin_root(Some(Choice::Forge(HERE)), admin_row(w::Connection::Connecting, None, None, true));
    let view = f::page_view(&root);
    words_in_order(view.tree(), &["Checked its address and your token", "Making the first repository"]);
    assert!(f::find(view.tree(), Role::Button, b"Connect as its admin").is_none());
}
#[test]
fn admin_stops_name_the_reason_and_keep_an_empty_token_field() {
    let refused = |status, stage| {
        w::AdminStop::Refused(w::AdminRefused::new(&w::CEILINGS, w::AdminRefusedParts { status, stage }).unwrap())
    };
    for (stop, line) in [
        (w::AdminStop::NotAdmin, "The token you gave is not a site admin’s. Paste one a site admin made."),
        (
            refused(401, w::SetupStage::Admin),
            "Forgejo refused the token. Check that it was copied whole, and that it hasn’t been deleted.",
        ),
        (refused(403, w::SetupStage::Bot), "The token lacks a scope: making the bot was refused."),
        (refused(403, w::SetupStage::Org), "The token lacks a scope: making the organization was refused."),
        (w::AdminStop::Unmarked, "A user temper didn’t make holds the bot’s login"),
        (w::AdminStop::Other, "Forgejo answered 500"),
    ] {
        let root =
            admin_root(Some(Choice::Forge(HERE)), admin_row(w::Connection::NotConnected, Some(stop), None, true));
        let view = f::page_view(&root);
        words_in_order(view.tree(), &[line, "temper kept nothing of the token."]);
        node(view.tree(), Role::TextField, "The site admin’s token".as_bytes());
        no_admin_token(view.tree());
    }
}
fn admin_first(state: w::FirstState) -> w::FirstRepository {
    w::FirstRepository::new(
        &w::CEILINGS,
        w::FirstRepositoryParts {
            playground: false,
            org: b"work".as_slice().into(),
            repository: b"hello".as_slice().into(),
            state,
            label: b"linux-arm".as_slice().into(),
        },
    )
    .unwrap()
}
#[test]
fn each_admin_create_step_returns_to_details_with_its_repository() {
    for (step, words) in [
        (w::CreateStep::Admin, "checking the admin's token"),
        (w::CreateStep::Org, "its organization"),
        (w::CreateStep::Repository, "its repository"),
        (w::CreateStep::File, "its starter workflow"),
    ] {
        let first = admin_first(w::FirstState::Failed(
            w::CreateFailed::new(&w::CEILINGS, w::CreateFailedParts { step }).unwrap(),
        ));
        let root = admin_root(Some(Choice::Forge(HERE)), admin_row(connected(), None, Some(first), true));
        let view = f::page_view(&root);
        words_in_order(
            view.tree(),
            &[
                "temper made its bot, but making work/hello stopped at",
                words,
                "Forgejo answered 500. Paste the token again to finish it.",
                "temper kept nothing of the token.",
            ],
        );
        node(view.tree(), Role::Button, b"Connect as its admin");
        no_admin_token(view.tree());
    }
}
#[test]
fn admin_ready_uses_the_durable_label_and_repository_link() {
    let mut parts = admin_row(connected(), None, Some(admin_first(made())), true).into_parts();
    parts.repositories = List::with_capacity(1);
    parts
        .repositories
        .push(
            w::Adoption::new(
                &w::CEILINGS,
                w::AdoptionParts { owner: b"work".as_slice().into(), name: b"hello".as_slice().into(), project: 7 },
            )
            .unwrap(),
        )
        .unwrap();
    let root = admin_root(Some(Choice::Forge(HERE)), w::ForgeRow::new(&w::CEILINGS, parts).unwrap());
    let view = f::page_view(&root);
    words_in_order(
        view.tree(),
        &[
            "Connected Your Forgejo.",
            "Made work/hello. Its CI runs on your forge’s runners labelled linux-arm; with none, a change’s checks wait.",
            "Open work/hello on Forgejo",
            "Adopt a repository",
        ],
    );
    let link = view
        .tree()
        .nodes()
        .iter()
        .find(|n| n.id == node(view.tree(), Role::Link, "Open work/hello on Forgejo ↗".as_bytes()))
        .unwrap();
    assert_eq!(link.external.as_deref(), Some(b"https://git.example.org/work/hello".as_slice()));
    assert_eq!(
        press(&root, Role::Link, b"Adopt a repository"),
        d::Action::Pages(pages::Act::Go { address: Address::Adopt { project: f::PROJECT } })
    );
}

#[test]
fn owned_details_draw_the_forgejo_pin_and_keep_the_unpinned_words() {
    for (pins, runner) in [(true, true), (true, false), (false, false)] {
        let root = f::forge_root(
            Address::ConnectForge { forge: Some(Choice::Forge(HERE)) },
            pinned_rows(w::Connection::NotSetUp, pins, runner),
        );
        for view in [f::page_view(&root), f::browser_page_view(&root)] {
            let expected = if pins {
                "temper box downloads Forgejo 17.0.0 and its runner"
            } else {
                "temper box downloads Forgejo and its runner"
            };
            words_in_order(view.tree(), &[expected, "starts them, and makes your first repository"]);
        }
    }
}

#[test]
fn owned_fetch_steps_draw_both_pins_or_keep_the_unpinned_words() {
    for (pins, runner, fetching, fetched) in [
        (true, true, "Fetching Forgejo 17.0.0 and its runner 11.2.1", "Fetched Forgejo 17.0.0 and its runner 11.2.1"),
        (true, false, "Fetching Forgejo and its runner", "Fetched Forgejo and its runner"),
        (false, false, "Fetching Forgejo and its runner", "Fetched Forgejo and its runner"),
    ] {
        for (step, expected) in [(w::SetUpStep::Fetch, fetching), (w::SetUpStep::Prepare, fetched)] {
            let connection = w::Connection::SettingUp(
                w::SettingUp::new(&w::CEILINGS, w::SettingUpParts { step: Some(step), fetched: None }).unwrap(),
            );
            let root = f::forge_root(
                Address::ConnectForge { forge: Some(Choice::Forge(HERE)) },
                pinned_rows(connection, pins, runner),
            );
            for view in [f::page_view(&root), f::browser_page_view(&root)] {
                words_in_order(
                    view.tree(),
                    &[expected, "Preparing", "Starting Forgejo and its runner", "Connecting", "Making"],
                );
            }
        }
    }
}

#[test]
fn owned_ready_offers_the_forgejo_link_after_adopt() {
    for facts_present in [true, false] {
        let root = f::forge_root(
            Address::ConnectForge { forge: Some(Choice::Forge(HERE)) },
            super::welcome::owned_adopted_rows(),
        );
        let mut view = f::view_of(f::browser_words());
        if facts_present {
            let mut facts = crate::forge_sign_in::Forges::new();
            facts.read("3 forgejo http://127.0.0.1:3000 owned");
            view.forge_sign_in(facts);
        }
        f::build(&mut view, &root);
        let link = f::find(view.tree(), Role::Link, "Open Forgejo ↗".as_bytes());
        assert_eq!(link.is_some(), facts_present);
        if let Some(link) = link {
            assert_eq!(
                crate::href(&mut view, link).as_deref(),
                Some(b"http://127.0.0.1:3000/user/oauth2/temper".as_slice())
            );
            words_in_order(view.tree(), &["Adopt a repository", "Open Forgejo ↗"]);
        }
    }
}

#[test]
fn the_connected_forge_offers_its_links_as_terminal_commands() {
    let mut view = f::page_view(&here(connected()));
    let control = view
        .tree()
        .nodes()
        .iter()
        .rev()
        .find(|node| node.role == Role::Link && node.name.as_deref() == Some(b"Adopt a repository"))
        .unwrap()
        .id;
    let (id, first) = f::terminal_command(
        view.tree(),
        &view.limits().tree,
        control,
        b"Adopt a repository",
        b"Connected Forgejo on this machine.",
    );
    assert_eq!(
        crate::act(&mut view, id, first),
        Some(d::Action::Pages(pages::Act::Go { address: Address::Adopt { project: f::PROJECT } }))
    );
}

#[test]
fn an_unfinished_github_offers_go_on_before_start_over() {
    let mut rows = github_rows(false);
    let mut parts = rows.get(0).unwrap().clone().into_parts();
    parts.app = Some(w::AppWay::Manifest);
    *rows.get_mut(0).unwrap() = w::ForgeRow::new(&w::CEILINGS, parts).unwrap();
    let root = f::forge_root(Address::Welcome { step: Step::Forge { forge: None } }, rows);
    let view = f::page_view(&root);
    words_in_order(view.tree(), &["FORGE · not finished", "Go on", "Start over"]);
    assert_eq!(
        press(&root, Role::Button, b"Go on"),
        d::Action::Forge(d::forge::Act::Forges(d::forge::forges::Act::GoOn {
            forge: GITHUB,
            back: b"/welcome?at=forge".as_slice().into()
        }))
    );
}

#[test]
fn the_managed_address_sets_host_port_and_tls() {
    for (address, host, port, tls) in [
        (b"https://forge.example:8443/".as_slice(), b"forge.example".as_slice(), 8443, true),
        (b"http://forge.example:3000", b"forge.example", 3000, false),
        (b"forge.example", b"forge.example", 443, true),
        (b"http://[::1]:3000", b"::1", 3000, false),
    ] {
        let mut root = f::forge_root(Address::ConnectForge { forge: Some(Choice::Managed) }, List::with_capacity(0));
        let mut view = f::page_view(&root);
        let id = node(view.tree(), Role::TextField, b"Its address");
        let action = crate::act(&mut view, id, Act::Input { value: address.into() }).unwrap();
        f::root_event(&mut root, d::Event::Act { action });
        let d::Action::Forge(d::forge::Act::Forges(d::forge::forges::Act::Managed { place })) =
            press(&root, Role::Button, b"Connect as its admin")
        else {
            panic!("managed connect")
        };
        assert_eq!(place.host.as_ref(), host);
        assert_eq!(place.port, port);
        assert_eq!(place.tls.as_deref(), if tls { Some(host) } else { None });
    }
}
#[test]
fn an_invalid_managed_address_cannot_send() {
    for address in [
        b"".as_slice(),
        b"https://",
        b"http://forge.example:0",
        b"https://forge.example:65536",
        b"https://forge.example/path",
        b"https://person@forge.example",
    ] {
        let mut root = f::forge_root(Address::ConnectForge { forge: Some(Choice::Managed) }, List::with_capacity(0));
        let mut view = f::page_view(&root);
        let id = node(view.tree(), Role::TextField, b"Its address");
        let action = crate::act(&mut view, id, Act::Input { value: address.into() }).unwrap();
        f::root_event(&mut root, d::Event::Act { action });
        let view = f::page_view(&root);
        assert!(has_state(view.tree(), Role::Button, b"Connect as its admin", State::Disabled));
    }
}

#[test]
fn starting_over_waits_for_the_removal_without_sending_twice() {
    let mut root = f::forge_root(Address::Welcome { step: Step::Forge { forge: None } }, github_rows(false));
    let action = press(&root, Role::Button, b"Start over");
    f::root_event(&mut root, d::Event::Act { action });
    let view = f::page_view(&root);
    assert!(root.forge().setup_form().removal.as_ref().unwrap().request.is_some());
    assert!(has_state(view.tree(), Role::Button, b"Start over", State::Disabled));
    assert!(f::find(view.tree(), Role::Button, b"Remove").is_none());
    words_in_order(view.tree(), &["The GitHub App stays yours: delete it in its settings.", "Start over"]);
}

#[test]
fn a_connected_github_row_shows_the_refused_log_origin() {
    let mut rows = github_rows(true);
    let mut parts = rows.get(0).unwrap().clone().into_parts();
    parts.ended = Some(
        w::ConnectStopped::new(
            &w::CEILINGS,
            w::ConnectStoppedParts { why: b"Job-log origin refused: https://unrelated.test:443".as_slice().into() },
        )
        .unwrap(),
    );
    *rows.get_mut(0).unwrap() = w::ForgeRow::new(&w::CEILINGS, parts).unwrap();
    let root = f::forge_root(Address::Forges, rows);
    let view = f::browser_page_view(&root);
    words_in_order(view.tree(), &["Job-log origin refused: https://unrelated.test:443"]);
}

fn tokens_row(connection: w::Connection, stopped: bool) -> w::ForgeRow {
    let mut parts = row(HERE, w::Kind::Forgejo, w::Mode::Connected, connection, w::Offer::NotOffered).into_parts();
    if stopped {
        parts.ended = Some(
            w::ConnectStopped::new(
                &w::CEILINGS,
                w::ConnectStoppedParts {
                    why: b"the git token answers as another user than the API token".as_slice().into(),
                },
            )
            .unwrap(),
        );
    }
    w::ForgeRow::new(&w::CEILINGS, parts).unwrap()
}
fn tokens_send(root: &mut d::Domain) {
    let action = press(root, Role::Button, b"Connect with its tokens");
    f::root_event(root, d::Event::Act { action });
}
fn tokens_bytes(seed: u64) -> Box<[u8]> {
    let mut rng = skein_lib::Rng::new(seed);
    let mut bytes = skein_lib::bytes::zeroed(32);
    for byte in &mut bytes {
        *byte = b'a'.checked_add(u8::try_from(rng.next_u64() % 26).unwrap()).unwrap();
    }
    bytes
}
fn no_tokens(tree: &skein_ui::Tree, values: &[Box<[u8]>]) {
    for node in tree.nodes() {
        for value in [node.name.as_deref(), node.description.as_deref(), node.value.as_ref().map(|v| v.text.as_ref())]
            .into_iter()
            .flatten()
        {
            for secret in values {
                assert!(skein_lib::bytes::find(value, secret).is_none());
            }
        }
        for run in &node.text {
            for secret in values {
                assert!(skein_lib::bytes::find(&run.text, secret).is_none());
            }
        }
    }
    let mut print = skein_ui_text::Print::new(&f::tree_limits()).unwrap();
    skein_ui_text::render(tree, b"temper", None, &mut print);
    for secret in values {
        assert!(skein_lib::bytes::find(print.text.as_slice(), secret).is_none());
    }
}
#[test]
fn switching_connect_ways_replaces_the_previous_fields() {
    let mut root = f::forge_root(Address::ConnectForge { forge: Some(Choice::Managed) }, List::with_capacity(0));
    let mut view = f::page_view(&root);
    let before: Vec<_> = view.tree().nodes().iter().filter(|n| n.role == Role::TextField).map(|n| n.id).collect();
    let choice = node(view.tree(), Role::Choice, "With its bot’s tokens".as_bytes());
    let action = crate::act(&mut view, choice, Act::Toggle).unwrap();
    f::root_event(&mut root, d::Event::Act { action });
    f::build(&mut view, &root);
    for name in [b"API token".as_slice(), b"Git token", b"Webhook secret"] {
        let id = node(view.tree(), Role::TextField, name);
        assert!(!before.contains(&id), "{} reuses the other way’s field", String::from_utf8_lossy(name));
    }
}

#[test]
fn your_forgejo_switches_forms_and_keeps_all_bot_tokens_secret() {
    let mut root = f::forge_root(Address::ConnectForge { forge: Some(Choice::Managed) }, List::with_capacity(0));
    let values = [tokens_bytes(1), tokens_bytes(2), tokens_bytes(3)];
    let view = f::page_view(&root);
    words_in_order(
        view.tree(),
        &[
            "Its address",
            "How temper connects",
            "As its admin",
            "With its bot’s tokens",
            "The site admin’s token",
            "Scopes: write:admin, read:user, write:organization, write:repository",
            "First repository",
            "Connect as its admin",
            "Other forges",
        ],
    );
    node(view.tree(), Role::Disclosure, b"Its runner label: docker");
    assert!(f::find(view.tree(), Role::TextField, b"API token").is_none());
    let action = press(&root, Role::Choice, "With its bot’s tokens".as_bytes());
    f::root_event(&mut root, d::Event::Act { action });
    let mut view = f::page_view(&root);
    words_in_order(
        view.tree(),
        &[
            "API token",
            "Scopes: read:user, write:issue, write:repository",
            "Git token",
            "Scopes: write:repository, read:user",
            "A webhook secret, if you made hooks to temper’s listener",
            "Webhook secret",
            "Connect with its tokens",
            "Other forges",
        ],
    );
    assert!(f::find(view.tree(), Role::TextField, "The site admin’s token".as_bytes()).is_none());
    for (name, value) in
        [(b"API token".as_slice(), &values[0]), (b"Git token", &values[1]), (b"Webhook secret", &values[2])]
    {
        let id = node(view.tree(), Role::TextField, name);
        assert!(crate::act(&mut view, id, Act::Input { value: value.clone() }).is_none());
        let field = view.tree().nodes().iter().find(|n| n.id == id).unwrap();
        assert_eq!(field.entry, Some(skein_ui::EntryKind::Secret));
        assert!(field.value.as_ref().unwrap().text.is_empty());
        admin_input(&mut root, name, value);
    }
    no_tokens(f::page_view(&root).tree(), &values);
    admin_input(&mut root, b"Its address", b"https://forge.example:8443");
    let action = press(&root, Role::Button, b"Connect with its tokens");
    let d::Action::Forge(d::forge::Act::Forges(d::forge::forges::Act::Connected { place })) = &action else {
        panic!("connected place")
    };
    assert_eq!(place.host.as_ref(), b"forge.example");
    assert_eq!(place.port, 8443);
    assert_eq!(place.tls.as_deref(), Some(b"forge.example".as_slice()));
    f::root_event(&mut root, d::Event::Act { action });
    for field in [d::forge::tokens::Field::Api, d::forge::tokens::Field::Git, d::forge::tokens::Field::Webhook] {
        assert!(!root.forge().tokens_form().has(field));
    }
    no_tokens(f::page_view(&root).tree(), &values);
}
#[test]
fn bot_tokens_local_refusals_mark_missing_tokens_and_oversized_bundles() {
    for (field, value, refusal) in [
        (b"API token".as_slice(), Box::<[u8]>::default(), "Required."),
        (b"Git token".as_slice(), Box::<[u8]>::default(), "Required."),
        (
            b"API token".as_slice(),
            skein_lib::bytes::zeroed(4096),
            "The pasted values are past 4 KiB, the most a request carries.",
        ),
        (
            b"Webhook secret".as_slice(),
            skein_lib::bytes::zeroed(4096),
            "The pasted values are past 4 KiB, the most a request carries.",
        ),
    ] {
        let mut root = admin_root(Some(Choice::Forge(HERE)), tokens_row(w::Connection::NotConnected, false));
        admin_input(&mut root, b"API token", &tokens_bytes(1));
        admin_input(&mut root, b"Git token", &tokens_bytes(2));
        admin_input(&mut root, field, &value);
        tokens_send(&mut root);
        let view = f::page_view(&root);
        words_in_order(view.tree(), &[refusal]);
        assert!(has_state(view.tree(), Role::TextField, field, State::Invalid));
        assert!(f::find(view.tree(), Role::TextField, "The site admin’s token".as_bytes()).is_none());
    }
}
#[test]
fn bot_tokens_connecting_and_ready_use_their_own_words() {
    let root = admin_root(Some(Choice::Forge(HERE)), tokens_row(w::Connection::Connecting, false));
    let view = f::page_view(&root);
    words_in_order(view.tree(), &["Checking that both answer as one user, not a site admin."]);
    assert!(f::find(view.tree(), Role::TextField, b"API token").is_none());
    let root = admin_root(Some(Choice::Forge(HERE)), tokens_row(connected(), false));
    words_in_order(f::page_view(&root).tree(), &["Connected Forgejo.", "temper · user 42"]);
}
#[test]
fn bot_tokens_stopped_form_is_alone_with_the_last_connect_under_its_button() {
    for connection in [
        w::Connection::NotConnected,
        w::Connection::Stopped(
            w::Stopped::new(&w::CEILINGS, w::StoppedParts { reason: b"stopped".as_slice().into() }).unwrap(),
        ),
    ] {
        let root = admin_root(Some(Choice::Forge(HERE)), tokens_row(connection, true));
        let view = f::page_view(&root);
        words_in_order(
            view.tree(),
            &[
                "Connect with its tokens",
                "The last connect stopped: the git token answers as another user than the API token",
            ],
        );
        for name in [b"API token".as_slice(), b"Git token", b"Webhook secret"] {
            let id = node(view.tree(), Role::TextField, name);
            let entry = view.tree().nodes().iter().find(|n| n.id == id).unwrap();
            assert_eq!(entry.entry, Some(skein_ui::EntryKind::Secret));
            assert!(entry.value.as_ref().unwrap().text.is_empty());
        }
        assert!(f::find(view.tree(), Role::TextField, b"Organization").is_none());
        assert!(f::find(view.tree(), Role::Button, b"Connect as its admin").is_none());
    }
}
#[test]
fn bot_tokens_connected_refusal_is_under_the_button_and_clears_the_fields() {
    let mut root = admin_root(Some(Choice::Forge(HERE)), tokens_row(w::Connection::NotConnected, false));
    admin_input(&mut root, b"API token", &tokens_bytes(1));
    admin_input(&mut root, b"Git token", &tokens_bytes(2));
    let action = press(&root, Role::Button, b"Connect with its tokens");
    let token = attempt(drive(&mut root, d::Event::Act { action }));
    answer(
        &mut root,
        token,
        w::Outcome::ConnectRefused(
            w::ConnectRefused::new(&w::CEILINGS, w::ConnectRefusedParts { why: w::ForgeRefusal::Connected }).unwrap(),
        ),
    );
    words_in_order(
        f::page_view(&root).tree(),
        &["Connect with its tokens", "Forge 3 is connected already: replace its tokens on Forges."],
    );
    assert!(!root.forge().tokens_form().has(d::forge::tokens::Field::Api));
}

fn unfinished_root(address: Address) -> d::Domain {
    let mut rows = List::with_capacity(1);
    rows.push(row(HERE, w::Kind::Forgejo, w::Mode::Owned, w::Connection::Connecting, w::Offer::Offered)).unwrap();
    f::forge_root(address, rows)
}

#[test]
fn start_over_names_where_it_ends() {
    for (address, end) in [
        (Address::ConnectForge { forge: None }, d::forge::forges::StartOverEnd::Forges),
        (Address::Welcome { step: Step::Forge { forge: None } }, d::forge::forges::StartOverEnd::Welcome),
    ] {
        let root = unfinished_root(address);
        for mut view in [f::page_view(&root), f::browser_page_view(&root)] {
            let start = node(view.tree(), Role::Button, b"Start over");
            assert_eq!(
                crate::act(&mut view, start, Act::Press),
                Some(d::Action::Forge(d::forge::Act::Forges(d::forge::forges::Act::StartOver { forge: HERE, end })))
            );
        }
    }
}

#[test]
fn deferred_setups_have_four_steps_and_finish_at_connected() {
    let mut copies = List::with_capacity(1);
    copies.push(Box::from(b"https://source.example/owner/repo".as_slice())).unwrap();
    let copies = w::CopyAddresses::new(&w::CEILINGS, w::CopyAddressesParts { addresses: copies }).unwrap();
    for starts_with in [w::StartsWith::Nothing, w::StartsWith::Copies(copies)] {
        for (connection, done) in [(w::Connection::Connecting, 3), (connected(), 4)] {
            let mut parts = row(HERE, w::Kind::Forgejo, w::Mode::Owned, connection, w::Offer::Offered).into_parts();
            parts.starts_with = starts_with.clone();
            let mut rows = List::with_capacity(1);
            rows.push(w::ForgeRow::new(&w::CEILINGS, parts).unwrap()).unwrap();
            let root = f::forge_root(Address::Welcome { step: Step::Forge { forge: Some(Choice::Forge(HERE)) } }, rows);
            if done == 4 {
                assert!(pages::welcome::forge_connected(d::forge::connect::row(HERE, root.forges(), None).unwrap()));
                continue;
            }
            for view in [f::page_view(&root), f::browser_page_view(&root)] {
                let tree = view.tree();
                let list = node(tree, Role::List, b"Each step");
                let start = tree.nodes().iter().position(|n| n.id == list).unwrap();
                let size = usize::try_from(tree.nodes()[start].size).unwrap();
                let items: Vec<_> = tree.nodes()[start..][..size].iter().filter(|n| n.role == Role::Item).collect();
                assert_eq!(items.len(), 4);
                assert_eq!(
                    items.iter().filter(|n| n.styles.as_slice().contains(&crate::look::FLOW_DONE)).count(),
                    done
                );
            }
        }
    }
}
