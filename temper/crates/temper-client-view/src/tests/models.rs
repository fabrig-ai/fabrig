//! Settings' supplied model offer (temper/docs/design/client/pages/README.md, The `Models` tab).
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
        new_until: None,
        number: 1,
        source: jig_client::contract::ModelSource::Stock,
        estimated: false,
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
    me.deployment_role = Some(c::HeldRole::Owner);
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
    sonnet.number = 2;
    sonnet.picker = false;
    member.models.push(sonnet).unwrap();
    if openai {
        let mut sol = model("gpt-6.1-sol", 18, "GPT-6.1 Sol");
        sol.number = 4;
        member.models.push(sol).unwrap();
    }
    member.defaults = List::from_boxed(Box::new([
        c::MeDefault {
            template: 7,
            choice: c::LlmChoice {
                later: None,
                account: None,
                main: c::ModelEffort { model: text("claude-sonnet-5-5"), effort: Some(text("low")) },
                subagents: List::with_capacity(0),
            },
        },
        c::MeDefault {
            template: 1,
            choice: c::LlmChoice {
                later: None,
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
fn tabs_put_models_between_forges_and_workers() {
    let root = fixture(true).0;
    for mut view in [f::page_view(&root), f::browser_page_view(&root)] {
        let tree = view.tree();
        assert_eq!(skein_ui::outline::check(tree), None);
        let nav = tree.nodes().iter().find(|n| n.styles.as_slice().contains(&look::SETTINGS_TABS)).unwrap();
        let links: Vec<_> = subtree(tree, nav).iter().filter(|n| n.role == Role::Link).collect();
        assert_eq!(
            links.iter().map(|n| n.name.as_deref().unwrap()).collect::<Vec<_>>(),
            [b"Accounts".as_slice(), b"Forges", b"Models", b"Workers", b"Members", b"Limits"]
        );
        let models = links[2];
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
        let id = named(tree, "ask temper.").id;
        assert_eq!(crate::href(&mut view, id).unwrap().as_ref(), b"/projects/1/ask");
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
#[expect(
    clippy::too_many_lines,
    reason = "one table holds payment clauses, list precedence and clock changes in both shells"
)]
fn each_section_says_what_pays() {
    const HOUR: u64 = 3_600_000_000_000;
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
            assert_eq!(headings, [b"Claude".as_slice(), b"OpenAI", b"Add a model by name"]);
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
    for (read, tried, failed, expected) in [
        (None, None, false, "These come with temper."),
        (Some(HOUR), Some(HOUR), false, "Read from OpenAI 2 hours ago."),
        (None, Some(HOUR), true, "Couldn't read OpenAI's list · 2 hours ago."),
        (Some(2 * HOUR), Some(HOUR), true, "Couldn't read OpenAI's list · 2 hours ago."),
    ] {
        let (mut root, _, watch) = fixture(true);
        let mut doc = d::pages::accounts::document(root.client()).unwrap().clone();
        doc.accounts.get_mut(1).unwrap().list = Some(c::AccountList {
            read: read.map(skein_lib::Wall::from_nanos),
            tried: tried.map(skein_lib::Wall::from_nanos),
            failure: failed.then_some(c::AccountListFailure::Deadline),
            left_out: 0,
        });
        // Newer failed states on accounts outside the payment filter cannot win.
        let mut other = doc.accounts.get(1).unwrap().clone();
        other.list = Some(c::AccountList {
            read: None,
            tried: Some(skein_lib::Wall::from_nanos(2 * HOUR)),
            failure: Some(c::AccountListFailure::Unreadable),
            left_out: 0,
        });
        other.holder = c::AccountHolder::Deployment;
        doc.accounts.push(other.clone()).unwrap();
        other.holder = doc.accounts.get(0).unwrap().holder;
        other.sources.get_mut(0).unwrap().usable = false;
        doc.accounts.push(other).unwrap();
        deliver(&mut root, watch, c::Delivery::Accounts(Box::new(doc)));
        for terminal in [true, false] {
            let mut view = if terminal { f::page_view(&root) } else { f::browser_page_view(&root) };
            crate::build(&mut view, &root, skein_lib::Wall::from_nanos(3 * HOUR), &mut skein_lib::Queue::uncapped());
            named(view.tree(), &["Your ChatGPT plan. ", expected].concat());
            named(view.tree(), "Your Claude subscription. These come with temper.");
            assert_eq!(main(view.tree()).iter().filter(|n| n.name.as_deref() == Some(b"Read again")).count(), 1);
            if read.is_some() || failed {
                let wake = view.next_change().unwrap();
                assert!(wake > skein_lib::Wall::from_nanos(3 * HOUR));
                crate::build(
                    &mut view,
                    &root,
                    skein_lib::Wall::from_nanos(4 * HOUR),
                    &mut skein_lib::Queue::uncapped(),
                );
                let text = String::from_utf8_lossy(&f::nodes_text(main(view.tree()))).into_owned();
                assert!(text.contains("3 hours ago"), "{text}");
            }
        }
    }

    for failed in [false, true] {
        let (mut root, _, watch) = fixture(true);
        let mut doc = d::pages::accounts::document(root.client()).unwrap().clone();
        let openai = doc.accounts.get_mut(1).unwrap();
        openai.list = Some(c::AccountList {
            read: Some(skein_lib::Wall::from_nanos(HOUR)),
            tried: Some(skein_lib::Wall::from_nanos(HOUR)),
            failure: failed.then_some(c::AccountListFailure::Deadline),
            left_out: 0,
        });
        let mut second = openai.clone();
        second.list.as_mut().unwrap().read = Some(skein_lib::Wall::from_nanos(2 * HOUR));
        second.list.as_mut().unwrap().tried = Some(skein_lib::Wall::from_nanos(2 * HOUR));
        doc.accounts.push(second).unwrap();
        deliver(&mut root, watch, c::Delivery::Accounts(Box::new(doc)));
        for mut view in [f::page_view(&root), f::browser_page_view(&root)] {
            crate::build(&mut view, &root, skein_lib::Wall::from_nanos(4 * HOUR), &mut skein_lib::Queue::uncapped());
            named(
                view.tree(),
                if failed {
                    "Your ChatGPT plan. Your ChatGPT plan. Couldn't read OpenAI's list · 2 hours ago."
                } else {
                    "Your ChatGPT plan. Your ChatGPT plan. Read from OpenAI 2 hours ago."
                },
            );
        }
    }
}

#[test]
fn an_owner_has_the_switch_and_a_member_the_word() {
    let (mut root, watch, _) = fixture(true);
    for role in [Some(c::HeldRole::Owner), Some(c::HeldRole::Policy { role: 1 }), None] {
        let mut me = root.pages().me(root.client()).unwrap().clone();
        me.deployment_role = role;
        change_me(&mut root, watch, me);
        for mut view in [f::page_view(&root), f::browser_page_view(&root)] {
            let tree = view.tree();
            assert_eq!(skein_ui::outline::check(tree), None);
            let checks: Vec<_> = main(tree).iter().filter(|n| n.role == Role::Check).collect();
            if role == Some(c::HeldRole::Owner) {
                assert_eq!(checks.len(), 3);
                let switch = named(tree, "Opus 5.5 in the picker");
                assert!(switch.styles.as_slice().contains(&look::SETTINGS_SWITCH));
                assert!(switch.states.as_slice().contains(&State::Pressed));
                assert!(!named(tree, "Sonnet 5.5 in the picker").states.as_slice().contains(&State::Pressed));
                let id = switch.id;
                assert_eq!(
                    crate::act(&mut view, id, Act::Toggle),
                    Some(d::Action::Pages(d::pages::Act::Models(d::pages::models::Act::Picker {
                        number: 1,
                        on: false
                    })))
                );
            } else {
                assert_eq!(checks.len(), 0);
            }
            named(view.tree(), "On");
            named(view.tree(), "Off");
        }
    }
    let mut me = root.pages().me(root.client()).unwrap().clone();
    me.deployment_role = Some(c::HeldRole::Owner);
    change_me(&mut root, watch, me);
    f::root_event(
        &mut root,
        d::Event::Act {
            action: d::Action::Pages(d::pages::Act::Models(d::pages::models::Act::Picker { number: 1, on: false })),
        },
    );
    for view in [f::page_view(&root), f::browser_page_view(&root)] {
        let switch = named(view.tree(), "Opus 5.5 in the picker");
        assert!(switch.states.as_slice().contains(&State::Pressed));
        assert!(switch.states.as_slice().contains(&State::Disabled));
    }
}

#[test]
fn a_row_says_where_it_came_from() {
    const WEEK: u64 = 604_800_000_000_000;
    let (mut root, watch, _) = fixture(true);
    for (source, estimated, suffix) in [
        (c::ModelSource::Person, true, " · added by you · estimated prices"),
        (c::ModelSource::Person, false, " · added by you"),
        (c::ModelSource::List, true, " · estimated prices"),
        (c::ModelSource::Stock, false, ""),
    ] {
        let mut me = root.pages().me(root.client()).unwrap().clone();
        let model = me.projects.get_mut(0).unwrap().models.get_mut(0).unwrap();
        model.source = source;
        model.estimated = estimated;
        change_me(&mut root, watch, me);
        for view in [f::page_view(&root), f::browser_page_view(&root)] {
            named(view.tree(), &["claude-opus-5-5 · high · the default", suffix].concat());
        }
    }
    for source in [c::ModelSource::List, c::ModelSource::Stock, c::ModelSource::Person] {
        let mut me = root.pages().me(root.client()).unwrap().clone();
        let model = me.projects.get_mut(0).unwrap().models.get_mut(0).unwrap();
        model.source = source;
        model.new_until = Some(skein_lib::Wall::from_nanos(WEEK));
        change_me(&mut root, watch, me);
        for mut view in [f::page_view(&root), f::browser_page_view(&root)] {
            for now in [WEEK - 1, WEEK, WEEK + 1] {
                crate::build(&mut view, &root, skein_lib::Wall::from_nanos(now), &mut skein_lib::Queue::uncapped());
                let row = main(view.tree())
                    .iter()
                    .find(|n| n.name.as_deref().is_some_and(|name| name.starts_with(b"claude-opus-5-5")))
                    .unwrap();
                assert_eq!(
                    row.name.as_deref().unwrap().ends_with(" · new".as_bytes()),
                    source == c::ModelSource::List && now < WEEK
                );
                if source == c::ModelSource::List && now < WEEK {
                    assert_eq!(view.next_change(), Some(skein_lib::Wall::from_nanos(WEEK)));
                }
            }
        }
    }
}
#[test]
fn left_out_says_how_many() {
    let (mut root, watch, _) = fixture(true);
    for count in [3, 0] {
        let mut me = root.pages().me(root.client()).unwrap().clone();
        me.projects.get_mut(0).unwrap().left_out = count;
        change_me(&mut root, watch, me);
        for view in [f::page_view(&root), f::browser_page_view(&root)] {
            let facts: Vec<_> =
                main(view.tree()).iter().filter(|n| n.styles.as_slice().contains(&look::SETTINGS_FACTS)).collect();
            let text = facts
                .iter()
                .map(|n| String::from_utf8(f::nodes_text(subtree(view.tree(), n)).into_vec()).unwrap())
                .collect::<Vec<_>>();
            let omitted: Vec<_> = text.iter().filter(|s| s.contains("more models")).map(String::as_str).collect();
            if count == 0 {
                assert_eq!(omitted.len(), 0);
            } else {
                assert_eq!(omitted.as_slice(), ["3 more models don't fit the catalogue."]);
            }
        }
    }
}

fn model_act(root: &mut d::Domain, act: d::pages::models::Act) {
    f::root_event(root, d::Event::Act { action: d::Action::Pages(d::pages::Act::Models(act)) });
}
#[test]
fn a_member_has_no_form() {
    let (mut root, watch, account_watch) = fixture(true);
    for role in [Some(c::HeldRole::Policy { role: 1 }), None] {
        let mut me = root.pages().me(root.client()).unwrap().clone();
        me.deployment_role = role;
        change_me(&mut root, watch, me);
        for view in [f::page_view(&root), f::browser_page_view(&root)] {
            assert!(!main(view.tree()).iter().any(|n| n.styles.as_slice().contains(&look::MODELS_ADD)));
            assert!(!main(view.tree()).iter().any(|n| n.role == Role::TextField || n.role == Role::Select));
        }
    }
    for (own, usable, listed) in [(true, true, true), (false, true, true), (true, false, true), (true, true, false)] {
        let mut doc = d::pages::accounts::document(root.client()).unwrap().clone();
        let account = doc.accounts.get_mut(1).unwrap();
        account.holder = if own {
            c::AccountHolder::Party(root.pages().me(root.client()).unwrap().party.number)
        } else {
            c::AccountHolder::Deployment
        };
        account.sources.get_mut(0).unwrap().usable = usable;
        account.list = listed.then_some(c::AccountList { read: None, tried: None, failure: None, left_out: 0 });
        deliver(&mut root, account_watch, c::Delivery::Accounts(Box::new(doc)));
        for mut view in [f::page_view(&root), f::browser_page_view(&root)] {
            let button = main(view.tree()).iter().find(|n| n.name.as_deref() == Some(b"Read again"));
            assert_eq!(button.is_some(), own && usable && listed);
            if let Some(button) = button {
                assert!(button.styles.as_slice().contains(&look::SETTINGS_SMALL));
                let id = button.id;
                assert_eq!(
                    crate::act(&mut view, id, Act::Press),
                    Some(d::Action::Pages(d::pages::Act::Models(d::pages::models::Act::Read(18))))
                );
            }
        }
    }
}
#[test]
fn the_add_form_binds_the_provider_name_and_press_in_both_shells() {
    let (mut root, _, _) = fixture(false);
    for view in [f::page_view(&root), f::browser_page_view(&root)] {
        assert_eq!(skein_ui::outline::check(view.tree()), None);
        let select = named(view.tree(), "Provider");
        let options: Vec<_> = subtree(view.tree(), select).iter().filter(|n| n.role == Role::Option).collect();
        assert_eq!(
            options.iter().map(|n| n.name.as_deref().unwrap()).collect::<Vec<_>>(),
            [b"Claude".as_slice(), b"OpenAI"]
        );
        assert!(options[0].states.as_slice().contains(&State::Selected));
        assert_eq!(named(view.tree(), "The model’s name").placeholder.as_deref(), Some(b"model-name".as_slice()));
        assert!(named(view.tree(), "Add").states.as_slice().contains(&State::Disabled));
    }
    for terminal in [true, false] {
        let mut view = if terminal { f::page_view(&root) } else { f::browser_page_view(&root) };
        let select = named(view.tree(), "Provider");
        let option = subtree(view.tree(), select).iter().find(|n| n.name.as_deref() == Some(b"OpenAI")).unwrap().id;
        assert_eq!(
            crate::act(&mut view, option, Act::Press),
            Some(d::Action::Pages(d::pages::Act::Models(d::pages::models::Act::Provider(18))))
        );
        let field = named(view.tree(), "The model’s name").id;
        assert_eq!(
            crate::act(&mut view, field, Act::Input { value: b"claude-haiku-5-5".as_slice().into() }),
            Some(d::Action::Pages(d::pages::Act::Models(d::pages::models::Act::Name(
                b"claude-haiku-5-5".as_slice().into()
            ))))
        );
        assert_eq!(
            crate::act(&mut view, field, Act::Submit),
            Some(d::Action::Pages(d::pages::Act::Models(d::pages::models::Act::Add)))
        );
    }
    model_act(&mut root, d::pages::models::Act::Name(b"claude-haiku-5-5".as_slice().into()));
    for mut view in [f::page_view(&root), f::browser_page_view(&root)] {
        let button = named(view.tree(), "Add");
        assert!(!button.states.as_slice().contains(&State::Disabled));
        let id = button.id;
        assert_eq!(
            crate::act(&mut view, id, Act::Press),
            Some(d::Action::Pages(d::pages::Act::Models(d::pages::models::Act::Add)))
        );
    }
}
#[test]
fn a_name_out_of_shape_keeps_add_off() {
    let (mut root, _, _) = fixture(true);
    for name in [b"".as_slice(), b"space name", b"slash/name", &[b'a'; 129]] {
        model_act(&mut root, d::pages::models::Act::Name(name.into()));
        for view in [f::page_view(&root), f::browser_page_view(&root)] {
            assert!(named(view.tree(), "Add").states.as_slice().contains(&State::Disabled));
        }
    }
}
#[test]
fn a_name_the_provider_holds_says_so() {
    let (mut root, _, _) = fixture(true);
    model_act(&mut root, d::pages::models::Act::Name(b"claude-sonnet-5-5".as_slice().into()));
    for view in [f::page_view(&root), f::browser_page_view(&root)] {
        named(view.tree(), "Claude already has that model.");
        assert!(named(view.tree(), "Add").states.as_slice().contains(&State::Disabled));
    }
    model_act(&mut root, d::pages::models::Act::Provider(18));
    for view in [f::page_view(&root), f::browser_page_view(&root)] {
        assert!(!named(view.tree(), "Add").states.as_slice().contains(&State::Disabled));
        assert!(!main(view.tree()).iter().any(|n| n.name.as_deref() == Some(b"Claude already has that model.")));
    }
}

fn send_add(root: &mut d::Domain) -> Token {
    send_model(root, d::pages::models::Act::Add)
}
fn send_model(root: &mut d::Domain, act: d::pages::models::Act) -> Token {
    let mut out = skein_lib::Queue::uncapped();
    let env = skein_lib::Env { now: skein_lib::Time::ZERO, wall: f::NOW, limits: f::domain_limits() };
    d::step(root, &env, d::Event::Act { action: d::Action::Pages(d::pages::Act::Models(act)) }, &mut out);
    d::step(root, &env, d::Event::Client { event: jig_client::Event::Stored }, &mut out);
    let mut sent = None;
    for request in &out {
        if let d::Request::Unlinked {
            request: jig_client::UnlinkedRequest::Down { down: jig_client::Down::Send { attempt, .. } },
        } = request
        {
            assert!(sent.is_none(), "Model act sends one request");
            sent = Some(*attempt);
        }
    }
    sent.expect("Model act sends a request")
}
fn answer_add(root: &mut d::Domain, attempt: Token, answer: c::Answer) {
    f::root_event(
        root,
        d::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Answer {
                    attempt,
                    envelope: skein_lib::RequestEnvelope::Final,
                    answer: Some(jig_client::Answered::Jig(answer)),
                    why: None,
                },
            },
        },
    );
}
#[test]
fn a_provider_without_an_account_can_be_added_to() {
    let (mut root, watch, _) = fixture(false);
    model_act(&mut root, d::pages::models::Act::Provider(18));
    model_act(&mut root, d::pages::models::Act::Name(b"gpt-next".as_slice().into()));
    let attempt = send_add(&mut root);
    for view in [f::page_view(&root), f::browser_page_view(&root)] {
        for name in ["Provider", "The model’s name", "Add"] {
            assert!(named(view.tree(), name).states.as_slice().contains(&State::Disabled));
        }
    }
    answer_add(
        &mut root,
        attempt,
        c::Answer::Done(c::Outcome::SettingsChanged(c::OutcomeSettingsChanged { revision: 1 })),
    );
    for view in [f::page_view(&root), f::browser_page_view(&root)] {
        named(view.tree(), "Added. It shows when an account of OpenAI can run it.");
        assert!(named(view.tree(), "The model’s name").value.as_ref().unwrap().text.is_empty());
        assert!(!main(view.tree()).iter().any(|n| n.name.as_deref() == Some(b"gpt-next")));
    }
    let mut me = root.pages().me(root.client()).unwrap().clone();
    let mut added = model("gpt-next", 18, "gpt-next");
    added.number = 64;
    added.source = c::ModelSource::Person;
    added.estimated = true;
    added.efforts.clear();
    added.effort = None;
    me.projects.get_mut(0).unwrap().models.push(added).unwrap();
    change_me(&mut root, watch, me);
    for view in [f::page_view(&root), f::browser_page_view(&root)] {
        named(view.tree(), "gpt-next · added by you · estimated prices");
        assert!(
            !main(view.tree())
                .iter()
                .any(|n| n.name.as_deref() == Some(b"Added. It shows when an account of OpenAI can run it."))
        );
    }
}
#[test]
fn a_refusal_shows_under_its_act() {
    for read in [false, true] {
        let (mut root, _, watch) = fixture(true);
        let act = if read {
            let mut doc = d::pages::accounts::document(root.client()).unwrap().clone();
            doc.accounts.get_mut(1).unwrap().list =
                Some(c::AccountList { read: None, tried: None, failure: None, left_out: 0 });
            deliver(&mut root, watch, c::Delivery::Accounts(Box::new(doc)));
            d::pages::models::Act::Read(18)
        } else {
            model_act(&mut root, d::pages::models::Act::Name(b"claude-haiku-5-5".as_slice().into()));
            d::pages::models::Act::Add
        };
        let attempt = send_model(&mut root, act);
        if read {
            for view in [f::page_view(&root), f::browser_page_view(&root)] {
                assert!(named(view.tree(), "Reading…").states.as_slice().contains(&State::Disabled));
            }
        }
        answer_add(&mut root, attempt, c::Answer::Refused { refusal: c::Refusal::NotOffered, at: None });
        for view in [f::page_view(&root), f::browser_page_view(&root)] {
            let anchor = if read {
                named(view.tree(), "Read again")
            } else {
                main(view.tree()).iter().find(|n| n.styles.as_slice().contains(&look::MODELS_ADD)).unwrap()
            };
            let parent = main(view.tree())
                .iter()
                .filter(|n| n.styles.as_slice().contains(&look::SETTINGS_SECTION))
                .find(|n| subtree(view.tree(), n).contains(anchor))
                .unwrap();
            let refusal = subtree(view.tree(), parent)
                .iter()
                .find(|n| n.styles.as_slice().contains(&look::SETTINGS_REFUSAL))
                .expect("the act's section holds its refusal");
            assert!(!f::nodes_text(subtree(view.tree(), refusal)).is_empty());
            if !read {
                assert_eq!(
                    named(view.tree(), "The model’s name").value.as_ref().unwrap().text.as_ref(),
                    b"claude-haiku-5-5"
                );
            }
        }
    }
}
#[test]
fn the_provider_select_leaves_out_the_playground_in_the_supplied_order() {
    let original = fixture(false).0;
    let me = original.pages().me(original.client()).unwrap().clone();
    let (mut root, watch, _, _, _) = f::root_at_with_playground(
        Address::Models { project: f::PROJECT },
        &f::domain_limits(),
        b"Shop",
        Some((f::PROJECT, true)),
    );
    assert_eq!(root.forge().playground_provider(), Some(17));
    change_me(&mut root, watch, me);
    for view in [f::page_view(&root), f::browser_page_view(&root)] {
        let options: Vec<_> = subtree(view.tree(), named(view.tree(), "Provider"))
            .iter()
            .filter(|n| n.role == Role::Option)
            .map(|n| n.name.as_deref().unwrap())
            .collect();
        assert_eq!(options, [b"OpenAI".as_slice()]);
    }
}
