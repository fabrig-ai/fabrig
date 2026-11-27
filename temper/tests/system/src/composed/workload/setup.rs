//! Settings and authentic party entrances for workloads (workloads.md, 5).
use super::{Goal, Party, REMOTE, World, engine, parties, people, scripts, tasks};
use skein_lib::{Duration, Map};
use temper_engine_domain_forge::{self as forge, settings};
use temper_fake_checkout::git::Remote;

pub(super) const SECOND_REPLY: u64 = 2000;

pub(super) fn party(number: u64) -> parties::Party {
    parties::Party::new(
        1,
        parties::Role::Owner,
        if number == 1 { 400 } else { SECOND_REPLY },
        Box::new([parties::Act::SignIn {
            provider: 0,
            subject: forge::user_key(1, if number == 1 { 7 } else { 8 }).subject,
        }]),
    )
}

pub(super) fn marker(change: &temper_workload::draw::Change) -> String {
    format!(".workload/{}.tests", change.name)
}

pub(super) fn config(seed: u64, workload: &temper_workload::draw::Workload) -> engine::Config {
    let mut config = super::super::agreement::config(seed);
    for project in &mut config.core.seed_projects {
        for role in &mut project.policy.policy.roles {
            role.period_spend = config.core.defaults.rules.period_spend / 2;
        }
    }
    for charter in &mut config.core.defaults.charters {
        if charter.number == 2 {
            charter.policy.tools.0 |= jig_domain_core::ToolFamilies::JIG.delegate.0;
        }
    }
    for template in &mut config.core.defaults.templates {
        if template.number == 2 {
            template.executor = people::TemplateExecutor::Charter(1);
            template.flags.ended_by = tasks::EndedBy::Closer(tasks::Closer { connector: 1, code: 6 });
        }
    }
    config.core.jobs = Box::new([temper_service_core::limits::prepare_bound()]);
    let rule = engine::LandingRule {
        sets: Box::default(),
        connector: 1,
        kind: 4,
        enforce: true,
        ci: false,
        up_to_date: false,
        pattern: jig_domain_authority::Pattern {
            segments: Box::new([b"forge".as_slice().into()]),
            last: jig_domain_authority::Last::Open(Box::default()),
        },
        gates: Box::default(),
        approvals: Box::default(),
    };
    config.kinds_stock = engine::landing_stock(&[rule], &Map::with_capacity(0), 2, 4);
    let landing = temper_engine_domain_world::oracle::config::landing(&mut config, 1);
    landing.by = temper_engine_domain_forge_change::Way::Turn;
    landing.batch = 3;
    landing.wait = temper_engine_domain_forge_change::Wait::Landed;
    landing.combine_main = engine::Combine::Merge;
    landing.combine_other = engine::Combine::Merge;
    let value = settings::Value::Gates(settings::ProjectGates {
        repositories: Box::new([forge::RepositoryGates {
            repository: temper_engine_forge_world::REPO,
            gates: workload
                .goals
                .iter()
                .flat_map(|goal| &goal.changes)
                .map(|change| temper_landing_fixtures::GateCheck {
                    name: change.name.replace('.', "_").into_bytes().into(),
                    workflow: format!("{}.yml", change.name).into_bytes().into(),
                    ignored: Box::default(),
                    blocking: true,
                    base: false,
                    report: Some(temper_engine_domain_forge_landing::GateReport {
                        artifact: b"report".as_slice().into(),
                        format: temper_engine_domain_forge_tiers::Format::Lines,
                        fuzzy: None,
                    }),
                })
                .collect(),
            tiers: Box::new([forge::Tier {
                name: b"after".as_slice().into(),
                workflow: b"after.yml".as_slice().into(),
                ignored: Box::default(),
                artifact: b"report".as_slice().into(),
                format: temper_engine_domain_forge_tiers::Format::Junit,
                follows: forge::Follows::Landing,
                span: None,
                description: Box::default(),
            }]),
        }]),
    });
    let mut stock = config.kinds_stock.into_vec();
    stock.push(settings::Stock {
        kind: 5,
        scope: 1,
        number: 1,
        value: settings::SizedValue { size: temper_engine_smith::kinds::size(&value), value },
    });
    config.kinds_stock = stock.into();
    config
}

pub(super) fn limits(
    limits: &mut engine::Limits,
    planned: &[skein_world::faults::Kind],
    workload: &temper_workload::draw::Workload,
    close_refusal: bool,
) {
    use temper_faults::bounds;

    if planned.contains(&bounds::FORGE_READ_ADMISSIONS) {
        // Two admitted reads make COMPOSED reach this table while the
        // other readers queue; the stock write table does not fill.
        limits.forge.client.pending = 2;
    } else if close_refusal {
        // Leave room for the capability refresh when a close is refused
        // alongside fresh gathers from released changes.
        limits.forge.client.pending += 1;
    }
    if planned.contains(&bounds::BRANCH_HOLDS) {
        // Six holds for a three-member train, two for a concurrent milestone
        // change, and five for the goal branch and saved-work branches.
        limits.forge.holds = 13;
    }
    if planned.contains(&bounds::LANDING_ROWS) {
        // Keep one row per milestone change and closer, one for the train,
        // and one for a held member landing after its release.
        let changes: usize = workload.goals.iter().filter(|goal| !goal.train).map(|goal| goal.changes.len()).sum();
        limits.forge.landings = u32::try_from(changes + workload.goals.len() + 1).expect("bounded landing rows");
    }
    // Activations take turns on the composed world's single Smith process.
    limits.fleet.slots = 1;
    // COMPOSED has at most 2 + 2 + 3 named changes, each with a tree-cued gate.
    limits.forge.gates = limits.forge.gates.max(7);
}

pub(super) fn forge(world: &mut World) {
    use skein_fake_forge_domain::api as f;
    let mut checks = super::super::agreement::checks();
    // Turn landings dispatch the tree-cued gates; pushes run no CI.
    checks.contexts = Box::default();
    let tree = world.forge.tree(world.forge.branch(REMOTE, b"main").expect("main"));
    let mut limits = temper_worker_checkout_world::forge::LIMITS;
    limits.workflows = 8;
    limits.contexts = 16;
    limits.workflow_runs = limits.workflow_runs.max(256);
    limits.items = limits.items.max(32);
    limits.statuses = limits.statuses.max(256);
    limits.observations = limits.observations.max(1024);
    world.forge = super::super::Forge::with_limits(world.seed, limits);
    let mut workflows: Vec<_> = world
        .workload()
        .goals
        .iter()
        .flat_map(|goal| &goal.changes)
        .map(|change| f::Workflow {
            file: format!("{}.yml", change.name).into_bytes().into(),
            jobs: Box::new([b"test".as_slice().into()]),
            requirements: Box::default(),
            fails: Box::default(),
            flaky: Box::default(),
            select: None,
            report: Some(f::Report {
                artifact: b"report".as_slice().into(),
                format: f::ReportFormat::Lines,
                deflate: false,
            }),
            latency_min: Duration::from_secs(1),
            latency_max: Duration::from_secs(1),
            silent: 0,
            passes: 1000,
            cue: Some(f::Cue::Tests { path: marker(change).into_bytes().into() }),
        })
        .collect();
    workflows.push(f::Workflow {
        file: b"after.yml".as_slice().into(),
        jobs: Box::new([b"test".as_slice().into()]),
        requirements: Box::default(),
        fails: Box::default(),
        flaky: Box::default(),
        select: None,
        report: Some(f::Report {
            artifact: b"report".as_slice().into(),
            format: f::ReportFormat::Junit,
            deflate: false,
        }),
        latency_min: Duration::from_secs(1),
        latency_max: Duration::from_secs(1),
        silent: 0,
        passes: 1000,
        cue: None,
    });
    world.forge.repository_with_workflows(REMOTE, b"main", tree, checks, workflows.into());
    world.forge.grant(REMOTE, 1, f::Permission::Admin);
    world.forge.grant(REMOTE, 7, f::Permission::Read);
    world.forge.grant(REMOTE, 8, f::Permission::Read);
}

pub(super) fn milestone(world: &World) -> people::Ask {
    let people::Ask::Create { project, mut batch } = world.chat_request(1, b"The workload milestone") else {
        unreachable!()
    };
    let member = &mut batch[0];
    member.executor =
        people::TaskExecutor::Procedure { connector: jig_domain_core::RECURRING, code: jig_domain_core::GATHERING };
    member.contract = people::Contract::Report { words: 128 };
    member.flags = people::Flags { listed: true, ended_by: people::EndedBy::Requester, ..people::Flags::PLAIN };
    member.tracked = Some(1);
    member.label = 4;
    member.authority.spend = 500;
    member.budget = people::Budget::Named;
    people::Ask::Create { project, batch }
}

pub(super) fn goal(world: &World, goal: &Goal, milestone: u64) -> people::Ask {
    let words = format!("{} Implement this workload goal.", scripts::goal_cue(goal));
    let people::Ask::Create { project, mut batch } = world.chat_request(1, words.as_bytes()) else { unreachable!() };
    let member = &mut batch[0];
    member.executor = people::TaskExecutor::Agent { charter: 1 };
    member.contract = people::Contract::Report { words: 1024 };
    member.flags = people::Flags {
        listed: true,
        ended_by: people::EndedBy::Closer(people::Closer { connector: 1, code: 6 }),
        ..people::Flags::PLAIN
    };
    member.tracked = Some(1);
    member.label = 2;
    member.symbols = Box::new([people::Symbol { connector: 1, symbol: 1 }]);
    member.authority.spend = 150;
    // temper/docs/design/domain/forge/branches.md, 2: reserve twelve tasks
    // for (two repairs + the change) * (three landing attempts + one),
    // beside the original work and closer. Both COMPOSED changes may be red.
    member.authority.delegation.tasks = 24;
    member.authority.delegation.depth = 4;
    member.budget = people::Budget::Named;
    member.llm = Some(people::LlmChoice {
        account: Some(account(goal.party)),
        main: people::ModelEffort { model: 0, effort: None },
        subagents: Box::default(),
        later: None,
    });
    member.dependencies = goal
        .after
        .iter()
        .map(|name| people::Dependency::Existing(world.workload_names().task(name).expect("earlier milestone goal")))
        .collect();
    people::Ask::Delegate { project, task: milestone, batch }
}

pub(super) fn account(party: Party) -> u32 {
    match party {
        Party::First => super::super::accounts::FIRST_ACCOUNT,
        Party::Second => super::super::accounts::OTHER_ACCOUNT,
    }
}

pub(super) fn tier_green(world: &World) -> bool {
    let key =
        Key::Forge(forge::Key::Tier { repository: temper_engine_forge_world::REPO, tier: b"after".as_slice().into() });
    let Some(engine::Record::Forge { row }) = world.store.rows.get(&key) else { return false };
    let forge::Stored::Tier(row) = row.as_ref() else { unreachable!() };
    row.record.run.is_none()
        && row.record.sequence > 0
        && matches!(&row.record.failing, temper_engine_domain_forge_tiers::Failures::Tests(tests) if tests.is_empty())
        && row.record.judged.as_ref().map(|judged| judged.tip)
            == world.forge.branch(REMOTE, b"main").map(temper_engine_forge_world::translate::commit)
}
use super::Key;
