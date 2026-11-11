#![expect(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    reason = "ordinary tests construct and inspect bounded approval people"
)]
use super::*;
use alloc::vec::Vec;
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

pub(crate) fn approval(role: u32, required: u32, counted: bool) -> forge::ShortApproval {
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

pub(crate) fn wait(short: Box<[forge::ShortApproval]>) -> forge::ApprovalWait {
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
        let mut bytes = size_of::<ApprovalRow>() + 13 + 2 * size_of::<ApprovalPerson>();
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
            + 6 * (size_of::<ApprovalSet>() + 40)
            + 16 * (size_of::<ApprovalPerson>() + 64),
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
            row.signed = (1..=8).map(|person| ApprovalPerson { person, name: Box::new([b's'; 64]) }).collect();
            row.can_sign = (9..=16).map(|person| ApprovalPerson { person, name: Box::new([b'c'; 64]) }).collect();
            let bytes = Approvals { rows: Box::new([row.clone()]), more: 0 }.owned_bytes();
            assert_eq!(bytes, size_of::<ApprovalRow>() + 13 + 16 * (size_of::<ApprovalPerson>() + 64));
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

pub(crate) fn members_fixture(members: &[(u64, Role, bool, &str, &str)]) -> (Domain, Env<Limits>) {
    let (mut domain, mut env) = crate::tests::resume_fixture::root();
    env.limits.people.people = 32;
    env.limits.people.holdings = 32;
    env.limits.people.identity_bytes = 128;
    domain.core.people = people::Domain::new(&env.limits.people, Box::new([]), 1, Box::new([]));
    domain.core.people.start_restore(env.wall);
    let mut holdings = List::uncapped();
    let mut out = Queue::uncapped();
    for &(person, role, account, name, login) in members {
        holdings.push(people::Holding { person, role }).unwrap();
        people::step(
            &mut domain.core.people,
            &env.with(env.limits.people),
            people::Event::Restore {
                record: people::Stored::Person {
                    number: person,
                    identity: people::Identity {
                        key: forge::user_key(if account { 1 } else { 3 }, person.saturating_add(10)),
                        name: name.as_bytes().into(),
                        login: login.as_bytes().into(),
                    },
                },
            },
            &mut out,
        );
    }
    people::step(
        &mut domain.core.people,
        &env.with(env.limits.people),
        people::Event::Restore { record: people::Stored::Roles { project: 1, holdings: holdings.into_boxed() } },
        &mut out,
    );
    assert!(domain.core.people.start_link());
    domain.core.people.start_sweep();
    assert!(domain.core.people.start_complete(&env.with(env.limits.people)));
    (domain, env)
}

pub(crate) fn counted(people: &[u64], role: u32) -> Box<[forge::Reviewer]> {
    people
        .iter()
        .map(|&person| forge::Reviewer { person, role, head: [1; 32], verdict: skein_forge::api::Verdict::Approve })
        .collect()
}

#[test]
fn a_partial_approval_names_signers_and_only_eligible_people_with_the_watcher_first() {
    let (domain, _) = members_fixture(&[
        (1, Role::Member, true, "Sam", "sam"),
        (2, Role::Member, true, "Kit", "kit"),
        (3, Role::Member, true, "Ann", "ann"),
        (4, Role::Member, false, "Lee", "lee"),
        (5, Role::Maintainer, true, "Pat", "pat"),
        (6, Role::Owner, true, "Owner", "owner"),
    ]);
    let mut approval = approval(2, 2, false);
    approval.counted = counted(&[1], 2);
    let wait = wait(Box::new([approval]));
    let row = row(&domain, &wait, 3).unwrap();
    assert_eq!(row.signed.as_ref(), &[ApprovalPerson { person: 1, name: b"Sam".as_slice().into() }]);
    assert_eq!(
        row.can_sign.as_ref(),
        &[
            ApprovalPerson { person: 3, name: b"Ann".as_slice().into() },
            ApprovalPerson { person: 2, name: b"Kit".as_slice().into() },
        ]
    );
    assert_eq!((row.can_sign_more, row.yours), (0, YourReview::Counts));
    assert!(super::row(&domain, &wait, 1).is_none(), "already counted watcher has no row");
    assert!(super::row(&domain, &wait, 5).is_none(), "another role has no row");
    let owner = super::row(&domain, &wait, 6).unwrap();
    assert_eq!(owner.yours, YourReview::OtherRole);
    assert!(!owner.can_sign.iter().any(|person| person.person == 6));
}

#[test]
fn approval_people_are_bounded_named_by_login_and_cut_on_a_character_boundary() {
    let long = "é".repeat(35);
    let members: Vec<_> = (1..=12).map(|person| (person, Role::Member, true, long.as_str(), "login")).collect();
    let (domain, _) = members_fixture(&members);
    let mut short = approval(2, 12, false);
    short.counted = counted(&(1..=10).collect::<Vec<_>>(), 2);
    let row = row(&domain, &wait(Box::new([short])), 12).unwrap();
    assert_eq!((row.signed.len(), row.counted), (8, 10));
    assert_eq!(row.signed.iter().map(|p| p.person).collect::<Vec<_>>(), (1..=8).collect::<Vec<_>>());
    for person in row.signed.iter().chain(row.can_sign.iter()) {
        assert_eq!(person.name.len(), 64);
        assert_eq!(core::str::from_utf8(&person.name).unwrap(), "é".repeat(32));
    }
    let row = super::row(&domain, &wait(Box::new([approval(2, 13, false)])), 12).unwrap();
    assert_eq!(row.yours, YourReview::TooFew);
    assert_eq!(row.can_sign.iter().map(|p| p.person).collect::<Vec<_>>(), [12, 1, 2, 3, 4, 5, 6, 7]);
    assert_eq!(row.can_sign_more, 4);
    let (domain, _) = members_fixture(&[(1, Role::Member, true, "", "login")]);
    assert_eq!(person(&domain, 1).name.as_ref(), b"login");
    assert!(person(&domain, 99).name.is_empty());
}

pub(crate) fn row_for(domain: &Domain, wait: &forge::ApprovalWait, watcher: u64) -> Option<ApprovalRow> {
    row(domain, wait, watcher)
}
