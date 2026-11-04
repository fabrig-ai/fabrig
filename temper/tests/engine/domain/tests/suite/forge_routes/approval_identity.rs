//! Approval settings require known forge people (temper/docs/design/domain/forge.md, 8.7 and 8.9).
use super::*;
use jig_domain_core::settings as s;
use temper_engine_domain_forge::settings;
use temper_engine_domain_world::kinds;

fn box_owner() -> people::IdentityKey {
    people::IdentityKey { provider: 3, subject: b"owner-a".as_slice().into() }
}

fn box_world() -> World {
    world_with_owner(box_owner())
}

fn defaults() -> s::Defaults {
    let llm = jig_protocol_config::Llm {
        endpoints: Box::new([]),
        models: walking::models(),
        descriptions: Box::new([None]),
        providers: Box::new([]),
    };
    temper_service::stock::defaults(&llm, 4096)
}

fn world_with_owner(identity: people::IdentityKey) -> World {
    let defaults = defaults();
    let mut world = World::configured_stock_limits(
        &defaults,
        &walking::models()[0],
        &temper_engine_domain_world::forge_routes::box_limits(),
    )
    .with_owner_identity(identity.clone());
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

fn forge_with_only_temper(world: &mut World) {
    // Only temper is returned at adoption; the box's person joins the forge later.
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

fn configuring_proposal(world: &mut World) -> engine::Assignment {
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
    batch[0].executor = people::TaskExecutor::Agent { charter: 3 };
    let run = assigned_chat(world, 92, people::Ask::Create { project, batch });
    assert_eq!(run.charter, 3);
    let mut charter = defaults().charters[0].clone();
    charter.policy.instructions = b"Keep reports short.".as_slice().into();
    let items: Box<[s::Item]> = Box::new([
        kinds::item(1, 0, 1, &approval_rule(0, 1)),
        s::Item { setting: s::Setting::Charter(1), edit: s::Edit::Set(s::Value::Charter(Box::new(charter))) },
    ]);
    let event = world.protocol.call(Token::new(7), run.task, run.attempt, 1, engine::Call {
        completion: 1, position: 1, tool: engine::Tool::ProposeSettings {
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
    let mut world = box_world();
    let before = kinds::revision(&world.store.rows);
    let run = configuring_proposal(&mut world);
    if let Some(engine::CallAnswer::Proposed { proposal, .. }) = world.answers.last().cloned() {
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
        for _ in 0..200 {
            world.tick();
        }
    }
    assert_eq!(kinds::revision(&world.store.rows), before, "an unusable rule must never become effective");
    let refusal = world
        .web_replies
        .iter()
        .find_map(|reply| match reply {
            people::Reply::Outcome(people::Outcome::Refused(people::Refusal::Judged { why, .. })) => {
                Some(String::from_utf8_lossy(why).into_owned())
            }
            _ => None,
        })
        .expect("acceptance refuses the unusable approval rule");
    for words in ["project 1", "role 0", "requires 1", "known forge", "add"] {
        assert!(
            refusal.contains(words),
            "actionable approval refusal needs {words:?}: {refusal:?}; answers={:?}; replies={:?}",
            world.answers,
            world.web_replies
        );
    }
    world.close();
}

#[test]
fn approval_rules_count_known_forge_people_with_the_required_role() {
    for (kind, scope, role, required, accepted) in [
        (1, 0, 0, 1, true),
        (2, 1, 0, 1, true),
        (1, 0, 0, 2, false),
        (2, 1, 0, 2, false),
        (1, 0, 1, 1, false),
        (2, 1, 1, 1, false),
    ] {
        let mut world = world_with_owner(people::IdentityKey { provider: 0, subject: 7_u64.to_be_bytes().into() });
        let before = kinds::revision(&world.store.rows);
        world.send(kinds::change(
            world.signed_in.expect("owner session"),
            before,
            94,
            Box::new([kinds::item(kind, scope, 1, &approval_rule(role, required))]),
        ));
        for _ in 0..200 {
            world.tick();
        }
        assert_eq!(
            kinds::revision(&world.store.rows),
            before + u32::from(accepted),
            "kind {kind} requiring {required} people: {:?}",
            world.web_replies
        );
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
                        settings::Value::Charter(_) => None,
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
