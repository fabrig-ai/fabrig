use crate::jig::digest::Digest;
use crate::tests::fixtures as f;
use jig_client::contract as c;
use skein_lib::List;

#[test]
fn account_reads_close_authorized_watches_after_the_first_projection() {
    for owner in [false, true] {
        let mut root = f::account_root(owner);
        let call = root.submit(&["account", "list", "--json"]);
        assert!(root.reads.is_empty(), "accounts has no ReadValue");
        let mut deployment = false;
        for watched in root.watches.values() {
            deployment |= *watched == f::DEPLOYMENT_WATCH;
        }
        assert_eq!(deployment, owner);
        f::deliver_accounts(&mut root, f::accounts());
        if owner {
            assert!(root.answer(call).is_none(), "owner waits for deployment too");
            f::deliver_deployment(&mut root, f::deployment());
        }
        let printed = root.answer(call).expect("first deliveries");
        assert_eq!(printed.status, 0, "{printed:?}");
        for fact in [
            b"\"number\":43".as_slice(),
            b"\"number\":71",
            b"\"provider\":9",
            b"\"until\":100",
            b"\"project\":1",
            b"\"provider_name\":",
            b"https://provider.example/usage",
        ] {
            assert!(f::json_has(&printed, fact), "{fact:?}");
        }
        assert_eq!(root.watches.len(), 1, "only shared me remains");
        let common = f::json_without_digest(&printed.stdout);
        assert_eq!(f::last_digest(&printed.stdout).as_ref(), Digest::of(&common).hex());
    }
}
#[test]
fn account_digests_distinguish_changes_to_all_projected_account_facts() {
    let original = f::accounts();
    let read = f::account_read(&mut f::account_root(false), original.clone(), None);
    let digest = f::last_digest(&read.stdout);
    for change in 0..6u8 {
        let mut value = original.clone();
        let row = value.accounts.get_mut(0).expect("account");
        match change {
            0 => row.sources.get_mut(0).expect("source").spent_until = Some(skein_lib::Wall::from_nanos(101)),
            1 => row.sources.get_mut(0).expect("source").state = c::AccountState::Granted,
            2 => row.uses = None,
            3 => row.place = 2,
            4 => row.cut = true,
            _ => {
                value.last_end = Some(c::AccountLastEnd {
                    source: f::SOURCE,
                    end: c::AccountEnd::Failed { failure: c::AccountFailure::MissingScope { scope: 7 } },
                });
            }
        }
        let changed = f::account_read(&mut f::account_root(false), value, None);
        assert_ne!(f::last_digest(&changed.stdout), digest, "change {change}");
    }
}
#[test]
fn a_moving_source_and_a_refused_move_print_their_names() {
    let mut value = f::accounts();
    value.accounts.get_mut(0).expect("account").sources.get_mut(0).expect("source").state = c::AccountState::Moving;
    value.last_end = Some(c::AccountLastEnd {
        source: f::SOURCE,
        end: c::AccountEnd::Refused { refusal: c::AccountRefusal::NotOurs },
    });
    let printed = f::account_read(&mut f::account_root(false), value, None);
    assert_eq!(printed.status, 0, "{printed:?}");
    for fact in [b"\"kind\":\"moving\"".as_slice(), b"\"refused\":\"not_ours\""] {
        assert!(f::json_has(&printed, fact), "{fact:?}");
    }
}
#[test]
fn models_group_only_project_authorized_models_without_guessing_provider_readiness() {
    let mut root = f::account_root(false);
    let mut me = f::me(f::PARTY);
    let project = me.projects.get_mut(0).expect("project");
    project.models = List::with_capacity(3);
    for (name, provider) in [(b"first".as_slice(), Some(f::PROVIDER)), (b"other", None), (b"last", Some(f::PROVIDER))] {
        let mut efforts = List::with_capacity(1);
        efforts.push(c::Text::new(b"low".as_slice().into(), 32).expect("effort")).expect("one effort");
        project
            .models
            .push(c::MeModel {
                prices: c::ModelPrices { input: 7, cached: 2, cache_write: Some(9), output: 11, unit: 1000 },
                description: Some(c::Text::new(b"Careful reasoning".as_slice().into(), 64).expect("description")),
                name: c::Text::new(name.into(), 32).expect("model"),
                provider,
                window: 8192,
                output: 1024,
                efforts,
            })
            .expect("models");
    }
    me.providers = f::provider_offers();
    root.deliver(c::Delivery::Me(Box::new(me)));
    for project in ["absent", "999"] {
        let call = root.submit(&["model", "list", "--project", project]);
        assert_eq!(root.answer(call).expect("membership error").status, 1);
        assert_eq!(root.watches.len(), 1, "invalid projects open no watch");
    }
    for flags in [&["model", "list", "--project", "shop", "--json"][..], &["model", "list", "--json"]] {
        let call = root.submit(flags);
        f::deliver_accounts(&mut root, f::accounts());
        let printed = root.answer(call).expect("models join");
        for fact in
            [b"\"window\":8192".as_slice(), b"\"output\":1024", b"first", b"other", b"last", b"\"efforts\":[\"low\"]"]
        {
            assert!(f::json_has(&printed, fact), "{fact:?}");
        }
        for fact in [
            b"\"description\":\"Careful reasoning\"".as_slice(),
            b"\"prices\":{\"input\":7,\"cached\":2,\"cache_write\":9,\"output\":11,\"unit\":1000}",
        ] {
            assert!(f::json_has(&printed, fact), "{fact:?}");
        }
        let mut groups = 0u32;
        let end = skein_lib::bytes::find(&printed.stdout, b"\"current\":").expect("model projection ends");
        for window in printed.stdout.get(..end).expect("models").windows(b"\"provider\":9,\"provider_name\"".len()) {
            if window == b"\"provider\":9,\"provider_name\"" {
                groups += 1;
            }
        }
        assert_eq!(groups, 1, "one group per supplied provider");
        for absent in [b"needs an account".as_slice(), b"OpenAI"] {
            assert!(!f::json_has(&printed, absent));
        }
        let accounts = f::account_read(&mut root, f::accounts(), None);
        assert_ne!(
            f::last_digest(&printed.stdout),
            f::last_digest(&accounts.stdout),
            "models is its own joined digest"
        );
    }
}
#[test]
fn account_output_truncates_oversized_projections_within_the_result_window() {
    let mut value = f::accounts();
    let mut row = value.accounts.get(0).expect("row").clone();
    let mut escaped = skein_lib::Writer::new(usize::try_from(crate::limits::TEXT).expect("text fits"));
    for _ in 0..crate::limits::TEXT {
        escaped.put(b"\"").expect("escaped text fits");
    }
    row.display = f::document_text(&escaped.finish());
    value.accounts = List::with_capacity(crate::limits::ROWS + 1);
    for _ in 0..=crate::limits::ROWS {
        value.accounts.push(row.clone()).expect("one past window");
        row.display = f::document_text(b"Work");
    }
    for json in [false, true] {
        let mut root = f::account_root(false);
        let flags = if json { &["account", "list", "--json"][..] } else { &["account", "list"][..] };
        let call = root.submit(flags);
        f::deliver_accounts(&mut root, value.clone());
        let printed = root.answer(call).expect("cut result");
        assert_eq!(printed.status, 0, "{printed:?}");
        assert!(printed.stdout.len() <= usize::try_from(crate::limits::TEXT).expect("text fits"));
        assert!(f::json_has(&printed, if json { b"\"cut\":true" } else { b"cut: true" }));
        assert!(f::json_has(&printed, b"bytes_left"));
    }
}

fn me_read(me: c::Me, json: bool) -> crate::jig::Printed {
    let mut root = f::account_root(true);
    root.deliver(c::Delivery::Me(Box::new(me)));
    let flags = if json { &["me", "--json"][..] } else { &["me"][..] };
    let call = root.submit(flags);
    root.answer(call).expect("me projection")
}
#[test]
fn me_provider_name_loss_marks_cut() {
    let mut me = f::me(f::PARTY);
    me.providers = f::provider_offers();
    me.providers.get_mut(0).expect("offer").name.cut = true;
    assert!(f::json_has(&me_read(me, true), b"\"cut\":true,\"retention\":"));
}
#[test]
fn me_program_list_loss_marks_cut() {
    let mut me = f::me(f::PARTY);
    me.providers = f::provider_offers();
    me.providers.get_mut(0).expect("offer").programs_cut = true;
    assert!(f::json_has(&me_read(me, true), b"\"cut\":true,\"retention\":"));
}
#[test]
fn me_provider_list_loss_marks_cut() {
    let mut me = f::me(f::PARTY);
    me.providers = f::provider_offers();
    me.providers_cut = true;
    assert!(f::json_has(&me_read(me, true), b"\"cut\":true,\"retention\":"));
}
#[test]
fn party_readiness_loss_marks_accounts_cut() {
    let mut party = f::accounts();
    let deployment = f::deployment();
    party.readiness_cut = true;
    let printed = f::account_read(&mut f::account_root(true), party, Some(deployment));
    assert!(f::json_without_digest(&printed.stdout).ends_with(b"\"cut\":true}"));
}
#[test]
fn deployment_readiness_loss_marks_accounts_cut() {
    let party = f::accounts();
    let mut deployment = f::deployment();
    deployment.readiness_cut = true;
    let printed = f::account_read(&mut f::account_root(true), party, Some(deployment));
    assert!(f::json_without_digest(&printed.stdout).ends_with(b"\"cut\":true}"));
}
#[test]
fn deployment_provider_name_loss_marks_accounts_cut() {
    let party = f::accounts();
    let mut deployment = f::deployment();
    deployment.providers = f::provider_offers();
    deployment.providers.get_mut(0).expect("offer").name.cut = true;
    let printed = f::account_read(&mut f::account_root(true), party, Some(deployment));
    assert!(f::json_without_digest(&printed.stdout).ends_with(b"\"cut\":true}"));
}
#[test]
fn deployment_program_list_loss_marks_accounts_cut() {
    let party = f::accounts();
    let mut deployment = f::deployment();
    deployment.providers = f::provider_offers();
    deployment.providers.get_mut(0).expect("offer").programs_cut = true;
    let printed = f::account_read(&mut f::account_root(true), party, Some(deployment));
    assert!(f::json_without_digest(&printed.stdout).ends_with(b"\"cut\":true}"));
}
#[test]
fn deployment_provider_list_loss_marks_accounts_cut() {
    let party = f::accounts();
    let mut deployment = f::deployment();
    deployment.providers = f::provider_offers();
    deployment.providers_cut = true;
    let printed = f::account_read(&mut f::account_root(true), party, Some(deployment));
    assert!(f::json_without_digest(&printed.stdout).ends_with(b"\"cut\":true}"));
}
#[test]
fn me_text_exposes_configured_provider_offers() {
    let mut root = f::account_root(true);
    let mut me = f::me(f::PARTY);
    me.providers = f::provider_offers();
    root.deliver(c::Delivery::Me(Box::new(me)));
    let call = root.submit(&["me"]);
    let printed = root.answer(call).expect("offers text");
    for fact in [
        "provider 9 · Scripted · sign-in, key, programs helper · manage usage https://provider.example/usage"
            .as_bytes(),
        b"cut: false\n",
    ] {
        assert!(f::has(&printed, fact), "{fact:?}");
    }
}
#[test]
fn deployment_offers_share_the_accounts_result_window() {
    const OFFERS: u32 = 64;
    let mut deployment = f::deployment();
    let offer = f::provider_offers().get(0).expect("offer").clone();
    deployment.providers = List::with_capacity(OFFERS);
    deployment.readiness = List::with_capacity(OFFERS);
    for _ in 0..OFFERS {
        deployment.providers.push(offer.clone()).expect("offer room");
        deployment.readiness.push(*f::payer_readiness().get(0).expect("readiness")).expect("readiness room");
    }
    for flags in [&["account", "list"][..], &["account", "list", "--json"]] {
        let mut root = f::account_root(true);
        let call = root.submit(flags);
        f::deliver_accounts(&mut root, f::accounts());
        f::deliver_deployment(&mut root, deployment.clone());
        let printed = root.answer(call).expect("bounded offers");
        assert_eq!(printed.status, 0);
        assert!(printed.stdout.len() <= usize::try_from(crate::limits::TEXT).expect("text fits"));
        assert!(f::json_has(&printed, if flags.len() > 2 { b"\"cut\":true" } else { b"cut: true" }));
    }
}

#[test]
fn account_projection_includes_configured_worker_login_facts() {
    let mut accounts = f::accounts();
    let mut programs = List::with_capacity(1);
    programs
        .push(c::ProgramLogin {
            name: b"work-login".as_slice().into(),
            state: Some(c::ProgramState::SignedInOtherwise { method: b"other-method".as_slice().into() }),
            sign_out: None,
            cleanup: false,
            cut: false,
        })
        .unwrap();
    accounts.program_hosts = List::with_capacity(1);
    accounts
        .program_hosts
        .push(c::ProgramHost {
            name: b"configured-worker".as_slice().into(),
            channel: 17,
            connected: true,
            awaiting_logins: false,
            programs,
            cut: false,
        })
        .unwrap();
    let mut root = f::account_root(false);
    let call = root.submit(&["account", "list", "--json"]);
    f::deliver_accounts(&mut root, accounts);
    let printed = root.answer(call).expect("account projection");
    for fact in [b"configured-worker".as_slice(), b"work-login", b"other-method"] {
        assert!(skein_lib::bytes::find(&printed.stdout, fact).is_some(), "{printed:?}");
    }
}

#[test]
fn model_cache_writes_default_to_the_supplied_input_price() {
    let mut root = f::account_root(false);
    let mut me = f::me(f::PARTY);
    let project = me.projects.get_mut(0).expect("project");
    project.models = List::with_capacity(1);
    project
        .models
        .push(c::MeModel {
            name: c::Text::new(b"reasoner".as_slice().into(), 32).expect("model"),
            provider: Some(f::PROVIDER),
            window: 8192,
            output: 1024,
            efforts: List::with_capacity(0),
            description: None,
            prices: c::ModelPrices { input: 17, cached: 3, cache_write: None, output: 29, unit: 100 },
        })
        .expect("model room");
    root.deliver(c::Delivery::Me(Box::new(me)));
    let call = root.submit(&["model", "list", "--json"]);
    f::deliver_accounts(&mut root, f::accounts());
    let printed = root.answer(call).expect("model prices");
    assert!(f::json_has(&printed, b"\"description\":null"));
    assert!(f::json_has(
        &printed,
        b"\"prices\":{\"input\":17,\"cached\":3,\"cache_write\":17,\"output\":29,\"unit\":100}"
    ));
}

#[test]
fn accounts_show_supplied_deployment_activity_and_no_party_activity() {
    for json in [false, true] {
        let mut root = f::account_root(true);
        let mut deployment = f::deployment();
        let row = deployment.accounts.get_mut(0).expect("account");
        row.runs = List::with_capacity(1);
        row.waiting = List::with_capacity(1);
        let mut chip = f::task_line().chip;
        chip.task = 211;
        row.runs.push(chip.clone()).expect("running task");
        chip.task = 223;
        row.waiting.push(chip).expect("waiting task");
        let call = root.submit(if json { &["account", "list", "--json"][..] } else { &["account", "list"][..] });
        f::deliver_accounts(&mut root, f::accounts());
        f::deliver_deployment(&mut root, deployment);
        let printed = root.answer(call).expect("activity");
        let expected: &[&[u8]] = if json {
            &[b"\"runs\":null,\"waiting\":null", b"\"runs\":[211],\"waiting\":[223]"]
        } else {
            &[b"runs: unavailable", b"waiting: unavailable", b"runs: 211", b"waiting: 223", b"Scripted"]
        };
        for fact in expected {
            assert!(f::has(&printed, fact), "{fact:?}: {printed:?}");
        }
    }
}
#[test]
fn model_text_prints_supplied_descriptions_prices_efforts_and_provider_name() {
    let mut root = f::account_root(false);
    let mut me = f::choice_me(100);
    me.providers = f::provider_offers();
    let model = me.projects.get_mut(0).expect("project").models.get_mut(0).expect("model");
    model.provider = Some(f::PROVIDER);
    model.description = Some(f::act_text(b"Reasoning and code"));
    model.prices = c::ModelPrices { input: 13, cached: 7, cache_write: None, output: 31, unit: 1000 };
    root.deliver(c::Delivery::Me(Box::new(me)));
    let call = root.submit(&["model", "list", "--project", "shop"]);
    f::deliver_accounts(&mut root, f::accounts());
    let printed = root.answer(call).expect("model text");
    assert_eq!(printed.status, 0);
    assert!(!printed.stdout.starts_with(b"{"));
    for fact in [
        b"Scripted".as_slice(),
        b"description: Reasoning and code",
        b"input: 13",
        b"cached: 7",
        b"cache_write: 13",
        b"output: 31",
        b"unit: 1000",
        b"- high",
    ] {
        assert!(f::has(&printed, fact), "{fact:?}: {printed:?}");
    }
}

#[test]
fn owner_accounts_include_each_supplied_deployment_host_login_state() {
    let mut deployment = f::deployment();
    deployment.hosts = List::with_capacity(1);
    deployment
        .hosts
        .push(c::Host {
            name: Some(b"deployment-worker".as_slice().into()),
            awaiting_logins: false,
            rule: c::HostRule::Open,
            programs: List::with_capacity(1),
            channel: 17,
            kind: c::HostKind::Worker,
            slots: 1,
            used: 0,
            free: 1,
            draining: false,
            connected: true,
            grace_until: None,
            grace_remaining: None,
            runs: List::with_capacity(0),
            cut: false,
        })
        .expect("host room");
    let host = deployment.hosts.get_mut(0).expect("fixture host");
    host.programs
        .push(c::ProgramLogin {
            name: b"deployment-login".as_slice().into(),
            state: Some(c::ProgramState::NotSignedIn),
            sign_out: None,
            cleanup: false,
            cut: false,
        })
        .expect("login");
    let printed = f::account_read(&mut f::account_root(true), f::accounts(), Some(deployment));
    for fact in [b"deployment-worker".as_slice(), b"deployment-login", b"not_signed_in"] {
        assert!(f::has(&printed, fact), "{printed:?}");
    }
}

fn program_accounts() -> c::Accounts {
    let mut accounts = f::accounts();
    let source = accounts.accounts.get_mut(0).expect("account").sources.get_mut(0).expect("source");
    source.kind = c::AccountKind::Program;
    source.name = Some(b"helper".as_slice().into());
    source.state = c::AccountState::Granted;
    source.usable = true;
    source.spent_until = None;
    source.expiring = false;
    source.attention = false;
    accounts.program_hosts = List::with_capacity(4);
    for name in [b"worker-one".as_slice(), b"worker-two", b"worker-three", b"worker-four"] {
        let mut programs = List::with_capacity(1);
        programs
            .push(c::ProgramLogin {
                name: b"helper".as_slice().into(),
                state: Some(c::ProgramState::Usable),
                sign_out: None,
                cleanup: false,
                cut: false,
            })
            .expect("program");
        accounts
            .program_hosts
            .push(c::ProgramHost {
                name: name.into(),
                channel: 17,
                connected: true,
                awaiting_logins: false,
                programs,
                cut: false,
            })
            .expect("host");
    }
    accounts
}

#[test]
fn one_account_owner_with_hosts_and_offers_is_whole() {
    for json in [true, false] {
        let mut deployment = f::deployment();
        deployment.accounts = List::with_capacity(0);
        deployment.hosts = List::with_capacity(0);
        deployment.providers = f::provider_offers();
        deployment.providers_cut = false;
        deployment.readiness = List::with_capacity(0);
        deployment.readiness_cut = false;
        let mut root = f::account_root(true);
        let call = root.submit(if json { &["account", "list", "--json"][..] } else { &["account", "list"][..] });
        f::deliver_accounts(&mut root, program_accounts());
        f::deliver_deployment(&mut root, deployment);
        let printed = root.answer(call).expect("owner accounts");
        assert_eq!(printed.status, 0);
        assert!(!f::has(&printed, if json { b"\"cut\":true" } else { b"cut: true" }), "{printed:?}");
        assert!(f::has(&printed, b"worker-three"));
        assert!(f::has(&printed, b"https://provider.example/usage"));
        if !json {
            assert!(f::has(&printed, "account 43 · provider 9 · Scripted · Work · yours".as_bytes()), "{printed:?}");
        }
    }
}

#[test]
fn account_list_text_names_the_account_source_and_host() {
    let mut root = f::account_root(false);
    let call = root.submit(&["account", "list"]);
    f::deliver_accounts(&mut root, program_accounts());
    let printed = root.answer(call).expect("account text");
    assert!(f::has(&printed, "account 43 · provider 9 · Scripted · Work · yours".as_bytes()), "{printed:?}");
    f::check_golden("account_list", &printed.stdout);
}

#[test]
fn account_text_explains_each_supplied_host_login_state() {
    let mut accounts = program_accounts();
    for (index, state) in [
        c::ProgramState::Usable,
        c::ProgramState::NotSignedIn,
        c::ProgramState::SignedInOtherwise { method: b"key".as_slice().into() },
        c::ProgramState::Failed { reason: c::ProgramFailure::Deadline, detail: b"check expired".as_slice().into() },
    ]
    .into_iter()
    .enumerate()
    {
        accounts
            .program_hosts
            .get_mut(u32::try_from(index).expect("host index"))
            .expect("host")
            .programs
            .get_mut(0)
            .expect("program")
            .state = Some(state);
    }
    let mut root = f::account_root(false);
    let call = root.submit(&["account", "list"]);
    f::deliver_accounts(&mut root, accounts);
    let printed = root.answer(call).expect("host observations");
    for fact in [
        "host worker-one: usable",
        "host worker-two: not signed in",
        "host worker-three: signed in otherwise (key)",
        "host worker-four: failed deadline · check expired",
        "verify: account verify 43 --source program",
    ] {
        assert!(f::has(&printed, fact.as_bytes()), "{printed:?}");
    }
}

#[test]
fn account_text_distinguishes_deployment_readiness_and_names_jig_loss() {
    for different in [false, true] {
        let mut accounts = f::accounts();
        accounts.readiness = f::payer_readiness();
        let mut deployment = f::deployment();
        deployment.readiness = accounts.readiness.clone();
        deployment.readiness_cut = true;
        if different {
            deployment.readiness.get_mut(0).expect("observation").readiness = c::Readiness::Usable;
        }
        let mut root = f::account_root(true);
        let call = root.submit(&["account", "list"]);
        f::deliver_accounts(&mut root, accounts.clone());
        f::deliver_deployment(&mut root, deployment.clone());
        let printed = root.answer(call).expect("readiness text");
        assert_eq!(f::occurrences(&printed.stdout, "readiness: provider 9 · party 73 · unknown".as_bytes()), 1);
        assert_eq!(f::has(&printed, b"deployment readiness:"), different);
        assert!(f::has(&printed, b"deployment readiness cut: true (jig)\ncut: true\n"));
        let json = f::account_read(&mut root, accounts, Some(deployment));
        assert!(f::has(&printed, &f::last_digest(&json.stdout)), "text and JSON share the bounded projection digest");
    }
}
