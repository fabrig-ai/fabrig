//! Reds of no goal's reach the triager without holding verification (after-landing.md, 7).
use super::*;
use temper_engine_domain_world::{
    kinds,
    tier_runs::{self, LANDING_TICKS, RUN_TICKS, task, tier, until},
};

fn world() -> (World, u64) {
    let mut world = tier_runs::findings_world();
    // One new finding fills this project's bound.
    world.env.limits.forge.findings_per_project = 1;
    let client = world.env.limits.forge.client;
    let mut world = world.with_forge_client(client);
    world.adopt();
    let mut hello = protocol::hello_slots(Token::new(7), 2, Box::new([]));
    let engine::Event::Worker(engine::WorkerInput::Link { link: engine::Link::Hello { hello: worker, .. }, .. }) =
        &mut hello
    else {
        panic!("worker hello")
    };
    worker.bounds.message_bytes = world.env.limits.tasks.message_bytes;
    world.send(hello);
    let ask = world.chat(1, b"Triage the project's findings");
    let triager = assigned_chat(&mut world, 92, ask);
    tiers::baseline(&mut world);
    let items: Box<[_]> = Box::new([kinds::item(
        15,
        1,
        1,
        &forge_top::settings::Value::ProjectTriager(forge_top::settings::ProjectTriager {
            owner: forge_top::settings::AreaOwner::Task(triager.task),
        }),
    )]);
    let against = kinds::revision(&world.store.rows);
    world.send(engine::Event::Party(engine::PartyInput::ChangeSettings {
        reply_to: ReplyTo::new(Token::new(300)),
        sign_in: world.signed_in.expect("project owner"),
        key: [30; 16],
        digest: [30; 32],
        kinds: kinds::decoded(&items),
        change: Box::new(jig_domain_core::settings::Change {
            against,
            items,
            running: jig_domain_core::settings::Running::Stays,
        }),
    }));
    world.until(Until::Ready);
    assert_eq!(kinds::revision(&world.store.rows), against + 1, "triager setting kept");
    (world, triager.task)
}

pub(super) fn findings(world: &World) -> Vec<&forge_top::Finding> {
    world
        .store
        .rows
        .values()
        .filter_map(|record| match record {
            Record::Forge { row } => match row.as_ref() {
                forge_top::Stored::Finding(finding) => Some(finding),
                _ => None,
            },
            Record::Core(_) => None,
        })
        .collect()
}

pub(super) fn news(world: &World, triager: u64) -> Vec<&tasks::Word> {
    task(world, triager)
        .inbox
        .iter()
        .filter(|word| matches!(word.kind, tasks::MessageKind::News { .. }) && word.words.starts_with(b"Finding "))
        .collect()
}

fn external_red(world: &mut World) -> skein_forge::api::Commit {
    let commit =
        fake::advance(&mut world.fake, &world.fake_env, b"org/repo", b"main", b"failures", b"suite.alpha\n", 1)
            .expect("outside landing");
    let commit = translate::commit(commit);
    until(world, RUN_TICKS * 2, "outside landing judged red", |world| tier(world).record.sequence == 2);
    assert_eq!(
        tier(world).record.failing,
        temper_engine_domain_forge_tiers::Failures::Tests(Box::new([b"suite.alpha".as_slice().into()]))
    );
    assert!(
        !world
            .store
            .rows
            .contains_key(&Key::Forge(forge_top::Key::Verification { repository: forge_world::REPO, commit })),
        "not a landing temper made"
    );
    commit
}

#[test]
fn an_outside_landings_red_is_the_project_triagers_finding() {
    let (mut world, triager) = world();
    let commit = external_red(&mut world);
    let rows = findings(&world);
    let [finding] = rows.as_slice() else { panic!("one red finding: {rows:?}") };
    assert_eq!(finding.project, 1);
    assert_eq!(finding.kind, forge_top::FindingKind::Red);
    assert_eq!(finding.severity, forge_top::Severity::High);
    assert_eq!(finding.area, None);
    assert_eq!(finding.state, forge_top::FindingState::New);
    assert_eq!(finding.title.as_ref(), b"Red after landing on main: tier slow");
    let body = std::str::from_utf8(&finding.body).expect("red words");
    assert!(
        body.starts_with(&format!(
            "Red after landing: tier slow run 2 blames landing {} on main: 1 failing.\nsuite.alpha\nRun: ",
            std::str::from_utf8(&forge_top::commit_text(&commit)).unwrap()
        )),
        "{body}"
    );
    assert!(body.contains("End of job test's log:"), "{body}");
    let words = news(&world, triager);
    assert_eq!(words.len(), 1);
    assert!(
        words[0].words.starts_with(b"Finding 1/1 is yours: Red after landing on main: tier slow\nred, high, no area\n")
    );
    assert!(matches!(words[0].kind, tasks::MessageKind::News { class: tasks::NewsClass::Wakes, .. }));
    for _ in 0..LANDING_TICKS {
        tier_runs::tick(&mut world);
    }
    assert_eq!(findings(&world).len(), 1);
    assert_eq!(news(&world, triager).len(), 1);
}

#[test]
fn full_new_findings_do_not_hold_an_outside_landings_red() {
    let (mut world, triager) = world();
    tiers::sign_in(&mut world);
    world.send(engine::Event::Party(engine::PartyInput::Application {
        scope: people::ApplicationScope::Project(1),
        reply_to: ReplyTo::new(Token::new(301)),
        sign_in: world.signed_in.expect("project owner"),
        key: [31; 16],
        digest: [31; 32],
        carried: None,
        request: engine::Application::FileFinding(forge_top::FileFinding {
            project: 1,
            kind: forge_top::FindingKind::Qa,
            severity: forge_top::Severity::Low,
            area: None,
            title: b"Existing new finding".as_slice().into(),
            body: b"Already awaiting triage".as_slice().into(),
        }),
    }));
    world.until(Until::Ready);
    assert_eq!(findings(&world).len(), 1);
    assert_eq!(findings(&world)[0].state, forge_top::FindingState::New);
    let before = findings(&world)[0].clone();
    external_red(&mut world);
    assert_eq!(findings(&world), [&before]);
    assert_eq!(news(&world, triager).len(), 1, "only the existing finding's news");
    assert!(tier(&world).record.run.is_none(), "red judgement completed despite the full table");
    fake::advance(&mut world.fake, &world.fake_env, b"org/repo", b"main", b"failures", b"", 1).expect("outside repair");
    until(&mut world, RUN_TICKS, "tier judges the next landing", |world| tier(world).record.sequence == 3);
    assert_eq!(tier(&world).record.failing, temper_engine_domain_forge_tiers::Failures::Tests(Box::new([])));
}
