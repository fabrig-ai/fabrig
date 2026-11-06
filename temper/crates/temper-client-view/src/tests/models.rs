//! Settings' supplied model offer (temper/docs/design/client/README.md, 3.5, The `Models` tab).
#![expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    reason = "ordinary tests inspect semantic rows, order and navigation"
)]
use super::fixtures as f;
use crate::look;
use jig_client::contract as c;
use skein_lib::{List, Token};
use skein_ui::{Act, Node, Role, State, Tree};
use temper_client_domain::{self as d, Address};

fn text(value: &str) -> c::Text {
    c::Text::new(value.as_bytes().into(), 256).unwrap()
}
fn deliver(root: &mut d::Domain, watch: Token, value: c::Delivery) {
    f::root_event(
        root,
        d::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched { watch, event: c::WatchEvent::Delivered(value) },
            },
        },
    );
}
fn model(name: &str, provider: u32, shown: &str) -> c::MeModel {
    c::MeModel {
        name: text(name),
        provider: Some(provider),
        window: 200_000,
        output: 16_384,
        efforts: List::from_boxed(Box::new([text("low"), text("medium"), text("high")])),
        effort: Some(text("high")),
        shown: Some(text(shown)),
        accounts: List::from_boxed(Box::new([provider])),
        picker: true,
        description: None,
        prices: c::ModelPrices { input: 1, cached: 1, cache_write: None, output: 5, unit: 1000 },
    }
}
fn fixture(openai: bool) -> (d::Domain, Token, Token) {
    let (mut root, me_watch, _, _, accounts_watch) =
        f::root_at_full(Address::Models { project: f::PROJECT }, &f::domain_limits(), b"Shop");
    let mut me = root.pages().me(root.client()).unwrap().clone();
    me.providers = List::uncapped();
    for (provider, name) in [(17, "anthropic"), (18, "openai")] {
        me.providers
            .push(c::ProviderOffer {
                provider,
                name: c::DocumentText::complete(text(name)),
                key_bytes: 4096,
                sign_in: true,
                key: true,
                programs: List::with_capacity(0),
                programs_cut: false,
                none: false,
                signed_in_plan: true,
                manage_usage: None,
            })
            .unwrap();
    }
    let member = me.projects.iter_mut().find(|p| p.project == f::PROJECT).unwrap();
    member.models = List::uncapped();
    member.models.push(model("claude-opus-5-5", 17, "Opus 5.5")).unwrap();
    let mut sonnet = model("claude-sonnet-5-5", 17, "Sonnet 5.5");
    sonnet.picker = false;
    member.models.push(sonnet).unwrap();
    if openai {
        member.models.push(model("gpt-6.1-sol", 18, "GPT-6.1 Sol")).unwrap();
    }
    member.defaults = List::from_boxed(Box::new([
        c::MeDefault {
            template: 3,
            choice: c::LlmChoice {
                account: None,
                main: c::ModelEffort { model: text("claude-sonnet-5-5"), effort: Some(text("low")) },
                subagents: List::with_capacity(0),
            },
        },
        c::MeDefault {
            template: 1,
            choice: c::LlmChoice {
                account: None,
                main: c::ModelEffort { model: text("claude-opus-5-5"), effort: Some(text("low")) },
                subagents: List::with_capacity(0),
            },
        },
    ]));
    let mut doc = super::accounts::document(1);
    let mut claude = doc.accounts.get(0).unwrap().clone();
    claude.provider = Some(17);
    claude.provider_name = Some(c::DocumentText::complete(text("anthropic")));
    claude.holder = c::AccountHolder::Party(me.party.number);
    claude.plan = true;
    claude.sources = List::from_boxed(Box::new([c::AccountSource {
        number: 1,
        place: 1,
        name: None,
        kind: c::AccountKind::Program,
        state: c::AccountState::Granted,
        usable: true,
        expiring: false,
        attention: false,
        spent_until: None,
    }]));
    doc.accounts = List::uncapped();
    doc.accounts.push(claude.clone()).unwrap();
    if openai {
        claude.number = 2;
        claude.provider = Some(18);
        claude.provider_name = Some(c::DocumentText::complete(text("openai")));
        claude.sources.get_mut(0).unwrap().kind = c::AccountKind::SignIn;
        doc.accounts.push(claude).unwrap();
    }
    deliver(&mut root, me_watch, c::Delivery::Me(Box::new(me)));
    let accounts_watch = accounts_watch.unwrap();
    deliver(&mut root, accounts_watch, c::Delivery::Accounts(Box::new(doc)));
    (root, me_watch, accounts_watch)
}
fn subtree<'a>(tree: &'a Tree, node: &Node) -> &'a [Node] {
    let at = tree.nodes().iter().position(|n| n.id == node.id).unwrap();
    &tree.nodes()[at..at.checked_add(usize::try_from(node.size).unwrap()).unwrap()]
}
fn main(tree: &Tree) -> &[Node] {
    subtree(tree, tree.nodes().iter().find(|n| n.role == Role::Main).unwrap())
}
fn named<'a>(tree: &'a Tree, name: &str) -> &'a Node {
    main(tree).iter().find(|n| n.name.as_deref() == Some(name.as_bytes())).unwrap_or_else(|| panic!("missing {name}"))
}
fn change_me(root: &mut d::Domain, watch: Token, me: c::Me) {
    deliver(root, watch, c::Delivery::Me(Box::new(me)));
}

#[test]
fn tabs_put_models_between_accounts_and_builders() {
    let root = fixture(true).0;
    for mut view in [f::page_view(&root), f::browser_page_view(&root)] {
        let tree = view.tree();
        assert_eq!(skein_ui::outline::check(tree), None);
        let nav = tree.nodes().iter().find(|n| n.styles.as_slice().contains(&look::SETTINGS_TABS)).unwrap();
        let links: Vec<_> = subtree(tree, nav).iter().filter(|n| n.role == Role::Link).collect();
        assert_eq!(
            links.iter().map(|n| n.name.as_deref().unwrap()).collect::<Vec<_>>(),
            [b"How work runs".as_slice(), b"Repositories", b"Forges", b"Accounts", b"Models", b"Builders"]
        );
        let models = links[4];
        assert!(models.states.as_slice().contains(&State::Current));
        let id = models.id;
        assert_eq!(crate::href(&mut view, id).unwrap().as_ref(), b"/projects/1/settings/models");
        assert_eq!(
            crate::act(&mut view, id, Act::Press),
            Some(d::Action::Pages(d::pages::Act::Go { address: Address::Models { project: f::PROJECT } }))
        );
    }
}
#[test]
fn a_row_says_name_effort_and_the_default() {
    let root = fixture(true).0;
    for (terminal, mut view) in [(true, f::page_view(&root)), (false, f::browser_page_view(&root))] {
        let tree = view.tree();
        let rows: Vec<_> =
            main(tree).iter().filter(|n| n.styles.as_slice().contains(&look::SETTINGS_SETTING)).collect();
        let words: Vec<_> =
            rows.iter().map(|n| String::from_utf8(f::nodes_text(subtree(tree, n)).into_vec()).unwrap()).collect();
        let expected = if terminal {
            [
                "Opus 5.5 claude-opus-5-5 · high · the default On",
                "Sonnet 5.5 claude-sonnet-5-5 · high Off",
                "GPT-6.1 Sol gpt-6.1-sol · high On",
            ]
        } else {
            [
                "Opus 5.5claude-opus-5-5 · high · the defaultOn",
                "Sonnet 5.5claude-sonnet-5-5 · highOff",
                "GPT-6.1 Solgpt-6.1-sol · highOn",
            ]
        };
        assert_eq!(words, expected);
        assert!(rows.iter().flat_map(|n| subtree(tree, n)).all(|n| n.role != Role::Check && n.binding.is_none()));
        let id = named(tree, "Configure temper.").id;
        assert_eq!(crate::href(&mut view, id).unwrap().as_ref(), b"/projects/1/settings/configure");
    }
}
#[test]
fn a_model_with_no_efforts_says_its_name_alone() {
    let (mut root, watch, _) = fixture(true);
    let mut me = root.pages().me(root.client()).unwrap().clone();
    let model = me.projects.get_mut(0).unwrap().models.get_mut(1).unwrap();
    model.efforts.clear();
    model.effort = None;
    change_me(&mut root, watch, me);
    for view in [f::page_view(&root), f::browser_page_view(&root)] {
        let detail = named(view.tree(), "claude-sonnet-5-5");
        assert!(detail.styles.as_slice().contains(&look::SETTINGS_DETAIL));
        assert!(!String::from_utf8_lossy(&f::nodes_text(main(view.tree()))).contains("claude-sonnet-5-5 ·"));
    }
}
#[test]
fn no_model_of_a_provider_with_no_account_shows() {
    let (mut root, me_watch, accounts_watch) = fixture(false);
    for mut view in [f::page_view(&root), f::browser_page_view(&root)] {
        let tree = view.tree();
        named(tree, "OpenAI");
        let link = named(tree, "Connect a plan or key ›");
        let id = link.id;
        let words = String::from_utf8(f::nodes_text(main(tree)).into_vec()).unwrap();
        assert!(!words.contains("gpt-") && !words.contains("GPT-"));
        assert_eq!(main(tree).iter().filter(|n| n.styles.as_slice().contains(&look::SETTINGS_SETTING)).count(), 2);
        assert_eq!(crate::href(&mut view, id).unwrap().as_ref(), b"/accounts/connect");
    }
    let mut me = root.pages().me(root.client()).unwrap().clone();
    me.projects.get_mut(0).unwrap().models.clear();
    change_me(&mut root, me_watch, me);
    let mut doc = d::pages::accounts::document(root.client()).unwrap().clone();
    doc.accounts.clear();
    deliver(&mut root, accounts_watch, c::Delivery::Accounts(Box::new(doc)));
    for view in [f::page_view(&root), f::browser_page_view(&root)] {
        let tree = view.tree();
        named(tree, "Claude");
        named(tree, "OpenAI");
        assert_eq!(
            main(tree)
                .iter()
                .filter(|n| n.role == Role::Link && n.name.as_deref() == Some("Connect a plan or key ›".as_bytes()))
                .count(),
            2
        );
        assert!(!main(tree).iter().any(|n| n.styles.as_slice().contains(&look::SETTINGS_SETTING)));
    }
}
#[test]
fn an_unusable_account_keeps_its_fix_and_no_row() {
    let (mut root, me_watch, accounts_watch) = fixture(true);
    let mut me = root.pages().me(root.client()).unwrap().clone();
    me.projects.get_mut(0).unwrap().models.truncate(2);
    change_me(&mut root, me_watch, me);
    let mut doc = d::pages::accounts::document(root.client()).unwrap().clone();
    let source = doc.accounts.get_mut(1).unwrap().sources.get_mut(0).unwrap();
    source.usable = false;
    source.state = c::AccountState::SignInAgain;
    deliver(&mut root, accounts_watch, c::Delivery::Accounts(Box::new(doc)));
    for mut view in [f::page_view(&root), f::browser_page_view(&root)] {
        let tree = view.tree();
        let id = named(tree, "Sign in again ›").id;
        let words = String::from_utf8(f::nodes_text(main(tree)).into_vec()).unwrap();
        assert!(!words.contains("gpt-") && !words.contains("Connect a plan or key"));
        assert_eq!(crate::href(&mut view, id).unwrap().as_ref(), b"/accounts");
    }
}
#[test]
fn each_section_says_what_pays() {
    let (mut root, _, watch) = fixture(true);
    for key in [false, true] {
        if key {
            let mut doc = d::pages::accounts::document(root.client()).unwrap().clone();
            let mut account = doc.accounts.get(1).unwrap().clone();
            account.plan = false;
            account.sources.get_mut(0).unwrap().kind = c::AccountKind::Key;
            doc.accounts.push(account.clone()).unwrap();
            // Another person's and an unusable account add no payment clause.
            account.holder = c::AccountHolder::Deployment;
            doc.accounts.push(account.clone()).unwrap();
            account.holder = doc.accounts.get(0).unwrap().holder;
            account.sources.get_mut(0).unwrap().usable = false;
            doc.accounts.push(account).unwrap();
            deliver(&mut root, watch, c::Delivery::Accounts(Box::new(doc)));
        }
        for view in [f::page_view(&root), f::browser_page_view(&root)] {
            let tree = view.tree();
            named(tree, "Your Claude subscription. These come with temper.");
            named(
                tree,
                if key {
                    "Your ChatGPT plan. Your API key. These come with temper."
                } else {
                    "Your ChatGPT plan. These come with temper."
                },
            );
            let headings: Vec<_> = main(tree)
                .iter()
                .filter(|n| n.role == Role::Heading { level: 2 })
                .map(|n| n.name.as_deref().unwrap())
                .collect();
            assert_eq!(headings, [b"Claude".as_slice(), b"OpenAI"]);
        }
    }
    for (kind, plan) in [(c::AccountKind::Program, false), (c::AccountKind::SignIn, true)] {
        let mut doc = d::pages::accounts::document(root.client()).unwrap().clone();
        let claude = doc.accounts.get_mut(0).unwrap();
        claude.plan = plan;
        claude.sources.get_mut(0).unwrap().kind = kind;
        deliver(&mut root, watch, c::Delivery::Accounts(Box::new(doc)));
        for view in [f::page_view(&root), f::browser_page_view(&root)] {
            named(view.tree(), "Your Claude subscription. These come with temper.");
        }
    }
    f::root_event(
        &mut root,
        d::Event::Client {
            event: jig_client::Event::Up { up: jig_client::Up::WatchEnded { watch, end: c::WatchEnd::Closed } },
        },
    );
    for view in [f::page_view(&root), f::browser_page_view(&root)] {
        assert_eq!(
            main(view.tree()).iter().filter(|n| n.name.as_deref() == Some(b"These come with temper.")).count(),
            2
        );
    }
}
