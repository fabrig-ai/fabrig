//! Failed section reads in a repair's paired brief (temper/docs/design/domain/forge/change/repairs.md).
use super::*;
use jig_domain_core::{documents, reads};
use skein_forge::api::Error;

const STEPS: u32 = 200;
const REASON: &[u8] = b"ci: the forge's answer did not match its read";

fn world() -> (World, engine::Assignment, engine::Assignment, Box<[u8]>) {
    // The shipped brief has room for both the pull identity and its unread marker.
    let small = temper_service::limits::profile(temper_service::limits::Profile::Small);
    let world = World::configured_limits(
        true,
        true,
        false,
        0,
        Gate::default(),
        None,
        StoryLimits { brief: Some(small.root.brief), result_bytes: Some(512), ..StoryLimits::default() },
    );
    change_world_in(world, Change::default())
}

fn facts(world: &mut World) -> Vec<engine::Fact> {
    let mut facts = Vec::new();
    while let Some(fact) = world.root.pop_fact() {
        facts.push(fact);
    }
    facts
}

fn pull_words(assigned: &engine::Assignment) -> &[u8] {
    assigned
        .sections
        .iter()
        .find_map(|section| match &section.body {
            engine::BriefBody::Text(words)
                if section.kind == engine::BriefKind::Forge(engine::ForgeBriefKind::Pull) =>
            {
                Some(words.as_ref())
            }
            _ => None,
        })
        .expect("pull section")
}

fn start(world: &mut World, producer: &engine::Assignment, branch: &[u8]) {
    let checks: &[&[u8]] = &[b"build"];
    assert!(matches!(
        world.external(2, raw::Op::Git(raw::Git::Create { branch: Box::from(branch), commit: 1 })),
        raw::Answer::Branch(raw::Created::Created)
    ));
    let head = fake::advance(&mut world.fake, &world.fake_env, b"org/repo", branch, b"file", b"first", 1)
        .expect("producer pushed its branch");
    for context in checks {
        world.external(
            1,
            raw::Op::Write(raw::Write::Status {
                commit: head,
                context: Box::from(*context),
                state: raw::Check::Failed,
                description: Box::new([]),
                link: Box::new([]),
            }),
        );
    }
    let event = world.protocol.answer(
        Token::new(7),
        producer.task,
        producer.attempt,
        5,
        tasks::End::Finished {
            result: tasks::TaskResult::Change {
                title: jig_domain_tasks::ChangeTitle::new(b"Small fix".as_slice().into()),
                connector: 0,
                kind: 2,
                resource: temper_engine_domain_forge::tag(forge_world::REPO.forge, forge_world::REPO.repository),
                words: Box::from(&b"pushed"[..]),
            },
            cancel_delegates: false,
        },
        engine::Parts::none(),
    );
    world.send(event);
}

fn repair(world: &World) -> Option<&tasks::TaskRecord> {
    world.store.rows.values().find_map(|stored| match stored {
        Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(row)))
            if row.spec.words.starts_with(b"Repair CI at ") =>
        {
            Some(row.as_ref())
        }
        _ => None,
    })
}

#[test]
fn a_repair_whose_pull_read_answers_wrongly_starts_with_words() {
    let (mut world, _chat, producer, branch) = world();
    world.fail_brief_reads = Some((true, false, Error::InvalidAnswer));
    start(&mut world, &producer, &branch);
    for _ in 0..STEPS {
        world.tick();
        if world.assigned.len() >= 3 {
            break;
        }
    }
    let assigned = world.assigned.get(2).expect("one unread pull leaves the CI repair assignable");
    let pull = pull_words(assigned);
    assert!(contains(pull, b"Pull request: #"));
    assert!(
        contains(pull, b"[the pull request could not be read: the forge's answer did not match its read]"),
        "{}",
        String::from_utf8_lossy(pull)
    );
    assert!(contains(ci_words(assigned), b"Failed step: 1 Run build\nEnd of its log:\n"));
    assert!(contains(ci_words(assigned), BUILD_LOG_END));
    let task = assigned.task;
    let facts = facts(&mut world);
    assert_eq!(facts.iter().filter(|fact| matches!(fact, engine::Fact::BriefReadRefused { .. })).count(), 1);
    assert!(facts.contains(&engine::Fact::BriefReadRefused {
        task,
        section: b"pull",
        stage: "item",
        reason: "the forge's answer did not match its read",
        heads: None,
        error: Some(Error::InvalidAnswer)
    }));
    assert!(!facts.iter().any(|fact| matches!(fact, engine::Fact::BriefFailed { .. })));
    world.close();
}

#[test]
fn a_repair_with_neither_section_ends_held_naming_the_reason() {
    let (mut world, _chat, producer, branch) = world();
    world.fail_brief_reads = Some((true, true, Error::InvalidAnswer));
    start(&mut world, &producer, &branch);
    for _ in 0..STEPS {
        world.tick();
        if repair(&world).is_some_and(|row| row.retry_notice.is_some()) {
            break;
        }
    }
    let row = repair(&world).expect("repair task");
    assert!(
        matches!(&row.retry_notice, Some(tasks::RetryNotice::Failed { class: tasks::Class::Permanent, count: 1, allowance, reason: Some(reason) })
        if reason.as_ref() == REASON && *allowance == world.env.limits.tasks.retries.permanent.retries),
        "{:?}",
        row.retry_notice
    );
    assert_eq!(row.attempt, 0);
    let task = row.number;
    let allowance = world.env.limits.tasks.retries.permanent.retries;
    let mut observed = facts(&mut world);
    for _ in 0..STEPS {
        world.tick();
        observed.extend(facts(&mut world));
        if matches!(
            live_task(&world, task).phase,
            tasks::Phase::Held { why: tasks::Hold::Failures(tasks::Class::Permanent), .. }
        ) {
            break;
        }
    }
    let row = live_task(&world, task);
    assert!(
        matches!(&row.phase, tasks::Phase::Held { why: tasks::Hold::Failures(tasks::Class::Permanent), source: Some(source), .. }
        if matches!(source.as_ref(), tasks::HoldSource::Preparation { words } if words.as_ref() == REASON)),
        "{:?}",
        row.phase
    );
    assert_eq!(row.attempt, 0);
    assert_eq!(row.tries.permanent, allowance + 1);
    assert_eq!(world.assigned.len(), 2, "no attempt starts with neither forge section");
    let revision = row.revisions.hold;
    let tasks::Party::Task(change) = row.requester else { panic!("repair belongs to change") };
    assert_eq!(observed.iter().filter(|fact| matches!(fact, engine::Fact::BriefFailed { task: failed, counts: "permanent", reason } if *failed == task && reason.as_ref() == REASON)).count(), usize::try_from(allowance + 1).expect("small retry allowance"));
    assert_eq!(
        observed
            .iter()
            .filter(|fact| matches!(fact, engine::Fact::BriefReadRefused { task: failed, .. } if *failed == task))
            .count(),
        usize::try_from(2 * (allowance + 1)).expect("two failed reads per brief")
    );
    let reads::Answer::Document(document) = world.party_read(reads::Read::Task { task }) else {
        panic!("repair document")
    };
    let documents::Document::Task(document) = *document else { panic!("task document") };
    let hold = document.hold.expect("held preparation");
    assert_eq!(hold.reason, tasks::Hold::Failures(tasks::Class::Permanent));
    assert_eq!(hold.words.expect("fixed preparation reason").bytes.as_ref(), REASON);
    let change_row = super::questions::row(&world, change);
    assert!(matches!(change_row.change.state, temper_engine_domain_forge_change::State::Repairing { .. }));
    let panel = world.oracle_document(world.signed_in.expect("owner"), 1, change).expect("change panel");
    let forge_top::Document::Change(panel) = panel else { panic!("change panel") };
    assert_eq!(panel.step, forge_top::PanelStep::Repairing(temper_engine_domain_forge_change::Repair::Ci));
    assert_eq!(panel.hold, None, "the repair's failure hold is independent of the change's stall");
    world.fail_brief_reads = None;
    world.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(Token::new(94)),
        sign_in: world.signed_in.expect("owner"),
        key: [94; 16],
        ask: people::Ask::Release { project: 1, task, revision, again: None },
    }));
    for _ in 0..STEPS {
        world.tick();
        if world.assigned.len() >= 3 {
            break;
        }
    }
    assert_eq!(world.assigned.get(2).expect("repair starts after its release").task, task);
    world.close();
}

#[test]
fn a_repair_whose_checks_cannot_be_read_starts_on_its_pull() {
    let (mut world, _chat, producer, branch) = world();
    world.fail_brief_reads = Some((false, true, Error::InvalidAnswer));
    start(&mut world, &producer, &branch);
    for _ in 0..STEPS {
        world.tick();
        if world.assigned.len() >= 3 {
            break;
        }
    }
    let assigned = world.assigned.get(2).expect("readable pull leaves the CI repair assignable");
    assert!(contains(pull_words(assigned), b"Pull request: #"));
    assert!(contains(pull_words(assigned), b"Title: Small fix"));
    assert!(contains(
        ci_words(assigned),
        b"[the failed checks could not be read: the forge's answer did not match its read]"
    ));
    assert!(live_task(&world, assigned.task).spec.words.starts_with(b"Repair CI at 975835de1c97: build\n"));
    let task = assigned.task;
    let facts = facts(&mut world);
    assert!(facts.contains(&engine::Fact::BriefReadRefused {
        task,
        section: b"ci",
        stage: "checks",
        reason: "the forge's answer did not match its read",
        heads: None,
        error: Some(Error::InvalidAnswer)
    }));
    assert!(!facts.iter().any(|fact| matches!(fact, engine::Fact::BriefFailed { .. })));
    world.close();
}

#[test]
fn a_repair_with_the_forge_not_connected_pauses() {
    repair_pauses(Error::NotConnected, b"pull: the forge is not connected");
}

#[test]
fn a_repair_with_the_forge_away_pauses_without_spending_a_try() {
    for (error, reason) in [
        (Error::Unavailable, b"pull: the forge could not be reached".as_slice()),
        (Error::Timeout, b"pull: the forge did not answer in time".as_slice()),
    ] {
        repair_pauses(error, reason);
    }
}

fn repair_pauses(error: Error, reason_words: &[u8]) {
    let (mut world, _chat, producer, branch) = world();
    world.fail_brief_reads = Some((true, false, error));
    start(&mut world, &producer, &branch);
    let allowance = world.env.limits.tasks.retries.permanent.retries;
    let mut observed = Vec::new();
    for _ in 0..STEPS {
        world.tick();
        observed.extend(facts(&mut world));
        if repair(&world).is_some_and(
            |row| matches!(row.retry_notice, Some(tasks::RetryNotice::Refused { count, .. }) if count > allowance),
        ) {
            break;
        }
    }
    let row = repair(&world).expect("repair task");
    assert!(
        matches!(&row.retry_notice, Some(tasks::RetryNotice::Refused { count, reason: Some(reason) }) if *count > allowance && reason.as_ref() == reason_words),
        "{:?}",
        row.retry_notice
    );
    assert!(!matches!(row.phase, tasks::Phase::Held { .. }));
    assert_eq!(row.tries.permanent, 0);
    assert_eq!(row.tries.transient, 0);
    assert_eq!(row.attempt, 0);
    assert_eq!(world.assigned.len(), 2);
    let task = row.number;
    assert!(observed.iter().any(|fact| matches!(fact, engine::Fact::BriefFailed { task: failed, counts: "capacity", reason } if *failed == task && reason.as_ref() == reason_words)));
    world.fail_brief_reads = None;
    for _ in 0..STEPS {
        world.tick();
        if world.assigned.len() >= 3 {
            break;
        }
    }
    assert_eq!(world.assigned.get(2).expect("repair starts once connected").task, task);
    world.close();
}
