use super as fixtures;
use jig_client::contract as c;
use skein_lib::{List, Token};
use temper_client_domain::{self as d, Address};

pub fn delivered(root: &mut d::Domain, watch: Token, value: c::Delivery) {
    fixtures::root_event(
        root,
        d::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched { watch, event: c::WatchEvent::Delivered(value) },
            },
        },
    );
}

#[must_use]
pub fn line_models(task_page: bool) -> List<c::MeModel> {
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

#[must_use]
pub fn provider_byline_root(
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

#[must_use]
pub fn line_fixture(
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
            lendable: false,
            lending: None,
            lent_end: None,
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

pub const NAME_BYTES: u32 = jig_client_protocol_wire::v21::CEILINGS.me_model_name;

pub fn wait_pointers(text: &[u8]) {
    let text = core::str::from_utf8(text).unwrap();
    assert!(!text.contains("temper ctl "), "page wait names ctl: {text}");
    for after in text.split("temper help ").skip(1) {
        let page = after.split(|c: char| !c.is_ascii_alphanumeric() && c != '-').next().unwrap();
        assert!(temper_docs::find(page.as_bytes()).is_some(), "missing help page {page}");
    }
}

#[must_use]
pub fn source(kind: c::AccountKind, usable: bool) -> c::AccountSource {
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

pub fn account(number: u32, kind: c::AccountKind, address: Option<&[u8]>) -> c::AccountLine {
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

/// The playground and real providers share a supplied catalogue.
#[must_use]
pub fn playground_offers_root(address: Address, playground: bool) -> (d::Domain, Token) {
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
            lendable: false,
            lending: None,
            lent_end: None,
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

/// A complete unavailable slice, shared by both composers' event-path tests.
#[must_use]
pub fn no_model_root(task_page: bool) -> d::Domain {
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
            lendable: false,
            lending: None,
            lent_end: None,
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

#[must_use]
pub fn model_text(name: &[u8]) -> c::Text {
    c::Text::new(Box::from(name), NAME_BYTES).unwrap()
}

#[must_use]
pub fn provider_offer(provider: u32, name: &[u8]) -> c::ProviderOffer {
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

#[must_use]
pub fn offer_row(name: &[u8], provider: Option<u32>) -> c::MeModel {
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

#[must_use]
pub fn picker_fixture(two: bool, openai: bool, hidden: bool) -> (d::Domain, Token, Token) {
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

#[must_use]
pub fn accounts(rows: &[c::AccountLine]) -> List<c::AccountLine> {
    let mut accounts = List::with_capacity(u32::try_from(rows.len()).unwrap());
    for row in rows {
        accounts.push(row.clone()).unwrap();
    }
    accounts
}

#[must_use]
pub fn line_facts(
    ready: Option<c::PayerReadiness>,
    accounts: List<c::AccountLine>,
    preference: Option<u32>,
    kept: Option<u32>,
    effort: Option<&[u8]>,
    task_page: bool,
) -> d::Domain {
    line_fixture(ready, accounts, preference, kept, effort, task_page).0
}
