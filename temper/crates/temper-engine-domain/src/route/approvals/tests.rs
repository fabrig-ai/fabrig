use super::*;
use jig_domain_people as people;
use skein_lib::Queue;

fn fixture(role: Option<Role>, known: bool) -> (Domain, Env<Limits>) {
    let (mut domain, env) = crate::tests::resume_fixture::root();
    domain.core.people = people::Domain::new(&env.limits.people, Box::new([]), 1, Box::new([]));
    domain.core.people.start_restore(env.wall);
    let mut holdings = List::with_capacity(2);
    if let Some(role) = role {
        holdings.push(people::Holding { person: 1, role }).unwrap();
    }
    // A second maintainer keeps a one-person approval possible even when the watcher has no account.
    holdings.push(people::Holding { person: 2, role: Role::Maintainer }).unwrap();
    for record in [
        people::Stored::Person {
            number: 1,
            identity: people::Identity {
                key: forge::user_key(if known { 1 } else { 3 }, 7),
                login: Box::new([]),
                name: Box::new([]),
            },
        },
        people::Stored::Person {
            number: 2,
            identity: people::Identity { key: forge::user_key(1, 8), login: Box::new([]), name: Box::new([]) },
        },
        people::Stored::Roles { project: 1, holdings: holdings.into_boxed() },
    ] {
        people::step(
            &mut domain.core.people,
            &env.with(env.limits.people),
            people::Event::Restore { record },
            &mut Queue::uncapped(),
        );
    }
    assert!(domain.core.people.start_link());
    domain.core.people.start_sweep();
    assert!(domain.core.people.start_complete(&env.with(env.limits.people)));
    (domain, env)
}

fn approval(role: u32, required: u32, counted: bool) -> forge::ShortApproval {
    forge::ShortApproval {
        scope: 1,
        rule: 9,
        role,
        required,
        sets: 0,
        counted: if counted {
            Box::new([forge::Reviewer { person: 1, role, head: [1; 32], verdict: skein_forge::api::Verdict::Approve }])
        } else {
            Box::new([])
        },
    }
}

fn wait(short: Box<[forge::ShortApproval]>) -> forge::ApprovalWait {
    forge::ApprovalWait {
        task: 99,
        project: 1,
        forge: 1,
        owner: b"org".as_slice().into(),
        name: b"repo".as_slice().into(),
        pull: 4,
        title: b"Change".as_slice().into(),
        since: Wall::from_nanos(6),
        whole: true,
        size: None,
        short,
    }
}

#[test]
fn a_watcher_sees_the_waits_of_their_role_and_whether_their_review_counts() {
    use Role::{Maintainer, Member, Owner};
    use YourReview::{Counts, NoAccount, OtherRole, TooFew};
    for (role, known, short, expected) in [
        (Some(Maintainer), true, Box::<[forge::ShortApproval]>::from([approval(1, 1, false)]), Some((1, Counts))),
        (Some(Maintainer), false, Box::new([approval(1, 1, false)]), Some((1, NoAccount))),
        (Some(Maintainer), false, Box::new([approval(1, 2, false)]), Some((1, TooFew))),
        (Some(Owner), true, Box::new([approval(1, 1, false)]), Some((1, OtherRole))),
        (Some(Owner), true, Box::new([approval(1, 1, false), approval(0, 1, false)]), Some((0, Counts))),
        (Some(Member), true, Box::new([approval(1, 1, false)]), None),
        (None, true, Box::new([approval(1, 1, false)]), None),
        (Some(Maintainer), true, Box::new([approval(1, 2, true)]), None),
    ] {
        let (domain, _) = fixture(role, known);
        let row = row(&domain, &wait(short), 1);
        let actual = match &row {
            Some(row) => Some((row.role, row.yours)),
            None => None,
        };
        assert_eq!(actual, expected, "role {role:?}, known {known}");
        if let Some(row) = row {
            assert_eq!(row.goal, None);
        }
    }
    for tracked in [None, Some(1)] {
        let (domain, env, task) =
            crate::tests::live_task(tracked, jig_domain_tasks::Executor::Procedure { connector: 0, code: 1 });
        let parent = domain.core.tasks.task(task).unwrap().clone();
        let mut child = parent.clone();
        child.number = task + 1;
        child.depth = 1;
        child.requester = jig_domain_tasks::Party::Task(task);
        child.tracked = None;
        let funding = *domain.core.tasks.funding(parent.funder).unwrap();
        let child_task = child.number;
        let records = [
            crate::Record::Core(jig_domain_core::Record::Tasks(jig_domain_tasks::Stored::Ledger(funding))),
            crate::Record::Core(jig_domain_core::Record::Tasks(jig_domain_tasks::Stored::Live(Box::new(parent)))),
            crate::Record::Core(jig_domain_core::Record::Tasks(jig_domain_tasks::Stored::Live(Box::new(child)))),
        ];
        let (mut domain, _) = crate::tests::resume_fixture::root_with_records(
            crate::tests::resume_fixture::config(7),
            &env.limits,
            &records,
        );
        let (people, _) = fixture(Some(Maintainer), true);
        domain.core.people = people.core.people;
        let mut wait = wait(Box::new([approval(1, 1, false)]));
        wait.task = child_task;
        let expected = match tracked {
            Some(_) => Some(task),
            None => None,
        };
        assert_eq!(row(&domain, &wait, 1).unwrap().goal, expected);
    }
}

#[test]
fn approval_sets_take_the_projects_names_in_number_order() {
    use temper_protocol_reaches::{Classes, PathSet};
    let (mut domain, env) = fixture(Some(Role::Maintainer), true);
    let classes = Classes {
        items: Box::new([]),
        sets: Box::new([
            PathSet { number: 3, name: b"C".as_slice().into(), patterns: Box::new([b"c/**".as_slice().into()]) },
            PathSet {
                number: 1,
                name: b"vsr's protocol core".as_slice().into(),
                patterns: Box::new([b"src/**".as_slice().into()]),
            },
        ]),
    };
    domain.forge.keep_settings(
        &env.with(env.limits.forge),
        1,
        None,
        &[settings::Asked {
            kind: 8,
            scope: 1,
            number: 1,
            edit: settings::Edit::Set(settings::SizedValue { value: settings::Value::Classes(classes), size: 128 }),
        }],
        &mut Queue::uncapped(),
    );
    for (sets, expected) in [
        (1, [(1, "vsr's protocol core")].as_slice()),
        (0, [].as_slice()),
        (0b1101, [(1, "vsr's protocol core"), (3, "C"), (4, "")].as_slice()),
    ] {
        let mut approval = approval(1, 1, false);
        approval.sets = sets;
        let row = row(&domain, &wait(Box::new([approval])), 1).unwrap();
        assert_eq!(row.sets.len(), expected.len());
        let mut bytes = size_of::<ApprovalRow>() + 13;
        for (set, (number, name)) in row.sets.iter().zip(expected) {
            assert_eq!((set.number, set.name.as_ref()), (*number, name.as_bytes()));
            bytes += size_of::<ApprovalSet>() + name.len();
        }
        let document = Approvals { rows: Box::new([row]), more: 0 };
        assert_eq!(document.owned_bytes(), bytes);
    }
}

#[test]
fn approvals_fit_one_delivery_and_count_the_rows_past_it() {
    let (domain, mut env) = fixture(Some(Role::Maintainer), true);
    let size = u32::try_from(
        size_of::<ApprovalRow>()
            + 2 * usize::try_from(env.limits.forge.name_bytes).unwrap()
            + forge::LOGGED_TITLE_BYTES
            + 6 * (size_of::<ApprovalSet>() + 40),
    )
    .unwrap();
    env.limits.journal.result_bytes = 0;
    for (bytes, bound) in [(size - 1, 0), (size, 1), (size * 2 - 1, 1), (size * 2, 2), (size * 40, 32)] {
        env.limits.journal.transcript_bytes = bytes;
        assert_eq!(crate::room::approval_rows(&env.limits), bound);
        let mut rows = List::with_capacity(35);
        for task in 0..35 {
            let mut row = row(&domain, &wait(Box::new([approval(1, 1, false)])), 1).unwrap();
            row.task = task;
            rows.push(row).unwrap();
        }
        let document = cut(rows.into_boxed(), bound);
        assert_eq!(document.rows.len(), usize::try_from(bound).unwrap());
        assert_eq!(document.more, 35 - bound);
        for (row, task) in document.rows.iter().zip(0..u64::from(bound)) {
            assert_eq!(row.task, task);
        }
        assert!(document.owned_bytes() <= usize::try_from(bytes).unwrap());
    }
    assert!(document(&domain, &env, 1).rows.is_empty());
}

#[test]
fn approval_rows_narrow_each_size_count_and_mark_any_cut() {
    use skein_forge::api::Size;
    let (domain, _) = fixture(Some(Role::Maintainer), true);
    let max = u64::from(u32::MAX);
    for (size, expected) in [
        (None, None),
        (
            Some(Size { files: 4, additions: 142, deletions: 18 }),
            Some(ApprovalSize { files: 4, additions: 142, deletions: 18, cut: false }),
        ),
        (
            Some(Size { files: max, additions: max, deletions: max }),
            Some(ApprovalSize { files: u32::MAX, additions: u32::MAX, deletions: u32::MAX, cut: false }),
        ),
        (
            Some(Size { files: 5_000_000_000, additions: 142, deletions: 18 }),
            Some(ApprovalSize { files: u32::MAX, additions: 142, deletions: 18, cut: true }),
        ),
        (
            Some(Size { files: 4, additions: max + 1, deletions: 18 }),
            Some(ApprovalSize { files: 4, additions: u32::MAX, deletions: 18, cut: true }),
        ),
        (
            Some(Size { files: 4, additions: 142, deletions: max + 1 }),
            Some(ApprovalSize { files: 4, additions: 142, deletions: u32::MAX, cut: true }),
        ),
        (
            Some(Size { files: max + 1, additions: u64::MAX, deletions: u64::MAX }),
            Some(ApprovalSize { files: u32::MAX, additions: u32::MAX, deletions: u32::MAX, cut: true }),
        ),
    ] {
        let mut wait = wait(Box::new([approval(1, 1, false)]));
        wait.size = size;
        assert_eq!(row(&domain, &wait, 1).unwrap().size, expected, "{size:?}");
    }
}
