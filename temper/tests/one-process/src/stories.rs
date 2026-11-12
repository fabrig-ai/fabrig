//! Native forms, LLM scripts and independent forge observations for repository stories.
use crate::{fixtures, world::Story};
use skein_fake_llm_domain::api::{Finish, Line, Piece, Script, Turn};
use skein_fake_person::{Find, Step};
use skein_lib::Duration;
use skein_ui::Role;
use std::collections::BTreeMap;
use temper_engine_domain as root;
use temper_fake_git_hosted::Remote;
use temper_playground::documents;
pub(crate) use temper_playground::fakes::forge_table;
pub mod playground;

/// The stock settings file is independent of the story, seed and phase.
pub(crate) fn settings_file() -> &'static [u8] {
    static FILE: std::sync::OnceLock<Vec<u8>> = std::sync::OnceLock::new();
    FILE.get_or_init(documents::settings_file)
}

pub(crate) fn adoption() -> bool {
    STORY.with(|slot| matches!(slot.get(), Story::Adoption))
}

pub(crate) fn playground() -> bool {
    STORY.with(|slot| matches!(slot.get(), Story::PlaygroundA))
}

pub(crate) fn model_list() -> bool {
    STORY.with(|slot| matches!(slot.get(), Story::ModelsList))
}

// One hook per push: the setup's topic push and the agent's change.
pub(crate) const ADOPT_HOOKS: usize = 2;
// Two one-token producer turns leave three, below the four-unit admission floor.
const CHANGE_ALLOWANCE: u64 = 5;
const RUN_ALLOWANCE: u64 = 8;

thread_local! { static STORY: std::cell::Cell<Story> = const { std::cell::Cell::new(Story::Chat { words: b"Hello" }) }; }
pub(crate) fn set(story: Story) {
    STORY.with(|slot| slot.set(story));
}

pub(crate) fn client_limits() -> temper_client_shell_terminal::limits::Limits {
    if playground() {
        // The whole story reads replies beyond the small chat fixture's text window.
        return temper_client_shell_terminal::limits::native();
    }
    let mut limits = fixtures::client_limits();
    if STORY.with(|slot| matches!(slot.get(), Story::Adopt | Story::CodingAtStop)) {
        // Batch this whole-exchange story without changing the other stories' schedule.
        limits.io.receive = 16_384;
        limits.io.intake = limits.io.receive;
        limits.io.output = limits.io.receive;
        limits.connections.http.read = limits.io.receive;
        limits.connections.http.send = limits.io.receive;
    }
    limits
}

pub(crate) fn person() -> Vec<Step<temper_client_domain::Address>> {
    let mut steps = vec![
        Step::Go {
            address: temper_client_domain::Address::Inbox { filter: temper_client_domain::address::Filter::ALL },
        },
        see(Find::exactly_named(Role::Link, b"How work runs")),
        press(Role::Link, b"How work runs"),
        see(Find::exactly_named(Role::Link, b"Repositories")),
        press(Role::Link, b"Repositories"),
        see(Find::named(Role::Link, b"Add repositories")),
        press(Role::Link, b"Add repositories"),
        see(Find::exactly_named(Role::Heading { level: 2 }, "Repositories · 0".as_bytes())),
        see(Find::named(Role::TextField, b"Owner")),
        ty(Role::TextField, b"Owner", b"acme"),
        Step::Type { find: Find::exactly_named(Role::TextField, b"Name"), words: b"app".as_slice().into() },
        ty(Role::TextField, b"Landing branch", b"main"),
        press(Role::Check, b"Make it the project's home"),
        see(Find::exactly_named(Role::Button, b"Adopt acme/app")),
        press(Role::Button, b"Adopt acme/app"),
        see(Find::text(b"acme/app")),
        Step::Go { address: temper_client_domain::Address::NewChat { draft: false } },
        see(Find::named(Role::TextArea, b"What do you want done?")),
        ty(Role::TextArea, b"What do you want done?", b"@plan Fix the adopted repository"),
        press(Role::Button, b"Send"),
    ];
    steps.push(see(Find::text(b"@plan Fix the adopted repository")));
    steps
}
pub(crate) fn adoption_person() -> Vec<Step<temper_client_domain::Address>> {
    let mut steps = vec![Step::Go { address: temper_client_domain::Address::Adopt { project: fixtures::PROJECT } }];
    steps.extend(person().into_iter().skip(fixtures::ADOPTION_FORM_START).take(fixtures::ADOPTION_FORM_STEPS));
    steps
}
fn see(find: Find) -> Step<temper_client_domain::Address> {
    Step::See { find, within: fixtures::PERSON_WAIT }
}
fn press(role: Role, name: &[u8]) -> Step<temper_client_domain::Address> {
    Step::Press { find: Find::named(role, name) }
}
fn ty(role: Role, name: &[u8], words: &[u8]) -> Step<temper_client_domain::Address> {
    Step::Type { find: Find::named(role, name), words: words.into() }
}
fn call(name: &[u8], arguments: &[u8]) -> Turn {
    Turn {
        lines: Box::new([Line::Call { name: name.into(), arguments: arguments.into() }]),
        finish: Finish::ToolCalls,
        tokens: fixtures::CALL_TOKENS,
    }
}
pub(crate) fn provider(story: Story) -> skein_fake_peers::llm::Peer {
    if matches!(story, Story::Adoption) {
        return fixtures::provider();
    }
    if matches!(story, Story::ChatReadThenWait { .. }) {
        return fixtures::read_then_wait_provider();
    }
    if matches!(story, Story::Chat { .. } | Story::NoWelcome | Story::ModelsList) {
        return fixtures::provider();
    }
    let grant = |kind| serde_json::json!({"connector":fixtures::FORGE_CONNECTOR,"kind":kind,"segments":["forge","127.0.0.1:3000"],"terminal":"open","last":"acme"});
    let mut delegate = serde_json::json!({"batch":[{
        "executor":{"kind":"procedure","connector":fixtures::FORGE_CONNECTOR,"code":fixtures::PROCEDURE_CODE},
        "spec":{"words":"@edit Fix","parameters":[{"name":fixtures::PARAMETER_NAMES[0],"kind":"resource","connector":fixtures::FORGE_CONNECTOR,"resource":fixtures::REPOSITORY_TAG},{"name":fixtures::PARAMETER_NAMES[1],"kind":"bytes","value":"main"}]},
        "contract":{"kind":"change","connector":fixtures::FORGE_CONNECTOR,"change_kind":fixtures::CHANGE_KIND,"words":fixtures::CHANGE_WORDS},
        "authority":{"tools":fixtures::PRODUCER_TOOLS,"grants":fixtures::GRANT_KINDS.map(grant),"delegation":{"kinds":[{"kind":"agent","number":fixtures::AGENT_KIND}],"tasks":fixtures::PRODUCER_TASKS,"depth":fixtures::PRODUCER_DEPTH},"budget":{"spend":fixtures::PRODUCER_SPEND},"notes":fixtures::PRODUCER_NOTES}
    }]});
    if matches!(story, Story::Merge) {
        delegate["batch"][0]["spec"]["parameters"]
            .as_array_mut()
            .expect("parameters")
            .extend([
                serde_json::json!({"name":fixtures::PARAMETER_NAMES[2],"kind":"bytes","value":std::str::from_utf8(fixtures::MERGE_BRANCH).expect("fixture branch")}),
                serde_json::json!({"name":10,"kind":"bytes","value":"Fix"}),
            ]);
    }
    if matches!(story, Story::HeldChange) {
        // The producer fits; its spend leaves less than another run's allowance.
        delegate["batch"][0]["authority"]["budget"]["spend"] = CHANGE_ALLOWANCE.into();
    }
    let delegate = serde_json::to_vec(&delegate).expect("delegate JSON");
    let finish = call(
        b"finish",
        br#"{"form":"change","text":"","fields":{"title":"Fix the file","body":"The file is fixed."}}"#,
    );
    let edits = if matches!(story, Story::CodingAtStop) {
        vec![
            call(b"shell", br#"{"command":"echo fixed > conflict.txt"}"#),
            call(b"wait", b"{}"),
            call(b"shell", br#"{"command":"echo fixed > conflict.txt"}"#),
            held_turn(),
        ]
    } else if matches!(story, Story::Merge) {
        vec![finish.clone()]
    } else {
        vec![call(b"shell", br#"{"command":"echo fixed > conflict.txt"}"#), finish.clone()]
    };
    fixtures::scripted_provider(Box::new([
        Script {
            cue: b"@plan".as_slice().into(),
            turns: Box::new([call(b"delegate", &delegate), call(b"wait", b"{}"), call(b"wait", b"{}")]),
        },
        Script { cue: b"@edit".as_slice().into(), turns: edits.into_boxed_slice() },
        Script {
            cue: b"Resolve the merge conflict".as_slice().into(),
            turns: Box::new([call(b"shell", br#"{"command":"echo fixed > conflict.txt"}"#), finish]),
        },
    ]))
}
pub(crate) fn forge(story: Story) -> temper_fake_git_hosted::Typed {
    let config = skein_fake_forge_domain::Config {
        limits: skein_fake_forge_domain::Limits {
            statuses: fixtures::FORGE_STATUSES,
            ..documents::hosted_git::CONFIG.limits
        },
        ..documents::hosted_git::CONFIG
    };
    let mut domain = skein_fake_forge_domain::Domain::new(&config, fixtures::SEED);
    let base = skein_fake_forge_domain::repository(
        &mut domain,
        &config,
        skein_fake_forge_domain::api::Setup {
            workflows: Box::new([]),
            id: fixtures::REPOSITORY,
            name: b"acme/app".as_slice().into(),
            default: b"main".as_slice().into(),
            tree: Box::new([skein_fake_forge_domain::api::File {
                path: b"conflict.txt".as_slice().into(),
                content: b"base\n".as_slice().into(),
            }]),
            labels: Box::new([]),
            checks: skein_fake_forge_domain::api::Checks {
                cues: Box::new([]),
                contexts: Box::new([b"check".as_slice().into()]),
                latency_min: Duration::ZERO,
                latency_max: Duration::ZERO,
                silent: fixtures::CI_SILENT,
                passes: fixtures::CI_PASSES,
                reruns: fixtures::CI_RERUNS,
                cue: None,
            },
            protection: None,
            hooked: matches!(story, Story::Adopt),
        },
    );
    skein_fake_forge_domain::grant(
        &mut domain,
        b"acme/app",
        fixtures::WORKER_USER,
        skein_fake_forge_domain::api::Permission::Write,
    );
    let forge = std::rc::Rc::new(std::cell::RefCell::new(skein_fake_peers::forge::Forge::new(domain, config)));
    let mut transport = temper_fake_git_hosted::Typed::new(forge, &forge_table());
    let mut identity = documents::hosted_git::identity();
    identity.login = b"temper".as_slice().into();
    transport.authenticate(Some(&(identity.login, identity.token)));
    assert_eq!(
        transport.commit_tree(base, None, BTreeMap::from([(b"conflict.txt".to_vec(), b"topic\n".to_vec())]), b"topic"),
        fixtures::BRANCH_HEAD
    );

    transport
        .push(b"http://127.0.0.1:3000/acme/app", b"topic", fixtures::BRANCH_HEAD, None)
        .expect("the fixture's existing topic is already on the forge");
    if matches!(story, Story::Merge) {
        let mut forge = transport.forge.borrow_mut();
        let config = forge.config;
        let env = skein_lib::Env { now: skein_lib::Time::ZERO, wall: skein_lib::Wall::from_nanos(0), limits: config };
        let moved = skein_fake_forge_domain::advance(
            &mut forge.domain,
            &env,
            b"acme/app",
            b"main",
            b"conflict.txt",
            b"moved\n",
            fixtures::MOVING_USER,
        )
        .expect("another party moves the base");
        assert_eq!(moved, fixtures::MOVED_BASE, "counted moved base");
    }
    while transport.forge.borrow_mut().domain.pop_observation().is_some() {}
    transport
}
pub(crate) fn prepare_merge_branch(transport: &temper_fake_git_hosted::Typed) {
    let mut forge = transport.forge.borrow_mut();
    let env = skein_lib::Env { now: forge.now, wall: skein_lib::Wall::EPOCH, limits: forge.config };
    let token = forge.next_reply();
    let skein_fake_peers::forge::Forge { domain, pending, .. } = &mut *forge;
    skein_fake_forge_domain::step(
        domain,
        &env,
        skein_fake_forge_domain::Event::Call {
            reply_to: skein_lib::ReplyTo::new(token),
            user: fixtures::WORKER_USER,
            repository: b"acme/app".as_slice().into(),
            op: skein_fake_forge_domain::api::Op::Git(skein_fake_forge_domain::api::Git::Create {
                branch: fixtures::MERGE_BRANCH.into(),
                commit: fixtures::BRANCH_HEAD,
            }),
        },
        pending,
    );
    let mut created = false;
    for _ in 0..32 {
        for _ in 0..forge.pending.len() {
            let reply = forge.pending.pop().expect("pending pass length");
            match reply {
                skein_fake_forge_domain::Request::Reply { to, result } => {
                    let destination = to.into_token();
                    if destination != token {
                        forge.pending.push(skein_fake_forge_domain::Request::Reply {
                            to: skein_lib::ReplyTo::new(destination),
                            result,
                        });
                        continue;
                    }
                    assert_eq!(
                        result,
                        Ok(skein_fake_forge_domain::api::Answer::Branch(
                            skein_fake_forge_domain::api::Created::Created
                        )),
                        "the existing private merge branch"
                    );
                    created = true;
                }
                reply @ skein_fake_forge_domain::Request::Hook { .. } => forge.pending.push(reply),
            }
        }
        if created {
            break;
        }
        let skein_fake_peers::forge::Forge { domain, pending, .. } = &mut *forge;
        skein_fake_forge_domain::fire(domain, &env, pending);
    }
    forge.domain.reclaim();
    assert!(created, "the existing private merge branch answered");
}
pub(crate) fn adopted(rows: &BTreeMap<root::Key, root::Record>) -> u32 {
    u32::try_from(rows.values().filter(|row| matches!(row,
        root::Record::Forge { row } if matches!(row.as_ref(), temper_engine_domain_forge::Stored::Repository(_))
    )).count()).expect("adopted repositories")
}
pub(crate) fn configure(value: &mut serde_json::Value) {
    if STORY.with(|slot| matches!(slot.get(), Story::ChatReadThenWait { .. })) {
        return;
    }
    if STORY.with(|slot| matches!(slot.get(), Story::Chat { .. } | Story::NoWelcome | Story::ModelsList)) {
        return;
    }
    documents::configure(value);
    if STORY.with(|slot| matches!(slot.get(), Story::HeldChange)) {
        value["world_settings"]["landing"]["deployment"] = serde_json::json!([{
            "connector": 1, "kind": 4,
            "pattern": {"segments": ["forge", "127.0.0.1:3000"], "last": "open", "value": "acme"},
            "enforce": true, "ci": true, "up_to_date": true,
            "gates": [{"number": 1, "blocking": true, "freshness": "exact"}], "approvals": []
        }]);
    }
}

pub(crate) fn change_held(rows: &BTreeMap<root::Key, root::Record>) -> bool {
    rows.values().filter(|row| matches!(row,
        root::Record::Core(jig_domain_core::Record::Tasks(jig_domain_tasks::Stored::Live(task)))
            if matches!(task.executor, jig_domain_tasks::Executor::Procedure { .. })
                && matches!(task.phase, jig_domain_tasks::Phase::Held { why: jig_domain_tasks::Hold::Budget, .. })
    )).count() == 1 && rows.values().any(|row| matches!(row,
        root::Record::Forge { row } if matches!(row.as_ref(), temper_engine_domain_forge::Stored::Change(change)
            if !change.change.gates.is_empty() && change.change.last_head.is_some())
    ))
}

pub(crate) fn held_change() -> bool {
    STORY.with(|slot| matches!(slot.get(), Story::HeldChange))
}

/// Chat-only seeds fit the stock ceiling; repository seeds use wider own defaults.
pub(crate) fn uses_stock() -> bool {
    STORY.with(|slot| {
        matches!(slot.get(), Story::Chat { .. } | Story::ChatReadThenWait { .. } | Story::NoWelcome | Story::ModelsList)
    })
}

thread_local! { static FIRST_START: std::cell::Cell<bool> = const { std::cell::Cell::new(false) }; }
pub(crate) fn first_start() -> bool {
    FIRST_START.with(std::cell::Cell::get)
}
thread_local! { static SETTINGS: std::cell::Cell<bool> = const { std::cell::Cell::new(false) }; }
thread_local! { static REVISION: std::cell::Cell<u32> = const { std::cell::Cell::new(1) }; }
pub(crate) fn setup_settings(phase: crate::world::Phase) {
    FIRST_START.with(|slot| {
        slot.set(
            !playground()
                && !STORY.with(|story| matches!(story.get(), Story::NoWelcome))
                && (matches!(phase, crate::world::Phase::Story | crate::world::Phase::Fault { .. })
                    || matches!(phase, crate::world::Phase::Drain)
                        && STORY.with(|story| matches!(story.get(), Story::CodingAtStop))),
        );
    });
    SETTINGS.with(|slot| slot.set(matches!(phase, crate::world::Phase::Drain) && uses_stock()));
    REVISION.with(|slot| {
        slot.set(if uses_stock() && matches!(phase, crate::world::Phase::Drain | crate::world::Phase::Restart) {
            2
        } else {
            1
        });
    });
}
pub(crate) fn needs_settings() -> bool {
    SETTINGS.with(std::cell::Cell::get)
}
pub(crate) fn settings_revision() -> u32 {
    REVISION.with(std::cell::Cell::get)
}
/// The service's typed defaults, starting from the binary's stock and the document's models.
pub(crate) fn given(document: &serde_json::Value) -> temper_service::stock::Given {
    let mut document = document.clone();
    document["world_settings"] = fixtures::world_document()["world_settings"].clone();
    let read_wait =
        STORY.with(|slot| matches!(slot.get(), Story::ChatReadThenWait { .. })) || crate::lower::github::waiting();
    let mut given = documents::given_with(&document, uses_stock(), read_wait);
    if STORY.with(|slot| matches!(slot.get(), Story::HeldChange)) {
        let temper_service::stock::Given::Own { jig, .. } = &mut given else {
            unreachable!("playground supplies own defaults");
        };
        jig.charters[0].spend = RUN_ALLOWANCE;
        jig.charters[0].policy.waiting = skein_lib::Duration::from_millis(u64::from(fixtures::CHAT_WAIT_MS));
        jig.templates[0].budget = jig_domain_people::TemplateBudget::AsAllowed { cap: Some(50) };
    }
    if STORY.with(|slot| matches!(slot.get(), Story::CodingAtStop)) {
        let temper_service::stock::Given::Own { jig, .. } = &mut given else { panic!("repository defaults") };
        jig.charters[0].policy.waiting = Duration::from_millis(u64::from(fixtures::CHAT_WAIT_MS));
    }
    crate::review_delivery::given(&mut given);
    crate::push::given(&mut given);
    given
}

/// The provider releases this only after serve has ended the agent.
pub(crate) const STOP_HOLD: &[u8] = b"stop-mid-turn";
fn held_turn() -> Turn {
    Turn {
        lines: Box::new([Line::Pieces {
            pieces: Box::new([Piece { text: b"after stop".as_slice().into(), hold: Some(STOP_HOLD.into()) }]),
        }]),
        finish: Finish::Stop,
        tokens: fixtures::CALL_TOKENS,
    }
}
