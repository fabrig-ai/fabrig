#![expect(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::disallowed_macros,
    reason = "ordinary tests inspect semantic trees and allocate maximal model-name fixtures"
)]
use super::fixtures;
use jig_client::contract as c;
use skein_lib::{List, Token};
use skein_ui::{Builder, Role};
use temper_client_domain::{self as d, Address};
fn delivered(root: &mut d::Domain, watch: Token, value: c::Delivery) {
    fixtures::root_event(
        root,
        d::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched { watch, event: c::WatchEvent::Delivered(value) },
            },
        },
    );
}
fn banner(usable: Option<bool>) -> (d::Domain, Token) {
    let (mut root, watch) = fixtures::account_root();
    if let Some(usable) = usable {
        delivered(
            &mut root,
            watch,
            c::Delivery::Accounts(Box::new(c::Accounts {
                program_hosts: List::with_capacity(0),
                accounts: List::with_capacity(0),
                signing: List::with_capacity(0),
                last_end: None,
                usable,
                settled: true,
                cut: false,
                readiness: List::with_capacity(0),
                readiness_cut: false,
            })),
        );
    }
    fixtures::root_event(&mut root, d::Event::Went { place: d::Place::Address(Address::NewChat { draft: false }) });
    (root, watch)
}
fn text(root: &d::Domain) -> String {
    let view = fixtures::page_view(root);
    let mut text = String::from_utf8(fixtures::text(view.tree()).into_vec()).unwrap();
    for node in view.tree().nodes() {
        if let Some(description) = &node.description {
            text.push_str(core::str::from_utf8(description).unwrap());
        }
    }
    text
}
#[test]
fn the_accounts_banner_precedes_main() {
    let view = fixtures::page_view(&banner(Some(false)).0);
    let nodes = view.tree().nodes();
    let banner = nodes.iter().position(|n| n.name.as_deref() == Some(b"Agents need an LLM account to run.")).unwrap();
    assert!(nodes.iter().position(|n| n.role == Role::Navigation).unwrap() < banner);
    assert!(banner < nodes.iter().position(|n| n.role == Role::Main).unwrap());
}
#[test]
fn usable_accounts_remove_the_banner() {
    assert!(!text(&banner(Some(true)).0).contains("Agents need"));
}
#[test]
fn unknown_accounts_have_no_banner() {
    assert!(!text(&banner(None).0).contains("Agents need"));
}
#[test]
fn a_behind_accounts_document_removes_the_banner() {
    let (mut root, watch) = banner(Some(false));
    fixtures::root_event(
        &mut root,
        d::Event::Client {
            event: jig_client::Event::Up { up: jig_client::Up::WatchEnded { watch, end: c::WatchEnd::Dropped } },
        },
    );
    assert!(!text(&root).contains("Agents need"));
}
#[test]
fn missing_accounts_leave_the_new_chat_composer_available() {
    assert!(fixtures::find(fixtures::page_view(&banner(Some(false)).0).tree(), Role::Button, b"Send").is_some());
}
#[test]
fn the_accounts_page_has_no_accounts_banner() {
    let (mut root, _) = banner(Some(false));
    fixtures::root_event(&mut root, d::Event::Went { place: d::Place::Address(Address::Accounts) });
    assert!(!text(&root).contains("Agents need"));
}
#[test]
fn an_absent_choice_uses_the_unsettled_default() {
    assert!(text(&banner(None).0).contains("the default model"));
}
fn task_root() -> d::Domain {
    account_readiness(None)
}
fn account_readiness(ready: Option<c::PayerReadiness>) -> d::Domain {
    account_facts(ready, List::with_capacity(0))
}
fn account_facts(ready: Option<c::PayerReadiness>, accounts: List<c::AccountLine>) -> d::Domain {
    line_facts(ready, accounts, Some(3), Some(7), None, true)
}
pub(super) fn line_models(task_page: bool) -> List<c::MeModel> {
    let mut models = List::with_capacity(3);
    if !task_page {
        models.push(offer_row(b"catalogue-first", Some(18))).unwrap();
    }
    let mut model = offer_row(if task_page { b"chosen" } else { b"gpt-6-luna" }, Some(17));
    // The shared accounts-page fixture holds account 1.
    model.accounts = List::from_boxed(Box::new([1]));
    models.push(model).unwrap();
    models.push(offer_row(b"second", None)).unwrap();
    models
}

fn line_facts(
    ready: Option<c::PayerReadiness>,
    accounts: List<c::AccountLine>,
    preference: Option<u32>,
    kept: Option<u32>,
    effort: Option<&[u8]>,
    task_page: bool,
) -> d::Domain {
    line_fixture(ready, accounts, preference, kept, effort, task_page).0
}
pub(super) fn provider_byline_root(
    address: Address,
    ready: [c::Readiness; 2],
    present: [bool; 2],
    models: bool,
) -> (d::Domain, Option<Token>) {
    let (mut root, me_watch, task_watch, _, accounts_watch) =
        fixtures::root_at_full(address, &fixtures::limits(), b"Shop");
    let sample = fixtures::playground_root(Address::NewChat { draft: false }, fixtures::PROJECT, true);
    let mut me = fixtures::asking_me(sample.pages().me(sample.client()).unwrap().clone());
    me.providers = List::with_capacity(2);
    let mut doc = super::accounts::document(0);
    doc.cut = false;
    doc.settled = true;
    doc.accounts = List::with_capacity(2);
    doc.readiness = List::with_capacity(2);
    for (index, (provider, name, kind)) in
        [(18, b"anthropic".as_slice(), c::AccountKind::Program), (17, b"openai".as_slice(), c::AccountKind::SignIn)]
            .into_iter()
            .enumerate()
    {
        let mut programs = List::with_capacity(1);
        if kind == c::AccountKind::Program {
            programs.push(c::Text::new(Box::from(b"claude_code".as_slice()), 32).unwrap()).unwrap();
        }
        me.providers
            .push(c::ProviderOffer {
                provider,
                name: c::DocumentText::complete(c::Text::new(Box::from(name), 32).unwrap()),
                key_bytes: 4096,
                sign_in: kind == c::AccountKind::SignIn,
                key: false,
                programs,
                programs_cut: false,
                none: false,
                signed_in_plan: kind == c::AccountKind::SignIn,
                manage_usage: None,
            })
            .unwrap();
        if present[index] {
            let mut row = account(provider, kind, None);
            row.provider = Some(provider);
            row.provider_name = Some(c::DocumentText::complete(c::Text::new(Box::from(name), 32).unwrap()));
            row.sources = List::with_capacity(1);
            row.sources.push(source(kind, true)).unwrap();
            doc.accounts.push(row).unwrap();
        }
        doc.readiness
            .push(c::PayerReadiness {
                provider,
                payer: c::AccountUse::Party(73),
                readiness: ready[index],
                settled: Some(true),
            })
            .unwrap();
    }
    doc.usable = ready.contains(&c::Readiness::Usable);
    let project = me.projects.iter_mut().find(|p| p.project == fixtures::PROJECT).unwrap();
    let sample_model = project.models.get(0).unwrap().clone();
    project.models = List::with_capacity(3);
    if models {
        for (provider, name) in [(17, "gpt-6.1-sol"), (17, "gpt-6-luna"), (18, "claude-opus-5-5")] {
            let mut model = sample_model.clone();
            model.provider = Some(provider);
            model.name = c::Text::new(Box::from(name.as_bytes()), 32).unwrap();
            project.models.push(model).unwrap();
        }
    }
    project.defaults = List::with_capacity(1);
    let preview = if ready[1] == c::Readiness::Usable && present[1] {
        Some(b"gpt-6.1-sol".as_slice())
    } else if ready[0] == c::Readiness::Usable && present[0] {
        Some(b"claude-opus-5-5".as_slice())
    } else {
        None
    };
    if models && let Some(preview) = preview {
        project
            .defaults
            .push(c::MeDefault {
                template: if let Address::Ask { .. } = root.pages().address() { 5 } else { jig_client::shapes::CHAT },
                choice: c::LlmChoice {
                    later: None,
                    account: None,
                    main: c::ModelEffort { model: model_text(preview), effort: None },
                    subagents: List::with_capacity(0),
                },
            })
            .unwrap();
    }
    delivered(&mut root, me_watch, c::Delivery::Me(Box::new(me)));
    delivered(&mut root, accounts_watch.unwrap(), c::Delivery::Accounts(Box::new(doc)));
    (root, task_watch)
}

#[test]
fn the_picker_closed_line_names_no_account() {
    for address in [Address::NewChat { draft: false }, Address::Ask { project: fixtures::PROJECT }] {
        let (root, _) = provider_byline_root(address, [c::Readiness::Usable, c::Readiness::Usable], [true, true], true);
        for view in [fixtures::page_view(&root), fixtures::browser_page_view(&root)] {
            let node = view
                .tree()
                .nodes()
                .iter()
                .find(|n| n.role == Role::Disclosure && n.name.as_deref() == Some(b"Model"))
                .unwrap();
            assert_eq!(node.description.as_deref(), Some(b"gpt-6.1-sol".as_slice()));
            assert!(!String::from_utf8_lossy(node.description.as_deref().unwrap()).contains("plan"));
        }
    }
}

fn line_fixture(
    ready: Option<c::PayerReadiness>,
    accounts: List<c::AccountLine>,
    preference: Option<u32>,
    kept: Option<u32>,
    effort: Option<&[u8]>,
    task_page: bool,
) -> (d::Domain, Token, Option<Token>, Token) {
    let (mut root, me_watch, watch, _, accounts_watch) = fixtures::root_at_full(
        if task_page { Address::Chat { task: 9 } } else { Address::NewChat { draft: false } },
        &fixtures::limits(),
        b"Shop",
    );
    let mut me = d::pages::accounts::current_me(root.client()).unwrap().clone();
    let models = line_models(task_page);
    me.projects.get_mut(0).unwrap().models = models;
    if !task_page {
        let template = fixtures::playground_root(Address::NewChat { draft: false }, fixtures::PROJECT, true);
        me.projects.get_mut(0).unwrap().templates =
            template.pages().me(template.client()).unwrap().projects.get(0).unwrap().templates.clone();
    }
    me.providers = List::with_capacity(1);
    me.providers
        .push(c::ProviderOffer {
            provider: 17,
            name: c::DocumentText::complete(
                c::Text::new(Box::from(if task_page { b"Supplied Provider".as_slice() } else { b"openai" }), 32)
                    .unwrap(),
            ),
            key_bytes: 4096,
            sign_in: true,
            key: true,
            programs: List::with_capacity(0),
            programs_cut: false,
            none: false,
            signed_in_plan: false,
            manage_usage: None,
        })
        .unwrap();
    delivered(&mut root, me_watch, c::Delivery::Me(Box::new(me)));
    let mut readiness = List::with_capacity(1);
    if let Some(ready) = ready {
        readiness.push(ready).unwrap();
    }
    delivered(
        &mut root,
        accounts_watch.unwrap(),
        c::Delivery::Accounts(Box::new(c::Accounts {
            program_hosts: List::with_capacity(0),
            accounts,
            signing: List::with_capacity(0),
            last_end: None,
            usable: true,
            settled: true,
            cut: false,
            readiness,
            readiness_cut: false,
        })),
    );
    let mut task = fixtures::task::document();
    task.chip.task = 9;
    task.project = fixtures::PROJECT;
    task.chip.executor = c::ChipExecutor::Agent { charter: 1 };
    task.budget.funder = c::Funder::Pool { project: fixtures::PROJECT, person: 73, period: 1 };
    task.acts = List::with_capacity(2);
    for act in [c::ActKind::Message, c::ActKind::Amend] {
        task.acts.push(c::Offered { act, admission: c::Admission::Allowed, effect: None }).unwrap();
    }
    task.llm = Some(c::LlmChoice {
        later: None,
        account: preference,
        main: c::ModelEffort {
            model: c::Text::new(Box::from(*b"chosen"), 32).unwrap(),
            effort: effort.map(|name| c::Text::new(Box::from(name), 32).unwrap()),
        },
        subagents: List::with_capacity(0),
    });
    task.kept_account = kept;
    if task_page {
        delivered(&mut root, watch.unwrap(), c::Delivery::Task(Box::new(task)));
    }
    (root, me_watch, watch, accounts_watch.unwrap())
}
#[test]
fn a_requested_model_is_labelled_for_the_next_run() {
    assert_eq!(using_text(&task_root()), "chosen");
}
#[test]
fn an_unknown_readiness_keeps_the_model_only() {
    assert_eq!(using_text(&task_root()), "chosen");
}
fn waiting_task(wait: Option<c::TaskRunWait>) -> c::Task {
    let mut task = fixtures::task::document();
    task.llm = None;
    task.run_wait = wait;
    task
}
fn wait(why: c::TaskWaitWhy) -> skein_ui::Tree {
    let mut b = Builder::new(&fixtures::tree_limits());
    let mut bindings = crate::frame::Bindings::new(32);
    b.open(Role::Page);
    let (account, source) = if why == c::TaskWaitWhy::Empty { (None, None) } else { (Some(7), Some(8)) };
    crate::choice::waiting(
        &mut b,
        &task_root(),
        &waiting_task(Some(c::TaskRunWait::Account { account, source, why })),
        &fixtures::words(),
        &mut bindings,
        &mut crate::words::Times::new(fixtures::NOW),
    );
    b.close();
    b.finish()
}
#[test]
fn an_empty_wait_connects_a_model_and_uses_waiting_style() {
    let tree = wait(c::TaskWaitWhy::Empty);
    assert!(fixtures::find(&tree, Role::Paragraph, b"No account can run this chat yet.").is_some());
    assert!(fixtures::find(&tree, Role::Link, b"Connect a model").is_some());
    let status = tree
        .nodes()
        .iter()
        .find(|node| node.role == Role::Status && node.name.as_deref() == Some(b"Waiting for an account"))
        .unwrap();
    let styles = status.styles.as_slice();
    assert!(styles.contains(&crate::look::STATUS_WAITING));
    assert!(!styles.contains(&crate::look::STATUS_ACTIVE));
}

#[test]
fn a_limited_wait_links_to_resume() {
    assert!(
        fixtures::find(
            &wait(c::TaskWaitWhy::Source(c::TaskSourceState::Limited { until: None })),
            Role::Link,
            b"Resume"
        )
        .is_some()
    );
}
#[test]
fn a_sign_in_wait_links_to_sign_in() {
    assert!(
        fixtures::find(
            &wait(c::TaskWaitWhy::Source(c::TaskSourceState::NeedsSignIn)),
            Role::Link,
            b"Sign in with an LLM provider"
        )
        .is_some()
    );
}
#[test]
fn an_unknown_reset_has_no_time() {
    assert!(
        !String::from_utf8(
            fixtures::text(&wait(c::TaskWaitWhy::Source(c::TaskSourceState::Limited { until: None }))).into_vec()
        )
        .unwrap()
        .contains("until")
    );
}
#[test]
fn a_known_reset_shows_its_utc_moment() {
    let until = Some(skein_lib::Wall::from_nanos(1_791_828_240_000_000_000));
    let shown = String::from_utf8(
        fixtures::text(&wait(c::TaskWaitWhy::Source(c::TaskSourceState::Limited { until }))).into_vec(),
    )
    .unwrap();
    assert!(shown.contains("Limited until Oct 12, 18:04 UTC"), "{shown}");
}

fn model_words() -> crate::words::Words {
    crate::words::Words::english(fixtures::FORGE, 128, crate::words::Shell::Terminal).unwrap()
}
const NAME_BYTES: u32 = jig_client_protocol_wire::v20::CEILINGS.me_model_name;
const EFFORT_BYTES: u32 = jig_client_protocol_wire::v20::CEILINGS.me_model_efforts_item;
const CUT_MARK: &[u8] = "…".as_bytes();
fn shown_bytes() -> usize {
    usize::try_from(crate::choice::MODEL_SHOWN).unwrap()
}
fn prefix_bytes() -> usize {
    shown_bytes().checked_sub(CUT_MARK.len()).expect("display bound includes cut mark")
}
fn catalogue(name: &[u8], efforts: &[&[u8]]) -> d::Domain {
    let mut declared = List::with_capacity(u32::try_from(efforts.len()).unwrap());
    for effort in efforts {
        declared.push(c::Text::new(Box::from(*effort), EFFORT_BYTES).unwrap()).unwrap();
    }
    let mut models = List::with_capacity(1);
    let mut model = offer_row(name, None);
    model.effort = declared.get(0).cloned();
    model.efforts = declared;
    models.push(model).unwrap();
    catalogue_rows(models)
}
fn catalogue_rows(models: List<c::MeModel>) -> d::Domain {
    let (mut root, me_watch, _) = fixtures::root_at(Address::NewChat { draft: false }, &fixtures::limits());
    let mut me = d::pages::accounts::current_me(root.client()).unwrap().clone();
    let project = me.projects.get_mut(0).unwrap();
    project.templates = List::with_capacity(1);
    project
        .templates
        .push(c::Template {
            resources: List::with_capacity(0),
            number: 1,
            name: c::DocumentText::complete(c::Text::new(Box::from(*b"Chat"), 32).unwrap()),
            executor: c::TemplateExecutor::Charter(1),
            contract: c::TaskContractDocument::Report { words: 32 },
            shallower: 1,
            budget: c::TemplateBudget::AsAllowed { cap: None },
            deadline: c::TemplateDeadline::None,
            tracked: None,
            flags: c::TaskFlags {
                ended_by: jig_client::contract::EndedBy::Result,
                conversation: true,
                listed: false,
                retention: c::Retention::Kept,
            },
            label: 7,
            grace: None,
            llm: None,
        })
        .unwrap();
    project.models = models;
    me.providers = List::with_capacity(2);
    for (number, name) in [(17, b"First".as_slice()), (18, b"Second".as_slice())] {
        let provider = provider_offer(number, name);
        me.providers.push(provider).unwrap();
    }
    delivered(&mut root, me_watch, c::Delivery::Me(Box::new(me)));
    root
}
fn catalogue_view(root: &d::Domain) -> crate::View {
    let mut tree = fixtures::tree_limits();
    tree.text_per_node = 1024;
    let mut view = crate::View::new(&crate::Limits { tree }, &fixtures::limits(), model_words(), fixtures::FORGE);
    fixtures::build(&mut view, root);
    view
}
fn shows_requested(view: &crate::View, label: &[u8]) {
    assert!(fixtures::find(view.tree(), Role::Button, label).is_some());
    let byline = fixtures::find(view.tree(), Role::Disclosure, b"Model").unwrap();
    assert_eq!(view.tree().nodes().iter().find(|n| n.id == byline).unwrap().description.as_deref(), Some(label));
}
fn marked_prefix() -> Vec<u8> {
    [vec![b'm'; prefix_bytes()], CUT_MARK.to_vec()].concat()
}
#[test]
fn a_long_model_name_shows_a_marked_prefix() {
    let mut root = catalogue(&vec![b'm'; usize::try_from(NAME_BYTES).unwrap()], &[]);
    edit(&mut root, d::pages::choice::Act::Model { index: 0, account: None });
    shows_requested(&catalogue_view(&root), &marked_prefix());
}
#[test]
fn a_name_at_the_display_bound_shows_whole() {
    let name = vec![b'm'; shown_bytes()];
    let mut root = catalogue(&name, &[]);
    edit(&mut root, d::pages::choice::Act::Model { index: 0, account: None });
    shows_requested(&catalogue_view(&root), &name);
}
#[test]
fn a_name_one_past_the_display_bound_shows_a_marked_prefix() {
    let name = vec![b'm'; shown_bytes() + 1];
    let mut root = catalogue(&name, &[]);
    edit(&mut root, d::pages::choice::Act::Model { index: 0, account: None });
    shows_requested(&catalogue_view(&root), &marked_prefix());
}
#[test]
fn a_cut_model_name_ends_at_a_utf8_boundary() {
    let letter = "é";
    let name = letter.repeat(usize::try_from(NAME_BYTES).unwrap() / letter.len());
    let shown = crate::choice::shown(name.as_bytes(), &model_words());
    assert_eq!(
        String::from_utf8(shown.into_vec()).unwrap(),
        format!("{}…", letter.repeat(prefix_bytes() / letter.len()))
    );
}
#[test]
fn only_the_selected_duplicate_model_shows_its_efforts() {
    let mut models = List::with_capacity(2);
    for (provider, effort) in [(17, b"first-effort".as_slice()), (18, b"second-effort".as_slice())] {
        let mut efforts = List::with_capacity(1);
        efforts.push(c::Text::new(Box::from(effort), EFFORT_BYTES).unwrap()).unwrap();
        let mut model = offer_row(b"shared", Some(provider));
        model.effort = efforts.get(0).cloned();
        model.efforts = efforts;
        models.push(model).unwrap();
    }
    let mut root = catalogue_rows(models);
    edit(&mut root, d::pages::choice::Act::Model { index: 1, account: None });
    let view = catalogue_view(&root);
    let efforts: Vec<_> = view
        .tree()
        .nodes()
        .iter()
        .filter(|node| {
            node.role == Role::Button
                && [b"first-effort".as_slice(), b"second-effort"].contains(&node.name.as_deref().unwrap_or_default())
        })
        .map(|node| node.name.as_deref())
        .collect();
    assert_eq!(efforts, [Some(b"second-effort".as_slice())]);
}

#[test]
fn the_chosen_model_and_effort_are_pressed() {
    let mut root = catalogue(b"needs-effort", &[b"low", b"high"]);
    edit(&mut root, d::pages::choice::Act::Model { index: 0, account: None });
    edit(&mut root, d::pages::choice::Act::Effort { index: 1 });
    let view = catalogue_view(&root);
    let states = |name: &[u8]| {
        let node = view.tree().nodes().iter().find(|n| n.role == Role::Button && n.name.as_deref() == Some(name));
        node.unwrap().states.as_slice().to_vec()
    };
    for name in [b"needs-effort".as_slice(), b"high"] {
        assert!(states(name).contains(&skein_ui::State::Pressed));
        assert!(!states(name).contains(&skein_ui::State::Selected));
    }
    assert!(!states(b"low").contains(&skein_ui::State::Pressed));
}
#[test]
fn a_model_press_enables_start_with_its_default_effort() {
    let mut root = catalogue(b"needs-effort", &[b"high"]);
    edit(&mut root, d::pages::choice::Act::Model { index: 0, account: None });
    fixtures::root_event(
        &mut root,
        d::Event::Act { action: d::Action::Pages(d::pages::Act::ChatWords { words: b"Hello".as_slice().into() }) },
    );
    let mut view = catalogue_view(&root);
    let send = view
        .tree()
        .nodes()
        .iter()
        .find(|n| n.role == Role::Button && n.name.as_deref() == Some(b"Send"))
        .expect("send");
    assert!(!send.states.as_slice().contains(&skein_ui::State::Disabled));
    let entry = fixtures::find(view.tree(), Role::TextArea, b"What do you want done?").expect("entry");
    assert!(crate::act(&mut view, entry, skein_ui::Act::Submit).is_some());
}

fn readiness(state: c::Readiness) -> c::PayerReadiness {
    c::PayerReadiness { provider: 17, payer: c::AccountUse::Party(73), readiness: state, settled: Some(true) }
}
#[test]
fn offered_models_do_not_infer_provider_readiness() {
    for ready in [None, Some(c::Readiness::Unknown), Some(c::Readiness::Unavailable), Some(c::Readiness::Usable)] {
        let root = account_readiness(ready.map(readiness));
        assert_eq!(using_text(&root), "chosen");
        assert!(!text(&root).contains("Account readiness unknown"));
    }
}
#[test]
fn another_payers_readiness_does_not_decide_this_payers() {
    let mut row = readiness(c::Readiness::Unavailable);
    row.payer = c::AccountUse::Project(fixtures::PROJECT);
    assert!(!text(&account_readiness(Some(row))).contains("needs an account"));
}
#[test]
fn the_provider_keeps_its_supplied_name() {
    assert!(text(&task_root()).contains("Supplied Provider"));
}
#[test]
fn the_accounts_banner_is_a_named_region() {
    let view = fixtures::page_view(&banner(Some(false)).0);
    let node = view.tree().nodes().iter().find(|n| n.styles.as_slice().contains(&crate::look::ACCOUNTS)).unwrap();
    assert_eq!(node.role, Role::Region);
    assert_eq!(node.name.as_deref(), Some(b"LLM account".as_slice()));
}
#[test]
fn the_picker_is_on_the_composers_hint_row() {
    let view = fixtures::page_view(&task_root());
    let nodes = view.tree().nodes();
    let row = nodes.iter().position(|n| n.styles.as_slice().contains(&crate::look::FOOT_ROW)).unwrap();
    let picker = nodes.iter().position(|n| n.role == Role::Disclosure && n.name.as_deref() == Some(b"Model")).unwrap();
    assert_eq!(nodes[picker].description.as_deref(), Some(b"chosen".as_slice()));
    assert!(nodes[picker].states.as_slice().contains(&skein_ui::State::FoldsAway));
    assert!(row < picker && picker < row + usize::try_from(nodes[row].size).unwrap());
}

fn edit(root: &mut d::Domain, act: d::pages::choice::Act) {
    fixtures::root_event(root, d::Event::Act { action: d::Action::Pages(d::pages::Act::Choice(act)) });
}

#[test]
fn changing_is_said_in_the_menu() {
    let mut root = task_root();
    edit(&mut root, d::pages::choice::Act::Model { index: 1, account: None });
    assert!(text(&root).contains("Changing…"));
    assert_eq!(using_text(&root), "chosen");
}
fn named_account() -> d::Domain {
    one_account(3, false)
}
fn one_account(number: u32, plan: bool) -> d::Domain {
    let mut accounts = List::with_capacity(1);
    accounts
        .push(c::AccountLine {
            list: None,
            facts: List::with_capacity(0),
            number,
            place: 1,
            uses: None,
            plan,
            provider: None,
            provider_name: None,
            manage_usage: None,
            display: c::DocumentText::complete(c::Text::new(Box::from(*b"Personal"), 32).unwrap()),
            holder: c::AccountHolder::Party(73),
            sources: List::with_capacity(0),
            cut: false,
        })
        .unwrap();
    account_facts(None, accounts)
}

#[test]
fn an_unread_readiness_withholds_the_accounts_words() {
    assert_eq!(using_text(&one_account(7, true)), "chosen");
}
#[test]
fn a_requested_account_waits_for_readiness() {
    let root = named_account();
    assert_eq!(using_text(&root), "chosen");
}
#[test]
fn a_waiting_account_shows_its_display() {
    let root = named_account();
    let mut b = Builder::new(&fixtures::tree_limits());
    b.open(Role::Page);
    crate::choice::waiting(
        &mut b,
        &root,
        &waiting_task(Some(c::TaskRunWait::Account {
            account: Some(3),
            source: None,
            why: c::TaskWaitWhy::Refreshing,
        })),
        &fixtures::words(),
        &mut crate::frame::Bindings::new(32),
        &mut crate::words::Times::new(fixtures::NOW),
    );
    b.close();
    assert!(String::from_utf8(fixtures::text(&b.finish()).into_vec()).unwrap().contains("Waiting account: Personal"));
}

#[test]
fn a_task_waiting_on_a_moving_account_says_why() {
    let root = named_account();
    let mut b = Builder::new(&fixtures::tree_limits());
    b.open(Role::Page);
    crate::choice::waiting(
        &mut b,
        &root,
        &waiting_task(Some(c::TaskRunWait::Account {
            account: Some(3),
            source: Some(8),
            why: c::TaskWaitWhy::Source(c::TaskSourceState::Moving),
        })),
        &fixtures::words(),
        &mut crate::frame::Bindings::new(32),
        &mut crate::words::Times::new(fixtures::NOW),
    );
    b.close();
    let tree = b.finish();
    assert!(
        fixtures::find(&tree, Role::Paragraph, b"Waiting because the account is moving to another deployment")
            .is_some()
    );
    assert!(!tree.nodes().iter().any(|node| node.role == Role::Button || node.role == Role::Link));
}

#[test]
fn a_refused_next_run_says_why() {
    use c::{HelloRefusal as F, HostKind as H, TaskWaitRefused as R};
    for (name, refusal, expected) in [
        (
            "engine",
            R::Hello { kind: H::Engine, at: fixtures::NOW, why: F::ReservedChannel, figures: None },
            "Next run refused · An engine host was refused at 00:00. Channel zero belongs to the engine's own host.",
        ),
        (
            "worker",
            R::Hello { kind: H::Worker, at: fixtures::NOW, why: F::Position { worker: 7 }, figures: None },
            "Next run refused · A worker was refused at 00:00. Unconfigured or occupied worker position 7.",
        ),
    ] {
        let root = task_root();
        let mut builder = Builder::new(&fixtures::tree_limits());
        let mut times = crate::words::Times::new(fixtures::NOW);
        crate::choice::waiting(
            &mut builder,
            &root,
            &waiting_task(Some(c::TaskRunWait::Refused(refusal))),
            &fixtures::words(),
            &mut crate::frame::Bindings::new(32),
            &mut times,
        );
        assert_eq!(fixtures::text(&builder.finish()).as_ref(), expected.as_bytes(), "{name}");
        let midnight = match refusal {
            R::Hello { .. } => Some(skein_lib::Wall::from_nanos(86_400_000_000_000)),
            R::Bounds { .. } => None,
        };
        assert_eq!(times.next, midnight, "{name}");
        let source = fixtures::chat::messaging_root();
        let mut task = source.pages().chat().unwrap().task(source.client()).unwrap().clone();
        task.flags.ended_by = c::EndedBy::Requester;
        task.run_wait = Some(c::TaskRunWait::Refused(refusal));
        let c::ChipPhaseFacts { phase, .. } = task.chip.details.as_mut();
        *phase = c::ChipPhase::Active(c::RunStage::Due);
        let (mut root, _, watch) =
            fixtures::root_at(Address::Chat { task: fixtures::chat::TASK }, &fixtures::domain_limits());
        delivered(&mut root, watch.unwrap(), c::Delivery::Task(Box::new(task)));
        let view = fixtures::page_view(&root);
        assert!(fixtures::find(view.tree(), Role::Status, b"Cannot start yet").is_some(), "{name}");
        assert!(fixtures::find(view.tree(), Role::Paragraph, expected.as_bytes()).is_some(), "{name}");
        assert!(fixtures::find(view.tree(), Role::TextArea, b"Message").is_some(), "{name}");
        assert!(fixtures::find(view.tree(), Role::Button, b"Send").is_some(), "{name}");
        assert_eq!(view.next_change(), midnight, "{name}");
    }
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "one table retains each hello cause and all ten bound clauses; client/pages/accounts.md, 2"
)]
fn every_hello_refusal_keeps_its_cause_and_numbers() {
    use c::{AgentShortfall as S, Figures, HelloRefusal as F, HostKind as K, TaskWaitRefused as R};
    let hello = |why| R::Hello { kind: K::Worker, at: fixtures::NOW, why, figures: None };
    let bounds = |why, required, offered| R::Bounds {
        kind: K::Worker,
        why: Some(why),
        figures: Some(Figures { required, offered }),
    };
    let mut rows = Vec::new();
    for (why, cause) in [
        (F::Workers { limit: 7 }, "No room for another worker. Limit: 7."),
        (
            F::StopBound { grace: skein_lib::Duration::from_secs(30) },
            "It would not stop before the engine's grace ends. Allowed grace: 30.",
        ),
        (F::Agent { why: S::Model { charter: 3, model: 8 } }, "Agent model missing for charter 3. Model position 8."),
        (
            F::Agent { why: S::Window { charter: 3, model: 8 } },
            "Agent window too small for charter 3. Model position 8.",
        ),
        (
            F::Agent { why: S::Output { charter: 3, model: 8 } },
            "Agent output too small for charter 3. Model position 8.",
        ),
        (F::Agent { why: S::Turns { charter: 2 } }, "Agent turns too few."),
        (F::Agent { why: S::Spend }, "Agent spend too small."),
        (F::Agent { why: S::Time { charter: 2 } }, "Agent time too short."),
        (F::Agent { why: S::Waiting { charter: 2 } }, "Agent waiting too short."),
        (
            F::Agent { why: S::Checks { charter: 2 } },
            "Agent check timeout exceeds reserve_time for charter 2. See temper help settings for held revisions.",
        ),
        (F::Agent { why: S::Message }, "Agent message bound too small."),
        (F::Agent { why: S::Inbox }, "Agent inbox too small."),
        (F::Agent { why: S::Unacknowledged }, "Agent unacknowledged turns too few."),
    ] {
        rows.push((hello(why), format!("Next run refused · A worker was refused at 00:00. {cause}")));
    }
    for (why, required, offered, clause) in [
        (S::Turns { charter: 3 }, 100_000, 800, "its charter asks 100,000 turns, and the largest worker takes 800."),
        (S::Time { charter: 3 }, 7_200_000, 3_600_000, "its charter asks 2 h a run, and the largest worker takes 1 h."),
        (
            S::Waiting { charter: 3 },
            1_800_000,
            600_000,
            "its charter asks 30 min of waiting, and the largest worker takes 10 min.",
        ),
        (
            S::Checks { charter: 3 },
            300_000,
            600_000,
            "its charter keeps 5 min for checks, and the quickest worker's checks take 10 min.",
        ),
        (
            S::Window { charter: 3, model: 1 },
            200_000,
            128_000,
            "its charter's model at position 1 asks a window of 200,000 tokens, and the largest worker takes 128,000.",
        ),
        (
            S::Output { charter: 3, model: 1 },
            64000,
            32000,
            "its charter's model at position 1 asks 64,000 output tokens, and the largest worker takes 32,000.",
        ),
        (S::Spend, 5000, 2000, "a run here may spend 5,000, and the largest worker takes 2,000."),
        (S::Message, 65536, 16384, "a message here may be 65,536 bytes, and the largest worker takes 16,384."),
        (S::Inbox, 12, 8, "a start here sends 12 messages, and the largest worker takes 8."),
        (S::Unacknowledged, 64, 32, "a run here keeps 64 turns unacknowledged, and the largest worker takes 32."),
        (
            S::Time { charter: 3 },
            59000,
            0,
            "its charter asks under 1 min a run, and the largest worker takes under 1 min.",
        ),
        (
            S::Waiting { charter: 3 },
            5_400_000,
            3_600_000,
            "its charter asks 1 h 30 min of waiting, and the largest worker takes 1 h.",
        ),
    ] {
        rows.push((bounds(why, required, offered), format!("No worker can take this: {clause}")));
    }
    rows.extend([
        (
            R::Bounds { kind: K::Worker, why: None, figures: None },
            String::from("No worker can take this: each falls short of another bound its charter asks."),
        ),
        (
            R::Bounds { kind: K::Worker, why: Some(S::Turns { charter: 3 }), figures: None },
            String::from("No worker can take this. Agent turns too few."),
        ),
        (
            R::Bounds {
                kind: K::Engine,
                why: Some(S::Turns { charter: 3 }),
                figures: Some(Figures { required: 100_000, offered: 800 }),
            },
            String::from("The engine's own host can't take this: its charter asks 100,000 turns, and it takes 800."),
        ),
        (
            R::Hello {
                kind: K::Worker,
                at: skein_lib::Wall::from_nanos(38_520_000_000_000),
                why: F::Agent { why: S::Turns { charter: 3 } },
                figures: Some(Figures { required: 100_000, offered: 800 }),
            },
            String::from(
                "No worker can take this: charter 3 asks 100,000 turns, and the worker refused at 10:42 takes 800.",
            ),
        ),
    ]);
    // The widest count, model position and duration fit the bound words' holes.
    rows.extend([
        (bounds(S::Window { charter: u32::MAX, model: u32::MAX }, u64::MAX, u64::MAX), String::from("No worker can take this: its charter's model at position 4294967295 asks a window of 18,446,744,073,709,551,615 tokens, and the largest worker takes 18,446,744,073,709,551,615.")),
        (bounds(S::Time { charter: u32::MAX }, u64::MAX, u64::MAX), String::from("No worker can take this: its charter asks 213503982334 d a run, and the largest worker takes 213503982334 d.")),
    ]);
    let root = task_root();
    let words = [fixtures::words(), fixtures::browser_words()];
    let bound_words = [
        crate::words::Words::english(fixtures::FORGE, 1, crate::words::Shell::Terminal).unwrap(),
        crate::words::Words::english(fixtures::FORGE, 1, crate::words::Shell::Browser).unwrap(),
    ];
    for (why, expected) in rows {
        wait_pointers(expected.as_bytes());
        let vocabularies = match why {
            R::Bounds { .. } => &bound_words,
            R::Hello { .. } => &words,
        };
        for w in vocabularies {
            let mut builder = Builder::new(&fixtures::tree_limits());
            let at = match why {
                R::Hello { at, .. } => at,
                R::Bounds { .. } => fixtures::NOW,
            };
            crate::choice::waiting(
                &mut builder,
                &root,
                &waiting_task(Some(c::TaskRunWait::Refused(why))),
                w,
                &mut crate::frame::Bindings::new(32),
                &mut crate::words::Times::new(at),
            );
            let text = fixtures::text(&builder.finish());
            wait_pointers(&text);
            assert_eq!(text.as_ref(), expected.as_bytes(), "{why:?}");
        }
    }
}

pub(super) fn wait_pointers(text: &[u8]) {
    let text = core::str::from_utf8(text).unwrap();
    assert!(!text.contains("temper ctl "), "page wait names ctl: {text}");
    for after in text.split("temper help ").skip(1) {
        let page = after.split(|c: char| !c.is_ascii_alphanumeric() && c != '-').next().unwrap();
        assert!(temper_docs::find(page.as_bytes()).is_some(), "missing help page {page}");
    }
}

fn source(kind: c::AccountKind, usable: bool) -> c::AccountSource {
    c::AccountSource {
        name: Some(Box::from(b"Claude Code".as_slice())),
        place: 1,
        number: 1,
        kind,
        state: c::AccountState::Granted,
        usable,
        expiring: false,
        attention: false,
        spent_until: None,
    }
}
fn account(number: u32, kind: c::AccountKind, address: Option<&[u8]>) -> c::AccountLine {
    let mut sources = List::with_capacity(2);
    sources.push(source(c::AccountKind::Program, false)).unwrap();
    sources.push(source(kind, true)).unwrap();
    c::AccountLine {
        list: None,
        facts: List::with_capacity(0),
        number,
        place: number,
        uses: None,
        plan: kind == c::AccountKind::SignIn,
        provider: Some(17),
        provider_name: None,
        manage_usage: address.map(Box::from),
        display: c::DocumentText::complete(c::Text::new(Box::from(b"Personal".as_slice()), 32).unwrap()),
        holder: c::AccountHolder::Party(73),
        sources,
        cut: false,
    }
}
fn accounts(rows: &[c::AccountLine]) -> List<c::AccountLine> {
    let mut accounts = List::with_capacity(u32::try_from(rows.len()).unwrap());
    for row in rows {
        accounts.push(row.clone()).unwrap();
    }
    accounts
}
pub(super) fn using_text(root: &d::Domain) -> String {
    let mut b = Builder::new(&fixtures::tree_limits());
    crate::choice::using(
        &mut b,
        root,
        root.pages().chat().and_then(|chat| chat.task(root.client())),
        &fixtures::words(),
    );
    let text = String::from_utf8(fixtures::text(&b.finish()).into_vec()).unwrap();
    let text = text.split(" · Manage usage").next().unwrap();
    if text == "Using the default model" {
        return text.to_owned();
    }
    if text.starts_with("Using your ") && text.contains("API key") {
        text.strip_prefix("Using your ").unwrap().to_owned()
    } else {
        text.strip_prefix("Using ").unwrap_or(text).to_owned()
    }
}
#[test]
fn a_keys_line_uses_the_configured_provider_name_and_appends_effort() {
    let root = line_facts(
        Some(readiness(c::Readiness::Usable)),
        accounts(&[account(3, c::AccountKind::Key, None)]),
        Some(3),
        Some(7),
        Some(b"high"),
        true,
    );
    assert_eq!(using_text(&root), "Supplied Provider API key · chosen · high");
}
#[test]
fn task_lines_take_their_own_templates_preview_and_offered_effort() {
    use super::chat::{model_choice, model_line_root};
    for (own, preview, charter, using, closed) in [
        (None, Some(1), 1, "GPT-6 Astra · high", "GPT-6 Astra · high"),
        (None, Some(5), 37, "GPT-6 Astra · high", "GPT-6 Astra · high"),
        (Some(model_choice("gpt-6-astra", None)), None, 1, "GPT-6 Astra · high", "GPT-6 Astra · high"),
        (Some(model_choice("gpt-6-astra", Some("low"))), None, 1, "GPT-6 Astra · low", "GPT-6 Astra · low"),
        (None, None, 37, "Using the default model", "the default model"),
    ] {
        let mut task = super::chat::asking_task();
        task.chip.executor = c::ChipExecutor::Agent { charter };
        task.activity = 1;
        task.llm = own;
        let root = model_line_root(task, None, preview);
        assert_eq!(using_text(&root), using);
        let task = root.pages().chat().unwrap().task(root.client()).unwrap();
        assert_eq!(crate::choice::closed_line(&root, Some(task), &fixtures::words()).as_ref(), closed.as_bytes());
    }
}
#[test]
fn a_plan_has_a_manage_usage_link_only_for_a_safe_address() {
    for address in [
        None,
        Some(b"https://provider.example/usage".as_slice()),
        Some(b"javascript:bad"),
        Some(b"https://provider.example/usage?secret"),
    ] {
        let root = line_facts(
            Some(readiness(c::Readiness::Usable)),
            accounts(&[account(3, c::AccountKind::SignIn, address)]),
            Some(3),
            None,
            None,
            true,
        );
        let mut b = Builder::new(&fixtures::tree_limits());
        crate::choice::using(&mut b, &root, root.pages().chat().unwrap().task(root.client()), &fixtures::words());
        let tree = b.finish();
        let link = tree.nodes().iter().find(|n| n.role == Role::Link && n.name.as_deref() == Some(b"Manage usage"));
        let safe = address == Some(b"https://provider.example/usage".as_slice());
        assert_eq!(link.is_some(), safe);
        if let Some(link) = link {
            assert_eq!(link.external.as_deref(), address);
        }
        assert_eq!(using_text(&root), "Supplied Provider plan · chosen");
    }
}
#[test]
fn a_program_line_uses_its_first_usable_sources_name() {
    let (mut root, me_watch, task_watch, _) = line_fixture(
        Some(readiness(c::Readiness::Usable)),
        accounts(&[account(3, c::AccountKind::Program, None)]),
        Some(3),
        None,
        Some(b"high"),
        true,
    );
    let mut me = d::pages::accounts::current_me(root.client()).unwrap().clone();
    let offer = me.providers.get_mut(0).unwrap();
    offer.name = c::DocumentText::complete(c::Text::new(Box::from(b"Claude".as_slice()), 32).unwrap());
    offer.key = false;
    offer.sign_in = false;
    offer.programs = List::with_capacity(1);
    offer.programs.push(c::Text::new(Box::from(b"claude_code".as_slice()), 32).unwrap()).unwrap();
    me.projects.get_mut(0).unwrap().models.get_mut(0).unwrap().name =
        c::Text::new(Box::from(b"Opus".as_slice()), 32).unwrap();
    let mut task = root.pages().chat().unwrap().task(root.client()).unwrap().clone();
    task.llm.as_mut().unwrap().main.model = c::Text::new(Box::from(b"Opus".as_slice()), 32).unwrap();
    delivered(&mut root, me_watch, c::Delivery::Me(Box::new(me)));
    delivered(&mut root, task_watch.unwrap(), c::Delivery::Task(Box::new(task)));
    assert_eq!(using_text(&root), "your Claude subscription, through Claude Code · Opus · high");
    let rendered = text(&root);
    assert!(!rendered.contains("Last claimed account") && !rendered.contains("Requested for next run"));
}

#[test]
fn preference_then_kept_then_the_first_usable_account_decides_the_line() {
    let rows = accounts(&[
        account(1, c::AccountKind::Key, None),
        account(7, c::AccountKind::SignIn, None),
        account(3, c::AccountKind::Key, None),
    ]);
    for (preference, kept, expected) in [
        (Some(3), Some(7), "Supplied Provider API key · chosen"),
        (None, Some(7), "Supplied Provider plan · chosen"),
        (None, None, "Supplied Provider API key · chosen"),
    ] {
        let root = line_facts(Some(readiness(c::Readiness::Usable)), rows.clone(), preference, kept, None, true);
        assert_eq!(using_text(&root), expected);
    }
}
#[test]
fn automatic_accounts_skip_other_providers_payers_and_unusable_sources() {
    let mut other_provider = account(1, c::AccountKind::SignIn, None);
    other_provider.provider = Some(18);
    let mut other_payer = account(2, c::AccountKind::Program, None);
    other_payer.uses = Some(List::with_capacity(0));
    let mut unusable = account(3, c::AccountKind::SignIn, None);
    unusable.sources.get_mut(1).unwrap().usable = false;
    let rows = accounts(&[other_provider, other_payer, unusable, account(4, c::AccountKind::Key, None)]);
    let root = line_facts(Some(readiness(c::Readiness::Usable)), rows, None, None, None, true);
    assert_eq!(using_text(&root), "Supplied Provider API key · chosen");
}
#[test]
fn a_new_chats_default_comes_from_its_supplied_preview() {
    let mut ready = readiness(c::Readiness::Usable);
    ready.payer = c::AccountUse::Party(73);
    let (mut root, watch, _, _) =
        line_fixture(Some(ready), accounts(&[account(3, c::AccountKind::Key, None)]), None, None, None, false);
    let mut me = d::pages::accounts::current_me(root.client()).unwrap().clone();
    let project = me.projects.get_mut(0).unwrap();
    let model = project.models.get_mut(2).unwrap();
    model.efforts = List::with_capacity(2);
    model.efforts.push(model_text(b"low")).unwrap();
    model.efforts.push(model_text(b"high")).unwrap();
    model.effort = Some(model_text(b"high"));
    project.defaults = List::with_capacity(2);
    for (template, name, effort) in [
        (jig_client::shapes::CHAT + 1, b"catalogue-first".as_slice(), None),
        (jig_client::shapes::CHAT, b"second".as_slice(), Some(model_text(b"high"))),
    ] {
        project
            .defaults
            .push(c::MeDefault {
                template,
                choice: c::LlmChoice {
                    later: None,
                    account: None,
                    main: c::ModelEffort { model: model_text(name), effort },
                    subagents: List::with_capacity(0),
                },
            })
            .unwrap();
    }
    delivered(&mut root, watch, c::Delivery::Me(Box::new(me)));
    assert_eq!(using_text(&root), "second · high");
}
#[test]
fn the_three_unsettled_lines_name_only_known_facts() {
    assert_eq!(using_text(&task_root()), "chosen");
    assert_eq!(using_text(&banner(None).0), "Using the default model");
    assert_eq!(using_text(&account_readiness(Some(readiness(c::Readiness::Unavailable)))), "chosen");
}
#[test]
fn the_model_disclosure_holds_its_rows_and_folds_away() {
    let view = fixtures::page_view(&task_root());
    let nodes = chooser_nodes(&view);
    assert!(!nodes[0].states.as_slice().contains(&skein_ui::State::Expanded));
    for name in [b"chosen".as_slice(), b"second"] {
        assert!(nodes.iter().any(|n| n.role == Role::Button && n.name.as_deref() == Some(name)));
    }
    assert!(!nodes.iter().any(|n| n.name.as_deref() == Some(b"Apply to next run")));
}

#[test]
fn a_live_chat_without_message_keeps_the_line_and_chooser() {
    let (mut root, me_watch, watch, _, _) =
        fixtures::root_at_full(Address::Chat { task: 9 }, &fixtures::limits(), b"Shop");
    let template = task_root();
    let mut task = template.pages().chat().unwrap().task(template.client()).unwrap().clone();
    let mut acts = List::with_capacity(1);
    acts.push(c::Offered { act: c::ActKind::Amend, admission: c::Admission::Allowed, effect: None }).unwrap();
    task.acts = acts;
    // Reuse the model catalogue on a root with an observable task watch.
    let mut me = d::pages::accounts::current_me(root.client()).unwrap().clone();
    me.projects.get_mut(0).unwrap().models =
        d::pages::accounts::current_me(template.client()).unwrap().projects.get(0).unwrap().models.clone();
    delivered(&mut root, me_watch, c::Delivery::Me(Box::new(me)));
    delivered(&mut root, watch.unwrap(), c::Delivery::Task(Box::new(task)));
    let view = fixtures::page_view(&root);
    assert!(fixtures::find(view.tree(), Role::TextArea, b"Message").is_none());
    assert_eq!(using_text(&root), "chosen");
    assert!(fixtures::find(view.tree(), Role::Disclosure, b"Model").is_some());
    assert!(
        fixtures::browser_page_view(&root)
            .tree()
            .nodes()
            .iter()
            .any(|n| n.styles.as_slice().contains(&crate::look::BYLINE))
    );
}

#[test]
fn choice_feedback_is_in_the_menu_and_waits_stay_above() {
    let (mut root, me_watch, watch, _, _) =
        fixtures::root_at_full(Address::Chat { task: 9 }, &fixtures::limits(), b"Shop");
    let template = task_root();
    let me = d::pages::accounts::current_me(template.client()).unwrap().clone();
    let mut task = template.pages().chat().unwrap().task(template.client()).unwrap().clone();
    task.run_wait = Some(c::TaskRunWait::Credential);
    delivered(&mut root, me_watch, c::Delivery::Me(Box::new(me)));
    delivered(&mut root, watch.unwrap(), c::Delivery::Task(Box::new(task)));
    let mut out = start_model_press(&mut root);
    let pending = fixtures::page_view(&root);
    assert!(fixtures::find(pending.tree(), Role::Disclosure, b"Model").is_some());

    let nodes = pending.tree().nodes();
    let status = nodes.iter().position(|n| n.name.as_deref() == Some("Changing…".as_bytes())).unwrap();
    let wait =
        nodes.iter().position(|n| n.name.as_deref() == Some(b"This work needs a credential from its forge.")).unwrap();
    assert!(wait < status);
    answer_model_press(
        &mut root,
        &mut out,
        c::Answer::Done(c::Outcome::Document(c::DocumentOutcome::Amended { task: 9 })),
    );
    let view = fixtures::page_view(&root);
    let nodes = view.tree().nodes();
    let chooser = nodes.iter().position(|n| n.role == Role::Disclosure && n.name.as_deref() == Some(b"Model")).unwrap();
    let end = chooser + usize::try_from(nodes[chooser].size).unwrap();
    let outcome = nodes.iter().position(|n| n.role == Role::Status && n.name.as_deref() == Some(b"Applied")).unwrap();
    let wait =
        nodes.iter().position(|n| n.name.as_deref() == Some(b"This work needs a credential from its forge.")).unwrap();
    let composer = nodes.iter().position(|n| n.styles.as_slice().contains(&crate::look::SESSION_COMPOSER)).unwrap();
    assert!(chooser < outcome && outcome < end && wait < composer && composer < chooser);
}

#[test]
fn own_key_and_plan_of_openai_follow_place_order() {
    for (first, second, expected) in [
        (c::AccountKind::Key, c::AccountKind::SignIn, "OpenAI API key · chosen"),
        (c::AccountKind::SignIn, c::AccountKind::Key, "ChatGPT plan · chosen"),
    ] {
        let mut rows = accounts(&[account(9, first, None), account(3, second, None)]);
        rows.get_mut(0).unwrap().place = 1;
        rows.get_mut(1).unwrap().place = 2;
        let (mut root, me_watch, _, _) =
            line_fixture(Some(readiness(c::Readiness::Usable)), rows, None, None, None, true);
        let mut me = d::pages::accounts::current_me(root.client()).unwrap().clone();
        me.providers.get_mut(0).unwrap().name =
            c::DocumentText::complete(c::Text::new(Box::from(b"openai".as_slice()), 32).unwrap());
        delivered(&mut root, me_watch, c::Delivery::Me(Box::new(me)));
        assert_eq!(using_text(&root), expected);
    }
}
#[test]
fn another_members_chat_and_project_work_never_name_the_viewers_plan() {
    for funder in [
        c::Funder::Pool { project: fixtures::PROJECT, person: 74, period: 1 },
        c::Funder::Period { project: fixtures::PROJECT, period: 1 },
    ] {
        let (mut root, _, task_watch, accounts_watch) = line_fixture(
            Some(readiness(c::Readiness::Usable)),
            accounts(&[account(3, c::AccountKind::SignIn, None)]),
            Some(3),
            Some(3),
            None,
            true,
        );
        let mut task = root.pages().chat().unwrap().task(root.client()).unwrap().clone();
        task.budget.funder = funder;
        let mut doc = d::pages::accounts::document(root.client()).unwrap().clone();
        doc.readiness.get_mut(0).unwrap().payer = match funder {
            c::Funder::Pool { person, .. } => c::AccountUse::Party(person),
            c::Funder::Period { project, .. } => c::AccountUse::Project(project),
            c::Funder::Recurring { .. } | c::Funder::Task(_) => unreachable!("fixture funder"),
        };
        delivered(&mut root, task_watch.unwrap(), c::Delivery::Task(Box::new(task)));
        delivered(&mut root, accounts_watch, c::Delivery::Accounts(Box::new(doc)));
        let line = using_text(&root);
        assert_eq!(line, "chosen");
        assert!(!line.contains("plan") && !line.contains("API key"));
    }
}
#[test]
fn a_held_pin_stays_pinned_while_fresh_selection_passes_to_the_next_account() {
    for state in [
        c::AccountState::Later { until: None },
        c::AccountState::SignInAgain,
        c::AccountState::Misconfigured,
        c::AccountState::Unloaded { at: c::AccountPlace::File, why: c::AccountUnread::Unreadable },
        c::AccountState::Cancelled,
        c::AccountState::Closing,
    ] {
        let mut held = account(3, c::AccountKind::SignIn, None);
        held.sources.get_mut(1).unwrap().state = state;
        if let c::AccountState::Later { .. } = state {
            // An unusable Later source waits in both possible underlying states.
            held.sources.get_mut(1).unwrap().usable = false;
        } else {
            // Even contradictory usable evidence cannot override a held deciding source.
            assert!(held.sources.get(1).unwrap().usable);
        }
        let rows = accounts(&[held, account(7, c::AccountKind::Key, None)]);
        for (pin, expected) in [(Some(3), "chosen"), (None, "Supplied Provider API key · chosen")] {
            let root = line_facts(Some(readiness(c::Readiness::Usable)), rows.clone(), pin, None, None, true);
            assert_eq!(using_text(&root), expected);
        }
    }
}
#[test]
fn a_moving_source_is_held_even_if_supplied_as_usable() {
    let mut moving = account(3, c::AccountKind::SignIn, None);
    moving.sources.get_mut(1).unwrap().state = c::AccountState::Moving;
    assert!(moving.sources.get(1).unwrap().usable);
    let rows = accounts(&[moving, account(7, c::AccountKind::Key, None)]);
    for (pin, expected) in [(Some(3), "chosen"), (None, "Supplied Provider API key · chosen")] {
        let root = line_facts(Some(readiness(c::Readiness::Usable)), rows.clone(), pin, None, None, true);
        assert_eq!(using_text(&root), expected);
    }
}
#[test]
fn a_waiting_direct_source_does_not_jump_to_a_later_direct_source() {
    let mut waiting = account(3, c::AccountKind::Key, None);
    waiting.sources.get_mut(0).unwrap().kind = c::AccountKind::SignIn;
    let rows = accounts(&[waiting, account(7, c::AccountKind::SignIn, None)]);
    for (pin, expected) in [(Some(3), "chosen"), (None, "Supplied Provider plan · chosen")] {
        let root = line_facts(Some(readiness(c::Readiness::Usable)), rows.clone(), pin, None, None, true);
        assert_eq!(using_text(&root), expected);
    }
}
#[test]
fn ineligible_preference_passes_to_the_kept_account() {
    let mut wrong = account(3, c::AccountKind::Key, None);
    wrong.uses = Some(List::with_capacity(0));
    let root = line_facts(
        Some(readiness(c::Readiness::Usable)),
        accounts(&[wrong, account(7, c::AccountKind::SignIn, None)]),
        Some(3),
        Some(7),
        None,
        true,
    );
    assert_eq!(using_text(&root), "Supplied Provider plan · chosen");
}
#[test]
fn truncated_documents_and_accounts_name_only_the_model() {
    for cut_document in [true, false] {
        let (mut root, _, _, watch) = line_fixture(
            Some(readiness(c::Readiness::Usable)),
            accounts(&[account(3, c::AccountKind::Key, None)]),
            Some(3),
            None,
            None,
            true,
        );
        let mut doc = d::pages::accounts::document(root.client()).unwrap().clone();
        if cut_document {
            doc.cut = true;
        } else {
            doc.accounts.get_mut(0).unwrap().cut = true;
        }
        delivered(&mut root, watch, c::Delivery::Accounts(Box::new(doc)));
        assert_eq!(using_text(&root), "chosen");
    }
}
#[test]
fn the_offer_cross_checks_the_translated_account_across_expiry() {
    for ready in [c::Readiness::Unknown, c::Readiness::Unavailable] {
        let root = line_facts(
            Some(readiness(ready)),
            accounts(&[account(3, c::AccountKind::Key, None)]),
            Some(3),
            None,
            None,
            true,
        );
        assert_eq!(using_text(&root), "Supplied Provider API key · chosen");
    }
}

#[test]
fn mixed_or_cut_program_offers_do_not_guess_a_models_endpoint() {
    for cut in [false, true] {
        let (mut root, me_watch, _, _) = line_fixture(
            Some(readiness(c::Readiness::Usable)),
            accounts(&[account(3, c::AccountKind::Key, None)]),
            Some(3),
            None,
            None,
            true,
        );
        let mut me = d::pages::accounts::current_me(root.client()).unwrap().clone();
        let offer = me.providers.get_mut(0).unwrap();
        offer.programs = List::with_capacity(1);
        offer.programs.push(c::Text::new(Box::from(b"claude_code".as_slice()), 32).unwrap()).unwrap();
        offer.programs_cut = cut;
        delivered(&mut root, me_watch, c::Delivery::Me(Box::new(me)));
        assert_eq!(using_text(&root), "chosen");
    }
}

#[test]
fn a_usable_later_source_names_no_account() {
    for (ready, pin) in [
        (c::Readiness::Usable, None),
        (c::Readiness::Unavailable, None),
        (c::Readiness::Usable, Some(3)),
        (c::Readiness::Unavailable, Some(3)),
    ] {
        let mut first = account(3, c::AccountKind::Key, None);
        first.sources.get_mut(1).unwrap().state = c::AccountState::Later { until: None };
        let rows = accounts(&[first, account(7, c::AccountKind::SignIn, None)]);
        let (mut root, me_watch, _, _) = line_fixture(Some(readiness(ready)), rows, pin, None, None, true);
        let mut me = d::pages::accounts::current_me(root.client()).unwrap().clone();
        me.providers.get_mut(0).unwrap().name =
            c::DocumentText::complete(c::Text::new(Box::from(b"openai".as_slice()), 32).unwrap());
        delivered(&mut root, me_watch, c::Delivery::Me(Box::new(me)));
        assert_eq!(using_text(&root), "chosen");
    }
}
#[test]
fn an_unusable_later_program_source_names_no_account_when_a_program_source_after_it_would_be_chosen() {
    for (ready, pin, after_usable) in [
        (c::Readiness::Usable, None, true),
        (c::Readiness::Unavailable, None, true),
        (c::Readiness::Usable, Some(3), true),
        (c::Readiness::Unavailable, Some(3), true),
        (c::Readiness::Usable, None, false),
    ] {
        // jig's Later goes on to the account's next program source; its Limited waits.
        let mut first = account(3, c::AccountKind::Program, None);
        first.sources.get_mut(0).unwrap().state = c::AccountState::Later { until: None };
        first.sources.get_mut(1).unwrap().usable = after_usable;
        let rows = accounts(&[first, account(7, c::AccountKind::Program, None)]);
        let (mut root, me_watch, _, _) = line_fixture(Some(readiness(ready)), rows, pin, None, None, true);
        let mut me = d::pages::accounts::current_me(root.client()).unwrap().clone();
        let offer = me.providers.get_mut(0).unwrap();
        offer.name = c::DocumentText::complete(c::Text::new(Box::from(b"Claude".as_slice()), 32).unwrap());
        offer.key = false;
        offer.sign_in = false;
        offer.programs = List::with_capacity(1);
        offer.programs.push(c::Text::new(Box::from(b"claude_code".as_slice()), 32).unwrap()).unwrap();
        delivered(&mut root, me_watch, c::Delivery::Me(Box::new(me)));
        let expected = match (after_usable, ready) {
            (true, _) => "chosen",
            // Both of jig's states wait on this account, so the next one is chosen.
            (false, _) => "your Claude subscription, through Claude Code · chosen",
        };
        assert_eq!(using_text(&root), expected, "ready {ready:?}, pin {pin:?}");
    }
}

#[test]
fn ready_names_the_usable_account_and_not_the_model() {
    let usable = Some(c::PayerReadiness {
        provider: 17,
        payer: c::AccountUse::Party(73),
        readiness: c::Readiness::Usable,
        settled: Some(true),
    });
    // The model and its effort are chosen elsewhere: Ready names the
    // account, by its kind and who it is.
    let mut plan = account(3, c::AccountKind::SignIn, None);
    plan.provider_name = Some(c::DocumentText::complete(c::Text::new(Box::from(b"openai".as_slice()), 32).unwrap()));
    let mut key = plan.clone();
    key.plan = false;
    key.sources.get_mut(1).unwrap().kind = c::AccountKind::Key;
    let mut program = plan.clone();
    program.plan = false;
    program.provider_name =
        Some(c::DocumentText::complete(c::Text::new(Box::from(b"anthropic".as_slice()), 32).unwrap()));
    program.sources.get_mut(0).unwrap().usable = true;
    for (row, expected) in [
        (plan, "ChatGPT plan · Personal"),
        (key, "OpenAI API key · billed to the key’s organization"),
        (program, "Claude subscription · Claude Code, signed in on this machine"),
    ] {
        let root = line_facts(usable, accounts(&[row]), None, None, None, false);
        let short = crate::pages::accounts::first_usable_short(&root, &fixtures::words()).expect("a usable account");
        assert_eq!(core::str::from_utf8(&short).unwrap(), expected);
    }
}

/// The playground and real providers share a supplied catalogue.
pub(super) fn playground_offers_root(address: Address, playground: bool) -> (d::Domain, Token) {
    use temper_client_protocol_wire::v1 as w;
    let (mut root, me_watch, _, _, accounts, forges) = fixtures::root_at_with_forges(
        address,
        &fixtures::domain_limits(),
        b"Shop",
        playground.then_some((fixtures::PROJECT, true, w::PlaygroundState::Running)),
        None,
        Some(super::connect_forge::rows(w::Connection::Connecting, w::Offer::Offered)),
    );
    let mut me = d::pages::accounts::current_me(root.client()).unwrap().clone();
    me.providers = List::with_capacity(2);
    for (provider, name, sign_in) in [(17, b"scripted".as_slice(), false), (18, b"openai".as_slice(), true)] {
        me.providers
            .push(c::ProviderOffer {
                provider,
                name: super::connect_model::text(name),
                key_bytes: 4096,
                sign_in,
                signed_in_plan: sign_in,
                key: true,
                programs: List::with_capacity(0),
                programs_cut: false,
                none: false,
                manage_usage: None,
            })
            .unwrap();
    }
    delivered(&mut root, me_watch, c::Delivery::Me(Box::new(me)));
    delivered(
        &mut root,
        accounts.unwrap(),
        c::Delivery::Accounts(Box::new(c::Accounts {
            program_hosts: List::with_capacity(0),
            accounts: List::with_capacity(0),
            signing: List::with_capacity(0),
            last_end: None,
            usable: false,
            settled: true,
            cut: false,
            readiness: List::with_capacity(0),
            readiness_cut: false,
        })),
    );
    (root, forges.unwrap())
}

#[test]
fn the_model_choice_leaves_the_scripted_provider_out() {
    for address in [Address::ConnectForge { forge: None }, Address::Forges] {
        let (mut root, _) = playground_offers_root(address.clone(), true);
        fixtures::root_event(
            &mut root,
            d::Event::Act { action: d::Action::Pages(d::pages::Act::Project { project: fixtures::SECOND_PROJECT }) },
        );
        let check = |root: &d::Domain| {
            let mut view = fixtures::page_view(root);
            let tree = view.tree();
            assert!(fixtures::find(tree, Role::Paragraph, b"Agents need an LLM account to run.").is_some());
            assert!(fixtures::find(tree, Role::Link, b"Use a scripted API key").is_none());
            let sign_in = fixtures::find(tree, Role::Link, b"Continue with ChatGPT").unwrap();
            let key = fixtures::find(tree, Role::Link, b"Use an API key").unwrap();
            for (node, kind) in [(sign_in, d::WayKind::SignIn), (key, d::WayKind::Key)] {
                assert_eq!(
                    crate::act(&mut view, node, skein_ui::Act::Press),
                    Some(d::Action::Pages(d::pages::Act::Go {
                        address: Address::ConnectModel { way: Some(d::ModelWay { provider: 18, source: None, kind }) }
                    }))
                );
            }
        };
        check(&root);
        // A provider named scripted in a configuration without a playground is a legitimate key offer.
        let (root, _) = playground_offers_root(address, false);
        assert!(fixtures::find(fixtures::page_view(&root).tree(), Role::Link, b"Use a scripted API key").is_some());
    }
}

/// A complete unavailable slice, shared by both composers' event-path tests.
pub(super) fn no_model_root(task_page: bool) -> d::Domain {
    let (mut root, _, task_watch, watch) = line_fixture(None, List::with_capacity(0), None, None, None, task_page);
    if let Some(task_watch) = task_watch {
        let mut task = root.pages().chat().unwrap().task(root.client()).unwrap().clone();
        task.run_wait = Some(c::TaskRunWait::Credential);
        delivered(&mut root, task_watch, c::Delivery::Task(Box::new(task)));
    }
    delivered(
        &mut root,
        watch,
        c::Delivery::Accounts(Box::new(c::Accounts {
            program_hosts: List::with_capacity(0),
            accounts: List::with_capacity(0),
            signing: List::with_capacity(0),
            last_end: None,
            usable: false,
            settled: true,
            cut: false,
            readiness: List::with_capacity(0),
            readiness_cut: false,
        })),
    );
    root
}
#[test]
fn configured_provider_words_name_the_whole_byline() {
    for (provider, kind, address, expected) in [
        (b"openai".as_slice(), c::AccountKind::Key, None, "OpenAI API key · chosen"),
        (b"openai", c::AccountKind::SignIn, None, "ChatGPT plan · chosen"),
        (
            b"openai",
            c::AccountKind::SignIn,
            Some(b"https://provider.example/usage".as_slice()),
            "ChatGPT plan · chosen",
        ),
        (b"anthropic", c::AccountKind::Program, None, "your Claude subscription, through Claude Code · chosen"),
        (b"local", c::AccountKind::Key, None, "local API key · chosen"),
    ] {
        let mut row = account(3, kind, address);
        row.sources.clear();
        let mut first = source(kind, true);
        first.name = Some(Box::from(b"a-login-not-a-program-name".as_slice()));
        row.sources.push(first).unwrap();
        let (mut root, me_watch, _, _) =
            line_fixture(Some(readiness(c::Readiness::Usable)), accounts(&[row]), Some(3), None, None, true);
        let mut me = d::pages::accounts::current_me(root.client()).unwrap().clone();
        let offer = me.providers.get_mut(0).unwrap();
        offer.name = c::DocumentText::complete(c::Text::new(Box::from(provider), 32).unwrap());
        if kind == c::AccountKind::Program {
            offer.key = false;
            offer.sign_in = false;
            offer.programs = List::with_capacity(1);
            offer.programs.push(c::Text::new(Box::from(b"claude_code".as_slice()), 32).unwrap()).unwrap();
        }
        delivered(&mut root, me_watch, c::Delivery::Me(Box::new(me)));
        assert_eq!(using_text(&root), expected);
    }
}

#[test]
fn chooser_groups_use_the_providers_words() {
    for (configured, shown) in [(b"openai".as_slice(), b"OpenAI".as_slice()), (b"anthropic", b"Claude")] {
        let (mut root, me_watch, _, _) = line_fixture(
            Some(readiness(c::Readiness::Usable)),
            accounts(&[account(3, c::AccountKind::Key, None)]),
            Some(3),
            None,
            None,
            true,
        );
        let mut me = d::pages::accounts::current_me(root.client()).unwrap().clone();
        me.providers.get_mut(0).unwrap().name =
            c::DocumentText::complete(c::Text::new(Box::from(configured), 32).unwrap());
        delivered(&mut root, me_watch, c::Delivery::Me(Box::new(me)));
        assert!(
            String::from_utf8_lossy(&fixtures::text(fixtures::page_view(&root).tree()))
                .contains(core::str::from_utf8(shown).unwrap())
        );
    }
}

#[test]
fn the_no_model_line_keeps_the_tasks_wait_feedback() {
    assert!(text(&no_model_root(true)).contains("This work needs a credential from its forge."));
}

#[test]
fn the_no_model_composer_offers_connect_a_model_as_a_terminal_command() {
    for task_page in [false, true] {
        let mut view = fixtures::page_view(&no_model_root(task_page));
        let control = fixtures::find(view.tree(), Role::Link, b"Connect a model").unwrap();
        let (node, first) = fixtures::terminal_command(
            view.tree(),
            &view.limits().tree,
            control,
            b"Connect a model",
            "No model yet · ".as_bytes(),
        );
        assert_eq!(
            crate::act(&mut view, node, first),
            Some(d::Action::Pages(d::pages::Act::Go { address: Address::ConnectModel { way: None } }))
        );
    }
}

#[test]
fn a_choice_that_cant_change_is_words_not_a_button() {
    let (mut root, _, task_watch, _) = line_fixture(None, List::with_capacity(0), Some(3), None, None, true);
    let mut task = root.pages().chat().unwrap().task(root.client()).unwrap().clone();
    task.acts.clear();
    delivered(&mut root, task_watch.unwrap(), c::Delivery::Task(Box::new(task)));
    let view = fixtures::page_view(&root);
    let node = view.tree().nodes().iter().find(|n| n.styles.as_slice().contains(&crate::look::MODEL_FIXED)).unwrap();
    assert_eq!(node.role, Role::Paragraph);
    assert_eq!(node.name.as_deref(), Some(b"chosen".as_slice()));
    assert!(fixtures::find(view.tree(), Role::Disclosure, b"Model").is_none());
}

#[test]
fn manage_usage_stays_with_the_agent_composers_line() {
    let root = line_facts(
        Some(readiness(c::Readiness::Usable)),
        accounts(&[account(3, c::AccountKind::SignIn, Some(b"https://provider.example/usage"))]),
        Some(3),
        None,
        None,
        true,
    );
    let view = fixtures::page_view(&root);
    assert!(!chooser_nodes(&view).iter().any(|node| node.name.as_deref() == Some(b"Manage usage")));
    let mut b = Builder::new(&fixtures::tree_limits());
    crate::choice::using(&mut b, &root, root.pages().chat().unwrap().task(root.client()), &fixtures::words());
    assert!(fixtures::find(&b.finish(), Role::Link, b"Manage usage").is_some());
}

#[test]
fn the_agent_composers_using_words_stay_whole() {
    let root = line_facts(
        Some(readiness(c::Readiness::Usable)),
        accounts(&[account(3, c::AccountKind::SignIn, Some(b"https://provider.example/usage"))]),
        Some(3),
        None,
        None,
        true,
    );
    let mut b = Builder::new(&fixtures::tree_limits());
    crate::choice::using(&mut b, &root, root.pages().chat().unwrap().task(root.client()), &fixtures::words());
    assert_eq!(
        core::str::from_utf8(&fixtures::text(&b.finish())).unwrap(),
        "Using Supplied Provider plan · chosen · Manage usage"
    );
}

#[test]
fn a_tasks_byline_keeps_the_whole_bounded_model_and_effort() {
    let (mut root, _, task_watch, _) = line_fixture(None, List::with_capacity(0), None, None, None, true);
    let mut task = root.pages().chat().unwrap().task(root.client()).unwrap().clone();
    let name = vec![b'm'; shown_bytes()];
    let effort = vec![b'e'; usize::try_from(EFFORT_BYTES).unwrap()];
    let choice = task.llm.as_mut().unwrap();
    choice.main.model = c::Text::new(name.clone().into_boxed_slice(), NAME_BYTES).unwrap();
    choice.main.effort = Some(c::Text::new(effort.clone().into_boxed_slice(), EFFORT_BYTES).unwrap());
    delivered(&mut root, task_watch.unwrap(), c::Delivery::Task(Box::new(task)));
    let view = catalogue_view(&root);
    let byline = view.tree().nodes().iter().find(|n| n.styles.as_slice().contains(&crate::look::MODEL)).unwrap();
    assert_eq!(byline.description.as_deref(), Some([&name[..], " · ".as_bytes(), &effort].concat().as_slice()));
}

#[test]
fn a_host_wait_names_the_account_and_its_program_command() {
    let mut row = account(3, c::AccountKind::Program, None);
    row.sources.clear();
    let mut program = source(c::AccountKind::Program, false);
    program.name = Some(Box::from(b"claude-alice".as_slice()));
    row.sources.push(program).unwrap();
    let root = account_facts(None, accounts(&[row]));
    let mut b = Builder::new(&fixtures::tree_limits());
    b.open(Role::Page);
    crate::choice::waiting(
        &mut b,
        &root,
        &waiting_task(Some(c::TaskRunWait::Account { account: Some(3), source: Some(1), why: c::TaskWaitWhy::Host })),
        &fixtures::words(),
        &mut crate::frame::Bindings::new(32),
        &mut crate::words::Times::new(fixtures::NOW),
    );
    b.close();
    let tree = b.finish();
    assert!(
        fixtures::find(&tree, Role::Paragraph, b"Personal's login isn't signed in on a host that can run this.")
            .is_some()
    );
    assert!(
        String::from_utf8(fixtures::text(&tree).into_vec())
            .unwrap()
            .contains("temper program-login --config CONFIG --data DATA --cache CACHE claude-alice")
    );
    assert!(fixtures::find(&tree, Role::Link, b"Accounts").is_some());
}

fn model_text(name: &[u8]) -> c::Text {
    c::Text::new(Box::from(name), NAME_BYTES).unwrap()
}
fn provider_offer(provider: u32, name: &[u8]) -> c::ProviderOffer {
    c::ProviderOffer {
        provider,
        name: c::DocumentText::complete(model_text(name)),
        key_bytes: 4096,
        sign_in: true,
        signed_in_plan: true,
        key: true,
        programs: List::with_capacity(0),
        programs_cut: false,
        none: false,
        manage_usage: None,
    }
}
fn offer_row(name: &[u8], provider: Option<u32>) -> c::MeModel {
    c::MeModel {
        new_until: None,
        number: 1,
        source: jig_client::contract::ModelSource::Stock,
        estimated: false,
        name: model_text(name),
        provider,
        window: 1024,
        output: 128,
        efforts: List::with_capacity(0),
        effort: None,
        shown: None,
        accounts: List::with_capacity(0),
        picker: true,
        description: None,
        prices: c::ModelPrices { input: 1, cached: 1, cache_write: None, output: 1, unit: 1000 },
    }
}
fn chooser_nodes(view: &crate::View) -> &[skein_ui::Node] {
    let nodes = view.tree().nodes();
    let at =
        nodes.iter().position(|node| node.role == Role::Disclosure && node.name.as_deref() == Some(b"Model")).unwrap();
    let end = at.checked_add(usize::try_from(nodes[at].size).unwrap()).unwrap();
    &nodes[at..end]
}

#[test]
fn the_panel_groups_offered_picker_rows_and_orders_efforts() {
    let (mut root, watch, _, _) = line_fixture(None, List::with_capacity(0), None, None, None, false);
    let mut me = d::pages::accounts::current_me(root.client()).unwrap().clone();
    me.providers = List::with_capacity(2);
    me.providers.push(provider_offer(18, b"Second provider")).unwrap();
    me.providers.push(provider_offer(17, b"First provider")).unwrap();
    let mut models = List::with_capacity(5);
    let mut displayed = offer_row(b"configured-name", Some(17));
    displayed.shown = Some(model_text(b"Shown name"));
    models.push(displayed).unwrap();
    let mut selected = offer_row(b"selected", Some(18));
    selected.efforts = List::with_capacity(2);
    selected.efforts.push(model_text(b"low")).unwrap();
    selected.efforts.push(model_text(b"high")).unwrap();
    selected.effort = Some(model_text(b"high"));
    models.push(selected).unwrap();
    let mut hidden = offer_row(b"hidden", Some(17));
    hidden.picker = false;
    models.push(hidden).unwrap();
    models.push(offer_row(b"later", Some(18))).unwrap();
    models.push(offer_row(b"unassigned", None)).unwrap();
    me.projects.get_mut(0).unwrap().models = models;
    delivered(&mut root, watch, c::Delivery::Me(Box::new(me)));
    edit(&mut root, d::pages::choice::Act::Model { index: 1, account: None });
    let mut view = fixtures::page_view(&root);
    let rows: Vec<_> = chooser_nodes(&view)
        .iter()
        .filter(|node| node.role == Role::Button)
        .map(|node| node.name.as_deref().unwrap())
        .collect();
    assert_eq!(rows, [b"Shown name".as_slice(), b"selected", b"later", b"unassigned", b"low", b"high", b"All models"]);
    let shown = fixtures::find(view.tree(), Role::Button, b"Shown name").unwrap();
    assert_eq!(
        crate::act(&mut view, shown, skein_ui::Act::Press),
        Some(d::Action::Pages(d::pages::Act::Choice(d::pages::choice::Act::Model { index: 0, account: None })))
    );
}

fn empty_provider(account: Option<c::AccountLine>) -> d::Domain {
    let rows = match account {
        Some(account) => accounts(&[account]),
        None => List::with_capacity(0),
    };
    let (mut root, watch, _, _) = line_fixture(None, rows, None, None, None, true);
    let mut me = d::pages::accounts::current_me(root.client()).unwrap().clone();
    me.providers = List::with_capacity(2);
    me.providers.push(provider_offer(17, b"First provider")).unwrap();
    me.providers.push(provider_offer(18, b"OpenAI")).unwrap();
    delivered(&mut root, watch, c::Delivery::Me(Box::new(me)));
    root
}
#[test]
fn a_provider_with_no_account_is_one_line_and_no_model() {
    let mut view = fixtures::page_view(&empty_provider(None));
    let nodes = chooser_nodes(&view);
    let at = nodes
        .iter()
        .position(|node| {
            node.styles.as_slice().contains(&crate::look::MODEL_FROM)
                && nodes
                    .iter()
                    .skip_while(|n| n.id != node.id)
                    .take(usize::try_from(node.size).unwrap())
                    .any(|n| n.text.iter().any(|run| run.text.as_ref() == b"OpenAI"))
        })
        .unwrap();
    assert!(
        nodes[at..at + usize::try_from(nodes[at].size).unwrap()]
            .iter()
            .any(|n| n.name.as_deref() == Some("Connect a plan or key ›".as_bytes()))
    );
    assert!(!nodes.iter().any(|node| node.role == Role::Button && node.name.as_deref() == Some(b"gpt-6-luna")));
    let connect = fixtures::find(view.tree(), Role::Link, "Connect a plan or key ›".as_bytes()).unwrap();
    assert_eq!(
        crate::act(&mut view, connect, skein_ui::Act::Press),
        Some(d::Action::Pages(d::pages::Act::Go { address: Address::ConnectModel { way: None } }))
    );
}
#[test]
fn an_account_stopped_being_usable_keeps_its_fix_and_no_model() {
    for (kind, state, label) in [
        (c::AccountKind::SignIn, c::AccountState::SignInAgain, "Sign in again ›"),
        (c::AccountKind::Program, c::AccountState::SignInAgain, "Check again ›"),
        (c::AccountKind::Key, c::AccountState::SignInAgain, "Replace ›"),
        (c::AccountKind::Key, c::AccountState::Misconfigured, "Replace ›"),
        (c::AccountKind::SignIn, c::AccountState::Later { until: None }, "Resume ›"),
    ] {
        let mut row = account(7, kind, None);
        row.provider = Some(18);
        row.sources.clear();
        let mut source = source(kind, false);
        source.state = state;
        row.sources.push(source).unwrap();
        let mut view = fixtures::page_view(&empty_provider(Some(row)));
        let fix = fixtures::find(view.tree(), Role::Link, label.as_bytes()).unwrap();
        assert!(fixtures::find(view.tree(), Role::Link, "Connect a plan or key ›".as_bytes()).is_none());
        assert_eq!(
            crate::act(&mut view, fix, skein_ui::Act::Press),
            Some(d::Action::Pages(d::pages::Act::Go { address: Address::Accounts }))
        );
    }
}
#[test]
fn an_empty_provider_without_a_fix_keeps_readiness_unknown() {
    let mut row = account(7, c::AccountKind::SignIn, None);
    row.provider = Some(18);
    row.sources.clear();
    let view = fixtures::page_view(&empty_provider(Some(row)));
    assert!(fixtures::find(view.tree(), Role::Paragraph, b"Account readiness unknown").is_some());
    assert!(fixtures::find(view.tree(), Role::Link, "Connect a plan or key ›".as_bytes()).is_none());
}
#[test]
fn the_playground_provider_disappears_without_offered_rows() {
    let (mut root, watch, _, _, _) = fixtures::root_at_with_playground(
        Address::NewChat { draft: false },
        &fixtures::domain_limits(),
        b"Shop",
        Some((fixtures::PROJECT, true)),
    );
    let sample = fixtures::playground_root(Address::NewChat { draft: false }, fixtures::PROJECT, true);
    let mut me = d::pages::accounts::current_me(sample.client()).unwrap().clone();
    me.projects.get_mut(0).unwrap().models.clear();
    delivered(&mut root, watch, c::Delivery::Me(Box::new(me)));
    let view = fixtures::page_view(&root);
    assert!(!chooser_nodes(&view).iter().any(|node| node.name.as_deref() == Some(b"Playground")));
}
#[test]
fn the_inactive_playground_has_no_connect_line() {
    let (mut root, watch, _, _) = line_fixture(None, List::with_capacity(0), None, None, None, false);
    let mut me = d::pages::accounts::current_me(root.client()).unwrap().clone();
    me.providers = List::from_boxed(Box::new([provider_offer(17, b"First provider"), provider_offer(3, b"demo")]));
    delivered(&mut root, watch, c::Delivery::Me(Box::new(me)));
    let view = fixtures::page_view(&root);
    assert!(chooser_nodes(&view).iter().all(|node| { node.text.iter().all(|run| run.text.as_ref() != b"demo") }));
}
#[test]
fn a_choice_outside_the_offer_names_its_missing_account() {
    let (mut root, watch, _, _) =
        line_fixture(Some(readiness(c::Readiness::Usable)), List::with_capacity(0), None, None, None, true);
    let mut me = d::pages::accounts::current_me(root.client()).unwrap().clone();
    me.projects.get_mut(0).unwrap().models.clear();
    delivered(&mut root, watch, c::Delivery::Me(Box::new(me)));
    assert_eq!(using_text(&root), "chosen");
}
#[test]
fn a_draft_outside_the_offer_names_its_missing_account() {
    let (mut root, watch, _, _) = line_fixture(None, List::with_capacity(0), None, None, None, false);
    edit(&mut root, d::pages::choice::Act::Model { index: 2, account: None });
    let mut me = d::pages::accounts::current_me(root.client()).unwrap().clone();
    me.projects.get_mut(0).unwrap().models.clear();
    delivered(&mut root, watch, c::Delivery::Me(Box::new(me)));
    assert_eq!(using_text(&root), "second");
}
#[test]
fn the_playgrounds_offered_model_ignores_document_readiness() {
    let root = fixtures::hidden_playground_root(Some(c::Readiness::Unavailable), 73);
    assert_eq!(using_text(&root), "Scripted model · Default");
}
#[test]
fn a_missing_preview_never_uses_the_catalogues_first_model() {
    let root = line_facts(Some(readiness(c::Readiness::Usable)), List::with_capacity(0), None, None, None, false);
    assert_eq!(using_text(&root), "Using the default model");
}
#[test]
fn an_accounts_models_are_the_rows_that_name_its_number() {
    let (mut root, watch, _, _) =
        line_fixture(None, accounts(&[account(3, c::AccountKind::Key, None)]), None, None, None, true);
    let mut me = d::pages::accounts::current_me(root.client()).unwrap().clone();
    let project = me.projects.get_mut(0).unwrap();
    project.models = List::with_capacity(2);
    let mut same_provider = offer_row(b"wrong-account", Some(17));
    same_provider.accounts = List::with_capacity(1);
    same_provider.accounts.push(7).unwrap();
    project.models.push(same_provider).unwrap();
    let mut different_provider = offer_row(b"right-account", None);
    different_provider.accounts = List::with_capacity(1);
    different_provider.accounts.push(3).unwrap();
    project.models.push(different_provider).unwrap();
    delivered(&mut root, watch, c::Delivery::Me(Box::new(me)));
    fixtures::root_event(&mut root, d::Event::Went { place: d::Place::Address(Address::Accounts) });
    let view = fixtures::page_view(&root);
    let nodes = view.tree().nodes();
    let row = nodes.iter().position(|node| node.styles.as_slice().contains(&crate::look::SETTINGS_ROW)).unwrap();
    let summary = core::str::from_utf8(nodes[row].description.as_deref().unwrap()).unwrap();
    assert!(summary.contains("right-account"), "{summary}");
    assert!(!summary.contains("wrong-account"), "{summary}");
}

#[test]
fn an_empty_wait_names_only_a_choice_the_task_holds() {
    for (choice, expected) in [
        (None, "No account can run this chat yet."),
        (
            Some(c::LlmChoice {
                later: None,
                account: None,
                main: c::ModelEffort { model: model_text(b"explicit"), effort: None },
                subagents: List::with_capacity(0),
            }),
            "No account can run explicit yet.",
        ),
    ] {
        let root = task_root();
        let mut task =
            waiting_task(Some(c::TaskRunWait::Account { account: None, source: None, why: c::TaskWaitWhy::Empty }));
        task.llm = choice;
        let mut b = Builder::new(&fixtures::tree_limits());
        let mut bindings = crate::frame::Bindings::new(32);
        crate::choice::waiting(
            &mut b,
            &root,
            &task,
            &fixtures::words(),
            &mut bindings,
            &mut crate::words::Times::new(fixtures::NOW),
        );
        let tree = b.finish();
        assert!(fixtures::find(&tree, Role::Paragraph, expected.as_bytes()).is_some());
        assert!(!String::from_utf8_lossy(&fixtures::text(&tree)).contains("chosen"));
        assert!(fixtures::find(&tree, Role::Link, b"Connect a model").is_some());
    }
}

#[test]
fn an_offered_display_name_is_used_on_the_line() {
    let (mut root, watch, _, _) = line_fixture(None, List::with_capacity(0), None, None, None, true);
    let mut me = d::pages::accounts::current_me(root.client()).unwrap().clone();
    me.projects.get_mut(0).unwrap().models.get_mut(0).unwrap().shown = Some(model_text(b"Offered name"));
    delivered(&mut root, watch, c::Delivery::Me(Box::new(me)));
    assert_eq!(using_text(&root), "Offered name");
}

pub(super) fn picker_fixture(two: bool, openai: bool, hidden: bool) -> (d::Domain, Token, Token) {
    let (mut root, watch, _, accounts_watch) = line_fixture(None, List::with_capacity(0), None, None, None, false);
    let mut me = d::pages::accounts::current_me(root.client()).unwrap().clone();
    me.providers = List::from_boxed(Box::new([provider_offer(17, b"anthropic"), provider_offer(18, b"openai")]));
    let mut models = List::with_capacity(3);
    for (name, shown, provider, offered, picker) in [
        (b"claude-opus-5-5".as_slice(), b"Opus 5.5".as_slice(), 17, true, true),
        (b"claude-sonnet-5-5", b"Sonnet 5.5", 17, hidden, false),
        (b"gpt-6.1-sol", b"GPT-6.1 Sol", 18, openai, true),
    ] {
        if offered {
            let mut row = offer_row(name, Some(provider));
            row.shown = Some(model_text(shown));
            row.accounts = if provider == 17 && two {
                List::from_boxed(Box::new([1, 3]))
            } else {
                List::from_boxed(Box::new([if provider == 17 { 1 } else { 2 }]))
            };
            row.picker = picker;
            row.efforts = List::from_boxed(Box::new([model_text(b"low"), model_text(b"high")]));
            row.effort = Some(model_text(b"high"));
            models.push(row).unwrap();
        }
    }
    let project = me.projects.get_mut(0).unwrap();
    project.models = models;
    project.defaults = List::from_boxed(Box::new([c::MeDefault {
        template: jig_client::shapes::CHAT,
        choice: c::LlmChoice {
            later: None,
            account: None,
            main: c::ModelEffort { model: model_text(b"claude-opus-5-5"), effort: Some(model_text(b"high")) },
            subagents: List::with_capacity(0),
        },
    }]));
    delivered(&mut root, watch, c::Delivery::Me(Box::new(me)));
    let mut doc = d::pages::accounts::document(root.client()).unwrap().clone();
    doc.accounts = List::with_capacity(3);
    for (number, provider, kind, display, present) in [
        (1, 17, c::AccountKind::Program, b"Personal".as_slice(), true),
        (2, 18, c::AccountKind::SignIn, b"Work".as_slice(), openai),
        (3, 17, c::AccountKind::Program, b"Other".as_slice(), two),
    ] {
        if present {
            let mut row = account(number, kind, None);
            row.provider = Some(provider);
            row.display = c::DocumentText::complete(model_text(display));
            row.sources = List::from_boxed(Box::new([source(kind, true)]));
            doc.accounts.push(row).unwrap();
        }
    }
    delivered(&mut root, accounts_watch, c::Delivery::Accounts(Box::new(doc)));
    (root, watch, accounts_watch)
}
fn picker_line(view: &crate::View) -> &[u8] {
    chooser_nodes(view)[0].description.as_deref().unwrap()
}
#[test]
fn a_new_chats_picker_says_what_create_would_use() {
    let (root, _, _) = picker_fixture(false, true, false);
    for view in [fixtures::page_view(&root), fixtures::browser_page_view(&root)] {
        assert_eq!(picker_line(&view), "Claude Opus 5.5 · high".as_bytes());
        let model = fixtures::find(view.tree(), Role::Button, b"Opus 5.5").unwrap();
        assert!(
            view.tree()
                .nodes()
                .iter()
                .find(|n| n.id == model)
                .unwrap()
                .states
                .as_slice()
                .contains(&skein_ui::State::Pressed)
        );
    }
    let me = d::pages::accounts::current_me(root.client()).unwrap();
    let default = jig_client::shapes::resolved_default(me, fixtures::PROJECT, jig_client::shapes::CHAT).unwrap();
    assert_eq!(default.main.model.bytes(), b"claude-opus-5-5");
    assert!(root.pages().choice().requested.is_none());
}
#[test]
fn a_heading_says_what_pays() {
    for (provider, kind, held, expected) in [
        (b"anthropic".as_slice(), c::AccountKind::Program, true, "Claudeyour subscription"),
        (b"anthropic", c::AccountKind::SignIn, true, "Claudeyour subscription"),
        (b"openai", c::AccountKind::SignIn, true, "OpenAIyour ChatGPT plan"),
        (b"openai", c::AccountKind::Key, true, "OpenAIyour API key"),
        (b"openai", c::AccountKind::Key, false, "OpenAI"),
    ] {
        let (mut root, watch, accounts_watch) = picker_fixture(false, false, false);
        let mut me = d::pages::accounts::current_me(root.client()).unwrap().clone();
        me.providers.get_mut(0).unwrap().name = c::DocumentText::complete(model_text(provider));
        delivered(&mut root, watch, c::Delivery::Me(Box::new(me)));
        let mut doc = d::pages::accounts::document(root.client()).unwrap().clone();
        doc.accounts.get_mut(0).unwrap().sources = List::from_boxed(Box::new([source(kind, true)]));
        if !held {
            doc.accounts.clear();
        }
        delivered(&mut root, accounts_watch, c::Delivery::Accounts(Box::new(doc)));
        let view = fixtures::page_view(&root);
        let nodes = chooser_nodes(&view);
        let at = nodes.iter().position(|n| n.styles.as_slice().contains(&crate::look::MODEL_FROM)).unwrap();
        let bytes = fixtures::nodes_text(&nodes[at..at + usize::try_from(nodes[at].size).unwrap()]);
        assert_eq!(String::from_utf8_lossy(&bytes), expected);
    }
}
#[test]
fn two_accounts_of_one_provider_head_each_and_a_press_prefers_its_account() {
    const HEADINGS: usize = 3; // Two Claude accounts and OpenAI's connect line.
    for (first_email, other_email, first, other) in [
        (None, None, "ClaudePersonal", "ClaudeOther"),
        (
            Some(b"alice@example.com".as_slice()),
            Some(b"bob@example.com".as_slice()),
            "Claudealice@example.com",
            "Claudebob@example.com",
        ),
        (Some(b"alice@example.com".as_slice()), None, "Claudealice@example.com", "ClaudeOther"),
    ] {
        let (mut root, _, accounts_watch) = picker_fixture(true, false, false);
        let mut doc = d::pages::accounts::document(root.client()).unwrap().clone();
        doc.accounts.get_mut(0).unwrap().facts = super::accounts::email_facts(first_email);
        doc.accounts.get_mut(1).unwrap().facts = super::accounts::email_facts(other_email);
        delivered(&mut root, accounts_watch, c::Delivery::Accounts(Box::new(doc)));
        let mut view = fixtures::page_view(&root);
        let nodes = chooser_nodes(&view);
        let headings: Vec<_> =
            nodes.iter().filter(|n| n.styles.as_slice().contains(&crate::look::MODEL_FROM)).collect();
        let labels = fixtures::nodes_text(nodes);
        assert!(String::from_utf8_lossy(&labels).contains(first));
        assert!(String::from_utf8_lossy(&labels).contains(other));
        assert_eq!(headings.len(), HEADINGS);
        let opus: Vec<_> =
            nodes.iter().filter(|n| n.role == Role::Button && n.name.as_deref() == Some(b"Opus 5.5")).collect();
        assert!(opus[0].states.as_slice().contains(&skein_ui::State::Pressed));
        assert!(!opus[1].states.as_slice().contains(&skein_ui::State::Pressed));
        let id = opus[1].id;
        assert_eq!(
            crate::act(&mut view, id, skein_ui::Act::Press),
            Some(d::Action::Pages(d::pages::Act::Choice(d::pages::choice::Act::Model { index: 0, account: Some(3) })))
        );
    }
}
#[test]
fn efforts_follow_the_chosen_model_and_none_has_no_row() {
    let (mut root, watch, _) = picker_fixture(false, true, false);
    let view = fixtures::page_view(&root);
    let nodes = chooser_nodes(&view);
    let last_model =
        nodes.iter().rposition(|n| n.role == Role::Button && n.name.as_deref() == Some(b"GPT-6.1 Sol")).unwrap();
    let effort = nodes.iter().position(|n| n.role == Role::Group && n.name.as_deref() == Some(b"Effort")).unwrap();
    assert!(last_model < effort);
    let mut me = d::pages::accounts::current_me(root.client()).unwrap().clone();
    me.projects.get_mut(0).unwrap().models.get_mut(0).unwrap().efforts.clear();
    delivered(&mut root, watch, c::Delivery::Me(Box::new(me)));
    assert!(fixtures::find(fixtures::page_view(&root).tree(), Role::Group, b"Effort").is_none());
}
#[test]
fn all_models_shows_where_the_picker_shows_fewer() {
    for hidden in [false, true] {
        let (root, _, _) = picker_fixture(false, true, hidden);
        let mut view = fixtures::page_view(&root);
        let all = fixtures::find(view.tree(), Role::Button, b"All models");
        assert_eq!(all.is_some(), hidden);
        if let Some(all) = all {
            let at = view.tree().nodes().iter().position(|n| n.id == all).unwrap();
            assert_eq!(fixtures::node_text(view.tree(), at).as_ref(), "All models · 3".as_bytes());
            assert_eq!(
                crate::act(&mut view, all, skein_ui::Act::Press),
                Some(d::Action::Pages(d::pages::Act::Choice(d::pages::choice::Act::All)))
            );
        }
    }
}
#[test]
fn all_models_finds_by_shown_name_and_name() {
    let (mut root, _, _) = picker_fixture(false, true, true);
    edit(&mut root, d::pages::choice::Act::All);
    for words in [b"SoNnEt 5.5".as_slice(), b"SONNET-5-5"] {
        edit(&mut root, d::pages::choice::Act::Find { words: Box::from(words) });
        let mut view = fixtures::page_view(&root);
        assert!(fixtures::find(view.tree(), Role::Button, b"Sonnet 5.5").is_some());
        assert!(fixtures::find(view.tree(), Role::Button, b"Opus 5.5").is_none());
        let field = fixtures::find(view.tree(), Role::TextField, b"Find a model").unwrap();
        assert!(
            view.tree()
                .nodes()
                .iter()
                .find(|n| n.id == field)
                .unwrap()
                .states
                .as_slice()
                .contains(&skein_ui::State::SubmitsOnEnter)
        );
        assert_eq!(
            crate::act(&mut view, field, skein_ui::Act::Input { value: Box::from(b"opus".as_slice()) }),
            Some(d::Action::Pages(d::pages::Act::Choice(d::pages::choice::Act::Find {
                words: Box::from(b"opus".as_slice())
            })))
        );
        assert_eq!(crate::act(&mut view, field, skein_ui::Act::Submit), None);
    }
}
#[test]
fn all_models_links_to_the_projects_models_and_marks_new() {
    const WEEK: u64 = 604_800_000_000_000;
    let (mut root, watch, _) = picker_fixture(false, true, true);
    edit(&mut root, d::pages::choice::Act::All);
    let mut view = fixtures::page_view(&root);
    let link = fixtures::find(view.tree(), Role::Link, b"Choose what the list shows, in Deployment").unwrap();
    assert_eq!(
        crate::act(&mut view, link, skein_ui::Act::Press),
        Some(d::Action::Pages(d::pages::Act::Go { address: Address::Models { project: 1 } }))
    );
    assert_eq!(crate::href(&mut view, link).as_deref(), Some(b"/projects/1/settings/models".as_slice()));
    assert!(!chooser_nodes(&view).iter().any(|n| n.name.as_deref() == Some(b"new")));
    assert!(fixtures::find(view.tree(), Role::Button, "‹ Models".as_bytes()).is_some());
    for source in [c::ModelSource::List, c::ModelSource::Stock, c::ModelSource::Person] {
        let mut me = d::pages::accounts::current_me(root.client()).unwrap().clone();
        let model = me.projects.get_mut(0).unwrap().models.get_mut(0).unwrap();
        model.source = source;
        model.new_until = Some(skein_lib::Wall::from_nanos(WEEK));
        delivered(&mut root, watch, c::Delivery::Me(Box::new(me)));
        for mut view in [fixtures::page_view(&root), fixtures::browser_page_view(&root)] {
            for now in [WEEK - 1, WEEK, WEEK + 1] {
                crate::build(&mut view, &root, skein_lib::Wall::from_nanos(now), &mut skein_lib::Queue::uncapped());
                let tags: Vec<_> = chooser_nodes(&view)
                    .iter()
                    .filter(|n| {
                        n.styles.as_slice().contains(&crate::look::MODEL_TAG) && {
                            let at = view.tree().nodes().iter().position(|node| node.id == n.id).unwrap();
                            let end = at + usize::try_from(n.size).unwrap();
                            fixtures::nodes_text(&view.tree().nodes()[at..end]).ends_with(b"new")
                        }
                    })
                    .collect();
                assert_eq!(tags.len(), usize::from(source == c::ModelSource::List && now < WEEK));
                if source == c::ModelSource::List && now < WEEK {
                    assert_eq!(view.next_change(), Some(skein_lib::Wall::from_nanos(WEEK)));
                }
            }
        }
    }
}
#[test]
fn a_started_chat_says_when_a_change_applies() {
    let view = fixtures::page_view(&task_root());
    assert!(chooser_nodes(&view).iter().any(|n| n.name.as_deref() == Some(b"A change applies from the next turn.")));
}
#[test]
fn no_model_of_a_provider_with_no_account_shows() {
    let (mut root, _, _) = picker_fixture(false, false, true);
    for all in [false, true] {
        if all {
            edit(&mut root, d::pages::choice::Act::All);
        }
        let view = fixtures::page_view(&root);
        assert_eq!(picker_line(&view), "Claude Opus 5.5 · high".as_bytes());
        let nodes = chooser_nodes(&view);
        assert!(!nodes.iter().any(|n| n.role == Role::Button && n.name.as_deref() == Some(b"GPT-6.1 Sol")));
        assert!(
            nodes
                .iter()
                .any(|n| n.role == Role::Link && n.name.as_deref() == Some("Connect a plan or key ›".as_bytes()))
        );
    }
}
#[test]
fn the_terminal_prints_the_list_as_lines() {
    let (root, _, _) = picker_fixture(false, true, false);
    let view = fixtures::page_view(&root);
    let mut print = skein_ui_text::Print::new(&view.limits().tree).unwrap();
    skein_ui_text::render(view.tree(), b"temper", None, &mut print);
    let printed = String::from_utf8_lossy(print.text.as_slice());
    assert!(printed.contains("Model: Claude Opus 5.5 · high"), "{printed}");
    assert!(
        printed
            .lines()
            .any(|line| line.trim_start().starts_with("- [") && line.ends_with("Opus 5.5 · default (pressed)")),
        "{printed}"
    );
    assert!(printed.lines().any(|line| line.trim() == "Effort:"), "{printed}");
    assert!(printed.find("Opus 5.5 · default").unwrap() < printed.find("Effort:").unwrap());
}

fn start_model_press(root: &mut d::Domain) -> skein_lib::Queue<d::Request> {
    let mut out = skein_lib::Queue::uncapped();
    d::step(
        root,
        &skein_lib::Env { now: skein_lib::Time::ZERO, wall: skein_lib::Wall::EPOCH, limits: fixtures::limits() },
        d::Event::Act {
            action: d::Action::Pages(d::pages::Act::Choice(d::pages::choice::Act::Model { index: 1, account: None })),
        },
        &mut out,
    );
    out
}

fn answer_model_press(root: &mut d::Domain, out: &mut skein_lib::Queue<d::Request>, answer: c::Answer) {
    d::step(
        root,
        &skein_lib::Env { now: skein_lib::Time::ZERO, wall: skein_lib::Wall::EPOCH, limits: fixtures::limits() },
        d::Event::Client { event: jig_client::Event::Stored },
        out,
    );
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
        .unwrap();
    fixtures::root_event(
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
fn a_refusal_says_why_in_the_menu() {
    let mut root = task_root();
    let mut out = start_model_press(&mut root);
    answer_model_press(&mut root, &mut out, c::Answer::Refused { refusal: c::Refusal::NotOffered, at: None });
    let view = fixtures::page_view(&root);
    assert_eq!(picker_line(&view), b"chosen");
    assert!(String::from_utf8_lossy(&fixtures::nodes_text(chooser_nodes(&view))).contains("not offered"));
}
#[test]
fn no_model_yet_has_no_picker() {
    for task in [false, true] {
        let view = fixtures::page_view(&no_model_root(task));
        assert!(!view.tree().nodes().iter().any(|n| n.styles.as_slice().contains(&crate::look::MODEL)));
        assert!(String::from_utf8_lossy(&fixtures::text(view.tree())).contains("No model yet"));
    }
}
#[test]
fn a_chats_bar_keeps_its_number_and_no_account() {
    let view = fixtures::browser_page_view(&named_account());
    let nodes = view.tree().nodes();
    let bar = nodes.iter().position(|n| n.styles.as_slice().contains(&crate::look::SESSION_TOP)).unwrap();
    let bar = &nodes[bar..bar + usize::try_from(nodes[bar].size).unwrap()];
    assert!(bar.iter().any(|n| n.role == Role::Paragraph && n.name.as_deref() == Some(b"T9")));
    assert!(!String::from_utf8_lossy(&fixtures::nodes_text(bar)).contains("API key"));
    assert!(!bar.iter().any(|n| n.role == Role::Disclosure && n.name.as_deref() == Some(b"Model")));
}

#[test]
fn the_playground_model_uses_its_words_over_a_shown_name() {
    let mut model = offer_row(crate::words::SCRIPTED_MODEL, None);
    model.shown = Some(model_text(b"Provider identifier"));
    let mut root = catalogue_rows(List::from_boxed(Box::new([model])));
    edit(&mut root, d::pages::choice::Act::Model { index: 0, account: None });
    assert_eq!(picker_line(&fixtures::page_view(&root)), b"Scripted model");
}
