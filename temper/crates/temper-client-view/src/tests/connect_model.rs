#![allow(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::disallowed_macros,
    reason = "semantic tree assertions"
)]
use super::{accounts::press, fixtures};
use jig_client::contract as c;
use skein_lib::List;
use skein_ui::{Act, Role};
use temper_client_domain::{self as d, pages::connect_model::Act as M};
const PROVIDER: u32 = 17;
const LOGIN: u32 = 91;
const WAITING: &str = "Claude Code is signing in, in the terminal where temper runs";
const APPROVE: &str = "Approve it on Anthropic’s page, in the tab Claude Code opened. This page moves on by itself.";
const HELP: &str = "Claude Code’s sign-in is in the terminal where temper runs. If no tab opened, press c there to copy its link. If Anthropic’s page shows a code, paste it there.";
const NOT_FINISHED: &str = "Claude Code’s sign-in didn’t finish.";

pub(super) fn terminal_root(welcome: bool, doc: c::Accounts) -> (d::Domain, skein_lib::Token) {
    let address = if welcome {
        d::Address::Welcome { step: d::Step::Model { way: None } }
    } else {
        d::Address::ConnectModel { way: None }
    };
    let (mut root, me_watch, _, _, watch) = fixtures::root_at_full(address, &fixtures::limits(), b"Shop");
    let mut me = d::pages::accounts::current_me(root.client()).unwrap().clone();
    me.providers = List::from_boxed(Box::new([
        offer(PROVIDER, b"openai", true, false),
        offer(PROVIDER + 1, b"anthropic", false, true),
    ]));
    fixtures::root_event(
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
    super::accounts::deliver(&mut root, watch.unwrap(), doc);
    (root, watch.unwrap())
}
pub(super) fn terminal_document(lendable: bool) -> c::Accounts {
    let mut doc = super::accounts::document(0);
    doc.lendable = lendable;
    doc.usable = false;
    doc.settled = true;
    doc.cut = false;
    doc
}
fn terminal_lending() -> c::TerminalLogin {
    c::TerminalLogin { number: LOGIN, provider: PROVIDER + 1, name: b"claude-pat".as_slice().into(), cancelled: false }
}
fn terminal_source(usable: bool) -> c::Accounts {
    let mut doc = super::accounts::card_document(b"anthropic", b"claude-pat", c::AccountKind::Program);
    doc.accounts.get_mut(0).unwrap().provider = Some(PROVIDER + 1);
    doc.accounts.get_mut(0).unwrap().sources.get_mut(0).unwrap().usable = usable;
    doc.accounts.get_mut(0).unwrap().sources.get_mut(0).unwrap().number = 43;
    doc.usable = usable;
    doc.settled = true;
    doc.cut = false;
    doc.signing.clear();
    doc
}
fn named_at(tree: &skein_ui::Tree, role: Role, name: &str) -> usize {
    tree.nodes()
        .iter()
        .position(|node| node.role == role && node.name.as_deref() == Some(name.as_bytes()))
        .unwrap_or_else(|| {
            panic!(
                "missing {role:?}: {name}; {:?}",
                tree.nodes()
                    .iter()
                    .filter(
                        |n| [Role::Heading { level: 1 }, Role::Status, Role::Region, Role::Paragraph].contains(&n.role)
                    )
                    .map(|n| (n.role, n.name.as_deref().map(String::from_utf8_lossy)))
                    .collect::<Vec<_>>()
            )
        })
}
pub(super) fn assert_terminal_cards(root: &d::Domain, welcome: bool, lendable: bool) {
    for w in [fixtures::words(), fixtures::browser_words()] {
        let mut view = fixtures::view_of(w);
        let tree = fixtures::build(&mut view, root);
        let cards: Vec<_> = tree
            .nodes()
            .iter()
            .filter(|n| {
                n.role == Role::Region
                    && [Some(b"Claude".as_slice()), Some(b"ChatGPT".as_slice())].contains(&n.name.as_deref())
            })
            .map(|n| n.name.as_deref().unwrap())
            .collect();
        assert_eq!(cards, [b"Claude".as_slice(), b"ChatGPT"]);
        let claude = named_at(tree, Role::Region, "Claude");
        let parts = super::inbox::subtree(tree, claude);
        let named: Vec<_> = parts
            .iter()
            .filter(|n| [Role::Heading { level: 2 }, Role::Paragraph, Role::Link, Role::Button].contains(&n.role))
            .map(|n| (n.role, n.name.as_deref().unwrap()))
            .collect();
        let (role, note, label) = if lendable {
            (Role::Button, "Signs in from the terminal where temper runs", "Sign in to Claude Code")
        } else {
            (Role::Link, "One command in a terminal, on this machine", "Use your Claude subscription")
        };
        assert_eq!(
            named,
            [
                (Role::Heading { level: 2 }, b"Claude".as_slice()),
                (Role::Paragraph, b"Your Claude subscription, through a Claude Code temper keeps apart from your own."),
                (Role::Paragraph, note.as_bytes()),
                (role, label.as_bytes()),
            ]
        );
        let primary = if welcome { crate::look::FLOW_PRIMARY } else { crate::look::CONNECT_PRIMARY };
        let secondary = if welcome { crate::look::FLOW_SECONDARY } else { crate::look::CONNECT_SECONDARY };
        let act = &tree.nodes()[named_at(tree, role, label)];
        assert!(act.styles.as_slice().contains(&primary));
        assert_eq!(tree.nodes().iter().filter(|n| n.styles.as_slice().contains(&primary)).count(), 1);
        let chatgpt = &tree.nodes()[named_at(tree, Role::Button, "Continue with ChatGPT")];
        assert!(chatgpt.styles.as_slice().contains(&secondary));
        named_at(tree, Role::Paragraph, "Sign in, in a new tab");
        assert!(!parts.iter().any(|n| n.role == Role::Code || n.role == Role::TextField));
        if !welcome {
            assert!(tree.nodes()[claude].styles.as_slice().contains(&crate::look::CONNECT_DEFAULT));
        }
        let id = act.id;
        if lendable {
            assert_eq!(
                crate::act(&mut view, id, Act::Press),
                Some(d::Action::Pages(d::pages::Act::ConnectModel(M::LoginProgram {
                    name: root.pages().account_name()
                })))
            );
        } else {
            assert_eq!(
                crate::act(&mut view, id, Act::Press),
                Some(d::Action::Pages(d::pages::Act::Go {
                    address: d::pages::connect_model::address(
                        root.pages().address(),
                        Some(d::ModelWay { source: None, provider: PROVIDER + 1, kind: d::WayKind::Program })
                    )
                }))
            );
        }
    }
}
#[test]
fn choose_puts_claude_first_with_the_documents_terminal_form() {
    for lendable in [false, true] {
        let (root, _) = terminal_root(false, terminal_document(lendable));
        assert_terminal_cards(&root, false, lendable);
    }
}
fn assert_terminal_note(root: &d::Domain, welcome: bool) {
    for w in [fixtures::words(), fixtures::browser_words()] {
        let browser = w.shell == crate::words::Shell::Browser;
        let mut view = fixtures::view_of(w);
        let tree = fixtures::build(&mut view, root);
        let title = named_at(tree, Role::Status, WAITING);
        let body = named_at(tree, Role::Paragraph, APPROVE);
        let cancel = named_at(tree, Role::Button, "Cancel");
        let help = named_at(tree, Role::Paragraph, HELP);
        assert!(title < body && body < cancel && cancel < help);
        assert!(tree.nodes()[body].styles.as_slice().contains(&crate::look::FLOW_MUTED));
        assert!(tree.nodes()[help].styles.as_slice().contains(&crate::look::FLOW_MUTED));
        assert!(tree.nodes()[cancel].styles.as_slice().contains(&if welcome {
            crate::look::FLOW_SECONDARY
        } else {
            crate::look::CONNECT_SECONDARY
        }));
        assert_eq!(tree.nodes().iter().any(|n| n.styles.as_slice().contains(&crate::look::WELCOME_SPIN)), browser);
        if browser {
            let disclosure = named_at(tree, Role::Disclosure, "No tab opened, or a code to paste?");
            assert!(cancel < disclosure && disclosure < help);
        }
        let note = tree.nodes().iter().position(|n| n.styles.as_slice().contains(&crate::look::WELCOME_AWAY)).unwrap();
        assert!(!super::inbox::subtree(tree, note).iter().any(|n| n.role == Role::TextField));
        assert!(fixtures::find(tree, Role::Region, b"Claude").is_none());
        if browser {
            let disclosure = named_at(tree, Role::Disclosure, "No tab opened, or a code to paste?");
            assert!(!super::inbox::subtree(tree, disclosure).iter().any(|n| n.role == Role::TextField));
        }
        let lead = if welcome {
            "temper’s agents run on a plan you sign in to."
        } else {
            "temper’s agents run on a plan you sign in to, or on an API key."
        };
        assert!(fixtures::find(tree, Role::Paragraph, lead.as_bytes()).is_none());
        let cancel_id = tree.nodes()[cancel].id;
        assert_eq!(
            crate::act(&mut view, cancel_id, Act::Press),
            Some(d::Action::Pages(d::pages::Act::ConnectModel(M::CancelLogin)))
        );
    }
}
pub(super) fn terminal_note_states(welcome: bool) {
    let (mut root, watch) = terminal_root(welcome, terminal_document(true));
    let name = root.pages().account_name();
    model_action(&mut root, M::LoginProgram { name });
    assert_terminal_note(&root, welcome);
    let mut doc = terminal_document(true);
    doc.lending = Some(terminal_lending());
    super::accounts::deliver(&mut root, watch, doc.clone());
    assert_terminal_note(&root, welcome);
    let (root, _) = terminal_root(welcome, doc);
    assert_terminal_note(&root, welcome);
    let mut doc = terminal_source(false);
    doc.settled = false;
    doc.lent_end = Some(c::AccountLentEnd { number: LOGIN, end: c::LoginEnd::SignedIn { source: 43 } });
    let (root, _) = terminal_root(welcome, doc);
    assert_terminal_note(&root, welcome);
}
#[test]
fn settings_keeps_the_terminal_login_note_across_documents_and_reopening() {
    terminal_note_states(false);
}
pub(super) fn terminal_ending_rows(welcome: bool) {
    let (mut root, watch) = terminal_root(welcome, terminal_document(true));
    let name = root.pages().account_name();
    let attempt = sent(&mut root, M::LoginProgram { name });
    answer(&mut root, attempt, c::DocumentOutcome::AccountLoginAsked { number: LOGIN });
    model_action(&mut root, M::CancelLogin);
    let mut doc = terminal_document(true);
    doc.lent_end = Some(c::AccountLentEnd { number: LOGIN, end: c::LoginEnd::Cancelled });
    super::accounts::deliver(&mut root, watch, doc);
    // The cancel's answer may still be out: the end leaves no progress or result line.
    for view in [fixtures::page_view(&root), fixtures::browser_page_view(&root)] {
        let tree = view.tree();
        let heading = named_at(tree, Role::Heading { level: 1 }, "Connect a model");
        let card = named_at(tree, Role::Region, "Claude");
        let between: Vec<_> = tree.nodes()[heading.checked_add(1).unwrap()..card]
            .iter()
            .filter(|n| n.role == Role::Paragraph || n.role == Role::Status)
            .map(|n| n.name.as_deref().unwrap_or_default())
            .collect();
        let lead = if welcome {
            "temper’s agents run on a plan you sign in to."
        } else {
            "temper’s agents run on a plan you sign in to, or on an API key."
        };
        assert_eq!(between, [lead.as_bytes()]);
    }
    for lendable in [false, true] {
        for (end, line) in [
            (c::LoginEnd::NotFinished { exit: None }, Some(NOT_FINISHED)),
            (c::LoginEnd::NotFinished { exit: Some(c::LoginExit::Exited { code: 1 }) }, Some(NOT_FINISHED)),
            (c::LoginEnd::SignedIn { source: 43 }, Some(NOT_FINISHED)),
            (
                c::LoginEnd::Refused { refusal: c::AccountRefusal::Uses },
                Some("The provider does not permit these uses."),
            ),
            (c::LoginEnd::Cancelled, None),
        ] {
            let mut doc = terminal_source(false);
            doc.lendable = lendable;
            // The failed source is not a connected card in these rows.
            doc.accounts.clear();
            doc.lent_end = Some(c::AccountLentEnd { number: LOGIN, end });
            let (root, _) = terminal_root(welcome, doc);
            for view in [fixtures::page_view(&root), fixtures::browser_page_view(&root)] {
                let tree = view.tree();
                assert!(
                    fixtures::find(tree, Role::Region, b"Claude").is_some(),
                    "end {end:?}, lendable {lendable}: {:?}",
                    root.pages().connect_model().unwrap().login_state(root.client())
                );
                let claude = named_at(tree, Role::Region, "Claude");
                assert!(fixtures::find(tree, Role::Status, WAITING.as_bytes()).is_none());
                if let Some(line) = line {
                    let at = tree
                        .nodes()
                        .iter()
                        .enumerate()
                        .find(|(i, n)| {
                            n.role == Role::Paragraph
                                && super::inbox::subtree(tree, *i)
                                    .iter()
                                    .flat_map(|n| n.text.iter().flat_map(|r| r.text.iter().copied()))
                                    .collect::<Vec<_>>()
                                    == line.as_bytes()
                        })
                        .map(|(i, _)| i)
                        .unwrap();
                    assert!(at < claude);
                    assert!(tree.nodes()[at].styles.as_slice().contains(&crate::look::FLOW_MUTED));
                } else {
                    assert!(fixtures::find(tree, Role::Paragraph, NOT_FINISHED.as_bytes()).is_none());
                }
                assert!(
                    fixtures::find(
                        tree,
                        if lendable { Role::Button } else { Role::Link },
                        if lendable { b"Sign in to Claude Code" } else { b"Use your Claude subscription" }
                    )
                    .is_some()
                );
            }
        }
    }
    terminal_press_refusals(welcome);
}
fn terminal_press_refusals(welcome: bool) {
    for (refusal, line) in [
        (c::Refusal::NoTerminal, None),
        (c::Refusal::LoginUnderWay, None),
        (c::Refusal::NotOffered, Some("This provider offers you no program login. Choose another way to connect.")),
    ] {
        let (mut root, _) = terminal_root(welcome, terminal_document(true));
        let name = root.pages().account_name();
        let attempt = sent(&mut root, M::LoginProgram { name });
        fixtures::root_event(
            &mut root,
            d::Event::Client {
                event: jig_client::Event::Up {
                    up: jig_client::Up::Answer {
                        attempt,
                        envelope: skein_lib::RequestEnvelope::Final,
                        why: None,
                        answer: Some(jig_client::Answered::Jig(c::Answer::Refused {
                            refusal: refusal.clone(),
                            at: None,
                        })),
                    },
                },
            },
        );
        if refusal == c::Refusal::LoginUnderWay {
            assert_terminal_note(&root, welcome);
        } else {
            for view in [fixtures::page_view(&root), fixtures::browser_page_view(&root)] {
                let tree = view.tree();
                assert!(
                    fixtures::find(tree, Role::Region, b"Claude").is_some(),
                    "press refused {refusal:?}: {:?}",
                    root.pages().connect_model().unwrap().login_state(root.client())
                );
                let claude = named_at(tree, Role::Region, "Claude");
                if refusal == c::Refusal::NoTerminal {
                    named_at(tree, Role::Link, "Use your Claude subscription");
                }
                if let Some(line) = line {
                    assert!(named_at(tree, Role::Paragraph, line) < claude);
                }
            }
        }
    }
}
#[test]
fn settings_terminal_login_endings_and_refusals_put_the_cards_back() {
    terminal_ending_rows(false);
}
#[test]
fn a_connected_claude_card_keeps_its_way_and_defaults_to_chatgpt() {
    let mut doc = terminal_source(false);
    doc.lendable = true;
    let (root, _) = terminal_root(false, doc);
    for mut view in [fixtures::page_view(&root), fixtures::browser_page_view(&root)] {
        let tree = view.tree();
        let claude = named_at(tree, Role::Region, "Claude");
        assert!(!tree.nodes()[claude].styles.as_slice().contains(&crate::look::CONNECT_DEFAULT));
        let chatgpt = named_at(tree, Role::Region, "ChatGPT");
        assert!(tree.nodes()[chatgpt].styles.as_slice().contains(&crate::look::CONNECT_DEFAULT));
        let another = tree.nodes()[named_at(tree, Role::Link, "Add another plan")].id;
        assert_eq!(
            crate::act(&mut view, another, Act::Press),
            Some(d::Action::Pages(d::pages::Act::Go {
                address: d::Address::ConnectModel {
                    way: Some(d::ModelWay { source: None, provider: PROVIDER + 1, kind: d::WayKind::Program })
                }
            }))
        );
    }
}

fn model_action(root: &mut d::Domain, act: M) {
    fixtures::root_event(root, d::Event::Act { action: d::Action::Pages(d::pages::Act::ConnectModel(act)) });
}
fn offer(provider: u32, name: &[u8], sign_in: bool, program: bool) -> c::ProviderOffer {
    let mut programs = List::with_capacity(1);
    if program {
        programs.push(c::Text::new(Box::from(b"claude_code".as_slice()), 32).unwrap()).unwrap();
    }
    c::ProviderOffer {
        provider,
        name: text(name),
        key_bytes: 4096,
        sign_in,
        signed_in_plan: true,
        key: true,
        programs,
        programs_cut: false,
        none: false,
        manage_usage: None,
    }
}
pub(super) fn text(value: &[u8]) -> c::DocumentText {
    c::DocumentText::complete(c::Text::new(Box::from(value), 64).unwrap())
}
fn ways_root(offers: List<c::ProviderOffer>, playground: bool) -> d::Domain {
    let limits = fixtures::limits();
    let (mut root, watch, _) = fixtures::root_at(d::Address::ConnectModel { way: None }, &limits);
    let mut me = d::pages::accounts::current_me(root.client()).unwrap().clone();
    me.providers = offers;
    me.providers_cut = true;
    me.deployment_role = Some(c::HeldRole::Owner);
    if playground {
        return fixtures::playground_frame_root(
            d::Address::ConnectModel { way: None },
            fixtures::PROJECT,
            true,
            None,
            Some(me),
        );
    }
    fixtures::root_event(
        &mut root,
        d::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched { watch, event: c::WatchEvent::Delivered(c::Delivery::Me(Box::new(me))) },
            },
        },
    );
    root
}
/// `openai`'s sign-in and `anthropic`'s program, each provider taking keys too.
pub(super) fn plans_root() -> d::Domain {
    let mut providers = List::with_capacity(2);
    providers.push(offer(PROVIDER, b"openai", true, false)).unwrap();
    providers.push(offer(PROVIDER + 1, b"anthropic", false, true)).unwrap();
    ways_root(providers, false)
}
pub(super) fn offered_root() -> d::Domain {
    let mut providers = List::with_capacity(1);
    providers.push(offer(PROVIDER, b"openai", true, false)).unwrap();
    ways_root(providers, false)
}
fn control_names(root: &d::Domain) -> Vec<Vec<u8>> {
    let mut view = fixtures::view();
    fixtures::build(&mut view, root)
        .nodes()
        .iter()
        .filter(|node| node.role == Role::Button || node.role == Role::Link)
        .filter_map(|node| node.name.as_ref().map(|name| name.to_vec()))
        .collect()
}
#[test]
fn the_model_choose_names_the_plan_sign_in_and_key_ways() {
    let root = offered_root();
    let view = fixtures::page_view(&root);
    let primary: Vec<_> = view
        .tree()
        .nodes()
        .iter()
        .filter(|n| n.styles.as_slice().contains(&crate::look::CONNECT_PRIMARY))
        .map(|n| n.name.as_deref().unwrap())
        .collect();
    assert_eq!(primary, [b"Continue with ChatGPT".as_slice()]);
    let names = control_names(&root);
    assert!(names.contains(&b"Continue with ChatGPT".to_vec()));
    let key = fixtures::find(view.tree(), Role::Link, b"Add key").unwrap();
    assert!(
        view.tree()
            .nodes()
            .iter()
            .find(|n| n.id == key)
            .unwrap()
            .styles
            .as_slice()
            .contains(&crate::look::CONNECT_SECONDARY)
    );
    assert!(names.contains(&b"Add key".to_vec()));
}
#[test]
fn two_key_providers_name_each_key_way_by_its_brand() {
    let mut providers = List::with_capacity(2);
    providers.push(offer(PROVIDER, b"openai", false, false)).unwrap();
    providers.push(offer(PROVIDER + 1, b"anthropic", false, false)).unwrap();
    let mut root = ways_root(providers, false);
    let view = fixtures::page_view(&root);
    let shown = words(&root);
    assert!(shown.contains("OpenAI API key"));
    assert!(shown.contains("Anthropic API key"));
    let me = d::pages::accounts::current_me(root.client()).unwrap();
    let ways = d::pages::accounts::ways(&me.providers, None, c::AskAccountHolder::Own);
    for (provider, label) in
        [(PROVIDER, b"Use an OpenAI API key".as_slice()), (PROVIDER + 1, b"Use an Anthropic API key")]
    {
        assert_eq!(
            crate::pages::connect_model::label(
                me,
                d::pages::accounts::Way::Key { provider },
                &ways,
                &fixtures::words()
            )
            .as_ref(),
            label
        );
    }
    let more = super::inbox::styled(view.tree(), crate::look::CONNECT_MORE);
    let rows = super::inbox::subtree(view.tree(), more);
    assert!(
        rows.iter()
            .any(|node| node.role == Role::Heading { level: 2 } && node.name.as_deref() == Some(b"Or an API key"))
    );
    for name in [b"OpenAI API key".as_slice(), b"Anthropic API key"] {
        let row = fixtures::find(view.tree(), Role::Slot, name).unwrap();
        assert!(rows.iter().any(|node| node.id == row));
        let index = view.tree().nodes().iter().position(|node| node.id == row).unwrap();
        let content = super::inbox::subtree(view.tree(), index);
        assert!(content.iter().any(|node| node.role == Role::Link && node.name.as_deref() == Some(b"Add key")));
    }
    assert_eq!(
        view.tree()
            .nodes()
            .iter()
            .filter(|node| node.role == Role::Link && node.name.as_deref() == Some(b"Add key"))
            .count(),
        2
    );
    for (provider, heading, description) in [
        (
            PROVIDER,
            "An OpenAI API key",
            "From OpenAI’s platform, under API keys. What temper’s agents use is billed to the key’s organization.",
        ),
        (
            PROVIDER + 1,
            "An Anthropic API key",
            "From the Claude Console, under API keys. What temper’s agents use is billed to the key’s organization.",
        ),
    ] {
        let address =
            d::Address::ConnectModel { way: Some(d::ModelWay { source: None, provider, kind: d::WayKind::Key }) };
        let mut choose = fixtures::page_view(&root);
        let links: Vec<_> = choose
            .tree()
            .nodes()
            .iter()
            .filter(|node| node.role == Role::Link && node.name.as_deref() == Some(b"Add key"))
            .map(|node| node.id)
            .collect();
        assert!(links.into_iter().any(|id| crate::act(&mut choose, id, Act::Press)
            == Some(d::Action::Pages(d::pages::Act::Go { address: address.clone() }))));
        fixtures::root_event(&mut root, d::Event::Went { place: d::Place::Address(address) });
        let details = fixtures::page_view(&root);
        assert!(fixtures::find(details.tree(), Role::Heading { level: 1 }, heading.as_bytes()).is_some());
        assert!(words(&root).contains(description));
        fixtures::root_event(
            &mut root,
            d::Event::Went { place: d::Place::Address(d::Address::ConnectModel { way: None }) },
        );
    }
}
#[test]
fn another_providers_key_details_keep_its_platform() {
    let mut providers = List::with_capacity(1);
    providers.push(offer(PROVIDER, b"Example", false, false)).unwrap();
    let mut root = ways_root(providers, false);
    fixtures::root_event(
        &mut root,
        d::Event::Went {
            place: d::Place::Address(d::Address::ConnectModel {
                way: Some(d::ModelWay { source: None, provider: PROVIDER, kind: d::WayKind::Key }),
            }),
        },
    );
    assert!(words(&root).contains(
        "From Example’s platform, under API keys. What temper’s agents use is billed to the key’s organization."
    ));
}
#[test]
fn a_provider_without_a_plan_names_its_sign_in_by_its_brand() {
    let mut providers = List::with_capacity(1);
    let mut provider = offer(PROVIDER, b"anthropic", true, false);
    provider.signed_in_plan = false;
    providers.push(provider).unwrap();
    assert!(control_names(&ways_root(providers, false)).contains(&b"Sign in to Anthropic".to_vec()));
}
#[test]
fn the_playgrounds_provider_offers_no_way() {
    let mut providers = List::with_capacity(2);
    providers.push(offer(PROVIDER, crate::words::SCRIPTED_MODEL, true, true)).unwrap();
    providers.push(offer(PROVIDER + 1, b"openai", true, false)).unwrap();
    let names = control_names(&ways_root(providers, true));
    assert!(names.contains(&b"Continue with ChatGPT".to_vec()));
    assert!(names.contains(&b"Add key".to_vec()));
    assert!(
        !names.iter().any(|name| name
            .windows(crate::words::SCRIPTED_MODEL.len())
            .any(|part| part == crate::words::SCRIPTED_MODEL))
    );
}
#[test]
fn the_deployment_holder_offers_keys_only() {
    let mut providers = List::with_capacity(2);
    providers.push(offer(PROVIDER, b"openai", true, false)).unwrap();
    providers.push(offer(PROVIDER + 1, b"anthropic", false, true)).unwrap();
    let mut root = ways_root(providers, false);
    model_action(&mut root, M::Holder { holder: c::AskAccountHolder::Deployment });
    let names = control_names(&root);
    assert!(names.contains(&b"Use your accounts".to_vec()));
    assert!(!names.contains(&b"Use deployment accounts".to_vec()));
    assert_eq!(names.iter().filter(|name| name.as_slice() == b"Add key").count(), 2);
    assert!(!names.contains(&b"Continue with ChatGPT".to_vec()));
    assert!(!names.contains(&b"Use your Claude subscription".to_vec()));
}
#[test]
fn a_program_way_shows_the_default_name_and_the_command_on_each_host() {
    let mut providers = List::with_capacity(1);
    providers.push(offer(PROVIDER, b"anthropic", false, true)).unwrap();
    let mut root = ways_root(providers, false);
    let mut view = fixtures::view();
    press(&mut view, &mut root, Role::Link, b"Use your Claude subscription", Act::Press);
    let tree = fixtures::build(&mut view, &root);
    let field = tree.nodes().iter().find(|node| node.name.as_deref() == Some(b"Name")).unwrap();
    assert_eq!(field.value.as_ref().unwrap().text.as_ref(), b"claude-pat");
    assert!(fixtures::find(tree, Role::Button, "I’ve signed in".as_bytes()).is_some());
    assert!(
        String::from_utf8(fixtures::text(tree).into_vec())
            .unwrap()
            .contains("temper program-login --config CONFIG --data DATA --cache CACHE claude-pat")
    );
}

pub(super) fn flow_fixture(kind: d::WayKind) -> (d::Domain, skein_lib::Token) {
    flow_fixture_at(
        kind,
        d::Address::ConnectModel { way: Some(d::ModelWay { source: None, provider: PROVIDER, kind }) },
    )
}
fn flow_fixture_at(kind: d::WayKind, address: d::Address) -> (d::Domain, skein_lib::Token) {
    let (mut root, me_watch, _, _, accounts_watch) = fixtures::root_at_full(address, &fixtures::limits(), b"Shop");
    let mut me = d::pages::accounts::current_me(root.client()).unwrap().clone();
    me.providers = List::with_capacity(1);
    me.providers
        .push(offer(PROVIDER, if kind == d::WayKind::Program { b"anthropic" } else { b"openai" }, true, true))
        .unwrap();
    fixtures::root_event(
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
    (root, accounts_watch.unwrap())
}
pub(super) fn sent(root: &mut d::Domain, act: M) -> skein_lib::Token {
    let limits = fixtures::limits();
    let env = skein_lib::Env { now: skein_lib::Time::ZERO, wall: fixtures::NOW, limits };
    let mut out = skein_lib::Queue::uncapped();
    d::step(root, &env, d::Event::Act { action: d::Action::Pages(d::pages::Act::ConnectModel(act)) }, &mut out);
    d::step(root, &env, d::Event::Client { event: jig_client::Event::Stored }, &mut out);
    out.iter()
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
            | d::Request::Push(_)
            | d::Request::Fact { .. } => None,
        })
        .expect("flow sends one request")
}
pub(super) fn answer(root: &mut d::Domain, attempt: skein_lib::Token, outcome: c::DocumentOutcome) {
    fixtures::root_event(
        root,
        d::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Answer {
                    attempt,
                    envelope: skein_lib::RequestEnvelope::Final,
                    answer: Some(jig_client::Answered::Jig(c::Answer::Done(c::Outcome::Document(outcome)))),
                    why: None,
                },
            },
        },
    );
}
fn words(root: &d::Domain) -> String {
    String::from_utf8(fixtures::text(fixtures::page_view(root).tree()).into_vec()).unwrap()
}
pub(super) fn ending(end: c::AccountEnd, display: &[u8]) -> d::Domain {
    let (mut root, watch) = flow_fixture(d::WayKind::SignIn);
    let attempt = sent(
        &mut root,
        M::SignIn { name: [1; 16], way: d::pages::accounts::Way::SignIn { provider: PROVIDER, plan: true } },
    );
    answer(
        &mut root,
        attempt,
        c::DocumentOutcome::AccountSignInStarted {
            source: 43,
            address: Box::from(b"https://issuer.example/visit".as_slice()),
        },
    );
    let mut doc = super::accounts::card_document(b"openai", display, c::AccountKind::SignIn);
    doc.usable = match &end {
        c::AccountEnd::Made { declined, .. } | c::AccountEnd::Renewed { declined, .. } => !declined,
        c::AccountEnd::Refused { .. } | c::AccountEnd::Failed { .. } => false,
    };
    doc.accounts.get_mut(0).unwrap().manage_usage = Some(Box::from(b"https://provider.example/usage".as_slice()));
    doc.signing.clear();
    doc.last_end = Some(c::AccountLastEnd { source: 43, end });
    super::accounts::deliver(&mut root, watch, doc);
    root
}
#[test]
fn a_key_lands_on_accounts_with_its_line_and_chat_link() {
    let (mut root, _) = flow_fixture(d::WayKind::Key);
    let mut view = fixtures::view();
    assert!(fixtures::find(fixtures::build(&mut view, &root), Role::TextField, b"Key (pasted once)").is_some());
    press(
        &mut view,
        &mut root,
        Role::TextField,
        b"Key (pasted once)",
        Act::Input { value: Box::from(b"private-once".as_slice()) },
    );
    let attempt = sent(&mut root, M::Submit { name: [1; 16] });
    answer(&mut root, attempt, c::DocumentOutcome::AccountKeyKept { source: 43 });
    let text = words(&root);
    for expected in ["Accounts", "Added your OpenAI API key. temper’s agents now run on it."] {
        assert!(text.contains(expected), "missing {expected}");
    }
    assert!(!format!("{:?}", fixtures::page_view(&root)).contains("private-once"));
    let tree = fixtures::build(&mut view, &root);
    let node = fixtures::find(tree, Role::Link, b"Start a chat").unwrap();
    assert_eq!(
        crate::act(&mut view, node, Act::Press),
        Some(d::Action::Pages(d::pages::Act::Go { address: d::Address::NewChat { draft: false } }))
    );
}
#[test]
fn an_anthropic_key_lands_on_accounts_with_its_brand() {
    let mut root = plans_root();
    fixtures::root_event(
        &mut root,
        d::Event::Went {
            place: d::Place::Address(d::Address::ConnectModel {
                way: Some(d::ModelWay { source: None, provider: PROVIDER + 1, kind: d::WayKind::Key }),
            }),
        },
    );
    let mut view = fixtures::view();
    press(
        &mut view,
        &mut root,
        Role::TextField,
        b"Key (pasted once)",
        Act::Input { value: Box::from(b"private-once".as_slice()) },
    );
    let attempt = sent(&mut root, M::Submit { name: [1; 16] });
    answer(&mut root, attempt, c::DocumentOutcome::AccountKeyKept { source: 43 });
    assert!(words(&root).contains("Added your Anthropic API key. temper’s agents now run on it."));
}
#[test]
fn a_key_added_on_the_welcome_continues_to_its_ready() {
    let way = d::ModelWay { source: None, provider: PROVIDER, kind: d::WayKind::Key };
    let (mut root, watch) =
        flow_fixture_at(d::WayKind::Key, d::Address::Welcome { step: d::Step::Model { way: Some(way) } });
    let mut view = fixtures::view();
    fixtures::build(&mut view, &root);
    press(
        &mut view,
        &mut root,
        Role::TextField,
        b"Key, pasted once",
        Act::Input { value: Box::from(b"private-once".as_slice()) },
    );
    let attempt = sent(&mut root, M::Submit { name: [1; 16] });
    answer(&mut root, attempt, c::DocumentOutcome::AccountKeyKept { source: 43 });
    let mut doc = super::accounts::card_document(b"openai", b"", c::AccountKind::Key);
    doc.usable = true;
    super::accounts::deliver(&mut root, watch, doc);
    let tree = fixtures::build(&mut view, &root);
    assert!(!words(&root).contains("Added your OpenAI API key"));
    assert_eq!(fixtures::find(tree, Role::Link, b"Start a chat"), None);
    let node = fixtures::find(tree, Role::Link, b"Continue").unwrap();
    assert_eq!(
        crate::act(&mut view, node, Act::Press),
        Some(d::Action::Pages(d::pages::Act::Go { address: d::Address::Welcome { step: d::Step::Ready } }))
    );
}
#[test]
fn a_program_keeps_its_command_while_its_check_is_out() {
    let (mut root, _) = flow_fixture(d::WayKind::Program);
    for expected in [
        "Your Claude subscription",
        "temper sets Claude Code up on this machine in an environment of its own: its settings, sessions and sign-in stay apart from your own Claude Code. In a terminal here:",
        "temper program-login --config CONFIG --data DATA --cache CACHE claude-pat",
        "It makes that environment, then Claude Code asks you to sign in to your subscription there. temper never reads its login.",
    ] {
        assert!(words(&root).contains(expected), "missing {expected}");
    }
    let view = fixtures::page_view(&root);
    assert!(fixtures::find(view.tree(), Role::TextField, b"Name").is_some());
    let act = fixtures::find(view.tree(), Role::Button, "I’ve signed in".as_bytes()).unwrap();
    assert!(
        view.tree()
            .nodes()
            .iter()
            .find(|n| n.id == act)
            .unwrap()
            .styles
            .as_slice()
            .contains(&crate::look::CONNECT_PRIMARY)
    );
    assert_eq!(
        view.tree().nodes().iter().filter(|n| n.styles.as_slice().contains(&crate::look::CONNECT_PRIMARY)).count(),
        1
    );
    model_action(&mut root, M::ProgramName { value: Box::from(b"claude-alice".as_slice()) });
    let attempt = sent(&mut root, M::Submit { name: [1; 16] });
    answer(
        &mut root,
        attempt,
        c::DocumentOutcome::AccountProgramDeclared { source: 43, name: Box::from(b"claude-alice".as_slice()) },
    );
    assert!(!words(&root).contains("Declared your Claude subscription"));
    assert!(words(&root).contains("temper program-login --config CONFIG --data DATA --cache CACHE claude-alice"));
    assert!(
        root.pages().connect_model().unwrap().request.pending.as_ref().unwrap().outcome.is_none(),
        "declaration chains its host check"
    );
}
#[test]
fn a_sign_in_address_without_a_request_offers_its_button() {
    let (root, _) = flow_fixture(d::WayKind::SignIn);
    let view = fixtures::page_view(&root);
    assert!(fixtures::find(view.tree(), Role::Button, b"Continue with ChatGPT").is_some());
    assert!(fixtures::find(view.tree(), Role::Link, b"Other ways").is_some());
    assert!(words(&root).contains("Your ChatGPT plan"));
}
#[test]
fn a_completed_sign_in_lands_with_a_quiet_chat_and_usage_in_its_row() {
    let root = ending(c::AccountEnd::Made { account: 1, declined: false }, b"alice@example.com");
    assert_eq!(root.pages().address(), &d::Address::Accounts);
    let mut view = fixtures::page_view(&root);
    let line = super::inbox::styled(view.tree(), crate::look::CONNECT_CONFIRM);
    let nodes = super::inbox::subtree(view.tree(), line);
    assert!(nodes.iter().any(|n| {
        n.text.iter().any(|r| r.text.as_ref() == "Added your ChatGPT plan. temper’s agents now run on it.".as_bytes())
    }));
    let chat = nodes.iter().find(|n| n.role == Role::Link && n.name.as_deref() == Some(b"Start a chat")).unwrap();
    assert!(chat.styles.as_slice().contains(&crate::look::CONNECT_QUIET));
    let chat_id = chat.id;
    assert_eq!(
        crate::act(&mut view, chat_id, Act::Press),
        Some(d::Action::Pages(d::pages::Act::Go { address: d::Address::NewChat { draft: false } }))
    );
    let row = super::inbox::subtree(view.tree(), super::inbox::styled(view.tree(), crate::look::SETTINGS_ROW));
    assert!(row.iter().any(|n| n.role == Role::Link && n.name.as_deref() == Some(b"Manage usage")));
}
#[test]
fn a_completed_sign_in_without_a_display_names_the_plan() {
    assert!(
        words(&ending(c::AccountEnd::Made { account: 1, declined: false }, b"")).contains("Added your ChatGPT plan.")
    );
}
#[test]
fn renewed_sign_ins_land_with_the_same_added_line() {
    for display in [b"alice@example.com".as_slice(), b""] {
        let root = ending(c::AccountEnd::Renewed { account: 1, declined: false }, display);
        assert_eq!(root.pages().address(), &d::Address::Accounts);
        assert!(words(&root).contains("Added your ChatGPT plan."));
    }
}

#[test]
fn declined_plan_usage_offers_sign_in_again_and_a_key_link() {
    for end in
        [c::AccountEnd::Made { account: 1, declined: true }, c::AccountEnd::Renewed { account: 1, declined: true }]
    {
        let root = ending(end, b"alice@example.com");
        assert!(words(&root).contains("You signed in without plan usage."));
        let mut view = fixtures::page_view(&root);
        let key = fixtures::find(view.tree(), Role::Link, b"Use an API key").unwrap();
        let Some(d::Action::Pages(d::pages::Act::Go {
            address: d::Address::ConnectModel { way: Some(d::ModelWay { kind: d::WayKind::Key, .. }) },
        })) = crate::act(&mut view, key, Act::Press)
        else {
            panic!("key link")
        };
        assert!(fixtures::find(view.tree(), Role::Button, b"Continue with ChatGPT").is_some());
    }
}
#[test]
fn other_sign_in_endings_keep_their_words_and_offer_another_sign_in() {
    use c::{AccountEnd as E, AccountFailure as F, AccountRefusal as R};
    for (end, expected) in [
        (E::Refused { refusal: R::Held { provider: PROVIDER } }, "Another person here holds that ChatGPT account."),
        (
            E::Failed { failure: F::Misconfigured },
            "OpenAI refused temper’s sign-in. Check the configuration’s registration.",
        ),
        (E::Failed { failure: F::SignInAgain }, "Needs sign-in"),
        (E::Failed { failure: F::MissingScope { scope: 19 } }, "Missing required scope 19"),
        (E::Refused { refusal: R::Limit { bound: c::AccountBound::Sources } }, "Source limit reached."),
    ] {
        let root = ending(end, b"");
        assert!(words(&root).contains(expected), "missing {expected}");
        assert!(fixtures::find(fixtures::page_view(&root).tree(), Role::Button, b"Continue with ChatGPT").is_some());
    }
}
#[test]
fn the_accounts_banner_links_each_way_to_its_address_and_is_hidden_in_the_flow() {
    let (mut root, watch) = flow_fixture(d::WayKind::Key);
    let mut doc = super::accounts::document(0);
    doc.usable = false;
    super::accounts::deliver(&mut root, watch, doc);
    fixtures::root_event(
        &mut root,
        d::Event::Went { place: d::Place::Address(d::Address::Chats { filter: d::ChatsFilter::All }) },
    );
    let mut view = fixtures::page_view(&root);
    for (name, kind) in [
        (b"Continue with ChatGPT".as_slice(), d::WayKind::SignIn),
        (b"Use your ChatGPT subscription", d::WayKind::Program),
        (b"Use an API key", d::WayKind::Key),
    ] {
        let id = fixtures::find(view.tree(), Role::Link, name).unwrap();
        assert_eq!(
            crate::act(&mut view, id, Act::Press),
            Some(d::Action::Pages(d::pages::Act::Go {
                address: d::Address::ConnectModel { way: Some(d::ModelWay { source: None, provider: PROVIDER, kind }) }
            }))
        );
    }
    fixtures::root_event(
        &mut root,
        d::Event::Went { place: d::Place::Address(d::Address::ConnectModel { way: None }) },
    );
    assert!(!words(&root).contains("Agents need an LLM account to run."));
}

#[test]
fn adding_a_real_model_keeps_the_playground_running() {
    let provider = PROVIDER + 1;
    let mut me = d::pages::accounts::current_me(offered_root().client()).unwrap().clone();
    me.providers = List::with_capacity(1);
    me.providers.push(offer(provider, b"openai", false, false)).unwrap();
    let (mut root, accounts_watch) = fixtures::playground_frame_with_watch(
        d::Address::ConnectModel { way: Some(d::ModelWay { source: None, provider, kind: d::WayKind::Key }) },
        fixtures::PROJECT,
        true,
        None,
        Some(me),
    );
    fixtures::root_event(
        &mut root,
        d::Event::AccountKey { value: skein_lib::Secret::new(b"private-once".as_slice().into()) },
    );
    let attempt = sent(&mut root, M::Submit { name: [1; 16] });
    answer(&mut root, attempt, c::DocumentOutcome::AccountKeyKept { source: 43 });
    let mut doc = super::accounts::card_document(b"openai", b"", c::AccountKind::Key);
    doc.accounts.get_mut(0).unwrap().provider = Some(provider);
    doc.cut = false;
    super::accounts::deliver(&mut root, accounts_watch, doc);
    assert_eq!(d::pages::welcome::model_usable(&root), Some(true));
    assert!(root.forge().playground_running());
    // Outside the chats pane the frame's "Set up temper" says it runs (client/view.md, 1).
    assert!(words(&root).contains("Set up temper"));
    assert!(!words(&root).contains("This is the playground"));
    assert!(!words(&root).contains("stepped aside"));
}

#[test]
fn a_returned_source_waits_for_the_watch_with_its_signing_card_alone() {
    let (mut root, _) = flow_fixture(d::WayKind::SignIn);
    assert!(d::pages::accounts::document(root.client()).is_none());
    fixtures::root_event(
        &mut root,
        d::Event::Went {
            place: d::Place::Address(d::Address::ConnectModel {
                way: Some(d::ModelWay { provider: PROVIDER, kind: d::WayKind::SignIn, source: Some(43) }),
            }),
        },
    );
    assert!(words(&root).contains("Your ChatGPT plan"));
    assert!(fixtures::find(fixtures::page_view(&root).tree(), Role::Button, b"Continue with ChatGPT").is_none());
    assert!(root.pages().connect_model().unwrap().request.pending.is_none());
}
#[test]
fn a_returned_source_draws_the_same_ending_words_as_an_own_sign_in() {
    let (mut root, watch) = flow_fixture(d::WayKind::SignIn);
    fixtures::root_event(
        &mut root,
        d::Event::Went {
            place: d::Place::Address(d::Address::ConnectModel {
                way: Some(d::ModelWay { provider: PROVIDER, kind: d::WayKind::SignIn, source: Some(43) }),
            }),
        },
    );
    let mut doc = super::accounts::card_document(b"openai", b"alice@example.com", c::AccountKind::SignIn);
    doc.signing.clear();
    doc.last_end = Some(c::AccountLastEnd { source: 43, end: c::AccountEnd::Made { account: 1, declined: true } });
    super::accounts::deliver(&mut root, watch, doc);
    assert!(words(&root).contains("You signed in without plan usage."));
    assert!(fixtures::find(fixtures::page_view(&root).tree(), Role::Button, b"Continue with ChatGPT").is_some());
}

#[test]
fn a_returned_source_waits_without_inventing_a_link() {
    for welcome in [false, true] {
        let (mut root, watch) = flow_fixture(d::WayKind::SignIn);
        let way = Some(d::ModelWay { provider: PROVIDER, kind: d::WayKind::SignIn, source: Some(43) });
        let address = if welcome {
            d::Address::Welcome { step: d::Step::Model { way } }
        } else {
            d::Address::ConnectModel { way }
        };
        fixtures::root_event(&mut root, d::Event::Went { place: d::Place::Address(address) });
        let mut doc = super::accounts::card_document(b"openai", b"", c::AccountKind::SignIn);
        doc.usable = false;
        doc.accounts.clear();
        doc.signing.get_mut(0).unwrap().source = 43;
        super::accounts::deliver(&mut root, watch, doc);
        assert!(root.pages().connect_model().unwrap().request.sign_in_address(root.client()).is_none());
        for view in [fixtures::page_view(&root), fixtures::browser_page_view(&root)] {
            let tree = view.tree();
            let text = String::from_utf8_lossy(&fixtures::text(tree)).into_owned();
            assert!(!text.contains("Go forward to that page"));
            assert!(fixtures::find(tree, Role::TextField, PASTE_FIELD).is_some());
            assert!(fixtures::find(tree, Role::Button, b"Finish connecting").is_some());
            assert!(!tree.nodes().iter().any(|n| n.styles.as_slice().contains(&crate::look::WELCOME_AGAIN)));
            assert!(!tree.nodes().iter().any(|n| n.external.as_deref() == Some(b"https://issuer.example/visit")));
            assert!(fixtures::find(tree, Role::Link, b"Open it again").is_none());
            assert!(fixtures::find(tree, Role::Link, "Open ChatGPT’s sign-in ›".as_bytes()).is_none());
            assert!(
                fixtures::find(tree, Role::Disclosure, "Signed in, but the page couldn’t be reached?".as_bytes())
                    .is_some()
            );
        }
    }
}

const PASTE_FIELD: &[u8] = "Address of the page that couldn’t be reached".as_bytes();
pub(super) fn pasting() -> (d::Domain, skein_lib::Token) {
    let (mut root, watch) = flow_fixture(d::WayKind::SignIn);
    let way = d::pages::accounts::Way::SignIn { provider: PROVIDER, plan: true };
    let action = M::SignIn { name: [1; 16], way };
    let attempt = sent(&mut root, action);
    answer(
        &mut root,
        attempt,
        c::DocumentOutcome::AccountSignInStarted {
            source: 43,
            address: Box::from(b"https://issuer.example/visit".as_slice()),
        },
    );
    (root, watch)
}
fn paste_input(root: &mut d::Domain, value: &[u8]) {
    fixtures::root_event(root, d::Event::SignInAddress { value: skein_lib::Secret::new(Box::from(value)) });
}
#[test]
fn signing_in_orders_the_note_link_and_folded_paste_in_both_shells() {
    let (root, _) = pasting();
    assert_waiting(&root);
}
/// The two pages share one waiting shape, including the terminal's honest title.
pub(super) fn assert_waiting(root: &d::Domain) {
    use crate::look;
    for (browser, view) in [(true, fixtures::browser_page_view(root)), (false, fixtures::page_view(root))] {
        let tree = view.tree();
        let nodes = tree.nodes();
        let at = |style| {
            nodes
                .iter()
                .position(|n| n.styles.as_slice().contains(&style))
                .expect("waiting page carries each part’s style")
        };
        let standalone = root.pages().welcome().is_none();
        let note = at(if standalone { look::CONNECT_PANEL } else { look::WELCOME_AWAY });
        let disclosure = at(look::WELCOME_PASTE);
        assert!(note < disclosure);
        let title = if browser { "Waiting for ChatGPT’s sign-in, in the new tab" } else { "ChatGPT’s sign-in" };
        let status =
            nodes.iter().position(|n| n.role == Role::Status && n.name.as_deref() == Some(title.as_bytes())).unwrap();
        let muted = nodes
            .iter()
            .position(|n| {
                (n.role == Role::Paragraph || standalone && n.role == Role::Text)
                    && (n.name.as_deref() == Some(b"Sign in and approve temper there. This page moves on by itself.")
                        || standalone
                            && n.text.iter().any(|run| {
                                run.text.as_ref() == b"Sign in and approve temper there. This page moves on by itself."
                            }))
            })
            .unwrap();
        assert!(note < status && status < muted && muted < disclosure);
        if !standalone {
            assert!(nodes[muted].styles.as_slice().contains(&look::FLOW_MUTED));
        }
        assert_eq!(nodes[disclosure].role, Role::Disclosure);
        assert_eq!(nodes[disclosure].name.as_deref(), Some("Signed in, but the page couldn’t be reached?".as_bytes()));
        assert!(nodes[disclosure].styles.as_slice().contains(&look::MODEL_SIGN_IN));
        assert!(!nodes[disclosure].states.as_slice().contains(&skein_ui::State::Expanded));
        let field =
            nodes.iter().position(|n| n.role == Role::TextField && n.name.as_deref() == Some(PASTE_FIELD)).unwrap();
        let finish = nodes
            .iter()
            .position(|n| n.role == Role::Button && n.name.as_deref() == Some(b"Finish connecting"))
            .unwrap();
        assert!(disclosure < field && field < finish);
        assert!(finish < disclosure.checked_add(usize::try_from(nodes[disclosure].size).unwrap()).unwrap());
        assert!(nodes[finish].styles.as_slice().contains(&look::FLOW_PRIMARY));
        assert_eq!(nodes[field].entry, Some(skein_ui::EntryKind::Secret));
        let text = String::from_utf8_lossy(&fixtures::text(tree)).into_owned();
        assert!(text.contains(
            "When ChatGPT is done, its tab shows a page that can’t be reached. Copy that page’s whole address"
        ));
        assert!(!text.contains("Go forward to that page"));
        assert!(!nodes.iter().any(|n| n.styles.as_slice().contains(&look::FLOW_CHOICE)));
        assert_eq!(nodes.iter().any(|n| n.styles.as_slice().contains(&look::WELCOME_SPIN)), browser && !standalone);
        if browser {
            let link = at(if standalone { look::CONNECT_SECONDARY } else { look::WELCOME_AGAIN });
            assert!(muted < link && link < disclosure);
            assert_eq!(nodes[link].role, Role::Link);
            assert_eq!(nodes[link].name.as_deref(), Some(b"Open it again".as_slice()));
            assert_eq!(fixtures::node_text(tree, link).as_ref(), "Open it again ›".as_bytes());
            assert_eq!(
                nodes[link].external.as_deref(),
                root.pages().connect_model().unwrap().request.sign_in_address(root.client())
            );
        } else {
            let link = nodes[note..disclosure]
                .iter()
                .find(|n| n.role == Role::Link && n.external.is_some())
                .expect("the terminal offers the sign-in address");
            assert_eq!(link.external.as_deref(), Some(b"https://issuer.example/visit".as_slice()));
            assert!(!text.contains("in the new tab"));
            let mut print = skein_ui_text::Print::new(&view.limits().tree).unwrap();
            skein_ui_text::render(tree, b"temper", None, &mut print);
            let text = String::from_utf8_lossy(print.text.as_slice());
            assert!(text.contains(title));
            assert!(text.contains("Open the sign-in page <https://issuer.example/visit>"));
            assert!(text.contains("Signed in, but the page couldn’t be reached?"));
        }
    }
}
fn paste_case(case: usize) -> d::Domain {
    use c::SignInRedirectWhy as R;
    let (mut root, watch) = pasting();
    if case == 1 {
        paste_input(&mut root, b"fixture");
    }
    if case == 2 {
        paste_input(&mut root, &[b'x'; 257]);
    }
    if (3..=9).contains(&case) {
        paste_input(&mut root, b"fixture");
        let attempt = sent(&mut root, M::Finish { name: [2; 16] });
        if case == 4 {
            answer(&mut root, attempt, c::DocumentOutcome::AccountFinishing { source: 43 });
        }
        if (5..=8).contains(&case) {
            let why = [R::Address, R::State, R::TooMany, R::NotWaiting]
                [case.checked_sub(5).expect("refusal case starts at five")];
            fixtures::root_event(
                &mut root,
                d::Event::Client {
                    event: jig_client::Event::Up {
                        up: jig_client::Up::Answer {
                            attempt,
                            envelope: skein_lib::RequestEnvelope::Final,
                            answer: Some(jig_client::Answered::Jig(c::Answer::Refused {
                                refusal: c::Refusal::SignInRedirect { why },
                                at: None,
                            })),
                            why: None,
                        },
                    },
                },
            );
        }
        if case == 9 {
            fixtures::root_event(
                &mut root,
                d::Event::Client {
                    event: jig_client::Event::Up {
                        up: jig_client::Up::Answer {
                            attempt,
                            envelope: skein_lib::RequestEnvelope::Lost,
                            answer: None,
                            why: None,
                        },
                    },
                },
            );
        }
    }
    if case == 10 {
        let mut doc = super::accounts::card_document(b"openai", b"fixture", c::AccountKind::SignIn);
        doc.signing.clear();
        doc.signing.push(c::AccountSignIn { source: 43, provider: 17, holder: c::AccountHolder::Party(73) }).unwrap();
        super::accounts::deliver(&mut root, watch, doc.clone());
        doc.signing.clear();
        doc.cut = false;
        super::accounts::deliver(&mut root, watch, doc);
    }
    root
}
#[test]
fn the_paste_reads_each_state_in_its_words() {
    // Each row drives the page through its public events, then reads the tree.
    for (case, message, field, enabled, button_name) in [
        (0, "When ChatGPT is done, its tab shows a page that can’t be reached.", true, false, "Finish connecting"),
        (1, "Copy that page’s whole address from the address bar", true, true, "Finish connecting"),
        (2, "That's longer than a sign-in's address. Copy the address bar alone.", true, false, "Finish connecting"),
        (3, "Finishing…", true, false, "Finishing…"),
        (4, "Finishing your sign-in…", false, false, ""),
        (
            5,
            "That isn't this sign-in's address. Copy the whole address of the page that couldn't be reached, from the address bar.",
            true,
            false,
            "Finish connecting",
        ),
        (
            6,
            "That address is from an earlier sign-in. Paste the one from this sign-in's tab, or start again.",
            true,
            false,
            "Finish connecting",
        ),
        (
            7,
            "That's too many addresses from other sign-ins. Continue with ChatGPT to start again.",
            false,
            true,
            "Continue with ChatGPT",
        ),
        (8, "This sign-in has ended.", false, true, "Continue with ChatGPT"),
        (9, "That address wasn't sent. Paste it again.", true, false, "Finish connecting"),
        (10, "This sign-in has ended.", false, true, "Continue with ChatGPT"),
    ] {
        let root = paste_case(case);
        let view = fixtures::page_view(&root);
        let page_words = words(&root);
        // Button names are accessible names and may not contribute text runs.
        assert!(
            page_words.contains(message)
                || view.tree().nodes().iter().any(|n| n.name.as_deref() == Some(message.as_bytes())),
            "state {case}"
        );
        let control =
            view.tree().nodes().iter().find(|n| n.role == Role::TextField && n.name.as_deref() == Some(PASTE_FIELD));
        assert_eq!(control.is_some(), field, "state {case}");
        if let Some(control) = control {
            assert_eq!(control.entry, Some(skein_ui::EntryKind::Secret));
            assert!(control.value.as_ref().unwrap().text.is_empty());
            assert_eq!(control.states.as_slice().contains(&skein_ui::State::Disabled), case == 3);
        }
        if button_name.is_empty() {
            assert!(fixtures::find(view.tree(), Role::Button, b"Finish connecting").is_none());
        } else {
            let id = fixtures::find(view.tree(), Role::Button, button_name.as_bytes()).unwrap();
            let control = view.tree().nodes().iter().find(|n| n.id == id).unwrap();
            assert_eq!(!control.states.as_slice().contains(&skein_ui::State::Disabled), enabled, "state {case}");
        }
    }
}
#[test]
fn the_paste_field_decodes_to_the_secret_event_and_finish_to_its_act() {
    let (mut root, _) = pasting();
    let mut view = fixtures::view();
    let node = fixtures::find(fixtures::build(&mut view, &root), Role::TextField, PASTE_FIELD).unwrap();
    let decoded = crate::event(&view, node, Act::Input { value: Box::from(b"fixture-private".as_slice()) }).unwrap();
    let d::Event::SignInAddress { value } = decoded else { panic!("secret event") };
    assert!(value.expose() == b"fixture-private");
    paste_input(&mut root, b"fixture-private");
    let node = fixtures::find(fixtures::build(&mut view, &root), Role::Button, b"Finish connecting").unwrap();
    assert_eq!(
        crate::act(&mut view, node, Act::Press),
        Some(d::Action::Pages(d::pages::Act::ConnectModel(M::Finish { name: root.pages().account_name() })))
    );
    assert!(!format!("{view:?}").contains("fixture-private"));
}
#[test]
fn choose_offers_no_entry_for_another_computer() {
    let root = offered_root();
    for view in [fixtures::browser_page_view(&root), fixtures::page_view(&root)] {
        assert!(fixtures::find(view.tree(), Role::Button, b"Continue with ChatGPT").is_some());
        assert!(
            fixtures::find(view.tree(), Role::Button, "This browser isn't on temper's computer ›".as_bytes()).is_none()
        );
    }
}
#[test]
fn the_connected_model_offers_continue_as_a_terminal_command() {
    for (email, display, status) in [
        (None, b"Legacy".as_slice(), "ChatGPT plan · Legacy"),
        (Some(b"alice@example.com".as_slice()), b"", "ChatGPT plan · alice@example.com"),
        (Some(b"alice@example.com".as_slice()), b"Different display", "ChatGPT plan · alice@example.com"),
    ] {
        let (mut root, watch) = super::welcome::model_with_forge(d::WayKind::SignIn);
        let mut doc = super::accounts::card_document(b"openai", display, c::AccountKind::SignIn);
        doc.accounts.get_mut(0).unwrap().facts = super::accounts::email_facts(email);
        super::accounts::deliver(&mut root, watch, doc);
        fixtures::root_event(
            &mut root,
            d::Event::Went {
                place: d::Place::Address(d::Address::Welcome {
                    step: d::Step::Model {
                        way: Some(d::ModelWay { provider: PROVIDER, kind: d::WayKind::SignIn, source: None }),
                    },
                }),
            },
        );
        for view in [fixtures::page_view(&root), fixtures::browser_page_view(&root)] {
            assert!(fixtures::find(view.tree(), Role::Status, status.as_bytes()).is_some());
        }
        let mut view = fixtures::page_view(&root);
        let control = view
            .tree()
            .nodes()
            .iter()
            .rev()
            .find(|node| node.role == Role::Link && node.name.as_deref() == Some(b"Continue"))
            .unwrap()
            .id;
        let (node, first) = fixtures::terminal_command(
            view.tree(),
            &view.limits().tree,
            control,
            b"Continue",
            format!("✓ {status}").as_bytes(),
        );
        assert_eq!(
            crate::act(&mut view, node, first),
            Some(d::Action::Pages(d::pages::Act::Go { address: d::Address::Welcome { step: d::Step::Ready } }))
        );
    }
}

pub(super) const ENVIRONMENT: &[u8] = b"TEMPER_CONFIG='/config space' TEMPER_DATA='/data' TEMPER_CACHE='/cache' temper";
pub(super) fn answer_environment(root: &mut d::Domain, words: &[u8]) {
    use temper_client_protocol_wire::{self as wire, v1 as w};
    let address = root.pages().address().clone();
    fixtures::root_event(root, d::Event::Went { place: d::Place::Address(d::Address::Accounts) });
    let limits = fixtures::limits();
    let env = skein_lib::Env { now: skein_lib::Time::ZERO, wall: fixtures::NOW, limits };
    let mut out = skein_lib::Queue::uncapped();
    d::step(root, &env, d::Event::Went { place: d::Place::Address(address) }, &mut out);
    let read = out
        .iter()
        .find_map(|request| {
            if let d::Request::Unlinked {
                request:
                    jig_client::UnlinkedRequest::Down {
                        down: jig_client::Down::Read { read, what: jig_client::Readable::Family { record, .. } },
                    },
            } = request
                && wire::decode_reading(&record.bytes, &w::CEILINGS)
                    .is_some_and(|reading| *reading.read() == w::Read::ProgramEnvironment)
            {
                Some(*read)
            } else {
                None
            }
        })
        .expect("program page reads its environment");
    let answer =
        w::ProgramEnvironmentAnswer::new(&w::CEILINGS, w::ProgramEnvironmentAnswerParts { environment: words.into() })
            .unwrap();
    fixtures::root_event(
        root,
        d::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::ReadDone {
                    read,
                    result: jig_client::Readout::Family {
                        family: wire::FORGE,
                        record: Some(wire::program_environment_answer(&answer)),
                    },
                },
            },
        },
    );
}
pub(super) fn command_line(root: &d::Domain) -> Vec<u8> {
    let view = fixtures::page_view(root);
    let nodes = view.tree().nodes();
    let position = nodes.iter().position(|node| node.role == Role::Code).expect("command code");
    let code = nodes.get(position).unwrap();
    let runs: Vec<_> = nodes
        .iter()
        .skip(position)
        .take(usize::try_from(code.size).unwrap())
        .flat_map(|node| node.text.iter())
        .collect();
    assert_eq!(runs.len(), 1, "the command is one plain run");
    assert_eq!(runs[0].run, skein_ui::Run::Plain);
    runs[0].text.to_vec()
}

#[test]
fn the_command_shows_directory_placeholders_without_words() {
    let (root, _) = flow_fixture(d::WayKind::Program);
    assert_eq!(command_line(&root), b"temper program-login --config CONFIG --data DATA --cache CACHE claude-pat");
}
#[test]
fn the_command_takes_the_words_first() {
    let (mut root, _) = flow_fixture(d::WayKind::Program);
    answer_environment(&mut root, ENVIRONMENT);
    assert_eq!(
        command_line(&root),
        b"TEMPER_CONFIG='/config space' TEMPER_DATA='/data' TEMPER_CACHE='/cache' temper program-login claude-pat"
    );
}

#[test]
fn both_waiting_pages_reject_an_unsafe_address() {
    for welcome in [false, true] {
        let (mut root, _) = flow_fixture(d::WayKind::SignIn);
        if welcome {
            fixtures::root_event(
                &mut root,
                d::Event::Went {
                    place: d::Place::Address(d::Address::Welcome {
                        step: d::Step::Model {
                            way: Some(d::ModelWay { provider: PROVIDER, kind: d::WayKind::SignIn, source: None }),
                        },
                    }),
                },
            );
        }
        let attempt = sent(
            &mut root,
            M::SignIn { name: [1; 16], way: d::pages::accounts::Way::SignIn { provider: PROVIDER, plan: true } },
        );
        answer(
            &mut root,
            attempt,
            c::DocumentOutcome::AccountSignInStarted { source: 43, address: b"javascript:sign-in".as_slice().into() },
        );
        for view in [fixtures::browser_page_view(&root), fixtures::page_view(&root)] {
            assert!(fixtures::find(view.tree(), Role::Alert, b"Sign-in address is not an http(s) address.").is_some());
            assert!(!view.tree().nodes().iter().any(|n| n.external.as_deref() == Some(b"javascript:sign-in")));
            assert!(fixtures::find(view.tree(), Role::Link, b"Open it again").is_none());
        }
    }
}
#[test]
fn ended_sign_ins_remove_the_note_on_both_pages() {
    for welcome in [false, true] {
        for end in [
            c::AccountEnd::Made { account: 1, declined: false },
            c::AccountEnd::Refused { refusal: c::AccountRefusal::Held { provider: PROVIDER } },
        ] {
            let mut root = ending(end, b"alice@example.com");
            if welcome {
                fixtures::root_event(
                    &mut root,
                    d::Event::Went {
                        place: d::Place::Address(d::Address::Welcome {
                            step: d::Step::Model {
                                way: Some(d::ModelWay { provider: PROVIDER, kind: d::WayKind::SignIn, source: None }),
                            },
                        }),
                    },
                );
            }
            for view in [fixtures::browser_page_view(&root), fixtures::page_view(&root)] {
                assert!(!view.tree().nodes().iter().any(|n| n.styles.as_slice().contains(&crate::look::WELCOME_AWAY)));
                let text = String::from_utf8_lossy(&fixtures::text(view.tree())).into_owned();
                let message = if match end {
                    c::AccountEnd::Made { .. } => true,
                    c::AccountEnd::Renewed { .. } | c::AccountEnd::Refused { .. } | c::AccountEnd::Failed { .. } => {
                        false
                    }
                } {
                    if welcome { "ChatGPT plan" } else { "Signed in to ChatGPT" }
                } else {
                    "Another person here holds that ChatGPT account."
                };
                assert!(text.contains(message), "{text}");
                assert!(!text.contains("Go forward to that page"));
            }
        }
    }
}

#[test]
fn a_reloaded_source_reads_only_its_kept_address_and_refusal() {
    for welcome in [false, true] {
        for (source, refused) in [(43, false), (43, true), (44, true)] {
            let way = Some(d::ModelWay { provider: PROVIDER, kind: d::WayKind::SignIn, source: Some(43) });
            let address = if welcome {
                d::Address::Welcome { step: d::Step::Model { way } }
            } else {
                d::Address::ConnectModel { way }
            };
            let (mut root, watch) = flow_fixture(d::WayKind::SignIn);
            fixtures::root_event(&mut root, d::Event::Went { place: d::Place::Address(address) });
            let mut view = fixtures::browser_page_view(&root);
            fixtures::root_event(
                &mut root,
                d::Event::SignInKept {
                    source,
                    address: Box::from(b"https://issuer.example/visit".as_slice()),
                    refused,
                },
            );
            for delivered in [false, true] {
                if delivered {
                    let mut doc = super::accounts::card_document(b"openai", b"", c::AccountKind::SignIn);
                    doc.usable = false;
                    doc.accounts.clear();
                    doc.signing.get_mut(0).unwrap().source = 43;
                    super::accounts::deliver(&mut root, watch, doc);
                }
                let page = root.pages().connect_model().unwrap();
                assert_eq!(
                    page.request.sign_in_address(root.client()),
                    (source == 43).then_some(b"https://issuer.example/visit".as_slice())
                );
                assert_eq!(page.tab_refused(), source == 43 && refused);
                let tree = fixtures::build(&mut view, &root);
                assert!(fixtures::find(tree, Role::Button, b"Continue with ChatGPT").is_none());
                let name =
                    if source == 43 && refused { "Open ChatGPT’s sign-in ›".as_bytes() } else { b"Open it again" };
                let link = fixtures::find(tree, Role::Link, name);
                assert_eq!(link.is_some(), source == 43);
                if let Some(link) = link {
                    let node = tree.nodes().iter().find(|node| node.id == link).unwrap();
                    assert_eq!(node.external.as_deref(), Some(b"https://issuer.example/visit".as_slice()));
                    assert!(node.styles.as_slice().contains(&if refused {
                        if root.pages().welcome().is_none() {
                            crate::look::CONNECT_PRIMARY
                        } else {
                            crate::look::FLOW_PRIMARY
                        }
                    } else if root.pages().welcome().is_none() {
                        crate::look::CONNECT_SECONDARY
                    } else {
                        crate::look::WELCOME_AGAIN
                    }));
                    let title =
                        if refused { "ChatGPT’s sign-in" } else { "Waiting for ChatGPT’s sign-in, in the new tab" };
                    assert!(fixtures::find(tree, Role::Status, title.as_bytes()).is_some());
                }
            }
            let mut doc = super::accounts::card_document(b"openai", b"", c::AccountKind::SignIn);
            doc.usable = false;
            doc.accounts.clear();
            doc.signing.clear();
            super::accounts::deliver(&mut root, watch, doc);
            fixtures::build(&mut view, &root);
            assert!(fixtures::find(view.tree(), Role::Button, b"Continue with ChatGPT").is_some());
            assert!(
                !view
                    .tree()
                    .nodes()
                    .iter()
                    .any(|node| node.external.as_deref() == Some(b"https://issuer.example/visit"))
            );
        }
    }
}

#[test]
fn both_model_pages_use_the_binary_word_the_box_answers() {
    for welcome in [false, true] {
        let (mut root, _) = flow_fixture(d::WayKind::Program);
        if welcome {
            fixtures::root_event(
                &mut root,
                d::Event::Went {
                    place: d::Place::Address(d::Address::Welcome {
                        step: d::Step::Model {
                            way: Some(d::ModelWay { provider: PROVIDER, kind: d::WayKind::Program, source: None }),
                        },
                    }),
                },
            );
        }
        for (answer, expected) in [
            (b"~/bin/temper".as_slice(), b"~/bin/temper program-login claude-pat".as_slice()),
            (b"", b"temper program-login claude-pat"),
        ] {
            answer_environment(&mut root, answer);
            assert_eq!(command_line(&root), expected, "welcome: {welcome}");
        }
    }
}
fn opened(out: &skein_lib::Queue<d::Request>, subject: &c::Subject) -> skein_lib::Token {
    out.iter()
        .find_map(|request| match request {
            d::Request::Unlinked {
                request:
                    jig_client::UnlinkedRequest::Down {
                        down: jig_client::Down::Open { watch, subject: jig_client::Watchable::Jig(found) },
                    },
            } if found == subject => Some(*watch),
            d::Request::Screen { .. }
            | d::Request::KeepLook { .. }
            | d::Request::KeepSignIn { .. }
            | d::Request::Tab { .. }
            | d::Request::Client { .. }
            | d::Request::Unlinked { .. }
            | d::Request::Show { .. }
            | d::Request::Leave { .. }
            | d::Request::SignOut
            | d::Request::Dropped { .. }
            | d::Request::Copy { .. }
            | d::Request::Push(_)
            | d::Request::Fact { .. } => None,
        })
        .unwrap()
}

pub(super) fn program_root_with_line(address: d::Address, line: Option<&[u8]>) -> (d::Domain, skein_lib::Token) {
    use temper_client_protocol_wire::{self as wire, v1 as w};
    let limits = fixtures::limits();
    let (offered, _) = flow_fixture(d::WayKind::Program);
    let me = d::pages::accounts::current_me(offered.client()).unwrap().clone();
    let accounts = super::accounts::document(0);
    let mut root = d::unlinked(&limits, fixtures::SEED);
    let mut out = skein_lib::Queue::uncapped();
    let env = skein_lib::Env { now: skein_lib::Time::ZERO, wall: fixtures::NOW, limits };
    for event in [
        d::Event::Client { event: jig_client::Event::StartUnlinked { kept: List::with_capacity(0) } },
        d::Event::Start { address, kept: List::with_capacity(0), with: Some(jig_client::With::Transport) },
        d::Event::Client {
            event: jig_client::Event::Up { up: jig_client::Up::Served { served: true, versions: None } },
        },
    ] {
        d::step(&mut root, &env, event, &mut out);
    }
    let me_watch = opened(&out, &c::Subject::Me);
    d::step(
        &mut root,
        &env,
        d::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched {
                    watch: me_watch,
                    event: c::WatchEvent::Delivered(c::Delivery::Me(Box::new(me))),
                },
            },
        },
        &mut out,
    );
    let accounts_watch = opened(&out, &c::Subject::Accounts);
    fixtures::root_event(
        &mut root,
        d::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched {
                    watch: accounts_watch,
                    event: c::WatchEvent::Delivered(c::Delivery::Accounts(Box::new(accounts))),
                },
            },
        },
    );
    let read = out
        .iter()
        .find_map(|request| match request {
            d::Request::Unlinked {
                request:
                    jig_client::UnlinkedRequest::Down {
                        down: jig_client::Down::Read { read, what: jig_client::Readable::Family { record, .. } },
                    },
            } if wire::decode_reading(&record.bytes, &w::CEILINGS).unwrap().read() == &w::Read::Deployment => {
                Some(*read)
            }
            d::Request::Screen { .. }
            | d::Request::KeepLook { .. }
            | d::Request::KeepSignIn { .. }
            | d::Request::Tab { .. }
            | d::Request::Client { .. }
            | d::Request::Unlinked { .. }
            | d::Request::Show { .. }
            | d::Request::Leave { .. }
            | d::Request::SignOut
            | d::Request::Dropped { .. }
            | d::Request::Copy { .. }
            | d::Request::Push(_)
            | d::Request::Fact { .. } => None,
        })
        .expect("live link reads deployment");
    let answer = w::DeploymentAnswer::new(
        &w::CEILINGS,
        w::DeploymentAnswerParts { push: None, welcome: true, owned: true, program_login: line.map(Into::into) },
    )
    .unwrap();
    fixtures::root_event(
        &mut root,
        d::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::ReadDone {
                    read,
                    result: jig_client::Readout::Family {
                        family: wire::FORGE,
                        record: Some(wire::deployment_answer(&answer)),
                    },
                },
            },
        },
    );
    (root, accounts_watch)
}

#[test]
fn a_program_way_prints_the_deployments_whole_command_or_the_placeholders() {
    for line in [
        None,
        Some(
            b"'/box/bin/temper' program-login --config '/box/config' --data '/box/data' --cache '/box/cache'"
                .as_slice(),
        ),
    ] {
        let (root, _) = program_root_with_line(
            d::Address::ConnectModel {
                way: Some(d::ModelWay { source: None, provider: PROVIDER, kind: d::WayKind::Program }),
            },
            line,
        );
        for view in [fixtures::page_view(&root), fixtures::browser_page_view(&root)] {
            let shown = String::from_utf8(fixtures::text(view.tree()).into_vec()).unwrap();
            let expected = match line {
                Some(line) => format!("{} claude-pat", core::str::from_utf8(line).unwrap()),
                None => "temper program-login --config CONFIG --data DATA --cache CACHE claude-pat".into(),
            };
            assert!(shown.contains(&expected), "{shown}");
            let code = view.tree().nodes().iter().position(|node| node.role == Role::Code).unwrap();
            let command = match line {
                Some(line) => format!("{} claude-pat", core::str::from_utf8(line).unwrap()),
                None => "temper program-login --config CONFIG --data DATA --cache CACHE claude-pat".into(),
            };
            assert_eq!(fixtures::node_text(view.tree(), code).as_ref(), command.as_bytes());
        }
    }
}

#[test]
fn the_program_command_renders_its_full_wire_bound() {
    let line = vec![b'x'; 8192];
    let (root, _) = program_root_with_line(
        d::Address::ConnectModel {
            way: Some(d::ModelWay { source: None, provider: PROVIDER, kind: d::WayKind::Program }),
        },
        Some(&line),
    );
    for words in [fixtures::words(), fixtures::browser_words()] {
        let domain = fixtures::limits();
        let limits = crate::Limits::of(&domain, &words).unwrap();
        let mut view = crate::View::new(&limits, &domain, words, fixtures::FORGE);
        fixtures::build(&mut view, &root);
        let expected = format!("{} claude-pat", core::str::from_utf8(&line).unwrap());
        assert!(fixtures::text(view.tree()).windows(expected.len()).any(|bytes| bytes == expected.as_bytes()));
    }
}

#[test]
fn the_program_environment_replaces_the_standalone_command_on_both_model_pages() {
    let way = Some(d::ModelWay { source: None, provider: PROVIDER, kind: d::WayKind::Program });
    let line = b"'/box/bin/temper' program-login --config '/box/config' --data '/box/data' --cache '/box/cache'";
    for address in [d::Address::ConnectModel { way }, d::Address::Welcome { step: d::Step::Model { way } }] {
        let (mut root, watch) = program_root_with_line(address, Some(line));
        let mut document = super::accounts::document(0);
        document.usable = false;
        document.settled = true;
        super::accounts::deliver(&mut root, watch, document);
        for (environment, expected) in [
            (ENVIRONMENT, [ENVIRONMENT, b" program-login claude-pat"].concat()),
            (b"".as_slice(), b"temper program-login claude-pat".to_vec()),
        ] {
            answer_environment(&mut root, environment);
            assert_eq!(command_line(&root), expected);
        }
    }
}

#[test]
fn a_connected_plan_offers_another_plan_without_a_default() {
    let (mut root, watch) = flow_fixture(d::WayKind::SignIn);
    fixtures::root_event(
        &mut root,
        d::Event::Went { place: d::Place::Address(d::Address::ConnectModel { way: None }) },
    );
    let doc = super::accounts::card_document(b"openai", b"Pat", c::AccountKind::SignIn);
    super::accounts::deliver(&mut root, watch, doc);
    let view = fixtures::page_view(&root);
    assert!(fixtures::find(view.tree(), Role::Text, "✓ Connected".as_bytes()).is_some());
    assert!(fixtures::find(view.tree(), Role::Button, b"Add another plan").is_some());
    assert!(!view.tree().nodes().iter().any(|node| node.styles.as_slice().contains(&crate::look::CONNECT_DEFAULT)));
    assert!(!view.tree().nodes().iter().any(|node| node.styles.as_slice().contains(&crate::look::CONNECT_PRIMARY)));
}
#[test]
fn choose_has_no_steps_or_choose_region_and_one_holder_switch() {
    let root = plans_root();
    let view = fixtures::page_view(&root);
    assert!(fixtures::find(view.tree(), Role::Region, b"Choose").is_none());
    assert!(fixtures::find(view.tree(), Role::List, b"Steps").is_none());
    let switches: Vec<_> = view
        .tree()
        .nodes()
        .iter()
        .filter(|node| {
            node.role == Role::Link
                && (node.name.as_deref() == Some(b"Use your accounts")
                    || node.name.as_deref() == Some(b"Use deployment accounts"))
        })
        .collect();
    assert_eq!(switches.len(), 1);
    assert!(switches[0].styles.as_slice().contains(&crate::look::CONNECT_QUIET));
    assert!(fixtures::find(view.tree(), Role::Heading { level: 2 }, b"Or an API key").is_some());
}
