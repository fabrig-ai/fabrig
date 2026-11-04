//! Approval settings require known forge people (temper/docs/design/domain/forge.md, 8.7 and 8.9).
use super::*;
use jig_domain_core::settings as s;
use temper_engine_domain_forge::settings;
use temper_engine_domain_world::kinds;

fn box_owner() -> people::IdentityKey {
    people::IdentityKey { provider: 3, subject: b"owner-a".as_slice().into() }
}

fn box_world() -> World {
    let llm = jig_protocol_config::Llm {
        endpoints: Box::new([]),
        models: walking::models(),
        descriptions: Box::new([None]),
        providers: Box::new([]),
    };
    let defaults = temper_service::stock::defaults(&llm, 4096);
    let mut world = World::configured_stock_limits(
        &defaults,
        &llm.models[0],
        &temper_engine_domain_world::forge_routes::box_limits(),
    )
    .with_owner_identity(box_owner());
    forge_with_only_temper(&mut world);
    world.until(Until::Ready);
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
    world.send(engine::Event::Party(engine::PartyInput::Application {
        scope: people::ApplicationScope::Project(1), reply_to: ReplyTo::new(Token::new(91)),
        sign_in: world.signed_in.expect("owner session"), key: [91; 16], digest: [5; 32], carried: None,
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
                contexts: Box::new([]),
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

fn approval_rule(people: u32) -> settings::Value {
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
        approvals: Box::new([settings::RuleApproval { role: 0, people, freshness: forge_top::JudgeFreshness::Clean }]),
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
    let mut charter = temper_service::stock::defaults(
        &jig_protocol_config::Llm {
            endpoints: Box::new([]),
            models: walking::models(),
            descriptions: Box::new([None]),
            providers: Box::new([]),
        },
        4096,
    )
    .charters[0]
        .clone();
    charter.policy.instructions = b"Keep reports short.".as_slice().into();
    let items: Box<[s::Item]> = Box::new([
        kinds::item(1, 0, 1, &approval_rule(1)),
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
    let refusal =
        world.call_errors.iter().map(|error| String::from_utf8_lossy(&error.text)).collect::<Vec<_>>().join("\n");
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
