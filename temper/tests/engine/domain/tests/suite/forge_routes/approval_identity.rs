//! Approval settings and waits require known forge people (temper/docs/design/domain/forge/change/gates.md, 2 and 3, and temper/docs/design/domain/forge/change/questions.md, 2).
use super::*;
use jig_domain_core::settings as s;
use temper_engine_domain_forge::settings;
use temper_engine_domain_world::kinds;

const ASK_CHARTER: u32 = 11;
const SETTINGS_CHARTER: u32 = 14;

fn box_owner() -> people::IdentityKey {
    people::IdentityKey { provider: 3, subject: b"owner-a".as_slice().into() }
}

fn box_world() -> World {
    world_with_owner(box_owner())
}

fn defaults() -> s::Defaults {
    let llm = jig_protocol_config::Llm {
        endpoints: Box::new([]),
        models: (walking::models())
            .into_iter()
            .enumerate()
            .map(|(at, model)| jig_domain_core::settings::CatalogueModel {
                first_read: None,
                number: u32::try_from(at).expect("bounded fixture models") + 1,
                model,
                shown: None,
                description: None,
                picker: true,
                withdrawn: false,
                source: jig_domain_core::settings::ModelSource::Stock,
                estimated: false,
            })
            .collect(),

        providers: Box::new([]),
    };
    let mut defaults = temper_service::stock::with_models(llm.models.clone(), 4096);
    temper_engine_domain_world::forge_routes::add_settings_charter(&mut defaults);
    defaults
}

fn world_with_owner(identity: people::IdentityKey) -> World {
    let defaults = defaults();
    let mut world = World::configured_stock_limits(
        &defaults,
        &walking::models()[0],
        &temper_engine_domain_world::forge_routes::box_limits(),
    )
    .with_settings_owner(&identity);
    forge_with_only_temper(&mut world);
    world.until(Until::Ready);
    world.send(engine::Event::Party(engine::PartyInput::SignedIn {
        held: None,
        reply_to: ReplyTo::new(Token::new(90)),
        identity: people::Identity {
            key: identity,
            login: b"owner".as_slice().into(),
            name: b"Owner".as_slice().into(),
        },
    }));
    world.until(Until::SignedIn);
    world.send(engine::Event::Party(engine::PartyInput::Application {
        scope: people::ApplicationScope::Project(1),
        reply_to: ReplyTo::new(Token::new(91)),
        sign_in: world.signed_in.expect("owner session"),
        key: [91; 16],
        digest: [5; 32],
        carried: None,
        request: engine::Application::Forge(forge_top::Ask::Adopt(temper_engine_domain_world::fixtures::adoption(1))),
    }));
    for _ in 0..200 {
        world.tick();
    }
    assert!(!world.adopted.is_empty(), "adoption: {:?}", world.web_replies);
    assert!(matches!(world.external(1, raw::Op::Read(raw::Read::Collaborators)),
        raw::Answer::Collaborators(rows) if rows.len() == 1 && rows[0].user == 1));
    world
}

pub(super) fn forge_with_only_temper(world: &mut World) {
    // Only temper is returned at adoption; the deployment owner is a separate identity.
    world.fake_env.limits.limits.users = 6;
    world.fake = fake::Domain::new(&world.fake_env.limits, 71);
    fake::repository(
        &mut world.fake,
        &world.fake_env.limits,
        raw::Setup {
            id: 2,
            name: b"org/repo".as_slice().into(),
            default: b"main".as_slice().into(),
            tree: Box::new([]),
            labels: Box::new([]),
            workflows: Box::new([]),
            protection: None,
            hooked: false,
            checks: raw::Checks {
                cues: Box::new([]),
                contexts: Box::new([b"build".as_slice().into()]),
                latency_min: Duration::ZERO,
                latency_max: Duration::ZERO,
                silent: 0,
                passes: 1000,
                reruns: 0,
                cue: None,
            },
        },
    );
    fake::grant(&mut world.fake, b"org/repo", 1, raw::Permission::Admin);
}

fn approval_rule(role: u32, people: u32) -> settings::Value {
    settings::Value::Rule(settings::LandingRule {
        sets: Box::new([]),
        pattern: forge_top::GatePattern {
            segments: Box::new([b"forge".as_slice().into()]),
            last: forge_top::GateLast::Open(Box::new([])),
        },
        enforce: true,
        ci: true,
        up_to_date: true,
        gates: Box::new([settings::RuleGate {
            number: 1,
            blocking: true,
            freshness: forge_top::JudgeFreshness::Clean,
        }]),
        approvals: Box::new([settings::RuleApproval { role, people, freshness: forge_top::JudgeFreshness::Clean }]),
    })
}

fn asking_proposal(world: &mut World, role: u32, key: u8) -> engine::Assignment {
    proposal(world, role, key, ASK_CHARTER)
}

fn proposal(world: &mut World, role: u32, key: u8, charter: u32) -> engine::Assignment {
    let mut hello_event = protocol::hello_slots(Token::new(7), 2, Box::new([]));
    let engine::Event::Worker(engine::WorkerInput::Link { link: engine::Link::Hello { hello, .. }, .. }) =
        &mut hello_event
    else {
        panic!("worker hello")
    };
    hello.bounds.turns = 800;
    hello.bounds.spend = 100_000;
    hello.bounds.time = Duration::from_secs(28_800);
    hello.bounds.message_bytes = world.env.limits.tasks.message_bytes.max(world.env.limits.tasks.result_bytes);
    hello.bounds.inbox = world.env.limits.tasks.run_inbox;
    world.send(hello_event);
    let people::Ask::Create { project, mut batch } =
        world.chat(1, b"Agent review and one person's approval. Keep reports short.")
    else {
        panic!("chat")
    };
    batch[0].executor = people::TaskExecutor::Agent { charter };
    let run = assigned_chat(world, key, people::Ask::Create { project, batch });
    assert_eq!(run.charter, charter);
    let mut charter = defaults().charters[0].clone();
    charter.policy.instructions = b"Keep reports short.".as_slice().into();
    let items: Box<[s::Item]> = Box::new([
        kinds::item(1, 0, 1, &approval_rule(role, 1)),
        s::Item { setting: s::Setting::Charter(1), edit: s::Edit::Set(s::Value::Charter(Box::new(charter))) },
    ]);
    let event = world.protocol.call(Token::new(7), run.task, run.attempt, u64::from(key), engine::Call {
        completion: 1, position: 1, tool: engine::Tool::ProposeSettings { more: false,
            against: kinds::revision(&world.store.rows), kinds: kinds::decoded(&items), items,
            reason: b"Review changes\ntemper enforces: agent review and one person's approval\nyour agents follow: keep reports short".as_slice().into(),
        },
    });
    world.send(event);
    for _ in 0..200 {
        world.tick();
    }
    run
}

#[test]
fn a_box_refuses_an_unfulfillable_person_approval_rule() {
    for (charter, at_call) in [(SETTINGS_CHARTER, false), (ASK_CHARTER, true)] {
        let mut world = box_world();
        let before = kinds::revision(&world.store.rows);
        let run = proposal(&mut world, 0, 92, charter);
        let refusal = if at_call {
            let Some(engine::CallAnswer::Parts(answer)) = world.answers.last() else {
                panic!("asking call refuses before recording a proposal: {:?}", world.answers)
            };
            let engine::PartsAnswer::Approvals { why } = answer.as_ref() else {
                panic!("asking call's approval refusal: {answer:?}")
            };
            String::from_utf8_lossy(why).into_owned()
        } else {
            let Some(engine::CallAnswer::Proposed { proposal, .. }) = world.answers.last().cloned() else {
                panic!("settings charter records a proposal before acceptance: {:?}", world.answers)
            };
            world.send(engine::Event::Party(engine::PartyInput::Ask {
                models: Box::new([]),
                reply_to: ReplyTo::new(Token::new(93)),
                sign_in: world.signed_in.expect("owner session"),
                key: [93; 16],
                ask: people::Ask::DecideProposal {
                    project: people::DEPLOYMENT,
                    proposer: people::Proposer::Task(run.task),
                    proposal,
                    decision: people::ProposalDecision::Accept,
                    running: Some(people::Running::Stays),
                },
            }));
            settle(&mut world);
            world
                .web_replies
                .iter()
                .find_map(|reply| match reply {
                    people::Reply::Outcome(people::Outcome::Refused(people::Refusal::Judged { why, .. })) => {
                        Some(String::from_utf8_lossy(why).into_owned())
                    }
                    _ => None,
                })
                .expect("acceptance refuses the unusable approval rule")
        };
        assert_eq!(kinds::revision(&world.store.rows), before, "an unusable rule must never become effective");
        for words in [
            "project 1 role 0 on forge forge.example requires 1 distinct known forge people; currently 0 eligible",
            "add",
        ] {
            assert!(
                refusal.contains(words),
                "charter {charter}: actionable approval refusal needs {words:?}: {refusal:?}; answers={:?}; replies={:?}",
                world.answers,
                world.web_replies
            );
        }
        world.close();
    }
}

#[expect(
    clippy::fn_params_excessive_bools,
    reason = "the table varies the link, box and forge identities independently"
)]
fn approval_identity_world(link: bool, under_box: bool, second_forge: bool, fake_forge: bool) -> World {
    let identity = if link { box_owner() } else { forge_top::user_key(1, 7) };
    let mut world = world_with_owner(identity);
    let person = world
        .store
        .rows
        .values()
        .find_map(|record| match record {
            Record::Core(jig_domain_core::Record::People(people::Stored::SignIn { person, number, .. }))
                if Some(*number) == world.signed_in =>
            {
                Some(*person)
            }
            _ => None,
        })
        .expect("signed-in owner");
    let forge = forge_top::ForgeRecord {
        number: 1,
        kind: forge_top::Kind::Forgejo,
        mode: if fake_forge { forge_top::Mode::Fake } else { forge_top::Mode::Connected },
        state: forge_top::ForgeState::Kept,
        place: forge_top::Place {
            api: forge_top::Address { host: b"forge.example".as_slice().into(), port: 443, tls: None },
            git: Box::new([]),
            web: None,
        },
        read_only: false,
        bot: forge_top::Bot { login: b"temper".as_slice().into(), name: Box::new([]), email: Box::new([]) },
        asked: false,
        first: None,
        git_program: None,
        person: link.then_some(forge_top::Link { user: 7, person }),
    };
    let mut records = vec![Record::Forge { row: Box::new(forge_top::Stored::Forge(forge.clone())) }];
    if second_forge {
        let repository = world
            .store
            .rows
            .values()
            .find_map(|record| match record {
                Record::Forge { row } => match row.as_ref() {
                    forge_top::Stored::Repository(repository) => Some(repository.clone()),
                    _ => None,
                },
                Record::Core(_) => None,
            })
            .expect("adopted repository");
        let mut other = repository;
        other.provider.forge = 2;
        other.host = b"other.example".as_slice().into();
        other.home = false;
        records.push(Record::Forge { row: Box::new(forge_top::Stored::Repository(other)) });
        records.push(Record::Forge {
            row: Box::new(forge_top::Stored::Forge(forge_top::ForgeRecord { number: 2, person: None, ..forge })),
        });
    }
    world.store.seed(records.into_boxed_slice(), Box::new([]));
    world = world.with_restart_under_box(under_box);
    world.restart_same();
    settle(&mut world);
    world
}

#[test]
fn approval_rules_count_known_forge_people_with_the_required_role() {
    const ONE_SHORT: &str =
        "project 1 role 0 on forge forge.example requires 2 distinct known forge people; currently 1 eligible.";
    const NO_LINK: &str =
        "project 1 role 0 on forge forge.example requires 1 distinct known forge people; currently 0 eligible.";
    const WRONG_ROLE: &str =
        "project 1 role 1 on forge forge.example requires 1 distinct known forge people; currently 0 eligible.";
    const OTHER_FORGE: &str =
        "project 1 role 0 on forge other.example requires 1 distinct known forge people; currently 0 eligible.";
    const FAKE: &str = "project 1: the playground's forge has no accounts, so no one can approve there. Name a branch pattern or a project that leaves its repository out, or remove the approval requirement.";
    for (kind, scope, role, required, link, under_box, second_forge, fake_forge, host, refusal) in [
        (1, 0, 0, 1, false, false, false, false, None, None),
        (2, 1, 0, 1, false, false, false, false, None, None),
        (1, 0, 0, 2, false, false, false, false, None, Some(ONE_SHORT)),
        (2, 1, 0, 2, false, false, false, false, None, Some(ONE_SHORT)),
        (1, 0, 1, 1, false, false, false, false, None, Some(WRONG_ROLE)),
        (2, 1, 1, 1, false, false, false, false, None, Some(WRONG_ROLE)),
        (1, 0, 0, 1, true, true, false, false, None, None),
        (2, 1, 0, 1, true, true, false, false, None, None),
        (1, 0, 0, 2, true, true, false, false, None, Some(ONE_SHORT)),
        (2, 1, 0, 2, true, true, false, false, None, Some(ONE_SHORT)),
        (1, 0, 0, 1, true, false, false, false, None, Some(NO_LINK)),
        (1, 0, 0, 1, false, false, true, false, None, Some(OTHER_FORGE)),
        (2, 1, 0, 1, false, false, true, false, Some(b"other.example".as_slice()), Some(OTHER_FORGE)),
        (2, 1, 0, 1, false, false, true, false, Some(b"forge.example".as_slice()), None),
        (1, 0, 0, 1, false, false, false, true, None, Some(FAKE)),
        (2, 1, 0, 2, false, false, false, true, None, Some(FAKE)),
    ] {
        let mut world = approval_identity_world(link, under_box, second_forge, fake_forge);
        let mut rule = approval_rule(role, required);
        if let Some(host) = host {
            let settings::Value::Rule(rule) = &mut rule else { panic!("approval rule") };
            rule.pattern.segments = Box::new([b"forge".as_slice().into(), host.into()]);
        }
        let before = kinds::revision(&world.store.rows);
        world.send(kinds::change(
            world.signed_in.expect("owner session"),
            before,
            94,
            Box::new([kinds::item(kind, scope, 1, &rule)]),
        ));
        settle(&mut world);
        assert_eq!(
            kinds::revision(&world.store.rows),
            before + u32::from(refusal.is_none()),
            "kind {kind} requiring {required} people, link {link}, box {under_box}, second forge {second_forge}, fake {fake_forge}: {:?}",
            world.web_replies
        );
        if let Some(expected) = refusal {
            let why = world
                .web_replies
                .iter()
                .rev()
                .find_map(|reply| match reply {
                    people::Reply::Outcome(people::Outcome::Refused(people::Refusal::Judged { why, .. })) => Some(why),
                    _ => None,
                })
                .expect("approval eligibility refusal");
            assert!(why.starts_with(expected.as_bytes()), "{}", String::from_utf8_lossy(why));
        }
        world.close();
    }
}

fn drop_approvals(world: &mut World) {
    let items: Vec<_> = world
        .store
        .rows
        .values()
        .filter_map(|record| match record {
            Record::Forge { row } => match row.as_ref() {
                forge_top::Stored::Setting(settings::SettingRow { key, edit: settings::Edit::Set(value) })
                    if key.slot == settings::SettingSlot::Default =>
                {
                    match &value.value {
                        settings::Value::Rule(rule) => {
                            let mut rule = rule.clone();
                            rule.approvals = Box::new([]);
                            Some(kinds::item(key.kind, key.scope, key.number, &settings::Value::Rule(rule)))
                        }
                        settings::Value::Charter(_)
                        | settings::Value::Questions(_)
                        | settings::Value::Ways(_)
                        | settings::Value::CharterWords(_)
                        | settings::Value::Goal(_)
                        | settings::Value::Classes(_)
                        | settings::Value::ProjectWords(_) => None,
                    }
                }
                _ => None,
            },
            Record::Core(_) => None,
        })
        .collect();
    assert_eq!(items.len(), 2, "deployment and project rules both enforce approvals");
    let before = kinds::revision(&world.store.rows);
    world.send(kinds::change(world.signed_in.expect("owner session"), before, 95, items.into_boxed_slice()));
    for _ in 0..200 {
        world.tick();
    }
    assert_eq!(kinds::revision(&world.store.rows), before + 1, "{:?}", world.web_replies);
}

#[test]
fn an_unknown_head_approval_waits_until_the_approval_requirement_is_removed() {
    let mut world = World::configured_limits(
        true,
        true,
        false,
        1000,
        Gate { approval: Some(engine::Freshness::Clean), ..Gate::default() },
        None,
        StoryLimits { kinds: true, ..StoryLimits::default() },
    );
    forge_with_only_temper(&mut world);
    let (mut world, _, producer, branch) = change_world_in(world, Change::default());
    fake::grant(&mut world.fake, b"org/repo", 2, raw::Permission::Write);
    let tasks::Party::Task(change) = live_task(&world, producer.task).requester else { panic!("producing change") };
    reviews::produce(&mut world, &producer, &branch);
    for _ in 0..200 {
        world.tick();
    }
    assert_eq!(world.merges, 0, "the rule waits for approval");
    let pull = light_rules::change_row(&world, change).pull.expect("opened pull");
    assert!(matches!(
        world.external(
            2,
            raw::Op::Write(raw::Write::Review {
                number: pull,
                verdict: Some(raw::Verdict::Approve),
                body: b"Looks good.".as_slice().into(),
            })
        ),
        raw::Answer::Reviewed(_)
    ));
    world.send(engine::Event::Forge(Box::new(forge_top::Event::Hint {
        hint: skein_forge::api::Hint {
            repository: forge_world::REPO,
            change: skein_forge::api::Change::Item(pull),
            key: None,
        },
    })));
    for _ in 0..200 {
        world.tick();
    }
    assert_eq!(world.merges, 0, "an unknown author's approval at the head cannot authorize merging");
    assert!(matches!(
        light_rules::change_row(&world, change).change.state,
        temper_engine_domain_forge_change::State::Queued { .. }
            | temper_engine_domain_forge_change::State::First { .. }
    ));
    drop_approvals(&mut world);
    assert_eq!(world.merges, 1, "removing the approval requirements releases the queue");
    world.close();
}

/// Reproduce an approval wait, an unmapped review, a satisfied rule and its removal.
pub(crate) fn approval_wait_story() {
    let (mut world, change) = waiting_approval();
    let panel = approval_panel(&mut world, change, 990);
    assert!(matches!(panel.step, forge_top::PanelStep::Queued { .. }), "a sign-off waits outside the queue");
    assert_eq!(panel.ci, temper_engine_domain_forge_change::Status::Passed);
    assert_eq!(panel.gates[0].verdict, temper_engine_domain_forge_change::Status::Passed);
    assert_approval_wait(&panel);
    post_approval(&mut world, change, 2);
    let panel = approval_panel(&mut world, change, 991);
    assert_approval_wait(&panel);
    assert_eq!(world.merges, 0, "unmapped forge reviews never authorize merging");
    assert_eq!(panel.unmapped_reviews.len(), 1, "{panel:?}");
    assert_eq!(panel.unmapped_reviews[0].account, 2);
    assert_eq!(Some(panel.unmapped_reviews[0].head), panel.head);
    post_approval(&mut world, change, 7);
    assert_eq!(world.merges, 1, "a mapped reviewer with role 0 satisfies both rules");
    let panel = approval_panel(&mut world, change, 992);
    assert!(panel.approvals.iter().all(|line| line.remaining != Some(1)), "the satisfied deficit must cease");
    drop_approvals(&mut world);
    assert!(approval_panel(&mut world, change, 993).approvals.is_empty(), "live rules determine the panel");
    world.close();
}

fn another_waiting_approval(world: &mut World, name: u64) -> u64 {
    let chat = world.assigned[0].clone();
    let before = world.assigned.len();
    let engine::CallAnswer::Delegated(made) =
        super::change_tool::change(world, &chat, name, None, b"Independent change")
    else {
        panic!("second change delegated: {:?}", world.answers);
    };
    let change = made[0].task;
    let producer = reviews::next_assignment(world, before);
    let branch = light_rules::change_row(world, change).branch.clone();
    reviews::produce(world, &producer, &branch);
    let head = world.fake.branch(b"org/repo", &branch).expect("produced head");
    super::update_first::status(world, head, raw::Check::Passed);
    let reviewer = reviews::next_assignment(world, before + 1);
    reviews::finish_review(
        world,
        &reviewer,
        tasks::TaskResult::Verdict { code: 1, fields: Box::new([]), words: b"approved".as_slice().into() },
    );
    settle(world);
    change
}

#[test]
fn a_later_approved_change_lands_while_the_earlier_waits_outside_the_queue() {
    use temper_engine_domain_forge_change::State;
    let (mut world, x) = waiting_approval();
    let since = light_rules::change_row(&world, x).change.ready_since;
    world.restart_same();
    settle(&mut world);
    assert!(matches!(light_rules::change_row(&world, x).change.state, State::Queued { .. }));
    world.env.now = world.env.now.saturating_add(Duration::from_secs(1));
    let y = another_waiting_approval(&mut world, 94);
    post_approval(&mut world, y, 7);
    assert_eq!(world.merges, 1, "Y lands while X's sign-off waits");
    assert!(matches!(light_rules::change_row(&world, y).change.state, State::Landed { .. }));
    let row = light_rules::change_row(&world, x);
    assert!(matches!(row.change.state, State::Queued { .. }));
    assert!(!row.change.owns_turn);
    assert_eq!(row.change.ready_since, since);
    world.signed_in = None;
    world.sign_in();
    let watcher = Token::new(994);
    watch_approvals_at(&mut world, watcher, 0);
    let document = approvals_of(&world, watcher);
    assert_eq!(document.rows.len(), 1);
    assert_eq!(document.rows[0].task, x);
    assert!(document.rows[0].sets.is_empty(), "these rules name no path set");
    post_approval(&mut world, x, 7);
    finish_change_gate(&mut world, x);
    assert_eq!(
        world.merges,
        2,
        "X returns and lands at its first ready time: {:?}",
        light_rules::change_row(&world, x).change
    );
    assert_eq!(light_rules::change_row(&world, x).change.ready_since, since);
    world.close();
}

#[test]
fn a_returning_signoff_follows_the_owner_and_yields_when_its_exact_gate_is_asked_again() {
    use temper_engine_domain_forge_change::State;
    let (mut world, x) = waiting_approval_with_client(true);
    let since = light_rules::change_row(&world, x).change.ready_since;
    world.env.now = world.env.now.saturating_add(Duration::from_secs(1));
    let y = another_waiting_approval(&mut world, 94);
    world.env.now = world.env.now.saturating_add(Duration::from_secs(1));
    let z = another_waiting_approval(&mut world, 95);
    super::update_first::move_base(&mut world, b"main");
    world.hold_write_answers = Some(HoldWrite::All);
    post_approval_read(&mut world, y, 7);
    for _ in 0..200 {
        world.tick();
        if !world.held_write_answers.is_empty() {
            break;
        }
    }
    assert!(matches!(light_rules::change_row(&world, y).change.state, State::Updating { .. }));
    assert!(!world.held_write_answers.is_empty());
    post_approval_read(&mut world, z, 7);
    post_approval_read(&mut world, x, 7);
    let panel = approval_panel(&mut world, x, 995);
    assert!(
        panel.approvals.iter().all(|line| line.remaining == Some(0)),
        "X's sign-off counts while Y updates: {panel:?}"
    );
    assert_eq!(world.merges, 0);
    assert!(light_rules::change_row(&world, y).change.owns_turn, "Y keeps its update turn");
    for task in [x, z] {
        let row = light_rules::change_row(&world, task);
        assert!(matches!(row.change.state, State::Queued { .. }), "{row:?}");
        assert!(!row.change.owns_turn, "a returning sign-off never overtakes the owner");
    }
    assert_eq!(light_rules::change_row(&world, x).change.ready_since, since);
    world.release_write_answers();
    world.env.now = world.env.now.saturating_add(world.env.limits.forge.queue_window);
    settle(&mut world);
    assert_eq!(world.merges, 1, "Y lands first");
    assert!(matches!(light_rules::change_row(&world, y).change.state, State::Landed { .. }));
    finish_change_gate(&mut world, x);
    settle(&mut world);
    assert_eq!(world.merges, 2, "Z keeps its turn while X's exact gate is asked again");
    assert!(matches!(light_rules::change_row(&world, z).change.state, State::Landed { .. }));
    assert!(matches!(light_rules::change_row(&world, x).change.state, State::Gating { .. }));
    assert!(!light_rules::change_row(&world, x).change.owns_turn);
    finish_change_gate(&mut world, x);
    assert_eq!(world.merges, 3);
    let merged: Vec<_> = world
        .writes
        .iter()
        .filter_map(|write| match write {
            skein_forge::api::Write::Merge { number, .. } => Some(*number),
            _ => None,
        })
        .collect();
    assert_eq!(merged, [y, z, x].map(|task| light_rules::change_row(&world, task).pull.expect("pull")));
    world.close();
}

fn finish_change_gate(world: &mut World, change: u64) {
    for _ in 0..200 {
        if let Some((task, _)) = light_rules::change_row(world, change).delegate
            && world.assigned.iter().any(|run| run.task == task)
        {
            break;
        }
        world.tick();
    }
    let (task, kind) = light_rules::change_row(world, change).delegate.unwrap_or_else(|| {
        panic!("exact gate at the updated head for {change}: {:?}", light_rules::change_row(world, change).change)
    });
    assert!(matches!(kind, temper_engine_domain_forge_change::Delegate::Gate { .. }));
    let reviewer = world.assigned.iter().rev().find(|run| run.task == task).expect("gate assigned").clone();
    reviews::finish_review(
        world,
        &reviewer,
        tasks::TaskResult::Verdict { code: 1, fields: Box::new([]), words: b"updated head approved".as_slice().into() },
    );
    settle(world);
}

fn assert_approval_wait(panel: &forge_top::Panel) {
    assert_eq!(panel.approvals.len(), 2, "each enforcing scope's requirement is visible");
    for line in &panel.approvals {
        assert_eq!((line.role, line.required, line.freshness), (0, 1, forge_top::JudgeFreshness::Clean));
        assert_eq!(line.read, forge_top::ReviewRead::Complete, "reviews read with none qualifying differs from unread");
        assert_eq!(line.remaining, Some(1));
        assert!(line.counted.is_empty());
    }
}

fn approval_panel(world: &mut World, change: u64, watcher: u64) -> forge_top::Panel {
    world.signed_in = None;
    world.sign_in();
    world.send(engine::Event::Party(engine::PartyInput::Watch {
        watcher: Token::new(watcher),
        sign_in: world.signed_in.expect("session"),
        subject: jig_domain_core::documents::Watched::Application {
            scope: people::ApplicationScope::Project(1),
            connector: 0,
            subject: Token::new(change),
        },
        window: jig_domain_core::documents::Window { rows: world.env.limits.documents.rows, after: None },
    }));
    for _ in 0..20 {
        world.tick();
    }
    let panel = world
        .panels
        .iter()
        .rev()
        .find(|(token, _)| *token == Token::new(watcher))
        .unwrap_or_else(|| panic!("change panel for {watcher}; delivered {:?}", world.panels))
        .1
        .clone();
    world.send(engine::Event::Party(engine::PartyInput::Unwatch { watcher: Token::new(watcher) }));
    for _ in 0..20 {
        world.tick();
    }
    panel
}

pub(super) fn post_approval_read(world: &mut World, change: u64, user: u64) {
    fake::grant(&mut world.fake, b"org/repo", user, raw::Permission::Write);
    let pull = light_rules::change_row(world, change).pull.expect("opened pull");
    assert!(matches!(
        world.external(
            user,
            raw::Op::Write(raw::Write::Review {
                number: pull,
                verdict: Some(raw::Verdict::Approve),
                body: b"Looks good.".as_slice().into(),
            })
        ),
        raw::Answer::Reviewed(_)
    ));
    world.send(engine::Event::Forge(Box::new(forge_top::Event::Hint {
        hint: skein_forge::api::Hint {
            repository: forge_world::REPO,
            change: skein_forge::api::Change::Item(pull),
            key: None,
        },
    })));
}

pub(super) fn post_approval(world: &mut World, change: u64, user: u64) {
    post_approval_read(world, change, user);
    world.env.now = world.env.now.saturating_add(world.env.limits.forge.queue_window);
    settle(world);
}

// The fake forge has no inline-remark write; its domain read supplies that row.
fn request_reads(world: &mut World, review: u64, reviewed: [u8; 32], remarks: bool) {
    use skein_forge::api::Answer;
    for event in &mut world.events {
        if let engine::Event::Forge(event) = event
            && let forge_top::Event::Client(skein_forge_client::Event::Answered { result: Ok(answer), .. }) =
                event.as_mut()
        {
            match answer {
                Answer::Reviews { reviews, .. } => {
                    for row in reviews {
                        if row.id == review {
                            row.commit = reviewed;
                        }
                    }
                }
                Answer::Remarks { remarks: rows, .. } if remarks => {
                    *rows = Box::new([skein_forge::api::Remark {
                        id: 901,
                        author: 7,
                        path: b"file".as_slice().into(),
                        line: 1,
                        body: b"Keep the blank line.".as_slice().into(),
                    }]);
                }
                _ => {}
            }
        }
    }
}

fn repaired_request_waits_for_approval(world: &mut World, change: u64, repair: &engine::Assignment) {
    use temper_engine_domain_forge_change::State;
    let row = light_rules::change_row(world, change);
    let head = row.change.last_head.expect("repaired head");
    let repairs = row.change.repairs;
    let branch = row.branch.clone();
    let assigned = world.assigned.len();
    // A repair's new head asks the gate and the person's approval afresh.
    let new = fake::advance(&mut world.fake, &world.fake_env, b"org/repo", &branch, b"file", b"repaired edge", 2)
        .expect("repair push");
    assert_ne!(translate::commit(new), head);
    reviews::finish_review(
        world,
        repair,
        tasks::TaskResult::Change {
            title: jig_domain_tasks::ChangeTitle::new(b"Repaired".as_slice().into()),
            connector: 0,
            kind: 2,
            resource: forge_top::tag(forge_world::REPO.forge, forge_world::REPO.repository),
            words: b"repaired".as_slice().into(),
        },
    );
    let reviewer = reviews::next_assignment(world, assigned);
    reviews::finish_review(
        world,
        &reviewer,
        tasks::TaskResult::Verdict {
            code: 1,
            fields: Box::new([]),
            words: b"approved repaired head".as_slice().into(),
        },
    );
    settle(world);
    let row = light_rules::change_row(world, change);
    assert_eq!(row.change.last_head, Some(translate::commit(new)));
    assert_eq!(row.change.repairs, repairs, "old request cannot repair again");
    assert!(matches!(row.change.state, State::First { .. } | State::Queued { .. }), "{row:?}");
    assert!(!matches!(live_task(world, change).phase, tasks::Phase::Held { .. }));
    assert_eq!(world.merges, 0, "approval still needed at the new head");
}

#[test]
fn a_persons_request_for_changes_repairs_with_its_remarks() {
    use temper_engine_domain_forge_change::{Delegate, Hold, Repair, State};
    for (body, remarks, moved_past, bound) in [
        (b"Please repair the edge.".as_slice(), true, false, false),
        (b"".as_slice(), false, false, false),
        (b"Old request".as_slice(), false, true, false),
        (b"At the bound".as_slice(), false, false, true),
    ] {
        let (mut world, change) = waiting_approval();
        assert!(matches!(light_rules::change_row(&world, change).change.state, State::Queued { .. }));
        if bound {
            let mut row = light_rules::change_row(&world, change).clone();
            row.change.repairs = world.env.limits.forge.change_policy.repairs;
            world.store.seed(
                Box::new([Record::Forge { row: Box::new(forge_top::Stored::Change(Box::new(row))) }]),
                Box::new([]),
            );
            world.restart_same();
            settle(&mut world);
        }
        let row = light_rules::change_row(&world, change);
        let head = row.change.last_head.expect("head awaiting approval");
        let reviewed = if moved_past { [99; 32] } else { head };
        let pull = row.pull.expect("opened pull");
        let repairs = row.change.repairs;
        let assigned = world.assigned.len();
        fake::grant(&mut world.fake, b"org/repo", 7, raw::Permission::Write);
        let raw::Answer::Reviewed(review) = world.external(
            7,
            raw::Op::Write(raw::Write::Review {
                number: pull,
                verdict: Some(raw::Verdict::RequestChanges),
                body: body.into(),
            }),
        ) else {
            panic!("person's request posted")
        };
        world.send(engine::Event::Forge(Box::new(forge_top::Event::Hint {
            hint: skein_forge::api::Hint {
                repository: forge_world::REPO,
                change: skein_forge::api::Change::Item(pull),
                key: None,
            },
        })));
        world.env.now = world.env.now.saturating_add(world.env.limits.forge.queue_window);
        for _ in 0..200 {
            request_reads(&mut world, review, reviewed, remarks);
            world.tick();
            assert!(
                !light_rules::change_row(&world, change).change.owns_turn,
                "an out-of-queue request repairs without taking a turn"
            );
        }
        let row = light_rules::change_row(&world, change);
        assert_eq!(world.merges, 0);
        if moved_past {
            assert_eq!(row.change.repairs, repairs);
            assert_eq!(world.assigned.len(), assigned, "old requests make no repair");
            assert!(!matches!(live_task(&world, change).phase, tasks::Phase::Held { .. }));
            assert!(matches!(row.change.state, State::First { .. } | State::Queued { .. }));
        } else if bound {
            assert_eq!(row.change.repairs, repairs);
            assert_eq!(world.assigned.len(), assigned, "no repair past the bound");
            assert!(matches!(row.change.state, State::Held { why: Hold::Repairs, .. }), "{row:?}");
            assert!(matches!(live_task(&world, change).phase, tasks::Phase::Held { .. }));
        } else {
            assert_eq!(row.change.repairs, repairs + 1, "{row:?}");
            assert!(!matches!(live_task(&world, change).phase, tasks::Phase::Held { .. }));
            let repair = world.assigned.get(assigned).expect("request delegates a repair").clone();
            assert_eq!(row.delegate, Some((repair.task, Delegate::Repair(Repair::Review { head: reviewed }))));
            let hex = engine::commit_text(&head);
            let short = core::str::from_utf8(&hex[..12]).expect("head12");
            assert!(live_task(&world, repair.task).spec.words.starts_with(
                format!("Repair a person's request for changes at {short}: changes requested on the forge").as_bytes()
            ));
            let section = repair
                .sections
                .iter()
                .find(|section| section.kind == engine::BriefKind::Forge(engine::ForgeBriefKind::Reviews))
                .expect("reviews section");
            let engine::BriefBody::Text(words) = &section.body else { panic!("quoted review words") };
            let words = core::str::from_utf8(words).expect("review text");
            if remarks {
                assert!(words.contains("Review: Please repair the edge."), "{words}");
                assert!(words.contains("Remark at file:1 Keep the blank line."), "{words}");
            } else {
                assert!(words.contains("[the request for changes gave no remarks]"), "{words}");
            }
            repaired_request_waits_for_approval(&mut world, change, &repair);
        }
        world.close();
    }
}

pub(super) fn approvals_of(world: &World, watcher: Token) -> engine::Approvals {
    world
        .approvals
        .iter()
        .rev()
        .find(|(token, _)| *token == watcher)
        .unwrap_or_else(|| panic!("approvals for {watcher:?}; delivered {:?}", world.approvals))
        .1
        .clone()
}

fn watch_approvals(world: &mut World, watcher: Token) {
    watch_approvals_at(world, watcher, 1);
}

fn watch_approvals_at(world: &mut World, watcher: Token, connector: u16) {
    world.send(engine::Event::Party(engine::PartyInput::Watch {
        watcher,
        sign_in: world.signed_in.expect("approval watcher session"),
        subject: jig_domain_core::documents::Watched::Application {
            scope: people::ApplicationScope::Own,
            connector,
            subject: Token::new(engine::APPROVALS_SUBJECT),
        },
        window: jig_domain_core::documents::Window { rows: world.env.limits.documents.rows, after: None },
    }));
    settle(world);
}

fn linked_box_world() -> World {
    let mut world = World::configured_stock_limits(
        &defaults(),
        &walking::models()[0],
        &temper_engine_domain_world::forge_routes::box_limits(),
    )
    .with_settings_owner(&box_owner())
    .with_box_setup();
    forge_with_only_temper(&mut world);
    world.until(Until::Ready);
    world.send(engine::Event::Forge(Box::new(forge_top::Event::Connection {
        forge: forge_world::REPO.forge,
        state: forge_top::Connection::Connected { id: 1, login: b"bot".as_slice().into() },
    })));
    world.send(engine::Event::Party(engine::PartyInput::SignedIn {
        held: None,
        reply_to: ReplyTo::new(Token::new(90)),
        identity: people::Identity {
            key: box_owner(),
            login: b"owner".as_slice().into(),
            name: b"Owner".as_slice().into(),
        },
    }));
    world.until(Until::SignedIn);
    readopt(&mut world, 91);
    world.send(engine::Event::Forge(Box::new(forge_top::Event::PersonOf {
        forge: forge_world::REPO.forge,
        answer: forge_top::PersonAnswer::Is(7),
    })));
    settle(&mut world);
    world
}

fn sign_in_other_project(world: &mut World) {
    let owner = world.signed_in.take().expect("owner session");
    world.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(Token::new(160)),
        sign_in: owner,
        key: [160; 16],
        ask: people::Ask::MakeProject { name: "Other project".into(), policy: 1 },
    }));
    settle(world);
    let Some(people::Reply::Outcome(people::Outcome::ProjectMade { project })) = world.web_replies.last() else {
        panic!("other project: {:?}", world.web_replies)
    };
    let project = *project;
    assert_ne!(project, 1);
    world.send(engine::Event::Party(engine::PartyInput::SignedIn {
        held: None,
        reply_to: ReplyTo::new(Token::new(161)),
        identity: people::Identity {
            key: forge_top::user_key(1, 8),
            login: b"elsewhere".as_slice().into(),
            name: b"Other person".as_slice().into(),
        },
    }));
    world.until(Until::SignedIn);
    let sign_in = world.signed_in.expect("other person's session");
    let Some(Record::Core(jig_domain_core::Record::People(people::Stored::SignIn { person, .. }))) =
        world.store.rows.get(&Key::Core(jig_domain_core::Key::People(people::Key::SignIn(sign_in))))
    else {
        panic!("other person's durable sign-in")
    };
    let person = *person;
    let holdings = Box::new([people::Holding { person, role: people::Role::Owner }]);
    world.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(Token::new(162)),
        sign_in: owner,
        key: [162; 16],
        ask: people::Ask::SetRoles { project, holdings: holdings.clone() },
    }));
    settle(world);
    assert_eq!(
        world.store.rows.get(&Key::Core(jig_domain_core::Key::People(people::Key::Roles(project)))),
        Some(&Record::Core(jig_domain_core::Record::People(people::Stored::Roles { project, holdings }))),
    );
    for record in world.store.rows.values() {
        if let Record::Core(jig_domain_core::Record::People(people::Stored::Roles { project: other, holdings })) =
            record
            && *other != project
        {
            assert!(holdings.iter().all(|holding| holding.person != person), "the other party belongs only there");
        }
    }
}

#[test]
fn a_box_persons_change_waits_under_needs_you_until_their_review_lands_it() {
    let mut world = linked_box_world();
    let before = kinds::revision(&world.store.rows);
    let run = asking_proposal(&mut world, 0, 92);
    let Some(engine::CallAnswer::Proposed { proposal, .. }) = world.answers.last().cloned() else {
        panic!("person's approval proposal: {:?}", world.answers)
    };
    world.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(Token::new(93)),
        sign_in: world.signed_in.expect("owner session"),
        key: [93; 16],
        ask: people::Ask::DecideProposal {
            project: people::DEPLOYMENT,
            proposer: people::Proposer::Task(run.task),
            proposal,
            decision: people::ProposalDecision::Accept,
            running: Some(people::Running::Stays),
        },
    }));
    settle(&mut world);
    assert_eq!(kinds::revision(&world.store.rows), before + 1, "{:?}", world.web_replies);
    let event = world.protocol.answer(
        Token::new(7),
        run.task,
        run.attempt,
        2,
        tasks::End::Finished {
            result: tasks::TaskResult::Report { words: b"Proposed.".as_slice().into() },
            cancel_delegates: false,
        },
        engine::Parts::none(),
    );
    world.send(event);
    settle(&mut world);
    let watcher = Token::new(170);
    watch_approvals(&mut world, watcher);
    assert!(approvals_of(&world, watcher).rows.is_empty());
    let change = stock_waiting_change(&mut world);
    world.env.now = world.env.now.saturating_add(world.env.limits.forge.queue_window);
    settle(&mut world);
    assert_eq!(world.merges, 0, "the person's approval keeps the change waiting after the step budget");
    let document = approvals_of(&world, watcher);
    assert_eq!(
        document.rows.len(),
        1,
        "the waiting change must reach the person's existing watch: {:?}",
        light_rules::change_row(&world, change)
    );
    let row = &document.rows[0];
    assert_eq!((row.task, row.project, row.forge), (change, 1, forge_world::REPO.forge));
    assert_eq!((&*row.owner, &*row.name), (b"org".as_slice(), b"repo".as_slice()));
    assert_eq!(Some(row.pull), light_rules::change_row(&world, change).pull);
    assert_eq!((row.required, row.counted, row.yours, row.whole), (1, 0, engine::YourReview::Counts, true));
    let raw::Answer::Pull(pull) = world.external(1, raw::Op::Read(raw::Read::Pull { number: row.pull })) else {
        panic!("waiting pull request")
    };
    let kept = light_rules::change_row(&world, change);
    assert_eq!(kept.change.last_head, Some(forge_world::translate::commit(pull.commit)));
    assert_eq!(pull.size, raw::Size { files: 1, additions: 1, deletions: 0 });
    assert_eq!(row.size, Some(engine::ApprovalSize { files: 1, additions: 1, deletions: 0, cut: false }));
    sign_in_other_project(&mut world);
    let other_watcher = Token::new(171);
    watch_approvals(&mut world, other_watcher);
    assert!(approvals_of(&world, other_watcher).rows.is_empty(), "another project's party sees no approval row");
    post_approval(&mut world, change, 7);
    assert_eq!(world.merges, 1, "the linked person's approval at the head permits landing");
    assert!(approvals_of(&world, watcher).rows.is_empty(), "the landed change leaves Needs you");
    world.close();
}

fn waiting_approval() -> (World, u64) {
    waiting_approval_with_client(false)
}

fn waiting_approval_with_client(concurrent: bool) -> (World, u64) {
    let mut world = World::configured_limits(
        true,
        true,
        false,
        1000,
        Gate { agent: true, approval: Some(engine::Freshness::Clean), ..Gate::default() },
        None,
        StoryLimits { kinds: true, ..StoryLimits::default() },
    );
    if concurrent {
        // Three fresh chains and their keep reads can progress beside the held update.
        let mut client = world.env.limits.forge.client;
        client.pending = 16;
        client.calls = 8;
        world = world.with_forge_client(client);
    }
    forge_with_only_temper(&mut world);
    let (mut world, _, producer, branch) = change_world_in(world, Change::default());
    fake::grant(&mut world.fake, b"org/repo", 2, raw::Permission::Write);
    let revision = kinds::revision(&world.store.rows);
    world.send(kinds::change(
        world.signed_in.expect("session"),
        revision,
        96,
        Box::new([kinds::item(1, 0, 1, &approval_rule(0, 1)), kinds::item(2, 1, 1, &approval_rule(0, 1))]),
    ));
    for _ in 0..200 {
        world.tick();
    }
    assert_eq!(kinds::revision(&world.store.rows), revision + 1, "{:?}", world.web_replies);
    let tasks::Party::Task(change) = live_task(&world, producer.task).requester else { panic!("change") };
    reviews::produce(&mut world, &producer, &branch);
    let reviewer = reviews::next_assignment(&mut world, 2);
    reviews::finish_review(
        &mut world,
        &reviewer,
        tasks::TaskResult::Verdict { code: 1, fields: Box::new([]), words: b"approved".as_slice().into() },
    );
    for _ in 0..200 {
        world.tick();
    }
    assert_eq!(world.merges, 0, "the approval rule keeps the change queued");
    (world, change)
}

pub(super) fn settle(world: &mut World) {
    for _ in 0..200 {
        world.tick();
    }
}

fn readopt(world: &mut World, key: u8) {
    world.send(engine::Event::Party(engine::PartyInput::Application {
        scope: people::ApplicationScope::Project(1),
        reply_to: ReplyTo::new(Token::new(u64::from(key))),
        sign_in: world.signed_in.expect("owner session"),
        key: [key; 16],
        digest: [5; 32],
        carried: None,
        request: engine::Application::Forge(forge_top::Ask::Adopt(temper_engine_domain_world::fixtures::adoption(1))),
    }));
    settle(world);
}

fn decide_proposal(world: &mut World, key: u8, charter: u32) {
    let run = proposal(world, 2, key, charter);
    let Some(engine::CallAnswer::Proposed { proposal, .. }) = world.answers.last().cloned() else {
        panic!("recorded settings proposal: {:?}", world.answers)
    };
    world.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(Token::new(u64::from(key))),
        sign_in: world.signed_in.expect("owner session"),
        key: [key + 20; 16],
        ask: people::Ask::DecideProposal {
            project: people::DEPLOYMENT,
            proposer: people::Proposer::Task(run.task),
            proposal,
            decision: people::ProposalDecision::Accept,
            running: Some(people::Running::Stays),
        },
    }));
    settle(world);
    let event = world.protocol.answer(
        Token::new(7),
        run.task,
        run.attempt,
        2,
        tasks::End::Finished {
            result: tasks::TaskResult::Report { words: b"Proposed.".as_slice().into() },
            cancel_delegates: false,
        },
        engine::Parts::none(),
    );
    world.send(event);
    settle(world);
}

fn collaborator_roles(world: &World) -> std::collections::BTreeMap<u64, people::Role> {
    let mut identities = std::collections::BTreeMap::new();
    let mut holdings = Vec::new();
    for record in world.store.rows.values() {
        match record {
            Record::Core(jig_domain_core::Record::People(people::Stored::Person { number, identity }))
                if identity.key.provider == 0 =>
            {
                let (forge, user) = temper_engine_domain_forge::key_user(&identity.key).expect("forge user key");
                assert_eq!(forge, 1);
                identities.insert(*number, user);
            }
            Record::Core(jig_domain_core::Record::People(people::Stored::Roles { project: 1, holdings: rows })) => {
                holdings = rows.to_vec();
            }
            _ => {}
        }
    }
    holdings.iter().filter_map(|holding| identities.get(&holding.person).map(|user| (*user, holding.role))).collect()
}

#[test]
fn stock_adoption_seeds_safe_human_approval_roles() {
    writer_identity_is_ineligible();
    assert_stock_roles();
    let mut world = box_world();
    let before = kinds::revision(&world.store.rows);
    decide_proposal(&mut world, 101, SETTINGS_CHARTER);
    assert_eq!(kinds::revision(&world.store.rows), before);
    let refusal = world
        .web_replies
        .iter()
        .find_map(|reply| match reply {
            people::Reply::Outcome(people::Outcome::Refused(people::Refusal::Judged { why, .. })) => {
                Some(String::from_utf8_lossy(why))
            }
            _ => None,
        })
        .unwrap_or_else(|| panic!("no eligible human: {:?}", world.web_replies));
    assert_eq!(
        refusal,
        "project 1 role 2 on forge forge.example requires 1 distinct known forge people; currently 0 eligible. Before accepting, add the person as a collaborator on the forge, give that role the permission in this project's policy if needed and adopt the repository again; then propose afresh against the newest revision, or remove the approval requirement."
    );
    for command in ["temper help", "temper ctl", "project show", "repository adopt"] {
        assert!(!refusal.contains(command), "approval refusal names no command: {refusal}");
    }
    assert!(collaborator_roles(&world).is_empty(), "temper is never a human member");
    fake::grant(&mut world.fake, b"org/repo", 3, raw::Permission::Read);
    fake::grant(&mut world.fake, b"org/repo", 5, raw::Permission::Admin);
    readopt(&mut world, 102);
    decide_proposal(&mut world, 103, SETTINGS_CHARTER);
    assert_eq!(kinds::revision(&world.store.rows), before, "other roles cannot fulfill member approvals");
    fake::grant(&mut world.fake, b"org/repo", 4, raw::Permission::Write);
    readopt(&mut world, 104);
    assert_eq!(
        collaborator_roles(&world),
        std::collections::BTreeMap::from([
            (3, people::Role::Observer),
            (4, people::Role::Member),
            (5, people::Role::Maintainer),
        ])
    );
    decide_proposal(&mut world, 105, ASK_CHARTER);
    assert_eq!(
        kinds::revision(&world.store.rows),
        before + 1,
        "known member makes proposal usable: {:?}",
        world.web_replies
    );
    stock_approval_lands(&mut world);
    world.close();
    duplicate_approvals_count_once();
}

fn stock_waiting_change(world: &mut World) -> u64 {
    let ask = world.chat(1, b"Make a small fix.");
    let chat = assigned_chat(world, 106, ask);
    let index = world.assigned.len();
    assert!(matches!(change_tool::change(world, &chat, 3, None, b"Small fix"), engine::CallAnswer::Delegated(_)));
    let producer = reviews::next_assignment(world, index);
    let tasks::Party::Task(change) = live_task(world, producer.task).requester else { panic!("change") };
    let branch = light_rules::change_row(world, change).branch.clone();
    fake::grant(&mut world.fake, b"org/repo", 2, raw::Permission::Write);
    stock_produce(world, &producer, &branch);
    let reviewer = reviews::next_assignment(world, index + 1);
    reviews::finish_review(
        world,
        &reviewer,
        tasks::TaskResult::Verdict { code: 1, fields: Box::new([]), words: b"approved".as_slice().into() },
    );
    settle(world);
    assert_eq!(world.merges, 0, "agent review still needs a human approval");
    change
}

fn stock_approval_lands(world: &mut World) {
    let change = stock_waiting_change(world);
    give_writer_legacy_membership(world);
    for user in [1, 7, 3, 5, 5] {
        post_approval(world, change, user);
        assert_eq!(world.merges, 0, "temper, unknown users, other roles and duplicates cannot count: {user}");
    }
    world.restart_same();
    settle(world);
    assert!(world.root.ready(), "restart completes");
    assert_eq!(world.merges, 0, "restart retains the approval wait");
    post_approval(world, change, 4);
    assert_eq!(world.merges, 1, "the known member's approval at the head permits landing");
}

fn writer_identity_is_ineligible() {
    let mut world = world_with_owner(temper_engine_domain_forge::user_key(1, 1_u64));
    let before = kinds::revision(&world.store.rows);
    world.send(kinds::change(
        world.signed_in.expect("session"),
        before,
        94,
        Box::new([kinds::item(1, 0, 1, &approval_rule(0, 1))]),
    ));
    settle(&mut world);
    assert_eq!(
        kinds::revision(&world.store.rows),
        before,
        "even an existing writer identity with owner role is not human"
    );
    world.close();
}

fn assert_stock_roles() {
    let defaults = defaults();
    let policy = defaults.policies.get(&1).expect("stock policy");
    assert_eq!(
        policy.permissions.as_ref(),
        &[
            people::PermissionRole { connector: 1, permission: 3, role: 1 },
            people::PermissionRole { connector: 1, permission: 2, role: 2 },
            people::PermissionRole { connector: 1, permission: 1, role: 3 },
        ]
    );
    assert_eq!(policy.policy.escalation_role, Some(0));
    let roles = &policy.policy.roles;
    assert_eq!((roles[0].requests.0, roles[0].decides.0), (511, 31));
    assert_eq!((roles[1].requests.0, roles[1].decides.0), (255, 31));
    assert_eq!((roles[2].requests.0, roles[2].decides.0), (185, 0));
    assert_eq!(roles[2].period_spend, roles[0].authority.budget.spend);
    assert_eq!((roles[3].requests.0, roles[3].decides.0, roles[3].period_spend), (128, 0, 0));
    assert_eq!(roles[3].authority.budget.spend, 0);
    assert_eq!(roles[3].authority.tools.0, 0);
    assert!(roles[3].authority.grants.is_empty());
    assert!(roles[3].authority.delegation.kinds.is_empty());
}

fn give_writer_legacy_membership(world: &mut World) {
    let owner = world.signed_in.expect("owner session");
    world.send(engine::Event::Party(engine::PartyInput::SignedIn {
        held: None,
        reply_to: ReplyTo::new(Token::new(150)),
        identity: people::Identity {
            key: temper_engine_domain_forge::user_key(1, 1_u64),
            login: Box::new([]),
            name: Box::new([]),
        },
    }));
    settle(world);
    world.signed_in = Some(owner);
    let mut holdings = Vec::new();
    let mut person = None;
    for row in world.store.rows.values() {
        match row {
            Record::Core(jig_domain_core::Record::People(people::Stored::Person { number, identity }))
                if identity.key == temper_engine_domain_forge::user_key(1, 1_u64) =>
            {
                person = Some(*number);
            }
            Record::Core(jig_domain_core::Record::People(people::Stored::Roles { project: 1, holdings: rows })) => {
                holdings = rows.to_vec();
            }
            _ => {}
        }
    }
    holdings.push(people::Holding { person: person.expect("retained writer identity"), role: people::Role::Member });
    world.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(Token::new(151)),
        sign_in: owner,
        key: [151; 16],
        ask: people::Ask::SetRoles { project: 1, holdings: holdings.into_boxed_slice() },
    }));
    settle(world);
    assert_eq!(collaborator_roles(world).get(&1), Some(&people::Role::Member), "legacy writer has the required role");
}

fn stock_produce(world: &mut World, producer: &engine::Assignment, branch: &[u8]) {
    world.external(2, raw::Op::Git(raw::Git::Create { branch: branch.into(), commit: 1 }));
    fake::advance(&mut world.fake, &world.fake_env, b"org/repo", branch, b"fix", b"checked", 1)
        .expect("stock producer pushes its head");
    // A recovered pull by another writer permits temper to post a review on the fake forge.
    assert!(matches!(
        world.external(
            2,
            raw::Op::Write(raw::Write::OpenPull {
                opens_empty: false,
                title: b"Small fix".as_slice().into(),
                body: Box::new([]),
                head: branch.into(),
                base: b"main".as_slice().into(),
            })
        ),
        raw::Answer::Created(_)
    ));
    let tasks::Contract::Change { connector, kind, .. } = live_task(world, producer.task).contract else {
        panic!("stock producer's change contract")
    };
    let event = world.protocol.answer(
        Token::new(7),
        producer.task,
        producer.attempt,
        5,
        tasks::End::Finished {
            result: tasks::TaskResult::Change {
                title: tasks::ChangeTitle::new(b"Small fix".as_slice().into()),
                connector,
                kind,
                resource: forge_top::tag(forge_world::REPO.forge, forge_world::REPO.repository),
                words: b"pushed".as_slice().into(),
            },
            cancel_delegates: false,
        },
        engine::Parts::none(),
    );
    world.send(event);
}

fn duplicate_approvals_count_once() {
    let mut world = box_world();
    for user in [4, 7] {
        fake::grant(&mut world.fake, b"org/repo", user, raw::Permission::Write);
    }
    readopt(&mut world, 102);
    decide_proposal(&mut world, 105, ASK_CHARTER);
    let before = kinds::revision(&world.store.rows);
    world.send(kinds::change(
        world.signed_in.expect("owner session"),
        before,
        94,
        Box::new([kinds::item(1, 0, 1, &approval_rule(2, 2))]),
    ));
    settle(&mut world);
    assert_eq!(kinds::revision(&world.store.rows), before + 1);
    let change = stock_waiting_change(&mut world);
    for _ in 0..2 {
        post_approval(&mut world, change, 4);
        assert_eq!(world.merges, 0, "two reviews by one member are one human approval");
    }
    post_approval(&mut world, change, 7);
    assert_eq!(world.merges, 1, "a second distinct member fulfills the two-person rule");
    world.close();
}
