//! Reds of no goal's reach the triager without holding verification (after-landing.md, 7).
mod follows;
use super::*;
use temper_engine_domain_world::{
    kinds,
    tier_runs::{self, LANDING_TICKS, RUN_TICKS, task, tier, until},
};

fn world() -> (World, u64) {
    world_with_wait(temper_engine_domain_forge_change::Wait::Cleared)
}

fn world_with_wait(wait: temper_engine_domain_forge_change::Wait) -> (World, u64) {
    let mut world = tier_runs::findings_world_with(tier_runs::Tier { wait, ..tier_runs::Tier::default() });
    // One new finding fills this project's bound.
    world.env.limits.forge.findings_per_project = 1;
    let client = world.env.limits.forge.client;
    let mut world = world.with_forge_client(client);
    world.adopt();
    let mut hello = protocol::hello_slots(Token::new(7), 3, Box::new([]));
    let engine::Event::Worker(engine::WorkerInput::Link { link: engine::Link::Hello { hello: worker, .. }, .. }) =
        &mut hello
    else {
        panic!("worker hello")
    };
    worker.bounds.message_bytes = world.env.limits.tasks.message_bytes;
    world.send(hello);
    let people::Ask::Create { project, mut batch } = world.chat(1, b"Triage the project's findings") else {
        panic!("triager")
    };
    batch[0].budget = people::Budget::AsAllowed { cap: Some(100) };
    let triager = assigned_chat(&mut world, 92, people::Ask::Create { project, batch });
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
    until(world, RUN_TICKS * 2, "outside landing judged red", |world| {
        tier(world).record.sequence == 2 && tier(world).posting.is_none()
    });
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

fn failed_status_link(world: &World, commit: skein_forge::api::Commit) -> &[u8] {
    world
        .writes
        .iter()
        .find_map(|write| match write {
            skein_forge::api::Write::Status { commit: tip, check: skein_forge::api::Check::Failed, link, .. }
                if *tip == commit =>
            {
                Some(link.as_ref())
            }
            _ => None,
        })
        .expect("judged status link")
}

#[test]
fn a_landing_ended_before_verification_posts_red_from_the_log_and_files_once_without_goal_news() {
    let (mut world, triager) = world_with_wait(temper_engine_domain_forge_change::Wait::Landed);
    tiers::sign_in(&mut world);
    let milestone = milestones::milestone(&mut world, b"Red findings");
    let tasks::Party::Person(person) = task(&world, milestone).requester else { panic!("milestone maker") };
    let goal = milestones::goal_with_words(
        &mut world,
        milestone,
        false,
        b"R2 Reds blamed\n\nDraft: temper/docs/design/draft/after-landing-repairs.md",
    );
    let number = tiers::produce(&mut world, &goal, 94, b"failures", b"a::one\na::two\n");
    until(&mut world, LANDING_TICKS, "change ended at landing", |world| {
        matches!(task(world, number).phase, tasks::Phase::Ended(_))
    });
    let commit = tier_runs::merge(&world, number);
    assert!(matches!(
        tier_runs::change(&world, number).change.state,
        temper_engine_domain_forge_change::State::Landed { .. }
    ));
    assert_eq!(tier(&world).record.sequence, 1, "no range verdict before the result");
    assert!(
        !world
            .store
            .rows
            .contains_key(&Key::Forge(forge_top::Key::Verification { repository: forge_world::REPO, commit }))
    );
    assert!(world.store.rows.values().any(|row| matches!(row, Record::Forge { row } if matches!(row.as_ref(), forge_top::Stored::Subscription(sub) if sub.task == goal.task && matches!(&sub.topic, forge_top::Topic::Landings { branch, .. } if branch.as_ref() == b"main")))));
    until(&mut world, RUN_TICKS * 3, "red verdict posted", |world| {
        tier(world).record.sequence == 2 && tier(world).posting.is_none()
    });
    assert!(tier_runs::red_news(&world, goal.task).is_empty(), "live subscribed goal hears no red");
    let rows = findings(&world);
    let [finding] = rows.as_slice() else { panic!("one project red: {rows:?}") };
    assert_eq!((finding.project, finding.kind, finding.area.as_deref()), (1, forge_top::FindingKind::Red, None));
    assert_eq!(finding.severity, forge_top::Severity::High);
    assert_eq!(finding.title.as_ref(), b"2 tests fail in slow since R2 landed");
    assert_eq!(
        finding.blame,
        Some(forge_top::FindingBlame {
            moved: None,
            strength: forge_top::BlameStrength::Owned,
            landings: Box::new([forge_top::BlamedLanding {
                commit,
                goals: Box::new([forge_top::BlamedGoal {
                    goal: goal.task,
                    milestone: Some(milestone),
                    person: Some(person)
                }]),
            }]),
        })
    );
    let runs = tier_runs::runs(&world);
    let logs = tier_runs::job_logs(&world, runs.last().expect("rerun"));
    let first = std::str::from_utf8(logs[0].end.as_ref().unwrap())
        .unwrap()
        .lines()
        .rfind(|line| !line.trim().is_empty())
        .unwrap();
    assert_eq!(
        finding.signature,
        Some(forge_top::FindingSignature {
            repository: forge_world::REPO,
            source: b"slow".as_slice().into(),
            ids: Box::new([b"a::one".as_slice().into(), b"a::two".as_slice().into()]),
            first: first.as_bytes().into(),
            many: false,
        })
    );
    let link = failed_status_link(&world, commit);
    let short = &forge_top::commit_text(&commit)[..12];
    let short = std::str::from_utf8(short).unwrap();
    let pull = tier_runs::change(&world, number).pull.unwrap();
    let mut expected = format!(
        "slow run 2 on main: landing {short}.\nRun: {}\nArtifact: report\nOwned by landing {short}.\nLanding {short}:\nGoal {}: R2 Reds blamed.\nPull request #{pull}.\nDraft: temper/docs/design/draft/after-landing-repairs.md.\nPerson {person}.\nFailing:\na::one\na::two",
        String::from_utf8_lossy(link),
        goal.task,
    );
    for log in logs {
        expected.push_str("\nEnd of job ");
        expected.push_str(&String::from_utf8_lossy(&log.name));
        expected.push_str("'s log:\n");
        expected.push_str(&String::from_utf8_lossy(log.end.as_ref().unwrap()));
    }
    assert_eq!(String::from_utf8_lossy(&finding.body), expected);
    assert_eq!(news(&world, triager).len(), 1);
    assert!(world.writes.iter().any(|write| matches!(write, skein_forge::api::Write::Status { commit: tip, check: skein_forge::api::Check::Failed, .. } if *tip == commit)));
    let pull = tier_runs::change(&world, number).pull.unwrap();
    assert!(world.writes.iter().any(|write| matches!(write, skein_forge::api::Write::Post { number, body, .. } if *number == pull && body.starts_with(b"Red: slow run 2 blames this landing for a::one, a::two."))));
    // The next tip inherits this red; the project's finding is kept just once.
    fake::advance(&mut world.fake, &world.fake_env, b"org/repo", b"main", b"file", b"unrelated", 1).unwrap();
    until(&mut world, RUN_TICKS * 3, "inherited red posted", |world| {
        tier(world).record.sequence == 3 && tier(world).posting.is_none()
    });
    assert_eq!(findings(&world).len(), 1);
    assert_eq!(news(&world, triager).len(), 1);
    assert_eq!(tier_runs::red_news(&world, goal.task).len(), 0);
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
    until(&mut world, RUN_TICKS, "tier judges the next landing", |world| {
        tier(world).record.sequence == 3 && tier(world).posting.is_none()
    });
    assert_eq!(tier(&world).record.failing, temper_engine_domain_forge_tiers::Failures::Tests(Box::new([])));
}

#[test]
fn a_dropped_owned_red_holds_until_a_run_passes_then_leaves_mine_and_is_erasable() {
    let (mut world, triager) = world_with_wait(temper_engine_domain_forge_change::Wait::Landed);
    tiers::sign_in(&mut world);
    let milestone = milestones::milestone(&mut world, b"Observed repairs");
    let goal = milestones::goal_with_words(&mut world, milestone, false, b"R3 Observe a dropped red");
    let change = tiers::produce(&mut world, &goal, 94, b"failures", b"suite.alpha\n");
    until(&mut world, RUN_TICKS * 3, "owned red filed after landing", |world| {
        matches!(task(world, change).phase, tasks::Phase::Ended(_))
            && tier(world).record.sequence == 2
            && tier(world).posting.is_none()
    });
    let finding = findings(&world)[0].clone();
    assert_eq!(finding.blame.as_ref().unwrap().strength, forge_top::BlameStrength::Owned);
    assert!(!finding.erasable());
    let triager = world.assigned.iter().find(|run| run.task == triager).unwrap().clone();
    let answer = world.scripted_call(
        &triager,
        401,
        engine::Tool::Finding(engine::FindingCall::Triage {
            number: finding.number,
            act: forge_top::Triage::Drop(b"Not pursuing".as_slice().into()),
        }),
    );
    assert!(
        matches!(answer, engine::CallAnswer::Finding(answer) if matches!(answer.as_ref(), engine::FindingToolAnswer::Write(forge_top::FindingReply { result: Ok(forge_top::FindingDone::Dropped), .. })))
    );
    assert_eq!(findings(&world)[0].state, forge_top::FindingState::Dropped);
    assert!(!findings(&world)[0].erasable());
    fake::advance(&mut world.fake, &world.fake_env, b"org/repo", b"main", b"file", b"unchanged failures", 1).unwrap();
    until(&mut world, RUN_TICKS * 3, "dropped red still holds", |world| {
        tier(world).record.sequence == 3 && tier(world).posting.is_none()
    });
    assert_eq!(findings(&world).len(), 1);
    assert_eq!(findings(&world)[0].number, finding.number);
    assert!(findings(&world)[0].uncleared_red());
    fake::advance(&mut world.fake, &world.fake_env, b"org/repo", b"main", b"failures", b"", 1).unwrap();
    until(&mut world, RUN_TICKS * 2, "run passes and clears dropped red", |world| {
        tier(world).record.sequence == 4 && tier(world).posting.is_none()
    });
    let cleared = findings(&world)[0];
    assert_eq!(cleared.state, forge_top::FindingState::Dropped);
    assert_eq!(cleared.cleared.unwrap().run, 4);
    assert!(cleared.erasable());
    let answer = world.scripted_call(
        &triager,
        402,
        engine::Tool::Finding(engine::FindingCall::List { state: None, mine: true }),
    );
    let engine::CallAnswer::Finding(answer) = answer else { panic!("mine list") };
    let engine::FindingToolAnswer::List(list) = answer.as_ref() else { panic!("mine document") };
    assert!(list.rows.iter().all(|row| row.number != finding.number));
}
