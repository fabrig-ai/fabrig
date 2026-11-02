use super::{commit_text, landing_words, news_words};
use alloc::boxed::Box;

const SHA1: [u8; 32] = [
    0xab, 0xcd, 0xef, 0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef, 0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef,
    0x01, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];
const SHA256: [u8; 32] = [0xab; 32];
const COMMITS: &[([u8; 32], &[u8])] = &[
    (SHA1, b"abcdef0123456789abcdef0123456789abcdef01"),
    (SHA256, b"abababababababababababababababababababababababababababababababab"),
];

#[test]
fn commit_text_uses_lowercase_digits_at_the_identifier_width() {
    for (commit, digits) in COMMITS {
        assert_eq!(commit_text(commit).as_ref(), *digits, "commit {commit:?}");
    }
}

#[test]
fn forge_news_prints_commit_ids_at_the_identifier_width_within_its_bound() {
    use super::forge::News;
    for (commit, digits) in COMMITS {
        for (news, prefix, suffix) in [
            (
                News::Ci { head: *commit, status: skein_forge::api::Ci::Passed },
                b"forge ci passed at ".as_slice(),
                b"".as_slice(),
            ),
            (
                News::Landing { before: *commit, after: SHA1, files: None },
                b"forge landing ".as_slice(),
                b" -> abcdef0123456789abcdef0123456789abcdef01".as_slice(),
            ),
        ] {
            let mut expected = skein_lib::Writer::new(prefix.len() + digits.len() + suffix.len());
            expected.put(prefix).unwrap();
            expected.put(digits).unwrap();
            expected.put(suffix).unwrap();
            let expected = expected.finish();
            let bound = u32::try_from(expected.len()).unwrap();
            assert_eq!(news_words(&news, bound), expected);
            assert_eq!(news_words(&news, bound - 1).as_ref(), &expected[..expected.len() - 1]);
        }
    }
}

#[test]
fn landing_words_are_the_merge_id_cut_to_the_contract_bound() {
    for (merge, bound, expected) in [
        (SHA1, 64, COMMITS[0].1),
        (SHA1, 40, COMMITS[0].1),
        (SHA256, 512, COMMITS[1].1),
        (SHA256, 64, COMMITS[1].1),
        (SHA256, 32, &COMMITS[1].1[..32]),
        (SHA1, 7, &b"abcdef0"[..]),
        (SHA1, 0, &b""[..]),
    ] {
        assert_eq!(landing_words(&merge, bound, None, 1, None).as_ref(), expected, "merge {merge:?}, bound {bound}");
    }
}

#[test]
fn not_installed_ends_the_roots_adoption_part_as_missing_access() {
    use jig_domain_people as people;
    assert_eq!(
        super::adoption_failure(skein_forge::api::Error::NotInstalled),
        people::Outcome::Refused(people::Refusal::Authority {
            lacking: people::Lack::ResourceAccess,
            proposable: None
        })
    );
    assert_eq!(super::adoption_failure(skein_forge::api::Error::Busy), people::Outcome::Again(people::Again::Busy));
}

fn adopted(forge: u16) -> super::forge::Repository {
    super::forge::Repository {
        project: 5,
        home: forge == 1,
        provider: skein_forge::api::Repository { forge, repository: 1 },
        host: Box::from(&b"forge.example"[..]),
        owner: Box::from(&b"org"[..]),
        name: Box::from(&b"repo"[..]),
        prefix: Box::from(&b"temper/"[..]),
        role: super::forge::Role::Owned,
        ci: true,
        checks: Box::new([]),
        kinds: super::forge::Kinds {
            read: true,
            push: true,
            open: true,
            land: true,
            review: true,
            status: true,
            comment: true,
            issue: true,
            branch: true,
        },
        protection: super::forge::Protection::Absent,
        settings: skein_forge::api::Settings {
            default_branch: Box::from(&b"main"[..]),
            merge: true,
            squash: true,
            rebase: false,
        },
    }
}

#[test]
fn tree_branches_count_prefix_slashes_against_authority_segments() {
    // forge, host, owner, repository, branch, temper, root, marker/task.
    const AUTHORITY_SEGMENTS: u32 = 8;
    // The old live prefix puts live-x in a branch segment of its own.
    const DEEP_AUTHORITY_SEGMENTS: u32 = 9;
    const BRANCH_BYTES: u32 = 128;
    for (prefix, segments, fits) in [
        (b"temper/live-x/".as_slice(), AUTHORITY_SEGMENTS, false),
        (b"temper/live-x/", DEEP_AUTHORITY_SEGMENTS, true),
        (b"temper/", AUTHORITY_SEGMENTS, true),
        (b"temper/live-x-", AUTHORITY_SEGMENTS, true),
    ] {
        let mut repository = adopted(1);
        repository.prefix = prefix.into();
        for marker in [b'c', b'r', b's'] {
            for attempt in [None, Some(3)] {
                let branch = super::tree_branch(&repository, 1, marker, 2, attempt, BRANCH_BYTES).expect("tree branch");
                let what = super::branch_what(&branch, BRANCH_BYTES).expect("branch parts");
                assert_eq!(super::resource_name(&repository, &what, segments).is_some(), fits, "branch {branch:?}");
            }
        }
    }
}

#[test]
fn delegations_with_one_repository_id_take_their_own_forges_holding() {
    use super::{forge, tasks};
    use alloc::boxed::Box;
    let (mut domain, env) = two_forges();
    let connector = domain.config.forge_connector;
    for number in [1, 3] {
        let spec = tasks::Spec {
            job: None,
            asks: false,
            words: Box::new([]),
            inputs: Box::new([]),
            parameters: Box::new([tasks::Parameter::Resource { name: 1, connector, resource: forge::tag(number, 1) }]),
        };
        let holdings = super::task_holdings(
            &mut domain,
            &env,
            5,
            7,
            8,
            tasks::Executor::Procedure { connector, code: 2 },
            &spec,
            None,
        )
        .expect("delegation admitted");
        let [tasks::Holding::Write { resource, .. }] = holdings.as_ref() else { panic!("one write holding") };
        let name = super::forge_name(connector, resource, env.limits.forge.name_bytes).expect("forge holding");
        assert_eq!((name.forge, name.repository), (number, 1));
    }
}

#[test]
fn saved_resources_keep_the_forge_in_their_path() {
    use super::forge;
    let tags = [forge::tag(1, 1), forge::tag(3, 1)];
    let resources = super::saved_resources(4, &tags).expect("valid tags");
    assert_ne!(resources[0].path, resources[1].path);
    assert_eq!(
        resources[0].path.as_ref(),
        &[alloc::boxed::Box::from(1_u16.to_be_bytes()), alloc::boxed::Box::from(1_u32.to_be_bytes())]
    );
    let decoded = super::saved_tags(&resources, 4).expect("valid saved paths");
    assert_eq!(decoded.as_ref(), tags.as_slice());
    let (domain, _) = two_forges();
    for (tag, number) in decoded.iter().zip([1, 3]) {
        assert_eq!(domain.forge.repository_tag(5, *tag).expect("own repository").provider.forge, number);
    }
    assert!(super::saved_resources(4, &[0x0001_0001_0000_0001]).is_none());
    let mut malformed = resources.clone();
    malformed[0].path[0] = alloc::boxed::Box::from(1_u32.to_be_bytes());
    assert!(super::saved_tags(&malformed, 4).is_none(), "a forge wider than u16 is invalid");
}

fn two_forges() -> (super::Domain, skein_lib::Env<super::Limits>) {
    use super::forge;
    let mut config = crate::tests::resume_fixture::config(7);
    let first = config.forges.first().expect("configured fixture forge").clone();
    let third = forge::Configured { number: 3, ..first.clone() };
    config.forges = Box::new([first, third]);
    let (mut domain, env) = crate::tests::resume_fixture::root_with_config(config);
    domain.forge.restore_begin(env.wall);
    let mut out = skein_lib::Queue::uncapped();
    for number in [1, 3] {
        forge::step(
            &mut domain.forge,
            &skein_lib::Env { now: env.now, wall: env.wall, limits: env.limits.forge },
            forge::Event::Restore {
                record: forge::Stored::Forge(forge::ForgeRecord {
                    git_program: None,
                    number,
                    kind: forge::Kind::Forgejo,
                    mode: forge::Mode::Connected,
                    state: forge::ForgeState::Kept,
                    place: forge::Place {
                        api: forge::Address { host: b"forge.example".as_slice().into(), port: 443, tls: None },
                        git: Box::new([]),
                        web: None,
                    },
                    read_only: false,
                    bot: forge::Bot {
                        login: b"bot".as_slice().into(),
                        name: b"Bot".as_slice().into(),
                        email: Box::new([]),
                    },
                    asked: false,
                    first: None,
                }),
            },
            &mut out,
        );
        forge::step(
            &mut domain.forge,
            &skein_lib::Env { now: env.now, wall: env.wall, limits: env.limits.forge },
            forge::Event::Restore { record: forge::Stored::Repository(adopted(number)) },
            &mut out,
        );
    }
    (domain, env)
}

#[test]
fn pushed_heads_route_only_to_their_own_forges_hold() {
    use super::{Work, forge};
    let (mut domain, env) = two_forges();
    let forge_env = skein_lib::Env { now: env.now, wall: env.wall, limits: env.limits.forge };
    let mut out = skein_lib::Queue::uncapped();
    for number in [1, 3] {
        let name = forge::Name {
            forge: number,
            repository: 1,
            what: forge::What::Branch(Box::new([Box::from(b"main".as_slice())])),
        };
        forge::step(
            &mut domain.forge,
            &forge_env,
            forge::Event::Restore {
                record: forge::Stored::Hold(forge::Hold {
                    name,
                    task: 7,
                    writer: Some(forge::Writer::Run { task: 8, attempt: 1 }),
                    drift: None,
                }),
            },
            &mut out,
        );
    }
    for number in [1, 3] {
        super::left(&mut domain, &env, 8, 1, Box::new([forge::Pushed { tag: forge::tag(number, 1), commit: [2; 32] }]));
        let Some(Work::Forge(forge::Event::Answered { pushed, .. })) = domain.work.pop() else {
            panic!("reported heads")
        };
        let [(name, commit)] = pushed.as_ref() else { panic!("one matching hold") };
        assert_eq!((name.forge, name.repository, *commit), (number, 1, [2; 32]));
        let Some(Work::Forge(forge::Event::Lost { task: 8, attempt: 1 })) = domain.work.pop() else {
            panic!("writer release follows reported heads")
        };
    }
}

#[test]
fn forge_resource_holds_name_the_branch_holder_and_failed_run() {
    use super::{forge, holds};
    let (domain, _) = two_forges();
    let resource = forge::Name {
        forge: 3,
        repository: 1,
        what: forge::What::Branch(Box::new([Box::from(b"temper".as_slice()), Box::from(b"7".as_slice())])),
    };
    assert!(domain.forge.change(7).is_none(), "an agent task has no change row");
    assert_eq!(holds::taken_words(&domain, &resource, 9).as_ref(), b"branch temper/7 in org/repo is taken by holder 9");
    assert_eq!(
        holds::lost_words(&domain, 7, 4, &resource, b"forge policy refused the operation").as_ref(),
        b"task 7's branch temper/7 in org/repo could not be read again after run 4 ended: forge policy refused the operation"
    );
}

#[test]
fn a_goals_failed_reread_routes_a_resource_hold_with_its_cause() {
    use super::{Work, forge, tasks};
    let (mut domain, env, task) = crate::tests::live_task(None, tasks::Executor::Agent { charter: 1 });
    let resource = forge::Name {
        forge: 1,
        repository: 1,
        what: forge::What::Branch(Box::new([Box::from(b"temper".as_slice()), Box::from(b"goal".as_slice())])),
    };
    let mut out = skein_lib::Queue::uncapped();
    domain.forge.restore_begin(env.wall);
    forge::step(
        &mut domain.forge,
        &skein_lib::Env { now: env.now, wall: env.wall, limits: env.limits.forge },
        forge::Event::Restore { record: forge::Stored::Repository(adopted(1)) },
        &mut out,
    );
    while out.pop().is_some() {}
    assert!(domain.forge.change(task).is_none());
    out.push(forge::Request::LostReadFailed {
        task,
        attempt: 4,
        resource,
        reason: b"forge policy refused the operation",
    });
    let mut decision = super::Decision::decide(&mut domain.journal, &env.limits.journal);
    super::outputs(&mut domain, &env, &mut decision, &mut out);
    let Some(Work::Tasks(event @ tasks::Event::HoldFrom { .. })) = domain.work.pop() else {
        panic!("reread failure holds the goal");
    };
    tasks::step(
        &mut domain.core.tasks,
        &skein_lib::Env { now: env.now, wall: env.wall, limits: env.limits.tasks },
        event,
        &mut skein_lib::Queue::uncapped(),
    );
    let tasks::Phase::Held { why, source: Some(source), .. } = &domain.core.tasks.task(task).unwrap().phase else {
        panic!("resource holds apply to agent tasks");
    };
    assert_eq!(*why, tasks::Hold::ResourceUnavailable);
    let tasks::HoldSource::Account { words, .. } = source.as_ref() else { panic!("retained cause") };
    assert_eq!(
        words.as_ref(),
        b"task 1's branch temper/goal in org/repo could not be read again after run 4 ended: forge policy refused the operation; resolve this reason, then task release 1 --hold 1; or task cancel 1 REASON"
    );
}

#[test]
fn writable_repositories_include_only_the_requested_project() {
    let (domain, _) = two_forges();
    assert_eq!(domain.forge.writable_repositories(5).len(), 2);
    assert!(domain.forge.writable_repositories(1).is_empty());
    assert!(domain.forge.readable_repositories(1).is_empty());
    assert_eq!(domain.forge.readable_repositories(5).len(), 2);
}

#[test]
fn writable_repositories_exclude_context_and_missing_push() {
    use super::forge;
    let (mut domain, env) = two_forges();
    let mut out = skein_lib::Queue::uncapped();
    let mut context = adopted(1);
    context.role = forge::Role::Context;
    let mut read_only = adopted(3);
    read_only.kinds.push = false;
    for repository in [context, read_only] {
        forge::step(
            &mut domain.forge,
            &skein_lib::Env { now: env.now, wall: env.wall, limits: env.limits.forge },
            forge::Event::Restore { record: forge::Stored::Repository(repository) },
            &mut out,
        );
    }
    assert!(domain.forge.writable_repositories(5).is_empty());
    assert_eq!(domain.forge.readable_repositories(5).len(), 2);
}

#[test]
fn writable_repositories_exclude_an_ended_forge() {
    use super::forge;
    let (mut domain, env) = two_forges();
    let mut record = domain.forge.record(1).expect("kept forge").clone();
    record.state = forge::ForgeState::Ended { secrets_erased: false, box_removed: false };
    let mut out = skein_lib::Queue::uncapped();
    forge::step(
        &mut domain.forge,
        &skein_lib::Env { now: env.now, wall: env.wall, limits: env.limits.forge },
        forge::Event::Restore { record: forge::Stored::Forge(record) },
        &mut out,
    );
    let repositories = domain.forge.writable_repositories(5);
    assert_eq!(repositories.len(), 1);
    assert_eq!(repositories[0].provider.forge, 3);
    assert_eq!(domain.forge.readable_repositories(5).len(), 1);
}

fn change_row(pull: Option<u64>) -> super::forge::ChangeRow {
    use super::{forge, forge_change as change};
    forge::ChangeRow {
        task: 7,
        repository: skein_forge::api::Repository { forge: 3, repository: 1 },
        branch: Box::from(b"temper/7".as_slice()),
        base: Box::from(b"main".as_slice()),
        title: Box::from(b"Change".as_slice()),
        body: Box::new([]),
        priority: 0,
        ci: None,
        change: change::Change {
            way: temper_engine_domain_forge_change::Way::Queue,
            landing: None,
            task: 7,
            state: change::State::Producing { requested: false },
            gates: Box::new([]),
            clean: Box::new([]),
            gate_failures: Box::new([]),
            released: None,
            repairs: 0,
            resolutions: 0,
            updates: 0,
            since: skein_lib::Wall::from_nanos(0),
            ready_since: None,
            owns_turn: false,
            last_head: None,
        },
        pull,
        pending: None,
        effect: change::EffectResult::None,
        delegate: None,
        delegate_status: change::Status::Unknown,
        verdicts: Box::new([]),
        result: forge::ChangeResult::empty(),
        gate_remarks: Box::new([]),
        drift: None,
        base_repair: false,
        queue_repair: None,
        queue_repairs: 0,
        producing_charter: 2,
        review_charter: 4,
        effect_account: None,
        producer_failure: None,
    }
}

fn change_names(pull: Option<u64>) -> Box<[super::forge::Name]> {
    use super::forge::{Name, What};
    Box::new([
        Name {
            forge: 3,
            repository: 1,
            what: What::Branch(Box::new([Box::from(b"temper".as_slice()), Box::from(b"7".as_slice())])),
        },
        Name {
            forge: 3,
            repository: 1,
            what: match pull {
                Some(number) => What::Pull(number),
                None => What::Branch(Box::new([Box::from(b"main".as_slice())])),
            },
        },
    ])
}

#[test]
fn a_ci_repair_keeps_its_failed_step_and_log_end_before_pull_contents_and_optional_context() {
    use crate::boundary::Work;
    use jig_domain_brief::{Core, GatherEvent, Planned};
    use skein_lib::{List, Queue, Writer};
    let (mut domain, mut env) = two_forges();
    domain.limits.brief.sections = 8;
    domain.limits.brief.brief_bytes = 14_592;
    domain.limits.brief.budgets.pull = 4608;
    domain.limits.brief.budgets.ci = 4096;
    env.limits.brief = domain.limits.brief;
    let mut row = change_row(Some(7));
    row.change.last_head = Some(SHA1);
    row.delegate = Some((8, super::forge_change::Delegate::Repair(super::forge_change::Repair::Ci)));
    super::forge::step(
        &mut domain.forge,
        &super::super::environment_forge(&env),
        super::forge::Event::Restore { record: super::forge::Stored::Change(row) },
        &mut Queue::uncapped(),
    );
    let mut wanted = List::with_capacity(8);
    for (kind, size, priority, required) in [
        (Core::Task, 4608, 0, true),
        (Core::Results, 128, 1, true),
        (Core::TranscriptTail, crate::TRANSCRIPT_TAIL_BYTES, 2, true),
        (Core::Plan, 128, 3, false),
        (Core::Attempts, 128, 4, false),
        (Core::NotesIndex, 64, 4, false),
    ] {
        let mut text = Writer::new(usize::try_from(size).unwrap());
        for _ in 0..size {
            text.put(b"x").unwrap();
        }
        wanted.push(Planned::Core { kind, text: text.finish(), limit: size, priority, required }).unwrap();
    }
    crate::assemble::finish_brief_plan(&mut domain, &env, 8, Some(7), wanted);
    let mut gathered = None;
    while let Some(work) = domain.work.pop() {
        if let Work::Brief(GatherEvent::Plan { budget, sections, .. }) = work {
            gathered = Some((budget, sections));
        }
    }
    let (budget, mut sections) = gathered.expect("repair plan");
    let mut pull = Writer::new(4608);
    let identity = b"Pull request: #7\nBranch: temper/7\nBase: main\nHead: abcdef0123456789abcdef0123456789abcdef01\nAddress: https://forge.example/org/repo/pulls/7\nTitle: Change\nBody: ";
    pull.put(identity).unwrap();
    for _ in identity.len()..4608 {
        pull.put(b"p").unwrap();
    }
    let pull = pull.finish();
    let mut ci = Writer::new(4096);
    let failure = b"Failed check: / changes (pull_request)\nDescription: CI failed\nLink: /actions/runs/2/jobs/1\nFailed step: 1 changes\nEnd of its log:\n";
    let end = b"\nMissing changes/2.md\nRepair the failed check.\n";
    ci.put(failure).unwrap();
    for _ in failure.len()..4096 - end.len() {
        ci.put(b"l").unwrap();
    }
    ci.put(end).unwrap();
    let ci = ci.finish();
    for section in &mut sections {
        if let Planned::Connector { kind, size, .. } = section {
            *size = u32::try_from(if *kind == 3 { pull.len() } else { ci.len() }).unwrap();
        }
    }
    let Planned::Connector { kind: 3, .. } = sections[6] else { panic!("pull precedes its reason") };
    let Planned::Connector { kind: 1, .. } = sections[7] else { panic!("CI follows the pull") };
    let plan = jig_domain_brief::plan(&sections, budget).expect("bounded repair");
    for placed in &plan.order {
        let section = &sections[usize::try_from(placed.index).unwrap()];
        match section {
            Planned::Connector { kind: 1, .. } => {
                assert_eq!(placed.size, 4096, "the repair's CI budget survives the maximal pull");
                let kept = &ci[..usize::try_from(placed.size).unwrap()];
                assert!(kept.starts_with(failure));
                assert!(kept.ends_with(end));
            }
            Planned::Connector { kind: 3, .. } => assert_eq!(placed.size, 3392, "pull contents yield first"),
            Planned::Core { limit, .. } => assert_eq!(placed.size, *limit, "other context still fits"),
            Planned::Connector { .. } => panic!("two repair sections"),
        }
    }
    assert_eq!(plan.size, budget);
}

#[test]
fn a_github_repair_prepares_when_the_ci_ceiling_fills_the_brief() {
    use crate::boundary::{ForgeBriefKind, Work};
    use jig_domain_brief::{Core, GatherDomain, GatherEvent, GatherRequest, Planned, gather_step};
    use skein_lib::{Env, Id, List, Queue};
    let (mut domain, mut env) = two_forges();
    domain.limits.brief.sections = 3;
    domain.limits.brief.read_bytes = 512;
    domain.limits.brief.brief_bytes = 512;
    domain.limits.brief.budgets.pull = 512;
    domain.limits.brief.budgets.ci = 512;
    env.limits.brief = domain.limits.brief;
    let mut row = change_row(Some(7));
    row.change.last_head = Some(SHA1);
    row.delegate = Some((8, super::forge_change::Delegate::Repair(super::forge_change::Repair::Ci)));
    super::forge::step(
        &mut domain.forge,
        &super::super::environment_forge(&env),
        super::forge::Event::Restore { record: super::forge::Stored::Change(row) },
        &mut Queue::uncapped(),
    );
    let mut wanted = List::with_capacity(3);
    wanted
        .push(Planned::Core {
            kind: Core::Task,
            text: b"Repair failed GitHub check check.\n".as_slice().into(),
            limit: 512,
            priority: 0,
            required: true,
        })
        .unwrap();
    crate::assemble::finish_brief_plan(&mut domain, &env, 8, Some(7), wanted);
    let limits = crate::limits::brief_limits(&env.limits.brief);
    let gather_env = Env { now: env.now, wall: env.wall, limits };
    let mut gather = GatherDomain::new(&limits);
    let mut out = Queue::uncapped();
    while let Some(work) = domain.work.pop() {
        if let Work::Brief(plan @ GatherEvent::Plan { .. }) = work {
            gather_step(&mut gather, &gather_env, plan, &mut out);
        }
    }
    let pull = b"Pull request: #7\nBranch: temper/7\nBase: main\nHead: abcdef0123456789abcdef0123456789abcdef01\nAddress: https://github.com/org/repo/pull/7\n";
    let ci = b"Failed check: check\nDescription: CI failed\nLink: /actions/runs/2/jobs/1\n[job log could not be read]\nRepair the failed check.\n";
    assert_eq!(out.len(), 2, "gather both required forge sections");
    for _ in 0..out.len() {
        let Some(GatherRequest::Gather { section, budget: 512, .. }) = out.pop() else { panic!("bounded forge read") };
        let kind = domain.brief_connectors.get(Id::from_token(section)).unwrap().kind;
        let size = match kind {
            ForgeBriefKind::Pull => pull.len(),
            ForgeBriefKind::Ci => ci.len(),
            ForgeBriefKind::Home | ForgeBriefKind::Reviews => panic!("pull and CI only"),
        };
        gather_step(
            &mut gather,
            &gather_env,
            GatherEvent::Ready { brief: skein_lib::Token::new(8), section, size: u32::try_from(size).unwrap() },
            &mut out,
        );
    }
    let Some(GatherRequest::Complete { order, .. }) = out.pop() else {
        panic!("the actual task, pull identity and CI sizes fit the unchanged 512-byte brief")
    };
    assert_eq!(order.len(), 3);
    assert!(out.is_empty(), "neither required section needs a cut or a refusal");
}

#[test]
fn a_changes_saved_pull_replaces_its_base_watch_within_two_names() {
    use super::{Decision, Work, forge};
    let (mut domain, mut env) = two_forges();
    env.limits.forge.resources_per_task = 2;
    let forge_env = skein_lib::Env { now: env.now, wall: env.wall, limits: env.limits.forge };
    let mut out = skein_lib::Queue::uncapped();
    forge::step(
        &mut domain.forge,
        &forge_env,
        forge::Event::Restore { record: forge::Stored::Names { task: 7, resources: change_names(None) } },
        &mut out,
    );
    forge::step(
        &mut domain.forge,
        &forge_env,
        forge::Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic },
        &mut out,
    );
    while let Some(request) = out.pop() {
        drop(request);
    }
    let mut decision = Decision::decide(&mut domain.journal, &env.limits.journal);
    for pull in [None, Some(23)] {
        out.push(forge::Request::Save {
            record: forge::Stored::Change(change_row(pull)),
            release: None,
            read_afresh: None,
        });
        super::outputs(&mut domain, &env, &mut decision, &mut out);
        if pull.is_none() {
            assert!(domain.work.is_empty(), "before the pull, keep branch and base");
        }
    }
    let Some(Work::Forge(event @ forge::Event::Rename { .. })) = domain.work.pop() else {
        panic!("name the known pull")
    };
    let forge::Event::Rename { task, resources } = &event else { panic!("pull watch") };
    assert_eq!(*task, 7);
    assert_eq!(*resources, change_names(Some(23)));
    forge::step(&mut domain.forge, &forge_env, event, &mut out);
    assert_eq!(domain.forge.names(7), Some(change_names(Some(23)).as_ref()));
    assert!(domain.work.is_empty(), "the branch's hold needs no change");
}

#[test]
fn a_restored_changes_wake_names_its_known_pull_again() {
    use super::{Decision, Work, forge};
    let (mut domain, mut env) = two_forges();
    env.limits.forge.resources_per_task = 2;
    let forge_env = skein_lib::Env { now: env.now, wall: env.wall, limits: env.limits.forge };
    let mut out = skein_lib::Queue::uncapped();
    // An old store, or a restart between saving the pull and saving its names.
    for record in
        [forge::Stored::Change(change_row(Some(23))), forge::Stored::Names { task: 7, resources: change_names(None) }]
    {
        forge::step(&mut domain.forge, &forge_env, forge::Event::Restore { record }, &mut out);
    }
    forge::step(
        &mut domain.forge,
        &forge_env,
        forge::Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic },
        &mut out,
    );
    while let Some(request) = out.pop() {
        drop(request);
    }
    out.push(forge::Request::WakeProcedure { task: 7 });
    let mut decision = Decision::decide(&mut domain.journal, &env.limits.journal);
    super::outputs(&mut domain, &env, &mut decision, &mut out);
    let Some(Work::Forge(event @ forge::Event::Rename { .. })) = domain.work.pop() else {
        panic!("restore pull watch")
    };
    let forge::Event::Rename { task, resources } = &event else { panic!("pull watch") };
    assert_eq!(*task, 7);
    assert_eq!(*resources, change_names(Some(23)));
    forge::step(&mut domain.forge, &forge_env, event, &mut out);
    let Some(Work::Tasks(super::tasks::Event::WakeProcedure { task: 7 })) = domain.work.pop() else {
        panic!("wake the change's procedure")
    };
    assert!(domain.work.is_empty());
    // Later wakes preserve the established watch's polling cadence.
    for _ in 0..out.len() {
        drop(out.pop());
    }
    out.push(forge::Request::WakeProcedure { task: 7 });
    super::outputs(&mut domain, &env, &mut decision, &mut out);
    let Some(Work::Tasks(super::tasks::Event::WakeProcedure { task: 7 })) = domain.work.pop() else {
        panic!("wake the change's procedure")
    };
    assert!(domain.work.is_empty());
}

/// A change's restored records with its forge state settled and the outputs drained.
fn restored_change(
    records: impl IntoIterator<Item = super::forge::Stored>,
) -> (super::Domain, skein_lib::Env<super::Limits>, skein_lib::Queue<super::forge::Request>) {
    use super::forge;
    let (mut domain, mut env) = two_forges();
    env.limits.forge.resources_per_task = 2;
    let forge_env = skein_lib::Env { now: env.now, wall: env.wall, limits: env.limits.forge };
    let mut out = skein_lib::Queue::uncapped();
    for record in records {
        forge::step(&mut domain.forge, &forge_env, forge::Event::Restore { record }, &mut out);
    }
    forge::step(
        &mut domain.forge,
        &forge_env,
        forge::Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic },
        &mut out,
    );
    while let Some(request) = out.pop() {
        drop(request);
    }
    (domain, env, out)
}

#[test]
fn an_ended_changes_pull_wake_names_nothing() {
    use super::{Decision, Work, forge};
    // Its release unnamed it: the change row stays, its names row does not.
    let (mut domain, env, mut out) = restored_change([forge::Stored::Change(change_row(Some(23)))]);
    out.push(forge::Request::WakeProcedure { task: 7 });
    let mut decision = Decision::decide(&mut domain.journal, &env.limits.journal);
    super::outputs(&mut domain, &env, &mut decision, &mut out);
    let Some(Work::Tasks(super::tasks::Event::WakeProcedure { task: 7 })) = domain.work.pop() else {
        panic!("wake the change's procedure")
    };
    assert!(domain.work.is_empty(), "an ended change watches nothing");
    assert_eq!(domain.forge.names(7), None);
}

#[test]
fn a_change_unnamed_after_its_pull_swap_was_queued_stays_unnamed() {
    use super::{Decision, Work, forge};
    let (mut domain, env, mut out) = restored_change([
        forge::Stored::Change(change_row(Some(23))),
        forge::Stored::Names { task: 7, resources: change_names(None) },
    ]);
    let forge_env = skein_lib::Env { now: env.now, wall: env.wall, limits: env.limits.forge };
    out.push(forge::Request::WakeProcedure { task: 7 });
    let mut decision = Decision::decide(&mut domain.journal, &env.limits.journal);
    super::outputs(&mut domain, &env, &mut decision, &mut out);
    let Some(Work::Forge(event @ forge::Event::Rename { .. })) = domain.work.pop() else { panic!("queue pull watch") };
    // The release completes before the queued swap applies.
    forge::step(&mut domain.forge, &forge_env, forge::Event::Unnamed { task: 7 }, &mut out);
    assert_eq!(domain.forge.names(7), None);
    while let Some(request) = out.pop() {
        drop(request);
    }
    forge::step(&mut domain.forge, &forge_env, event, &mut out);
    assert_eq!(domain.forge.names(7), None, "the swap revives no watch");
    assert!(out.is_empty());
}

#[test]
fn landing_words_name_the_pull_and_use_the_forges_address_when_it_fits() {
    use super::forge::Kind;
    let repository = adopted(1);
    for (kind, git, web, line) in [
        (
            Kind::GitHub,
            b"https://git.example".as_slice(),
            Some(b"https://github.example/".as_slice()),
            b"pull request #7 of org/repo on forge.example: https://github.example/org/repo/pull/7".as_slice(),
        ),
        (
            Kind::Forgejo,
            b"https://forge.example/base".as_slice(),
            None,
            b"pull request #7 of org/repo on forge.example: https://forge.example/base/org/repo/pulls/7".as_slice(),
        ),
        (
            Kind::Forgejo,
            b"https://forge.example/base/".as_slice(),
            None,
            b"pull request #7 of org/repo on forge.example: https://forge.example/base/org/repo/pulls/7".as_slice(),
        ),
        (
            Kind::GitHub,
            b"https://git.example".as_slice(),
            None,
            b"pull request #7 of org/repo on forge.example".as_slice(),
        ),
        (Kind::Forgejo, b"".as_slice(), None, b"pull request #7 of org/repo on forge.example".as_slice()),
    ] {
        let configured = landing_forge(kind, git, web);
        let expected = expected_landing(line);
        assert_eq!(landing_words(&SHA1, 512, Some(&repository), 7, Some(&configured)), expected);
        let bound = u32::try_from(expected.len()).unwrap();
        assert_eq!(landing_words(&SHA1, bound, Some(&repository), 7, Some(&configured)), expected);
        assert_eq!(landing_words(&SHA1, bound - 1, Some(&repository), 7, Some(&configured)).as_ref(), COMMITS[0].1);
    }
    let line = b"pull request #7 of org/repo on forge.example";
    let bound = u32::try_from(COMMITS[0].1.len() + 1 + line.len()).unwrap();
    let expected = expected_landing(line);
    assert_eq!(landing_words(&SHA1, bound, Some(&repository), 7, None), expected);
    assert_eq!(landing_words(&SHA1, bound - 1, Some(&repository), 7, None).as_ref(), COMMITS[0].1);
    assert_eq!(landing_words(&SHA1, 7, Some(&repository), 7, None).as_ref(), b"abcdef0");
    let long = [b'x'; 2048];
    for kind in [Kind::GitHub, Kind::Forgejo] {
        let configured = landing_forge(kind, &long, Some(&long));
        assert_eq!(landing_words(&SHA1, 4096, Some(&repository), 7, Some(&configured)), expected);
    }
}

#[expect(clippy::manual_map, reason = "the strict subset uses explicit matches")]
fn landing_forge(kind: super::forge::Kind, git: &[u8], web: Option<&[u8]>) -> super::forge::Configured {
    super::forge::Configured {
        admin_connect: false,
        bot: Box::new([]),
        owned: None,
        git: git.into(),
        web: match web {
            Some(web) => Some(web.into()),
            None => None,
        },
        read_only: false,
        number: 1,
        kind,
        mode: super::forge::Mode::Connected,
        asked: false,
    }
}

fn expected_landing(line: &[u8]) -> Box<[u8]> {
    let bytes = COMMITS[0].1.len().checked_add(1).unwrap().checked_add(line.len()).unwrap();
    let mut writer = skein_lib::Writer::new(bytes);
    writer.put(COMMITS[0].1).unwrap();
    writer.put(b"\n").unwrap();
    writer.put(line).unwrap();
    writer.finish()
}

#[test]
fn a_review_never_starts_with_a_cut_pull_identity() {
    let row = change_row(Some(42));
    let identity = b"Review this change at the named head\nPull request: #42\nBranch: temper/7\nBase: main\nHead: abcdef0123456789abcdef0123456789abcdef01\n";
    let head = commit_text(&SHA1);
    let parameters = [
        super::tasks::Parameter::Bytes { name: 2, value: row.base.clone() },
        super::tasks::Parameter::Bytes { name: 3, value: row.branch.clone() },
        super::tasks::Parameter::Bytes { name: 8, value: head },
    ];
    let bound = u32::try_from(identity.len() + row.base.len() + row.branch.len() + COMMITS[0].1.len()).unwrap();
    assert_eq!(super::review_words(&row, 1, &SHA1, &parameters, bound).unwrap().as_ref(), identity);
    assert!(super::review_words(&row, 1, &SHA1, &parameters, bound - 1).is_none());
    assert!(super::review_words(&change_row(None), 1, &SHA1, &parameters, bound).is_none());
}

#[test]
fn a_landed_result_names_the_landing_whole_after_its_pull_identity() {
    for (merge, landed) in [
        (SHA1, b"Landed: pull request #7 merged into release/v1 as abcdef012345.".as_slice()),
        (SHA256, b"Landed: pull request #7 merged into release/v1 as abababababab.".as_slice()),
    ] {
        let (mut domain, env) = two_forges();
        let mut row = change_row(Some(7));
        row.base = Box::from(b"release/v1".as_slice());
        row.change.last_head = Some(merge);
        let identity = landing_words(&merge, 512, domain.forge.repository(row.repository), 7, None);
        let evidence = row.result.words(&merge, 512);
        let edge = u32::try_from(identity.len() + 1 + landed.len()).unwrap();
        let mut out = skein_lib::Queue::uncapped();
        super::forge::step(
            &mut domain.forge,
            &super::super::environment_forge(&env),
            super::forge::Event::Restore { record: super::forge::Stored::Change(row) },
            &mut out,
        );
        let mut expected = skein_lib::Writer::new(identity.len() + 1 + landed.len());
        expected.put(&identity).unwrap();
        expected.put(b"\n").unwrap();
        expected.put(landed).unwrap();
        let prefix = expected.finish();
        let mut expected = skein_lib::Writer::new(prefix.len() + 1 + evidence.bytes.len());
        expected.put(&prefix).unwrap();
        expected.put(b"\n").unwrap();
        expected.put(&evidence.bytes).unwrap();
        assert_eq!(super::change_landing_words(&domain, 7, &merge, 7, 512), expected.finish());
        assert_eq!(super::change_landing_words(&domain, 7, &merge, 7, edge), prefix);
        // One byte short omits the whole Landed line and fits evidence as before.
        let bound = edge - 1;
        let fitted = evidence.fitted(bound - u32::try_from(identity.len()).unwrap() - 1).unwrap();
        let mut expected = skein_lib::Writer::new(identity.len() + 1 + fitted.len());
        expected.put(&identity).unwrap();
        expected.put(b"\n").unwrap();
        expected.put(&fitted).unwrap();
        assert_eq!(super::change_landing_words(&domain, 7, &merge, 7, bound), expected.finish());
        // Without room for the pull line, the merge id and fitted evidence stay unchanged.
        let bound = u32::try_from(identity.len() - 1).unwrap();
        let digits = commit_text(&merge);
        let fitted = evidence.fitted(bound - u32::try_from(digits.len()).unwrap() - 1).unwrap();
        let mut expected = skein_lib::Writer::new(digits.len() + 1 + fitted.len());
        expected.put(&digits).unwrap();
        expected.put(b"\n").unwrap();
        expected.put(&fitted).unwrap();
        assert_eq!(super::change_landing_words(&domain, 7, &merge, 7, bound), expected.finish());
        assert_eq!(super::change_landing_words(&domain, 7, &merge, 7, 7).as_ref(), &digits[..7]);
    }
}

#[test]
fn a_small_landed_result_cuts_only_evidence_and_keeps_both_identity_lines() {
    let (mut domain, env) = two_forges();
    let mut row = change_row(Some(7));
    row.change.last_head = Some(SHA1);
    row.result.head = Some(SHA1);
    row.result.checks.bytes = Box::from(b"build passed; release note passed".as_slice());
    row.result.repair_count = 1;
    row.result.repairs.bytes = Box::from(b"CI failed: release note".as_slice());
    row.result.reviews.bytes = Box::from(b"approved at abcdef0123456789abcdef0123456789abcdef01".as_slice());
    let identity = landing_words(
        &SHA1,
        512,
        domain.forge.repository(row.repository),
        7,
        domain.forge.configured(row.repository.forge),
    );
    let mut out = skein_lib::Queue::uncapped();
    super::forge::step(
        &mut domain.forge,
        &super::super::environment_forge(&env),
        super::forge::Event::Restore { record: super::forge::Stored::Change(row) },
        &mut out,
    );
    let whole = super::change_landing_words(&domain, 7, &SHA1, 7, 512);
    let bound = u32::try_from(identity.len() + 90).unwrap();
    let cut = super::change_landing_words(&domain, 7, &SHA1, 7, bound);
    assert!(cut.starts_with(&identity));
    let mut kept = None;
    for (at, pair) in cut.windows(2).enumerate() {
        if pair == b"\n[" {
            kept = Some(at);
        }
    }
    let kept = kept.expect("cut marker");
    assert!(whole.starts_with(&cut[..kept]));
    let lost = skein_lib::Decimal::of(u64::try_from(whole.len() - kept).unwrap());
    assert_eq!(&cut[kept + 2..cut.len() - b" bytes cut]\n".len()], lost.as_bytes());
    assert!(cut.ends_with(b" bytes cut]\n"));
    assert!(cut.len() <= usize::try_from(bound).unwrap());
}

#[test]
fn a_ci_repairs_words_open_with_its_head_and_two_failed_contexts() {
    let mut row = change_row(Some(42));
    row.change.last_head = Some(SHA1);
    row.result.head = Some(SHA1);
    row.result.failed.bytes = Box::from(b"/ check (pull_request), / changes (pull_request)".as_slice());
    let words = super::repair_words(&row, super::forge_change::Repair::Ci, &[], 512).unwrap();
    assert_eq!(words.as_ref(), b"Repair CI at abcdef012345: / check (pull_request), / changes (pull_request)\nRepair the failed CI on the change branch");
    // A client cuts contexts, keeping the complete opening and pinned head.
    row.result.failed.bytes = Box::from([b'x'; 200]);
    let words = super::repair_words(&row, super::forge_change::Repair::Ci, &[], 512).unwrap();
    assert!(words[..128].starts_with(b"Repair CI at abcdef012345: "));
    assert!(words.ends_with(b"\nRepair the failed CI on the change branch"));
    assert!(super::repair_words(&row, super::forge_change::Repair::Ci, &[], 64).is_none());
}

#[test]
fn a_base_break_repairs_words_open_with_its_head_and_base() {
    let mut row = change_row(Some(42));
    row.change.last_head = Some(SHA1);
    let words = super::repair_words(&row, super::forge_change::Repair::Semantic, &[], 512).unwrap();
    assert_eq!(
        words.as_ref(),
        b"Repair the base's break at abcdef012345: main\nRepair the base update's semantic conflict"
    );
}

#[test]
fn a_gate_repairs_words_open_with_its_number_kind_and_head() {
    use super::forge_change::{Freshness, Gate, GateKind, Repair};
    let mut row = change_row(Some(42));
    row.change.last_head = Some(SHA1);
    for (kind, expected) in [
        (
            GateKind::Agent,
            b"Repair gate 7 (review) at abcdef012345: changes requested\nRepair the requested gate changes".as_slice(),
        ),
        (
            GateKind::Person,
            b"Repair gate 7 (approval) at abcdef012345: changes requested\nRepair the requested gate changes",
        ),
        (
            GateKind::Check,
            b"Repair gate 7 (check) at abcdef012345: changes requested\nRepair the requested gate changes",
        ),
    ] {
        row.change.gates =
            Box::new([Gate { number: 7, kind, blocking: true, freshness: Freshness::Exact, eager: false }]);
        let parameters = [super::tasks::Parameter::Bytes { name: 2, value: row.base.clone() }];
        let bound = u32::try_from(expected.len() + row.base.len()).unwrap();
        assert_eq!(super::repair_words(&row, Repair::Gate(7), &parameters, bound).unwrap().as_ref(), expected);
        assert!(super::repair_words(&row, Repair::Gate(7), &parameters, 64).is_none());
    }
    assert!(super::repair_words(&row, Repair::Gate(8), &[], 512).is_none(), "an unknown gate cannot be named");
}

fn procedure_root() -> (super::Domain, skein_lib::Env<super::Limits>) {
    use super::{authority, forge};
    let mut config = crate::tests::resume_fixture::config(7);
    let kinds = Box::new([
        authority::Executor::Charter(1),
        authority::Executor::Procedure(super::LANDING_CODE),
        authority::Executor::Procedure(super::AFTER_LANDING_CODE),
    ]);
    config.core.defaults.rules.maximum_run_spend = 1000;
    config.core.defaults.rules.ceiling.delegation.kinds = kinds.clone();
    config.core.seed_projects[0].policy.policy.ceiling.delegation.kinds = kinds;
    let mut limits = crate::tests::resume_fixture::limits();
    limits.authority.executors = 3;
    limits.tasks.executor_kinds = 3;
    limits.tasks.parameters = 4;
    limits.tasks.spec_bytes = 256;
    let (mut domain, env) = crate::tests::resume_fixture::root_with_config_and_limits(config, &limits);
    for event in [
        jig_domain_core::Event::Tasks(super::tasks::Event::Restored),
        jig_domain_core::Event::Period { project: 1, period: 1, budget: 1000, next: None },
    ] {
        drop(jig_domain_core::step(&mut domain.core, &crate::limits::environment_core(&env), event));
    }
    domain.forge.restore_begin(env.wall);
    let mut repository = adopted(1);
    repository.project = 1;
    let mut out = skein_lib::Queue::uncapped();
    forge::step(
        &mut domain.forge,
        &super::super::environment_forge(&env),
        forge::Event::Restore { record: forge::Stored::Repository(repository) },
        &mut out,
    );
    (domain, env)
}

fn forge_request(domain: &mut super::Domain, env: &skein_lib::Env<super::Limits>, request: super::forge::Request) {
    let mut out = skein_lib::Queue::uncapped();
    out.push(request);
    let mut decision = super::Decision::decide(&mut domain.journal, &env.limits.journal);
    super::outputs(domain, env, &mut decision, &mut out);
    crate::decision::accept_routed(&mut domain.journal, &mut domain.core.counters, &env.limits.journal, decision, true)
        .unwrap();
}

fn make_tier(domain: &mut super::Domain, env: &skein_lib::Env<super::Limits>) -> u64 {
    use super::{Work, forge, tasks};
    let repository = adopted(1).provider;
    forge_request(
        domain,
        env,
        forge::Request::TierRun { repository, tier: Box::from(b"fuzzy".as_slice()), number: 7, tip: SHA1 },
    );
    let Some(Work::Core(event)) = domain.work.pop() else { panic!("make tier task") };
    let jig_domain_core::Event::ConnectorRepair { executor, spec, contract, holdings, .. } = &event else {
        panic!("deployment seed")
    };
    assert_eq!(*executor, tasks::Executor::Procedure { connector: 0, code: super::AFTER_LANDING_CODE });
    assert_eq!(spec.words.as_ref(), b"Tier fuzzy run 7");
    assert_eq!(
        spec.parameters.as_ref(),
        &[
            tasks::Parameter::Resource { name: 1, connector: 0, resource: forge::tag(1, 1) },
            tasks::Parameter::Bytes { name: 8, value: commit_text(&SHA1) },
            tasks::Parameter::Bytes { name: 10, value: Box::from(b"fuzzy".as_slice()) },
            tasks::Parameter::Number { name: 11, value: 7 },
        ]
    );
    assert_eq!(*contract, tasks::Contract::Report { words: env.limits.tasks.result_bytes });
    assert!(holdings.is_empty());
    let Some(Work::Forge(forge::Event::TierTask { task, repository: got, tier, number: 7 })) = domain.work.pop() else {
        panic!("answer tier task")
    };
    assert_eq!(got, repository);
    assert_eq!(tier.as_ref(), b"fuzzy");
    assert!(domain.work.is_empty());
    drop(jig_domain_core::step(&mut domain.core, &crate::limits::environment_core(env), event));
    let row = domain.core.tasks.task(task).expect("tier task admitted");
    assert_eq!(row.numbers.budget, 0);
    assert_eq!(row.requester, tasks::Party::Deployment { project: 1 });
    task
}

#[test]
fn a_tier_task_starts_while_the_period_is_reserved_whole() {
    use super::tasks;
    let (mut domain, env) = procedure_root();
    let funder = tasks::Funder::Period { project: 1, period: domain.core.period(1) };
    reserve_period(&mut domain, &env);
    let ledger = *domain.core.tasks.funding(funder).expect("reserved period");
    assert!(domain.core.repair_seed(1).is_none(), "agent repair still needs unreserved spend");
    make_tier(&mut domain, &env);
    assert_eq!(domain.core.tasks.funding(funder).unwrap().numbers.reserved, ledger.numbers.budget);
}

#[test]
fn a_tier_without_a_project_seed_is_refused() {
    use super::{Work, forge};
    let (mut domain, env) = two_forges(); // The repository names project 5; only project 1 has a policy.
    forge_request(
        &mut domain,
        &env,
        forge::Request::TierRun {
            repository: adopted(1).provider,
            tier: Box::from(b"fuzzy".as_slice()),
            number: 7,
            tip: SHA1,
        },
    );
    let Some(Work::Forge(forge::Event::TierRefused { repository, tier, number: 7 })) = domain.work.pop() else {
        panic!("refuse missing seed")
    };
    assert_eq!(repository, adopted(1).provider);
    assert_eq!(tier.as_ref(), b"fuzzy");
    assert!(domain.work.is_empty());
}

#[test]
fn a_held_tier_keeps_its_words_and_a_release_steps_it_again() {
    use super::{Work, forge, tasks};
    let (mut domain, env) = procedure_root();
    let task = make_tier(&mut domain, &env);
    offer_tier(&mut domain, &env, task);
    let Some(Work::Forge(forge::Event::StepTier { task: got, heard, .. })) = domain.work.pop() else {
        panic!("first tier step")
    };
    assert_eq!(got, task);
    assert!(!heard.released && !heard.cancelled);
    let words = b"Tier fuzzy run 7 held after 3 runner faults, last: dispatch";
    forge_request(
        &mut domain,
        &env,
        forge::Request::TierDecision {
            task,
            decision: forge::TierDecision::Held { words: Box::from(words.as_slice()) },
            entry: None,
        },
    );
    let Some(Work::Core(event)) = domain.work.pop() else { panic!("hold tier") };
    let jig_domain_core::Event::ProcedureStep { decision, .. } = &event else { panic!("held step") };
    assert_eq!(
        decision.end,
        super::StepEnd::HoldFrom {
            why: tasks::Hold::Procedure,
            source: Box::new(tasks::HoldSource::Account {
                connector: 0,
                entry: None,
                words: Box::from(words.as_slice())
            }),
        }
    );
    assert!(decision.messages.is_empty());
    drop(jig_domain_core::step(&mut domain.core, &crate::limits::environment_core(&env), event));
    let tasks::Phase::Held { why, source, .. } = &domain.core.tasks.task(task).unwrap().phase else {
        panic!("held tier")
    };
    assert_eq!(*why, tasks::Hold::Procedure);
    assert_eq!(
        source.as_deref(),
        Some(&tasks::HoldSource::Account { connector: 0, entry: None, words: Box::from(words.as_slice()) })
    );
    let release = jig_domain_core::step(
        &mut domain.core,
        &crate::limits::environment_core(&env),
        jig_domain_core::Event::Tasks(tasks::Event::Control {
            reply_to: skein_lib::ReplyTo::new(skein_lib::Token::new(123)),
            by: tasks::Party::Person(1),
            task,
            control: tasks::Control::Release { again: None, message: None },
        }),
    );
    assert_eq!(domain.core.tasks.task(task).unwrap().due_reason, tasks::Why::Released, "{release:?}");
    offer_tier(&mut domain, &env, task);
    let Some(Work::Forge(forge::Event::StepTier { task: got, heard, .. })) = domain.work.pop() else {
        panic!("released tier step")
    };
    assert_eq!(got, task);
    assert!(heard.released && !heard.cancelled);
}

#[test]
fn a_done_tier_ends_with_the_verdicts_words() {
    use super::{Work, forge, tasks};
    let (mut domain, env) = procedure_root();
    let task = make_tier(&mut domain, &env);
    let words = b"Tier fuzzy run 7: green, cleared 2 landings";
    forge_request(
        &mut domain,
        &env,
        forge::Request::TierDecision {
            task,
            decision: forge::TierDecision::Done { words: Box::from(words.as_slice()) },
            entry: None,
        },
    );
    let Some(Work::Core(event)) = domain.work.pop() else { panic!("finish tier") };
    let jig_domain_core::Event::ProcedureStep { decision, .. } = &event else { panic!("done step") };
    assert_eq!(decision.end, super::StepEnd::Result(tasks::TaskResult::Report { words: Box::from(words.as_slice()) }));
    drop(jig_domain_core::step(&mut domain.core, &crate::limits::environment_core(&env), event));
    let tasks::Phase::Closing(closing) = &domain.core.tasks.task(task).unwrap().phase else { panic!("closing tier") };
    assert_eq!(closing.ending, tasks::Ending::Done(tasks::TaskResult::Report { words: Box::from(words.as_slice()) }));
}

fn reserve_period(domain: &mut super::Domain, env: &skein_lib::Env<super::Limits>) {
    use super::tasks;
    let seed = domain.core.repair_seed(1).expect("unreserved period can fund an agent");
    let task = seed.number();
    drop(jig_domain_core::step(
        &mut domain.core,
        &crate::limits::environment_core(env),
        jig_domain_core::Event::ConnectorRepair {
            seed: Box::new(seed),
            executor: tasks::Executor::Agent { charter: 1 },
            spec: tasks::Spec {
                job: None,
                asks: false,
                words: Box::from(b"reserve the period".as_slice()),
                parameters: Box::new([]),
                inputs: Box::new([]),
            },
            contract: tasks::Contract::Report { words: env.limits.tasks.result_bytes },
            holdings: Box::new([]),
        },
    ));
    assert_eq!(domain.core.tasks.task(task).unwrap().numbers.budget, 1000);
    let funder = tasks::Funder::Period { project: 1, period: domain.core.period(1) };
    let ledger = domain.core.tasks.funding(funder).unwrap();
    assert_eq!(ledger.numbers.reserved, ledger.numbers.budget);
}

#[test]
fn a_landing_by_turn_starts_while_the_period_is_reserved_whole() {
    use super::{Work, forge, forge_change, tasks};
    let (mut domain, env) = procedure_root();
    reserve_period(&mut domain, &env);
    let mut row = change_row(Some(7));
    row.repository = adopted(1).provider;
    row.change.way = forge_change::Way::Turn;
    row.change.state = forge_change::State::Ready { head: SHA1, ready_since: env.wall };
    let mut out = skein_lib::Queue::uncapped();
    forge::step(
        &mut domain.forge,
        &super::super::environment_forge(&env),
        forge::Event::Restore { record: forge::Stored::Change(row) },
        &mut out,
    );
    assert!(domain.forge.project_landing(
        1,
        forge_change::Way::Turn,
        Box::new([]),
        forge::AfterLanding {
            verdict: temper_engine_domain_forge_tiers::Settings {
                retries: 2,
                suspects: temper_engine_domain_forge_tiers::Suspects::Bisect
            },
            bound: skein_lib::Duration::from_secs(7200),
        }
    ));
    super::start_landing(&mut domain, &env, 7);
    let Some(Work::Forge(forge::Event::Landing { row })) = domain.work.pop() else { panic!("landing row") };
    let task = row.landing.task;
    assert_eq!(row.landing.members[0].change, 7);
    let Some(Work::Core(event)) = domain.work.pop() else { panic!("landing task") };
    drop(jig_domain_core::step(&mut domain.core, &crate::limits::environment_core(&env), event));
    let row = domain.core.tasks.task(task).expect("zero-budget landing admitted");
    assert_eq!(row.executor, tasks::Executor::Procedure { connector: 0, code: super::LANDING_CODE });
    assert_eq!(row.numbers.budget, 0);
    assert_eq!(row.spec.words.as_ref(), b"Land Change");
}

#[expect(clippy::manual_map, reason = "the strict subset uses explicit matches")]
fn result_tier(name: &[u8], branch: Option<&[u8]>) -> crate::translate::TierCheck {
    crate::translate::TierCheck {
        name: name.into(),
        workflow: Box::from(b"checks.yml".as_slice()),
        ignored: Box::new([]),
        artifact: Box::from(b"report".as_slice()),
        format: temper_engine_domain_forge_tiers::Format::Lines,
        branch: match branch {
            Some(branch) => Some(Box::from(branch)),
            None => None,
        },
    }
}

#[test]
fn cleared_and_uncleared_results_qualify_the_landing_before_its_evidence() {
    use super::{forge, forge_change};
    for (state, line) in [
        (forge_change::State::Cleared { merge: SHA1 }, b"Cleared by fuzzy, browser.".as_slice()),
        (
            forge_change::State::Uncleared { merge: SHA1 },
            b"Not cleared: no verdict within the verification bound.".as_slice(),
        ),
    ] {
        let (mut domain, env) = two_forges();
        let mut row = change_row(Some(7));
        row.change.state = state;
        row.change.last_head = Some(SHA1);
        domain
            .config
            .landings
            .insert(
                5,
                crate::translate::ProjectLanding {
                    by: crate::translate::Way::Queue,
                    repositories: Box::new([crate::translate::RepositoryChecks {
                        repository: row.repository,
                        gates: Box::new([]),
                        tiers: Box::new([
                            result_tier(b"fuzzy", None),
                            result_tier(b"release", Some(b"release/v1")),
                            result_tier(b"browser", Some(b"main")),
                        ]),
                    }]),
                    after: crate::translate::AfterLandingCheck { tier_retries: 2, verification_bound_ms: 7_200_000 },
                },
            )
            .unwrap();
        let mut out = skein_lib::Queue::uncapped();
        forge::step(
            &mut domain.forge,
            &super::super::environment_forge(&env),
            forge::Event::Restore { record: forge::Stored::Change(row) },
            &mut out,
        );
        let words = super::change_landing_words(&domain, 7, &SHA1, 7, 512);
        let expected = b"\nLanded: pull request #7 merged into main as abcdef012345.\n";
        let at = skein_lib::bytes::find(&words, expected).expect("landing identity");
        assert_eq!(&words[at + expected.len()..at + expected.len() + line.len()], line);
        assert_eq!(words[at + expected.len() + line.len()], b'\n', "evidence follows verification");
        let bound = u32::try_from(at + expected.len() + line.len()).unwrap();
        let exact = super::change_landing_words(&domain, 7, &SHA1, 7, bound);
        assert!(exact.ends_with(line));
        let short = super::change_landing_words(&domain, 7, &SHA1, 7, bound - 1);
        for part in short.windows(line.len() - 1) {
            assert_ne!(part, &line[..line.len() - 1], "qualification is kept whole");
        }
    }
}

#[test]
fn an_uncleared_cancelled_change_names_cancellation() {
    use super::{forge, forge_change, tasks};
    let (mut domain, env, task) =
        crate::tests::live_task(None, tasks::Executor::Procedure { connector: 0, code: super::CHANGE_CODE });
    let mut row = change_row(Some(7));
    row.task = task;
    row.change.task = task;
    row.change.state = forge_change::State::Uncleared { merge: SHA1 };
    domain.forge.restore_begin(env.wall);
    let mut out = skein_lib::Queue::uncapped();
    forge::step(
        &mut domain.forge,
        &super::super::environment_forge(&env),
        forge::Event::Restore { record: forge::Stored::Change(row) },
        &mut out,
    );
    drop(jig_domain_core::step(
        &mut domain.core,
        &crate::limits::environment_core(&env),
        jig_domain_core::Event::Tasks(tasks::Event::Control {
            reply_to: skein_lib::ReplyTo::new(skein_lib::Token::new(123)),
            by: tasks::Party::Deployment { project: 1 },
            task,
            control: tasks::Control::Cancel { reason: Box::new([]), grace: None, message: None },
        }),
    ));
    assert_eq!(
        super::change_verification_words(&domain, task, &SHA1, 128).as_ref(),
        b"Not cleared: cancelled while verifying."
    );
}

fn offer_tier(domain: &mut super::Domain, env: &skein_lib::Env<super::Limits>, task: u64) {
    let context = domain.core.tasks.run_context(task).unwrap();
    let (_, _, step) = domain.core.tasks.procedure_due(task).unwrap();
    let mut decision = super::Decision::decide(&mut domain.journal, &env.limits.journal);
    super::super::asks::ask(
        domain,
        env,
        &mut decision,
        domain.config.forge_connector,
        jig_domain_core::Ask::StartProcedure { context, step },
    );
    crate::decision::accept_routed(&mut domain.journal, &mut domain.core.counters, &env.limits.journal, decision, true)
        .unwrap();
}

#[test]
fn a_tier_without_a_task_number_is_refused() {
    use super::{Work, forge};
    let (mut domain, env) = procedure_root();
    let mut deployment = domain.core.counters.deployment();
    deployment.tasks = u64::MAX;
    domain.core.counters = crate::Counters::new(deployment);
    forge_request(
        &mut domain,
        &env,
        forge::Request::TierRun {
            repository: adopted(1).provider,
            tier: Box::from(b"fuzzy".as_slice()),
            number: 7,
            tip: SHA1,
        },
    );
    let Some(Work::Forge(forge::Event::TierRefused { number: 7, .. })) = domain.work.pop() else {
        panic!("exhausted task number refuses tier")
    };
    assert!(domain.work.is_empty());
}

#[test]
fn a_waiting_tier_ends_its_step_and_is_stepped_on_a_forge_wake() {
    use super::{Work, forge, tasks};
    let (mut domain, env) = procedure_root();
    let task = make_tier(&mut domain, &env);
    forge_request(
        &mut domain,
        &env,
        forge::Request::TierDecision {
            task,
            decision: forge::TierDecision::Wait { until: Some(skein_lib::Wall::from_nanos(100)) },
            entry: None,
        },
    );
    let Some(Work::Core(event)) = domain.work.pop() else { panic!("wait tier step") };
    let jig_domain_core::Event::ProcedureStep { decision, .. } = &event else { panic!("wait step") };
    assert_eq!(decision.end, super::StepEnd::Wait);
    drop(jig_domain_core::step(&mut domain.core, &crate::limits::environment_core(&env), event));
    assert!(domain.core.tasks.procedure_due(task).is_none());
    forge_request(&mut domain, &env, forge::Request::WakeProcedure { task });
    let Some(Work::Tasks(event @ tasks::Event::WakeProcedure { .. })) = domain.work.pop() else { panic!("wake tier") };
    drop(jig_domain_core::step(
        &mut domain.core,
        &crate::limits::environment_core(&env),
        jig_domain_core::Event::Tasks(event),
    ));
    offer_tier(&mut domain, &env, task);
    let Some(Work::Forge(forge::Event::StepTier {
        task: got,
        heard: forge::TierHeard { released: false, cancelled: false },
        ..
    })) = domain.work.pop()
    else {
        panic!("step woken tier")
    };
    assert_eq!(got, task);
    assert!(domain.work.is_empty());
}

#[test]
fn a_tier_effect_without_a_description_is_vetoed_before_it_can_run() {
    use super::{Work, forge, tasks};
    let (mut domain, env) = procedure_root();
    let task = make_tier(&mut domain, &env);
    forge_request(
        &mut domain,
        &env,
        forge::Request::TierDecision { task, decision: forge::TierDecision::Wait { until: None }, entry: Some(8) },
    );
    let Some(Work::Forge(forge::Event::VetoTier { task: got, entry: 8 })) = domain.work.pop() else {
        panic!("veto tier entry")
    };
    assert_eq!(got, task);
    let Some(Work::Tasks(tasks::Event::Hold { task: got, why: tasks::Hold::Effects })) = domain.work.pop() else {
        panic!("hold refused effect")
    };
    assert_eq!(got, task);
    assert!(domain.forge_effects.is_empty());
    assert!(domain.work.is_empty());
}
